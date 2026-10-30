use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "kutascribe",
        opts,
        Box::new(|_| Ok(Box::new(KutaApp::default()))),
    )
}

#[derive(Default)]
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
}

impl eframe::App for KutaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("kutascribe — local speech-to-text");
            ui.label("Audio + video → .txt (no cloud). Uses the kutascribe single binary.");
            ui.separator();

            if self.outdir.is_empty() {
                self.outdir = "./transcripts".to_string();
            }
            if self.model.is_empty() {
                self.model = "large-v3-turbo".to_string();
            }
            if self.language.is_empty() {
                self.language = "en".to_string();
            }
            if self.device.is_empty() {
                self.device = "auto".to_string();
            }
            if self.chunk_secs == 0 {
                self.chunk_secs = 60;
            }

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
                        for d in ["auto","cpu","vulkan","cuda"] {
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
                self.running = true;
                self.log = "starting…\n".to_string();
                let files = self.files.clone();
                let outdir = self.outdir.clone();
                let model = self.model.clone();
                let lang = self.language.clone();
                let ts = self.timestamps;
                let chunk = self.chunk_secs;
                let dev = self.device.clone();
                // run CLI in background thread, stream to log via channel? simple: blocking run then update
                let mut cmd = std::process::Command::new("kutascribe");
                for f in &files {
                    cmd.arg(f);
                }
                cmd.arg("-o").arg(&outdir)
                    .arg("--model").arg(&model)
                    .arg("--language").arg(&lang)
                    .arg("--chunk-secs").arg(chunk.to_string())
                    .arg("--device").arg(&dev);
                if ts {
                    cmd.arg("--timestamps");
                }
                match cmd.output() {
                    Ok(o) => {
                        let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                        s.push_str(&String::from_utf8_lossy(&o.stderr));
                        self.log = s;
                    }
                    Err(e) => self.log = format!("failed to run `kutascribe` binary: {}\nIs it installed? cargo install --path rust", e),
                }
                self.running = false;
            }
            if self.running {
                ui.spinner();
            }
            ui.separator();
            ui.label("Log:");
            egui::ScrollArea::vertical().max_height(220.).show(ui, |ui| {
                ui.monospace(&self.log);
            });
            ui.small("Distro: Linux AppImage (cargo-appimage) / Windows (cargo-wix .msi) / Mac (.dmg) / brew tap. See README.");
        });
    }
}
