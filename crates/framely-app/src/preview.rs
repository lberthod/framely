//! Utilitaires pour l'affichage de la preview : damier de transparence et
//! downscale des très grandes images avant upload GPU (l'export continue de
//! travailler sur l'image pleine résolution, voir `main.rs`).

const CHECKER_CELL: u32 = 8;
const CHECKER_LIGHT: [u8; 3] = [235, 235, 235];
const CHECKER_DARK: [u8; 3] = [214, 214, 214];

/// Composite un buffer RGBA (alpha droit) sur un damier gris clair/gris
/// foncé, pour visualiser la transparence dans la preview (l'export garde
/// la vraie transparence, ce damier n'est jamais écrit sur disque).
pub fn composite_on_checkerboard(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; rgba.len()];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            let alpha = rgba[i + 3] as u32;
            let checker = if ((x / CHECKER_CELL) + (y / CHECKER_CELL)).is_multiple_of(2) {
                CHECKER_LIGHT
            } else {
                CHECKER_DARK
            };
            for c in 0..3 {
                let fg = rgba[i + c] as u32;
                out[i + c] = ((fg * alpha + checker[c] as u32 * (255 - alpha)) / 255) as u8;
            }
            out[i + 3] = 255;
        }
    }
    out
}

/// Downscale (plus proche voisin) un buffer RGBA si sa plus grande
/// dimension dépasse `max_dim`. Suffisant pour une preview qui sera de
/// toute façon réduite à l'écran ; ne doit jamais être utilisé pour l'export.
pub fn downscale_for_preview(
    width: u32,
    height: u32,
    rgba: &[u8],
    max_dim: u32,
) -> (u32, u32, Vec<u8>) {
    let largest = width.max(height);
    if largest <= max_dim || largest == 0 {
        return (width, height, rgba.to_vec());
    }

    let scale = max_dim as f32 / largest as f32;
    let new_w = ((width as f32 * scale).round() as u32).max(1);
    let new_h = ((height as f32 * scale).round() as u32).max(1);

    let mut out = vec![0u8; (new_w * new_h * 4) as usize];
    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = ((x as f32 / scale) as u32).min(width - 1);
            let src_y = ((y as f32 / scale) as u32).min(height - 1);
            let src_i = ((src_y * width + src_x) * 4) as usize;
            let dst_i = ((y * new_w + x) * 4) as usize;
            out[dst_i..dst_i + 4].copy_from_slice(&rgba[src_i..src_i + 4]);
        }
    }
    (new_w, new_h, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkerboard_composite_makes_transparent_pixels_opaque() {
        let rgba = vec![0u8, 0, 0, 0]; // 1x1 transparent pixel
        let out = composite_on_checkerboard(1, 1, &rgba);
        assert_eq!(out[3], 255);
    }

    #[test]
    fn checkerboard_composite_preserves_opaque_pixels() {
        let rgba = vec![10u8, 20, 30, 255];
        let out = composite_on_checkerboard(1, 1, &rgba);
        assert_eq!(out, vec![10, 20, 30, 255]);
    }

    #[test]
    fn downscale_leaves_small_images_untouched() {
        let rgba = vec![1u8; 4 * 4 * 4];
        let (w, h, out) = downscale_for_preview(4, 4, &rgba, 100);
        assert_eq!((w, h), (4, 4));
        assert_eq!(out, rgba);
    }

    #[test]
    fn downscale_shrinks_large_images() {
        let rgba = vec![1u8; 2000 * 10 * 4];
        let (w, h, out) = downscale_for_preview(2000, 10, &rgba, 1000);
        assert_eq!(w, 1000);
        assert!(h <= 10);
        assert_eq!(out.len(), (w * h * 4) as usize);
    }
}
