// Minimal loader for the native binding.
// Exports whatever the native addon exports. The postinstall script
// downloads the correct `addon.node` into this directory as `addon.node`.
const path = require('node:path');
const fs = require('node:fs');

const candidates = [
  path.join(__dirname, 'addon.node'),
  path.join(__dirname, 'pos-receipt-printer.node'),
  path.join(__dirname, 'pos_receipt_printer.node'),
  // historically some builds used dist/ folder
  path.join(__dirname, '..', 'dist', 'addon.node'),
  path.join(__dirname, '..', 'dist', 'pos-receipt-printer.node'),
];

let binding = null;
// 1) Try local candidates (during dev or if dist was published)
for (const p of candidates) {
  if (fs.existsSync(p)) {
    binding = require(p);
    break;
  }
}

// 2) If running inside Electron and app is packaged, electron-builder typically
// places unpacked native modules under resources/app.asar.unpacked
if (!binding && process?.versions?.electron) {
  try {
    const resourcesPath = process.resourcesPath || '';
    const unpackedCandidates = [
      // typical installed path: <app>/resources/app.asar.unpacked/node_modules/<pkg>/lib/addon.node
      path.join(resourcesPath, 'app.asar.unpacked', 'node_modules', 'pos-receipt-printer', 'lib', 'addon.node'),
      path.join(resourcesPath, 'app.asar.unpacked', 'node_modules', 'pos-receipt-printer', 'dist', 'addon.node'),
      // alternative: resources/app/node_modules when not using asar
      path.join(resourcesPath, 'app', 'node_modules', 'pos-receipt-printer', 'lib', 'addon.node'),
      path.join(resourcesPath, 'app', 'node_modules', 'pos-receipt-printer', 'dist', 'addon.node'),
      // installer layout on Windows: resources/app.asar.unpacked/<...>
    ];

    for (const p of unpackedCandidates) {
      if (fs.existsSync(p)) {
        binding = require(p);
        break;
      }
    }
  } catch {
    // ignore and proceed to error below
  }
}

if (binding) {
  module.exports = binding;
} else {
  throw new Error(
    'pos-receipt-printer: native binding not found.\n' +
      'Checked: ' + candidates.join(', ') + '\n' +
      (process?.versions?.electron
        ? 'Also checked common Electron unpacked locations under process.resourcesPath.\n'
        : '') +
      'If you packaged the app with asar, ensure that native .node files are unpacked (see electron-builder asarUnpack).\n' +
      'For local testing you can run `napi build --release` and copy the resulting .node to dist/addon.node',
  );
}

