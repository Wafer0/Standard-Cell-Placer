// Congestion analysis module for routing resource management.
// Implements grid-based congestion estimation for placement optimization.

use crate::fp::{Module, Nets};

// Grid-based congestion map
#[derive(Clone, Debug)]
pub struct CongestionMap {
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: f64,
    pub h_demand: Vec<Vec<f64>>,
    pub v_demand: Vec<Vec<f64>>,
    pub h_capacity: Vec<Vec<f64>>,
    pub v_capacity: Vec<Vec<f64>>,
    pub overflow: Vec<Vec<f64>>,
}

impl CongestionMap {
    /// Creates a new congestion map with specified grid dimensions
    pub fn new(floorplan_width: f64, floorplan_height: f64, grid_size: usize) -> Self {
        let grid_width = grid_size;
        let grid_height = grid_size;
        let cell_size = floorplan_width.max(floorplan_height) / grid_size as f64;

        // Initialize demand and capacity matrices
        let h_demand = vec![vec![0.0; grid_width]; grid_height];
        let v_demand = vec![vec![0.0; grid_width]; grid_height];
        let h_capacity = vec![vec![10.0; grid_width]; grid_height];
        let v_capacity = vec![vec![10.0; grid_width]; grid_height];
        let overflow = vec![vec![0.0; grid_width]; grid_height];

        CongestionMap {
            grid_width,
            grid_height,
            cell_size,
            h_demand,
            v_demand,
            h_capacity,
            v_capacity,
            overflow,
        }
    }

    /// Maps a coordinate to grid cell index
    fn coord_to_grid(&self, coord: f64) -> usize {
        let idx = (coord / self.cell_size).floor() as usize;
        idx.min(self.grid_width - 1).min(self.grid_height - 1)
    }

    /// Estimates routing demand for a single net using bounding box
    pub fn add_net_demand(
        &mut self,
        net: &[usize],
        modules: &[Module],
        terminals: &[Module],
        num_modules: usize,
    ) {
        if net.len() < 2 {
            return;
        }

        // Compute bounding box
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for &idx in net {
            let (x, y, w, h) = if idx < num_modules {
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

            let cx = x + w / 2.0;
            let cy = y + h / 2.0;

            min_x = min_x.min(cx);
            max_x = max_x.max(cx);
            min_y = min_y.min(cy);
            max_y = max_y.max(cy);
        }

        // Map bounding box to grid cells
        let x1 = self.coord_to_grid(min_x);
        let x2 = self.coord_to_grid(max_x);
        let y1 = self.coord_to_grid(min_y);
        let y2 = self.coord_to_grid(max_y);

        // Add horizontal demand
        for y in y1..=y2 {
            for x in x1..x2 {
                if y < self.grid_height && x < self.grid_width {
                    self.h_demand[y][x] += 1.0;
                }
            }
        }

        // Add vertical demand
        for x in x1..=x2 {
            for y in y1..y2 {
                if y < self.grid_height && x < self.grid_width {
                    self.v_demand[y][x] += 1.0;
                }
            }
        }
    }

    /// Computes congestion overflow for each grid cell
    pub fn compute_overflow(&mut self) {
        for i in 0..self.grid_height {
            for j in 0..self.grid_width {
                let h_overflow = (self.h_demand[i][j] - self.h_capacity[i][j]).max(0.0);
                let v_overflow = (self.v_demand[i][j] - self.v_capacity[i][j]).max(0.0);
                self.overflow[i][j] = h_overflow + v_overflow;
            }
        }
    }

    /// Returns total overflow across all grid cells
    pub fn get_total_overflow(&self) -> f64 {
        self.overflow.iter().flat_map(|row| row.iter()).sum()
    }

    /// Returns maximum overflow in any single cell
    pub fn get_max_overflow(&self) -> f64 {
        self.overflow
            .iter()
            .flat_map(|row| row.iter())
            .fold(0.0, |acc, &x| acc.max(x))
    }

    /// Returns the number of congested cells
    pub fn get_num_congested_cells(&self) -> usize {
        self.overflow
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&x| x > 0.0)
            .count()
    }

    /// Resets all demand values (call before recomputing)
    pub fn reset_demand(&mut self) {
        for i in 0..self.grid_height {
            for j in 0..self.grid_width {
                self.h_demand[i][j] = 0.0;
                self.v_demand[i][j] = 0.0;
                self.overflow[i][j] = 0.0;
            }
        }
    }
}

// Analyzes congestion for the entire floorplan
pub fn analyze_congestion(
    modules: &[Module],
    terminals: &[Module],
    network: &Nets,
    num_modules: usize,
    width: f64,
    height: f64,
    grid_size: usize,
) -> CongestionMap {
    let mut congestion_map = CongestionMap::new(width, height, grid_size);

    // Add demand from each net
    for net in network {
        congestion_map.add_net_demand(net, modules, terminals, num_modules);
    }

    // Compute overflow
    congestion_map.compute_overflow();

    congestion_map
}

// Computes congestion cost for optimization objective
pub fn compute_congestion_cost(congestion_map: &CongestionMap) -> f64 {
    let total_overflow = congestion_map.get_total_overflow();
    let max_overflow = congestion_map.get_max_overflow();

    // Cost combines total overflow and maximum overflow
    total_overflow + 5.0 * max_overflow
}
