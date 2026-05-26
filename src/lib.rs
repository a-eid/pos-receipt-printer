use napi::bindgen_prelude::*;
use napi_derive::napi;

use escpos::{driver::SerialPortDriver, printer::Printer, utils::*};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, GrayImage, Luma};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale, point};
use ar_reshaper::reshape_line;
use serde::Deserialize;

pub mod renderer;
pub mod types;

use crate::types::{JsPrintElement, JsPrintOptions, PrintOptions};
use crate::renderer::render_elements;

const DEFAULT_COM_PORT: &str = "COM7";
const DEFAULT_BAUD_RATE: u32 = 9600;

pub(crate) fn env_port_or_default(port: Option<String>) -> String {
    if let Some(p) = port { return normalize_com_port(&p); }
    if let Ok(p) = std::env::var("PRINTER_COM_PORT") { return normalize_com_port(&p); }
    normalize_com_port(DEFAULT_COM_PORT)
}
pub(crate) fn env_baud_or_default(baud: Option<u32>) -> u32 {
    if let Some(b) = baud { return b; }
    std::env::var("PRINTER_BAUD_RATE").ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_BAUD_RATE)
}
pub(crate) fn normalize_com_port(port: &str) -> String {
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

pub(crate) fn shape(s: &str) -> String { reshape_line(s) }

pub(crate) fn is_ltr_char(c: char) -> bool {
    if c == ' ' || c == '\u{00A0}' { return false; }
    if c.is_ascii_alphanumeric() { return true; }
    if ('\u{0660}'..='\u{0669}').contains(&c)
        || ('\u{06F0}'..='\u{06F9}').contains(&c) { return true; }
    matches!(c, ':'|'.'|','|'-'|'–'|'—'|'/')
}

pub(crate) fn measure(scale: Scale, font: &Font, s: &str) -> i32 {
    let mut x = 0.0f32;
    for g in font.layout(s, scale, point(0.0, 0.0)) {
        x += g.unpositioned().h_metrics().advance_width;
    }
    x.round() as i32
}

pub(crate) fn draw_crisp(img: &mut RgbImage, s: &str, x: i32, y: i32, scale: Scale, font: &Font) {
    draw_text_mut(img, Rgb([0,0,0]), x, y, scale, font, s);
}

pub(crate) fn draw_ltr_right(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, x_right: i32, y: i32) {
    let w = measure(scale, font, s);
    draw_crisp(img, s, x_right - w, y, scale, font);
}

pub(crate) fn draw_ltr_center(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, paper_w: i32, y: i32) {
    let w = measure(scale, font, s);
    draw_crisp(img, s, (paper_w - w)/2, y, scale, font);
}

pub(crate) fn draw_mixed_rtl_right(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, x_right: i32, y: i32) {
    let shaped = shape(logical);
    let mut runs: Vec<(bool, String)> = Vec::new();
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

    let _total_w: i32 = runs.iter().map(|(_, t)| measure(scale, font, t)).sum();
    let mut right = x_right;

    for (is_ltr, seg) in runs.into_iter() {
        let seg_w = measure(scale, font, &seg);
        if is_ltr {
            draw_ltr_right(img, font, scale, &seg, right, y);
        } else {
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

pub(crate) fn draw_mixed_rtl_center(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, paper_w: i32, y: i32) {
    let shaped = shape(logical);
    let w = measure(scale, font, &shaped);
    let x = (paper_w - w)/2;
    draw_mixed_rtl_right(img, font, scale, &shaped, x + w, y);
}

pub(crate) fn draw_mixed_rtl_left(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, x_left: i32, y: i32) {
    let shaped = shape(logical);
    let w = measure(scale, font, &shaped);
    draw_mixed_rtl_right(img, font, scale, &shaped, x_left + w, y);
}


pub(crate) fn draw_dotted(img: &mut RgbImage, y: i32, left: i32, right: i32) {
    let y = y.max(0) as u32;
    let mut x = left.max(0);
    while x < right {
        for dx in 0..3 {
            if x + dx < right { img.put_pixel((x + dx) as u32, y, Rgb([0,0,0])); }
        }
        x += 10;
    }
}

pub(crate) fn wrap_mixed_rtl(font: &Font, scale: Scale, logical: &str, max_w: i32) -> Vec<String> {
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

    if out.len() > 2 {
        out.truncate(2);
    }
    if out.len() == 2 {
        let ell = "…";
        let mut s2 = out[1].clone();
        while measure(scale, font, &shape(&(s2.clone() + ell))) > max_w && !s2.is_empty() {
            s2.pop();
        }
        out[1] = format!("{}{}", s2.trim_end(), ell);
    }
    out
}

pub(crate) fn pack_esc_star_24(gray: &GrayImage, y0: u32, threshold: u8) -> Vec<u8> {
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

// ===================== Legacy N-API payloads =====================
#[napi(object)]
pub struct JsItem {
    pub name: String,
    pub qty: String,
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
    pub uuid: Option<String>,
    pub port: Option<String>,
    pub baud: Option<u32>,
}

// ===================== Legacy data (backward compat) =====================
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
    _uuid: Option<String>,
}

// ===================== Legacy rendering (backward compat) =====================
fn render_receipt(data: &ReceiptData) -> GrayImage {
    let paper_width_px: u32 = 576;
    let paper_w = paper_width_px as i32;
    let mut img: RgbImage = ImageBuffer::from_pixel(paper_width_px, 2000, Rgb([255,255,255]));
    let margin_h: i32 = 0;
    let inner_w = paper_w - margin_h*2;
    let right_edge = margin_h + inner_w;
    let row_gap: i32 = 32;
    let mut y: i32 = -28;

    let font_bytes = include_bytes!("fonts/NotoSansArabic-Regular.ttf");
    let font = Font::try_from_bytes(font_bytes).expect("font");

    let title_size: f32 = 90.0;
    let header_dt_size: f32 = 45.0;
    let header_no_size: f32 = 46.0;
    let header_cols_size: f32 = 42.0;
    let item_size: f32 = 44.0;
    let total_label_size: f32 = 48.0;
    let total_value_size: f32 = 66.0;
    let footer_size: f32 = 45.0;
    let footer_phones_size: f32 = 56.0;
    let cols: [f32; 4] = [0.60, 0.11, 0.12, 0.17];

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(title_size), &data.store_name, paper_w, y);
    y += title_size as i32 - 8;

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(header_dt_size), &data.date_time_line, paper_w, y);
    y += header_dt_size as i32 + 2;

    draw_ltr_center(&mut img, &font, Scale::uniform(header_no_size), &data.invoice_no, paper_w, y);
    y += header_no_size as i32 + 2;

    let w_name  = (inner_w as f32 * cols[0]) as i32;
    let w_qty   = (inner_w as f32 * cols[1]) as i32;
    let w_total = (inner_w as f32 * cols[3]) as i32;

    let r_name  = right_edge;
    let r_qty   = r_name  - w_name;
    let r_price = r_qty   - w_qty;
    let r_total = r_price - w_total;

    let s_head = Scale::uniform(header_cols_size);
    draw_mixed_rtl_right(&mut img, &font, s_head, "الصنف",  r_name,  y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "الكمية", r_qty,   y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "السعر",  r_price, y);
    draw_mixed_rtl_right(&mut img, &font, s_head, "القيمة", r_total, y);
    y += row_gap - 6;

    let s_item = Scale::uniform(item_size);
    for it in &data.items {
        let lines = wrap_mixed_rtl(&font, s_item, &it.name, w_name).into_iter().take(2).collect::<Vec<_>>();
        let line_count = lines.len().max(1);

        for (i, ln) in lines.iter().enumerate() {
            let yy = y + (i as i32) * (row_gap - 4);
            draw_mixed_rtl_right(&mut img, &font, s_item, ln, r_name, yy);
            if i == 0 {
                draw_ltr_right(&mut img, &font, s_item, &it.qty_str, r_qty, yy);
                draw_ltr_right(&mut img, &font, s_item, &format!("{:.2}", it.price), r_price, yy);
                draw_ltr_right(&mut img, &font, s_item, &format!("{:.2}", it.total), r_total, yy);
            }
        }

        y += (line_count as i32) * (row_gap - 4);
    }

    y += 18;
    draw_dotted(&mut img, y, margin_h, paper_w - margin_h);
    y += 12;

    if data.discount > 0.0001 {
        let gap = 12;
        let label = "الخصم";
        let lw = measure(Scale::uniform(total_label_size), &font, &shape(label));
        let right = right_edge;
        draw_ltr_right(&mut img, &font, Scale::uniform(total_label_size),
                       &format!("{:.2}", data.discount), right - lw - gap, y);
        draw_mixed_rtl_right(&mut img, &font, Scale::uniform(total_label_size), label, right, y);
        y += row_gap - 6;
    }

    let gap = 12;
    let label = "إجمالي الفاتورة";
    let lw = measure(Scale::uniform(total_label_size), &font, &shape(label));
    let right = right_edge;
    draw_ltr_right(&mut img, &font, Scale::uniform(total_value_size),
                   &format!("{:.2}", data.total), right - lw - gap, y - 10);
    draw_mixed_rtl_right(&mut img, &font, Scale::uniform(total_label_size), label, right, y);
    y += row_gap;

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(footer_size), &data.footer_address,  paper_w, y);
    y += footer_size as i32 + 2;

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(footer_size), &data.footer_delivery, paper_w, y);
    y += footer_size as i32 + 2;

    if !data.footer_phones.is_empty() {
        draw_ltr_center(&mut img, &font, Scale::uniform(footer_phones_size), &data.footer_phones, paper_w, y);
        y += footer_phones_size as i32 + 2;
    }

    let used_h = (y as u32).min(1998);
    DynamicImage::ImageRgb8(img)
        .crop_imm(0, 0, paper_width_px, used_h)
        .to_luma8()
}

// ===================== Shared printer send =====================
fn send_to_printer(gray: &GrayImage, barcodes: &[types::BarcodeCommand], port: &str, baud: u32, threshold: u8) -> Result<String> {
    let driver = SerialPortDriver::open(port, baud, None)
        .map_err(|e| Error::from_reason(format!("open {} @{}: {}", port, baud, e)))?;

    let mut obj = Printer::new(driver, Protocol::default(), None);
    obj.debug_mode(None);
    let mut p = obj.init().map_err(|e| Error::from_reason(e.to_string()))?;

    let w = gray.width();
    let n = w as u16;
    let n_l = (n & 0xFF) as u8;
    let n_h = ((n >> 8) & 0xFF) as u8;

    let mut y0 = 0u32;
    while y0 < gray.height() {
        let band = pack_esc_star_24(gray, y0, threshold);
        p = p.custom(&[0x1B, 0x2A, 33, n_l, n_h]).map_err(|e| Error::from_reason(e.to_string()))?;
        p = p.custom(&band).map_err(|e| Error::from_reason(e.to_string()))?;
        p = p.custom(&[0x0A]).map_err(|e| Error::from_reason(e.to_string()))?;
        y0 += 24;
    }

    for barcode in barcodes {
        let bytes = barcode.to_escpos_bytes();
        p = p.custom(&bytes).map_err(|e| Error::from_reason(e.to_string()))?;
    }

    p = p.custom(&[0x0A]).map_err(|e| Error::from_reason(e.to_string()))?;
    p = p.print_cut().map_err(|e| Error::from_reason(e.to_string()))?;
    p.print().map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(format!("✅ Receipt printed on {}", port))
}

// ===================== N-API: print (new element-based) =====================
#[napi(js_name = "print")]
pub async fn print_elements(
    elements: Vec<JsPrintElement>,
    options: Option<JsPrintOptions>,
) -> Result<String> {
    let opts = options.unwrap_or(JsPrintOptions {
        port: None,
        baud: None,
        paper_width_px: None,
        threshold: None,
    });

    let internal_opts = PrintOptions {
        port: env_port_or_default(opts.port),
        baud: env_baud_or_default(opts.baud),
        paper_width_px: opts.paper_width_px.unwrap_or(576),
        threshold: opts.threshold.unwrap_or(150),
        ..PrintOptions::default()
    };

    let port = internal_opts.port.clone();
    let baud = internal_opts.baud;
    let threshold = internal_opts.threshold;

    let res = napi::tokio::task::spawn_blocking(move || -> Result<String> {
        let result = render_elements(&elements, &internal_opts);
        send_to_printer(&result.image, &result.barcodes, &port, baud, threshold)
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("join error: {e}")))??;

    Ok(res)
}

// ===================== N-API: printReceipt (legacy) =====================
#[napi(js_name = "printReceipt")]
pub async fn print_receipt(payload: JsPrintPayload) -> Result<String> {
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
        _uuid: payload.uuid,
    };

    let port = env_port_or_default(payload.port);
    let baud = env_baud_or_default(payload.baud);

    let res = napi::tokio::task::spawn_blocking(move || -> Result<String> {
        let gray = render_receipt(&data);
        let barcodes: Vec<types::BarcodeCommand> = Vec::new();
        send_to_printer(&gray, &barcodes, &port, baud, 150)
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("join error: {e}")))??;

    Ok(res)
}