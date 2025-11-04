// Minimal loader for the native binding.
// Exports whatever the native addon exports. The postinstall script
// downloads the correct `addon.node` into this directory as `addon.node`.
const path = require('node:path');
const fs = require('node:fs');

const candidates = [
  path.join(__dirname, 'addon.node'),
  path.join(__dirname, 'pos-receipt-printer.node'),
  path.join(__dirname, 'pos_receipt_printer.node'),
];

let binding = null;
for (const p of candidates) {
  if (fs.existsSync(p)) {
    binding = require(p);
    break;
  }
}

if (binding) {
  module.exports = binding;
} else {
  throw new Error(
    'pos-receipt-printer: native binding not found.\n' +
      'Checked: ' + candidates.join(', ') + '\n' +
      'If you installed from a GitHub tag, ensure the Release contains the matching .node for your platform, and that package.json.version at that tag matches the tag.\n' +
      'For local testing you can run `napi build --release` and copy the resulting .node to dist/addon.node',
  );
}

