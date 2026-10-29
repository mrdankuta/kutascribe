# hrc-transcribe

Local GPU speech-to-text for **audio + video** → timestamped `.txt`.

- No cloud, no API key — runs on your machine (NVIDIA 4070 tested)
- Handles `mp3/wav/m4a/ogg/flac` + `mp4/mkv/webm/mov/avi` (ffmpeg/PyAV extracts audio from video)
- Default model `large-v3-turbo` (~1.6GB, ~3-4GB VRAM) — best speed/accuracy tradeoff
- Leaves VoxType dictation untouched — separate env

## Install

```bash
cd ~/Deving/hrc-transcribe
uv venv --python 3.11
uv pip install -e .
# GPU (fixes libcublas.so.12 on CUDA systems):
uv pip install -e ".[gpu]"
# or shareable:
uv tool install -e .
```

Needs `ffmpeg` on PATH (`sudo pacman -S ffmpeg` / `sudo apt install ffmpeg`).

## Use

```bash
# single file
hrc-transcribe ~/Downloads/hrcmee/HRC7.mp3 -o ~/Downloads/hrcmee/transcripts

# whole folder (audio + video)
hrc-transcribe ~/Downloads/hrcmee/ -o ~/Downloads/hrcmee/transcripts

# video
hrc-transcribe ~/Downloads/BWAIWorkshop\ -\ Day\ 1*.mp4 -o ./transcripts

# other model / language
hrc-transcribe ./audio/ --model medium --language auto -o ./transcripts
```

Output format (`HRC1mp3.txt`):
```
Source: HRC1mp3.mp3
Language: en (p=0.99)
Duration: 1705.3s

[00:00:00 -> 00:00:08] ... text ...
[00:00:08 -> 00:00:15] ... text ...
```

## Notes

- First run downloads the model to `~/.cache/huggingface` (~1.6GB for turbo)
- `device=auto` tries CUDA float16, falls back to CPU int8
- Your VoxType dictation (`base.en`, CPU) is untouched — this is a separate batch tool
- Rust? Inference speed is bound by the model/GPU kernels, not Python. `faster-whisper` (CTranslate2) is currently fastest. Rust + whisper.cpp gives a nicer single binary for sharing, but slower to build and slightly slower inference. Stick with Python unless distribution becomes painful.
