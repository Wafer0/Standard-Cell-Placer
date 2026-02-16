// Parallel simulated annealing implementation using the Rayon library.
// Executes multiple independent SA chains concurrently and aggregates results.

use crate::btree::BTree;
use crate::sa::sa_floorplan;
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct SAResult {
    pub cost: f64,
    pub area: f64,
    pub wire_length: f64,
    pub width: f64,
    pub height: f64,
    pub run_id: usize,
}

// Runs SA in parallel with multiple seeds and returns all results
pub fn parallel_sa_floorplan(
    filename: &str,
    alpha: f64,
    beta: f64,
    times: usize,
    init_temp: f64,
    term_temp: f64,
    num_runs: usize,
    timing_weight: f64,
    congestion_weight: f64,
) -> (BTree, Vec<SAResult>) {
    println!("\n========== PARALLEL SIMULATED ANNEALING ==========");
    println!("Running {} independent SA chains in parallel...", num_runs);
    println!("==================================================\n");

    // Run parallel SA chains
    let results: Vec<(BTree, SAResult)> = (0..num_runs)
        .into_par_iter()
        .map(|run_id| {
            let mut fp = BTree::new(alpha, beta);
            fp.read(filename);
            fp.init();

            // Use different output filename for each run
            let run_filename = format!("{}_run{}", filename, run_id);
            sa_floorplan(
                &mut fp,
                &run_filename,
                times,
                init_temp,
                term_temp,
                timing_weight,
                congestion_weight,
            );

            let result = SAResult {
                cost: fp.get_cost(),
                area: fp.get_area(),
                wire_length: fp.get_wire_length(),
                width: fp.get_width(),
                height: fp.get_height(),
                run_id,
            };

            (fp, result)
        })
        .collect();

    // Find best result
    let best_idx = results
        .iter()
        .enumerate()
        .min_by(|(_, (_, r1)), (_, (_, r2))| r1.cost.partial_cmp(&r2.cost).unwrap())
        .map(|(idx, _)| idx)
        .unwrap();

    let (best_fp, best_result) = &results[best_idx];
    let all_results: Vec<SAResult> = results.iter().map(|(_, r)| r.clone()).collect();

    println!("\n========== PARALLEL SA SUMMARY ==========");
    println!("Run    Cost         Area         WireLength   Width    Height");
    println!("---------------------------------------------------------------------");
    for result in &all_results {
        let marker = if result.run_id == best_result.run_id {
            " *"
        } else {
            "  "
        };
        println!(
            "{}{}  {:.2e}  {:.2e}  {:.2e}  {:.2}  {:.2}",
            marker,
            result.run_id,
            result.cost,
            result.area,
            result.wire_length,
            result.width,
            result.height
        );
    }
    println!("=========================================");
    println!("* = Best solution");

    // Calculate statistics
    let costs: Vec<f64> = all_results.iter().map(|r| r.cost).collect();
    let avg_cost: f64 = costs.iter().sum::<f64>() / costs.len() as f64;
    let min_cost = costs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_cost = costs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    println!("\nCost Statistics:");
    println!("  Best:    {:.2e}", min_cost);
    println!("  Average: {:.2e}", avg_cost);
    println!("  Worst:   {:.2e}", max_cost);
    println!("  Std Dev: {:.2e}", calculate_stddev(&costs, avg_cost));

    (best_fp.clone(), all_results)
}

fn calculate_stddev(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}
