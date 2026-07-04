//! Entrées/sorties : presse-papiers, export fichier, drag-out, import.
//! Squelette Sprint 0 — implémentation réelle prévue au Sprint 2.

use framely_core::RawImage;
use std::path::Path;

#[derive(Debug)]
pub enum IoError {
    ClipboardEmpty,
    NotImplemented,
}

pub fn read_image_from_clipboard() -> Result<RawImage, IoError> {
    Err(IoError::NotImplemented)
}

pub fn write_image_to_clipboard(_image: &RawImage) -> Result<(), IoError> {
    Err(IoError::NotImplemented)
}

pub fn export_to_file(_image: &RawImage, _path: &Path) -> Result<(), IoError> {
    Err(IoError::NotImplemented)
}

pub fn import_from_file(_path: &Path) -> Result<RawImage, IoError> {
    Err(IoError::NotImplemented)
}
