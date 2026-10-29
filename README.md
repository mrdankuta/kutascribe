# kutascribe

Local speech-to-text for **audio + video** → timestamped `.txt`. Single binary.

- No cloud, no API key
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg extracts audio)
- Output: `[HH:MM:SS -> HH:MM:SS] text` in `.txt`
- Leaves VoxType dictation untouched

## Rust single binary (primary)

```bash
cd ~/Deving/kutascribe/rust
cargo build --release
./target/release/kutascribe ~/Downloads/hrcmee/HRC7.mp3 -o /tmp/out --model tiny
./target/release/kutascribe ~/Downloads/hrcmee/ -o ~/Downloads/hrcmee/transcripts --model large-v3-turbo

# install globally
cargo install --path .
# or from GitHub
cargo install --git https://github.com/mrdankuta/kutascribe --locked
```

Models auto-download to `~/.cache/kutascribe/models/` as `ggml-<name>.bin` from `huggingface.co/ggerganov/whisper.cpp`:
`tiny` (75M, fast test) / `base` / `small` / `medium` / `large-v3-turbo` (~1.6GB, best)

Needs `ffmpeg` on PATH.

Current build: CPU portable (no system deps). For GPU: `sudo pacman -S vulkan-headers shaderc` then enable `vulkan` feature in `rust/Cargo.toml`.

## Python GPU version (legacy, faster on NVIDIA)

`faster-whisper` + CUDA, ~10x realtime on RTX 4070. Kept for batch GPU jobs.

```bash
cd ~/Deving/kutascribe
uv venv --python 3.11
uv pip install -e ".[gpu]"
kutascribe-py ~/Downloads/hrcmee/ -o ./transcripts --model large-v3-turbo
```

## Format

```
Source: HRC7.mp3
Language: en

[00:00:00 -> 00:00:02] and they will be able to combat for us.
```
