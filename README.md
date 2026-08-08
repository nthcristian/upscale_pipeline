# upscale_pipeline

Generate AI videos via [OpenRouter](https://openrouter.ai) and upscale them with ffmpeg.

## Prerequisites

- Rust nightly (2024 edition)
- ffmpeg and ffprobe on `$PATH`
- An OpenRouter API key in `.env`:

```
OPENROUTER_API_KEY=sk-or-v1-...
```

## Usage

```bash
# Text-to-video
cargo run -- --prompt "A mountain at sunrise"

# With a reference image
cargo run -- --prompt "A mountain at sunrise" --image assets/ref.jpg

# Full options
cargo run -- \
  --prompt "The mountain stands above the clouds" \
  --image assets/ref.jpg \
  --aspect-ratio 16:9 \
  --duration 4 \
  --model bytedance/seedance-2.0-fast
```

| Flag | Description |
|---|---|
| `-p`, `--prompt` | Video generation prompt **(required)** |
| `-i`, `--image` | Reference image to upload and use as input |
| `-a`, `--aspect-ratio` | `1:1`, `16:9`, or `9:16` |
| `-d`, `--duration` | Duration in seconds |
| `-m`, `--model` | Model ID (default: `bytedance/seedance-2.0-fast`) |
| `-h`, `--help` | Print help |

## Pipeline

```
--image? → tmpfiles upload → resolve direct URL ─┐
                                                   ├→ OpenRouter job (480p)
--prompt, AR, duration ────────────────────────────┘
     ↓
  poll every 30s
     ↓
  download to /tmp/
     ↓
  upscale to 1280px (nlmeans → lanczos → cas → libx265 10-bit)
     ↓
  output/{uuid}.mp4
```

## Structure

```
src/
├── main.rs                    — CLI entry point and pipeline orchestration
├── upscale.rs                 — ffmpeg-based video upscaling
└── api/
    ├── mod.rs                 — module root
    ├── openrouter/
    │   ├── client.rs          — OpenRouter HTTP client
    │   └── types/             — request and response models
    └── tmpfiles/
        └── client.rs          — tmpfiles.org image upload
```
