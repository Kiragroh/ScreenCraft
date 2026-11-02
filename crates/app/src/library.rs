//! A persistent, multi-folder library. Moving a whole package to the local
//! application trash preserves all video tracks, input data and project edits.
use crate::prefs::Prefs;
use eframe::egui::{self, ViewportBuilder, ViewportId};
use screencraft_engine::Package;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TRASH: &str = ".ScreenCraft-Trash";

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub modified: u64,
    pub duration_ms: Option<f64>,
    pub camera: bool,
    pub cursor: bool,
    pub warning: bool,
    pub trash: bool,
}

#[derive(Default)]
pub struct Scan { pub entries: Vec<Entry>, pub errors: Vec<String> }

fn is_package(p: &Path) -> bool {
    p.extension().is_some_and(|s| s.eq_ignore_ascii_case("screencraft"))
}

fn entry(path: PathBuf, trash: bool) -> Entry {
    let project = Package::open(&path).load_project().ok();
    let name = project.as_ref().map(|p| p.name.clone())
        .unwrap_or_else(|| path.file_stem().unwrap_or_default().to_string_lossy().into_owned());
    Entry {
        modified: path.metadata().and_then(|m| m.modified()).ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
        duration_ms: project.as_ref().map(|p| p.media.duration_ms),
        camera: project.as_ref().and_then(|p| p.media.camera.as_ref()).is_some_and(|p| path.join(p).is_file()),
        cursor: project.as_ref().is_some_and(|p| !p.media.camera_only && path.join(&p.media.input).is_file()),
        warning: path.join("recording-warnings.txt").is_file(),
        path, name, trash,
    }
}

/// No fixed entry limit. Do not follow directory links or junctions; overlapping
/// roots are canonicalized so each recording appears once.
pub fn scan(roots: &[PathBuf]) -> Scan {
    let mut result = Scan::default();
    let mut visited = HashSet::new();
    let mut pending: Vec<_> = roots.iter().map(|p| (p.clone(), 0usize)).collect();
    while let Some((dir, depth)) = pending.pop() {
        let key = match dir.canonicalize() { Ok(p) => p, Err(e) => {
            result.errors.push(format!("{}: {e}", dir.display())); continue;
        }};
        if !visited.insert(key) { continue; }
        if is_package(&dir) { result.entries.push(entry(dir, false)); continue; }
        let files = match std::fs::read_dir(&dir) { Ok(f) => f, Err(e) => {
            result.errors.push(format!("{}: {e}", dir.display())); continue;
        }};
        for file in files {
            let file = match file { Ok(f) => f, Err(e) => { result.errors.push(e.to_string()); continue; } };
            let p = file.path();
            let Ok(meta) = file.metadata() else { continue };
            if !meta.is_dir() || is_link(&p) { continue; }
            if file.file_name() == TRASH {
                if let Ok(bins) = std::fs::read_dir(p) {
                    for bin in bins.flatten().filter(|b| !is_link(&b.path())) {
                        if let Ok(items) = std::fs::read_dir(bin.path()) {
                            for item in items.flatten() {
                                if is_package(&item.path()) && item.path().is_dir() && !is_link(&item.path()) {
                                    result.entries.push(entry(item.path(), true));
                                }
                            }
                        }
                    }
                }
            } else if depth < 32 {
                pending.push((p, depth + 1));
            } else {
                result.errors.push(format!("Ordner zu tief verschachtelt: {}", p.display()));
            }
        }
    }
    result.entries.sort_by(|a,b| b.modified.cmp(&a.modified).then(a.path.cmp(&b.path)));
    result
}

fn is_link(p: &Path) -> bool {
    let Ok(m) = p.symlink_metadata() else { return true };
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 { return true; }
    }
    m.file_type().is_symlink()
}

pub fn move_to_trash(path: &Path) -> Result<PathBuf, String> {
    if !is_package(path) || !path.is_dir() || is_link(path) {
        return Err("Nur vorhandene ScreenCraft-Aufnahmepakete können gelöscht werden.".into());
    }
    let parent = path.parent().ok_or("Aufnahmeordner fehlt")?;
    let trash = parent.join(TRASH);
    if trash.exists() && (is_link(&trash) || !trash.is_dir()) { return Err("Ungültiger Papierkorbpfad".into()); }
    std::fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let bin = trash.join(format!("{nonce}-{}", std::process::id()));
    std::fs::create_dir(&bin).map_err(|e| e.to_string())?;
    let target = bin.join(path.file_name().ok_or("Aufnahmename fehlt")?);
    if let Err(e) = std::fs::rename(path, &target) {
        let _ = std::fs::remove_dir(&bin); // empty directory created by this operation only
        return Err(format!("Aufnahme konnte nicht verschoben werden: {e}"));
    }
    Ok(target)
}

pub fn restore(path: &Path) -> Result<PathBuf, String> {
    let bin = path.parent().ok_or("Papierkorbpfad fehlt")?;
    let trash = bin.parent().ok_or("Papierkorbpfad fehlt")?;
    if trash.file_name().is_none_or(|n| n != TRASH) || !is_package(path)
        || is_link(path) || is_link(bin) || is_link(trash) {
        return Err("Ungültiger Papierkorbeintrag".into());
    }
    let target = trash.parent().ok_or("Ursprungsordner fehlt")?.join(path.file_name().ok_or("Name fehlt")?);
    if target.exists() { return Err("Am ursprünglichen Ort existiert bereits eine Aufnahme mit diesem Namen.".into()); }
    std::fs::rename(path, &target).map_err(|e| format!("Wiederherstellen fehlgeschlagen: {e}"))?;
    let _ = std::fs::remove_dir(bin); // remove only if empty
    Ok(target)
}

#[derive(Default)]
pub struct Library {
    pub visible: bool,
    data: Scan,
    receiver: Option<Receiver<Scan>>,
    scanned: Option<Instant>,
    query: String,
    folder: Option<PathBuf>,
    trash: bool,
    confirm: Option<Entry>,
    message: Option<String>,
}

impl Library {
    pub fn open(&mut self) { self.visible = true; self.scanned = None; }
    fn refresh(&mut self, prefs: &Prefs, ctx: &egui::Context) {
        if self.receiver.is_some() { return; }
        let roots = prefs.library_roots();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        self.receiver = Some(rx);
        std::thread::spawn(move || { let _ = tx.send(scan(&roots)); ctx.request_repaint(); });
    }
    pub fn show(&mut self, ctx: &egui::Context, prefs: &mut Prefs, protected: &[PathBuf], recording: bool) -> Option<PathBuf> {
        if !self.visible { return None; }
        if let Some(rx) = &self.receiver {
            if let Ok(data) = rx.try_recv() { self.data = data; self.receiver = None; self.scanned = Some(Instant::now()); }
        }
        if self.scanned.is_none_or(|t| t.elapsed() > Duration::from_secs(5)) { self.refresh(prefs, ctx); }
        let mut open = None;
        let mut changed = false;
        ctx.show_viewport_immediate(ViewportId::from_hash_of("media-library"),
            ViewportBuilder::default().with_title("ScreenCraft – Medien").with_inner_size([1050., 700.]).with_min_inner_size([780., 420.]), |ui, _| {
            if ui.ctx().input(|i| i.viewport().close_requested()) { self.visible = false; }
            ui.heading("Medien");
            ui.horizontal(|ui| {
                ui.label("Suche:"); ui.text_edit_singleline(&mut self.query);
                if ui.button("Aktualisieren").clicked() { changed = true; }
                if self.receiver.is_some() { ui.spinner(); ui.label("Ordner werden gelesen …"); }
            });
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(!self.trash && self.folder.is_none(), "Alle Aufnahmen").clicked() { self.folder = None; self.trash = false; }
                if ui.selectable_label(self.trash, "Papierkorb").clicked() { self.trash = true; self.folder = None; }
                if ui.button("Ordner favorisieren …").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                        if !prefs.favorite_folders.contains(&p) { prefs.favorite_folders.push(p); prefs.save(); changed = true; }
                    }
                }
                if ui.button("Aufnahmeordner ändern …").clicked() {
                    if let Some(p) = rfd::FileDialog::new().set_directory(&prefs.recordings_dir).pick_folder() {
                        prefs.set_recordings_dir(p); prefs.save(); changed = true;
                    }
                }
            });
            ui.small(format!("Neue Aufnahmen: {}", prefs.recordings_dir.display()));
            ui.horizontal_wrapped(|ui| {
                ui.label("Favoriten:");
                let mut remove = None;
                for (i, p) in prefs.favorite_folders.iter().enumerate() {
                    let name = p.file_name().unwrap_or(p.as_os_str()).to_string_lossy();
                    if ui.selectable_label(self.folder.as_ref() == Some(p), format!("★ {name}")).on_hover_text(p.display().to_string()).clicked() {
                        self.folder = Some(p.clone()); self.trash = false;
                    }
                    if ui.small_button("×").on_hover_text("Nur Favorit entfernen; Dateien behalten").clicked() { remove = Some(i); }
                }
                if let Some(i) = remove {
                    let removed = prefs.favorite_folders.remove(i);
                    if self.folder.as_ref() == Some(&removed) { self.folder = None; }
                    prefs.save(); changed = true;
                }
            });
            ui.separator();
            if recording { ui.label("Aufnahme läuft: Löschen ist währenddessen gesperrt."); }
            if let Some(message) = &self.message { ui.label(message); }
            if !self.data.errors.is_empty() {
                ui.collapsing(format!("{} Ordnerhinweise", self.data.errors.len()), |ui| { for e in &self.data.errors { ui.label(e); } });
            }
            let query = self.query.to_lowercase();
            let entries: Vec<_> = self.data.entries.iter().filter(|e| e.trash == self.trash
                && self.folder.as_ref().is_none_or(|p| e.path.starts_with(p))
                && (query.is_empty() || e.name.to_lowercase().contains(&query) || e.path.to_string_lossy().to_lowercase().contains(&query))).cloned().collect();
            ui.label(format!("{} Aufnahmen", entries.len()));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for e in entries {
                    ui.push_id(&e.path, |ui| {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.strong(&e.name);
                                if let Some(ms) = e.duration_ms { ui.label(format!("{}:{:02}", (ms / 60000.) as u64, ((ms / 1000.) as u64) % 60)); }
                                else { ui.colored_label(egui::Color32::YELLOW, "Unvollständige Aufnahme"); }
                            });
                            ui.small(e.path.display().to_string());
                            if e.warning {
                                ui.colored_label(egui::Color32::YELLOW, "Aufnahmehinweis vorhanden (recording-warnings.txt)");
                            }
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Video");
                                if e.camera { ui.label("· Webcam separat"); }
                                if e.cursor { ui.label("· Mausdaten separat"); }
                                if e.trash {
                                    if ui.add_enabled(!recording, egui::Button::new("Wiederherstellen")).clicked() {
                                        self.message = Some(match restore(&e.path) { Ok(_) => "Aufnahme wiederhergestellt.".into(), Err(e) => e }); changed = true;
                                    }
                                } else {
                                    if ui.add_enabled(e.duration_ms.is_some(), egui::Button::new("Bearbeiten")).clicked() { open = Some(e.path.clone()); }
                                    if ui.button("Dateien").clicked() { let _ = crate::editor::open_url(&e.path.to_string_lossy()); }
                                    let blocked = recording || protected.iter().any(|p| same_path(p, &e.path));
                                    if ui.add_enabled(!blocked, egui::Button::new("Löschen …")).on_hover_text("Komplette Aufnahme in den ScreenCraft-Papierkorb verschieben. Offene Aufnahmen erst schließen.").clicked() {
                                        self.confirm = Some(e.clone());
                                    }
                                }
                            });
                        });
                    });
                }
            });
            if let Some(e) = self.confirm.clone() {
                egui::Modal::new(egui::Id::new("trash-confirm")).show(ui.ctx(), |ui| {
                    ui.heading("Aufnahme in den Papierkorb verschieben?"); ui.label(&e.name);
                    ui.label("Bildschirm, Webcam, Mausdaten und Bearbeitungen bleiben zusammen und können wiederhergestellt werden.");
                    ui.horizontal(|ui| {
                        if ui.button("Abbrechen").clicked() { self.confirm = None; }
                        if ui.add_enabled(!recording && !protected.iter().any(|p| same_path(p, &e.path)), egui::Button::new("In Papierkorb verschieben")).clicked() {
                            if protected.iter().any(|p| p.canonicalize().ok().zip(e.path.canonicalize().ok()).is_some_and(|(a,b)| a == b)) {
                                self.message = Some("Diese Aufnahme ist noch im Editor geöffnet.".into()); self.confirm = None; return;
                            }
                            self.message = Some(match move_to_trash(&e.path) { Ok(_) => "Aufnahme im Papierkorb. Dort lässt sie sich wiederherstellen.".into(), Err(e) => e });
                            self.confirm = None; changed = true;
                        }
                    });
                });
            }
            ui.ctx().request_repaint_after(Duration::from_millis(500));
        });
        if changed { self.scanned = None; self.refresh(prefs, ctx); }
        open
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) { a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()) } else { a == b }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        let p = std::env::temp_dir().join(format!("screencraft-library-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&p).unwrap(); p
    }
    #[test]
    fn lists_every_recording_across_nested_overlapping_roots_including_incomplete() {
        let root = temp(); let nested = root.join("nested"); std::fs::create_dir(&nested).unwrap();
        for n in 0..12 { let p = Package::create_in(&nested, &n.to_string()).unwrap(); std::fs::write(p.dir.join("screen.mp4"), b"synthetic").unwrap(); }
        let result = scan(&[root.clone(), nested]); assert!(result.errors.is_empty()); assert_eq!(result.entries.len(), 12);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn trash_restore_preserves_all_tracks_and_never_overwrites() {
        let root = temp(); let p = Package::create_in(&root, "test").unwrap();
        for name in ["screen.mp4", "camera.mp4", "input.json", "project.json"] { std::fs::write(p.dir.join(name), name.as_bytes()).unwrap(); }
        let trashed = move_to_trash(&p.dir).unwrap(); assert!(!p.dir.exists());
        let scanned = scan(&[root.clone()]); assert_eq!(scanned.entries.len(), 1); assert!(scanned.entries[0].trash);
        std::fs::create_dir(&p.dir).unwrap(); assert!(restore(&trashed).is_err()); std::fs::remove_dir(&p.dir).unwrap();
        assert_eq!(restore(&trashed).unwrap(), p.dir);
        for name in ["screen.mp4", "camera.mp4", "input.json", "project.json"] { assert_eq!(std::fs::read(p.dir.join(name)).unwrap(), name.as_bytes()); }
        assert!(move_to_trash(&root).is_err()); assert!(restore(&p.dir).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn changing_output_preserves_history_and_old_settings_load() {
        let mut p: Prefs = serde_json::from_str(r#"{"recordings_dir":"old"}"#).unwrap();
        p.set_recordings_dir("new".into()); p.favorite_folders.push("favorite".into());
        assert_eq!(p.library_roots(), vec![PathBuf::from("new"), PathBuf::from("old"), PathBuf::from("favorite")]);
        let read: Prefs = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap(); assert_eq!(p.library_roots(), read.library_roots());
    }
}
