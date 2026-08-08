use anyhow::Context;

/// 1K target for the longer dimension (1920 px horizontal for landscape,
/// 1920 px vertical for portrait). The shorter dimension is derived
/// automatically by ffmpeg to preserve the exact source aspect ratio.
const TARGET_LONG_SIDE: u32 = 1920;

/// Probe the first video stream of `input` and return its width and height.
async fn probe_dimensions(input: &str) -> anyhow::Result<(u32, u32)> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=s=x:p=0",
            input,
        ])
        .output()
        .await
        .context("failed to spawn ffprobe")?;

    anyhow::ensure!(
        output.status.success(),
        "ffprobe exited with error:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).context("ffprobe stdout is not valid UTF-8")?;
    let dims = stdout.trim();

    let (w_str, h_str) = dims
        .split_once('x')
        .with_context(|| format!("unexpected ffprobe output: '{dims}'"))?;

    let w: u32 = w_str.parse().context("invalid width from ffprobe")?;
    let h: u32 = h_str.parse().context("invalid height from ffprobe")?;

    anyhow::ensure!(w > 0 && h > 0, "video dimensions are zero: {w}x{h}");

    Ok((w, h))
}

/// Upscale `input` to 1K, preserving the original aspect ratio exactly.
///
/// The longer dimension is scaled to [`TARGET_LONG_SIDE`]_px (1920); the
/// shorter dimension is computed by ffmpeg to match the source ratio.
/// No cropping or letterboxing is applied.
///
/// # Filter pipeline
///
/// 1. **`hqdn3d`** — light spatial denoise before scaling (prevents noise
///    amplification during the upscale).
/// 2. **`scale`** — lanczos upscale with accurate rounding. `-1` lets ffmpeg
///    derive the other dimension from the source aspect ratio.
/// 3. **`cas`** — Contrast Adaptive Sharpen to recover fine detail after the
///    resize without amplifying flat-area artifacts.
///
/// # Encoding
///
/// Uses **libx265** (HEVC) with 10-bit colour (`yuv420p10le`), preset
/// `veryslow`, and CRF 16. The `hvc1` tag ensures Apple compatibility.
pub async fn upscale_video(input: &str, output: &str) -> anyhow::Result<()> {
    let (src_w, src_h) = probe_dimensions(input).await?;

    // Anchor the longer dimension at 1920; let ffmpeg compute the other via -1.
    let scale_spec = if src_w >= src_h {
        // Landscape / square — drive width, derive height.
        format!("{TARGET_LONG_SIDE}:-1")
    } else {
        // Portrait — drive height, derive width.
        format!("-1:{TARGET_LONG_SIDE}")
    };

    // Filter graph: denoise → scale → sharpen
    let vf = format!(
        "hqdn3d=4:3:6:4.5,\
         scale={scale_spec}:flags=lanczos+accurate_rnd,\
         cas=0.7"
    );

    let status = tokio::process::Command::new("ffmpeg")
        .args([
            "-i",
            input,
            "-vf",
            &vf,
            "-c:v",
            "libx265",
            "-preset",
            "veryslow",
            "-crf",
            "16",
            "-pix_fmt",
            "yuv420p10le",
            "-tag:v",
            "hvc1",
            "-c:a",
            "copy",
            output,
        ])
        .status()
        .await
        .context("failed to spawn ffmpeg")?;

    anyhow::ensure!(status.success(), "ffmpeg exited with an error");

    Ok(())
}
