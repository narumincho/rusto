use rusto::map_override::{CIRCLE_GRID_SPACING, CIRCLE_SOURCE_HYPOTENUSE};
use std::fs;
use std::path::Path;

fn seafloor_mc_y(mc_x: f64) -> f64 {
    30.0 + 7.0 * (mc_x * 0.08).sin() + 3.0 * (mc_x * 0.23).cos()
}

fn to_svg_x(cx: f64, mc_x: f64, scale: f64) -> f64 {
    cx + mc_x * scale
}

fn to_svg_y(cy: f64, mc_y: f64, scale: f64) -> f64 {
    cy - (mc_y - 45.0) * scale
}

fn generate_seafloor_path(cx: f64, cy: f64, scale: f64, bottom_y: f64) -> (String, String) {
    let mut line_points = Vec::new();
    let steps = 168; // 840px / 5px
    for i in 0..=steps {
        let x = cx - 420.0 + (i as f64) * 5.0;
        let mc_x = (x - cx) / scale;
        let mc_y = seafloor_mc_y(mc_x);
        let y = to_svg_y(cy, mc_y, scale);
        line_points.push((x, y));
    }

    let mut line_d = format!("M {:.2} {:.2}", line_points[0].0, line_points[0].1);
    for pt in line_points.iter().skip(1) {
        line_d.push_str(&format!(" L {:.2} {:.2}", pt.0, pt.1));
    }

    let mut fill_d = line_d.clone();
    fill_d.push_str(&format!(
        " L {:.2} {:.2} L {:.2} {:.2} Z",
        cx + 420.0,
        bottom_y,
        cx - 420.0,
        bottom_y
    ));

    (line_d, fill_d)
}

fn generate_water_path(cx: f64, cy: f64, scale: f64) -> String {
    let water_level_mc_y = 62.0;
    let water_svg_y = to_svg_y(cy, water_level_mc_y, scale);

    let mut d = format!("M {:.2} {:.2}", cx - 420.0, water_svg_y);
    d.push_str(&format!(" L {:.2} {:.2}", cx + 420.0, water_svg_y));

    // Follow the seafloor from right to left
    let steps = 168;
    for i in (0..=steps).rev() {
        let x = cx - 420.0 + (i as f64) * 5.0;
        let mc_x = (x - cx) / scale;
        let mc_y = seafloor_mc_y(mc_x);
        let y = to_svg_y(cy, mc_y, scale);
        d.push_str(&format!(" L {:.2} {:.2}", x, y));
    }

    d.push_str(" Z");
    d
}

fn main() -> anyhow::Result<()> {
    // 1. Load constants and perform calculations
    let r = CIRCLE_SOURCE_HYPOTENUSE;
    let spacing = *CIRCLE_GRID_SPACING as f64;

    // Close case (distance = spacing = 131)
    let d_close = spacing;
    let y_diff_close = (r * r - (d_close / 2.0) * (d_close / 2.0)).sqrt();
    let y_intersect_close_upper = 45.0 + y_diff_close;
    let y_intersect_close_lower = 45.0 - y_diff_close;

    // Diagonal case (distance = spacing * sqrt(2) ≈ 185.26)
    let d_far = spacing * 2.0_f64.sqrt();
    let y_diff_far = (r * r - (d_far / 2.0) * (d_far / 2.0)).sqrt();
    let y_intersect_far_upper = 45.0 + y_diff_far;
    let y_intersect_far_lower = 45.0 - y_diff_far;

    println!("Calculated Conduit Spacing and Overlaps:");
    println!("  Radius (R): {}", r);
    println!("  Close Spacing (D): {} blocks", d_close);
    println!(
        "    Intersection Y-coordinates: {:.2}, {:.2}",
        y_intersect_close_upper, y_intersect_close_lower
    );
    println!("  Diagonal Spacing (D): {:.2} blocks", d_far);
    println!(
        "    Intersection Y-coordinates: {:.2}, {:.2}",
        y_intersect_far_upper, y_intersect_far_lower
    );

    // SVG parameters
    let width = 1920.0;
    let height = 950.0;
    let scale = 2.8;
    let cy = 450.0;
    let cx_left = 500.0;
    let cx_right = 1420.0;
    let top_y = 150.0;
    let bottom_y = 780.0;

    let (left_seafloor_stroke, left_seafloor_fill) =
        generate_seafloor_path(cx_left, cy, scale, bottom_y);
    let (right_seafloor_stroke, right_seafloor_fill) =
        generate_seafloor_path(cx_right, cy, scale, bottom_y);
    let left_water_fill = generate_water_path(cx_left, cy, scale);
    let right_water_fill = generate_water_path(cx_right, cy, scale);

    // Left panel conduit centers
    let x_l1 = to_svg_x(cx_left, -d_close / 2.0, scale);
    let x_l2 = to_svg_x(cx_left, d_close / 2.0, scale);

    // Right panel conduit centers
    let x_r1 = to_svg_x(cx_right, -d_far / 2.0, scale);
    let x_r2 = to_svg_x(cx_right, d_far / 2.0, scale);

    let r_px = r * scale;

    // Y ticks for ruler
    let y_ticks = vec![140, 120, 100, 80, 62, 45, 40, 20, 0, -20, -40, -60];

    let mut svg = String::new();
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="100%" height="100%">
  <defs>
    <!-- Import Outfit font from Google Fonts -->
    <style type="text/css">
      @import url('https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;700&amp;display=swap');
      text {{
        font-family: 'Outfit', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      }}
    </style>
    
    <!-- Background Gradient -->
    <linearGradient id="bg-grad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#0b0f19" />
      <stop offset="100%" stop-color="#161f30" />
    </linearGradient>
    
    <!-- Ocean Water Gradient -->
    <linearGradient id="water-grad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#00d2ff" stop-opacity="0.12" />
      <stop offset="100%" stop-color="#0066ff" stop-opacity="0.25" />
    </linearGradient>
    
    <!-- Ground Gradient -->
    <linearGradient id="ground-grad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#181d28" />
      <stop offset="100%" stop-color="#090a0f" />
    </linearGradient>
    
    <!-- Glow Filters -->
    <filter id="glow-cyan" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="6" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <filter id="glow-magenta" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="6" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <filter id="glow-conduit" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="8" result="blur" />
      <feColorMatrix type="matrix" values="0 0 0 0 0.0   0 0 0 0 1.0   0 0 0 0 1.0  0 0 0 0.8 0" />
      <feMerge>
        <feMergeNode />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    
    <!-- Overlap Region Fill Gradient -->
    <linearGradient id="overlap-grad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#ff00ff" stop-opacity="0.08" />
      <stop offset="100%" stop-color="#ab00ff" stop-opacity="0.18" />
    </linearGradient>
  </defs>

  <!-- Background -->
  <rect width="100%" height="100%" fill="url(#bg-grad)" />

  <!-- Main Dashboard Header -->
  <g transform="translate(960, 65)" text-anchor="middle">
    <text font-size="28" font-weight="700" fill="#ffffff" letter-spacing="3">CONDUIT EFFECT RANGE ANALYSIS</text>
    <text font-size="14" font-weight="300" fill="#6c7d9c" letter-spacing="1" dy="22">Side-View Vertical Cross-Section Slice &amp; Intersection Profile</text>
  </g>
"##,
        width, height
    ));

    // Render Panel helper closure
    let render_panel = |panel_idx: usize,
                        cx: f64,
                        title: &str,
                        subtitle: &str,
                        x1: f64,
                        x2: f64,
                        water_fill: &str,
                        seafloor_fill: &str,
                        seafloor_stroke: &str,
                        y_upper: f64,
                        y_lower: f64,
                        dist_val: f64|
     -> String {
        let mut p = String::new();

        let label_x_offset = if panel_idx == 0 { -435.0 } else { -435.0 };

        p.push_str(&format!(
            r##"
  <!-- ==================== PANEL {} ==================== -->
  <g id="panel-{}">
    <!-- Panel Header -->
    <g transform="translate({}, 115)" text-anchor="middle">
      <text font-size="18" font-weight="600" fill="#00e5ff" letter-spacing="1">{}</text>
      <text font-size="12" font-weight="300" fill="#a0aec0" dy="18">{}</text>
    </g>

    <!-- Panel Border / Clipping Area Container -->
    <g>
      <!-- Panel Background Mask / Base Grid Area -->
      <rect x="{}" y="{}" width="840" height="630" fill="#0d1321" rx="8" stroke="#1d263b" stroke-width="1.5" />
      
      <!-- Subtle internal vertical grid lines for depth/scale -->
      <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#161f33" stroke-width="1" />
      <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#161f33" stroke-width="1" />
      <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#161f33" stroke-width="1" />

      <!-- Water Mass -->
      <path d="{}" fill="url(#water-grad)" />

      <!-- Ground Mass -->
      <path d="{}" fill="url(#ground-grad)" />
      
      <!-- Seafloor Line -->
      <path d="{}" fill="none" stroke="#2c3e50" stroke-width="3" />
      <path d="{}" fill="none" stroke="#10b981" stroke-width="1.5" opacity="0.8" />
"##,
            panel_idx + 1, panel_idx + 1,
            cx, title, subtitle,
            cx - 420.0, top_y,
            cx, top_y, cx, bottom_y, // center line
            cx - 210.0, top_y, cx - 210.0, bottom_y, // left quarter line
            cx + 210.0, top_y, cx + 210.0, bottom_y, // right quarter line
            water_fill,
            seafloor_fill,
            seafloor_stroke,
            seafloor_stroke
        ));

        // Horizontal Ruler and Ticks
        p.push_str("      <!-- Vertical Depth Ruler & Horizontal Reference Grid -->\n");
        for &tick_y in &y_ticks {
            let svg_y_val = to_svg_y(cy, tick_y as f64, scale);

            // Grid lines inside the panel
            let (stroke_color, stroke_width, dash_style) = if tick_y == 62 {
                ("#00a2ff", "1.5", "") // Sea Level
            } else if tick_y == 45 {
                ("#00ffff", "1.5", "3,3") // Conduit Level
            } else if tick_y == 20 || tick_y == 40 {
                ("#10b981", "1.0", "4,4") // Seafloor boundaries
            } else {
                ("#1f293d", "0.75", "") // standard ticks
            };

            p.push_str(&format!(
                r##"      <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" {} />
"##,
                cx - 420.0,
                svg_y_val,
                cx + 420.0,
                svg_y_val,
                stroke_color,
                stroke_width,
                if dash_style.is_empty() {
                    "".to_string()
                } else {
                    format!("stroke-dasharray=\"{}\"", dash_style)
                }
            ));

            // Ticks text label next to the ruler line (at the left border)
            let tick_label = if tick_y == 62 {
                "Y=62 (Sea)".to_string()
            } else if tick_y == 45 {
                "Y=45 (Conduit)".to_string()
            } else if tick_y == 40 {
                "Y=40 (Bed max)".to_string()
            } else if tick_y == 20 {
                "Y=20 (Bed min)".to_string()
            } else {
                format!("Y={}", tick_y)
            };

            p.push_str(&format!(
                r##"      <text x="{}" y="{}" fill="#6c7d9c" font-size="10" font-weight="600" text-anchor="end" dx="-10" dy="3">{}</text>
"##,
                cx + label_x_offset, svg_y_val, tick_label
            ));
        }

        // Overlap region highlighters
        let svg_y_upper = to_svg_y(cy, y_upper, scale);
        let svg_y_lower = to_svg_y(cy, y_lower, scale);

        p.push_str("      <!-- Coverage Circles and Overlap Area -->\n");
        // We crop/clip circles so they do not bleed outside the panel box (cx-420 to cx+420, top_y to bottom_y)
        p.push_str(&format!(
            r##"      <g clip-path="url(#panel-clip-{})">
        <!-- Overlap area -->
        <path d="M {:.2} {:.2} A {:.2} {:.2} 0 0 1 {:.2} {:.2} A {:.2} {:.2} 0 0 1 {:.2} {:.2}" fill="url(#overlap-grad)" stroke="#ff00ff" stroke-width="2" filter="url(#glow-magenta)" />
        
        <!-- Left Circle -->
        <circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none" stroke="#00ffff" stroke-width="1.5" stroke-opacity="0.65" />
        <circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none" stroke="#00ffff" stroke-width="1.0" stroke-opacity="0.3" stroke-dasharray="5 5" />
        
        <!-- Right Circle -->
        <circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none" stroke="#00ffff" stroke-width="1.5" stroke-opacity="0.65" />
        <circle cx="{:.2}" cy="{:.2}" r="{:.2}" fill="none" stroke="#00ffff" stroke-width="1.0" stroke-opacity="0.3" stroke-dasharray="5 5" />
      </g>
"##,
            panel_idx + 1,
            cx, svg_y_upper, r_px, r_px, cx, svg_y_lower, r_px, r_px, cx, svg_y_upper,
            x1, cy, r_px,
            x1, cy, r_px,
            x2, cy, r_px,
            x2, cy, r_px
        ));

        // Conduit Nodes
        p.push_str(&format!(
            r##"      <!-- Conduit Core Nodes -->
      <g filter="url(#glow-conduit)">
        <circle cx="{:.2}" cy="{:.2}" r="8" fill="#ffffff" />
        <circle cx="{:.2}" cy="{:.2}" r="4" fill="#00ffff" />
        <circle cx="{:.2}" cy="{:.2}" r="8" fill="#ffffff" />
        <circle cx="{:.2}" cy="{:.2}" r="4" fill="#00ffff" />
      </g>
      <!-- Horizontal distance line between conduits -->
      <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#00ffff" stroke-width="1.5" stroke-dasharray="2,2" opacity="0.7" />
      <rect x="{:.2}" y="{:.2}" width="100" height="20" rx="4" fill="#0d1321" stroke="#00e5ff" stroke-width="1" transform="translate(-50, -10)" />
      <text x="{:.2}" y="{:.2}" fill="#00ffff" font-size="10" font-weight="700" text-anchor="middle" dy="4">D = {:.2}m</text>
"##,
            x1, cy, x1, cy,
            x2, cy, x2, cy,
            x1, cy, x2, cy,
            cx, cy,
            cx, cy, dist_val
        ));

        // Highlight intersection points and display their Y coordinate values
        p.push_str(&format!(
            r##"
      <!-- Intersection Marks and Labels -->
      <!-- Upper Intersection -->
      <g>
        <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#ff00ff" stroke-width="1.25" stroke-dasharray="3,3" />
        <circle cx="{:.2}" cy="{:.2}" r="5" fill="#ffffff" stroke="#ff00ff" stroke-width="2" filter="url(#glow-magenta)" />
        <circle cx="{:.2}" cy="{:.2}" r="2" fill="#ff00ff" />
        
        <!-- Label bubble -->
        <rect x="{:.2}" y="{:.2}" width="95" height="24" rx="4" fill="#180e22" stroke="#ff00ff" stroke-width="1" transform="translate(12, -12)" />
        <text x="{:.2}" y="{:.2}" fill="#ff55ff" font-size="11" font-weight="700" text-anchor="start" dx="20" dy="4">Y = {:.2}</text>
      </g>
      
      <!-- Lower Intersection -->
      <g>
        <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#ff00ff" stroke-width="1.25" stroke-dasharray="3,3" />
        <circle cx="{:.2}" cy="{:.2}" r="5" fill="#ffffff" stroke="#ff00ff" stroke-width="2" filter="url(#glow-magenta)" />
        <circle cx="{:.2}" cy="{:.2}" r="2" fill="#ff00ff" />
        
        <!-- Label bubble -->
        <rect x="{:.2}" y="{:.2}" width="95" height="24" rx="4" fill="#180e22" stroke="#ff00ff" stroke-width="1" transform="translate(12, -12)" />
        <text x="{:.2}" y="{:.2}" fill="#ff55ff" font-size="11" font-weight="700" text-anchor="start" dx="20" dy="4">Y = {:.2}</text>
      </g>
"##,
            cx - 420.0, svg_y_upper, cx + 420.0, svg_y_upper,
            cx, svg_y_upper,
            cx, svg_y_upper,
            cx, svg_y_upper,
            cx, svg_y_upper, y_upper,

            cx - 420.0, svg_y_lower, cx + 420.0, svg_y_lower,
            cx, svg_y_lower,
            cx, svg_y_lower,
            cx, svg_y_lower,
            cx, svg_y_lower, y_lower
        ));

        p.push_str(&format!(
            r##"    </g>

    <!-- Clip path definition for this panel -->
    <clipPath id="panel-clip-{}">
      <rect x="{}" y="{}" width="840" height="630" rx="8" />
    </clipPath>
  </g>
"##,
            panel_idx + 1,
            cx - 420.0,
            top_y
        ));

        p
    };

    // Render Left Panel (Close Case)
    svg.push_str(&render_panel(
        0,
        cx_left,
        "CLOSE COUPLING SLICE (ADJACENT)",
        &format!(
            "Cross section along Z-axis (or X-axis) | Separation: {} blocks",
            d_close
        ),
        x_l1,
        x_l2,
        &left_water_fill,
        &left_seafloor_fill,
        &left_seafloor_stroke,
        y_intersect_close_upper,
        y_intersect_close_lower,
        d_close,
    ));

    // Render Right Panel (Diagonal Case)
    svg.push_str(&render_panel(
        1,
        cx_right,
        "DIAGONAL SLICE (FAR COUPLING)",
        &format!(
            "Cross section along diagonal grid line (45°) | Separation: {:.2} blocks",
            d_far
        ),
        x_r1,
        x_r2,
        &right_water_fill,
        &right_seafloor_fill,
        &right_seafloor_stroke,
        y_intersect_far_upper,
        y_intersect_far_lower,
        d_far,
    ));

    // Footer/Legend info
    svg.push_str(&format!(
        r##"
  <!-- General Legend/Metadata -->
  <g transform="translate(960, 890)" text-anchor="middle" font-size="12" fill="#6c7d9c">
    <rect x="-350" y="-20" width="700" height="35" rx="6" fill="#0d1321" stroke="#1d263b" stroke-width="1" />
    <g transform="translate(-250, 2)">
      <circle cx="0" cy="-2" r="5" fill="#00ffff" />
      <text x="12" y="2" text-anchor="start" font-weight="600">Conduit Node (Y=45)</text>
    </g>
    <g transform="translate(-70, 2)">
      <circle cx="0" cy="-2" r="5" fill="none" stroke="#00ffff" stroke-width="1.5" />
      <text x="12" y="2" text-anchor="start" font-weight="600">Coverage Boundary (R=96)</text>
    </g>
    <g transform="translate(130, 2)">
      <rect x="-6" y="-7" width="12" height="10" fill="url(#overlap-grad)" stroke="#ff00ff" stroke-width="1" />
      <text x="12" y="2" text-anchor="start" font-weight="600">Overlap Area (Intersection)</text>
    </g>
  </g>
</svg>
"##
    ));

    // Write file to output folder
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    let output_path = output_dir.join("conduit_coverage.svg");
    fs::write(&output_path, svg)?;
    println!(
        "Successfully generated conduit coverage SVG at {:?}",
        output_path
    );

    Ok(())
}
