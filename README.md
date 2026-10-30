# kutascribe

Local speech-to-text for **audio + video** → `.txt`. Single binary + desktop app.

- No cloud, no API key
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg extracts audio)
- Default: plain paragraphs, 60s chunks. Opt-in timestamps.
- GPU auto (Vulkan/Linux, Metal/Mac, CUDA/optional), else CPU.

```
kutascribe/            # workspace root (cargo build --bin kutascribe-cli / --bin kutascribe-gui)
  rust/                # CLI single binary (whisper-rs) -> `kutascribe`
  app/                 # desktop app (eframe) -> `kutascribe-gui`, calls `kutascribe` in PATH
  ui-gpui/             # GPUI skeleton (blocked upstream, see its README)
  packaging/           # AppImage script + Brew formula
  .github/workflows/   # release builds (linux/windows/mac)
```

## Install / build

```bash
# workspace root
cargo build --release --bin kutascribe-cli --bin kutascribe-gui  # CPU portable (works everywhere)
cargo build --release --bin kutascribe-cli --features vulkan  # local GPU (needs vulkan-headers+shaderc)
cargo build --release --bin kutascribe-cli --no-default-features --features metal  # Mac GPU (needs vulkan-headers+shaderc+ffmpeg)
cargo build --release --bin kutascribe-cli --no-default-features  # CPU portable
cargo install --path rust --bin kutascribe-cli --locked
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe-cli --locked
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe-gui --locked  # GUI
# local GPU:
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe-cli --locked --features vulkan

# CPU-only everywhere (no system deps besides ffmpeg):
cargo install --git https://github.com/mrdankuta/kutascribe --bin kutascribe-cli --locked --no-default-features
```

## Use (CLI)

```bash
kutascribe-cli HRC7.mp3 -o out/
kutascribe-cli ./hrcmee/ -o ./transcripts --model large-v3-turbo
kutascribe-cli video.mp4 -o out/ --timestamps --chunk-secs 120
kutascribe-cli audio.mp3 -o out/ --chunk-secs 0 --timestamps  # native 2-8s segments
kutascribe-cli audio.mp3 -o out/ --device cpu
```

Flags: `--timestamps` (default off), `--chunk-secs 60` (0=native), `--device auto|cpu|vulkan|cuda`, `--model tiny|base|small|medium|large-v3-turbo`, `--language en|auto`

Models auto-download to `~/.cache/kutascribe/models/ggml-*.bin`.

> SIGILL/AVX512 note (fixed in v0.2.9+): CI runners have AVX512, most laptops (e.g. i9-13900H) don't.
> ggml with `GGML_NATIVE=ON` (`-march=native`) bakes AVX512 in → instant crash on model load.
> Releases now build with `GGML_NATIVE=OFF GGML_AVX512=OFF` (portable + runtime dispatch).
> Local GPU build on your own CPU is safe, but use the same flags when sharing binaries.

## Desktop app

`app/` — Add files/folder, model/lang/device, timestamps checkbox, chunk slider, Transcribe. Shells out to `kutascribe` in PATH (so install CLI first).

```bash
cargo run -p kutascribe-gui
cargo build --release --bin kutascribe-gui
```

## How it all fits

- `kutascribe` (Rust): core engine. Video→ffmpeg→wav16k→whisper.cpp→merge→`.txt`. Single binary, GPU auto.
- `kutascribe-gui` (eframe): thin GUI over the CLI. No whisper linking, just spawns `kutascribe` so CLI/GUI never drift. For AppImage we bundle both binaries + depend on host ffmpeg/Vulkan loader.
- Python legacy archived in branch `legacy/python-faster-whisper` (faster-whisper CUDA).
- `ui-gpui`: future native GPUI frontend. Blocked: crates.io gpui 0.2.2 broken on Rust 1.98, real GPUI needs Zed checkout.

## Distribute

- **Linux AppImage:** `./packaging/appimage.sh` (needs `appimagetool`). Bundles `kutascribe-gui` + `kutascribe`. Host needs: ffmpeg + Vulkan loader (via GPU drivers). Output: `Kutascribe-x86_64.AppImage`. CI builds it on tag.
- **Windows:** `cargo wix` → `.msi`. Build CPU (`--no-default-features`) for portability, bundle `ffmpeg.exe` or winget dep. CI: `windows-latest` CPU.
- **Mac (Intel+ARM):** `cargo-bundle` → `.dmg`, notarize. CI builds both `x86_64-apple-darwin` + `aarch64-apple-darwin` CPU. GPU via Metal is future (`whisper-rs/metal`).
- **Brew:** `packaging/Formula/kutascribe.rb` → `brew tap mrdankuta/kutascribe && brew install kutascribe` (depends on ffmpeg, CPU portable).
- **Release flow:** `git tag v0.1.0 && git push --tags` → `.github/workflows/release.yml` builds linux/windows/mac + AppImage, upload to GitHub Releases.
