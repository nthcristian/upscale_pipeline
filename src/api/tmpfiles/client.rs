use futures_util::StreamExt;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

const BASE_URL: &'static str = "https://tmpfiles.org/api/v1/upload";

pub struct TmpFilesClient {
    client: reqwest::Client,
}

impl TmpFilesClient {
    pub async fn upload(&self, file_path: String) -> anyhow::Result<String> {
        let form = reqwest::multipart::Form::new()
            .file("file", file_path)
            .await?;

        let response = self.client.post(BASE_URL).multipart(form).send().await?;
        let text = response.text().await?;

        let data: ApiResponse = serde_json::from_str(&text)?;

        let status = data
            .status
            .clone()
            .ok_or(anyhow::anyhow!("Invalid response\n{:?}", data))?;

        match status.as_str() {
            "success" => {
                let internal_data = data
                    .data
                    .clone()
                    .ok_or(anyhow::anyhow!("Invalid response\n{:?}", data))?;

                return Ok(internal_data.url);
            }
            _ => {
                anyhow::bail!("Request failed:\n{:?}", data);
            }
        }
    }

    pub async fn get_download_link(&self, url: String) -> anyhow::Result<String> {
        let raw_html = self.client.get(url).send().await?.text().await?;

        let download_url = raw_html
            .split("\n")
            .find(|it| it.contains("class=\"download\""))
            .ok_or(anyhow::anyhow!("Could't find download link"))?
            .split("\"")
            .nth(3 as usize)
            .ok_or(anyhow::anyhow!(
                "Found \"download class\" but could't find download link"
            ))?;

        Ok(String::from(download_url.trim()))
    }

    pub async fn download_from(&self, url: String) -> anyhow::Result<()> {
        let mut stream = self
            .client
            .get(url)
            .header(ACCEPT, "image/jpeg")
            .send()
            .await?
            .bytes_stream();

        let mut file = tokio::fs::File::create("output.jpg").await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    pub status: Option<String>,
    pub data: Option<ApiResponseDataField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ApiResponseDataField {
    pub url: String,
}
