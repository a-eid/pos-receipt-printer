use image::RgbImage;

use crate::draw_dotted;
use crate::types::{JsPrintElement, RenderContext};

pub fn render(img: &mut RgbImage, _elem: &JsPrintElement, ctx: &RenderContext) -> i32 {
    draw_dotted(img, ctx.y, ctx.margin_h, ctx.paper_width_px - ctx.margin_h);
    ctx.y + 12
}
