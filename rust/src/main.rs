use anyhow::{bail, Result};
use clap::Parser;
use std::path::PathBuf;
use kutascribe::{backend_info, collect_inputs, shellexpand, transcribe_one, TranscribeOptions};

#[derive(Parser, Debug)]
#[command(name = "kutascribe-cli", version)]
struct Args {
    #[arg(required = true)]
    inputs: Vec<String>,
    #[arg(short, long, default_value = "./transcripts")]
    output_dir: String,
    #[arg(long, default_value = "large-v3-turbo")]
    model: String,
    #[arg(long, default_value = "en")]
    language: String,
    #[arg(long, default_value_t = false)]
    timestamps: bool,
    #[arg(long, default_value_t = 60)]
    chunk_secs: u64,
    #[arg(long, default_value = "auto")]
    device: String,
    /// Vulkan GPU index (vulkaninfo order: 0=Intel iGPU, 1=NVIDIA on hybrid). Try 1 for NVIDIA.
    #[arg(long, default_value_t = 0)]
    gpu_device: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let opt = TranscribeOptions {
        model: args.model.clone(),
        language: args.language.clone(),
        timestamps: args.timestamps,
        chunk_secs: args.chunk_secs,
        device: args.device.clone(),
        gpu_device: args.gpu_device,
    };
    let files = collect_inputs(&args.inputs);
    if files.is_empty() {
        bail!("No audio/video files found.");
    }
    println!("Found {} file(s)", files.len());
    println!("Backend: {}", backend_info(&opt.device));
    println!(
        "Timestamps: {}, chunk_secs: {}",
        if opt.timestamps { "on" } else { "off (default)" },
        opt.chunk_secs
    );

    let ctx = kutascribe::load_context_with_device(&opt.model, opt.gpu_device)?;
    println!("GPU device index: {} (0=usually Intel iGPU, 1=NVIDIA on hybrid; check vulkaninfo --summary)", opt.gpu_device);
    let outdir = PathBuf::from(shellexpand(&args.output_dir));
    std::fs::create_dir_all(&outdir)?;

    for src in files {
        transcribe_one(&ctx, &src, &outdir, &opt, &|s| println!("{}", s))?;
    }
    println!("\nAll done -> {}", outdir.display());
    Ok(())
}
