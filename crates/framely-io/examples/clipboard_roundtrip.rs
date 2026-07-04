use framely_core::RawImage;

fn main() {
    let mut pixels = vec![0u8; 8 * 8 * 4];
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&[200, 50, 100, 255]);
    }
    let image = RawImage {
        width: 8,
        height: 8,
        pixels,
    };

    framely_io::write_image_to_clipboard(&image).expect("write should succeed");
    println!("Wrote 8x8 image to clipboard");

    let read_back = framely_io::read_image_from_clipboard().expect("read should succeed");
    println!(
        "Read back {}x{}, matches: {}",
        read_back.width,
        read_back.height,
        read_back.pixels == image.pixels
    );
}
