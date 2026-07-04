//! Presets de dégradés, presets de marque, persistance des réglages utilisateur.

use framely_core::{Ratio, Scale};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// Réglages qui survivent entre deux lancements de l'app : dernier dossier
/// d'export, dernier ratio/échelle/fond choisis explicitement par
/// l'utilisateur (l'auto-balance continue de s'appliquer à chaque nouvelle
/// image, mais ces trois préférences priment une fois l'image chargée).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub last_export_dir: Option<PathBuf>,
    pub last_ratio: Ratio,
    pub last_scale: Scale,
    pub last_background_id: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_export_dir: None,
            last_ratio: Ratio::default(),
            last_scale: Scale::default(),
            last_background_id: "default".to_string(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Framely")
            .join("settings.json"),
    )
}

/// Charge les derniers réglages persistés, ou les valeurs par défaut si
/// aucun fichier n'existe encore (premier lancement) ou s'il est illisible.
pub fn load_settings() -> AppSettings {
    settings_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Persiste les réglages sur disque, dans
/// `~/Library/Application Support/Framely/settings.json`.
pub fn save_settings(settings: &AppSettings) -> std::io::Result<()> {
    let path = settings_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "dossier utilisateur introuvable",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_by_id_falls_back_to_first_preset() {
        let known = gradient_by_id("ocean");
        assert_eq!(known.id, "ocean");

        let unknown = gradient_by_id("does-not-exist");
        assert_eq!(unknown.id, builtin_gradients()[0].id);
    }

    #[test]
    fn settings_roundtrip_through_json() {
        let settings = AppSettings {
            last_export_dir: Some(PathBuf::from("/tmp/framely-exports")),
            last_ratio: Ratio::Fixed(16, 9),
            last_scale: Scale::X1,
            last_background_id: "ocean".to_string(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.last_export_dir, settings.last_export_dir);
        assert_eq!(restored.last_background_id, settings.last_background_id);
    }
}
