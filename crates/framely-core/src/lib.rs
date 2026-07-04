//! Types partagés du domaine Framely : Document, Style, options de rendu/export.
//! Contenu volontairement minimal au Sprint 0 — la logique arrive au Sprint 1.

use serde::{Deserialize, Serialize};

/// Image source brute (capture ou import), jamais modifiée après acquisition.
#[derive(Clone, Debug)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    /// RGBA8, row-major.
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Background {
    Gradient(String),
    Solid([u8; 4]),
    Transparent,
    Image(String),
}

impl Default for Background {
    fn default() -> Self {
        Background::Gradient("default".to_string())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ShadowParams {
    pub intensity: f32,
    pub blur: f32,
    pub offset_y: f32,
}

impl Default for ShadowParams {
    fn default() -> Self {
        Self {
            intensity: 0.35,
            blur: 24.0,
            offset_y: 12.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Ratio {
    #[default]
    Auto,
    Fixed(u32, u32),
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Scale {
    X1,
    #[default]
    X2,
}

/// État de réglages courant, appliqué à une image source pour produire le rendu final.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    pub background: Background,
    pub padding: u16,
    pub corner_radius: f32,
    pub shadow: ShadowParams,
    pub ratio: Ratio,
    pub scale: Scale,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Background::default(),
            padding: 64,
            corner_radius: 12.0,
            shadow: ShadowParams::default(),
            ratio: Ratio::default(),
            scale: Scale::default(),
        }
    }
}

/// Document ouvert dans l'éditeur : image source + réglages + historique undo/redo.
pub struct Document {
    pub source: RawImage,
    pub style: Style,
    history: Vec<Style>,
    redo_stack: Vec<Style>,
}

impl Document {
    pub fn new(source: RawImage) -> Self {
        Self {
            source,
            style: Style::default(),
            history: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Applique un nouveau style, en poussant l'ancien sur la pile d'annulation.
    pub fn apply_style(&mut self, new_style: Style) {
        self.history.push(self.style.clone());
        self.redo_stack.clear();
        self.style = new_style;
    }

    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.redo_stack.push(self.style.clone());
            self.style = previous;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.history.push(self.style.clone());
            self.style = next;
            true
        } else {
            false
        }
    }

    pub fn reset_to_default(&mut self) {
        self.apply_style(Style::default());
    }

    /// Pousse `previous` sur la pile d'annulation sans changer `self.style`.
    /// Utilisé pour les réglages continus (curseurs) : l'appelant mute déjà
    /// `style` en direct à chaque frame pour la preview, et n'appelle
    /// `commit_history` qu'une fois le geste terminé (relâchement du
    /// curseur), pour ne pousser qu'une seule entrée d'annulation par geste
    /// plutôt qu'une par frame de glissement.
    pub fn commit_history(&mut self, previous: Style) {
        self.history.push(previous);
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_image() -> RawImage {
        RawImage {
            width: 10,
            height: 10,
            pixels: vec![0; 10 * 10 * 4],
        }
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut doc = Document::new(dummy_image());
        let original_padding = doc.style.padding;

        let mut changed = doc.style.clone();
        changed.padding = 128;
        doc.apply_style(changed);
        assert_eq!(doc.style.padding, 128);

        assert!(doc.undo());
        assert_eq!(doc.style.padding, original_padding);

        assert!(doc.redo());
        assert_eq!(doc.style.padding, 128);

        assert!(!doc.redo());
    }

    #[test]
    fn commit_history_supports_drag_style_edits() {
        let mut doc = Document::new(dummy_image());
        let original_padding = doc.style.padding;
        let snapshot = doc.style.clone();

        // Simule un glisser de curseur : mutation directe à chaque frame...
        doc.style.padding = 50;
        doc.style.padding = 90;
        doc.style.padding = 128;
        assert!(!doc.can_undo());

        // ...puis une seule entrée d'annulation au relâchement.
        doc.commit_history(snapshot);
        assert!(doc.can_undo());
        assert!(!doc.can_redo());

        assert!(doc.undo());
        assert_eq!(doc.style.padding, original_padding);
        assert!(doc.can_redo());

        assert!(doc.redo());
        assert_eq!(doc.style.padding, 128);
    }
}
