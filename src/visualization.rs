// Visualization module for generating SVG floorplan diagrams.
// Produces publication-quality vector graphics of placement results.

use std::fs::File;
use std::io::Write;

#[derive(Clone)]
pub struct Rectangle {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub is_terminal: bool,
}

// Generates an SVG visualization of the floorplan
pub fn generate_svg(
    filename: &str,
    modules: Vec<Rectangle>,
    terminals: Vec<Rectangle>,
    chip_width: f64,
    chip_height: f64,
) -> std::io::Result<()> {
    let margin = 50.0;
    let scale_factor = 800.0 / chip_width.max(chip_height);
    let svg_width = chip_width * scale_factor + 2.0 * margin;
    let svg_height = chip_height * scale_factor + 2.0 * margin;

    let output_file = format!("./out/{}.svg", filename);
    let mut file = File::create(&output_file)?;

    // SVG header
    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        file,
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        svg_width, svg_height
    )?;

    // Background
    writeln!(
        file,
        "  <rect width=\"100%\" height=\"100%\" fill=\"#f5f5f5\"/>"
    )?;

    // Title
    writeln!(
        file,
        "  <text x=\"{}\" y=\"30\" font-family=\"Arial\" font-size=\"20\" font-weight=\"bold\" text-anchor=\"middle\">Floorplan: {}</text>",
        svg_width / 2.0,
        filename
    )?;

    // Chip boundary
    writeln!(
        file,
        "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"2\"/>",
        margin,
        margin,
        chip_width * scale_factor,
        chip_height * scale_factor
    )?;

    // Color palette for modules
    let colors = vec![
        "#FFB6C1", "#87CEEB", "#98FB98", "#DDA0DD", "#F0E68C", "#FFE4B5", "#B0E0E6", "#FFA07A",
        "#D8BFD8", "#AFEEEE",
    ];

    // Draw modules
    for (i, module) in modules.iter().enumerate() {
        let x = margin + module.x * scale_factor;
        let y = margin + module.y * scale_factor;
        let w = module.width * scale_factor;
        let h = module.height * scale_factor;
        let color = colors[i % colors.len()];

        // Module rectangle
        writeln!(
            file,
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"#333\" stroke-width=\"1.5\"/>",
            x, y, w, h, color
        )?;

        // Module label
        let font_size = (10.0_f64).min(h / 3.0).max(6.0);
        writeln!(
            file,
            "  <text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{}</text>",
            x + w / 2.0,
            y + h / 2.0,
            font_size,
            module.name
        )?;
    }

    // Draw terminals
    for terminal in terminals.iter() {
        let x = margin + terminal.x * scale_factor;
        let y = margin + terminal.y * scale_factor;
        let size = 6.0;

        // Terminal circle
        writeln!(
            file,
            "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"red\" stroke=\"darkred\" stroke-width=\"1\"/>",
            x, y, size
        )?;

        // Terminal label (small)
        writeln!(
            file,
            "  <text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"8\" text-anchor=\"middle\">{}</text>",
            x,
            y - 10.0,
            terminal.name
        )?;
    }

    // Legend
    let legend_y = svg_height - 30.0;
    writeln!(
        file,
        "  <text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"12\">Dimensions: {:.1} x {:.1}</text>",
        margin,
        legend_y,
        chip_width,
        chip_height
    )?;

    writeln!(file, "</svg>")?;

    println!("SVG visualization saved to: {}", output_file);
    Ok(())
}

// Reads placement and generates visualization
pub fn visualize_placement(filename: &str) -> std::io::Result<()> {
    use std::io::BufRead;

    // Read module dimensions from .blocks file
    let blocks_file = format!("./input/{}.blocks", filename);
    let file = File::open(&blocks_file)?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let line1 = lines.next().unwrap()?;
    let parts1: Vec<&str> = line1.split(':').collect();
    let _num_softblock: usize = parts1[1].trim().parse().unwrap();

    let line2 = lines.next().unwrap()?;
    let parts2: Vec<&str> = line2.split(':').collect();
    let num_modules: usize = parts2[1].trim().parse().unwrap();

    let line3 = lines.next().unwrap()?;
    let parts3: Vec<&str> = line3.split(':').collect();
    let _num_terminals: usize = parts3[1].trim().parse().unwrap();

    lines.next(); // Skip empty line

    let mut module_dims = Vec::new();
    for _i in 0..num_modules {
        let line = lines.next().unwrap()?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        let blockname = parts[0].to_string();

        let x1_str = parts[3].trim_start_matches('(').trim_end_matches(',');
        let y1_str = parts[4].trim_end_matches(')');
        let x3_str = parts[7].trim_start_matches('(').trim_end_matches(',');
        let y3_str = parts[8].trim_end_matches(')');

        let x1: f64 = x1_str.parse().unwrap();
        let y1: f64 = y1_str.parse().unwrap();
        let x3: f64 = x3_str.parse().unwrap();
        let y3: f64 = y3_str.parse().unwrap();

        let width = x3 - x1;
        let height = y3 - y1;

        module_dims.push((blockname, width, height));
    }

    // Read placement
    let placement_file = format!("./out/{}.pl", filename);
    let file = File::open(&placement_file)?;
    let reader = std::io::BufReader::new(file);

    let mut modules = Vec::new();
    let mut terminals = Vec::new();
    let mut in_terminals = false;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            in_terminals = true;
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0].to_string();
            let x: f64 = parts[1].parse().unwrap();
            let y: f64 = parts[2].parse().unwrap();

            if !in_terminals {
                // Find dimensions
                let dims = module_dims.iter().find(|(n, _, _)| n == &name);
                if let Some((_, w, h)) = dims {
                    modules.push(Rectangle {
                        name,
                        x,
                        y,
                        width: *w,
                        height: *h,
                        is_terminal: false,
                    });
                }
            } else {
                terminals.push(Rectangle {
                    name,
                    x,
                    y,
                    width: 0.0,
                    height: 0.0,
                    is_terminal: true,
                });
            }
        }
    }

    // Calculate chip dimensions
    let mut chip_width: f64 = 0.0;
    let mut chip_height: f64 = 0.0;
    for module in &modules {
        chip_width = chip_width.max(module.x + module.width);
        chip_height = chip_height.max(module.y + module.height);
    }

    generate_svg(filename, modules, terminals, chip_width, chip_height)
}
