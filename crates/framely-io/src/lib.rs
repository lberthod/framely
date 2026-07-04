//! Entrées/sorties : presse-papiers, export fichier, import fichier.
//! Le drag-out natif (glisser la preview vers une autre app) est branché
//! séparément dans `framely-app` (Sprint 3, nécessite un pont `objc2` vers
//! `NSDraggingSession`).

use framely_core::RawImage;
use std::path::Path;

#[derive(Debug)]
pub enum IoError {
    ClipboardEmpty,
    ClipboardError(String),
    FileError(String),
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::ClipboardEmpty => write!(f, "presse-papiers vide ou sans image"),
            IoError::ClipboardError(m) => write!(f, "erreur presse-papiers : {m}"),
            IoError::FileError(m) => write!(f, "erreur fichier : {m}"),
        }
    }
}

impl std::error::Error for IoError {}

/// Lit une image depuis le presse-papiers système.
pub fn read_image_from_clipboard() -> Result<RawImage, IoError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| IoError::ClipboardError(e.to_string()))?;
    let image = clipboard.get_image().map_err(|e| match e {
        arboard::Error::ContentNotAvailable => IoError::ClipboardEmpty,
        other => IoError::ClipboardError(other.to_string()),
    })?;

    Ok(RawImage {
        width: image.width as u32,
        height: image.height as u32,
        pixels: image.bytes.into_owned(),
    })
}

/// Copie l'image donnée dans le presse-papiers système (geste ⌘C).
pub fn write_image_to_clipboard(image: &RawImage) -> Result<(), IoError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| IoError::ClipboardError(e.to_string()))?;
    let data = arboard::ImageData {
        width: image.width as usize,
        height: image.height as usize,
        bytes: std::borrow::Cow::Borrowed(&image.pixels),
    };
    clipboard
        .set_image(data)
        .map_err(|e| IoError::ClipboardError(e.to_string()))
}

/// Exporte l'image donnée vers un fichier. Le format est déduit de
/// l'extension du chemin (`.png`, `.jpg`/`.jpeg`).
pub fn export_to_file(image: &RawImage, path: &Path) -> Result<(), IoError> {
    let buffer = image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone())
        .ok_or_else(|| {
            IoError::FileError("dimensions incohérentes avec le buffer de pixels".into())
        })?;
    buffer
        .save(path)
        .map_err(|e| IoError::FileError(e.to_string()))
}

/// Importe une image depuis un fichier (PNG/JPEG). Le HEIC n'est pas
/// supporté par la crate `image` — import HEIC laissé en backlog.
pub fn import_from_file(path: &Path) -> Result<RawImage, IoError> {
    let img = image::open(path).map_err(|e| IoError::FileError(e.to_string()))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RawImage {
        width,
        height,
        pixels: rgba.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_image() -> RawImage {
        let mut pixels = vec![0u8; 4 * 4 * 4];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[10, 20, 30, 255]);
        }
        RawImage {
            width: 4,
            height: 4,
            pixels,
        }
    }

    #[test]
    fn export_then_import_png_roundtrip() {
        let dir = std::env::temp_dir().join(format!("framely-io-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.png");

        let original = sample_image();
        export_to_file(&original, &path).expect("export should succeed");
        let imported = import_from_file(&path).expect("import should succeed");

        assert_eq!(imported.width, original.width);
        assert_eq!(imported.height, original.height);
        assert_eq!(imported.pixels, original.pixels);

        std::fs::remove_dir_all(&dir).ok();
    }
}
