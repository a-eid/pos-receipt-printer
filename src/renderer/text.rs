use image::RgbImage;

use crate::types::{Alignment, JsPrintElement, RenderContext};
use crate::{draw_mixed_rtl_center, draw_mixed_rtl_left, draw_mixed_rtl_right, wrap_mixed_rtl};
use rusttype::Scale;

pub fn render(img: &mut RgbImage, elem: &JsPrintElement, ctx: &mut RenderContext) -> i32 {
    let value = match &elem.value {
        Some(v) if !v.is_empty() => v,
        _ => return ctx.y,
    };

    let style = elem
        .style
        .as_ref()
        .map(|s| s.to_internal())
        .unwrap_or_default();

    let base_size: f32 = match style.font {
        crate::types::FontType::A => 44.0,
        crate::types::FontType::B => 30.0,
    };

    let scale_factor = (style.width.max(1).min(8) as f32).max(style.height.max(1).min(8) as f32);
    let scale = Scale::uniform(base_size * scale_factor);

    let max_width = ctx.inner_width;
    let lines = wrap_mixed_rtl(ctx.font, scale, value, max_width);

    for (i, line) in lines.iter().enumerate() {
        let line_y = ctx.y + (i as i32) * (ctx.row_gap - 4);

        match style.align {
            Alignment::Center => {
                draw_mixed_rtl_center(img, ctx.font, scale, line, ctx.paper_width_px, line_y);
            }
            Alignment::Right => {
                draw_mixed_rtl_right(img, ctx.font, scale, line, ctx.right_edge, line_y);
            }
            Alignment::Left => {
                draw_mixed_rtl_left(img, ctx.font, scale, line, ctx.margin_h, line_y);
            }
        }
    }

    let line_count = lines.len().max(1) as i32;
    ctx.y + line_count * (ctx.row_gap - 4)
}
