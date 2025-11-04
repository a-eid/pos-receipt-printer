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
# Replace vX.Y.Z with the tagged version you want
npm install github:a-eid/pos-receipt-printer#vX.Y.Z
```

Or with pnpm:

```bash
pnpm add github:a-eid/pos-receipt-printer#vX.Y.Z
```

During install, a postinstall script downloads the pre-built binary for your platform from the GitHub Release that matches the package version. Local compilation is intentionally disabled; if a matching binary is not available, installation will fail with a clear error message.

### Supported Platforms

- ✅ Windows x64
- ✅ macOS Apple Silicon (ARM64)
- ✅ macOS Intel (x64)
- ✅ Linux x64

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

- Node.js 20+
- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- pnpm (recommended) or npm

### Build in CI

All official binaries are built in GitHub Actions when pushing a tag. Artifacts are attached to the corresponding GitHub Release.

### Create a Release

1. Commit your changes
2. Create and push a tag:

```bash
git tag v0.1.0
git push origin main --tags
```

3. GitHub Actions will automatically:
  - Build binaries for Windows, macOS (Intel & ARM), and Linux
  - Attach all pre-built `.node` binaries to the GitHub Release

### Testing Locally

For contributors developing this repository, you can still build locally:

```bash
pnpm i --ignore-scripts
pnpm run build:ci
```

## How It Works

1. **CI/CD Pipeline**: When you push a tag (e.g., `v0.1.0`), GitHub Actions builds native binaries for all platforms
2. **Release Assets**: The `.node` binaries are automatically attached to the GitHub Release
3. **Installation**: When users install from GitHub, a postinstall script downloads the matching pre-built binary
4. **No Local Builds**: If no pre-built binary exists for the user’s platform, installation fails explicitly (no local compilation).

## Troubleshooting

### Binary not found after install

If you see "Cannot find module", ensure:
- You're using the correct platform (Windows/macOS/Linux)
- The GitHub release has the corresponding `.node` file
- For Electron apps, `.node` files are unpacked (see `asarUnpack` config)

### Build from source fails

Ensure you have:
- Rust toolchain installed (`rustup` from [rustup.rs](https://rustup.rs/))
- Node.js 20 or higher
- On Windows: Visual Studio Build Tools
- On Linux: `build-essential` package

### Serial port access denied

On Linux, add your user to the `dialout` group:
```bash
sudo usermod -a -G dialout $USER
# Log out and back in for changes to take effect
```

## Repository

GitHub: [https://github.com/a-eid/pos-receipt-printer](https://github.com/a-eid/pos-receipt-printer)

## License

MIT
