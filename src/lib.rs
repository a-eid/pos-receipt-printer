use napi::bindgen_prelude::*;
use napi_derive::napi;

use escpos::{driver::SerialPortDriver, printer::Printer, utils::*};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, GrayImage, Luma};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale, point, PositionedGlyph};
use ar_reshaper::reshape_line;
use serde::Deserialize;

// ===================== Defaults =====================
const DEFAULT_COM_PORT: &str = "COM7";
const DEFAULT_BAUD_RATE: u32 = 9600;

// ===================== Helpers =====================
fn env_port_or_default(port: Option<String>) -> String {
    if let Some(p) = port { return normalize_com_port(&p); }
    if let Ok(p) = std::env::var("PRINTER_COM_PORT") { return normalize_com_port(&p); }
    normalize_com_port(DEFAULT_COM_PORT)
}
fn env_baud_or_default(baud: Option<u32>) -> u32 {
    if let Some(b) = baud { return b; }
    std::env::var("PRINTER_BAUD_RATE").ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_BAUD_RATE)
}
fn normalize_com_port(port: &str) -> String {
    #[cfg(windows)]
    {
        let up = port.to_uppercase();
        if up.starts_with("COM") {
            if let Ok(n) = up[3..].parse::<u32>() {
                if n > 9 { return format!("\\\\.\\{}", up); }
            }
        }
        port.to_string()
    }
    #[cfg(not(windows))] { port.to_string() }
}

// ===================== Data =====================
#[derive(Clone, Deserialize)]
struct Item {
    name: String,
    qty_str: String,
    price: f32,
    total: f32,
}

#[derive(Clone)]
struct ReceiptData {
    store_name: String,
    date_time_line: String,
    invoice_no: String,
    items: Vec<Item>,
    discount: f32,
    total: f32,
    footer_address: String,
    footer_delivery: String,
    footer_phones: String,
}

#[derive(Clone)]
struct Layout {
    paper_width_px: u32,
    threshold: u8,
    margin_h: i32,
    margin_top: i32,
    margin_bottom: i32,
    row_gap: i32,
    fonts: Fonts,
    cols: [f32; 4], // [name, qty, price, total] (fractions of inner width)
}
#[derive(Clone)]
struct Fonts {
    title: f32,
    header_dt: f32,
    header_no: f32,
    header_cols: f32,
    item: f32,
    total_label: f32,
    total_value: f32,
    footer: f32,
    footer_phones: f32,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            paper_width_px: 576,
            threshold: 150,
            margin_h: 0,
            margin_top: -28,
            margin_bottom: 0,
            row_gap: 32,
            fonts: Fonts {
                title: 90.0,
                header_dt: 45.0,
                header_no: 46.0,
                header_cols: 42.0,
                item: 44.0,
                total_label: 48.0,
                total_value: 66.0,
                footer: 45.0,
                footer_phones: 56.0,
            },
            cols: [0.60, 0.11, 0.12, 0.17],
        }
    }
}

// ===================== N-API payloads =====================
#[napi(object)]
pub struct JsItem {
    pub name: String,
    pub qty: String, // keep as string for stability
    pub price: f64,
    pub total: f64,
}
#[napi(object)]
pub struct JsFooter {
    pub address: String,
    #[napi(js_name = "lastLine")]
    pub last_line: String,
    pub phones: Option<String>,
}
#[napi(object)]
pub struct JsPrintPayload {
    pub title: String,
    pub time: String,
    pub number: String,
    pub items: Vec<JsItem>,
    pub total: f64,
    pub discount: Option<f64>,
    pub footer: JsFooter,
    pub port: Option<String>,
    pub baud: Option<u32>,
}

// ===================== Text shaping/measurement =====================
fn shape(s: &str) -> String { reshape_line(s) }

// Treat spaces as neutral (do NOT force them into LTR).
fn is_ltr_char(c: char) -> bool {
    if c == ' ' || c == '\u{00A0}' { return false; }              // neutral spaces
    if c.is_ascii_alphanumeric() { return true; }                  // Latin letters/digits
    if ('\u{0660}'..='\u{0669}').contains(&c)                      // Arabic-Indic digits
        || ('\u{06F0}'..='\u{06F9}').contains(&c) { return true; }
    matches!(c, ':'|'.'|','|'-'|'–'|'—'|'/')                       // some punctuation as LTR
}

// Accurate width including spaces using rusttype layout
fn measure(scale: Scale, font: &Font, s: &str) -> i32 {
    let mut x = 0.0f32;
    for g in font.layout(s, scale, point(0.0, 0.0)) {
        x += g.unpositioned().h_metrics().advance_width;
    }
    x.round() as i32
}

fn draw_crisp(img: &mut RgbImage, s: &str, x: i32, y: i32, scale: Scale, font: &Font) {
    draw_text_mut(img, Rgb([0,0,0]), x, y, scale, font, s);
}

fn draw_ltr_right(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, x_right: i32, y: i32) {
    let w = measure(scale, font, s);
    draw_crisp(img, s, x_right - w, y, scale, font);
}

fn draw_ltr_center(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, paper_w: i32, y: i32) {
    let w = measure(scale, font, s);
    draw_crisp(img, s, (paper_w - w)/2, y, scale, font);
}

// Mixed RTL/LTR drawing (right aligned). Spaces are preserved.
fn draw_mixed_rtl_right(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, x_right: i32, y: i32) {
    let shaped = shape(logical);
    // Segment into runs based on LTR/RTL; spaces join to previous run to preserve spacing.
    let mut runs: Vec<(bool, String)> = Vec::new(); // (is_ltr, text)
    let mut cur = String::new();
    let mut cur_is_ltr: Option<bool> = None;

    for ch in shaped.chars() {
        let is_space = ch == ' ' || ch == '\u{00A0}';
        let ltr = if is_space { cur_is_ltr.unwrap_or(false) } else { is_ltr_char(ch) };
        match cur_is_ltr {
            None => { cur_is_ltr = Some(ltr); cur.push(ch); }
            Some(kind) if kind == ltr || is_space => cur.push(ch),
            Some(_) => { runs.push((cur_is_ltr.unwrap(), cur.clone())); cur.clear(); cur_is_ltr = Some(ltr); cur.push(ch); }
        }
    }
    if !cur.is_empty() { runs.push((cur_is_ltr.unwrap_or(false), cur)); }

    let total_w: i32 = runs.iter().map(|(_, t)| measure(scale, font, t)).sum();
    let mut right = x_right;

    for (is_ltr, seg) in runs.into_iter() {
        let seg_w = measure(scale, font, &seg);
        if is_ltr {
            draw_ltr_right(img, font, scale, &seg, right, y);
        } else {
            // Draw RTL char-by-char (right to left), keeping spaces
            let mut x = right - seg_w;
            for ch in seg.chars().rev() {
                let s = ch.to_string();
                let cw = measure(scale, font, &s);
                draw_crisp(img, &s, x, y, scale, font);
                x += cw;
            }
        }
        right -= seg_w;
    }
}

fn draw_mixed_rtl_center(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, paper_w: i32, y: i32) {
    let shaped = shape(logical);
    let w = measure(scale, font, &shaped);
    let x = (paper_w - w)/2;
    draw_mixed_rtl_right(img, font, scale, &shaped, x + w, y);
}

// Simple dotted separator
fn draw_dotted(img: &mut RgbImage, y: i32, left: i32, right: i32) {
    let y = y.max(0) as u32;
    let mut x = left.max(0);
    while x < right {
        for dx in 0..3 {
            if x + dx < right { img.put_pixel((x + dx) as u32, y, Rgb([0,0,0])); }
        }
        x += 10;
    }
}

// ====== Wrapping (max 2 lines with ellipsis) ======
fn wrap_mixed_rtl(font: &Font, scale: Scale, logical: &str, max_w: i32) -> Vec<String> {
    // Keep whitespace tokens with split_inclusive so we never drop spaces
    let tokens: Vec<&str> = logical.split_inclusive(char::is_whitespace).collect();
    let mut out: Vec<String> = Vec::new();
    let mut line = String::new();

    for tok in tokens {
        let test = format!("{}{}", line, tok);
        let test_w = measure(scale, font, &shape(&test));
        if test_w <= max_w || line.is_empty() {
            line.push_str(tok);
        } else {
            out.push(line.trim_end().to_string());
            line = tok.to_string();
            if out.len() == 2 { break; }
        }
    }
    if out.len() < 2 && !line.is_empty() {
        out.push(line.trim_end().to_string());
    }

    // If more than 2 lines needed, ellipsize the second
    if out.len() > 2 {
        out.truncate(2);
    }
    if out.len() == 2 {
        // ensure second line fits with ellipsis if needed
        let ell = "…";
        let mut s2 = out[1].clone();
        while measure(scale, font, &shape(&(s2.clone() + ell))) > max_w && !s2.is_empty() {
            s2.pop();
        }
        out[1] = format!("{}{}", s2.trim_end(), ell);
    }
    out
}

// ===================== Rendering =====================
fn render_receipt(data: &ReceiptData, layout: &Layout) -> GrayImage {
    let paper_w = layout.paper_width_px as i32;
    let mut img: RgbImage = ImageBuffer::from_pixel(layout.paper_width_px, 2000, Rgb([255,255,255]));
    let margin_h = layout.margin_h;
    let inner_w = paper_w - margin_h*2;
    let right_edge = margin_h + inner_w;
    let mut y = layout.margin_top;

    let font_bytes = include_bytes!("fonts/NotoSansArabic-Regular.ttf");
    let font = Font::try_from_bytes(font_bytes).expect("font");

    // Title
    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.title), &data.store_name, paper_w, y);
    y += layout.fonts.title as i32 - 8;

    // Date/Time
    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.header_dt), &data.date_time_line, paper_w, y);
    y += layout.fonts.header_dt as i32 + 2;

    // Receipt number (centered, plain LTR digits)
    draw_ltr_center(&mut img, &font, Scale::uniform(layout.fonts.header_no), &data.invoice_no, paper_w, y);
    y += layout.fonts.header_no as i32 + 2;

    // Columns
    let w_name  = (inner_w as f32 * layout.cols[0]) as i32;
    let w_qty   = (inner_w as f32 * layout.cols[1]) as i32;
    let w_price = (inner_w as f32 * layout.cols[2]) as i32;
    let w_total = (inner_w as f32 * layout.cols[3]) as i32;

    let r_name  = right_edge;
    let r_qty   = r_name  - w_name;
    let r_price = r_qty   - w_qty;
    let r_total = r_price - w_total;

    // Headings
    let s_head = Scale::uniform(layout.fonts.header_cols);
    draw_mixed_rtl_right(&mut img, &font, s_head, "الصنف",  r_name,  y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "الكمية", r_qty,   y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "السعر",  r_price, y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "القيمة", r_total, y);
    y += layout.row_gap - 6;

    // Rows with wrapping (max 2 lines for name)
    let s_item = Scale::uniform(layout.fonts.item);
    for it in &data.items {
        let lines = wrap_mixed_rtl(&font, s_item, &it.name, w_name).into_iter().take(2).collect::<Vec<_>>();
        let line_count = lines.len().max(1);

        for (i, ln) in lines.iter().enumerate() {
            let yy = y + (i as i32) * (layout.row_gap - 4);

            // Name (RTL mixed, wrapped)
            draw_mixed_rtl_right(&mut img, &font, s_item, ln, r_name, yy);

            // Other columns only on the first visual line
            if i == 0 {
                draw_ltr_right(&mut img, &font, s_item, &it.qty_str, r_qty, yy);
                draw_ltr_right(&mut img, &font, s_item, &format!("{:.2}", it.price), r_price, yy);
                draw_ltr_right(&mut img, &font, s_item, &format!("{:.2}", it.total), r_total, yy);
            }
        }

        y += (line_count as i32) * (layout.row_gap - 4);
    }

    // Separator
    y += 18;
    draw_dotted(&mut img, y, margin_h, paper_w - margin_h);
    y += 12;

    // Discount (optional)
    if data.discount > 0.0001 {
        let gap = 12;
        let label = "الخصم";
        let lw = measure(Scale::uniform(layout.fonts.total_label), &font, &shape(label));
        let right = right_edge;
        draw_ltr_right(&mut img, &font, Scale::uniform(layout.fonts.total_label),
                       &format!("{:.2}", data.discount), right - lw - gap, y);
        draw_mixed_rtl_right(&mut img, &font, Scale::uniform(layout.fonts.total_label), label, right, y);
        y += layout.row_gap - 6;
    }

    // Grand total
    let gap = 12;
    let label = "إجمالي الفاتورة";
    let lw = measure(Scale::uniform(layout.fonts.total_label), &font, &shape(label));
    let right = right_edge;
    draw_ltr_right(&mut img, &font, Scale::uniform(layout.fonts.total_value),
                   &format!("{:.2}", data.total), right - lw - gap, y - 10);
    draw_mixed_rtl_right(&mut img, &font, Scale::uniform(layout.fonts.total_label), label, right, y);
    y += layout.row_gap;

    // Footer
    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.footer), &data.footer_address,  paper_w, y);
    y += layout.fonts.footer as i32 + 2;

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.footer), &data.footer_delivery, paper_w, y);
    y += layout.fonts.footer as i32 + 2;

    if !data.footer_phones.is_empty() {
        draw_ltr_center(&mut img, &font, Scale::uniform(layout.fonts.footer_phones), &data.footer_phones, paper_w, y);
        y += layout.fonts.footer_phones as i32 + 2;
    }

    y += layout.margin_bottom;

    // Crop & grayscale
    let used_h = (y as u32).min(1998);
    DynamicImage::ImageRgb8(img)
        .crop_imm(0, 0, layout.paper_width_px, used_h)
        .to_luma8()
}

// ===================== ESC * 24 band pack =====================
fn pack_esc_star_24(gray: &GrayImage, y0: u32, threshold: u8) -> Vec<u8> {
    let w = gray.width();
    let h = gray.height();
    let mut band = Vec::with_capacity((w * 3) as usize);
    for x in 0..w {
        for byte in 0..3 {
            let mut b = 0u8;
            for bit in 0..8 {
                let yy = y0 + (byte * 8 + bit) as u32;
                if yy < h {
                    let Luma([pix]) = *gray.get_pixel(x, yy);
                    if pix <= threshold { b |= 1 << (7 - bit); }
                }
            }
            band.push(b);
        }
    }
    band
}

// ===================== N-API entry =====================
#[napi(js_name = "printReceipt")]
pub async fn print_receipt(payload: JsPrintPayload) -> Result<String> {
    // Convert payload to internal structs
    let items: Vec<Item> = payload.items.into_iter()
        .map(|i| Item { name: i.name, qty_str: i.qty, price: i.price as f32, total: i.total as f32 })
        .collect();

    let data = ReceiptData {
        store_name: payload.title,
        date_time_line: payload.time,
        invoice_no: payload.number,
        items,
        discount: payload.discount.unwrap_or(0.0) as f32,
        total: payload.total as f32,
        footer_address: payload.footer.address,
        footer_delivery: payload.footer.last_line,
        footer_phones: payload.footer.phones.unwrap_or_default(),
    };

    let layout = Layout::default();
    let port = env_port_or_default(payload.port);
    let baud = env_baud_or_default(payload.baud);

    // Blocking I/O in spawn_blocking to satisfy Send bounds
    let res = napi::tokio::task::spawn_blocking(move || -> Result<String> {
        let driver = SerialPortDriver::open(&port, baud, None)
            .map_err(|e| Error::from_reason(format!("open {} @{}: {}", port, baud, e)))?;

        let mut obj = Printer::new(driver, Protocol::default(), None);
        obj.debug_mode(None);
        let mut p = obj.init().map_err(|e| Error::from_reason(e.to_string()))?;

        let gray = render_receipt(&data, &layout);

        // ESC * 24-dot double density
        let w = gray.width();
        let n = w as u16;
        let nL = (n & 0xFF) as u8;
        let nH = ((n >> 8) & 0xFF) as u8;

        let mut y0 = 0u32;
        while y0 < gray.height() {
            let band = pack_esc_star_24(&gray, y0, layout.threshold);
            p = p.custom(&[0x1B, 0x2A, 33, nL, nH]).map_err(|e| Error::from_reason(e.to_string()))?;
            p = p.custom(&band).map_err(|e| Error::from_reason(e.to_string()))?;
            p = p.custom(&[0x0A]).map_err(|e| Error::from_reason(e.to_string()))?;
            y0 += 24;
        }

        p = p.custom(&[0x0A]).map_err(|e| Error::from_reason(e.to_string()))?;
        p = p.print_cut().map_err(|e| Error::from_reason(e.to_string()))?;
        p.print().map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(format!("✅ Receipt printed on {}", port))
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("join error: {e}")))??;

    Ok(res)
}