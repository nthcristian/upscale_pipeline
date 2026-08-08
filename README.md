# upscale_pipeline

Generate AI videos via [OpenRouter](https://openrouter.ai) and optionally upscale them with ffmpeg.

## Prerequisites

- Rust nightly (2024 edition)
- ffmpeg and ffprobe on `$PATH`
- An OpenRouter API key in `.env`:

```
OPENROUTER_API_KEY=sk-or-v1-...
```

## Usage

```bash
# Minimal — text to 480p video, no upscale
cargo run -- --prompt "A mountain at sunrise" --duration 4

# With a reference image and upscale to 1280px
cargo run -- --prompt "A mountain at sunrise" --duration 4 --image assets/ref.jpg --size 1280

# Full options
cargo run -- \
  --prompt "The mountain stands above the clouds" \
  --duration 4 \
  --image assets/ref.jpg \
  --resolution HD \
  --aspect-ratio 16:9 \
  --size 1920
```

| Flag | Description |
|---|---|
| `-p`, `--prompt` | Video generation prompt **(required)** |
| `-d`, `--duration` | Duration in seconds **(required)** |
| `-i`, `--image` | Reference image to upload and use as input |
| `-a`, `--aspect-ratio` | `1:1`, `16:9`, or `9:16` |
| `-m`, `--model` | Model ID (default: `bytedance/seedance-2.0-fast`) |
| `-r`, `--resolution` | `SD` (480p), `HD` (720p), or `FHD` (1080p) (default: `SD`) |
| `-s`, `--size` | Upscale longer side to N pixels; omit to skip upscale entirely |
| `-h`, `--help` | Print help |

## Pipeline

```
--image? → tmpfiles upload → resolve direct URL ─┐
                                                   ├→ OpenRouter job
--prompt, resolution, AR, duration ────────────────┘
     ↓
  poll every 30s
     ↓
  download to /tmp/
     ↓
  --size?  yes → upscale to target px (nlmeans → lanczos → cas → libx265 10-bit)
           no  → move directly to output/
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
