use image::{Rgb, RgbImage};

use crate::types::{Alignment, JsPrintElement, QrErrorCorrection, RenderContext};

pub fn render(img: &mut RgbImage, elem: &JsPrintElement, ctx: &mut RenderContext) -> i32 {
    let value = match &elem.value {
        Some(v) if !v.is_empty() => v,
        _ => return ctx.y,
    };

    let module_size = elem.size.unwrap_or(4).clamp(1, 16) as usize;
    let ec = elem
        .error_correction
        .as_ref()
        .map(|s| QrErrorCorrection::from_js(Some(s.clone())))
        .unwrap_or_default();

    let ec_level = match ec {
        QrErrorCorrection::L => qrcode::EcLevel::L,
        QrErrorCorrection::M => qrcode::EcLevel::M,
        QrErrorCorrection::Q => qrcode::EcLevel::Q,
        QrErrorCorrection::H => qrcode::EcLevel::H,
    };

    let qr_code = match qrcode::QrCode::with_error_correction_level(value.as_bytes(), ec_level) {
        Ok(q) => q,
        Err(_) => return ctx.y,
    };

    let qr_width_px = qr_code.width();
    let render_size = qr_width_px * module_size;
    let render_size_i = render_size as i32;

    let x_start = match Alignment::Center {
        _ => (ctx.paper_width_px - render_size_i) / 2,
    };

    for (row_idx, row) in qr_code.to_colors().chunks(qr_width_px).enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let is_dark = matches!(cell, qrcode::Color::Dark);
            if !is_dark {
                continue;
            }

            let base_x = x_start + (col_idx as i32) * (module_size as i32);
            let base_y = ctx.y + (row_idx as i32) * (module_size as i32);

            for dy in 0..module_size {
                for dx in 0..module_size {
                    let px = base_x + dx as i32;
                    let py = base_y + dy as i32;
                    if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height()
                    {
                        img.put_pixel(px as u32, py as u32, Rgb([0, 0, 0]));
                    }
                }
            }
        }
    }

    ctx.y + render_size_i + ctx.row_gap
}
