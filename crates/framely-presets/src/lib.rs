//! Presets de dégradés, presets de marque, persistance des réglages utilisateur.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GradientPreset {
    pub id: String,
    pub label: String,
    pub colors: Vec<[u8; 4]>,
}

fn hex(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

/// Catalogue de dégradés préréglés (14 presets choisis pour bien rendre
/// dans la majorité des cas — voir README.md section "beau par défaut").
pub fn builtin_gradients() -> Vec<GradientPreset> {
    vec![
        GradientPreset {
            id: "default".into(),
            label: "Indigo".into(),
            colors: vec![hex(99, 102, 241), hex(236, 72, 153)],
        },
        GradientPreset {
            id: "sunset".into(),
            label: "Coucher de soleil".into(),
            colors: vec![hex(255, 153, 102), hex(255, 94, 98)],
        },
        GradientPreset {
            id: "ocean".into(),
            label: "Océan".into(),
            colors: vec![hex(46, 49, 146), hex(27, 255, 255)],
        },
        GradientPreset {
            id: "purple_love".into(),
            label: "Violet".into(),
            colors: vec![hex(204, 43, 94), hex(117, 58, 136)],
        },
        GradientPreset {
            id: "mango".into(),
            label: "Mangue".into(),
            colors: vec![hex(255, 226, 89), hex(255, 167, 81)],
        },
        GradientPreset {
            id: "aqua_marine".into(),
            label: "Aigue-marine".into(),
            colors: vec![hex(26, 41, 128), hex(38, 208, 206)],
        },
        GradientPreset {
            id: "cherry".into(),
            label: "Cerise".into(),
            colors: vec![hex(235, 51, 73), hex(244, 92, 67)],
        },
        GradientPreset {
            id: "blush".into(),
            label: "Blush".into(),
            colors: vec![hex(221, 94, 137), hex(247, 187, 151)],
        },
        GradientPreset {
            id: "frost".into(),
            label: "Givre".into(),
            colors: vec![hex(0, 4, 40), hex(0, 78, 146)],
        },
        GradientPreset {
            id: "mint".into(),
            label: "Menthe".into(),
            colors: vec![hex(0, 176, 155), hex(150, 201, 61)],
        },
        GradientPreset {
            id: "lavender".into(),
            label: "Lavande".into(),
            colors: vec![hex(127, 0, 255), hex(225, 0, 255)],
        },
        GradientPreset {
            id: "steel".into(),
            label: "Acier".into(),
            colors: vec![hex(35, 37, 38), hex(65, 67, 69)],
        },
        GradientPreset {
            id: "coral".into(),
            label: "Corail".into(),
            colors: vec![hex(255, 81, 47), hex(221, 36, 118)],
        },
        GradientPreset {
            id: "emerald".into(),
            label: "Émeraude".into(),
            colors: vec![hex(52, 143, 80), hex(86, 180, 211)],
        },
    ]
}

/// Renvoie le preset correspondant à l'id, ou le premier preset par défaut.
pub fn gradient_by_id(id: &str) -> GradientPreset {
    let gradients = builtin_gradients();
    gradients
        .iter()
        .find(|g| g.id == id)
        .cloned()
        .unwrap_or_else(|| gradients[0].clone())
}

#[derive(Debug)]
pub enum PresetsError {
    NotImplemented,
}

/// Charge les derniers réglages/dossier/ratio/format persistés.
/// Stub Sprint 0 — implémentation disque au Sprint 3.
pub fn load_last_settings() -> Result<(), PresetsError> {
    Err(PresetsError::NotImplemented)
}
