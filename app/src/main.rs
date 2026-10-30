use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "kutascribe-app",
        opts,
        Box::new(|_| Ok(Box::new(KutaApp::default()))),
    )
}

struct KutaApp {
    files: Vec<PathBuf>,
    outdir: String,
    model: String,
    language: String,
    timestamps: bool,
    chunk_secs: u64,
    device: String,
    log: String,
    running: bool,
    rx: Option<Receiver<String>>,
}

impl Default for KutaApp {
    fn default() -> Self {
        Self {
            files: vec![],
            outdir: "./transcripts".to_string(),
            model: "large-v3-turbo".to_string(),
            language: "en".to_string(),
            timestamps: false,
            chunk_secs: 60,
            device: "auto".to_string(),
            log: String::new(),
            running: false,
            rx: None,
        }
    }
}

impl eframe::App for KutaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // drain background logs
        if let Some(rx) = &self.rx {
            while let Ok(line) = rx.try_recv() {
                if line == "__DONE__" {
                    self.running = false;
                    self.rx = None;
                    self.log.push_str("\nAll done.\n");
                    break;
                } else if line == "__ERROR__" {
                    // next message is the error text
                    if let Ok(e) = rx.try_recv() {
                        self.log.push_str(&format!("\nERROR: {}\n", e));
                    }
                    self.running = false;
                    self.rx = None;
                    break;
                } else {
                    self.log.push_str(&line);
                    self.log.push('\n');
                }
            }
            // keep repainting while running
            if self.running {
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("kutascribe-app — local speech-to-text");
            ui.label("Audio + video → .txt (no cloud). Same core as CLI, single binary.");
            ui.separator();

            if ui.button("Add files…").clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("audio/video", &["mp3","wav","m4a","ogg","flac","mp4","mkv","webm","mov","avi"])
                    .pick_files()
                {
                    self.files.extend(paths);
                }
            }
            if ui.button("Add folder…").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.files.push(dir);
                }
            }
            ui.label(format!("{} input(s)", self.files.len()));
            for f in self.files.iter().take(8) {
                ui.monospace(f.display().to_string());
            }

            ui.horizontal(|ui| {
                ui.label("Output:");
                ui.text_edit_singleline(&mut self.outdir);
            });
            ui.horizontal(|ui| {
                ui.label("Model:");
                ui.text_edit_singleline(&mut self.model);
                ui.label("Lang:");
                ui.text_edit_singleline(&mut self.language);
                ui.label("Device:");
                egui::ComboBox::from_id_salt("device")
                    .selected_text(&self.device)
                    .show_ui(ui, |ui| {
                        for d in ["auto","cpu","vulkan","cuda","metal"] {
                            ui.selectable_value(&mut self.device, d.to_string(), d);
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.timestamps, "timestamps (default off)");
                ui.add(egui::Slider::new(&mut self.chunk_secs, 0..=300).text("chunk secs (0=native, 60=paragraphs)"));
            });

            let can_run = !self.files.is_empty() && !self.running;
            if ui.add_enabled(can_run, egui::Button::new("Transcribe")).clicked() {
                let (tx, rx) = channel::<String>();
                self.rx = Some(rx);
                self.running = true;
                self.log = format!(
                    "Backend: {}\nModel: {}\n",
                    kutascribe::backend_info(&self.device),
                    self.model
                );
                let patterns: Vec<String> = self
                    .files
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                let opt = kutascribe::TranscribeOptions {
                    model: self.model.clone(),
                    language: self.language.clone(),
                    timestamps: self.timestamps,
                    chunk_secs: self.chunk_secs,
                    device: self.device.clone(),
                };
                let outdir = PathBuf::from(self.outdir.clone());
                std::thread::spawn(move || {
                    let send = |s: String| {
                        let _ = tx.send(s);
                    };
                    send(format!("Found {} input(s)", patterns.len()));
                    let files = kutascribe::collect_inputs(&patterns);
                    if files.is_empty() {
                        let _ = tx.send("__ERROR__".to_string());
                        let _ = tx.send("No audio/video files found.".to_string());
                        return;
                    }
                    let ctx = match kutascribe::load_context(&opt.model) {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx.send("__ERROR__".to_string());
                            let _ = tx.send(format!("model load: {:#}", e));
                            return;
                        }
                    };
                    std::fs::create_dir_all(&outdir).ok();
                    // WhisperContext is not Sync, so stay in this one thread — fine.
                    for src in files {
                        let tx2 = tx.clone();
                        match kutascribe::transcribe_one(&ctx, &src, &outdir, &opt, &|s| {
                            let _ = tx2.send(s);
                        }) {
                            Ok(_) => {}
                            Err(e) => {
                                let _ = tx.send(format!("ERROR {}: {:#}", src.display(), e));
                            }
                        }
                    }
                    let _ = tx.send("__DONE__".to_string());
                });
            }
            if self.running {
                ui.spinner();
            }
            ui.separator();
            ui.label("Log:");
            egui::ScrollArea::vertical().max_height(220.).show(ui, |ui| {
                ui.monospace(&self.log);
            });
            ui.small(format!(
                "Backend: {} | single binary (lib, no shell-out) | ffmpeg + Vulkan loader on host",
                kutascribe::backend_info(&self.device)
            ));
        });
    }
}
