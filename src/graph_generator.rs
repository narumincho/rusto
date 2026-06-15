use base64::Engine;
use resvg::tiny_skia;
use resvg::usvg;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Deserialize, Debug)]
pub struct ItemData {
    pub name: String,
    pub value: f64,
    pub market_premium: Option<f64>,
    pub unit: String,
    pub image_url: Option<String>,
}

fn sanitize_url(url: &str) -> String {
    url.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub async fn generate_graph() -> Result<(), anyhow::Error> {
    // 1. Read JSON input data
    let data_path = "./input/chart_data.json";
    let file = File::open(data_path)?;
    let items: Vec<ItemData> = serde_json::from_reader(file)?;

    // Create cache directory if it doesn't exist
    let cache_dir = "./input/cache";
    std::fs::create_dir_all(cache_dir)?;

    // 2. Prepare images (cache check, download and base64-encode)
    // Use compliant MediaWiki User-Agent to bypass Cloudflare protection
    let client = reqwest::Client::builder()
        .user_agent("MyMinecraftImageDownloader/1.0 (Contact: Narumi)")
        .build()?;

    let mut embedded_images = Vec::new();

    for item in &items {
        let mut base64_image = None;
        if let Some(ref url) = item.image_url {
            if !url.is_empty() {
                let cache_filename = format!("{}.png", sanitize_url(url));
                let cache_path = std::path::Path::new(cache_dir).join(&cache_filename);

                let mut bytes = Vec::new();
                let mut loaded_from_cache = false;

                // Try loading from cache first
                if cache_path.exists() {
                    if let Ok(mut f) = File::open(&cache_path) {
                        let mut temp_bytes = Vec::new();
                        if f.read_to_end(&mut temp_bytes).is_ok() {
                            // Check if the file is actually an HTML error page instead of a PNG
                            let is_html = temp_bytes.starts_with(b"<!DOCTYPE")
                                || temp_bytes.starts_with(b"<html")
                                || temp_bytes.starts_with(b"<svg")
                                || temp_bytes.starts_with(b"<?xml");

                            if is_html {
                                println!(
                                    "Cached file for {} is an HTML page. Removing and re-downloading...",
                                    item.name
                                );
                                let _ = std::fs::remove_file(&cache_path);
                            } else {
                                bytes = temp_bytes;
                                println!("Loaded texture for {} from cache", item.name);
                                loaded_from_cache = true;
                            }
                        }
                    }
                }

                // Download if not loaded from cache
                if !loaded_from_cache {
                    println!("Downloading image for {}: {}", item.name, url);
                    match client.get(url).send().await {
                        Ok(resp) => {
                            match resp.bytes().await {
                                Ok(resp_bytes) => {
                                    let resp_vec = resp_bytes.to_vec();

                                    // Check if downloaded content is HTML
                                    let is_html = resp_vec.starts_with(b"<!DOCTYPE")
                                        || resp_vec.starts_with(b"<html")
                                        || resp_vec.starts_with(b"<svg")
                                        || resp_vec.starts_with(b"<?xml");

                                    if is_html {
                                        eprintln!(
                                            "Warning: Downloaded content for {} is HTML, not a PNG.",
                                            item.name
                                        );
                                    } else {
                                        bytes = resp_vec;
                                        // Save to cache
                                        match File::create(&cache_path) {
                                            Ok(mut f) => {
                                                if f.write_all(&bytes).is_ok() {
                                                    println!(
                                                        "Saved downloaded texture for {} to cache",
                                                        item.name
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!(
                                                    "Failed to create cache file for {}: {:?}",
                                                    item.name, e
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to read bytes from {}: {:?}", item.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to download image for {}: {:?}", item.name, e);
                        }
                    }
                }

                if !bytes.is_empty() {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    base64_image = Some(format!("data:image/png;base64,{}", b64));
                }
            }
        }
        embedded_images.push(base64_image);
    }

    // 3. Build SVG String
    let width = 900;
    let height = 520;

    let mut svg = String::new();
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <defs>
    <linearGradient id="bgGradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#1e1e2e"/>
      <stop offset="100%" stop-color="#11111b"/>
    </linearGradient>
    <linearGradient id="baseGradient" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#3b82f6"/>
      <stop offset="100%" stop-color="#60a5fa"/>
    </linearGradient>
    <linearGradient id="premiumGradient" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#fbbf24"/>
      <stop offset="100%" stop-color="#f59e0b"/>
    </linearGradient>
  </defs>

  <!-- Background -->
  <rect width="{}" height="{}" rx="16" ry="16" fill="url(#bgGradient)" stroke="#313244" stroke-width="2"/>

  <!-- Title -->
  <text x="50%" y="45" font-size="22" font-weight="bold" fill="#cdd6f4" font-family="sans-serif" text-anchor="middle" letter-spacing="0.5">
    Minecraft アイテム価値・市場価格 比較
  </text>
  <text x="50%" y="70" font-size="13" fill="#a6adc8" font-family="sans-serif" text-anchor="middle">
    Base Value &amp; Market Premium per Chest Capacity
  </text>

"##,
        width, height, width, height, width, height
    ));

    // Draw Grid Lines (from 0 to 11, step 1)
    let grid_x_start = 280.0;
    let grid_scale = 40.0;
    let grid_y_start = 95.0;
    let grid_y_end = 435.0;

    // Grid lines for 0 to 11
    for v in 0..=11 {
        let x = grid_x_start + (v as f64) * grid_scale;
        // Subtle dashed lines
        svg.push_str(&format!(
            r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#313244" stroke-width="1" stroke-dasharray="4"/>
  <text x="{}" y="455" font-size="11" fill="#bac2de" font-family="sans-serif" text-anchor="middle">{}</text>
"##,
            x, grid_y_start, x, grid_y_end, x, v
        ));
    }

    // X-Axis Baseline
    svg.push_str(&format!(
        r##"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#45475a" stroke-width="2"/>
  <text x="{}" y="485" font-size="13" fill="#cdd6f4" font-family="sans-serif" text-anchor="middle" font-weight="bold">価値 (価格 / LC)</text>
"##,
        grid_x_start,
        grid_y_end,
        grid_x_start + 11.0 * grid_scale,
        grid_y_end,
        grid_x_start + 5.5 * grid_scale
    ));

    // Draw Items
    for (i, item) in items.iter().enumerate() {
        let item_y = 110.0 + (i as f64) * 65.0;
        let bar_y = item_y + 10.0;
        let bar_h = 20.0;

        // Draw Row background block for hovering visual effect (premium design)
        svg.push_str(&format!(
            r##"  <!-- Row Background for {} -->
  <rect x="25" y="{}" width="850" height="55" rx="8" ry="8" fill="#181825" opacity="0.4"/>
"##,
            item.name, item_y
        ));

        // Draw Icon
        if let Some(ref data_url) = embedded_images[i] {
            svg.push_str(&format!(
                r##"  <image href="{}" x="40" y="{}" width="36" height="36" />
"##,
                data_url,
                item_y + 10.0
            ));
        } else {
            // Render a beautiful 3D block fallback in SVG
            let cube_x = 40.0;
            let cube_y = item_y + 10.0;
            svg.push_str(&format!(
                r##"  <g transform="translate({}, {})">
    <!-- Top Face -->
    <path d="M 18 2 L 34 10 L 18 18 L 2 10 Z" fill="#a1a1aa" stroke="#71717a" stroke-width="0.5"/>
    <!-- Left Face -->
    <path d="M 2 10 L 18 18 L 18 34 L 2 26 Z" fill="#52525b" stroke="#3f3f46" stroke-width="0.5"/>
    <!-- Right Face -->
    <path d="M 18 18 L 34 10 L 34 26 L 18 34 Z" fill="#71717a" stroke="#52525b" stroke-width="0.5"/>
  </g>
"##,
                cube_x, cube_y
            ));
        }

        // Draw Item Name
        svg.push_str(&format!(
            r##"  <text x="90" y="{}" font-size="14" fill="#cdd6f4" font-family="sans-serif" font-weight="bold" dominant-baseline="middle">{}</text>
"##,
            item_y + 28.0,
            item.name
        ));

        // Bar Track
        svg.push_str(&format!(
            r##"  <rect x="{}" y="{}" width="{}" height="{}" rx="4" ry="4" fill="#313244" opacity="0.3"/>
"##,
            grid_x_start,
            bar_y,
            11.0 * grid_scale,
            bar_h
        ));

        // Calculate lengths
        let premium_val = item.market_premium.unwrap_or(0.0);
        let base_w = item.value * grid_scale;
        let premium_w = premium_val * grid_scale;

        if premium_val > 0.0 {
            // Draw base bar (slightly truncated for 2px gap at the right edge)
            let draw_base_w = (base_w - 2.0).max(0.0);
            svg.push_str(&format!(
                r##"  <rect x="{}" y="{}" width="{}" height="{}" rx="4" ry="4" fill="url(#baseGradient)" />
"##,
                grid_x_start, bar_y, draw_base_w, bar_h
            ));
            // Draw premium bar (slightly shifted and truncated)
            let draw_premium_w = (premium_w - 2.0).max(0.0);
            svg.push_str(&format!(
                r##"  <rect x="{}" y="{}" width="{}" height="{}" rx="4" ry="4" fill="url(#premiumGradient)" />
"##,
                grid_x_start + base_w + 2.0,
                bar_y,
                draw_premium_w,
                bar_h
            ));
        } else {
            // Draw solid base bar
            svg.push_str(&format!(
                r##"  <rect x="{}" y="{}" width="{}" height="{}" rx="4" ry="4" fill="url(#baseGradient)" />
"##,
                grid_x_start, bar_y, base_w, bar_h
            ));
        }

        // Label details on the right of the bar
        let label_x = grid_x_start + base_w + premium_w + 12.0;
        let label_y = item_y + 28.0;

        let mut tspan_premium = String::new();
        if premium_val > 0.0 {
            tspan_premium = format!(
                r##"<tspan fill="#fbbf24" font-weight="bold"> (+{:.1})</tspan>"##,
                premium_val
            );
        }

        svg.push_str(&format!(
            r##"  <text x="{}" y="{}" font-size="13" font-family="sans-serif" dominant-baseline="middle">
    <tspan fill="#89b4fa" font-weight="bold">{:.1}</tspan>
    {}
    <tspan fill="#a6adc8"> / {}</tspan>
  </text>
"##,
            label_x, label_y, item.value, tspan_premium, item.unit
        ));
    }

    svg.push_str("</svg>\n");

    // 4. Save SVG file for reference/debugging
    let svg_out_path = "./output/value_comparison_chart.svg";
    let mut file = File::create(svg_out_path)?;
    file.write_all(svg.as_bytes())?;
    println!("Saved SVG to {}", svg_out_path);

    // 5. Render to PNG using resvg & usvg
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let mut opt = usvg::Options::default();
    opt.fontdb = std::sync::Arc::new(fontdb);

    let tree = usvg::Tree::from_str(&svg, &opt)?;

    // Create resvg Pixmap and render
    let size = tree.size();
    let pixmap_width = size.width() as u32;
    let pixmap_height = size.height() as u32;
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_width, pixmap_height)
        .ok_or_else(|| anyhow::anyhow!("Failed to create tiny-skia Pixmap"))?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let png_out_path = "./output/value_comparison_chart.png";
    pixmap.save_png(png_out_path)?;
    println!("Successfully rendered PNG graph to {}", png_out_path);

    Ok(())
}
