//! Image de démonstration procédurale, affichée au premier lancement avant
//! toute capture/import réelle (Sprint 2). Mime une fenêtre d'app simple :
//! fond clair + barre de titre + quelques blocs de contenu.

use framely_core::RawImage;

pub fn generate(width: u32, height: u32) -> RawImage {
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let set = |pixels: &mut [u8], x: u32, y: u32, color: [u8; 4]| {
        if x < width && y < height {
            let i = ((y * width + x) * 4) as usize;
            pixels[i..i + 4].copy_from_slice(&color);
        }
    };

    let bg = [246, 246, 248, 255];
    let titlebar = [225, 226, 230, 255];
    let card = [255, 255, 255, 255];
    let accent = [99, 102, 241, 255];
    let text_line = [200, 202, 208, 255];

    for y in 0..height {
        for x in 0..width {
            set(&mut pixels, x, y, bg);
        }
    }

    let titlebar_h = (height as f32 * 0.12) as u32;
    for y in 0..titlebar_h {
        for x in 0..width {
            set(&mut pixels, x, y, titlebar);
        }
    }
    for (i, dot_color) in [[255, 95, 86, 255], [255, 189, 46, 255], [39, 201, 63, 255]]
        .iter()
        .enumerate()
    {
        let cx = 20 + i as u32 * 24;
        let cy = titlebar_h / 2;
        for dy in 0..12u32 {
            for dx in 0..12u32 {
                let px = cx + dx;
                let py = cy + dy;
                let dist = ((dx as i32 - 6).pow(2) + (dy as i32 - 6).pow(2)) as f32;
                if dist <= 36.0 {
                    set(&mut pixels, px, py, *dot_color);
                }
            }
        }
    }

    let card_margin = (width as f32 * 0.06) as u32;
    let card_top = titlebar_h + card_margin;
    for y in card_top..height.saturating_sub(card_margin) {
        for x in card_margin..width.saturating_sub(card_margin) {
            set(&mut pixels, x, y, card);
        }
    }

    let block_x = card_margin + card_margin;
    let block_y = card_top + card_margin;
    let block_w = (width - 2 * card_margin - 2 * card_margin).min(width);
    for y in block_y..(block_y + 24).min(height) {
        for x in block_x..(block_x + block_w).min(width) {
            set(&mut pixels, x, y, accent);
        }
    }

    for line in 0..5u32 {
        let ly = block_y + 48 + line * 20;
        if ly + 8 >= height {
            break;
        }
        let line_w = block_w.saturating_sub(line * 40);
        for y in ly..(ly + 8).min(height) {
            for x in block_x..(block_x + line_w).min(width) {
                set(&mut pixels, x, y, text_line);
            }
        }
    }

    RawImage {
        width,
        height,
        pixels,
    }
}
