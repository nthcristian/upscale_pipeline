use anyhow::Context;

const TARGET_LONG_SIDE: u32 = 1280;

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

pub async fn upscale_video(input: &str, output: &str) -> anyhow::Result<()> {
    let (src_w, src_h) = probe_dimensions(input).await?;

    let scale_spec = if src_w >= src_h {
        format!("{TARGET_LONG_SIDE}:-2")
    } else {
        format!("-2:{TARGET_LONG_SIDE}")
    };

    let vf = format!(
        "format=yuv420p10le,\
         nlmeans=s=1.5,\
         scale={scale_spec}:flags=lanczos+accurate_rnd:param0=3,\
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
            "-x265-params",
            "aq-mode=3:no-sao=1",
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
