// Standalone validation tool for floorplan placements.

use standard_cell_placer::validation::validate_placement;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: validate <filename>");
        eprintln!("Example: validate B10");
        std::process::exit(1);
    }
    
    let filename = &args[1];
    println!("Validating placement for: {}", filename);
    
    let result = validate_placement(filename);
    result.print();
    
    if !result.is_valid {
        std::process::exit(1);
    }
}
