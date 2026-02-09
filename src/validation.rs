// Validation module for verifying the correctness of floorplan placements.
// Performs overlap detection, area consistency checks, and boundary validation.

use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub has_overlaps: bool,
    pub overlapping_pairs: Vec<(String, String)>,
    pub area_valid: bool,
    pub computed_area: f64,
    pub reported_area: f64,
    pub total_module_area: f64,
    pub out_of_bounds: Vec<String>,
}

impl ValidationResult {
    pub fn print(&self) {
        println!("\n========== VALIDATION RESULTS ==========");
        if self.is_valid {
            println!("✓ Placement is VALID");
        } else {
            println!("✗ Placement has ERRORS");
        }
        
        println!("\n--- Overlap Check ---");
        if self.has_overlaps {
            println!("✗ Found {} overlapping pairs:", self.overlapping_pairs.len());
            for (m1, m2) in &self.overlapping_pairs {
                println!("  - {} overlaps with {}", m1, m2);
            }
        } else {
            println!("✓ No overlaps detected");
        }
        
        println!("\n--- Area Check ---");
        println!("Bounding box area: {:.2}", self.computed_area);
        println!("Reported area: {:.2}", self.reported_area);
        println!("Total module area: {:.2}", self.total_module_area);
        let utilization = (self.total_module_area / self.computed_area) * 100.0;
        println!("Utilization: {:.2}%", utilization);
        
        if (self.computed_area - self.reported_area).abs() < 0.01 {
            println!("✓ Area values match");
        } else {
            println!("✗ Area mismatch!");
        }
        
        if !self.out_of_bounds.is_empty() {
            println!("\n--- Out of Bounds ---");
            println!("✗ {} modules extend beyond bounding box:", self.out_of_bounds.len());
            for m in &self.out_of_bounds {
                println!("  - {}", m);
            }
        }
        
        println!("========================================\n");
    }
}

// Checks if two rectangles overlap
fn rectangles_overlap(x1: f64, y1: f64, w1: f64, h1: f64, 
                      x2: f64, y2: f64, w2: f64, h2: f64) -> bool {
    // No overlap if one rectangle is to the left of the other
    if x1 + w1 <= x2 || x2 + w2 <= x1 {
        return false;
    }
    // No overlap if one rectangle is above the other
    if y1 + h1 <= y2 || y2 + h2 <= y1 {
        return false;
    }
    true
}

// Validates a placement file
pub fn validate_placement(filename_short: &str) -> ValidationResult {
    let blocks_file = format!("./input/{}.blocks", filename_short);
    let placement_file = format!("./out/{}.pl", filename_short);
    let result_file = format!("./out/{}.result", filename_short);
    
    // Read module dimensions from .blocks file
    let mut modules = Vec::new();
    let file = File::open(&blocks_file).expect("Failed to open blocks file");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let line1 = lines.next().unwrap().unwrap();
    let parts1: Vec<&str> = line1.split(':').collect();
    let _num_softblock: usize = parts1[1].trim().parse().unwrap();
    
    let line2 = lines.next().unwrap().unwrap();
    let parts2: Vec<&str> = line2.split(':').collect();
    let num_modules: usize = parts2[1].trim().parse().unwrap();
    
    let line3 = lines.next().unwrap().unwrap();
    let parts3: Vec<&str> = line3.split(':').collect();
    let _num_terminals: usize = parts3[1].trim().parse().unwrap();
    
    lines.next(); // Skip empty line
    
    for _i in 0..num_modules {
        let line = lines.next().unwrap().unwrap();
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
        
        modules.push((blockname, 0.0, 0.0, width, height));
    }
    
    // Read placement coordinates from .pl file
    let file = File::open(&placement_file).expect("Failed to open placement file");
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            break; // End of modules
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0];
            let x: f64 = parts[1].parse().unwrap();
            let y: f64 = parts[2].parse().unwrap();
            
            // Update module position
            for module in &mut modules {
                if module.0 == name {
                    module.1 = x;
                    module.2 = y;
                    break;
                }
            }
        }
    }
    
    // Check for overlaps
    let mut overlapping_pairs = Vec::new();
    for i in 0..modules.len() {
        for j in (i+1)..modules.len() {
            if rectangles_overlap(
                modules[i].1, modules[i].2, modules[i].3, modules[i].4,
                modules[j].1, modules[j].2, modules[j].3, modules[j].4
            ) {
                overlapping_pairs.push((modules[i].0.clone(), modules[j].0.clone()));
            }
        }
    }
    
    // Calculate bounding box
    let mut max_x = 0.0_f64;
    let mut max_y = 0.0_f64;
    for module in &modules {
        max_x = max_x.max(module.1 + module.3);
        max_y = max_y.max(module.2 + module.4);
    }
    let computed_area = max_x * max_y;
    
    // Calculate total module area
    let total_module_area: f64 = modules.iter().map(|m| m.3 * m.4).sum();
    
    // Read reported results
    let file = File::open(&result_file).expect("Failed to open result file");
    let reader = BufReader::new(file);
    let mut reported_area = 0.0;
    
    for line in reader.lines() {
        let line = line.unwrap();
        if line.contains("Total area") {
            let parts: Vec<&str> = line.split('=').collect();
            reported_area = parts[1].trim().parse().unwrap();
            break;
        }
    }
    
    // Check for out of bounds modules
    let mut out_of_bounds = Vec::new();
    for module in &modules {
        if module.1 < 0.0 || module.2 < 0.0 ||
           module.1 + module.3 > max_x + 0.01 ||
           module.2 + module.4 > max_y + 0.01 {
            out_of_bounds.push(module.0.clone());
        }
    }
    
    ValidationResult {
        is_valid: overlapping_pairs.is_empty() && out_of_bounds.is_empty(),
        has_overlaps: !overlapping_pairs.is_empty(),
        overlapping_pairs,
        area_valid: (computed_area - reported_area).abs() < 0.01,
        computed_area,
        reported_area,
        total_module_area,
        out_of_bounds,
    }
}
