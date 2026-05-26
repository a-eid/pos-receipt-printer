// ===================== Element-Based API (new) =====================

export type Alignment = "left" | "center" | "right";

export type FontType = "A" | "B";

export interface TextStyle {
  font?: FontType;
  bold?: boolean;
  underline?: boolean;
  align?: Alignment;
  width?: number;
  height?: number;
}

export interface TableColumn {
  label: string;
  width: number;
  align?: Alignment;
}

export type BarcodeFormat =
  | "CODE128"
  | "CODE39"
  | "EAN13"
  | "EAN8"
  | "UPC_A"
  | "UPC_E"
  | "ITF"
  | "CODABAR";

export type HriPosition = "none" | "above" | "below" | "both";

export type QrErrorCorrection = "L" | "M" | "Q" | "H";

export type PrintElement =
  | { type: "text"; value: string; style?: TextStyle }
  | { type: "table"; columns: TableColumn[]; rows: string[][]; style?: TextStyle }
  | { type: "qrcode"; value: string; size?: number; error_correction?: QrErrorCorrection }
  | { type: "barcode"; value: string; format?: BarcodeFormat; height_dots?: number; width_mult?: number; hri?: HriPosition }
  | { type: "image"; path: string; width_px?: number }
  | { type: "divider" }
  | { type: "feed"; lines?: number }
  | { type: "cut"; partial?: boolean };

export interface PrintOptions {
  port?: string;
  baud?: number;
  paper_width_px?: number;
  threshold?: number;
}

export function print(elements: PrintElement[], options?: PrintOptions): Promise<string>;

// ===================== Legacy API (backward compatible) =====================

export type Qty = string | number;

export interface Item {
  name: string;
  qty: Qty;
  price: number;
  total: number;
}

export interface Footer {
  address: string;
  lastLine: string;
  phones?: string;
}

export interface PrintPayload {
  title: string;
  time: string;
  number: string;
  items: Item[];
  total: number;
  discount?: number;
  footer: Footer;
  uuid?: string;
  port?: string;
  baud?: number;
}

export function printReceipt(payload: PrintPayload): Promise<string>;
