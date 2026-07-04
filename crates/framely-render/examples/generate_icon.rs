//! Génère les PNG d'un `.iconset` pour l'icône d'app Framely (placeholder
//! procédural : dégradé indigo + cadre blanc, à remplacer par un vrai
//! design). Usage : `cargo run -p framely-render --example generate_icon`
//! puis `iconutil -c icns packaging/AppIcon.iconset -o packaging/AppIcon.icns`.

use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, Rect,
    SpreadMode, Stroke, Transform,
};

const SIZES: &[(&str, u32)] = &[
    ("icon_16x16", 16),
    ("icon_16x16@2x", 32),
    ("icon_32x32", 32),
    ("icon_32x32@2x", 64),
    ("icon_128x128", 128),
    ("icon_128x128@2x", 256),
    ("icon_256x256", 256),
    ("icon_256x256@2x", 512),
    ("icon_512x512", 512),
    ("icon_512x512@2x", 1024),
];

fn main() {
    let out_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/AppIcon.iconset");
    std::fs::create_dir_all(&out_dir).expect("créer le dossier iconset");

    for (name, size) in SIZES {
        let pixmap = render_icon(*size);
        let path = out_dir.join(format!("{name}.png"));
        pixmap.save_png(&path).expect("écrire le PNG");
        println!("Écrit {}", path.display());
    }
}

fn render_icon(size: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(size, size).expect("taille d'icône valide");
    let s = size as f32;

    let stops = vec![
        GradientStop::new(0.0, Color::from_rgba8(99, 102, 241, 255)),
        GradientStop::new(1.0, Color::from_rgba8(236, 72, 153, 255)),
    ];
    if let Some(shader) = LinearGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(s, s),
        stops,
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        let paint = Paint {
            shader,
            anti_alias: true,
            ..Default::default()
        };
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, s, s) {
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

    // Cadre blanc au centre : symbolise le "cadrage" de la capture.
    let margin = s * 0.24;
    if let Some(frame_rect) = Rect::from_xywh(margin, margin, s - 2.0 * margin, s - 2.0 * margin) {
        let path = PathBuilder::from_rect(frame_rect);
        let paint = Paint {
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 235)),
            anti_alias: true,
            ..Default::default()
        };
        let stroke = Stroke {
            width: (s * 0.045).max(1.0),
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    pixmap
}
