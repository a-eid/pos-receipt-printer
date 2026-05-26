use image::{GenericImageView, Rgb, RgbImage};

use crate::types::{Alignment, JsPrintElement, RenderContext};

pub fn render(img: &mut RgbImage, elem: &JsPrintElement, ctx: &mut RenderContext) -> i32 {
    let path = match &elem.path {
        Some(p) if !p.is_empty() => p,
        _ => return ctx.y,
    };

    let source = match image::open(path) {
        Ok(img) => img,
        Err(_) => return ctx.y,
    };

    let target_width = elem
        .width_px
        .unwrap_or(ctx.inner_width as u32)
        .min(ctx.inner_width as u32);

    let aspect = source.height() as f32 / source.width() as f32;
    let target_height = (target_width as f32 * aspect) as u32;

    let resized = source.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Nearest,
    );

    let x_offset = match Alignment::Center {
        _ => (ctx.paper_width_px - target_width as i32) / 2,
    };

    for y in 0..target_height {
        for x in 0..target_width {
            let pixel = resized.get_pixel(x, y);
            let gray =
                (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;

            if gray <= ctx.threshold {
                let px = (x_offset + x as i32) as u32;
                let py = (ctx.y + y as i32) as u32;
                if px < img.width() && py < img.height() {
                    img.put_pixel(px, py, Rgb([0, 0, 0]));
                }
            }
        }
    }

    ctx.y + target_height as i32 + ctx.row_gap
}
