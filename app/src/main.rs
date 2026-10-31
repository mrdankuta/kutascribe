use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

fn main() -> eframe::Result<()> {
    let mut opts = eframe::NativeOptions::default();
    opts.viewport = opts.viewport.with_inner_size([900., 620.]);
    eframe::run_native(
        "kutascribe-gui",
        opts,
        Box::new(|cc| {
            // Polished dark theme (egui 0.31 API-safe).
            let mut visuals = egui::Visuals::dark();
            visuals.selection.bg_fill = egui::Color32::from_rgb(137, 180, 250);
            cc.egui_ctx.set_visuals(visuals);
            Ok(Box::new(KutaApp::default()))
        }),
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
    gpu_device: i32,
    log: String,
    running: bool,
    rx: Option<Receiver<String>>,
    auto_scroll: bool,
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
            gpu_device: 1,
            log: String::new(),
            running: false,
            rx: None,
            auto_scroll: true,
        }
    }
}

impl eframe::App for KutaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag-drop files onto window.
        for f in ctx.input(|i| i.raw.dropped_files.clone()) {
            if let Some(p) = f.path {
                if p.is_dir() || is_media(&p) {
                    if !self.files.contains(&p) {
                        self.files.push(p);
                    }
                }
            }
        }
        // Drain background logs.
        if let Some(rx) = &self.rx {
            while let Ok(line) = rx.try_recv() {
                if line == "__DONE__" {
                    self.running = false;
                    self.rx = None;
                    self.log.push_str("\nAll done.\n");
                    break;
                } else if line == "__ERROR__" {
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
            if self.running {
                ctx.request_repaint();
            }
        }

        // Header.
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("kutascribe");
                ui.label("local speech-to-text  •  audio + video → .txt  •  no cloud");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(format!("backend: {}", kutascribe::backend_info(&self.device)));
                });
            });
        });

        // Left: inputs + settings.
        egui::SidePanel::left("side").min_width(320.).show(ctx, |ui| {
            ui.heading("Inputs");
            ui.horizontal(|ui| {
                if ui.button("➕ Files…").clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("audio/video", &["mp3","wav","m4a","ogg","flac","mp4","mkv","webm","mov","avi"])
                        .pick_files()
                    {
                        for p in paths {
                            if !self.files.contains(&p) {
                                self.files.push(p);
                            }
                        }
                    }
                }
                if ui.button("📁 Folder…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        if !self.files.contains(&dir) {
                            self.files.push(dir);
                        }
                    }
                }
                if !self.files.is_empty() && ui.button("Clear").clicked() {
                    self.files.clear();
                }
            });
            ui.small("Tip: drag & drop files anywhere onto this window.");
            ui.separator();
            egui::ScrollArea::vertical().max_height(160.).show(ui, |ui| {
                if self.files.is_empty() {
                    ui.weak("No inputs yet.");
                }
                let mut rm: Option<usize> = None;
                for (i, f) in self.files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.monospace(f.display().to_string());
                        if ui.small_button("✕").clicked() {
                            rm = Some(i);
                        }
                    });
                }
                if let Some(i) = rm {
                    self.files.remove(i);
                }
            });
            ui.separator();
            ui.heading("Settings");
            egui::Grid::new("settings").num_columns(2).spacing([8., 6.]).show(ui, |ui| {
                ui.label("Output:");
                ui.text_edit_singleline(&mut self.outdir);
                ui.end_row();
                ui.label("Model:");
                egui::ComboBox::from_id_salt("model")
                    .selected_text(&self.model)
                    .show_ui(ui, |ui| {
                        for m in ["tiny","base","small","medium","large-v3","large-v3-turbo"] {
                            ui.selectable_value(&mut self.model, m.to_string(), m);
                        }
                    });
                ui.end_row();
                ui.label("Language:");
                ui.text_edit_singleline(&mut self.language);
                ui.end_row();
                ui.label("Device:");
                ui.add(egui::Slider::new(&mut self.gpu_device, 0..=2).text("gpu idx (0=Intel, 1=NVIDIA)"));
                egui::ComboBox::from_id_salt("device")
                    .selected_text(&self.device)
                    .show_ui(ui, |ui| {
                        for d in ["auto","cpu","vulkan","cuda","metal"] {
                            ui.selectable_value(&mut self.device, d.to_string(), d);
                        }
                    });
                ui.end_row();
            });
            ui.checkbox(&mut self.timestamps, "timestamps (default off)");
            ui.add(egui::Slider::new(&mut self.chunk_secs, 0..=300).text("chunk secs (0=native, 60=paragraphs)"));
            ui.separator();
            let can_run = !self.files.is_empty() && !self.running;
            if ui.add_enabled(can_run, egui::Button::new("▶ Transcribe")).clicked() {
                self.start();
            }
            if self.running {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("transcribing… (UI stays responsive)");
                });
            }
        });

        // Center: status + log.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Log");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.auto_scroll, "auto-scroll");
                    if ui.button("Copy log").clicked() {
                        ui.output_mut(|o| o.copied_text = self.log.clone());
                    }
                    if ui.button("Clear log").clicked() {
                        self.log.clear();
                    }
                });
            });
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    if self.log.is_empty() {
                        ui.weak("Add files on the left, pick a model, hit Transcribe.\n\nDefaults: no timestamps, 60s paragraphs. Check timestamps for [HH:MM:SS -> HH:MM:SS] chunks.\nGPU auto if built with --features vulkan/metal, else CPU.");
                    } else {
                        ui.monospace(&self.log);
                    }
                });
            ui.separator();
            ui.small("Single binary (lib, no shell-out)  •  ffmpeg + Vulkan loader on host  •  models in ~/.cache/kutascribe/models/");
        });
    }
}

fn is_media(p: &PathBuf) -> bool {
    matches!(
        p.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()).as_deref(),
        Some("mp3"|"wav"|"m4a"|"ogg"|"flac"|"opus"|"wma"|"aac"|"mp4"|"mkv"|"webm"|"mov"|"avi")
    )
}

impl KutaApp {
    fn start(&mut self) {
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
            gpu_device: self.gpu_device,
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
            let ctx = match kutascribe::load_context_with_device(&opt.model, opt.gpu_device) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send("__ERROR__".to_string());
                    let _ = tx.send(format!("model load: {:#}", e));
                    return;
                }
            };
            std::fs::create_dir_all(&outdir).ok();
            for src in files {
                let tx2 = tx.clone();
                if let Err(e) = kutascribe::transcribe_one(&ctx, &src, &outdir, &opt, &|s| {
                    let _ = tx2.send(s);
                }) {
                    let _ = tx.send(format!("ERROR {}: {:#}", src.display(), e));
                }
            }
            let _ = tx.send("__DONE__".to_string());
        });
    }
}
