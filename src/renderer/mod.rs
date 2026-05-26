pub mod barcode;
pub mod divider;
pub mod image_render;
pub mod qr;
pub mod table;
pub mod text;

use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use rusttype::Font;

use crate::types::{BarcodeCommand, JsPrintElement, PrintOptions, RenderContext, RenderResult};

pub const MAX_IMAGE_HEIGHT: u32 = 5000;

pub fn render_elements(elements: &[JsPrintElement], options: &PrintOptions) -> RenderResult {
    let mut img: RgbImage = ImageBuffer::from_pixel(
        options.paper_width_px,
        MAX_IMAGE_HEIGHT,
        Rgb([255, 255, 255]),
    );

    let font_bytes = include_bytes!("../fonts/NotoSansArabic-Regular.ttf");
    let font = Font::try_from_bytes(font_bytes).expect("font load failed");

    let paper_w = options.paper_width_px as i32;
    let inner_w = paper_w - options.margin_h * 2;

    let mut ctx = RenderContext {
        paper_width_px: paper_w,
        margin_h: options.margin_h,
        inner_width: inner_w,
        right_edge: options.margin_h + inner_w,
        y: options.margin_top,
        row_gap: options.row_gap,
        threshold: options.threshold,
        font: &font,
    };

    let mut has_cut = false;
    let mut barcodes: Vec<BarcodeCommand> = Vec::new();

    for elem in elements {
        ctx.y = match elem.r#type.as_str() {
            "text" => text::render(&mut img, elem, &mut ctx),
            "table" => table::render(&mut img, elem, &mut ctx),
            "qrcode" => qr::render(&mut img, elem, &mut ctx),
            "barcode" => {
                if let Some(cmd) = barcode::render(elem, &mut ctx) {
                    barcodes.push(cmd);
                }
                ctx.y
            }
            "image" => image_render::render(&mut img, elem, &mut ctx),
            "divider" => divider::render(&mut img, elem, &ctx),
            "feed" => {
                let lines = elem.lines.unwrap_or(1);
                ctx.y + (lines as i32) * ctx.row_gap
            }
            "cut" => {
                has_cut = true;
                ctx.y
            }
            _ => ctx.y,
        };
    }

    ctx.y += options.margin_bottom;

    if !has_cut {
        ctx.y += ctx.row_gap;
    }

    let used_h = (ctx.y as u32).min(MAX_IMAGE_HEIGHT - 2);
    let gray = DynamicImage::ImageRgb8(img)
        .crop_imm(0, 0, options.paper_width_px, used_h)
        .to_luma8();

    RenderResult {
        image: gray,
        barcodes,
    }
}
