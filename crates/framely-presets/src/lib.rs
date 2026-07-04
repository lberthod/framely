//! Presets de dégradés, presets de marque, persistance des réglages utilisateur.
//! Squelette Sprint 0 — le catalogue de presets et la persistance disque arrivent
//! aux Sprints 1/3.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GradientPreset {
    pub id: String,
    pub label: String,
    pub colors: Vec<[u8; 4]>,
}

/// Catalogue de dégradés préréglés. Rempli au Sprint 1 (12-16 presets validés).
pub fn builtin_gradients() -> Vec<GradientPreset> {
    vec![GradientPreset {
        id: "default".to_string(),
        label: "Défaut".to_string(),
        colors: vec![[99, 102, 241, 255], [236, 72, 153, 255]],
    }]
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
