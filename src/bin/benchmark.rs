// Comprehensive benchmarking suite for the floorplan placer.
// Executes parallel SA runs across multiple benchmark designs with automated validation.

use standard_cell_placer::parallel_sa::{parallel_sa_floorplan, SAResult};
use standard_cell_placer::validation::validate_placement;
use standard_cell_placer::visualization::visualize_placement;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

struct BenchmarkConfig {
    filename: String,
    alpha: f64,
    times: usize,
    init_temp: f64,
    term_temp: f64,
    num_runs: usize,
    timing_weight: f64,
    congestion_weight: f64,
}

impl BenchmarkConfig {
    fn new(filename: String) -> Self {
        BenchmarkConfig {
            filename,
            alpha: 0.5,
            times: 100,
            init_temp: 1000.0,
            term_temp: 0.01,
            num_runs: 8,
            timing_weight: 0.0,
            congestion_weight: 0.0,
        }
    }

    fn with_params(
        filename: String,
        alpha: f64,
        times: usize,
        init_temp: f64,
        term_temp: f64,
        num_runs: usize,
        timing_weight: f64,
        congestion_weight: f64,
    ) -> Self {
        BenchmarkConfig {
            filename,
            alpha,
            times,
            init_temp,
            term_temp,
            num_runs,
            timing_weight,
            congestion_weight,
        }
    }
}

fn run_benchmark(config: &BenchmarkConfig) -> (f64, SAResult) {
    println!("\n{}", "=".repeat(70));
    println!("Benchmarking: {}", config.filename);
    println!("{}\n", "=".repeat(70));

    let start_time = Instant::now();

    let beta = 1.0 - config.alpha;
    let (mut best_fp, results) = parallel_sa_floorplan(
        &config.filename,
        config.alpha,
        beta,
        config.times,
        config.init_temp,
        config.term_temp,
        config.num_runs,
        config.timing_weight,
        config.congestion_weight,
    );

    let elapsed = start_time.elapsed().as_secs_f64();

    // Save best result
    best_fp.print_result();

    // Validate
    println!("\nValidating placement...");
    let validation = validate_placement(&config.filename);
    validation.print();

    // Visualize
    println!("Generating visualization...");
    match visualize_placement(&config.filename) {
        Ok(_) => println!("✓ Visualization saved"),
        Err(e) => eprintln!("✗ Visualization failed: {}", e),
    }

    let best_result = results
        .iter()
        .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap())
        .unwrap()
        .clone();

    (elapsed, best_result)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    // Usage: benchmark [benchmarks...] or
    //        benchmark <benchmark> <num_runs> <alpha> <times> <init_temp> <term_temp> [timing_weight] [congestion_weight]

    let (benchmarks, config_params): (
        Vec<String>,
        Option<(usize, f64, usize, f64, f64, f64, f64)>,
    ) = if args.len() >= 7 {
        // Custom parameters provided
        let num_runs = args[2].parse().unwrap_or(8);
        let alpha = args[3].parse().unwrap_or(0.5);
        let times = args[4].parse().unwrap_or(100);
        let init_temp = args[5].parse().unwrap_or(1000.0);
        let term_temp = args[6].parse().unwrap_or(0.01);
        let timing_weight = if args.len() > 7 {
            args[7].parse().unwrap_or(0.0)
        } else {
            0.0
        };
        let congestion_weight = if args.len() > 8 {
            args[8].parse().unwrap_or(0.0)
        } else {
            0.0
        };
        (
            vec![args[1].clone()],
            Some((
                num_runs,
                alpha,
                times,
                init_temp,
                term_temp,
                timing_weight,
                congestion_weight,
            )),
        )
    } else if args.len() > 1 {
        // Run specific benchmarks with default params
        (args[1..].iter().map(|s| s.to_string()).collect(), None)
    } else {
        // Run all default benchmarks
        (
            vec!["B10", "B30", "B50", "B100", "B200", "B300"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            None,
        )
    };

    println!("\n{}", "#".repeat(70));
    println!("# COMPREHENSIVE FLOORPLAN BENCHMARK SUITE");
    println!("{}\n", "#".repeat(70));

    let mut results = Vec::new();

    for benchmark in &benchmarks {
        let config = if let Some((
            num_runs,
            alpha,
            times,
            init_temp,
            term_temp,
            timing_weight,
            congestion_weight,
        )) = config_params
        {
            BenchmarkConfig::with_params(
                benchmark.clone(),
                alpha,
                times,
                init_temp,
                term_temp,
                num_runs,
                timing_weight,
                congestion_weight,
            )
        } else {
            BenchmarkConfig::new(benchmark.clone())
        };
        let (elapsed, result) = run_benchmark(&config);
        results.push((benchmark.clone(), elapsed, result));
    }

    // Generate summary report
    println!("\n\n{}", "=".repeat(70));
    println!("BENCHMARK SUMMARY");
    println!("{}\n", "=".repeat(70));

    println!(
        "{:<10} {:>12} {:>12} {:>12} {:>10} {:>10}",
        "Benchmark", "Runtime(s)", "Cost", "Area", "WireLen", "Util(%)"
    );
    println!("{}", "-".repeat(70));

    for (name, elapsed, result) in &results {
        // Read module count for utilization calc
        let blocks_file = format!("./input/{}.blocks", name);
        let total_area = if let Ok(file) = File::open(&blocks_file) {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            let line1 = lines.next().unwrap().unwrap();
            let parts1: Vec<&str> = line1.split(':').collect();
            let _num_softblock: usize = parts1[1].trim().parse().unwrap();

            let line2 = lines.next().unwrap().unwrap();
            let parts2: Vec<&str> = line2.split(':').collect();
            let num_modules: usize = parts2[1].trim().parse().unwrap();

            lines.next(); // Skip terminals line
            lines.next(); // Skip empty line

            let mut total = 0.0;
            for _i in 0..num_modules {
                let line = lines.next().unwrap().unwrap();
                let parts: Vec<&str> = line.split_whitespace().collect();

                let x1_str = parts[3].trim_start_matches('(').trim_end_matches(',');
                let y1_str = parts[4].trim_end_matches(')');
                let x3_str = parts[7].trim_start_matches('(').trim_end_matches(',');
                let y3_str = parts[8].trim_end_matches(')');

                let x1: f64 = x1_str.parse().unwrap();
                let y1: f64 = y1_str.parse().unwrap();
                let x3: f64 = x3_str.parse().unwrap();
                let y3: f64 = y3_str.parse().unwrap();

                total += (x3 - x1) * (y3 - y1);
            }
            total
        } else {
            0.0
        };

        let utilization = if result.area > 0.0 {
            (total_area / result.area) * 100.0
        } else {
            0.0
        };

        println!(
            "{:<10} {:>12.2} {:>12.2e} {:>12.2e} {:>12.2e} {:>9.1}%",
            name, elapsed, result.cost, result.area, result.wire_length, utilization
        );
    }

    println!("\n{}\n", "=".repeat(70));

    // Save JSON results
    let json_output = format!("./out/benchmark_results.json");
    if let Ok(mut file) = File::create(&json_output) {
        writeln!(file, "{{").unwrap();
        writeln!(file, "  \"benchmarks\": [").unwrap();
        for (i, (name, elapsed, result)) in results.iter().enumerate() {
            let comma = if i < results.len() - 1 { "," } else { "" };
            writeln!(file, "    {{").unwrap();
            writeln!(file, "      \"name\": \"{}\",", name).unwrap();
            writeln!(file, "      \"runtime_seconds\": {},", elapsed).unwrap();
            writeln!(file, "      \"cost\": {},", result.cost).unwrap();
            writeln!(file, "      \"area\": {},", result.area).unwrap();
            writeln!(file, "      \"wirelength\": {},", result.wire_length).unwrap();
            writeln!(file, "      \"width\": {},", result.width).unwrap();
            writeln!(file, "      \"height\": {}", result.height).unwrap();
            writeln!(file, "    }}{}", comma).unwrap();
        }
        writeln!(file, "  ]").unwrap();
        writeln!(file, "}}").unwrap();
        println!("Results saved to: {}", json_output);
    }
}
