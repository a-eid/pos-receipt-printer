#!/usr/bin/env node

/**
 * Quick verification script to ensure all files are in place before release
 */

const fs = require('fs');
const path = require('path');

const requiredFiles = [
  'package.json',
  'Cargo.toml',
  'index.js',
  'index.d.ts',
  'build.rs',
  'src/lib.rs',
  'src/fonts/NotoSansArabic-Regular.ttf',
  '.github/workflows/release.yml',
  '.cargo/config.toml',
  '.gitignore'
];

const requiredInPackageJson = [
  'name',
  'version',
  'main',
  'types',
  'repository',
  'scripts.build',
  'scripts.install'
];

console.log('🔍 Verifying release setup...\n');

let allGood = true;

// Check files exist
console.log('📁 Checking required files:');
for (const file of requiredFiles) {
  const exists = fs.existsSync(path.join(__dirname, file));
  console.log(`  ${exists ? '✅' : '❌'} ${file}`);
  if (!exists) allGood = false;
}

// Check package.json content
console.log('\n📦 Checking package.json:');
try {
  const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, 'package.json'), 'utf8'));
  
  for (const field of requiredInPackageJson) {
    const keys = field.split('.');
    let value = pkg;
    for (const key of keys) {
      value = value?.[key];
    }
    const exists = value !== undefined;
    console.log(`  ${exists ? '✅' : '❌'} ${field}: ${exists ? '✓' : 'missing'}`);
    if (!exists) allGood = false;
  }
  
  // Check repository URL
  if (pkg.repository?.url) {
    const hasCorrectRepo = pkg.repository.url.includes('a-eid/pos-receipt-printer');
    console.log(`  ${hasCorrectRepo ? '✅' : '⚠️'} Repository URL: ${pkg.repository.url}`);
    if (!hasCorrectRepo) {
      console.log('    ⚠️  Expected: github.com/a-eid/pos-receipt-printer');
    }
  }
} catch (error) {
  console.log('  ❌ Error reading package.json:', error.message);
  allGood = false;
}

// Check Cargo.toml
console.log('\n🦀 Checking Cargo.toml:');
try {
  const cargo = fs.readFileSync(path.join(__dirname, 'Cargo.toml'), 'utf8');
  const hasNapi = cargo.includes('napi = ');
  const hasCdylib = cargo.includes('crate-type = ["cdylib"]');
  const hasNapiBuild = cargo.includes('napi-build');
  
  console.log(`  ${hasNapi ? '✅' : '❌'} napi dependency`);
  console.log(`  ${hasCdylib ? '✅' : '❌'} cdylib crate-type`);
  console.log(`  ${hasNapiBuild ? '✅' : '❌'} napi-build`);
  
  if (!hasNapi || !hasCdylib || !hasNapiBuild) allGood = false;
} catch (error) {
  console.log('  ❌ Error reading Cargo.toml:', error.message);
  allGood = false;
}

// Check GitHub Actions workflow
console.log('\n⚙️  Checking GitHub Actions:');
try {
  const workflow = fs.readFileSync(path.join(__dirname, '.github/workflows/release.yml'), 'utf8');
  const hasWindows = workflow.includes('windows-latest');
  const hasMacOS = workflow.includes('macos-latest');
  const hasLinux = workflow.includes('ubuntu-latest');
  const hasRelease = workflow.includes('softprops/action-gh-release');
  
  console.log(`  ${hasWindows ? '✅' : '❌'} Windows build`);
  console.log(`  ${hasMacOS ? '✅' : '❌'} macOS build`);
  console.log(`  ${hasLinux ? '✅' : '❌'} Linux build`);
  console.log(`  ${hasRelease ? '✅' : '❌'} GitHub Release action`);
  
  if (!hasWindows || !hasMacOS || !hasLinux || !hasRelease) allGood = false;
} catch (error) {
  console.log('  ❌ Error reading workflow:', error.message);
  allGood = false;
}

// Final verdict
console.log('\n' + '='.repeat(50));
if (allGood) {
  console.log('✅ All checks passed! Ready to create a release.');
  console.log('\n📋 Next steps:');
  console.log('  1. git add .');
  console.log('  2. git commit -m "chore: prepare for v0.1.0 release"');
  console.log('  3. git push origin main');
  console.log('  4. git tag v0.1.0');
  console.log('  5. git push origin main --tags');
  console.log('\n🔗 Monitor the build: https://github.com/a-eid/pos-receipt-printer/actions');
  process.exit(0);
} else {
  console.log('❌ Some checks failed. Please review the errors above.');
  console.log('\n📖 See PRE_RELEASE_CHECKLIST.md for more details.');
  process.exit(1);
}
