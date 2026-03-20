#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const packageJsonPath = path.join(__dirname, "../package.json");
const cargoTomlPath = path.join(__dirname, "../Cargo.toml");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const version = packageJson.version;

if (!version) {
    console.error("Error: No version found in package.json");
    process.exit(1);
}

let cargoToml = fs.readFileSync(cargoTomlPath, "utf8");

const newCargoToml = cargoToml.replace(/^version = "[\d.]+"/m, `version = "${version}"`);

if (cargoToml === newCargoToml) {
    console.log("Cargo.toml version was already up to date.");
} else {
    fs.writeFileSync(cargoTomlPath, newCargoToml);
    console.log(`Successfully synced version ${version} to Cargo.toml`);
}