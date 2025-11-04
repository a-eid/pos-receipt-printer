use napi::bindgen_prelude::*;
use napi_derive::napi;

use escpos::{driver::SerialPortDriver, printer::Printer, utils::*};
use image::{ImageBuffer, Rgb, RgbImage, GrayImage, Luma};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use ar_reshaper::reshape_line;
use serde::Deserialize;

// ====== Defaults ======
const DEFAULT_COM_PORT: &str = "COM7";
const DEFAULT_BAUD_RATE: u32 = 9600;

// ====== Helpers ======
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

// ====== Data ======
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
    cols: [f32; 4],
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
            cols: [0.60, 0.12, 0.12, 0.16],
        }
    }
}

// ====== Frontend payload (Node) ======
// JS-facing, napi will convert JS objects into these Rust structs.
#[napi(object)]
pub struct JsItem {
    pub name: String,
    // accept qty as string in JS to keep conversion simple (you can change to f64 if you prefer)
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
    pub port: Option<String>,
    pub baud: Option<u32>,
}

// ====== Arabic shaping + drawing ======
fn shape(s: &str) -> String { reshape_line(s) }
fn draw_crisp(img: &mut RgbImage, s: &str, x: i32, y: i32, scale: Scale, font: &Font) {
    draw_text_mut(img, Rgb([0,0,0]), x, y, scale, font, s);
}
fn is_ltr_char(c: char) -> bool {
    c.is_ascii() || ('\u{0660}'..='\u{0669}').contains(&c) || ('\u{06F0}'..='\u{06F9}').contains(&c) || ":./-–—,".contains(c)
}
fn draw_mixed_rtl_right(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, x_right: i32, y: i32) {
    let shaped = shape(logical);
    let mut runs: Vec<(bool, String, i32)> = Vec::new();
    let mut cur = String::new();
    let mut cur_is_ltr = None::<bool>;

    for ch in shaped.chars() {
        let ltr = is_ltr_char(ch);
        match cur_is_ltr {
            None => { cur_is_ltr = Some(ltr); cur.push(ch); }
            Some(kind) if kind == ltr => cur.push(ch),
            Some(_) => {
                let w = if cur_is_ltr.unwrap() { text_size(scale, font, &cur).0 as i32 }
                        else { cur.chars().map(|c| text_size(scale, font, &c.to_string()).0 as i32).sum() };
                runs.push((cur_is_ltr.unwrap(), cur.clone(), w));
                cur.clear(); cur_is_ltr = Some(ltr); cur.push(ch);
            }
        }
    }
    if !cur.is_empty() {
        let w = if cur_is_ltr.unwrap() { text_size(scale, font, &cur).0 as i32 }
                else { cur.chars().map(|c| text_size(scale, font, &c.to_string()).0 as i32).sum() };
        runs.push((cur_is_ltr.unwrap(), cur.clone(), w));
    }

    let total_w: i32 = runs.iter().map(|r| r.2).sum();
    let mut right = x_right;

    for (is_ltr, seg, w) in runs.into_iter() {
        let start_x = right - w;
        if is_ltr {
            let (ww, _) = text_size(scale, font, &seg);
            draw_crisp(img, &seg, right - ww as i32, y, scale, font);
        } else {
            let chars: Vec<char> = seg.chars().collect();
            let cw: Vec<i32> = chars.iter().map(|c| text_size(scale, font, &c.to_string()).0 as i32).collect();
            let mut x = start_x;
            for i in (0..chars.len()).rev() {
                draw_crisp(img, &chars[i].to_string(), x, y, scale, font);
                x += cw[i];
            }
        }
        right -= w;
        if right < x_right - total_w { break; }
    }
}
fn draw_mixed_rtl_center(img: &mut RgbImage, font: &Font, scale: Scale, logical: &str, paper_w: i32, y: i32) {
    let shaped = shape(logical);
    let mut runs: Vec<(bool, String, i32)> = Vec::new();
    let mut cur = String::new();
    let mut cur_is_ltr = None::<bool>;
    for ch in shaped.chars() {
        let ltr = is_ltr_char(ch);
        match cur_is_ltr {
            None => { cur_is_ltr = Some(ltr); cur.push(ch); }
            Some(kind) if kind == ltr => cur.push(ch),
            Some(_) => {
                let w = if cur_is_ltr.unwrap() { text_size(scale, font, &cur).0 as i32 }
                        else { cur.chars().map(|c| text_size(scale, font, &c.to_string()).0 as i32).sum() };
                runs.push((cur_is_ltr.unwrap(), cur.clone(), w));
                cur.clear(); cur_is_ltr = Some(ltr); cur.push(ch);
            }
        }
    }
    if !cur.is_empty() {
        let w = if cur_is_ltr.unwrap() { text_size(scale, font, &cur).0 as i32 }
                else { cur.chars().map(|c| text_size(scale, font, &c.to_string()).0 as i32).sum() };
        runs.push((cur_is_ltr.unwrap(), cur.clone(), w));
    }
    let total_w: i32 = runs.iter().map(|r| r.2).sum();
    let mut right = (paper_w + total_w) / 2;
    for (is_ltr, seg, w) in runs.into_iter() {
        let start_x = right - w;
        if is_ltr {
            let (ww, _) = text_size(scale, font, &seg);
            draw_crisp(img, &seg, right - ww as i32, y, scale, font);
        } else {
            let chars: Vec<char> = seg.chars().collect();
            let cw: Vec<i32> = chars.iter().map(|c| text_size(scale, font, &c.to_string()).0 as i32).collect();
            let mut x = start_x;
            for i in (0..chars.len()).rev() {
                draw_crisp(img, &chars[i].to_string(), x, y, scale, font);
                x += cw[i];
            }
        }
        right -= w;
    }
}
fn draw_ltr_right(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, x_right: i32, y: i32) {
    let (w, _) = text_size(scale, font, s);
    draw_crisp(img, s, x_right - w as i32, y, scale, font);
}
fn draw_ltr_center(img: &mut RgbImage, font: &Font, scale: Scale, s: &str, paper_w: i32, y: i32) {
    let (w, _) = text_size(scale, font, s);
    let x = (paper_w - w as i32) / 2;
    draw_crisp(img, s, x, y, scale, font);
}
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

// ====== Rendering ======
fn render_receipt(data: &ReceiptData, layout: &Layout) -> GrayImage {
    let paper_w = layout.paper_width_px as i32;
    let mut img: RgbImage = ImageBuffer::from_pixel(layout.paper_width_px, 1800, Rgb([255,255,255]));
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

    // Receipt number
    draw_ltr_center(&mut img, &font, Scale::uniform(layout.fonts.header_no), &data.invoice_no, paper_w, y);
    y += layout.fonts.header_no as i32 + 2;

    // Columns (RTL)
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

    // Rows
    let s_item = Scale::uniform(layout.fonts.item);
    for it in &data.items {
        draw_mixed_rtl_right(&mut img, &font, s_item, &it.name,   r_name,  y);
        draw_ltr_right(&mut img,      &font, s_item, &it.qty_str, r_qty,   y);
        draw_ltr_right(&mut img,      &font, s_item, &format!("{:.2}", it.price), r_price, y);
        draw_ltr_right(&mut img,      &font, s_item, &format!("{:.2}", it.total), r_total, y);
        y += layout.row_gap;
    }

    // Dotted separator
    y += 18;
    draw_dotted(&mut img, y, margin_h, paper_w - margin_h);
    y += 12;

    // Discount
    if data.discount > 0.0001 {
        let gap = 12;
        let label = "الخصم";
        let (lw, _) = text_size(Scale::uniform(layout.fonts.total_label), &font, &shape(label));
        let right = right_edge;
        draw_ltr_right(&mut img, &font, Scale::uniform(layout.fonts.total_label),
                       &format!("{:.2}", data.discount), right - lw as i32 - gap, y);
        draw_mixed_rtl_right(&mut img, &font, Scale::uniform(layout.fonts.total_label), label, right, y);
        y += layout.row_gap - 6;
    }

    // Grand total
    let gap = 12;
    let label = "إجمالي الفاتورة";
    let (lw, _) = text_size(Scale::uniform(layout.fonts.total_label), &font, &shape(label));
    let right = right_edge;
    draw_ltr_right(&mut img, &font, Scale::uniform(layout.fonts.total_value),
                   &format!("{:.2}", data.total), right - lw as i32 - gap, y - 10);
    draw_mixed_rtl_right(&mut img, &font, Scale::uniform(layout.fonts.total_label), label, right, y);
    y += layout.row_gap;

    // Footer
    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.footer), &data.footer_address,  paper_w, y);
    y += layout.fonts.footer as i32 + 2;

    draw_mixed_rtl_center(&mut img, &font, Scale::uniform(layout.fonts.footer), &data.footer_delivery, paper_w, y);
    y += layout.fonts.footer as i32 + 2;

    if !data.footer_phones.is_empty() {
        draw_ltr_center(&mut img, &font, Scale::uniform(layout.fonts.footer_phones), &data.footer_phones, paper_w, y);
        y += layout.fonts.footer_phones as i32 + 2; // include phones in crop
    }

    y += layout.margin_bottom;

    // Crop & grayscale
    let used_h = (y as u32).min(1798);
    image::DynamicImage::ImageRgb8(img)
        .crop_imm(0, 0, layout.paper_width_px, used_h)
        .to_luma8()
}

// ====== ESC * 24 band pack ======
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

// ====== N-API entrypoint ======
#[napi(js_name = "printReceipt")]
pub async fn print_receipt(payload: JsPrintPayload) -> Result<String> {
    // Convert JS-facing payload into pure Rust data synchronously.
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

    // Run blocking serial + rendering work in a blocking task so the future is Send.
    let port_clone = port.clone();
    let res = tokio::task::spawn_blocking(move || -> Result<String> {
        let driver = SerialPortDriver::open(&port_clone, baud, None)
            .map_err(|e| Error::from_reason(format!("open {} @{}: {}", port_clone, baud, e)))?;

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

        Ok(format!("✅ Receipt printed on {}", port_clone))
    })
    .await
    .map_err(|e| Error::from_reason(format!("join error: {}", e)))??;

    Ok(res)
}
