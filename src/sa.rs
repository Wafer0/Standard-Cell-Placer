use crate::btree::BTree;
use crate::fp::rand_01;
use std::fs::File;
use std::io::Write;

pub fn sa_floorplan(
    fp: &mut BTree,
    filename_short: &str,
    k: usize,
    init_t: f64,
    term_t: f64,
    timing_weight: f64,
    congestion_weight: f64,
) {
    let mut mt;
    let mut uphill;
    let mut reject;
    let mut pre_cost;
    let mut best;
    let mut cost;
    let mut d_cost;
    let mut reject_rate;

    let n = k * fp.size();
    let mut t = init_t;
    let conv_rate = 0.99;
    let tratio = 0.85;

    let use_advanced = timing_weight > 0.0 || congestion_weight > 0.0;
    let clock_period = 10.0;
    let grid_size = 20;

    fp.packing();
    fp.keep_sol();
    fp.keep_best();

    pre_cost = if use_advanced {
        fp.get_cost_with_timing_congestion(
            timing_weight,
            congestion_weight,
            clock_period,
            grid_size,
        )
    } else {
        fp.get_cost()
    };
    best = pre_cost;

    let mut count = 0;

    let f_dir = format!("./out/{}.SA", filename_short);
    let mut frsa = File::create(&f_dir).expect("Failed to create SA output file");
    let start_time = std::time::Instant::now();

    loop {
        count += 1;
        mt = 0;
        uphill = 0;
        reject = 0;
        println!("Iteration {}, T= {:.2}", count, t);

        while uphill < n && mt < 2 * n {
            fp.perturb();
            fp.packing();

            cost = if use_advanced {
                fp.get_cost_with_timing_congestion(
                    timing_weight,
                    congestion_weight,
                    clock_period,
                    grid_size,
                )
            } else {
                fp.get_cost()
            };

            d_cost = cost - pre_cost;
            let p = (-d_cost / t).exp();

            if d_cost <= 0.0 || rand_01() < p {
                fp.keep_sol();
                pre_cost = cost;
                if d_cost > 0.0 {
                    uphill += 1;
                }

                if cost < best {
                    fp.keep_best();
                    best = cost;
                    println!(
                        "   ==>  Cost= {}, Area= {}, Wire= {}",
                        best,
                        fp.get_area(),
                        fp.get_wire_length()
                    );
                    let elapsed = start_time.elapsed().as_secs_f64();
                    writeln!(
                        frsa,
                        "{:.6} {:.6} {:.6} {:.6}",
                        best,
                        fp.get_area(),
                        fp.get_wire_length(),
                        elapsed
                    )
                    .unwrap();
                    assert!(fp.get_area() >= fp.get_total_area());
                }
            } else {
                reject += 1;
                fp.recover();
            }
            mt += 1;
        }

        t = tratio * t;
        reject_rate = reject as f64 / mt as f64;
        println!("  T= {:.2}, reject= {:.2}\n", t, reject_rate);

        if reject_rate >= conv_rate || t <= term_t {
            break;
        }
    }

    if reject_rate >= conv_rate {
        println!("\n  Convergent!\n");
    } else if t <= term_t {
        println!("\n  Cooling Enough!\n");
    }

    fp.recover_best();
    fp.packing();
}
