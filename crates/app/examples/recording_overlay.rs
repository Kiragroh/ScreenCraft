//! Exercise the actual recording panel and border without starting any capture.
#![cfg_attr(windows, windows_subsystem = "windows")]
use eframe::egui;
use screencraft::recorder::{Action, Phase, Recorder};
struct Preview {
    displays: Vec<screencraft_capture::DisplayInfo>,
    rec: Recorder,
    prefs: screencraft::prefs::Prefs,
}
impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.heading("ScreenCraft: Paneltest ohne Aufnahme");
        ui.label("Stop schliesst diese Vorschau. Keine Erfassung aktiv.");
        if matches!(self.rec.show(ui, &mut self.prefs), Action::Stop) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(d) = self.displays.iter().find(|d| d.is_main).or(self.displays.first()) {
            screencraft::area::draw_frame(ui.ctx(), screencraft_core::project::Rect {
                x: d.x, y: d.y, width: d.width, height: d.height,
            }, "rec", &self.displays);
        }
    }
}
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("ScreenCraft Paneltest - keine Aufnahme")
            .with_inner_size([480.0, 140.0]).with_transparent(false).with_always_on_top(),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native("ScreenCraft Paneltest", options, Box::new(|cc| {
        screencraft::theme::install(&cc.egui_ctx);
        let prefs = screencraft::prefs::Prefs::default();
        // Do not call refresh/live.sync/controller.start: no screen, microphone or camera capture.
        let mut rec = Recorder::new(&cc.egui_ctx, &prefs);
        rec.phase = Phase::Recording;
        rec.started_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(83));
        Ok(Box::new(Preview { displays: screencraft_capture::list_sources().unwrap_or_default().displays, rec, prefs }))
    }))
}
