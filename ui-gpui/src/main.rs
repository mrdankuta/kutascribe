use gpui::{div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions};

struct Kutascribe {
    status: SharedString,
    timestamps: bool,
    chunk_secs: u64,
    device: SharedString,
}

impl Render for Kutascribe {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x1e1e2e))
            .size_full()
            .justify_center()
            .items_center()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .text_xl()
                    .child("kutascribe — local speech-to-text"),
            )
            .child(format!(
                "timestamps: {} | chunk: {}s | device: {}",
                if self.timestamps { "on" } else { "off (default)" },
                self.chunk_secs,
                self.device
            ))
            .child(format!("status: {}", self.status))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x45475a))
                            .rounded_md()
                            .child("Drop audio/video here (mp3, wav, mp4, mkv)"),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x89b4fa))
                            .text_color(rgb(0x000000))
                            .rounded_md()
                            .child("Transcribe (calls kutascribe-core)"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa6adc8))
                    .child("CLI already works: kutascribe input.mp3 -o transcripts/ --timestamps --chunk-secs 60 --device auto"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(640.), px(480.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Kutascribe {
                    status: "idle — drag files or use CLI".into(),
                    timestamps: false,
                    chunk_secs: 60,
                    device: "auto".into(),
                })
            },
        )
        .unwrap();
    });
}
