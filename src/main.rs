use crate::api::{AspectRatio, Resolution, VideoRequest};

mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = api::OpenRouterClient::new()?;

    let job = client
        .create_job(&VideoRequest {
            model: String::from("bytedance/seedance-1-5-pro"),
            prompt: String::from("The mountain stands proud, it reaches above the clouds as they almost hide its peak, turning it into a mystic place and a great objective"),
            duration: Some(4),
            resolution: Some(Resolution::SD),
            aspect_ratio: Some(AspectRatio::Square),
            input_references: None,
            size: None
        })
        .await?;

    println!("Created new video gen job..");

    let download_urls = client.poll_until_done(&job).await?;

    println!(
        "Received download links.. {} in total..",
        download_urls.len()
    );

    for (index, url) in download_urls.iter().enumerate() {
        println!("Downloading link #{}..", index);
        client
            .download_from(url, &format!("{}-{}.mp4", "hey", uuid::Uuid::new_v4()))
            .await?;
        println!("Download of link #{} completed!", index);
    }

    Ok(())
}
