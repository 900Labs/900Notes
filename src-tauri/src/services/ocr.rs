use std::env::temp_dir;
use std::fs;
use std::io::Write;
use std::process::Command;

pub fn ocr_image_bytes(data: &[u8], mime_type: &str) -> Result<String, String> {
    let ext = match mime_type {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/tiff" | "image/tif" => "tif",
        "image/webp" => "webp",
        _ => "img",
    };

    let dir = temp_dir();
    let input_path = dir.join(format!("900notes_ocr_input.{}", ext));
    let output_base = dir.join("900notes_ocr_output");

    let mut file =
        fs::File::create(&input_path).map_err(|e| format!("failed to create temp file: {}", e))?;
    file.write_all(data)
        .map_err(|e| format!("failed to write temp file: {}", e))?;
    drop(file);

    let result = Command::new("tesseract")
        .arg(&input_path)
        .arg(&output_base)
        .arg("--psm")
        .arg("3")
        .output();

    let _ = fs::remove_file(&input_path);

    match result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("not found") || stderr.contains("No such file") {
                    return Err(
                        "Tesseract is not installed. Install it with: brew install tesseract (macOS), apt install tesseract-ocr (Linux)"
                            .to_string(),
                    );
                }
                let _ = fs::remove_file(format!("{}.txt", output_base.display()));
                return Err(format!("Tesseract failed: {}", stderr));
            }
            let output_file = format!("{}.txt", output_base.display());
            let text = fs::read_to_string(&output_file)
                .map_err(|e| format!("failed to read OCR output: {}", e));
            let _ = fs::remove_file(&output_file);
            text
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(
                    "Tesseract is not installed. Install it with: brew install tesseract (macOS), apt install tesseract-ocr (Linux)"
                        .to_string(),
                )
            } else {
                Err(format!("failed to run tesseract: {}", e))
            }
        }
    }
}
