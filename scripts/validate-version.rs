use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let cargo_toml_path = Path::new("Cargo.toml");

    let cargo_version = match read_package_version(cargo_toml_path) {
        Ok(version) => version,
        Err(err) => exit_with_error(&err),
    };

    validate_semver(&cargo_version).unwrap_or_else(|err| exit_with_error(&err));

    if let Some(expected) = env::args().nth(1) {
        validate_semver(&expected).unwrap_or_else(|err| exit_with_error(&err));
        if cargo_version != expected {
            eprintln!("❌ Version Mismatch!");
            eprintln!("   - expected:   \"{}\"", expected);
            eprintln!("   - Cargo.toml: \"{}\"", cargo_version);
            process::exit(1);
        }
    }

    println!("✅ Cargo.toml version is valid: {}", cargo_version);
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{message}");
    process::exit(1);
}

fn read_package_version(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Error reading {}: {err}", path.display()))?;

    let mut in_package_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = trimmed == "[package]";
            continue;
        }

        if in_package_section && trimmed.starts_with("version") {
            let (_, raw_value) = trimmed
                .split_once('=')
                .ok_or_else(|| "Error: Could not parse version string in Cargo.toml".to_string())?;
            let value = raw_value.trim();
            let version = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .ok_or_else(|| "Error: Could not parse version string in Cargo.toml".to_string())?;
            return Ok(version.to_string());
        }
    }

    Err("Error: Could not find package version in Cargo.toml".to_string())
}

fn validate_semver(version: &str) -> Result<(), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Error: Invalid version format \"{}\". Expected: X.Y.Z",
            version
        ));
    }

    for part in parts {
        part.parse::<u64>().map_err(|_| {
            format!(
                "Error: Invalid version format \"{}\". Expected: X.Y.Z",
                version
            )
        })?;
    }

    Ok(())
}
