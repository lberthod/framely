//! Framely — enjoliveur de screenshots.
//! Sprint 3 : undo/redo, raccourcis clavier complets, persistance des
//! réglages, ratios réseaux sociaux, damier de transparence et downscale
//! de preview pour les très grandes images.

mod demo_image;
mod overlay;
mod preview;
mod system_integration;

use framely_core::{Background, Document, Ratio, Scale, Style};
use overlay::{OverlayOutcome, SelectionOverlay};
use std::time::Duration;
use system_integration::{SystemAction, SystemIntegration};

/// Preview downscalée à cette taille max (px) pour les très grandes
/// captures (5K/6K) — l'export continue de travailler en pleine résolution.
const MAX_PREVIEW_DIM: u32 = 2200;

const RATIO_PRESETS: &[(&str, Ratio)] = &[
    ("Auto", Ratio::Auto),
    ("16:9", Ratio::Fixed(16, 9)),
    ("1:1", Ratio::Fixed(1, 1)),
    ("4:3", Ratio::Fixed(4, 3)),
    ("3:2", Ratio::Fixed(3, 2)),
    ("X / Twitter", Ratio::Fixed(16, 9)),
    ("Instagram", Ratio::Fixed(1, 1)),
    ("LinkedIn", Ratio::Fixed(1200, 627)),
];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Framely",
        options,
        Box::new(|_cc| Ok(Box::new(FramelyApp::new()))),
    )
}

struct FramelyApp {
    document: Document,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    status: String,
    system: SystemIntegration,
    window_picker_open: bool,
    available_windows: Vec<framely_capture::WindowInfo>,
    selection_overlay: SelectionOverlay,
    settings: framely_presets::AppSettings,
    /// Snapshot du style pris au début d'un glisser de curseur, pour ne
    /// pousser qu'une seule entrée d'annulation par geste (voir
    /// `Document::commit_history`).
    slider_edit_snapshot: Option<Style>,
}

impl FramelyApp {
    fn new() -> Self {
        let settings = framely_presets::load_settings();
        let source = demo_image::generate(900, 560);
        let mut document = Document::new(source);
        document.style = framely_render::auto_balance(&document.source);
        document.style.ratio = settings.last_ratio;
        document.style.scale = settings.last_scale;
        document.style.background = Background::Gradient(settings.last_background_id.clone());

        Self {
            document,
            texture: None,
            dirty: true,
            status: "Prêt".to_string(),
            system: SystemIntegration::new(),
            window_picker_open: false,
            available_windows: Vec::new(),
            selection_overlay: SelectionOverlay::new(),
            settings,
            slider_edit_snapshot: None,
        }
    }

    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if !self.dirty {
            return;
        }
        if let Some(rendered) = framely_render::render(&self.document.source, &self.document.style)
        {
            let pixels = if matches!(self.document.style.background, Background::Transparent) {
                preview::composite_on_checkerboard(
                    rendered.width,
                    rendered.height,
                    &rendered.pixels,
                )
            } else {
                rendered.pixels
            };
            let (width, height, pixels) = preview::downscale_for_preview(
                rendered.width,
                rendered.height,
                &pixels,
                MAX_PREVIEW_DIM,
            );

            let image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &pixels,
            );
            self.texture = Some(ctx.load_texture("preview", image, egui::TextureOptions::LINEAR));
        }
        self.dirty = false;
    }

    fn load_source(&mut self, source: framely_core::RawImage) {
        self.document = Document::new(source);
        self.document.style = framely_render::auto_balance(&self.document.source);
        self.document.style.ratio = self.settings.last_ratio;
        self.document.style.scale = self.settings.last_scale;
        self.document.style.background =
            Background::Gradient(self.settings.last_background_id.clone());
        self.dirty = true;
    }

    /// Applique une modification discrète (choix de fond/ratio, case à
    /// cocher, bouton) en poussant l'état précédent sur la pile d'annulation.
    fn edit_style(&mut self, f: impl FnOnce(&mut Style)) {
        let mut new_style = self.document.style.clone();
        f(&mut new_style);
        self.document.apply_style(new_style);
        self.dirty = true;
    }

    fn persist_settings(&mut self) {
        self.settings.last_ratio = self.document.style.ratio;
        self.settings.last_scale = self.document.style.scale;
        if let Background::Gradient(id) = &self.document.style.background {
            self.settings.last_background_id = id.clone();
        }
        if let Err(e) = framely_presets::save_settings(&self.settings) {
            self.status = format!("Réglages non sauvegardés : {e}");
        }
    }

    fn handle_system_action(&mut self, action: SystemAction, ctx: &egui::Context) {
        match action {
            SystemAction::CaptureZone => self.start_zone_selection(),
            SystemAction::CaptureWindow => self.open_window_picker(),
            SystemAction::Paste => self.paste_from_clipboard(),
            SystemAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn start_zone_selection(&mut self) {
        self.selection_overlay.open();
        self.status = "Glissez pour sélectionner une zone (Échap pour annuler)".to_string();
    }

    fn capture_region(&mut self, x: f64, y: f64, width: f64, height: f64, scale: f32) {
        match framely_capture::capture_region(None, x, y, width, height, scale as f64) {
            Ok(image) => {
                self.load_source(image);
                self.status = "Zone capturée".to_string();
            }
            Err(e) => self.status = format!("Capture impossible : {e}"),
        }
    }

    fn open_window_picker(&mut self) {
        match framely_capture::list_windows() {
            Ok(windows) => {
                self.available_windows = windows;
                self.window_picker_open = true;
            }
            Err(e) => self.status = format!("Liste des fenêtres indisponible : {e}"),
        }
    }

    fn capture_window(&mut self, window_id: u32) {
        match framely_capture::capture_window(window_id) {
            Ok(image) => {
                self.load_source(image);
                self.status = "Fenêtre capturée".to_string();
            }
            Err(e) => self.status = format!("Capture de fenêtre impossible : {e}"),
        }
        self.window_picker_open = false;
    }

    fn paste_from_clipboard(&mut self) {
        match framely_io::read_image_from_clipboard() {
            Ok(image) => {
                self.load_source(image);
                self.status = "Image collée depuis le presse-papiers".to_string();
            }
            Err(e) => self.status = format!("Presse-papiers : {e}"),
        }
    }

    fn copy_result_to_clipboard(&mut self) {
        match framely_render::render(&self.document.source, &self.document.style) {
            Some(rendered) => {
                let image = framely_core::RawImage {
                    width: rendered.width,
                    height: rendered.height,
                    pixels: rendered.pixels,
                };
                match framely_io::write_image_to_clipboard(&image) {
                    Ok(()) => self.status = "Copié dans le presse-papiers".to_string(),
                    Err(e) => self.status = format!("Copie impossible : {e}"),
                }
            }
            None => self.status = "Rien à copier".to_string(),
        }
    }

    fn export_result_to_file(&mut self) {
        let Some(rendered) = framely_render::render(&self.document.source, &self.document.style)
        else {
            self.status = "Rien à exporter".to_string();
            return;
        };
        let image = framely_core::RawImage {
            width: rendered.width,
            height: rendered.height,
            pixels: rendered.pixels,
        };

        let mut dialog = rfd::FileDialog::new()
            .set_file_name("framely.png")
            .add_filter("PNG", &["png"]);
        if let Some(dir) = &self.settings.last_export_dir {
            dialog = dialog.set_directory(dir);
        }

        if let Some(path) = dialog.save_file() {
            match framely_io::export_to_file(&image, &path) {
                Ok(()) => {
                    self.status = format!("Exporté vers {}", path.display());
                    if let Some(parent) = path.parent() {
                        self.settings.last_export_dir = Some(parent.to_path_buf());
                        self.persist_settings();
                    }
                }
                Err(e) => self.status = format!("Export impossible : {e}"),
            }
        }
    }

    fn cycle_background(&mut self, direction: i32) {
        let gradients = framely_presets::builtin_gradients();
        let current_idx = match &self.document.style.background {
            Background::Gradient(id) => gradients.iter().position(|g| &g.id == id).unwrap_or(0),
            _ => 0,
        };
        let len = gradients.len() as i32;
        let next_idx = ((current_idx as i32 + direction).rem_euclid(len)) as usize;
        let next_id = gradients[next_idx].id.clone();
        self.edit_style(|style| style.background = Background::Gradient(next_id));
        self.persist_settings();
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};

        let undo = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
        let redo = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
        let reset = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);
        let copy = KeyboardShortcut::new(Modifiers::COMMAND, Key::C);
        let paste = KeyboardShortcut::new(Modifiers::COMMAND, Key::V);
        let export = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

        ctx.input_mut(|i| {
            if i.consume_shortcut(&redo) {
                if self.document.redo() {
                    self.dirty = true;
                    self.status = "Rétabli".to_string();
                }
            } else if i.consume_shortcut(&undo) && self.document.undo() {
                self.dirty = true;
                self.status = "Annulé".to_string();
            }
            if i.consume_shortcut(&reset) {
                self.document.reset_to_default();
                self.dirty = true;
                self.status = "Réinitialisé".to_string();
            }
            if i.consume_shortcut(&copy) {
                self.copy_result_to_clipboard();
            }
            if i.consume_shortcut(&paste) {
                self.paste_from_clipboard();
            }
            if i.consume_shortcut(&export) {
                self.export_result_to_file();
            }
            if i.key_pressed(Key::ArrowRight) {
                self.cycle_background(1);
            }
            if i.key_pressed(Key::ArrowLeft) {
                self.cycle_background(-1);
            }
        });
    }
}

impl eframe::App for FramelyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Nécessaire pour continuer à traiter les raccourcis globaux et les
        // clics du menu bar même quand la fenêtre principale n'a pas le
        // focus (l'app doit rester réactive en tâche de fond).
        ctx.request_repaint_after(Duration::from_millis(100));

        if let Some(action) = self.system.poll() {
            self.handle_system_action(action, ctx);
        }

        if self.selection_overlay.is_active() {
            match self.selection_overlay.show(ctx) {
                OverlayOutcome::Selected {
                    x,
                    y,
                    width,
                    height,
                    pixels_per_point,
                } => self.capture_region(x, y, width, height, pixels_per_point),
                OverlayOutcome::Cancelled => {
                    self.status = "Sélection annulée".to_string();
                }
                OverlayOutcome::None => {}
            }
        }

        self.handle_keyboard_shortcuts(ctx);
        self.refresh_preview(ctx);

        egui::TopBottomPanel::top("actions_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Framely");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export").clicked() {
                        self.export_result_to_file();
                    }
                    if ui.button("Copier").clicked() {
                        self.copy_result_to_clipboard();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} · {}×{}",
                    self.status, self.document.source.width, self.document.source.height
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("⌘C copier · ⇧⌘2 capturer · ⌘V coller · ⌘Z annuler");
                });
            });
        });

        egui::SidePanel::right("settings_panel")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Réglages");
                ui.separator();

                ui.label("Fond");
                let gradients = framely_presets::builtin_gradients();
                let current_label = match &self.document.style.background {
                    Background::Gradient(id) => gradients
                        .iter()
                        .find(|g| &g.id == id)
                        .map(|g| g.label.clone())
                        .unwrap_or_else(|| "Dégradé".to_string()),
                    Background::Solid(_) => "Couleur unie".to_string(),
                    Background::Transparent => "Transparent".to_string(),
                    Background::Image(_) => "Image".to_string(),
                };
                egui::ComboBox::from_id_salt("background_picker")
                    .selected_text(current_label)
                    .show_ui(ui, |ui| {
                        for preset in &gradients {
                            let selected = matches!(
                                &self.document.style.background,
                                Background::Gradient(id) if id == &preset.id
                            );
                            if ui.selectable_label(selected, &preset.label).clicked() {
                                let id = preset.id.clone();
                                self.edit_style(|style| {
                                    style.background = Background::Gradient(id)
                                });
                                self.persist_settings();
                            }
                        }
                        let transparent_selected =
                            matches!(self.document.style.background, Background::Transparent);
                        if ui
                            .selectable_label(transparent_selected, "Transparent")
                            .clicked()
                        {
                            self.edit_style(|style| style.background = Background::Transparent);
                            self.persist_settings();
                        }
                    });
                ui.label("← / → pour parcourir les dégradés");
                ui.add_space(8.0);

                let mut padding = self.document.style.padding;
                let resp = ui.add(egui::Slider::new(&mut padding, 0..=200).text("Marge"));
                if resp.drag_started() {
                    self.slider_edit_snapshot = Some(self.document.style.clone());
                }
                if resp.changed() {
                    self.document.style.padding = padding;
                    self.dirty = true;
                }
                if resp.drag_stopped() {
                    if let Some(prev) = self.slider_edit_snapshot.take() {
                        self.document.commit_history(prev);
                    }
                }

                let mut corner_radius = self.document.style.corner_radius;
                let resp = ui.add(egui::Slider::new(&mut corner_radius, 0.0..=48.0).text("Coins"));
                if resp.drag_started() {
                    self.slider_edit_snapshot = Some(self.document.style.clone());
                }
                if resp.changed() {
                    self.document.style.corner_radius = corner_radius;
                    self.dirty = true;
                }
                if resp.drag_stopped() {
                    if let Some(prev) = self.slider_edit_snapshot.take() {
                        self.document.commit_history(prev);
                    }
                }

                let mut shadow_intensity = self.document.style.shadow.intensity;
                let resp =
                    ui.add(egui::Slider::new(&mut shadow_intensity, 0.0..=1.0).text("Ombre"));
                if resp.drag_started() {
                    self.slider_edit_snapshot = Some(self.document.style.clone());
                }
                if resp.changed() {
                    self.document.style.shadow.intensity = shadow_intensity;
                    self.dirty = true;
                }
                if resp.drag_stopped() {
                    if let Some(prev) = self.slider_edit_snapshot.take() {
                        self.document.commit_history(prev);
                    }
                }

                ui.separator();
                ui.label("Ratio");
                let ratio_label = RATIO_PRESETS
                    .iter()
                    .find(|(_, r)| ratios_equal(*r, self.document.style.ratio))
                    .map(|(label, _)| *label)
                    .unwrap_or("Personnalisé");
                egui::ComboBox::from_id_salt("ratio_picker")
                    .selected_text(ratio_label)
                    .show_ui(ui, |ui| {
                        for (label, ratio) in RATIO_PRESETS {
                            let selected = ratios_equal(*ratio, self.document.style.ratio);
                            if ui.selectable_label(selected, *label).clicked() {
                                let ratio = *ratio;
                                self.edit_style(|style| style.ratio = ratio);
                                self.persist_settings();
                            }
                        }
                    });

                ui.separator();
                let mut scale_x2 = matches!(self.document.style.scale, Scale::X2);
                if ui.checkbox(&mut scale_x2, "Export @2x").changed() {
                    let scale = if scale_x2 { Scale::X2 } else { Scale::X1 };
                    self.edit_style(|style| style.scale = scale);
                    self.persist_settings();
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.document.can_undo(), egui::Button::new("Annuler (⌘Z)"))
                        .clicked()
                        && self.document.undo()
                    {
                        self.dirty = true;
                    }
                    if ui
                        .add_enabled(
                            self.document.can_redo(),
                            egui::Button::new("Rétablir (⇧⌘Z)"),
                        )
                        .clicked()
                        && self.document.redo()
                    {
                        self.dirty = true;
                    }
                });
                if ui.button("Réinitialiser (⌘R)").clicked() {
                    self.document.reset_to_default();
                    self.dirty = true;
                }

                ui.separator();
                if ui.button("Capturer une zone (⇧⌘2)").clicked() {
                    self.start_zone_selection();
                }
                if ui.button("Capturer une fenêtre (⇧⌘4)").clicked() {
                    self.open_window_picker();
                }
                if ui.button("Coller (⌘V)").clicked() {
                    self.paste_from_clipboard();
                }

                ui.separator();
                ui.label("Annoter");
                ui.horizontal(|ui| {
                    let _ = ui.button("✎");
                    let _ = ui.button("▢");
                    let _ = ui.button("→");
                    let _ = ui.button("🅰");
                    let _ = ui.button("▨");
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                if let Some(texture) = &self.texture {
                    let available = ui.available_size();
                    let tex_size = texture.size_vec2();
                    let scale = (available.x / tex_size.x)
                        .min(available.y / tex_size.y)
                        .min(1.0);
                    ui.image((texture.id(), tex_size * scale));
                } else {
                    ui.label("Preview live — glisser une image ou ⇧⌘2 pour capturer");
                }
            });
        });

        if self.window_picker_open {
            let mut open = self.window_picker_open;
            let mut chosen: Option<u32> = None;
            egui::Window::new("Choisir une fenêtre à capturer")
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for window in &self.available_windows {
                                let label = if window.title.is_empty() {
                                    window.app_name.clone()
                                } else {
                                    format!("{} — {}", window.app_name, window.title)
                                };
                                if ui.button(label).clicked() {
                                    chosen = Some(window.id);
                                }
                            }
                        });
                });
            self.window_picker_open = open;
            if let Some(id) = chosen {
                self.capture_window(id);
            }
        }
    }
}

fn ratios_equal(a: Ratio, b: Ratio) -> bool {
    match (a, b) {
        (Ratio::Auto, Ratio::Auto) => true,
        (Ratio::Fixed(aw, ah), Ratio::Fixed(bw, bh)) => aw == bw && ah == bh,
        _ => false,
    }
}
