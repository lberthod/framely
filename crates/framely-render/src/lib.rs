//! Pipeline d'enjolivement : fond + marge + coins arrondis + ombre.
//! Squelette Sprint 0 — l'implémentation du pipeline arrive au Sprint 1.

use framely_core::{RawImage, Style};

/// Rendu final : image RGBA8 prête à afficher/exporter.
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Calcule des réglages esthétiques par défaut pour une image donnée.
/// C'est le cœur du "beau par défaut" — voir ARCHITECTURE.md section 6.
pub fn auto_balance(_source: &RawImage) -> Style {
    Style::default()
}

/// Compose l'image source avec le style donné. Stub Sprint 0 : renvoie l'image
/// source telle quelle, sans fond/marge/ombre (implémenté au Sprint 1).
pub fn render(source: &RawImage, _style: &Style) -> RenderedImage {
    RenderedImage {
        width: source.width,
        height: source.height,
        pixels: source.pixels.clone(),
    }
}
