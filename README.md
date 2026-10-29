# kutascribe

Local GPU speech-to-text for **audio + video** → timestamped `.txt`.

- No cloud, no API key — runs on your machine (NVIDIA 4070 tested)
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg/PyAV extracts audio from video)
- Default model `large-v3-turbo` (~1.6GB, ~3-4GB VRAM) — best speed/accuracy tradeoff
- Leaves VoxType dictation untouched — separate env

## Install (Python v0.1)

```bash
cd ~/Deving/kutascribe
uv venv --python 3.11
uv pip install -e .
# GPU (fixes libcublas.so.12 on CUDA systems):
uv pip install -e ".[gpu]"
```

Needs `ffmpeg` on PATH.

## Use

```bash
# single file
kutascribe ~/Downloads/hrcmee/HRC7.mp3 -o ~/Downloads/hrcmee/transcripts

# whole folder (audio + video)
kutascribe ~/Downloads/hrcmee/ -o ~/Downloads/hrcmee/transcripts

# video
kutascribe ~/Downloads/BWAIWorkshop*.mp4 -o ./transcripts
```

Output (`HRC1mp3.txt`):
```
Source: HRC1mp3.mp3
Language: en (p=0.99)
Duration: 1705.3s

[00:00:00 -> 00:00:08] ... text ...
```

## Rust single-binary (in progress)

See `rust/` — `cargo build --release` → single `kutascribe` binary using whisper.cpp (Vulkan/CUDA/CPU), no Python needed. Goal: `cargo install --git https://github.com/mrdankuta/kutascribe`.
