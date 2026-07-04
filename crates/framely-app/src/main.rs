//! Framely — enjoliveur de screenshots.
//! Sprint 1 : pipeline de rendu réel branché sur la preview live avec une
//! image de démonstration. Capture/IO réels arrivent au Sprint 2.

mod demo_image;

use framely_core::{Background, Document, Ratio, Scale};

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
}

impl eframe::App for FramelyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_preview(ctx);

        egui::TopBottomPanel::top("actions_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Framely");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = ui.button("Export");
                    let _ = ui.button("Copier");
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Prêt · {}×{} · démo",
                    self.document.source.width, self.document.source.height
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("⌘C copier");
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
    }
}
