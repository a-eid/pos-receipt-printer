const { printReceipt } = require("./index.node");

async function run() {
	try {
		const payload = {
			title: "Test Store",
			time: "2026-03-15",
			number: "12345",
			items: [{ name: "Test Item", qty: "1", price: 10.0, total: 10.0 }],
			total: 10.0,
			footer: { address: "123 Test St", lastLine: "Thanks for shopping!" },
			uuid: "fb_ho0hcYMtV030wGTSvE", // nanoid format
			port: "COM7", // Change if on a different OS, but we just want to ensure types map correctly
		};

		console.log("Calling printReceipt with payload:", payload);
		const result = await printReceipt(payload);
		console.log("Result:", result);
	} catch (err) {
		console.error(
			"Caught error (this is expected if no printer is connected):",
			err.message,
		);
	}
}

run();
