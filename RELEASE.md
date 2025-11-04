# Release Checklist

Follow these steps to create a new release with pre-built binaries.

## Prerequisites

1. Ensure all changes are committed and pushed to main branch
2. Test the package locally with `pnpm build`
3. Verify the font file exists at `src/fonts/NotoSansArabic-Regular.ttf`

## Release Steps

### 1. Update Version

Update the version in both files:

**package.json:**
```json
{
  "version": "0.1.0"  // Update this
}
```

**Cargo.toml:**
```toml
[package]
version = "0.1.0"  # Update this
```

### 2. Commit Version Bump

```bash
git add package.json Cargo.toml
git commit -m "chore: bump version to 0.1.0"
git push origin main
```

### 3. Create and Push Tag

```bash
# Create a tag (must start with 'v')
git tag v0.1.0

# Push the tag to trigger CI
git push origin v0.1.0
```

### 4. Wait for CI Build

The GitHub Actions workflow will automatically:
- Build native binaries for:
  - Windows x64
  - macOS Apple Silicon (ARM64)
  - macOS Intel (x64)
  - Linux x64
- Upload all binaries to the GitHub Release

You can monitor the build progress at:
https://github.com/YOUR_USERNAME/pos-receipt-printer/actions

### 5. Verify Release

Once the workflow completes, check the release page:
https://github.com/YOUR_USERNAME/pos-receipt-printer/releases

You should see:
- Release tag: `v0.1.0`
- Assets attached:
  - `pos_receipt_printer.win32-x64-msvc.node`
  - `pos_receipt_printer.darwin-arm64.node`
  - `pos_receipt_printer.darwin-x64.node`
  - `pos_receipt_printer.linux-x64-gnu.node`

## Using the Release

### In Consumer Projects

Install directly from GitHub:

```bash
npm install github:YOUR_USERNAME/pos-receipt-printer#v0.1.0
```

Or with pnpm:

```bash
pnpm add github:YOUR_USERNAME/pos-receipt-printer#v0.1.0
```

The `postinstall` script will automatically:
1. Try to download the matching pre-built binary from the release
2. If no match is found, fall back to building from source

### Testing the Installation

Create a test script:

```javascript
// test.js
const { printReceipt } = require('pos-receipt-printer');

printReceipt({
  title: "Test Store",
  time: "Nov 4, 2025",
  number: "001",
  items: [
    { name: "Test Item", qty: 1, price: 10, total: 10 }
  ],
  total: 10,
  footer: {
    address: "Test Address",
    lastLine: "Thank you!"
  },
  port: "COM7",  // Adjust for your system
  baud: 9600
}).then(result => {
  console.log("Success:", result);
}).catch(error => {
  console.error("Error:", error);
});
```

Run:
```bash
node test.js
```

## Troubleshooting

### Build Fails on CI

- Check the Actions logs for specific errors
- Ensure all dependencies are listed in Cargo.toml
- Verify the font file is included in the repository

### Binary Not Downloaded

The install script will fall back to building from source if:
- No matching pre-built binary exists for the platform
- GitHub API rate limit is reached
- Network issues

To force a local build:
```bash
npm run build
```

### Different Node.js Versions

The pre-built binaries are compatible with Node.js 16+. If you encounter issues:
- Check `package.json` engines field
- Ensure your Node.js version is supported
- Consider building from source for your specific Node version

## Publishing to npm (Optional)

If you want to publish to npm registry instead of using GitHub releases:

1. Update `.npmignore` to exclude unnecessary files
2. Login to npm: `npm login`
3. Publish: `npm publish`

Note: GitHub releases approach is recommended for Electron apps as it keeps binaries separate from npm package.

## Version Numbering

Follow semantic versioning:
- **Major (1.0.0)**: Breaking changes
- **Minor (0.1.0)**: New features, backward compatible
- **Patch (0.0.1)**: Bug fixes, backward compatible

## Next Steps

After releasing:
1. Update the README with the new version number
2. Document any breaking changes in CHANGELOG.md
3. Notify users of the new release
4. Update dependent projects to use the new version
