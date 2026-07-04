//! Pipeline d'enjolivement : fond + marge + coins arrondis + ombre.
//!
//! `tiny-skia` n'a pas de filtre de flou gaussien : l'ombre portée est
//! approximée par un empilement de rectangles arrondis semi-transparents
//! de rayon croissant (voir `draw_shadow`), ce qui reste très bon marché
//! et suffisant visuellement pour une ombre "douce".

use framely_core::{Background, Ratio, RawImage, Scale, Style};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Mask, Paint, PathBuilder, Pixmap, PixmapPaint,
    Point, Rect, Shader, SpreadMode, Transform,
};

/// Rendu final : image RGBA8 (alpha droit, non prémultiplié) prête à
/// afficher (egui) ou exporter.
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

const SHADOW_LAYERS: usize = 8;

/// Calcule des réglages esthétiques par défaut pour une image donnée.
/// C'est le cœur du "beau par défaut" — voir ARCHITECTURE.md section 6.
pub fn auto_balance(source: &RawImage) -> Style {
    let max_dim = source.width.max(source.height) as f32;
    let padding = (max_dim * 0.08).clamp(32.0, 160.0) as u16;

    let gradients = framely_presets::builtin_gradients();
    let idx = (source.width as usize).wrapping_add(source.height as usize) % gradients.len();
    let background = Background::Gradient(gradients[idx].id.clone());

    Style {
        background,
        padding,
        ..Style::default()
    }
}

/// Compose l'image source avec le style donné : fond, marge, ombre, coins
/// arrondis, mise à l'échelle. Renvoie `None` si les dimensions calculées
/// sont invalides (image source vide).
pub fn render(source: &RawImage, style: &Style) -> Option<RenderedImage> {
    if source.width == 0 || source.height == 0 {
        return None;
    }

    let scale = match style.scale {
        Scale::X1 => 1.0,
        Scale::X2 => 2.0,
    };

    let padding = style.padding as f32 * scale;
    let content_w = source.width as f32 * scale;
    let content_h = source.height as f32 * scale;

    let mut canvas_w = content_w + 2.0 * padding;
    let mut canvas_h = content_h + 2.0 * padding;

    if let Ratio::Fixed(rw, rh) = style.ratio {
        let target = rw as f32 / rh as f32;
        let current = canvas_w / canvas_h;
        if current < target {
            canvas_w = canvas_h * target;
        } else {
            canvas_h = canvas_w / target;
        }
    }

    let canvas_w = canvas_w.round().max(1.0) as u32;
    let canvas_h = canvas_h.round().max(1.0) as u32;

    let content_x = (canvas_w as f32 - content_w) / 2.0;
    let content_y = (canvas_h as f32 - content_h) / 2.0;
    let corner_radius = style.corner_radius * scale;

    let mut pixmap = Pixmap::new(canvas_w, canvas_h)?;

    draw_background(&mut pixmap, &style.background);
    draw_shadow(
        &mut pixmap,
        content_x,
        content_y,
        content_w,
        content_h,
        corner_radius,
        &style.shadow,
        scale,
    );
    draw_content(
        &mut pixmap,
        source,
        content_x,
        content_y,
        content_w,
        content_h,
        corner_radius,
        scale,
    );

    Some(RenderedImage {
        width: canvas_w,
        height: canvas_h,
        pixels: unpremultiply(pixmap.data()),
    })
}

fn draw_background(pixmap: &mut Pixmap, background: &Background) {
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);

    match background {
        Background::Transparent => {
            // Pixmap::new() est déjà entièrement transparent.
        }
        Background::Solid(rgba) => {
            pixmap.fill(color_from_rgba(*rgba));
        }
        Background::Image(_) => {
            // TODO(v1.0): fond image perso — non implémenté au Sprint 1.
            pixmap.fill(Color::from_rgba8(240, 240, 240, 255));
        }
        Background::Gradient(id) => {
            let preset = framely_presets::gradient_by_id(id);
            let stop_count = preset.colors.len().max(1);
            let stops: Vec<GradientStop> = preset
                .colors
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let position = if stop_count > 1 {
                        i as f32 / (stop_count - 1) as f32
                    } else {
                        0.0
                    };
                    GradientStop::new(position, color_from_rgba(*c))
                })
                .collect();

            if let Some(shader) = LinearGradient::new(
                Point::from_xy(0.0, 0.0),
                Point::from_xy(w, h),
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let paint = Paint {
                    shader,
                    anti_alias: true,
                    ..Default::default()
                };
                if let Some(rect) = Rect::from_xywh(0.0, 0.0, w, h) {
                    let path = PathBuilder::from_rect(rect);
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_shadow(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    shadow: &framely_core::ShadowParams,
    scale: f32,
) {
    if shadow.intensity <= 0.0 {
        return;
    }
    let offset_y = shadow.offset_y * scale;
    let max_spread = shadow.blur * scale;

    for i in 0..SHADOW_LAYERS {
        let t = i as f32 / (SHADOW_LAYERS - 1) as f32;
        let spread = t * max_spread;
        let alpha = (shadow.intensity * (1.0 - t).powi(2) * 0.5).clamp(0.0, 1.0);
        if alpha <= 0.001 {
            continue;
        }

        let Some(path) = rounded_rect_path(
            x - spread,
            y + offset_y - spread,
            w + 2.0 * spread,
            h + 2.0 * spread,
            radius + spread,
        ) else {
            continue;
        };

        let paint = Paint {
            shader: Shader::SolidColor(Color::from_rgba(0.0, 0.0, 0.0, alpha).unwrap()),
            anti_alias: true,
            ..Default::default()
        };
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_content(
    pixmap: &mut Pixmap,
    source: &RawImage,
    x: f32,
    y: f32,
    content_w: f32,
    content_h: f32,
    radius: f32,
    scale: f32,
) {
    let Some(source_pixmap) = Pixmap::from_vec(
        premultiply(&source.pixels),
        tiny_skia::IntSize::from_wh(source.width, source.height).unwrap_or_else(|| {
            tiny_skia::IntSize::from_wh(1, 1).expect("1x1 is always a valid size")
        }),
    ) else {
        return;
    };

    let mut mask = match Mask::new(pixmap.width(), pixmap.height()) {
        Some(mask) => mask,
        None => return,
    };
    if let Some(clip_path) = rounded_rect_path(x, y, content_w, content_h, radius) {
        mask.fill_path(&clip_path, FillRule::Winding, true, Transform::identity());
    }

    let transform = Transform::from_translate(x, y).pre_scale(scale, scale);
    pixmap.draw_pixmap(
        0,
        0,
        source_pixmap.as_ref(),
        &PixmapPaint::default(),
        transform,
        Some(&mask),
    );
}

/// Construit un chemin de rectangle à coins arrondis via des courbes de
/// Bézier cubiques (tiny-skia n'a pas de primitive rounded-rect intégrée).
fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<tiny_skia::Path> {
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let r = radius.max(0.0).min(w / 2.0).min(h / 2.0);
    let k = 0.552_284_8 * r;

    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

fn color_from_rgba(rgba: [u8; 4]) -> Color {
    Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3])
}

/// tiny-skia stocke ses pixels en RGBA prémultiplié : l'image source
/// (alpha droit) doit être prémultipliée avant d'entrer dans un `Pixmap`.
fn premultiply(data: &[u8]) -> Vec<u8> {
    data.chunks_exact(4)
        .flat_map(|c| {
            let a = c[3] as u32;
            let r = (c[0] as u32 * a / 255) as u8;
            let g = (c[1] as u32 * a / 255) as u8;
            let b = (c[2] as u32 * a / 255) as u8;
            [r, g, b, c[3]]
        })
        .collect()
}

/// Inverse de `premultiply`, pour renvoyer un buffer RGBA en alpha droit
/// (format attendu par egui / l'export PNG).
fn unpremultiply(data: &[u8]) -> Vec<u8> {
    data.chunks_exact(4)
        .flat_map(|c| {
            let a = c[3];
            if a == 0 {
                [0, 0, 0, 0]
            } else {
                let a32 = a as u32;
                let r = ((c[0] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
                let g = ((c[1] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
                let b = ((c[2] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
                [r, g, b, a]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_image(w: u32, h: u32) -> RawImage {
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[10, 20, 30, 255]);
        }
        RawImage {
            width: w,
            height: h,
            pixels,
        }
    }

    #[test]
    fn render_produces_padded_canvas() {
        let source = dummy_image(100, 60);
        let style = Style {
            padding: 20,
            scale: Scale::X1,
            ratio: Ratio::Auto,
            ..Style::default()
        };

        let rendered = render(&source, &style).expect("render should succeed");
        assert_eq!(rendered.width, 140); // 100 + 2*20
        assert_eq!(rendered.height, 100); // 60 + 2*20
        assert_eq!(rendered.pixels.len(), (140 * 100 * 4) as usize);
    }

    #[test]
    fn render_respects_fixed_ratio() {
        let source = dummy_image(100, 100);
        let style = Style {
            padding: 10,
            ratio: Ratio::Fixed(16, 9),
            scale: Scale::X1,
            ..Style::default()
        };

        let rendered = render(&source, &style).expect("render should succeed");
        let ratio = rendered.width as f32 / rendered.height as f32;
        assert!((ratio - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn auto_balance_scales_padding_with_image_size() {
        let small = auto_balance(&dummy_image(100, 100));
        let large = auto_balance(&dummy_image(2000, 2000));
        assert!(large.padding >= small.padding);
    }

    #[test]
    fn premultiply_unpremultiply_roundtrip_is_stable() {
        let original = vec![200u8, 100, 50, 128];
        let premul = premultiply(&original);
        let restored = unpremultiply(&premul);
        for i in 0..3 {
            assert!((original[i] as i16 - restored[i] as i16).abs() <= 1);
        }
        assert_eq!(original[3], restored[3]);
    }
}
