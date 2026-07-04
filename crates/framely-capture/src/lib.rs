//! Capture d'écran (zone / fenêtre) via ScreenCaptureKit.
//! Squelette Sprint 0 — intégration ScreenCaptureKit prévue au Sprint 2.

use framely_core::RawImage;

#[derive(Debug)]
pub enum CaptureError {
    PermissionDenied,
    NotImplemented,
}

pub enum CaptureKind {
    Region,
    Window,
}

/// Point d'entrée de capture. Stub Sprint 0 : renvoie `NotImplemented`,
/// le binding ScreenCaptureKit arrive au Sprint 2 (voir SPRINT.md).
pub fn capture(_kind: CaptureKind) -> Result<RawImage, CaptureError> {
    Err(CaptureError::NotImplemented)
}
