use crate::{
    app::state::AppState,
    domain::models::StoredFile,
    utils::html::{escape_attr, escape_html},
};

pub fn build_view_html(state: &AppState, file: &StoredFile) -> String {
    let title = content_filename(file);
    let description = format!(
        "Uploaded on {}",
        file.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    let raw_url = raw_route_url(state, file);
    let download_url = download_route_url(state, file);
    let canonical_url = view_route_url(state, file);
    let theme_colour = file
        .preferred_hex_colour
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "#7289da".to_string());

    let escaped_title = escape_html(&title);
    let escaped_description = escape_html(&description);
    let escaped_title_attr = escape_attr(&title);
    let escaped_description_attr = escape_attr(&description);
    let escaped_raw_url = escape_attr(&raw_url);
    let escaped_download_url = escape_attr(&download_url);
    let escaped_canonical_url = escape_attr(&canonical_url);
    let escaped_uploader = escape_html(&file.uploader);
    let escaped_mime = escape_html(&file.mime_type);
    let escaped_size = escape_html(&human_file_size(file.size));
    let escaped_theme_colour = escape_attr(&theme_colour);

    let image_or_link = preview_markup(&file.mime_type, &escaped_raw_url, &escaped_title_attr);
    let og_type = open_graph_type(&file.mime_type);
    let twitter_card = twitter_card_type(&file.mime_type);
    let og_image_meta_tag = open_graph_image_meta_tag(state, file);
    let twitter_image_meta_tag = twitter_image_meta_tag(state, file);
    let video_meta_tags = video_meta_tags(file, &escaped_raw_url, &escaped_canonical_url);

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>{title}</title>

  <meta name="theme-color" content="{theme_colour}" />
  <meta name="description" content="{description_attr}" />
  <meta property="og:type" content="{og_type}" />
  <meta property="og:title" content="{title_attr}" />
  <meta property="og:description" content="{description_attr}" />
  <meta property="og:url" content="{canonical}" />
  <link rel="canonical" href="{canonical}" />
  {og_image_meta_tag}
  {video_meta_tags}
  <meta name="twitter:card" content="{twitter_card}" />
  <meta name="twitter:title" content="{title_attr}" />
  <meta name="twitter:description" content="{description_attr}" />
  {twitter_image_meta_tag}

  <style>
    :root {{
      color-scheme: dark;
      --accent: {theme_colour};
      --accent-soft: color-mix(in srgb, var(--accent) 20%, transparent);
    }}
    * {{
      box-sizing: border-box;
    }}
    body {{
      margin: 0;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background:
        radial-gradient(circle at top, var(--accent-soft), transparent 40%),
        linear-gradient(180deg, #0f1115 0%, #161a22 100%);
      color: #e5e7eb;
    }}
    .wrap {{
      max-width: 1100px;
      margin: 0 auto;
      padding: 32px 20px 48px;
    }}
    .card {{
      background: rgba(24, 24, 27, 0.82);
      border: 1px solid color-mix(in srgb, var(--accent) 25%, rgba(148, 163, 184, 0.14));
      border-radius: 24px;
      padding: 24px;
      box-shadow: 0 20px 50px rgba(0, 0, 0, 0.35);
      backdrop-filter: blur(10px);
    }}
    h1 {{
      margin: 0 0 10px;
      font-size: clamp(1.6rem, 4vw, 2.4rem);
      word-break: break-word;
      color: #f3f4f6;
      letter-spacing: -0.02em;
    }}
    .meta {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      color: #cbd5e1;
      font-size: 0.95rem;
      margin-bottom: 22px;
    }}
    .meta span {{
      padding: 8px 12px;
      border-radius: 999px;
      background: rgba(15, 23, 42, 0.55);
      border: 1px solid rgba(148, 163, 184, 0.16);
    }}
    .preview {{
      padding: 18px;
      border-radius: 20px;
      background: rgba(15, 23, 42, 0.55);
      border: 1px solid rgba(148, 163, 184, 0.12);
      overflow: hidden;
      margin-bottom: 22px;
    }}
    .actions {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
    }}
    .button {{
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      min-width: 140px;
      padding: 12px 18px;
      border-radius: 14px;
      text-decoration: none;
      font-weight: 700;
      transition: transform 0.15s ease, box-shadow 0.15s ease;
    }}
    .button.primary {{
      color: #111827;
      background: linear-gradient(135deg, {theme_colour}, #ffffff);
      box-shadow: 0 10px 25px color-mix(in srgb, {theme_colour} 45%, transparent);
    }}
    .button.secondary {{
      color: #f8fafc;
      background: rgba(30, 41, 59, 0.9);
      border: 1px solid rgba(148, 163, 184, 0.18);
    }}
    .button:hover {{
      transform: translateY(-1px);
    }}
    .footer {{
      margin-top: 24px;
      color: #94a3b8;
      font-size: 0.88rem;
    }}
    a {{ color: inherit; }}
  </style>
</head>
<body>
  <main class="wrap">
    <section class="card">
      <h1>{title}</h1>
      <div class="meta">
        <span>Uploader: {uploader}</span>
        <span>Type: {mime}</span>
        <span>Size: {size}</span>
      </div>

      <div class="preview">
        {preview}
      </div>

      <div class="actions">
        <a class="button primary" href="{raw}">Open raw</a>
        <a class="button secondary" href="{download}" download>Download</a>
      </div>

      <p class="footer">{description}</p>
    </section>
  </main>
</body>
</html>"#,
        title = escaped_title,
        theme_colour = escaped_theme_colour,
        description_attr = escaped_description_attr,
        og_type = og_type,
        title_attr = escaped_title_attr,
        canonical = escaped_canonical_url,
        og_image_meta_tag = og_image_meta_tag,
        video_meta_tags = video_meta_tags,
        twitter_card = twitter_card,
        twitter_image_meta_tag = twitter_image_meta_tag,
        uploader = escaped_uploader,
        mime = escaped_mime,
        size = escaped_size,
        preview = image_or_link,
        raw = escaped_raw_url,
        download = escaped_download_url,
        description = escaped_description,
    )
}

pub fn content_filename(file: &StoredFile) -> String {
    format!("{}.{}", file.id, file.extension)
}

fn raw_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/u/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn download_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/download/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn view_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/v/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn human_file_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let mut value = size as f64;
    let mut unit_index = 0usize;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{value:.2} {}", UNITS[unit_index])
    }
}

fn preview_markup(mime_type: &str, raw_url: &str, title_attr: &str) -> String {
    if mime_type.starts_with("image/") {
        return format!(
            r#"<img src="{raw}" alt="{alt}" style="max-width:min(100%, 960px); max-height:70vh; border-radius:16px; display:block; margin:0 auto;" />"#,
            raw = raw_url,
            alt = title_attr
        );
    }

    if mime_type.starts_with("video/") {
        return format!(
            r#"<video controls playsinline preload="metadata" style="width:min(100%, 960px); max-height:70vh; border-radius:16px; display:block; margin:0 auto; background:#000;">
  <source src="{raw}" type="{mime}">
  Your browser does not support the video tag.
</video>"#,
            raw = raw_url,
            mime = escape_attr(mime_type)
        );
    }

    if mime_type.starts_with("audio/") {
        return format!(
            r#"<audio controls preload="metadata" style="width:min(100%, 720px); display:block; margin:0 auto;">
  <source src="{raw}" type="{mime}">
  Your browser does not support the audio element.
</audio>"#,
            raw = raw_url,
            mime = escape_attr(mime_type)
        );
    }

    format!(
        r#"<p style="text-align:center; margin:2rem 0;"><a href="{raw}" style="color:var(--accent); font-weight:600; text-decoration:none;">Open raw file</a></p>"#,
        raw = raw_url
    )
}

fn open_graph_image_meta_tag(state: &AppState, file: &StoredFile) -> String {
    if !file.mime_type.starts_with("image/") {
        return String::new();
    }

    format!(
        r#"<meta property="og:image" content="{}" />"#,
        escape_attr(&raw_route_url(state, file))
    )
}

fn twitter_image_meta_tag(state: &AppState, file: &StoredFile) -> String {
    if !file.mime_type.starts_with("image/") {
        return String::new();
    }

    format!(
        r#"<meta name="twitter:image" content="{}" />"#,
        escape_attr(&raw_route_url(state, file))
    )
}

fn video_meta_tags(file: &StoredFile, raw_url: &str, canonical_url: &str) -> String {
    if !file.mime_type.starts_with("video/") {
        return String::new();
    }

    let mime = escape_attr(&file.mime_type);

    format!(
        r#"<meta property="og:video" content="{raw}" />
  <meta property="og:video:url" content="{raw}" />
  <meta property="og:video:secure_url" content="{raw}" />
  <meta property="og:video:type" content="{mime}" />
  <meta property="og:video:width" content="1280" />
  <meta property="og:video:height" content="720" />
  <meta name="twitter:url" content="{canonical}" />
  <meta name="twitter:player" content="{canonical}" />
  <meta name="twitter:player:width" content="1280" />
  <meta name="twitter:player:height" content="720" />
  <meta name="twitter:player:stream" content="{raw}" />
  <meta name="twitter:player:stream:content_type" content="{mime}" />"#,
        raw = raw_url,
        mime = mime,
        canonical = canonical_url,
    )
}

fn open_graph_type(mime_type: &str) -> String {
    if mime_type.starts_with("video/") {
        mime_type.to_string()
    } else {
        "website".to_string()
    }
}

fn twitter_card_type(mime_type: &str) -> &'static str {
    if mime_type.starts_with("video/") {
        "player"
    } else {
        "summary_large_image"
    }
}
