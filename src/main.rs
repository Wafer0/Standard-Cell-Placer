mod btree;
mod fp;
mod sa;
mod timing;
mod congestion;

use btree::BTree;
use sa::sa_floorplan;
use std::env;

fn print_usage() {
    eprintln!("Usage: floorplan <filename> <alpha> <times> <init_temp> <term_temp> [timing_weight] [congestion_weight]");
    eprintln!();
    eprintln!("Parameters:");
    eprintln!("  filename           - Input file name without extension (e.g., B10)");
    eprintln!("  alpha              - Weight for area in cost function (0.0-1.0)");
    eprintln!("  times              - Number of iterations per temperature");
    eprintln!("  init_temp          - Initial temperature for simulated annealing");
    eprintln!("  term_temp          - Terminal temperature for simulated annealing");
    eprintln!("  timing_weight      - (Optional) Weight for timing cost (default: 0.0)");
    eprintln!("  congestion_weight  - (Optional) Weight for congestion cost (default: 0.0)");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  floorplan B10 0.5 100 1000 0.01");
    eprintln!("  floorplan B10 0.5 100 1000 0.01 0.1 0.05");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        print_usage();
        std::process::exit(1);
    }

    let filename = &args[1];
    let alpha: f64 = args[2].parse().expect("Invalid alpha");
    let beta = 1.0 - alpha;
    let times: usize = args[3].parse().expect("Invalid times");
    let init_temp: f64 = args[4].parse().expect("Invalid init_temp");
    let term_temp: f64 = args[5].parse().expect("Invalid term_temp");
    
    let timing_weight: f64 = if args.len() > 6 {
        args[6].parse().expect("Invalid timing_weight")
    } else {
        0.0
    };
    
    let congestion_weight: f64 = if args.len() > 7 {
        args[7].parse().expect("Invalid congestion_weight")
    } else {
        0.0
    };

    let start_time = std::time::Instant::now();

    let mut fp = BTree::new(alpha, beta);
    fp.read(filename);
    println!("Read file finished...");

    fp.init();
    println!("Initial finished...");

    if timing_weight > 0.0 || congestion_weight > 0.0 {
        println!("Using advanced cost function:");
        if timing_weight > 0.0 {
            println!("  Timing weight: {}", timing_weight);
        }
        if congestion_weight > 0.0 {
            println!("  Congestion weight: {}", congestion_weight);
        }
    }

    sa_floorplan(&mut fp, filename, times, init_temp, term_temp, timing_weight, congestion_weight);

    let elapsed = start_time.elapsed().as_secs_f64();
    fp.print_result();
    println!("run time = {} s", elapsed);
}
