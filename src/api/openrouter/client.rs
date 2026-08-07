use std::{env, time::Duration};

use dotenv::dotenv;
use futures_util::StreamExt;
use reqwest::{
    self,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tokio::{fs::File, io::AsyncWriteExt, time::sleep};

use crate::api::{
    DownloadResponse, Job, JobRespose, PollStatus, PollingResponse, ResponseError, VideoRequest,
};

const BASE_VIDEO_URL: &str = "https://openrouter.ai/api/v1/videos";

fn load_key() -> anyhow::Result<String> {
    dotenv()?;

    let key = env::var("OPENROUTER_API_KEY")?;

    Ok(key)
}

fn handle_error(_err: &Option<ResponseError>) -> anyhow::Result<()> {
    if let Some(err) = _err {
        match err {
            ResponseError::Complex { code, message } => {
                anyhow::bail!("code {}: {}", code, message)
            }
            ResponseError::Simple(message) => anyhow::bail!("{}", message),
        }
    }

    Ok(())
}

pub struct OpenRouterClient {
    client: reqwest::Client,
}

impl OpenRouterClient {
    pub async fn create_job(&self, payload: &VideoRequest) -> anyhow::Result<Job> {
        let result = self
            .client
            .post(BASE_VIDEO_URL)
            .body(serde_json::to_string(&payload)?)
            .send()
            .await?;

        let text = &result.text().await?;
        let data: JobRespose = serde_json::from_str(&text)?;

        handle_error(&data.error)?;

        let (id, polling_url) = match (&data.id, &data.polling_url) {
            (Some(id), Some(polling_url)) => (id.clone(), polling_url.clone()),
            _ => anyhow::bail!("JobResponse didn´t come with needed fields\n{:?}", data),
        };

        let job = Job { id, polling_url };

        Ok(job)
    }

    pub async fn poll_until_done(&self, job: &Job) -> anyhow::Result<Vec<String>> {
        loop {
            let result = self.client.get(&job.polling_url).send().await?;

            let text = result.text().await?;
            let data: PollingResponse = serde_json::from_str(&text)?;

            handle_error(&data.error)?;

            match data.status {
                Some(PollStatus::Completed) => {
                    let urls = data.unsigned_urls.clone().ok_or(anyhow::anyhow!(
                        "No unsigned urls came within PollingResponse\n{:?}",
                        &data
                    ))?;

                    return Ok(urls);
                }
                Some(PollStatus::Failed) => {
                    anyhow::bail!("Job {:?} has failed\n{:?}", data.id, data)
                }
                Some(PollStatus::Cancelled) => {
                    anyhow::bail!("Job {:?} has been cancelled\n{:?}", data.id, &data)
                }
                Some(PollStatus::Expired) => {
                    anyhow::bail!("Job {:?} has expired\n{:?}", data.id, &data)
                }
                _ => (),
            }

            sleep(Duration::from_secs(30)).await;
        }
    }

    pub async fn download_from(&self, url: &String, output: &String) -> anyhow::Result<()> {
        let result = self
            .client
            .get(url)
            .header(CONTENT_TYPE, "video/mp4")
            .send()
            .await?;

        if !result.status().is_success() {
            let text = result.text().await?;
            let data: DownloadResponse = serde_json::from_str(&text)?;

            handle_error(&data.error)?;
            anyhow::bail!("Response didn't return success");
        }

        let mut stream = result.bytes_stream();
        let mut file = File::create(output).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        Ok(())
    }

    pub fn new() -> anyhow::Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(AUTHORIZATION, format!("Bearer {}", load_key()?).parse()?);
        headers.append(CONTENT_TYPE, "application/json".parse()?);

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }
}
