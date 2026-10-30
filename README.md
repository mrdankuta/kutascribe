# kutascribe

Local speech-to-text for **audio + video** → `.txt`. Single binary + desktop app.

- No cloud, no API key
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg extracts audio)
- Default: plain paragraphs, 60s chunks. Opt-in timestamps.
- GPU auto if built with Vulkan, else CPU.

```
kutascribe/            # workspace root (cargo build --bin kutascribe / --bin kuta-app)
  rust/                # CLI single binary (whisper-rs) -> `kutascribe`
  app/                 # desktop app (eframe) -> `kuta-app`, calls `kutascribe` in PATH
  ui-gpui/             # GPUI skeleton (blocked upstream, see its README)
  src/kutascribe/      # Python legacy (faster-whisper CUDA) -> `kutascribe-py`
  dist/                # AppImage script + Brew formula
  .github/workflows/   # release builds (linux/windows/mac)
```

## Install / build

```bash
# workspace root
cargo build --release --bin kutascribe          # GPU default (needs vulkan-headers+shaderc+ffmpeg)
cargo build --release --bin kutascribe --no-default-features  # CPU portable
cargo install --path rust --bin kutascribe --locked
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe --locked
cargo install --git https://github.com/mrdankuta/kutascribe --bin kuta-app --locked  # GUI

# CPU-only everywhere (no system deps besides ffmpeg):
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe --locked --no-default-features
```

## Use (CLI)

```bash
kutascribe HRC7.mp3 -o out/
kutascribe ./hrcmee/ -o ./transcripts --model large-v3-turbo
kutascribe video.mp4 -o out/ --timestamps --chunk-secs 120
kutascribe audio.mp3 -o out/ --chunk-secs 0 --timestamps  # native 2-8s segments
kutascribe audio.mp3 -o out/ --device cpu
```

Flags: `--timestamps` (default off), `--chunk-secs 60` (0=native), `--device auto|cpu|vulkan|cuda`, `--model tiny|base|small|medium|large-v3-turbo`, `--language en|auto`

Models auto-download to `~/.cache/kutascribe/models/ggml-*.bin`.

## Desktop app

`app/` — Add files/folder, model/lang/device, timestamps checkbox, chunk slider, Transcribe. Shells out to `kutascribe` in PATH (so install CLI first).

```bash
cargo run -p kuta-app
cargo build --release --bin kuta-app
```

## How it all fits

- `kutascribe` (Rust): core engine. Video→ffmpeg→wav16k→whisper.cpp→merge→`.txt`. Single binary, GPU auto.
- `kuta-app` (eframe): thin GUI over the CLI. No whisper linking, just spawns `kutascribe` so CLI/GUI never drift. For AppImage we bundle both binaries + depend on host ffmpeg/Vulkan loader.
- `kutascribe-py` (Python): legacy fast GPU batch path (faster-whisper). Keep for your 4070 bulk jobs.
- `ui-gpui`: future native GPUI frontend. Blocked: crates.io gpui 0.2.2 broken on Rust 1.98, real GPUI needs Zed checkout.

## Distribute

- **Linux AppImage:** `./dist/appimage.sh` (needs `appimagetool`). Bundles `kuta-app` + `kutascribe`. Host needs: ffmpeg + Vulkan loader (via GPU drivers). Output: `Kutascribe-x86_64.AppImage`. CI builds it on tag.
- **Windows:** `cargo wix` → `.msi`. Build CPU (`--no-default-features`) for portability, bundle `ffmpeg.exe` or winget dep. CI: `windows-latest` CPU.
- **Mac (Intel+ARM):** `cargo-bundle` → `.dmg`, notarize. CI builds both `x86_64-apple-darwin` + `aarch64-apple-darwin` CPU. GPU via Metal is future (`whisper-rs/metal`).
- **Brew:** `dist/Formula/kutascribe.rb` → `brew tap mrdankuta/kutascribe && brew install kutascribe` (depends on ffmpeg, CPU portable).
- **Release flow:** `git tag v0.1.0 && git push --tags` → `.github/workflows/release.yml` builds linux/windows/mac + AppImage, upload to GitHub Releases.
