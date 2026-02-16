use crate::congestion::{analyze_congestion, compute_congestion_cost};
use crate::fp::*;
use crate::timing::{compute_net_wirelength, TimingAnalyzer};
use std::collections::VecDeque;
use std::io::BufRead;

pub const NIL: i32 = -1;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub parent: i32,
    pub left: i32,
    pub right: i32,
    pub m_id: usize,
    pub rotate: bool,
    pub flip: bool,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.left == NIL && self.right == NIL
    }
}

#[derive(Clone, Debug)]
pub struct Contour {
    pub front: i32,
    pub back: i32,
}

#[derive(Clone, Debug)]
struct Solution {
    nodes_root: usize,
    nodes: Vec<Node>,
    cost: f64,
}

impl Solution {
    fn new() -> Self {
        Solution {
            nodes_root: 0,
            nodes: Vec::new(),
            cost: 1.0,
        }
    }

    fn clear(&mut self) {
        self.cost = 1.0;
        self.nodes.clear();
    }
}

#[derive(Clone)]
pub struct BTree {
    // FPlan fields
    pub area: f64,
    pub width: f64,
    pub height: f64,
    pub total_area: f64,
    pub wire_length: f64,
    pub num_modules: usize,
    pub num_terminals: usize,
    pub num_nets: usize,
    pub num_pins: usize,
    pub modules: Vec<Module>,
    pub terminals: Vec<Module>,
    pub network: Nets,
    pub norm_area: f64,
    pub norm_wl: f64,
    pub cost_alpha: f64,
    pub cost_beta: f64,
    pub terminal_array: Vec<(usize, usize)>,
    filename_short: String,

    // BTree specific fields
    contour_root: i32,
    contour: Vec<Contour>,
    nodes_root: usize,
    nodes: Vec<Node>,
    best_sol: Solution,
    last_sol: Solution,
    changed_root: usize,
}

impl BTree {
    pub fn new(alpha: f64, beta: f64) -> Self {
        BTree {
            area: 0.0,
            width: 0.0,
            height: 0.0,
            total_area: 0.0,
            wire_length: 0.0,
            num_modules: 0,
            num_terminals: 0,
            num_nets: 0,
            num_pins: 0,
            modules: Vec::new(),
            terminals: Vec::new(),
            network: Vec::new(),
            norm_area: 1.0,
            norm_wl: 1.0,
            cost_alpha: alpha,
            cost_beta: beta,
            terminal_array: Vec::new(),
            filename_short: String::new(),
            contour_root: NIL,
            contour: Vec::new(),
            nodes_root: 0,
            nodes: Vec::new(),
            best_sol: Solution::new(),
            last_sol: Solution::new(),
            changed_root: 0,
        }
    }

    pub fn read(&mut self, fr: &str) {
        self.filename_short = fr.to_string();
        let filename = format!("./input/{}.blocks", fr);
        let file = std::fs::File::open(&filename).expect("Failed to open blocks file");
        let reader = std::io::BufReader::new(file);
        let mut lines = reader.lines();

        let line1 = lines.next().unwrap().unwrap();
        let parts1: Vec<&str> = line1.split(':').collect();
        let _num_softblock: usize = parts1[1].trim().parse().unwrap();

        let line2 = lines.next().unwrap().unwrap();
        let parts2: Vec<&str> = line2.split(':').collect();
        self.num_modules = parts2[1].trim().parse().unwrap();

        let line3 = lines.next().unwrap().unwrap();
        let parts3: Vec<&str> = line3.split(':').collect();
        self.num_terminals = parts3[1].trim().parse().unwrap();

        // Skip empty line
        lines.next();

        for i in 0..self.num_modules {
            let line = lines.next().unwrap().unwrap();
            let parts: Vec<&str> = line.split_whitespace().collect();
            let blockname = parts[0].to_string();
            // Format: name hardrectilinear 4 (x1, y1) (x2, y2) (x3, y3) (x4, y4)
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
            let area = width * height;

            let module = Module {
                id: i,
                name: blockname,
                width,
                height,
                x: x1,
                y: y1,
                rx: x1 + width,
                ry: y1 + height,
                area,
                degree: 0,
                is_terminal: false,
                child: Vec::new(),
            };
            self.modules.push(module);
        }

        for i in 0..self.num_terminals {
            // Skip any empty lines before terminals
            let line = loop {
                if let Some(Ok(l)) = lines.next() {
                    if !l.trim().is_empty() {
                        break l;
                    }
                } else {
                    panic!("Expected terminal line but got None");
                }
            };
            let parts: Vec<&str> = line.split_whitespace().collect();
            let terminalname = parts[0].to_string();

            let terminal = Module {
                id: i,
                name: terminalname,
                width: 0.0,
                height: 0.0,
                x: (i * 2) as f64,
                y: 0.0,
                rx: (i * 2) as f64,
                ry: 0.0,
                area: 0.0,
                degree: 0,
                is_terminal: true,
                child: Vec::new(),
            };
            self.terminals.push(terminal);
        }

        let filename = format!("./input/{}.nets", fr);
        let file = std::fs::File::open(&filename).expect("Failed to open nets file");
        let reader = std::io::BufReader::new(file);
        let mut lines = reader.lines();

        let line1 = lines.next().unwrap().unwrap();
        let parts1: Vec<&str> = line1.split(':').collect();
        self.num_nets = parts1[1].trim().parse().unwrap();

        let line2 = lines.next().unwrap().unwrap();
        let parts2: Vec<&str> = line2.split(':').collect();
        self.num_pins = parts2[1].trim().parse().unwrap();

        for _k in 0..self.num_nets {
            // Read "NetDegree : N"
            let line = lines.next().unwrap().unwrap();
            let parts: Vec<&str> = line.split(':').collect();
            let pins_in_net: usize = parts[1].trim().parse().unwrap();

            let mut net: Net = Vec::new();
            for _i in 0..pins_in_net {
                let line = lines.next().unwrap().unwrap();
                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts[0];

                let mut found = false;
                for j in 0..self.num_modules {
                    if name == self.modules[j].name {
                        net.push(j);
                        found = true;
                        break;
                    }
                }
                if !found {
                    for j in 0..self.num_terminals {
                        if name == self.terminals[j].name {
                            net.push(self.num_modules + j);
                            break;
                        }
                    }
                }
            }
            self.network.push(net);
        }

        for i in 0..self.num_nets {
            for j in 0..self.network[i].len() {
                let idx = self.network[i][j];
                if idx >= self.num_modules {
                    let term_idx = idx - self.num_modules;
                    for k in 0..self.network[i].len() {
                        let mod_idx = self.network[i][k];
                        if mod_idx < self.num_modules {
                            self.terminals[term_idx].child.push(mod_idx);
                        }
                    }
                }
            }
        }

        for i in 0..self.num_terminals {
            self.terminals[i].degree = self.terminals[i].child.len();
            self.terminal_array.push((i, self.terminals[i].degree));
        }
        self.terminal_array.sort_by(|a, b| b.1.cmp(&a.1));
    }

    pub fn init(&mut self) {
        self.contour.resize(
            self.num_modules,
            Contour {
                front: NIL,
                back: NIL,
            },
        );
        self.nodes.resize(
            self.num_modules,
            Node {
                id: 0,
                parent: NIL,
                left: NIL,
                right: NIL,
                m_id: 0,
                rotate: false,
                flip: false,
            },
        );

        self.nodes_root = 0;

        for i in 0..self.num_modules {
            self.nodes[i].id = i;
            self.nodes[i].m_id = self.modules[i].id;
            if i == 0 {
                self.nodes[i].parent = NIL;
            } else {
                self.nodes[i].parent = ((i + 1) / 2 - 1) as i32;
            }
            if (2 * i + 1) < self.num_modules {
                self.nodes[i].left = (2 * i + 1) as i32;
            } else {
                self.nodes[i].left = NIL;
            }
            if (2 * i + 2) < self.num_modules {
                self.nodes[i].right = (2 * i + 2) as i32;
            } else {
                self.nodes[i].right = NIL;
            }
        }

        // Calculate total area (sum of all module areas)
        self.total_area = self.modules.iter().map(|m| m.area).sum();

        self.best_sol.clear();
        self.last_sol.clear();
        self.clear();
    }

    fn clear(&mut self) {
        self.contour_root = NIL;
        self.area = 0.0;
        self.wire_length = 0.0;
    }

    pub fn packing(&mut self) {
        self.clear();

        let mut stack = VecDeque::new();
        let p = self.nodes_root;
        self.place_module(p, NIL as usize, true);

        let n = &self.nodes[p];
        if n.right != NIL {
            stack.push_back(n.right as usize);
        }
        if n.left != NIL {
            stack.push_back(n.left as usize);
        }

        while let Some(p) = stack.pop_back() {
            let n_parent = self.nodes[p].parent;
            let n_right = self.nodes[p].right;
            let n_left = self.nodes[p].left;

            assert!(n_parent != NIL);
            let is_left = self.nodes[n_parent as usize].left == p as i32;
            self.place_module(p, n_parent as usize, is_left);

            if n_right != NIL {
                stack.push_back(n_right as usize);
            }
            if n_left != NIL {
                stack.push_back(n_left as usize);
            }
        }

        let mut max_x: f64 = -1.0;
        let mut max_y: f64 = -1.0;
        let mut p = self.contour_root;
        while p != NIL {
            max_x = max_x.max(self.modules[p as usize].rx);
            max_y = max_y.max(self.modules[p as usize].ry);
            p = self.contour[p as usize].front;
        }

        self.width = max_x;
        self.height = max_y;
        self.area = self.height * self.width;
        self.place_terminal();
        self.calc_wire_length();
    }

    fn place_module(&mut self, node_id: usize, abut: usize, is_left: bool) {
        let mod_id = self.nodes[node_id].m_id;
        let mod_abut = if abut == NIL as usize {
            NIL as usize
        } else {
            self.nodes[abut].m_id
        };
        let w = self.modules[mod_id].width;
        let h = self.modules[mod_id].height;

        if abut == NIL as usize {
            self.contour_root = mod_id as i32;
            self.contour[mod_id].back = NIL;
            self.contour[mod_id].front = NIL;
            self.modules[mod_id].x = 0.0;
            self.modules[mod_id].y = 0.0;
            self.modules[mod_id].rx = self.modules[mod_id].x + w;
            self.modules[mod_id].ry = self.modules[mod_id].y + h;
            return;
        }

        let mut p: i32;

        if is_left {
            let abut_width = self.modules[mod_abut].width;
            self.modules[mod_id].x = self.modules[mod_abut].x + abut_width;
            self.modules[mod_id].rx = self.modules[mod_id].x + w;
            p = self.contour[mod_abut].front;

            self.contour[mod_abut].front = mod_id as i32;
            self.contour[mod_id].back = mod_abut as i32;

            if p == NIL {
                self.modules[mod_id].y = 0.0;
                self.modules[mod_id].ry = h;
                self.contour[mod_id].front = NIL;
                return;
            }
        } else {
            self.modules[mod_id].x = self.modules[mod_abut].x;
            self.modules[mod_id].rx = self.modules[mod_id].x + w;
            p = mod_abut as i32;

            let n = self.contour[mod_abut].back;

            if n == NIL {
                self.contour_root = mod_id as i32;
                self.contour[mod_id].back = NIL;
            } else {
                self.contour[n as usize].front = mod_id as i32;
                self.contour[mod_id].back = n;
            }
        }

        let mut min_y = i32::MIN as f64;
        assert!(p != NIL);

        while p != NIL {
            let bx = self.modules[p as usize].rx;
            let by = self.modules[p as usize].ry;
            min_y = min_y.max(by);

            if bx >= self.modules[mod_id].rx {
                self.modules[mod_id].y = min_y;
                self.modules[mod_id].ry = self.modules[mod_id].y + h;
                if bx > self.modules[mod_id].rx {
                    self.contour[mod_id].front = p;
                    self.contour[p as usize].back = mod_id as i32;
                } else {
                    let n = self.contour[p as usize].front;
                    self.contour[mod_id].front = n;
                    if n != NIL {
                        self.contour[n as usize].back = mod_id as i32;
                    }
                }
                break;
            }
            p = self.contour[p as usize].front;
        }

        if p == NIL {
            self.modules[mod_id].y = if min_y == i32::MIN as f64 { 0.0 } else { min_y };
            self.modules[mod_id].ry = self.modules[mod_id].y + h;
            self.contour[mod_id].front = NIL;
        }
    }

    pub fn perturb(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(0..self.num_modules);
        let swap_rate = 0.5;

        if swap_rate > rand_01() {
            let mut temp = 0;
            let mut p;
            loop {
                p = rng.gen_range(0..self.num_modules);
                temp += 1;
                if temp >= 100
                    || (n != p
                        && self.nodes[n].parent != p as i32
                        && self.nodes[p].parent != n as i32)
                {
                    break;
                }
            }
            if temp < 100 {
                self.swap_node(n, p);
            }
        } else {
            let mut p;
            loop {
                p = rng.gen_range(0..self.num_modules);
                if n != p {
                    break;
                }
            }
            self.delete_node(n);
            self.insert_node(p, n);
        }
    }

    fn swap_node(&mut self, n1_idx: usize, n2_idx: usize) {
        let n1_left = self.nodes[n1_idx].left;
        let n1_right = self.nodes[n1_idx].right;
        let n2_left = self.nodes[n2_idx].left;
        let n2_right = self.nodes[n2_idx].right;

        if n1_left != NIL {
            self.nodes[n1_left as usize].parent = n2_idx as i32;
        }
        if n1_right != NIL {
            self.nodes[n1_right as usize].parent = n2_idx as i32;
        }
        if n2_left != NIL {
            self.nodes[n2_left as usize].parent = n1_idx as i32;
        }
        if n2_right != NIL {
            self.nodes[n2_right as usize].parent = n1_idx as i32;
        }

        if self.nodes[n1_idx].parent != NIL {
            let parent_idx = self.nodes[n1_idx].parent as usize;
            if self.nodes[parent_idx].left == n1_idx as i32 {
                self.nodes[parent_idx].left = n2_idx as i32;
            } else {
                self.nodes[parent_idx].right = n2_idx as i32;
            }
        } else {
            self.changed_root = n1_idx;
            self.nodes_root = n2_idx;
        }

        if self.nodes[n2_idx].parent != NIL {
            let parent_idx = self.nodes[n2_idx].parent as usize;
            if self.nodes[parent_idx].left == n2_idx as i32 {
                self.nodes[parent_idx].left = n1_idx as i32;
            } else {
                self.nodes[parent_idx].right = n1_idx as i32;
            }
        } else {
            self.nodes_root = n1_idx;
        }

        // Use split_at_mut to avoid multiple mutable borrows
        if n1_idx < n2_idx {
            let (left, right) = self.nodes.split_at_mut(n2_idx);
            std::mem::swap(&mut left[n1_idx].left, &mut right[0].left);
            std::mem::swap(&mut left[n1_idx].right, &mut right[0].right);
            std::mem::swap(&mut left[n1_idx].parent, &mut right[0].parent);
        } else {
            let (left, right) = self.nodes.split_at_mut(n1_idx);
            std::mem::swap(&mut left[n2_idx].left, &mut right[0].left);
            std::mem::swap(&mut left[n2_idx].right, &mut right[0].right);
            std::mem::swap(&mut left[n2_idx].parent, &mut right[0].parent);
        }
    }

    fn insert_node(&mut self, parent_idx: usize, node_idx: usize) {
        self.nodes[node_idx].parent = parent_idx as i32;
        let edge = rand_bool();

        if edge {
            let parent_left = self.nodes[parent_idx].left;
            self.nodes[node_idx].left = parent_left;
            self.nodes[node_idx].right = NIL;
            if parent_left != NIL {
                self.nodes[parent_left as usize].parent = node_idx as i32;
            }
            self.nodes[parent_idx].left = node_idx as i32;
        } else {
            let parent_right = self.nodes[parent_idx].right;
            self.nodes[node_idx].left = NIL;
            self.nodes[node_idx].right = parent_right;
            if parent_right != NIL {
                self.nodes[parent_right as usize].parent = node_idx as i32;
            }
            self.nodes[parent_idx].right = node_idx as i32;
        }
    }

    fn delete_node(&mut self, node_idx: usize) {
        let mut child = NIL as usize;
        let mut subchild = NIL as usize;
        let mut subparent = NIL as usize;

        if !self.nodes[node_idx].is_leaf() {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut left = rng.gen_bool(0.5);
            if self.nodes[node_idx].left == NIL {
                left = false;
            }
            if self.nodes[node_idx].right == NIL {
                left = true;
            }

            if left {
                child = self.nodes[node_idx].left as usize;
                let node_right = self.nodes[node_idx].right;
                if node_right != NIL {
                    subchild = if self.nodes[child].right != NIL {
                        self.nodes[child].right as usize
                    } else {
                        NIL as usize
                    };
                    subparent = node_right as usize;
                    self.nodes[node_right as usize].parent = child as i32;
                    self.nodes[child].right = node_right;
                }
            } else {
                child = self.nodes[node_idx].right as usize;
                let node_left = self.nodes[node_idx].left;
                if node_left != NIL {
                    subchild = if self.nodes[child].left != NIL {
                        self.nodes[child].left as usize
                    } else {
                        NIL as usize
                    };
                    subparent = node_left as usize;
                    self.nodes[node_left as usize].parent = child as i32;
                    self.nodes[child].left = node_left;
                }
            }
            self.nodes[child].parent = self.nodes[node_idx].parent;
        }

        if self.nodes[node_idx].parent == NIL {
            self.nodes_root = child;
        } else {
            let parent_idx = self.nodes[node_idx].parent as usize;
            if self.nodes[parent_idx].left == node_idx as i32 {
                self.nodes[parent_idx].left = child as i32;
            } else {
                self.nodes[parent_idx].right = child as i32;
            }
        }

        if subchild != NIL as usize && subparent != NIL as usize {
            let mut current = subparent;
            loop {
                if self.nodes[current].left == NIL || self.nodes[current].right == NIL {
                    self.nodes[subchild].parent = current as i32;
                    if self.nodes[current].left == NIL {
                        self.nodes[current].left = subchild as i32;
                    } else {
                        self.nodes[current].right = subchild as i32;
                    }
                    break;
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    current = if rng.gen_bool(0.5) {
                        self.nodes[current].left as usize
                    } else {
                        self.nodes[current].right as usize
                    };
                }
            }
        }
    }

    pub fn keep_sol(&mut self) {
        self.last_sol.nodes_root = self.nodes_root;
        self.last_sol.nodes = self.nodes.clone();
        self.last_sol.cost = self.get_cost();
    }

    pub fn keep_best(&mut self) {
        self.best_sol.nodes_root = self.nodes_root;
        self.best_sol.nodes = self.nodes.clone();
        self.best_sol.cost = self.get_cost();
    }

    pub fn recover(&mut self) {
        self.nodes_root = self.last_sol.nodes_root;
        self.nodes = self.last_sol.nodes.clone();
    }

    pub fn recover_best(&mut self) {
        self.nodes_root = self.best_sol.nodes_root;
        self.nodes = self.best_sol.nodes.clone();
    }

    pub fn get_cost(&self) -> f64 {
        self.cost_alpha * self.area + self.cost_beta * self.wire_length
    }

    // Computes cost with optional timing and congestion components
    pub fn get_cost_with_timing_congestion(
        &self,
        timing_weight: f64,
        congestion_weight: f64,
        clock_period: f64,
        grid_size: usize,
    ) -> f64 {
        let mut base_cost = self.cost_alpha * self.area + self.cost_beta * self.wire_length;

        // Add timing cost if weight is non-zero
        if timing_weight > 0.0 {
            let mut timing_analyzer =
                TimingAnalyzer::new(self.num_modules, self.num_nets, clock_period);

            // Compute net delays based on wirelengths
            for (net_idx, net) in self.network.iter().enumerate() {
                let wl =
                    compute_net_wirelength(net, &self.modules, &self.terminals, self.num_modules);
                timing_analyzer.compute_net_delay(net_idx, wl);
            }

            timing_analyzer.analyze_timing(&self.modules, &self.network, self.num_modules);
            let timing_cost = timing_analyzer.compute_timing_cost(self.num_modules);
            base_cost += timing_weight * timing_cost;
        }

        // Add congestion cost if weight is non-zero
        if congestion_weight > 0.0 {
            let congestion_map = analyze_congestion(
                &self.modules,
                &self.terminals,
                &self.network,
                self.num_modules,
                self.width,
                self.height,
                grid_size,
            );
            let congestion_cost = compute_congestion_cost(&congestion_map);
            base_cost += congestion_weight * congestion_cost;
        }

        base_cost
    }

    pub fn get_area(&self) -> f64 {
        self.area
    }

    pub fn get_wire_length(&self) -> f64 {
        self.wire_length
    }

    pub fn get_width(&self) -> f64 {
        self.width
    }

    pub fn get_height(&self) -> f64 {
        self.height
    }

    pub fn size(&self) -> usize {
        self.num_modules
    }

    pub fn get_total_area(&self) -> f64 {
        self.total_area
    }

    fn calc_wire_length(&mut self) -> f64 {
        self.wire_length = 0.0;

        for net in &self.network {
            if net.is_empty() {
                continue;
            }

            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;

            for &idx in net {
                let (x, y, width, height) = if idx < self.num_modules {
                    (
                        self.modules[idx].x,
                        self.modules[idx].y,
                        self.modules[idx].width,
                        self.modules[idx].height,
                    )
                } else {
                    let term_idx = idx - self.num_modules;
                    (
                        self.terminals[term_idx].x,
                        self.terminals[term_idx].y,
                        self.terminals[term_idx].width,
                        self.terminals[term_idx].height,
                    )
                };

                let center_x = x + width / 2.0;
                let center_y = y + height / 2.0;

                max_x = max_x.max(center_x);
                max_y = max_y.max(center_y);
                min_x = min_x.min(center_x);
                min_y = min_y.min(center_y);
            }

            self.wire_length += (max_x - min_x) + (max_y - min_y);
        }

        self.wire_length
    }

    fn place_terminal(&mut self) {
        for i in 0..self.num_terminals {
            let t_id = self.terminal_array[i].0;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;

            for &v in &self.terminals[t_id].child {
                max_x = max_x.max(self.modules[v].x + self.modules[v].width / 2.0);
                max_y = max_y.max(self.modules[v].y + self.modules[v].height / 2.0);
                min_x = min_x.min(self.modules[v].x + self.modules[v].width / 2.0);
                min_y = min_y.min(self.modules[v].y + self.modules[v].height / 2.0);
            }

            let mid_x = (max_x + min_x) / 2.0;
            let mid_y = (max_y + min_y) / 2.0;
            let d1 = min_x;
            let d2 = self.width - max_x;
            let d3 = min_y;
            let d4 = self.height - max_y;

            if d1 <= d2 && d1 <= d3 && d1 <= d4 {
                self.terminals[t_id].x = 0.0;
                self.terminals[t_id].rx = 0.0;
                self.terminals[t_id].y = mid_y;
                self.terminals[t_id].ry = mid_y;
            } else if d2 <= d1 && d2 <= d3 && d2 <= d4 {
                self.terminals[t_id].x = self.width;
                self.terminals[t_id].rx = self.width;
                self.terminals[t_id].y = mid_y;
                self.terminals[t_id].ry = mid_y;
            } else if d3 <= d1 && d3 <= d2 && d3 <= d4 {
                self.terminals[t_id].x = mid_x;
                self.terminals[t_id].rx = mid_x;
                self.terminals[t_id].y = 0.0;
                self.terminals[t_id].ry = 0.0;
            } else {
                self.terminals[t_id].x = mid_x;
                self.terminals[t_id].rx = mid_x;
                self.terminals[t_id].y = self.height;
                self.terminals[t_id].ry = self.height;
            }
        }

        for i in 0..self.num_terminals {
            while self.terminal_violate(self.terminals[i].x, self.terminals[i].y, i) {
                if self.terminals[i].y == 0.0 && self.terminals[i].x > 0.0 {
                    if self.terminals[i].x - 1.0 < 0.0 {
                        self.terminals[i].x = 0.0;
                        self.terminals[i].rx = 0.0;
                    } else {
                        self.terminals[i].x -= 1.0;
                        self.terminals[i].rx -= 1.0;
                    }
                } else if self.terminals[i].x == 0.0 && self.terminals[i].y < self.height {
                    if self.terminals[i].y + 1.0 > self.height {
                        self.terminals[i].y = self.height;
                        self.terminals[i].ry = self.height;
                    } else {
                        self.terminals[i].y += 1.0;
                        self.terminals[i].ry += 1.0;
                    }
                } else if self.terminals[i].y == self.height && self.terminals[i].x < self.width {
                    if self.terminals[i].x + 1.0 > self.width {
                        self.terminals[i].x = self.width;
                        self.terminals[i].rx = self.width;
                    } else {
                        self.terminals[i].x += 1.0;
                        self.terminals[i].rx += 1.0;
                    }
                } else if self.terminals[i].x == self.width && self.terminals[i].y > 0.0 {
                    if self.terminals[i].y - 1.0 < 0.0 {
                        self.terminals[i].y = 0.0;
                        self.terminals[i].ry = 0.0;
                    } else {
                        self.terminals[i].y -= 1.0;
                        self.terminals[i].ry -= 1.0;
                    }
                }
            }
        }
    }

    fn terminal_violate(&self, mid_x: f64, mid_y: f64, id: usize) -> bool {
        for i in 0..self.num_terminals {
            if i == id {
                continue;
            }
            if (mid_x - self.terminals[i].x).abs() + (mid_y - self.terminals[i].y).abs() < 2.0 {
                return true;
            }
        }
        false
    }

    pub fn print_result(&self) {
        let filename = format!("./out/{}.result", self.filename_short);
        let mut frresult = std::fs::File::create(&filename).expect("Failed to create result file");

        let filename_pl = format!("./out/{}.pl", self.filename_short);
        let mut frplace =
            std::fs::File::create(&filename_pl).expect("Failed to create placement file");

        println!("{}:", self.filename_short);
        println!("#modules = {}", self.num_modules);
        println!("#nets = {}", self.num_nets);
        println!("Total wirelength = {}", self.wire_length);
        println!("Total area = {}", self.area);
        println!("Height = {}, Width = {}", self.height, self.width);

        use std::io::Write;
        writeln!(frresult, "{}:", self.filename_short).unwrap();
        writeln!(frresult, "#modules = {}", self.num_modules).unwrap();
        writeln!(frresult, "#nets = {}", self.num_nets).unwrap();
        writeln!(frresult, "Total wirelength = {}", self.wire_length).unwrap();
        writeln!(frresult, "Total area = {}", self.area).unwrap();
        writeln!(frresult, "Height = {}, Width = {}", self.height, self.width).unwrap();

        for module in &self.modules {
            writeln!(frplace, "{} {} {}", module.name, module.x, module.y).unwrap();
        }
        writeln!(frplace).unwrap();
        for terminal in &self.terminals {
            writeln!(frplace, "{} {} {}", terminal.name, terminal.x, terminal.y).unwrap();
        }
    }
}
