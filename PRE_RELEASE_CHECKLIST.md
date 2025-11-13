# Pre-Release Checklist

Before creating your first release, verify everything is in place:

## ✅ Repository Setup

- [ ] Repository is at: https://github.com/a-eid/pos-receipt-printer
- [ ] All files are committed and pushed to `main` branch
- [ ] GitHub Actions is enabled for the repository

## ✅ Required Files

- [ ] `.github/workflows/release.yml` - CI workflow for building binaries
- [ ] `Cargo.toml` - Rust package configuration
- [ ] `package.json` - Node.js package configuration
- [ ] `index.js` - Native module loader
- [ ] `index.d.ts` - TypeScript definitions
- [ ] `src/lib.rs` - Main Rust source code
- [ ] `src/fonts/NotoSansArabic-Regular.ttf` - Arabic font file
- [ ] `build.rs` - N-API build script
- [ ] `.cargo/config.toml` - Rust build configuration
- [ ] `.gitignore` - Ignore build artifacts

## ✅ Local Build Test

Before pushing a release, test locally:

```bash
# Install dependencies
pnpm install

# Build the native module
pnpm build

# Verify the .node file is created
ls -la pos_receipt_printer.*.node
```

## ✅ Create First Release

1. **Commit all changes:**
   ```bash
   git add .
   git commit -m "chore: prepare for v0.1.0 release"
   git push origin main
   ```

2. **Create and push tag:**
   ```bash
   git tag v0.1.0
   git push origin main --tags
   ```

3. **Monitor GitHub Actions:**
   - Go to: https://github.com/a-eid/pos-receipt-printer/actions
   - Wait for the "build-and-release-prebuilts" workflow to complete
   - Should see 4 jobs running (Windows, macOS ARM, macOS Intel, Linux)

4. **Verify Release:**
   - Go to: https://github.com/a-eid/pos-receipt-printer/releases
   - Check that v0.1.0 release exists
   - Verify it has 4 `.node` files attached:
     - `pos_receipt_printer.win32-x64-msvc.node`
     - `pos_receipt_printer.darwin-arm64.node`
     - `pos_receipt_printer.darwin-x64.node`
     - `pos_receipt_printer.linux-x64-gnu.node`

## ✅ Test Installation

After release is published, test installation:

```bash
# Create a test directory
mkdir test-install
cd test-install

# Initialize a new project
npm init -y

# Install from GitHub release
pnpm add github:a-eid/pos-receipt-printer#v0.1.0

# Verify installation
node -e "console.log(require('pos-receipt-printer'))"
```

## ✅ Expected Output

On successful installation, you should see:

```
Downloading pre-built binary for pos-receipt-printer...
✓ Binary downloaded and installed successfully
```

If you see compilation output, it means no pre-built binary matched your platform.

## 🚨 Common Issues

### Issue: GitHub Actions fails with "permission denied"

**Solution:** Go to repository Settings → Actions → General → Workflow permissions, and enable "Read and write permissions"

### Issue: `.node` files not attached to release

**Solution:** Check the GitHub Actions logs. Common causes:
- Build failed due to missing dependencies
- `GITHUB_TOKEN` permissions not set correctly

### Issue: Installation tries to build from source

**Solution:** 
- Verify the release has the correct `.node` file for your platform
- Check the file naming matches the pattern: `pos_receipt_printer.<platform>-<arch>.node`
- Ensure you're installing with the correct tag: `#v0.1.0`

## 📝 Next Steps

After successful release:

1. Update version in `Cargo.toml` and `package.json` for next release
2. Update installation instructions in README with new version
3. Test in your Electron app
4. Document any API changes in CHANGELOG.md

## 🎉 Success Criteria

Your release is ready when:

- ✅ GitHub Actions workflow completed successfully
- ✅ Release has all 4 platform binaries attached
- ✅ Test installation downloads binary (no compilation)
- ✅ Test app can import and use the module
- ✅ Receipt prints correctly on thermal printer

---

**Need help?** Check the GitHub Actions logs at:
https://github.com/a-eid/pos-receipt-printer/actions
