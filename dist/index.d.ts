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
  port?: string;
  baud?: number;
}

/**
 * Print a receipt. Returns a human-readable success string.
 */
export function printReceipt(payload: PrintPayload): Promise<string>;

export as namespace PosReceiptPrinter;
