# upscale_pipeline

Generate AI videos via [OpenRouter](https://openrouter.ai) and optionally upscale them with ffmpeg.

## Prerequisites

- Rust
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
cargo run -- --prompt "A mountain at sunrise" --duration 4 --image assets/ref.jpg --upscale 1280

# Full options
cargo run -- \
  --prompt "The mountain stands above the clouds" \
  --duration 4 \
  --image assets/ref.jpg \
  --resolution HD \
  --aspect-ratio 16:9 \
  --upscale 1920
```

| Flag | Description |
|---|---|
| `-p`, `--prompt` | Video generation prompt **(required)** |
| `-d`, `--duration` | Duration in seconds **(required)** |
| `-i`, `--image` | Reference image to upload and use as input |
| `-a`, `--aspect-ratio` | `1:1`, `16:9`, or `9:16` |
| `-m`, `--model` | Model ID (default: `bytedance/seedance-2.0-fast`) |
| `-r`, `--resolution` | `SD` (480p), `HD` (720p), or `FHD` (1080p) (default: `SD`) |
| `-u`, `--upscale` | Upscale longer side to N pixels; omit to skip upscale entirely |
| `-h`, `--help` | Print help |

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
