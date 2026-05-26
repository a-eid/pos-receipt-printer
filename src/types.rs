//! Type definitions for the element-based receipt printing API.
//!
//! This module defines all the N-API structs that are exposed to JavaScript/TypeScript,
//! as well as the internal Rust enums used for dispatch and rendering.

use napi_derive::napi;

// ===================== Text Styling =====================

/// Text alignment options for receipt elements.
#[derive(Clone, Debug, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl Alignment {
    /// Parse alignment from a JS string, defaulting to Left.
    pub fn from_js(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("center") => Self::Center,
            Some("right") => Self::Right,
            _ => Self::Left,
        }
    }
}

/// Text styling options that can be applied to text and table elements.
#[derive(Clone, Debug)]
pub struct TextStyle {
    /// Font selection: "A" (default, larger) or "B" (smaller).
    pub font: FontType,
    /// Bold/emphasis mode.
    pub bold: bool,
    /// Underline mode.
    pub underline: bool,
    /// Text alignment.
    pub align: Alignment,
    /// Width multiplier (1-8). Affects rendering scale.
    pub width: u32,
    /// Height multiplier (1-8). Affects rendering scale.
    pub height: u32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: FontType::A,
            bold: false,
            underline: false,
            align: Alignment::Left,
            width: 1,
            height: 1,
        }
    }
}

/// Font type selection for thermal printers.
#[derive(Clone, Debug, Default)]
pub enum FontType {
    /// Standard font (default, ~42 chars per line on 80mm paper).
    #[default]
    A,
    /// Condensed font (~56 chars per line on 80mm paper).
    B,
}

impl FontType {
    /// Parse font type from a JS string, defaulting to Font A.
    pub fn from_js(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("B") | Some("b") => Self::B,
            _ => Self::A,
        }
    }
}

// ===================== Table Structures =====================

/// Column definition for table elements.
#[derive(Clone, Debug)]
pub struct TableColumn {
    /// Column header label.
    pub label: String,
    /// Column width as a fraction of the inner paper width (0.0 - 1.0).
    pub width: f64,
    /// Column alignment (overrides default left).
    pub align: Alignment,
}

/// A single cell in a table row.
#[derive(Clone, Debug)]
pub struct TableCell {
    /// Cell content as a string.
    pub value: String,
}

/// A row in a table body.
#[derive(Clone, Debug)]
pub struct TableRow {
    /// Cell values, one per column.
    pub cells: Vec<TableCell>,
}

// ===================== Element Types =====================

/// Barcode format types supported by ESC/POS.
#[derive(Clone, Debug, Default)]
pub enum BarcodeFormat {
    #[default]
    Code128,
    Code39,
    Ean13,
    Ean8,
    UpcA,
    UpcE,
    Itf,
    Codabar,
}

impl BarcodeFormat {
    /// Parse barcode format from a JS string, defaulting to Code128.
    pub fn from_js(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("CODE39") | Some("code39") => Self::Code39,
            Some("EAN13") | Some("ean13") => Self::Ean13,
            Some("EAN8") | Some("ean8") => Self::Ean8,
            Some("UPC_A") | Some("upc_a") => Self::UpcA,
            Some("UPC_E") | Some("upc_e") => Self::UpcE,
            Some("ITF") | Some("itf") => Self::Itf,
            Some("CODABAR") | Some("codabar") => Self::Codabar,
            _ => Self::Code128,
        }
    }

    /// Get the ESC/POS barcode type code for GS k command.
    pub fn escpos_code(&self) -> u8 {
        match self {
            Self::UpcA => 65,
            Self::UpcE => 66,
            Self::Ean13 => 67,
            Self::Ean8 => 68,
            Self::Code39 => 69,
            Self::Itf => 70,
            Self::Codabar => 71,
            Self::Code128 => 73,
        }
    }
}

/// Human-readable interpretation (HRI) position for barcodes.
#[derive(Clone, Debug, Default)]
pub enum HriPosition {
    /// No text printed with barcode.
    None,
    /// Text above the barcode.
    Above,
    /// Text below the barcode (default for most use cases).
    #[default]
    Below,
    /// Text both above and below.
    Both,
}

impl HriPosition {
    /// Parse HRI position from a JS string, defaulting to Below.
    pub fn from_js(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("none") => Self::None,
            Some("above") => Self::Above,
            Some("both") => Self::Both,
            _ => Self::Below,
        }
    }

    /// Get the ESC/POS HRI position code.
    pub fn escpos_code(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Above => 1,
            Self::Below => 2,
            Self::Both => 3,
        }
    }
}

/// QR code error correction levels.
#[derive(Clone, Debug, Default)]
pub enum QrErrorCorrection {
    /// Low (7% recovery).
    L,
    /// Medium (15% recovery) — default.
    #[default]
    M,
    /// Quartile (25% recovery).
    Q,
    /// High (30% recovery).
    H,
}

impl QrErrorCorrection {
    /// Parse error correction from a JS string, defaulting to M.
    pub fn from_js(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("L") | Some("l") => Self::L,
            Some("Q") | Some("q") => Self::Q,
            Some("H") | Some("h") => Self::H,
            _ => Self::M,
        }
    }
}

// ===================== N-API Structs =====================

/// N-API text style options exposed to JavaScript.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsTextStyle {
    /// Font type: "A" or "B". Defaults to "A".
    pub font: Option<String>,
    /// Enable bold/emphasis. Defaults to false.
    pub bold: Option<bool>,
    /// Enable underline. Defaults to false.
    pub underline: Option<bool>,
    /// Text alignment: "left", "center", or "right". Defaults to "left".
    pub align: Option<String>,
    /// Width multiplier (1-8). Defaults to 1.
    pub width: Option<u32>,
    /// Height multiplier (1-8). Defaults to 1.
    pub height: Option<u32>,
}

impl JsTextStyle {
    /// Convert JS style to internal TextStyle, applying defaults.
    pub fn to_internal(&self) -> TextStyle {
        TextStyle {
            font: FontType::from_js(self.font.clone()),
            bold: self.bold.unwrap_or(false),
            underline: self.underline.unwrap_or(false),
            align: Alignment::from_js(self.align.clone()),
            width: self.width.unwrap_or(1).clamp(1, 8),
            height: self.height.unwrap_or(1).clamp(1, 8),
        }
    }
}

/// N-API table column definition exposed to JavaScript.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsTableColumn {
    /// Column header label.
    pub label: String,
    /// Column width as a fraction of inner paper width (0.0 - 1.0).
    pub width: f64,
    /// Column alignment: "left", "center", or "right". Defaults to "left".
    pub align: Option<String>,
}

impl JsTableColumn {
    /// Convert JS column to internal TableColumn.
    pub fn to_internal(&self) -> TableColumn {
        TableColumn {
            label: self.label.clone(),
            width: self.width.clamp(0.0, 1.0),
            align: Alignment::from_js(self.align.clone()),
        }
    }
}

/// N-API table row exposed to JavaScript.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsTableRow {
    /// Cell values, one per column.
    pub cells: Vec<String>,
}

impl JsTableRow {
    /// Convert JS row to internal TableRow.
    pub fn to_internal(&self) -> TableRow {
        TableRow {
            cells: self
                .cells
                .iter()
                .map(|v| TableCell { value: v.clone() })
                .collect(),
        }
    }
}

/// The main print element — a tagged union of all supported element types.
///
/// Each element has a `type` field that determines which fields are relevant.
/// Unknown types are silently skipped.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct JsPrintElement {
    /// Element type. Required. One of:
    /// - "text": Text with optional styling
    /// - "table": Table with columns and rows
    /// - "qrcode": QR code from a string value
    /// - "barcode": 1D barcode (Code128, EAN13, etc.)
    /// - "image": Image from file path
    /// - "divider": Horizontal separator line
    /// - "feed": Feed paper N lines
    /// - "cut": Paper cut (full or partial)
    pub r#type: String,

    // --- Text element fields ---
    /// Text content (for "text" type).
    pub value: Option<String>,
    /// Text style (for "text" and "table" types).
    pub style: Option<JsTextStyle>,

    // --- Table element fields ---
    /// Column definitions (for "table" type).
    pub columns: Option<Vec<JsTableColumn>>,
    /// Row data (for "table" type).
    pub rows: Option<Vec<JsTableRow>>,

    // --- QR code fields ---
    /// QR code module size in dots (1-16). Default: 4.
    pub size: Option<u32>,
    /// Error correction level: "L", "M", "Q", or "H". Default: "M".
    pub error_correction: Option<String>,

    // --- Barcode fields ---
    /// Barcode format: "CODE128", "CODE39", "EAN13", etc. Default: "CODE128".
    pub format: Option<String>,
    /// Barcode height in dots (1-255). Default: 80.
    pub height_dots: Option<u32>,
    /// Barcode width multiplier (1-6). Default: 2.
    pub width_mult: Option<u32>,
    /// HRI (text) position: "none", "above", "below", "both". Default: "below".
    pub hri: Option<String>,

    // --- Image fields ---
    /// File path to image (for "image" type).
    pub path: Option<String>,
    /// Image width in pixels. Default: auto (full paper width).
    pub width_px: Option<u32>,
    /// Image alignment: "left", "center", or "right". Default: "center".
    pub align: Option<String>,

    // --- Divider fields ---
    /// Character to use for divider. Default: "-".
    pub char: Option<String>,

    // --- Feed fields ---
    /// Number of lines to feed (for "feed" type). Default: 1.
    pub lines: Option<u32>,

    // --- Cut fields ---
    /// If true, perform a partial cut (leaves a small bridge). Default: false (full cut).
    pub partial: Option<bool>,
}

/// Print options for the receipt printer.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct JsPrintOptions {
    /// Serial port path (e.g., "COM7", "/dev/ttyUSB0").
    /// Defaults to PRINTER_COM_PORT env var or "COM7".
    pub port: Option<String>,
    /// Baud rate. Defaults to PRINTER_BAUD_RATE env var or 9600.
    pub baud: Option<u32>,
    /// Paper width in pixels. Default: 576 (80mm thermal paper).
    pub paper_width_px: Option<u32>,
    /// Threshold for converting grayscale to black/white (0-255). Default: 150.
    pub threshold: Option<u8>,
}

/// Internal print options with resolved defaults.
#[derive(Clone, Debug)]
pub struct PrintOptions {
    pub port: String,
    pub baud: u32,
    pub paper_width_px: u32,
    pub threshold: u8,
    pub margin_h: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub row_gap: i32,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            port: String::from("COM7"),
            baud: 9600,
            paper_width_px: 576,
            threshold: 150,
            margin_h: 0,
            margin_top: -28,
            margin_bottom: 0,
            row_gap: 32,
        }
    }
}

/// Render context passed to all element renderers.
/// Contains the current Y position, font, paper dimensions, and style state.
#[derive(Clone)]
pub struct RenderContext<'a> {
    /// Paper width in pixels.
    pub paper_width_px: i32,
    /// Horizontal margin in pixels.
    pub margin_h: i32,
    /// Inner width (paper_width - 2 * margin_h).
    pub inner_width: i32,
    /// Right edge x-coordinate.
    pub right_edge: i32,
    /// Current Y position (advances as elements are rendered).
    pub y: i32,
    /// Row gap between lines in pixels.
    pub row_gap: i32,
    /// Binarization threshold.
    pub threshold: u8,
    /// The loaded font.
    pub font: &'a rusttype::Font<'a>,
}

/// Barcode command to be sent as raw ESC/POS after the bit-image.
pub enum BarcodeCommand {
    Code128 {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    Code39 {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    Ean13 {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    Ean8 {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    UpcA {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    UpcE {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    Itf {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
    Codabar {
        data: String,
        height: u8,
        width: u8,
        hri: u8,
    },
}

impl BarcodeCommand {
    pub fn to_escpos_bytes(&self) -> Vec<u8> {
        let mut cmds = Vec::new();

        let (data, type_code, height, width_mult, hri_pos) = match self {
            Self::Code128 {
                data,
                height,
                width,
                hri,
            } => (data, 73u8, *height, *width, *hri),
            Self::Code39 {
                data,
                height,
                width,
                hri,
            } => (data, 69u8, *height, *width, *hri),
            Self::Ean13 {
                data,
                height,
                width,
                hri,
            } => (data, 67u8, *height, *width, *hri),
            Self::Ean8 {
                data,
                height,
                width,
                hri,
            } => (data, 68u8, *height, *width, *hri),
            Self::UpcA {
                data,
                height,
                width,
                hri,
            } => (data, 65u8, *height, *width, *hri),
            Self::UpcE {
                data,
                height,
                width,
                hri,
            } => (data, 66u8, *height, *width, *hri),
            Self::Itf {
                data,
                height,
                width,
                hri,
            } => (data, 70u8, *height, *width, *hri),
            Self::Codabar {
                data,
                height,
                width,
                hri,
            } => (data, 71u8, *height, *width, *hri),
        };

        cmds.push(0x0A);
        cmds.push(0x0A);
        cmds.extend_from_slice(&[0x1B, 0x61, 0x01]);
        cmds.extend_from_slice(&[0x1D, 0x68, height]);
        cmds.extend_from_slice(&[0x1D, 0x77, width_mult]);
        cmds.extend_from_slice(&[0x1D, 0x48, hri_pos]);

        if type_code == 73 {
            let mut payload = vec![123u8, 66u8];
            payload.extend_from_slice(data.as_bytes());
            cmds.extend_from_slice(&[0x1D, 0x6B, type_code, payload.len() as u8]);
            cmds.extend(payload);
        } else {
            cmds.extend_from_slice(&[0x1D, 0x6B, type_code, data.len() as u8]);
            cmds.extend_from_slice(data.as_bytes());
        }

        cmds.extend_from_slice(&[0x1B, 0x61, 0x00]);
        cmds
    }
}

/// Render result containing the image and any barcode commands.
pub struct RenderResult {
    pub image: image::GrayImage,
    pub barcodes: Vec<BarcodeCommand>,
}
