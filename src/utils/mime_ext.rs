use std::path::Path;

pub fn extension_from_mime(mime: &str, original_name: Option<&str>) -> String {
    if let Some(name) = original_name {
        if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
            let cleaned = ext.trim().trim_start_matches('.').to_ascii_lowercase();
            if !cleaned.is_empty() {
                return cleaned;
            }
        }
    }

    mime_guess::get_mime_extensions_str(mime)
        .and_then(|arr| arr.first().copied())
        .unwrap_or("bin")
        .to_string()
}
