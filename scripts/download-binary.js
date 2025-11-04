#!/usr/bin/env node
/*
  Download the prebuilt native binary for this platform/arch from GitHub Releases
  and place it at dist/addon.node. Fails with a clear message if not available.
*/
const fs = require("fs");
const path = require("path");
const https = require("https");

const pkg = require("../package.json");

function ensureDir(dir) {
	if (!fs.existsSync(dir)) {
		fs.mkdirSync(dir, { recursive: true });
	}
}

function getTarget() {
	const platform = process.platform; // 'win32' | 'darwin' | 'linux'
	const arch = process.arch; // 'x64' | 'arm64' | ...

	if (!["win32", "darwin", "linux"].includes(platform)) {
		throw new Error(`Unsupported platform: ${platform}`);
	}

	if (!["x64", "arm64"].includes(arch)) {
		throw new Error(`Unsupported architecture: ${arch}`);
	}

	// Map to expected asset filename suffix
	// Windows x64 → pos-receipt-printer-win32-x64.node
	// macOS Intel → pos-receipt-printer-darwin-x64.node
	// macOS ARM → pos-receipt-printer-darwin-arm64.node
	// Linux x64 → pos-receipt-printer-linux-x64-gnu.node
	let asset = `pos-receipt-printer-${platform}-${arch}`;
	if (platform === "linux") {
		// Only GNU libc builds are provided
		asset += "-gnu";
	}
	asset += ".node";
	return { platform, arch, asset };
}

function getReleaseTag() {
	return `v${pkg.version}`;
}

function getDownloadUrl() {
	const tag = getReleaseTag();
	return `https://github.com/a-eid/pos-receipt-printer/releases/download/${tag}/pos-receipt-printer-win32-x64.node`;
}

// /${asset}

function download(url, dest) {
	return new Promise((resolve, reject) => {
		const file = fs.createWriteStream(dest);

		const request = https.get(
			url,
			{
				headers: {
					"User-Agent": "pos-receipt-printer-installer",
					Accept: "application/octet-stream",
				},
			},
			(response) => {
				if (
					response.statusCode &&
					response.statusCode >= 300 &&
					response.statusCode < 400 &&
					response.headers.location
				) {
					// Follow redirect
					response.resume();
					https
						.get(response.headers.location, (res2) => {
							if (res2.statusCode !== 200) {
								file.close();
								fs.unlink(dest, () => {});
								return reject(
									new Error(
										`Failed to download binary. HTTP ${res2.statusCode} for redirected URL`,
									),
								);
							}
							res2.pipe(file);
							file.on("finish", () => file.close(resolve));
						})
						.on("error", (err) => {
							file.close();
							fs.unlink(dest, () => {});
							reject(err);
						});
					return;
				}

				if (response.statusCode !== 200) {
					file.close();
					fs.unlink(dest, () => {});
					return reject(
						new Error(`Failed to download binary. HTTP ${response.statusCode}`),
					);
				}

				response.pipe(file);
				file.on("finish", () => file.close(resolve));
			},
		);

		request.on("error", (err) => {
			file.close();
			fs.unlink(dest, () => {});
			reject(err);
		});
	});
}

(async () => {
	try {
		const { asset, platform, arch } = getTarget();
		const url = getDownloadUrl(asset);

		const distDir = path.join(__dirname, "..", "dist");
		const outPath = path.join(distDir, "addon.node");

		ensureDir(distDir);

		process.stdout.write(
			`Downloading prebuilt binary for ${platform}-${arch}:\n  ${url}\n`,
		);
		await download(url, outPath);

		// Basic sanity check: file exists and size > 0
		const stat = fs.statSync(outPath);
		if (!stat || stat.size === 0) {
			throw new Error("Downloaded binary is empty.");
		}

		console.log(`Saved to ${outPath} (${stat.size} bytes)`);
	} catch (err) {
		const msg =
			`\n\npos-receipt-printer: No prebuilt binary found for your platform.\n` +
			`Reason: ${err.message}\n\n` +
			`This package does not build on install. Please ensure a matching release asset exists for your platform.\n` +
			`Supported assets:\n` +
			`  - pos-receipt-printer-win32-x64.node\n` +
			`  - pos-receipt-printer-darwin-x64.node\n` +
			`  - pos-receipt-printer-darwin-arm64.node\n` +
			`  - pos-receipt-printer-linux-x64-gnu.node\n\n` +
			`If you're on Alpine Linux (musl), it's currently unsupported. Consider using a glibc-based distro.\n`;
		console.error(msg);
		process.exit(1);
	}
})();
