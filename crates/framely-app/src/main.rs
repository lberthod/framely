//! Framely — enjoliveur de screenshots.
//! Sprint 2 : capture d'écran réelle (ScreenCaptureKit), presse-papiers,
//! raccourcis globaux et menu bar. La sélection interactive d'une zone par
//! glisser-déposer (overlay plein écran) est un chantier à part entière,
//! laissé en backlog — voir SPRINT.md ; "Capturer une zone" capture pour
//! l'instant l'écran principal en entier.

mod demo_image;
mod system_integration;

use framely_core::{Background, Document, Ratio, Scale};
use std::time::Duration;
use system_integration::{SystemAction, SystemIntegration};

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
}

impl FramelyApp {
    fn new() -> Self {
        let source = demo_image::generate(900, 560);
        let mut document = Document::new(source);
        document.style = framely_render::auto_balance(&document.source);

        Self {
            document,
            texture: None,
            dirty: true,
            status: "Prêt".to_string(),
            system: SystemIntegration::new(),
            window_picker_open: false,
            available_windows: Vec::new(),
        }
    }

    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if !self.dirty {
            return;
        }
        if let Some(rendered) = framely_render::render(&self.document.source, &self.document.style)
        {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [rendered.width as usize, rendered.height as usize],
                &rendered.pixels,
            );
            self.texture = Some(ctx.load_texture("preview", image, egui::TextureOptions::LINEAR));
        }
        self.dirty = false;
    }

    fn load_source(&mut self, source: framely_core::RawImage) {
        self.document = Document::new(source);
        self.document.style = framely_render::auto_balance(&self.document.source);
        self.dirty = true;
    }

    fn handle_system_action(&mut self, action: SystemAction, ctx: &egui::Context) {
        match action {
            SystemAction::CaptureZone => self.capture_zone(),
            SystemAction::CaptureWindow => self.open_window_picker(),
            SystemAction::Paste => self.paste_from_clipboard(),
            SystemAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn capture_zone(&mut self) {
        match framely_capture::capture_display(None) {
            Ok(image) => {
                self.load_source(image);
                self.status = "Capture d'écran effectuée".to_string();
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

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("framely.png")
            .add_filter("PNG", &["png"])
            .save_file()
        {
            match framely_io::export_to_file(&image, &path) {
                Ok(()) => self.status = format!("Exporté vers {}", path.display()),
                Err(e) => self.status = format!("Export impossible : {e}"),
            }
        }
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
                    ui.label("⌘C copier · ⇧⌘2 capturer · ⌘V coller");
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
                                self.document.style.background =
                                    Background::Gradient(preset.id.clone());
                                self.dirty = true;
                            }
                        }
                    });
                ui.add_space(8.0);

                let mut padding = self.document.style.padding;
                if ui
                    .add(egui::Slider::new(&mut padding, 0..=200).text("Marge"))
                    .changed()
                {
                    self.document.style.padding = padding;
                    self.dirty = true;
                }

                let mut corner_radius = self.document.style.corner_radius;
                if ui
                    .add(egui::Slider::new(&mut corner_radius, 0.0..=48.0).text("Coins"))
                    .changed()
                {
                    self.document.style.corner_radius = corner_radius;
                    self.dirty = true;
                }

                let mut shadow_intensity = self.document.style.shadow.intensity;
                if ui
                    .add(egui::Slider::new(&mut shadow_intensity, 0.0..=1.0).text("Ombre"))
                    .changed()
                {
                    self.document.style.shadow.intensity = shadow_intensity;
                    self.dirty = true;
                }

                ui.separator();
                ui.label("Ratio");
                let ratio_label = match self.document.style.ratio {
                    Ratio::Auto => "Auto",
                    Ratio::Fixed(16, 9) => "16:9",
                    Ratio::Fixed(1, 1) => "1:1",
                    Ratio::Fixed(4, 3) => "4:3",
                    Ratio::Fixed(3, 2) => "3:2",
                    Ratio::Fixed(_, _) => "Personnalisé",
                };
                egui::ComboBox::from_id_salt("ratio_picker")
                    .selected_text(ratio_label)
                    .show_ui(ui, |ui| {
                        for (label, ratio) in [
                            ("Auto", Ratio::Auto),
                            ("16:9", Ratio::Fixed(16, 9)),
                            ("1:1", Ratio::Fixed(1, 1)),
                            ("4:3", Ratio::Fixed(4, 3)),
                            ("3:2", Ratio::Fixed(3, 2)),
                        ] {
                            if ui.selectable_label(ratio_label == label, label).clicked() {
                                self.document.style.ratio = ratio;
                                self.dirty = true;
                            }
                        }
                    });

                ui.separator();
                let mut scale_x2 = matches!(self.document.style.scale, Scale::X2);
                if ui.checkbox(&mut scale_x2, "Export @2x").changed() {
                    self.document.style.scale = if scale_x2 { Scale::X2 } else { Scale::X1 };
                    self.dirty = true;
                }

                ui.separator();
                if ui.button("Réinitialiser (⌘R)").clicked() {
                    self.document.style = framely_render::auto_balance(&self.document.source);
                    self.dirty = true;
                }

                ui.separator();
                if ui.button("Capturer une zone (⇧⌘2)").clicked() {
                    self.capture_zone();
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
