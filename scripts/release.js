#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const packageJsonPath = path.join(__dirname, "../package.json");
const syncVersionScript = path.join(__dirname, "sync-version.js");
const validateVersionScript = path.join(__dirname, "validate-version.js");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const currentVersion = packageJson.version;

if (!currentVersion) {
    console.error("Error: No version found in package.json");
    process.exit(1);
}

const versionArg = process.argv[2];

if (!versionArg) {
    console.error("Error: Please specify version bump type (patch/minor/major) or exact version (e.g., 0.3.0)");
    process.exit(1);
}

let newVersion;
if (["patch", "minor", "major"].includes(versionArg)) {
    const [major, minor, patch] = currentVersion.split(".").map(Number);

    switch (versionArg) {
        case "patch":
            newVersion = `${major}.${minor}.${patch + 1}`;
            break;
        case "minor":
            newVersion = `${major}.${minor + 1}.0`;
            break;
        case "major":
            newVersion = `${major + 1}.0.0`;
            break;
    }
} else {
    if (!/^\d+\.\d+\.\d+$/.test(versionArg)) {
        console.error(`Error: Invalid version format "${versionArg}". Expected: X.Y.Z`);
        process.exit(1);
    }
    newVersion = versionArg;
}

packageJson.version = newVersion;
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
console.log(`Updated package.json to ${newVersion}`);

try {
    console.log("Syncing Cargo.toml...");
    execSync(`node "${syncVersionScript}"`, { stdio: "inherit" });
    
    console.log("Validating versions...");
    execSync(`node "${validateVersionScript}"`, { stdio: "inherit" });
} catch (error) {
    console.error("Failed to complete version synchronisation.");
    process.exit(1);
}

try {
    console.log("\nChanges to be committed:");
    execSync("git status --short", { stdio: "inherit" });
} catch (error) {

}

process.exit(0);