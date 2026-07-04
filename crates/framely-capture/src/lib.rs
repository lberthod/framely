//! Capture d'écran (écran entier / fenêtre / zone) via ScreenCaptureKit.
//!
//! La sélection interactive au clavier/souris (overlay plein écran avec
//! glisser-déposer et dimensions en direct) n'est pas incluse ici : ce module
//! expose les primitives de capture, l'overlay UI est branché séparément
//! dans `framely-app`.

use framely_core::RawImage;
use screencapturekit::error::SCError;
use screencapturekit::screenshot_manager::{CGImageExt, SCScreenshotManager};
use screencapturekit::shareable_content::{SCShareableContent, SCWindow};
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;

#[derive(Debug)]
pub enum CaptureError {
    /// La permission "Enregistrement de l'écran" n'a pas été accordée.
    PermissionDenied(String),
    NotFound(String),
    Failed(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::PermissionDenied(m) => write!(f, "permission refusée : {m}"),
            CaptureError::NotFound(m) => write!(f, "introuvable : {m}"),
            CaptureError::Failed(m) => write!(f, "échec de capture : {m}"),
        }
    }
}

impl std::error::Error for CaptureError {}

fn map_sc_error(err: SCError) -> CaptureError {
    match err {
        SCError::PermissionDenied(m) | SCError::NoShareableContent(m) => {
            CaptureError::PermissionDenied(m)
        }
        SCError::DisplayNotFound(m) | SCError::WindowNotFound(m) => CaptureError::NotFound(m),
        other => CaptureError::Failed(other.to_string()),
    }
}

#[derive(Clone, Debug)]
pub struct DisplayInfo {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub app_name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Liste les écrans disponibles pour la capture.
pub fn list_displays() -> Result<Vec<DisplayInfo>, CaptureError> {
    let content = SCShareableContent::get().map_err(map_sc_error)?;
    Ok(content
        .displays()
        .iter()
        .map(|d| DisplayInfo {
            id: d.display_id(),
            width: d.width(),
            height: d.height(),
        })
        .collect())
}

/// Liste les fenêtres visibles à l'écran, capturables individuellement.
pub fn list_windows() -> Result<Vec<WindowInfo>, CaptureError> {
    let content = SCShareableContent::get().map_err(map_sc_error)?;
    Ok(content
        .windows()
        .into_iter()
        .filter(|w| w.is_on_screen())
        .map(window_info)
        .collect())
}

fn window_info(window: SCWindow) -> WindowInfo {
    let frame = window.frame();
    let app_name = window
        .owning_application()
        .map(|a| a.application_name())
        .unwrap_or_default();
    WindowInfo {
        id: window.window_id(),
        title: window.title().unwrap_or_default(),
        app_name,
        x: frame.origin.x,
        y: frame.origin.y,
        width: frame.size.width,
        height: frame.size.height,
    }
}

/// Capture l'écran entier. `display_id` sélectionne un écran précis
/// (obtenu via [`list_displays`]) ; `None` capture l'écran principal.
pub fn capture_display(display_id: Option<u32>) -> Result<RawImage, CaptureError> {
    let content = SCShareableContent::get().map_err(map_sc_error)?;
    let displays = content.displays();
    let display = match display_id {
        Some(id) => displays
            .iter()
            .find(|d| d.display_id() == id)
            .ok_or_else(|| CaptureError::NotFound(format!("écran {id}")))?,
        None => displays
            .first()
            .ok_or_else(|| CaptureError::NotFound("aucun écran disponible".into()))?,
    };

    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(display.width())
        .with_height(display.height());

    let image = SCScreenshotManager::capture_image(&filter, &config).map_err(map_sc_error)?;
    cgimage_to_raw(&image)
}

/// Capture une fenêtre précise, identifiée par son id (voir [`list_windows`]).
pub fn capture_window(window_id: u32) -> Result<RawImage, CaptureError> {
    let content = SCShareableContent::get().map_err(map_sc_error)?;
    let window = content
        .windows()
        .into_iter()
        .find(|w| w.window_id() == window_id)
        .ok_or_else(|| CaptureError::NotFound(format!("fenêtre {window_id}")))?;

    let frame = window.frame();
    let filter = SCContentFilter::create().with_window(&window).build();
    let config = SCStreamConfiguration::new()
        .with_width(frame.size.width.round() as u32)
        .with_height(frame.size.height.round() as u32);

    let image = SCScreenshotManager::capture_image(&filter, &config).map_err(map_sc_error)?;
    cgimage_to_raw(&image)
}

/// Capture une zone rectangulaire de l'écran donné. `x`/`y`/`width`/`height`
/// sont en points (comme les coordonnées d'une fenêtre ou d'un overlay de
/// sélection egui) ; `scale` (typiquement le `pixels_per_point` de l'écran,
/// 2.0 en Retina) détermine la résolution de sortie en pixels, pour un
/// export net.
pub fn capture_region(
    display_id: Option<u32>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scale: f64,
) -> Result<RawImage, CaptureError> {
    let content = SCShareableContent::get().map_err(map_sc_error)?;
    let displays = content.displays();
    let display = match display_id {
        Some(id) => displays
            .iter()
            .find(|d| d.display_id() == id)
            .ok_or_else(|| CaptureError::NotFound(format!("écran {id}")))?,
        None => displays
            .first()
            .ok_or_else(|| CaptureError::NotFound("aucun écran disponible".into()))?,
    };

    let rect = screencapturekit::cg::CGRect::new(x, y, width, height);
    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build()
        .set_content_rect(rect);

    let config = SCStreamConfiguration::new()
        .with_width((width * scale).round() as u32)
        .with_height((height * scale).round() as u32);

    let image = SCScreenshotManager::capture_image(&filter, &config).map_err(map_sc_error)?;
    cgimage_to_raw(&image)
}

fn cgimage_to_raw(
    image: &screencapturekit::screenshot_manager::CGImage,
) -> Result<RawImage, CaptureError> {
    let width = image.width() as u32;
    let height = image.height() as u32;
    let pixels = image.rgba_data().map_err(map_sc_error)?;
    Ok(RawImage {
        width,
        height,
        pixels,
    })
}
