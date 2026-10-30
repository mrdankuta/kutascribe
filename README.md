# kutascribe

Local speech-to-text for **audio + video** → `.txt`. Single binary.

- No cloud, no API key
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg extracts audio)
- Default: plain paragraphs, 60s chunks. Opt-in timestamps.
- GPU auto if built with Vulkan, else CPU.

## CLI (primary)

```bash
cd ~/Deving/kutascribe/rust
cargo build --release                        # CPU portable (6.9M)
cargo build --release --features vulkan      # GPU (needs vulkan-headers+shaderc) — 44M, use gpu=1

./target/release/kutascribe HRC7.mp3 -o out/
./target/release/kutascribe ./hrcmee/ -o ./transcripts --model large-v3-turbo
./target/release/kutascribe video.mp4 -o out/ --timestamps --chunk-secs 120
./target/release/kutascribe audio.mp3 -o out/ --chunk-secs 0 --timestamps  # native 2-8s segments
./target/release/kutascribe audio.mp3 -o out/ --device cpu   # force CPU
```

Flags:
- `--timestamps` (default off), `--chunk-secs 60` (0=native), `--device auto|cpu|vulkan|cuda`, `--model tiny|base|small|medium|large-v3-turbo`, `--language en|auto`

Models auto-download to `~/.cache/kutascribe/models/ggml-*.bin`.

## Desktop app

`app/` (eframe, builds today) — Add files/folder, pick model/lang/device, toggle timestamps, chunk slider, Transcribe button. Calls the `kutascribe` binary in PATH.

```bash
cd ~/Deving/kutascribe/app
cargo run
cargo build --release  # -> target/release/kuta-app
```

`ui-gpui/` is the GPUI skeleton (blocked: crates.io gpui 0.2.2 broken on Rust 1.98, needs Zed checkout — see its README).

## Distribute

- **Linux AppImage:** `cargo install cargo-appimage`, `cargo appimage --release -p kuta-app` → `Kutascribe-x86_64.AppImage` (bundle `ffmpeg` + model downloader)
- **Windows:** `cargo install cargo-wix`, `cargo wix` → `.msi` (add ffmpeg to PATH or bundle)
- **Mac:** `cargo-bundle` → `.dmg`, `xcode-select --install` needed, notarize for Gatekeeper
- **Brew:** create tap `mrdankuta/kutascribe` with `Formula/kutascribe.rb`:
  ```ruby
  depends_on "ffmpeg"
  def install; system "cargo", "install", "--locked", "--root", prefix, "--path", "rust"; end
  ```
  then `brew install mrdankuta/kutascribe/kutascribe`

## Python legacy

`src/kutascribe/` + `kutascribe-py` (faster-whisper CUDA, ~10x realtime on 4070). Kept for batch GPU jobs.
