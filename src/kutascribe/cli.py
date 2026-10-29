from __future__ import annotations
import argparse
from pathlib import Path
from .transcribe import load_model, transcribe_file, collect_inputs


def main():
    ap = argparse.ArgumentParser(
        prog="kutascribe",
        description="Transcribe audio/video to timestamped .txt (Whisper, local GPU)",
    )
    ap.add_argument("inputs", nargs="+", help="audio/video files, dirs, or globs")
    ap.add_argument("-o", "--output-dir", default="./transcripts", help="where .txt files go")
    ap.add_argument("--model", default="large-v3-turbo", help="tiny/base/small/medium/large-v3/large-v3-turbo")
    ap.add_argument("--language", default="en", help="en, auto, fr, ...")
    ap.add_argument("--device", default="auto", choices=["auto", "cuda", "cpu"])
    ap.add_argument("--beam-size", type=int, default=5)
    ap.add_argument("--overwrite", action="store_true")
    args = ap.parse_args()

    files = collect_inputs(args.inputs)
    if not files:
        print("No audio/video files found.")
        return 1

    print(f"Found {len(files)} file(s)")
    model = load_model(args.model, args.device)
    outdir = Path(args.output_dir).expanduser()

    for f in files:
        transcribe_file(
            model, f, outdir,
            language=args.language,
            beam_size=args.beam_size,
            overwrite=args.overwrite,
        )
    print(f"\nAll done -> {outdir.resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
