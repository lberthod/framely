//! Overlay plein écran pour la sélection interactive d'une zone à capturer
//! (glisser-déposer avec dimensions en direct), comme décrit dans le
//! document produit. Repose sur le support multi-viewport natif d'egui
//! (`show_viewport_immediate`), qui crée une vraie fenêtre OS transparente
//! et sans décoration par-dessus l'écran.
//!
//! Limite connue : l'overlay se place sur l'écran principal (celui que
//! `with_fullscreen` choisit par défaut), pas nécessairement celui sous le
//! curseur en configuration multi-écrans — cf. SPRINT.md.

use egui::{Color32, Pos2, Rect, Stroke, ViewportBuilder, ViewportId};

pub enum OverlayOutcome {
    None,
    Cancelled,
    Selected {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        pixels_per_point: f32,
    },
}

pub struct SelectionOverlay {
    active: bool,
    drag_start: Option<Pos2>,
    drag_current: Option<Pos2>,
}

impl SelectionOverlay {
    pub fn new() -> Self {
        Self {
            active: false,
            drag_start: None,
            drag_current: None,
        }
    }

    pub fn open(&mut self) {
        self.active = true;
        self.drag_start = None;
        self.drag_current = None;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// À appeler chaque frame tant que `is_active()` est vrai. Renvoie
    /// `Selected` une fois le glisser-déposer terminé, `Cancelled` sur Échap
    /// ou sélection trop petite, `None` tant que l'utilisateur interagit.
    pub fn show(&mut self, ctx: &egui::Context) -> OverlayOutcome {
        if !self.active {
            return OverlayOutcome::None;
        }

        let mut outcome = OverlayOutcome::None;
        let viewport_id = ViewportId::from_hash_of("framely_selection_overlay");
        let builder = ViewportBuilder::default()
            .with_fullscreen(true)
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_active(true);

        let drag_start = &mut self.drag_start;
        let drag_current = &mut self.drag_current;

        ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                outcome = OverlayOutcome::Cancelled;
            }

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Color32::from_black_alpha(90)))
                .show(ctx, |ui| {
                    let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

                    if response.drag_started() {
                        *drag_start = response.interact_pointer_pos();
                    }
                    if response.dragged() {
                        *drag_current = ctx.pointer_latest_pos().or(*drag_current);
                    }

                    if let (Some(start), Some(current)) = (*drag_start, *drag_current) {
                        let rect = Rect::from_two_pos(start, current);
                        ui.painter()
                            .rect_filled(rect, 0.0, Color32::from_white_alpha(24));
                        ui.painter()
                            .rect_stroke(rect, 0.0, Stroke::new(2.0, Color32::WHITE));

                        let label = format!("{:.0} × {:.0}", rect.width(), rect.height());
                        ui.painter().text(
                            rect.left_top() + egui::vec2(4.0, -4.0),
                            egui::Align2::LEFT_BOTTOM,
                            label,
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                    }

                    if response.drag_stopped() {
                        outcome = match (*drag_start, *drag_current) {
                            (Some(start), Some(current)) => {
                                let rect = Rect::from_two_pos(start, current);
                                if rect.width() >= 4.0 && rect.height() >= 4.0 {
                                    OverlayOutcome::Selected {
                                        x: rect.min.x as f64,
                                        y: rect.min.y as f64,
                                        width: rect.width() as f64,
                                        height: rect.height() as f64,
                                        pixels_per_point: ctx.pixels_per_point(),
                                    }
                                } else {
                                    OverlayOutcome::Cancelled
                                }
                            }
                            _ => OverlayOutcome::Cancelled,
                        };
                    }
                });
        });

        if !matches!(outcome, OverlayOutcome::None) {
            self.active = false;
            self.drag_start = None;
            self.drag_current = None;
        }

        outcome
    }
}
