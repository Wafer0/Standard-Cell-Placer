// Standalone visualization tool for floorplan placements.

use standard_cell_placer::visualization::visualize_placement;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: visualize <filename>");
        eprintln!("Example: visualize B10");
        std::process::exit(1);
    }
    
    let filename = &args[1];
    println!("Generating SVG visualization for: {}", filename);
    
    match visualize_placement(filename) {
        Ok(_) => println!("Visualization complete!"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
