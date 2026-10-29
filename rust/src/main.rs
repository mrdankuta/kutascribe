use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

/// kutascribe: local speech-to-text (audio+video) -> .txt
/// Single binary, whisper.cpp via whisper-rs.
#[derive(Parser, Debug)]
#[command(name = "kutascribe", version)]
struct Args {
    /// Input files, dirs, or globs (mp3/wav/m4a + mp4/mkv/webm/mov)
    #[arg(required = true)]
    inputs: Vec<String>,

    /// Where .txt files go
    #[arg(short, long, default_value = "./transcripts")]
    output_dir: String,

    /// Model file or preset: tiny/base/small/medium/large-v3-turbo
    #[arg(long, default_value = "large-v3-turbo")]
    model: String,

    /// Language: en, auto, fr...
    #[arg(long, default_value = "en")]
    language: String,

    /// Include timestamps? Default off (plain paragraphs).
    #[arg(long, default_value_t = false)]
    timestamps: bool,

    /// Merge native whisper segments into larger chunks up to N seconds.
    /// 0 = native whisper segmentation (2-8s). 60 = ~1-min paragraphs.
    #[arg(long, default_value_t = 60)]
    chunk_secs: u64,

    /// Compute device: auto, cpu, vulkan, cuda.
    /// auto = GPU if compiled in + available, else CPU.
    #[arg(long, default_value = "auto")]
    device: String,
}

const AUDIO_EXTS: &[&str] = &["mp3", "wav", "m4a", "ogg", "flac", "opus", "wma", "aac"];
const VIDEO_EXTS: &[&str] = &["mp4", "mkv", "webm", "mov", "avi", "m4v"];

fn fmt_ts(sec: f64) -> String {
    let s = sec.max(0.0) as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn model_path(name: &str) -> Result<PathBuf> {
    let p = Path::new(name);
    if p.exists() {
        return Ok(p.to_path_buf());
    }
    let fname = if name.ends_with(".bin") {
        name.to_string()
    } else {
        format!("ggml-{}.bin", name)
    };
    let dir = dirs::cache_dir()
        .unwrap_or(PathBuf::from("."))
        .join("kutascribe")
        .join("models");
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(fname.clone());
    if dest.exists() {
        return Ok(dest);
    }
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        fname
    );
    println!("Downloading model {} ...", url);
    println!("  -> {}", dest.display());
    let mut resp = reqwest::blocking::get(&url)
        .with_context(|| format!("download {}", url))?
        .error_for_status()?;
    let mut out = std::fs::File::create(&dest)?;
    std::io::copy(&mut resp, &mut out)?;
    println!("Model downloaded.");
    Ok(dest)
}

fn shellexpand(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    s.to_string()
}

fn collect_inputs(patterns: &[String]) -> Vec<PathBuf> {
    let mut out = vec![];
    for pat in patterns {
        let expanded = shellexpand(pat);
        let p = PathBuf::from(&expanded);
        if p.is_dir() {
            for e in walkdir::WalkDir::new(&p).max_depth(1) {
                if let Ok(entry) = e {
                    if entry.file_type().is_file() {
                        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                            let ext_lc = ext.to_lowercase();
                            if AUDIO_EXTS.contains(&ext_lc.as_str())
                                || VIDEO_EXTS.contains(&ext_lc.as_str())
                            {
                                out.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            }
        } else if p.is_file() {
            out.push(p);
        } else if let Ok(paths) = glob::glob(&expanded) {
            for g in paths.flatten() {
                out.push(g);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn backend_info(requested: &str) -> &'static str {
    #[cfg(feature = "vulkan")]
    {
        if requested == "cpu" { return "cpu (forced)"; }
        return "vulkan (auto)";
    }
    #[cfg(feature = "cuda")]
    {
        if requested == "cpu" { return "cpu (forced)"; }
        return "cuda (auto)";
    }
    #[cfg(not(any(feature = "vulkan", feature = "cuda")))]
    {
        let _ = requested;
        return "cpu (this binary built without GPU; rebuild with --features vulkan)";
    }
}

struct Chunk { start: f64, end: f64, text: String }

fn merge_segments(segs: Vec<(f64, f64, String)>, chunk_secs: u64) -> Vec<Chunk> {
    if chunk_secs == 0 {
        return segs.into_iter().map(|(s,e,t)| Chunk{start:s,end:e,text:t}).collect();
    }
    let mut out: Vec<Chunk> = vec![];
    for (s,e,t) in segs {
        let t = t.trim().to_string();
        if t.is_empty() { continue; }
        if let Some(last) = out.last_mut() {
            if e - last.start <= chunk_secs as f64 {
                last.end = e;
                last.text.push(' ');
                last.text.push_str(&t);
                continue;
            }
        }
        out.push(Chunk{start:s,end:e,text:t});
    }
    out
}

fn to_wav_16k(src: &Path, tmp: &Path) -> Result<()> {
    // video works here: ffmpeg extracts the audio track
    let st = Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-i",
            &src.to_string_lossy(),
            "-ar",
            "16000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(tmp)
        .status()
        .context("run ffmpeg (install ffmpeg)")?;
    if !st.success() {
        bail!("ffmpeg failed for {}", src.display());
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = collect_inputs(&args.inputs);
    if files.is_empty() {
        bail!("No audio/video files found.");
    }
    println!("Found {} file(s)", files.len());
    println!("Backend: {}", backend_info(&args.device));
    println!("Timestamps: {}, chunk_secs: {}", if args.timestamps {"on"} else {"off (default)"}, args.chunk_secs);

    let mpath = model_path(&args.model)?;
    println!("Model: {}", mpath.display());

    let ctx = whisper_rs::WhisperContext::new_with_params(
        &mpath,
        whisper_rs::WhisperContextParameters::default(),
    )
    .map_err(|e| anyhow::anyhow!("load model: {:?}", e))?;
    let outdir = PathBuf::from(shellexpand(&args.output_dir));
    std::fs::create_dir_all(&outdir)?;

    for src in files {
        let stem = src
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("out");
        let out = outdir.join(format!("{}.txt", stem));
        if out.exists() && out.metadata().map(|m| m.len()).unwrap_or(0) > 1000 {
            println!("SKIP {} (exists)", src.display());
            continue;
        }
        println!(
            "=== {} ===",
            src.file_name().unwrap_or_default().to_string_lossy()
        );

        let tmp = std::env::temp_dir().join(format!("kutascribe-{}.wav", stem));
        to_wav_16k(&src, &tmp)?;

        let reader = hound::WavReader::open(&tmp)?;
        let samples_i16: Vec<i16> = reader
            .into_samples::<i16>()
            .map(|s| s.unwrap_or(0))
            .collect();
        // mono already (ffmpeg -ac 1), so int16 -> float32 is enough
        let mut samples_f32 = vec![0.0f32; samples_i16.len()];
        whisper_rs::convert_integer_to_float_audio(&samples_i16, &mut samples_f32)
            .map_err(|e| anyhow::anyhow!("audio convert: {:?}", e))?;

        let mut state = ctx
            .create_state()
            .map_err(|e| anyhow::anyhow!("create state: {:?}", e))?;
        let mut params =
            whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        if args.language != "auto" {
            params.set_language(Some(&args.language));
        }
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &samples_f32[..])
            .map_err(|e| anyhow::anyhow!("transcribe: {:?}", e))?;

        let n = state.full_n_segments();
        let mut raw: Vec<(f64,f64,String)> = vec![];
        for i in 0..n {
            if let Some(seg) = state.get_segment(i) {
                let text = seg.to_str_lossy().unwrap_or_default().to_string();
                let t0 = seg.start_timestamp() as f64 / 100.0;
                let t1 = seg.end_timestamp() as f64 / 100.0;
                raw.push((t0, t1, text));
            }
        }
        let chunks = merge_segments(raw, args.chunk_secs);
        let mut f = std::fs::File::create(&out)?;
        use std::io::Write;
        writeln!(
            f,
            "Source: {}",
            src.file_name().unwrap_or_default().to_string_lossy()
        )?;
        writeln!(f, "Language: {}", args.language)?;
        if args.timestamps {
            writeln!(f, "Timestamps: on, chunk_secs: {}", args.chunk_secs)?;
        } else {
            writeln!(f, "Timestamps: off, chunk_secs: {}", args.chunk_secs)?;
        }
        writeln!(f)?;
        for c in &chunks {
            if args.timestamps {
                writeln!(f, "[{} -> {}] {}", fmt_ts(c.start), fmt_ts(c.end), c.text.trim())?;
            } else {
                writeln!(f, "{}", c.text.trim())?;
                writeln!(f)?;
            }
        }
        println!("DONE -> {} ({} chunks from {} segments)", out.display(), chunks.len(), n);
        let _ = std::fs::remove_file(&tmp);
    }
    println!("\nAll done -> {}", outdir.display());
    Ok(())
}
