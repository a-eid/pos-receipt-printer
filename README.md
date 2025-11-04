# pos-receipt-printer

ESC/POS receipt printing (serial) with Arabic shaping for Electron/Node.js

## Features

- ✅ Native N-API bindings (no Python/GYP required)
- ✅ Pre-built binaries for Windows, macOS (Intel & ARM), and Linux
- ✅ Arabic text shaping and RTL support
- ✅ Serial port communication for thermal printers
- ✅ Works with Electron apps
- ✅ Zero runtime dependencies

## Installation

```bash
npm install github:YOUR_USERNAME/pos-receipt-printer#v0.1.0
```

Or with pnpm:

```bash
pnpm add github:YOUR_USERNAME/pos-receipt-printer#v0.1.0
```

The install script will automatically download the pre-built binary for your platform. If no pre-built binary is available, it will attempt to build from source (requires Rust).

## Usage

### In Electron

**Main Process:**

```typescript
import { app, ipcMain } from "electron";
import { printReceipt } from "pos-receipt-printer";

ipcMain.handle("print-receipt", (_event, payload) => printReceipt(payload));
```

**Preload Script:**

```typescript
import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("printer", {
  printReceipt: (payload) => ipcRenderer.invoke("print-receipt", payload),
});
```

**Renderer Process:**

```typescript
await window.printer.printReceipt({
  title: "اسواق ابو عمر",
  time: "٤ نوفمبر - ٤:٠٩ صباحا",
  number: "123456",
  items: [
    { name: "عرض تفاح", qty: 0.96, price: 70, total: 67.20 }
  ],
  total: 67.20,
  footer: {
    address: "دمياط الجديدة - المركزية - مقابل البنك الأهلي القديم",
    lastLine: "خدمة توصيل للمنازل ٢٤ ساعة",
    phones: "01533333161 - 01533333262"
  }
});
```

### Electron Builder Configuration

Add to your `electron-builder` config to ensure `.node` files are not packed into asar:

```json
{
  "asarUnpack": ["**/*.node"]
}
```

## API

### `printReceipt(payload: PrintPayload): Promise<string>`

Prints a receipt to the specified serial port.

**Payload:**

```typescript
interface PrintPayload {
  title: string;          // Store name
  time: string;           // Date/time line
  number: string;         // Receipt number
  items: Item[];          // Line items
  total: number;          // Grand total
  discount?: number;      // Optional discount
  footer: Footer;         // Footer information
  port?: string;          // Optional COM port (defaults to COM7 or env var)
  baud?: number;          // Optional baud rate (defaults to 9600)
}

interface Item {
  name: string;
  qty: string | number;
  price: number;
  total: number;
}

interface Footer {
  address: string;
  lastLine: string;
  phones?: string;
}
```

## Environment Variables

- `PRINTER_COM_PORT` - Default serial port (e.g., `COM7` on Windows, `/dev/ttyUSB0` on Linux)
- `PRINTER_BAUD_RATE` - Default baud rate (default: `9600`)

## Development

### Prerequisites

- Node.js 16+
- Rust toolchain
- pnpm (recommended)

### Build from Source

```bash
pnpm install
pnpm build
```

### Create a Release

1. Commit your changes
2. Create and push a tag:

```bash
git tag v0.1.0
git push --tags
```

3. GitHub Actions will automatically build binaries for all platforms and attach them to the release

## License

MIT
