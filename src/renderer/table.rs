use image::RgbImage;

use crate::types::{Alignment, JsPrintElement, RenderContext};
use crate::{draw_mixed_rtl_center, draw_mixed_rtl_left, draw_mixed_rtl_right, wrap_mixed_rtl};
use rusttype::Scale;

pub fn render(img: &mut RgbImage, elem: &JsPrintElement, ctx: &mut RenderContext) -> i32 {
    let columns: Vec<crate::types::TableColumn> = match &elem.columns {
        Some(cols) => cols.iter().map(|c| c.to_internal()).collect(),
        None => return ctx.y,
    };

    let rows: Vec<crate::types::TableRow> = match &elem.rows {
        Some(r) => r.iter().map(|r| r.to_internal()).collect(),
        None => return ctx.y,
    };

    if columns.is_empty() {
        return ctx.y;
    }

    let style = elem
        .style
        .as_ref()
        .map(|s| s.to_internal())
        .unwrap_or_default();

    let base_size: f32 = match style.font {
        crate::types::FontType::A => 42.0,
        crate::types::FontType::B => 28.0,
    };

    let scale = Scale::uniform(base_size);
    let header_scale = Scale::uniform(base_size * 1.1);
    let inner_w = ctx.inner_width as f32;

    let mut col_rights: Vec<i32> = Vec::with_capacity(columns.len() + 1);
    let mut x = ctx.right_edge;
    for col in &columns {
        col_rights.push(x);
        x -= (inner_w * col.width as f32) as i32;
    }
    col_rights.push(x);

    let mut y = ctx.y;

    for (ci, col) in columns.iter().enumerate() {
        let r = col_rights[ci];
        draw_mixed_rtl_right(img, ctx.font, header_scale, &col.label, r, y);
    }

    y += ctx.row_gap - 6;

    for row in &rows {
        let mut max_lines = 1u32;

        let mut cell_lines: Vec<Vec<String>> = Vec::with_capacity(row.cells.len());

        for (ci, cell) in row.cells.iter().enumerate() {
            if ci >= columns.len() {
                break;
            }

            let r = col_rights[ci];
            let next_r = col_rights[ci + 1];
            let col_width = r - next_r;

            let lines = wrap_mixed_rtl(ctx.font, scale, &cell.value, col_width);
            max_lines = max_lines.max(lines.len() as u32);
            cell_lines.push(lines);
        }

        for (ci, lines) in cell_lines.iter().enumerate() {
            if ci >= columns.len() {
                break;
            }

            let col = &columns[ci];
            let r = col_rights[ci];
            let next_r = col_rights[ci + 1];

            for (li, line) in lines.iter().enumerate() {
                let line_y = y + (li as i32) * (ctx.row_gap - 4);

                match col.align {
                    Alignment::Right => {
                        draw_mixed_rtl_right(img, ctx.font, scale, line, r, line_y);
                    }
                    Alignment::Center => {
                        draw_mixed_rtl_center(
                            img,
                            ctx.font,
                            scale,
                            line,
                            next_r + (r - next_r),
                            line_y,
                        );
                    }
                    Alignment::Left => {
                        draw_mixed_rtl_left(img, ctx.font, scale, line, next_r, line_y);
                    }
                }
            }
        }

        y += (max_lines as i32) * (ctx.row_gap - 4);
    }

    y
}
