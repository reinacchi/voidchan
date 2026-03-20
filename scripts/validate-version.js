#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const rootPackageJsonPath = path.join(__dirname, "../package.json");
const cargoTomlPath = path.join(__dirname, "../Cargo.toml");

const packageJson = JSON.parse(fs.readFileSync(rootPackageJsonPath, "utf8"));
const expectedVersion = packageJson.version;

if (!expectedVersion) {
    console.error("Error: No version found in package.json");
    process.exit(1);
}

const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const cargoMatch = cargoToml.match(/^version = "([\d.]+)"/m);
const cargoVersion = cargoMatch ? cargoMatch[1] : null;

if (!cargoVersion) {
    console.error("Error: Could not find a version string in Cargo.toml");
    process.exit(1);
}

if (cargoVersion !== expectedVersion) {
    console.error("❌ Version Mismatch!");
    console.error(`   - package.json: "${expectedVersion}"`);
    console.error(`   - Cargo.toml:   "${cargoVersion}"`);
    process.exit(1);
}

console.log(`✅ Versions are in sync: ${expectedVersion}`);
process.exit(0);