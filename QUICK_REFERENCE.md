# Quick Reference Card

## 📦 Installation (For Users)

```bash
pnpm add github:YOUR_USERNAME/pos-receipt-printer#v0.1.0
```

## 🚀 Usage (Electron)

```typescript
// Main Process
import { ipcMain } from "electron";
import { printReceipt } from "pos-receipt-printer";

ipcMain.handle("print-receipt", (_e, payload) => printReceipt(payload));

// Preload
import { contextBridge, ipcRenderer } from "electron";
contextBridge.exposeInMainWorld("printer", {
  printReceipt: (payload) => ipcRenderer.invoke("print-receipt", payload),
});

// Renderer
await window.printer.printReceipt({
  title: "اسواق ابو عمر",
  time: "٤ نوفمبر - ٤:٠٩ صباحا",
  number: "123456",
  items: [{ name: "عرض تفاح", qty: 0.96, price: 70, total: 67.20 }],
  total: 67.20,
  footer: {
    address: "دمياط الجديدة - المركزية",
    lastLine: "خدمة توصيل ٢٤ ساعة",
    phones: "01533333161"
  }
});
```

## 🏗️ Building & Release (For Maintainers)

### Local Build
```bash
pnpm install
pnpm build
```

### Create Release
```bash
# 1. Commit all changes
git add .
git commit -m "feat: your changes"
git push

# 2. Tag and push
git tag v0.1.0
git push origin v0.1.0

# 3. CI builds automatically!
# Check: https://github.com/YOUR_USERNAME/pos-receipt-printer/actions
```

## 🔧 Configuration

### Environment Variables
```bash
# Windows
set PRINTER_COM_PORT=COM7
set PRINTER_BAUD_RATE=9600

# macOS/Linux
export PRINTER_COM_PORT=/dev/ttyUSB0
export PRINTER_BAUD_RATE=9600
```

### Or in Code
```typescript
await printReceipt({
  ...payload,
  port: "COM7",
  baud: 9600
});
```

### Electron Builder
```json
{
  "build": {
    "asarUnpack": ["**/*.node"]
  }
}
```

## 📁 Key Files

| File | Purpose |
|------|---------|
| `index.js` | Native module loader |
| `index.d.ts` | TypeScript definitions |
| `src/lib.rs` | Main Rust implementation |
| `.github/workflows/release.yml` | CI/CD pipeline |
| `package.json` | NPM config with install script |
| `Cargo.toml` | Rust dependencies |

## 🎯 Supported Platforms

- ✅ Windows x64
- ✅ macOS Apple Silicon (ARM64)
- ✅ macOS Intel (x64)
- ✅ Linux x64

## 💡 Tips

1. **First time setup**: Install `@napi-rs/cli` globally
   ```bash
   npm i -g @napi-rs/cli
   ```

2. **Test before releasing**: Build locally first
   ```bash
   pnpm build
   ```

3. **Version tags**: Must start with 'v' (e.g., `v0.1.0`)

4. **Check CI logs**: Monitor builds at GitHub Actions

5. **Fallback**: If pre-built unavailable, installs build from source (needs Rust)

## 🐛 Common Issues

**"Cannot find module"**
- Add `"asarUnpack": ["**/*.node"]` to electron-builder config

**"Port not found"**
- Check COM port name (Windows: `COM7`, Linux: `/dev/ttyUSB0`)
- Set environment variables or pass in payload

**Build fails**
- Ensure Rust toolchain installed
- Check font file exists: `src/fonts/NotoSansArabic-Regular.ttf`
- Verify all dependencies in `Cargo.toml`

## 📚 Full Documentation

- [README.md](README.md) - Complete guide
- [RELEASE.md](RELEASE.md) - Release process
- [SETUP_COMPLETE.md](SETUP_COMPLETE.md) - Setup details
- [examples/usage.ts](examples/usage.ts) - Code examples

---

**Need help?** Check the full documentation files above or open an issue on GitHub.
