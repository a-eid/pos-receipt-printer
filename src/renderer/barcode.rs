use crate::types::{BarcodeCommand, BarcodeFormat, HriPosition, JsPrintElement, RenderContext};

pub fn render(elem: &JsPrintElement, _ctx: &mut RenderContext) -> Option<BarcodeCommand> {
    let value = match &elem.value {
        Some(v) if !v.is_empty() => v,
        _ => return None,
    };

    let format = elem
        .format
        .as_ref()
        .map(|f| BarcodeFormat::from_js(Some(f.clone())))
        .unwrap_or_default();

    let hri = elem
        .hri
        .as_ref()
        .map(|h| HriPosition::from_js(Some(h.clone())))
        .unwrap_or_default();

    let height = elem.height_dots.unwrap_or(80).clamp(1, 255) as u8;
    let width = elem.width_mult.unwrap_or(2).clamp(1, 6) as u8;
    let hri_code = hri.escpos_code();
    let data = value.to_string();

    Some(match format {
        BarcodeFormat::Code128 => BarcodeCommand::Code128 {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::Code39 => BarcodeCommand::Code39 {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::Ean13 => BarcodeCommand::Ean13 {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::Ean8 => BarcodeCommand::Ean8 {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::UpcA => BarcodeCommand::UpcA {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::UpcE => BarcodeCommand::UpcE {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::Itf => BarcodeCommand::Itf {
            data,
            height,
            width,
            hri: hri_code,
        },
        BarcodeFormat::Codabar => BarcodeCommand::Codabar {
            data,
            height,
            width,
            hri: hri_code,
        },
    })
}
