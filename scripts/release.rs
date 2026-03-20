use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

fn main() {
    let cargo_toml_path = Path::new("Cargo.toml");

    let current_version = match read_package_version(cargo_toml_path) {
        Ok(version) => version,
        Err(err) => exit_with_error(&err),
    };

    let version_arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => exit_with_error(
            "Error: Please specify version bump type (patch/minor/major) or exact version (e.g., 0.3.0)",
        ),
    };

    let new_version = if matches!(version_arg.as_str(), "patch" | "minor" | "major") {
        match bump_version(&current_version, &version_arg) {
            Ok(version) => version,
            Err(err) => exit_with_error(&err),
        }
    } else {
        validate_semver(&version_arg).unwrap_or_else(|err| exit_with_error(&err));
        version_arg
    };

    if let Err(err) = write_package_version(cargo_toml_path, &new_version) {
        exit_with_error(&err);
    }

    println!("Updated Cargo.toml to {}", new_version);

    println!("Validating version...");
    let status = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "validate-version", "--", &new_version])
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(_) => exit_with_error("Validation failed."),
        Err(err) => exit_with_error(&format!("Failed to run validator: {err}")),
    }

    println!("\nChanges to be committed:");
    match Command::new("git").args(["status", "--short"]).status() {
        Ok(_) => {}
        Err(_) => {
            println!("git status unavailable.");
        }
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{message}");
    process::exit(1);
}

fn read_package_version(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Error reading {}: {err}", path.display()))?;

    extract_package_version(&content)
}

fn write_package_version(path: &Path, new_version: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Error reading {}: {err}", path.display()))?;

    let updated = replace_package_version(&content, new_version)?;
    fs::write(path, updated).map_err(|err| format!("Error writing {}: {err}", path.display()))
}

fn bump_version(current_version: &str, bump_kind: &str) -> Result<String, String> {
    let (major, minor, patch) = parse_version_triplet(current_version)?;

    let next = match bump_kind {
        "patch" => (major, minor, patch + 1),
        "minor" => (major, minor + 1, 0),
        "major" => (major + 1, 0, 0),
        _ => return Err(format!("Unsupported bump type: {bump_kind}")),
    };

    Ok(format!("{}.{}.{}", next.0, next.1, next.2))
}

fn validate_semver(version: &str) -> Result<(), String> {
    parse_version_triplet(version).map(|_| ())
}

fn parse_version_triplet(version: &str) -> Result<(u64, u64, u64), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Error: Invalid version format \"{}\". Expected: X.Y.Z",
            version
        ));
    }

    let major = parts[0].parse::<u64>().map_err(|_| {
        format!(
            "Error: Invalid version format \"{}\". Expected: X.Y.Z",
            version
        )
    })?;
    let minor = parts[1].parse::<u64>().map_err(|_| {
        format!(
            "Error: Invalid version format \"{}\". Expected: X.Y.Z",
            version
        )
    })?;
    let patch = parts[2].parse::<u64>().map_err(|_| {
        format!(
            "Error: Invalid version format \"{}\". Expected: X.Y.Z",
            version
        )
    })?;

    Ok((major, minor, patch))
}

fn extract_package_version(content: &str) -> Result<String, String> {
    let mut in_package_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = trimmed == "[package]";
            continue;
        }

        if in_package_section && trimmed.starts_with("version") {
            return parse_version_line(trimmed)
                .ok_or_else(|| "Error: Could not parse version string in Cargo.toml".to_string());
        }
    }

    Err("Error: Could not find package version in Cargo.toml".to_string())
}

fn replace_package_version(content: &str, new_version: &str) -> Result<String, String> {
    let mut output = Vec::new();
    let mut in_package_section = false;
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = trimmed == "[package]";
            output.push(line.to_string());
            continue;
        }

        if in_package_section && trimmed.starts_with("version") && !replaced {
            let indent_len = line.len() - line.trim_start().len();
            let indent = &line[..indent_len];
            output.push(format!("{indent}version = \"{new_version}\""));
            replaced = true;
            continue;
        }

        output.push(line.to_string());
    }

    if !replaced {
        return Err("Error: Could not find package version in Cargo.toml".to_string());
    }

    Ok(output.join("\n") + "\n")
}

fn parse_version_line(line: &str) -> Option<String> {
    let (_, raw_value) = line.split_once('=')?;
    let value = raw_value.trim();
    let stripped = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(stripped.to_string())
}
