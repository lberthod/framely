fn main() {
    match framely_capture::capture_display(None) {
        Ok(image) => {
            println!("Captured {}x{} pixels", image.width, image.height);
            let non_zero = image.pixels.iter().any(|&b| b != 0);
            println!("Has non-zero pixel data: {non_zero}");
        }
        Err(e) => println!("ERR: {e}"),
    }

    match framely_capture::list_windows() {
        Ok(windows) => {
            println!("Found {} on-screen windows", windows.len());
            for w in windows.iter().take(5) {
                println!("  [{}] {} - {}", w.id, w.app_name, w.title);
            }
        }
        Err(e) => println!("ERR listing windows: {e}"),
    }
}
