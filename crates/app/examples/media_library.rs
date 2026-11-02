//! Test the real library UI against synthetic local fixtures only.
#![cfg_attr(windows, windows_subsystem = "windows")]
use eframe::egui;
struct Preview { library: screencraft::library::Library, prefs: screencraft::prefs::Prefs }
impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.heading("Medientest – ausschließlich Testaufnahmen");
        self.library.show(ui.ctx(), &mut self.prefs, &[], false);
        if !self.library.visible { ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close); }
    }
}
fn main() -> eframe::Result {
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default()
        .with_title("ScreenCraft Medientest").with_inner_size([500., 80.]), ..Default::default() };
    eframe::run_native("ScreenCraft Medientest", options, Box::new(|cc| {
        screencraft::theme::install(&cc.egui_ctx);
        let root = std::env::current_exe().expect("example path").parent().expect("example folder").join("media-test");
        let mut prefs = screencraft::prefs::Prefs::default();
        prefs.recordings_dir = root.clone(); prefs.favorite_folders = vec![root.join("Favorit")];
        let mut library = screencraft::library::Library::default(); library.open();
        Ok(Box::new(Preview { library, prefs }))
    }))
}
