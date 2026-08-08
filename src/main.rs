use std::env;

use anyhow::Context;

mod api;
mod upscale;

const DEFAULT_MODEL: &str = "bytedance/seedance-2.0-fast";
const USAGE: &str = "\
Usage: upscale_pipeline --prompt <PROMPT> [OPTIONS]

Options:
  -p, --prompt <TEXT>        Video generation prompt (required)
  -d, --duration <SECS>      Video duration in seconds (required)
  -m, --model <MODEL>        Model ID (default: bytedance/seedance-1-5-pro)
  -i, --image <PATH>         Input reference image to upload
  -a, --aspect-ratio <AR>    Target aspect ratio: 1:1, 16:9, or 9:16
  -r, --resolution <RES>     Output resolution: SD, HD, or FHD (default: SD)
  -s, --size <PX>            Upscale longer side to N pixels (omitting skips upscale)
  -h, --help                 Print this help";

struct Args {
    prompt: String,
    image: Option<String>,
    aspect_ratio: Option<api::AspectRatio>,
    duration: i32,
    model: String,
    resolution: Option<api::Resolution>,
    upscale_to: Option<u32>,
}

fn parse_args() -> anyhow::Result<Args> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        std::process::exit(0);
    }

    let mut prompt = None;
    let mut image = None;
    let mut aspect_ratio = None;
    let mut duration = None;
    let mut model = None;
    let mut resolution = None;
    let mut upscale_to = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--prompt" | "-p" => {
                i += 1;
                prompt = Some(args.get(i).context("missing value for --prompt")?.clone());
            }
            "--image" | "-i" => {
                i += 1;
                image = Some(args.get(i).context("missing value for --image")?.clone());
            }
            "--aspect-ratio" | "-a" => {
                i += 1;
                let val = args.get(i).context("missing value for --aspect-ratio")?;
                aspect_ratio = Some(match val.as_str() {
                    "1:1" => api::AspectRatio::Square,
                    "16:9" => api::AspectRatio::Landscape,
                    "9:16" => api::AspectRatio::Portrait,
                    other => {
                        anyhow::bail!("invalid aspect ratio '{other}'. Use 1:1, 16:9, or 9:16")
                    }
                });
            }
            "--duration" | "-d" => {
                i += 1;
                let val = args.get(i).context("missing value for --duration")?;
                duration = Some(
                    val.parse::<i32>()
                        .context("--duration must be an integer (seconds)")?,
                );
            }
            "--model" | "-m" => {
                i += 1;
                model = Some(args.get(i).context("missing value for --model")?.clone());
            }
            "--resolution" | "-r" => {
                i += 1;
                let val = args.get(i).context("missing value for --resolution")?;
                resolution = Some(match val.to_lowercase().as_str() {
                    "sd" | "480p" => api::Resolution::SD,
                    "hd" | "720p" => api::Resolution::HD,
                    "fhd" | "1080p" => api::Resolution::FHD,
                    other => {
                        anyhow::bail!("invalid resolution '{other}'. Use SD, HD, or FHD")
                    }
                });
            }
            "--upscale" | "-u" => {
                i += 1;
                let val = args.get(i).context("missing value for --upscale")?;
                upscale_to = Some(
                    val.parse::<u32>()
                        .context("--upscale must be a positive integer (pixels)")?,
                );
            }
            other => anyhow::bail!("unknown argument: {other}\n\n{USAGE}"),
        }
        i += 1;
    }

    let prompt = prompt.context(format!("--prompt is required\n\n{USAGE}"))?;
    let duration = duration.context(format!("--duration is required\n\n{USAGE}"))?;

    Ok(Args {
        prompt,
        image,
        aspect_ratio,
        duration,
        model: model.unwrap_or_else(|| DEFAULT_MODEL.into()),
        resolution: resolution.map_or(Some(api::Resolution::SD), |it| Some(it)),
        upscale_to,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_args()?;
    let job_id = uuid::Uuid::new_v4();

    // ── Step 1: Upload reference image to tmpfiles (if provided) ──────────
    let input_references = if let Some(ref img_path) = args.image {
        println!("Uploading reference image...");
        let tmpfiles = api::TmpFilesClient::new();

        let page_url = tmpfiles
            .upload(img_path.clone())
            .await
            .context("failed to upload image to tmpfiles")?;
        let direct_url = tmpfiles
            .get_download_link(page_url)
            .await
            .context("failed to resolve tmpfiles download link")?;

        println!("  image URL: {direct_url}");
        Some(vec![api::InputReference::from_url(direct_url)])
    } else {
        None
    };

    // ── Step 2: Submit video generation job at 480p ──────────────────────
    let client = api::OpenRouterClient::new()?;

    let request = api::VideoRequest {
        model: args.model,
        prompt: args.prompt,
        duration: args.duration,
        resolution: args.resolution,
        aspect_ratio: args.aspect_ratio,
        size: None,
        input_references,
    };

    println!("Submitting video generation job...");
    let job = client
        .create_job(&request)
        .await
        .context("failed to create video generation job")?;
    println!("  job id: {}", job.id);

    // ── Step 3: Poll until complete ──────────────────────────────────────
    println!("Waiting for job to finish (polling every 30s)...");
    let download_urls = client
        .poll_until_done(&job)
        .await
        .context("video generation job did not complete successfully")?;
    println!(
        "  job complete — {} video(s) generated",
        download_urls.len()
    );

    // ── Step 4: Download to /tmp, upscale, move to output/ ───────────────
    std::fs::create_dir_all("output").context("failed to create output/")?;

    for (i, url) in download_urls.iter().enumerate() {
        let tmp_path = format!("/tmp/{job_id}-{i}.mp4");
        let out_path = format!("output/{job_id}-{i}.mp4");

        print!("Video {}/{}: downloading", i + 1, download_urls.len());

        client
            .download_from(url, &tmp_path)
            .await
            .context("failed to download generated video")?;

        if let Some(size) = args.upscale_to {
            println!(" → upscaling to {size}px...");
            upscale::upscale_video(&tmp_path, &out_path, size)
                .await
                .context("failed to upscale video")?;
        } else {
            println!(" → output");
            std::fs::copy(&tmp_path, &out_path).context("failed to copy video to output/")?;
        }

        // Clean up the temporary download.
        if let Err(e) = std::fs::remove_file(&tmp_path) {
            eprintln!("  warning: could not remove temp file {tmp_path}: {e}");
        }

        println!("  done → {out_path}");
    }

    println!("All done.");
    Ok(())
}
