export type Qty = string | number;

export interface Item {
	name: string;
	qty: Qty;
	price: string | number;
	total: string | number;
	/** Pre-discount unit price. Accepts string or number. When set and > price, a discount line is shown. */
	originalPrice?: string | number;
}

export interface Footer {
	address: string;
	/** aka "last line" */
	lastLine: string;
	phones?: string;
}

export interface PrintPayload {
	title: string;
	time: string;
	number: string;
	items: Item[];
	total: string | number;
	discount?: string | number;
	footer: Footer;
	/** Optional UUID/nanoid to print as a 1D Code 128 barcode at the bottom */
	uuid?: string;
	/** Serial port (defaults via env PRINTER_COM_PORT or COM7 on Windows) */
	port?: string;
	/** Baud (defaults via env PRINTER_BAUD_RATE or 9600) */
	baud?: number;
}

/**
 * Print a receipt. Returns a human-readable success string.
 */
export function printReceipt(payload: PrintPayload): Promise<string>;

export as namespace PosReceiptPrinter;
