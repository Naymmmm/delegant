use base64::Engine;
use image::{DynamicImage, Rgba};
use std::io::Cursor;
use xcap::Monitor;

use crate::error::{AppError, AppResult};
use crate::screen::a11y::{get_a11y_tree, A11yNode};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureResult {
    pub base64: String,
    pub orig_width: u32,
    pub orig_height: u32,
    pub scaled_width: u32,
    pub scaled_height: u32,
    pub scale_factor: f64,
    pub media_type: String,
    pub nodes: Vec<A11yNode>,
}

pub fn capture_screenshot(max_width: u32, max_height: u32) -> AppResult<CaptureResult> {
    let monitors = Monitor::all().map_err(|e| AppError::Screenshot(e.to_string()))?;
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Screenshot("No monitor found".into()))?;

    let mut image = monitor
        .capture_image()
        .map_err(|e| AppError::Screenshot(e.to_string()))?;

    let orig_width = image.width();
    let orig_height = image.height();

    // Get A11y nodes to draw Set-of-Mark
    let nodes = get_a11y_tree().unwrap_or_else(|_| Vec::new());

    // Draw Set-of-Mark overlays via imageproc
    if !nodes.is_empty() {
        if let Ok(font_bytes) = std::fs::read("C:\\Windows\\Fonts\\arial.ttf") {
            if let Ok(font) = ab_glyph::FontVec::try_from_vec(font_bytes) {
                let red = Rgba([255u8, 0, 0, 255]);
                let bg = Rgba([255u8, 255, 255, 230]);
                let scale = ab_glyph::PxScale { x: 18.0, y: 18.0 };

                for node in &nodes {
                    let (x, y, r, b) = node.rect;
                    // Clamp to image dimensions
                    let x1 = (x.max(0) as u32).min(orig_width - 1);
                    let y1 = (y.max(0) as u32).min(orig_height - 1);
                    let x2 = (r.max(0) as u32).min(orig_width - 1);
                    let y2 = (b.max(0) as u32).min(orig_height - 1);

                    let rect = imageproc::rect::Rect::at(x1 as i32, y1 as i32)
                        .of_size((x2 - x1).max(1), (y2 - y1).max(1));

                    // Draw bounding box
                    imageproc::drawing::draw_hollow_rect_mut(&mut image, rect, red);

                    // Draw text background box
                    let text = format!("[{}]", node.id);
                    let text_width = text.len() as i32 * 10;
                    let bg_rect = imageproc::rect::Rect::at(x1 as i32, (y1 as i32 - 18).max(0))
                        .of_size(text_width as u32 + 4, 18);
                    imageproc::drawing::draw_filled_rect_mut(&mut image, bg_rect, bg);

                    // Draw text ID
                    imageproc::drawing::draw_text_mut(
                        &mut image,
                        red,
                        (x1 + 2) as i32,
                        (y1 as i32 - 18).max(0),
                        scale,
                        &font,
                        &text,
                    );
                }
            }
        }
    }

    let dynamic = DynamicImage::ImageRgba8(image);

    // Calculate scale factor to fit within max dimensions
    let scale_x = max_width as f64 / orig_width as f64;
    let scale_y = max_height as f64 / orig_height as f64;
    let scale_factor = scale_x.min(scale_y).min(1.0);

    let scaled_width = (orig_width as f64 * scale_factor) as u32;
    let scaled_height = (orig_height as f64 * scale_factor) as u32;

    let scaled = if scale_factor < 1.0 {
        // Use Triangle (bilinear) for speed
        dynamic.resize_exact(
            scaled_width,
            scaled_height,
            image::imageops::FilterType::Triangle,
        )
    } else {
        dynamic
    };

    // Encode to JPEG
    let mut buf = Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 72);
    scaled
        .write_with_encoder(encoder)
        .map_err(|e| AppError::Screenshot(e.to_string()))?;

    let base64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());

    Ok(CaptureResult {
        base64,
        orig_width,
        orig_height,
        scaled_width,
        scaled_height,
        scale_factor,
        media_type: "image/jpeg".to_string(),
        nodes,
    })
}
