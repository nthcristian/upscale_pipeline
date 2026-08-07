# upscale_pipeline

A Rust CLI tool for generating AI videos via the [OpenRouter](https://openrouter.ai) API.

## Features

- Submit video generation jobs with configurable resolution, aspect ratio, and duration
- Poll job status until completion
- Download generated videos concurrently from signed URLs
- Modular architecture with separated API client, request types, and response types

## Prerequisites

- Rust 2024 edition (nightly)
- An [OpenRouter API key](https://openrouter.ai/keys) set in a `.env` file:
  ```
  OPENROUTER_API_KEY=sk-or-v1-...
  ```

## Usage

```bash
cargo run
```

## Project Structure

```
src/
├── main.rs                              — Entry point
└── api/
    ├── mod.rs                           — API module root
    └── openrouter/
        ├── mod.rs                       — Re-exports
        ├── client.rs                    — HTTP client and job orchestration
        └── types/
            ├── mod.rs                   — Type re-exports
            ├── external.rs              — API response/request types
            └── internal.rs              — Domain types (Job, VideoRequest, enums)
```

## Dependencies

| Crate | Purpose |
|---|---|
| `reqwest` | HTTP client with streaming support |
| `serde` / `serde_json` | Serialization of request/response payloads |
| `tokio` | Async runtime |
| `anyhow` | Flexible error handling |
| `dotenv` | Environment variable loading |
| `futures-util` | Async stream utilities |
| `uuid` | Unique output filenames |
