// Timing-driven placement module for critical path optimization.
// Provides static timing analysis and timing-aware cost functions.

use crate::fp::{Module, Nets};

// Represents timing information for a module
#[derive(Clone, Debug)]
pub struct TimingInfo {
    pub arrival_time: f64,
    pub required_time: f64,
    pub slack: f64,
}

// Timing analysis engine for computing critical paths
pub struct TimingAnalyzer {
    pub module_timing: Vec<TimingInfo>,
    pub net_delays: Vec<f64>,
    pub clock_period: f64,
    pub unit_delay: f64,
}

impl TimingAnalyzer {
    pub fn new(num_modules: usize, num_nets: usize, clock_period: f64) -> Self {
        TimingAnalyzer {
            module_timing: vec![
                TimingInfo {
                    arrival_time: 0.0,
                    required_time: clock_period,
                    slack: clock_period,
                };
                num_modules
            ],
            net_delays: vec![0.0; num_nets],
            clock_period,
            unit_delay: 0.1,
        }
    }

    // Computes net delay based on wire length (Elmore delay model)
    pub fn compute_net_delay(&mut self, net_idx: usize, wire_length: f64) {
        self.net_delays[net_idx] = self.unit_delay * wire_length;
    }

    /// Performs static timing analysis using topological traversal
    pub fn analyze_timing(
        &mut self,
        modules: &[Module],
        network: &Nets,
        num_modules: usize,
    ) -> f64 {
        // Forward propagation: compute arrival times
        for net_idx in 0..network.len() {
            if network[net_idx].is_empty() {
                continue;
            }

            let mut max_arrival = 0.0;
            let mut driver_idx = None;

            // Find driver (first module in net) and its arrival time
            for &idx in &network[net_idx] {
                if idx < num_modules {
                    if driver_idx.is_none() {
                        driver_idx = Some(idx);
                        max_arrival = self.module_timing[idx].arrival_time;
                    }
                }
            }

            if let Some(driver) = driver_idx {
                let net_delay = self.net_delays[net_idx];

                // Update arrival times for all loads
                for &idx in &network[net_idx] {
                    if idx < num_modules && idx != driver {
                        let new_arrival = max_arrival + net_delay;
                        if new_arrival > self.module_timing[idx].arrival_time {
                            self.module_timing[idx].arrival_time = new_arrival;
                        }
                    }
                }
            }
        }

        // Backward propagation: compute required times
        for module_idx in (0..num_modules).rev() {
            self.module_timing[module_idx].required_time = self.clock_period;

            // Find all nets where this module is a driver
            for (net_idx, net) in network.iter().enumerate() {
                if !net.is_empty() && net[0] == module_idx {
                    let net_delay = self.net_delays[net_idx];

                    for &load_idx in net.iter().skip(1) {
                        if load_idx < num_modules {
                            let required = self.module_timing[load_idx].required_time - net_delay;
                            if required < self.module_timing[module_idx].required_time {
                                self.module_timing[module_idx].required_time = required;
                            }
                        }
                    }
                }
            }
        }

        // Compute slack for each module
        let mut max_delay = 0.0;
        for module_idx in 0..num_modules {
            self.module_timing[module_idx].slack =
                self.module_timing[module_idx].required_time
                    - self.module_timing[module_idx].arrival_time;

            if self.module_timing[module_idx].arrival_time > max_delay {
                max_delay = self.module_timing[module_idx].arrival_time;
            }
        }

        max_delay
    }

    /// Computes timing cost based on critical path and negative slack
    pub fn compute_timing_cost(&self, num_modules: usize) -> f64 {
        let mut total_negative_slack = 0.0;
        let mut critical_path_length = 0.0;

        for module_idx in 0..num_modules {
            let slack = self.module_timing[module_idx].slack;

            if slack < 0.0 {
                total_negative_slack += slack.abs();
            }

            if self.module_timing[module_idx].arrival_time > critical_path_length {
                critical_path_length = self.module_timing[module_idx].arrival_time;
            }
        }

        // Cost is weighted sum of critical path length and total negative slack
        critical_path_length + 10.0 * total_negative_slack
    }

    /// Returns the critical path delay
    pub fn get_critical_path_delay(&self) -> f64 {
        self.module_timing
            .iter()
            .map(|t| t.arrival_time)
            .fold(0.0, f64::max)
    }

    /// Returns number of timing violations
    pub fn get_num_violations(&self) -> usize {
        self.module_timing
            .iter()
            .filter(|t| t.slack < 0.0)
            .count()
    }
}

// Computes wirelength-based delay for a single net using bounding box method
pub fn compute_net_wirelength(
    net: &[usize],
    modules: &[Module],
    terminals: &[Module],
    num_modules: usize,
) -> f64 {
    if net.is_empty() {
        return 0.0;
    }

    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;

    for &idx in net {
        let (x, y, width, height) = if idx < num_modules {
            (
                modules[idx].x,
                modules[idx].y,
                modules[idx].width,
                modules[idx].height,
            )
        } else {
            let term_idx = idx - num_modules;
            (
                terminals[term_idx].x,
                terminals[term_idx].y,
                terminals[term_idx].width,
                terminals[term_idx].height,
            )
        };

        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;

        max_x = max_x.max(center_x);
        max_y = max_y.max(center_y);
        min_x = min_x.min(center_x);
        min_y = min_y.min(center_y);
    }

    (max_x - min_x) + (max_y - min_y)
}
