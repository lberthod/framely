//! Framely — enjoliveur de screenshots.
//! Squelette Sprint 0 : layout statique (preview + panneau réglages + barre d'état).
//! La logique (rendu live, capture, IO) arrive aux sprints suivants.

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Framely",
        options,
        Box::new(|_cc| Ok(Box::new(FramelyApp::default()))),
    )
}

#[derive(Default)]
struct FramelyApp {
    padding: u16,
    corner_radius: f32,
    shadow_intensity: f32,
}

impl eframe::App for FramelyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                ui.label("Prêt · aucune image chargée");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("⌘C copier");
                });
            });
        });

        egui::SidePanel::right("settings_panel")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Réglages");
                ui.separator();

                ui.label("Fond");
                ui.label("[dégradés vignettes]");
                ui.add_space(8.0);

                ui.add(egui::Slider::new(&mut self.padding, 0..=200).text("Marge"));
                ui.add(egui::Slider::new(&mut self.corner_radius, 0.0..=48.0).text("Coins"));
                ui.add(egui::Slider::new(&mut self.shadow_intensity, 0.0..=1.0).text("Ombre"));

                ui.separator();
                ui.label("Ratio");
                egui::ComboBox::from_label("")
                    .selected_text("Auto")
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "Auto");
                        let _ = ui.selectable_label(false, "16:9");
                        let _ = ui.selectable_label(false, "1:1");
                        let _ = ui.selectable_label(false, "4:3");
                        let _ = ui.selectable_label(false, "3:2");
                    });

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
                ui.label("Preview live — glisser une image ou ⇧⌘2 pour capturer");
            });
        });
    }
}
