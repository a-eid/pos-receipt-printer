// ===================================
// Electron Main Process Example
// ===================================

import { app, ipcMain } from "electron";
import { printReceipt } from "pos-receipt-printer";

app.whenReady().then(() => {
  // Register the handler
  ipcMain.handle("print-receipt", async (_event, payload) => {
    try {
      const result = await printReceipt(payload);
      return { success: true, message: result };
    } catch (error) {
      console.error("Print error:", error);
      return { success: false, error: error.message };
    }
  });
});

// ===================================
// Electron Preload Script
// ===================================

import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("printer", {
  printReceipt: (payload) => ipcRenderer.invoke("print-receipt", payload),
});

// ===================================
// Electron Renderer Process
// ===================================

// TypeScript types are automatically available
const payload = {
  title: "اسواق ابو عمر",
  time: "٤ نوفمبر - ٤:٠٩ صباحا",
  number: "123456",
  items: [
    { 
      name: "عرض تفاح", 
      qty: 0.96, 
      price: 70, 
      total: 67.20 
    },
    { 
      name: "موز", 
      qty: "1 كجم", 
      price: 30, 
      total: 30.00 
    }
  ],
  total: 97.20,
  discount: 10,
  footer: {
    address: "دمياط الجديدة - المركزية - مقابل البنك الأهلي القديم",
    lastLine: "خدمة توصيل للمنازل ٢٤ ساعة",
    phones: "01533333161 - 01533333262"
  },
  // Optional: specify port and baud rate
  port: "COM7", // or "/dev/ttyUSB0" on Linux
  baud: 9600
};

// Call the printer
const result = await window.printer.printReceipt(payload);
console.log("Print result:", result);

// ===================================
// Electron Builder Configuration
// ===================================

// Add to your electron-builder config (package.json or electron-builder.yml)
{
  "build": {
    "asarUnpack": [
      "**/*.node"
    ],
    "files": [
      "dist/**/*",
      "node_modules/**/*"
    ]
  }
}

// Or in electron-builder.yml:
// asarUnpack:
//   - "**/*.node"

// ===================================
// Environment Variables (optional)
// ===================================

// Set default COM port and baud rate
// Windows:
// set PRINTER_COM_PORT=COM7
// set PRINTER_BAUD_RATE=9600

// macOS/Linux:
// export PRINTER_COM_PORT=/dev/ttyUSB0
// export PRINTER_BAUD_RATE=9600

// ===================================
// Node.js Direct Usage (non-Electron)
// ===================================

import { printReceipt } from "pos-receipt-printer";

const payload = {
  title: "Test Store",
  time: "Nov 4, 2025 - 10:30 AM",
  number: "INV-001",
  items: [
    { name: "Item 1", qty: 2, price: 10.00, total: 20.00 }
  ],
  total: 20.00,
  footer: {
    address: "123 Main St",
    lastLine: "Thank you!",
    phones: "123-456-7890"
  }
};

try {
  const result = await printReceipt(payload);
  console.log(result); // "✅ Receipt printed on COM7"
} catch (error) {
  console.error("Failed to print:", error);
}
