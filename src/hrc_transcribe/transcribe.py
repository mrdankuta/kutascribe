from __future__ import annotations
import os
from pathlib import Path
from faster_whisper import WhisperModel

# audio + video extensions (video works because ffmpeg/PyAV extracts the audio track)
AUDIO_EXTS = {".mp3", ".wav", ".m4a", ".ogg", ".flac", ".opus", ".wma", ".aac"}
VIDEO_EXTS = {".mp4", ".mkv", ".webm", ".mov", ".avi", ".m4v"}
ALL_EXTS = AUDIO_EXTS | VIDEO_EXTS


def format_ts(s: float) -> str:
    h = int(s // 3600)
    m = int((s % 3600) // 60)
    sec = int(s % 60)
    return f"{h:02d}:{m:02d}:{sec:02d}"


_model_cache: dict = {}


def load_model(model_name: str = "large-v3-turbo", device: str = "auto"):
    """Load once, reuse. device=auto tries CUDA then falls back to CPU."""
    key = (model_name, device)
    if key in _model_cache:
        return _model_cache[key]

    order = ["cuda", "cpu"] if device == "auto" else [device]
    last_err = None
    for dev in order:
        compute = "float16" if dev == "cuda" else "int8"
        try:
            print(f"Loading {model_name} on {dev} ({compute})...", flush=True)
            m = WhisperModel(model_name, device=dev, compute_type=compute)
            _model_cache[key] = m
            return m
        except Exception as e:
            print(f"  {dev} failed: {e}", flush=True)
            last_err = e
    raise RuntimeError(f"Could not load model: {last_err}")


def transcribe_file(
    model,
    src: Path,
    outdir: Path,
    language: str = "en",
    beam_size: int = 5,
    overwrite: bool = False,
) -> Path:
    outdir.mkdir(parents=True, exist_ok=True)
    out = outdir / (src.stem + ".txt")
    if out.exists() and out.stat().st_size > 1000 and not overwrite:
        print(f"SKIP {src.name} (exists)", flush=True)
        return out

    print(f"=== {src.name} ===", flush=True)
    segments, info = model.transcribe(
        str(src),
        language=language if language != "auto" else None,
        beam_size=beam_size,
        vad_filter=True,
        vad_parameters=dict(min_silence_duration_ms=500),
    )
    n = 0
    with open(out, "w", encoding="utf-8") as o:
        o.write(f"Source: {src.name}\n")
        o.write(f"Model: {model.model.model_path if hasattr(model, 'model') else 'whisper'}\n")
        o.write(f"Language: {info.language} (p={info.language_probability:.2f})\n")
        o.write(f"Duration: {info.duration:.1f}s\n\n")
        for seg in segments:
            o.write(f"[{format_ts(seg.start)} -> {format_ts(seg.end)}] {seg.text.strip()}\n")
            n += 1
    print(f"DONE {src.name}: {n} segments -> {out}", flush=True)
    return out


def collect_inputs(paths: list[str]) -> list[Path]:
    out: list[Path] = []
    for p in paths:
        pp = Path(p).expanduser()
        if pp.is_dir():
            for f in sorted(pp.iterdir()):
                if f.suffix.lower() in ALL_EXTS:
                    out.append(f)
        elif pp.is_file():
            out.append(pp)
        else:
            # glob pattern
            import glob
            for g in sorted(glob.glob(str(pp))):
                gp = Path(g)
                if gp.suffix.lower() in ALL_EXTS:
                    out.append(gp)
    return out
