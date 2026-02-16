use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub const INF: i32 = 0x7FFFFFFF;

#[derive(Clone, Debug)]
pub struct Module {
    pub id: usize,
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub rx: f64,
    pub ry: f64,
    pub area: f64,
    pub degree: usize,
    pub is_terminal: bool,
    pub child: Vec<usize>,
}

pub type Net = Vec<usize>; // Store module/terminal indices instead of pointers
pub type Nets = Vec<Net>;

pub trait FloorPlan {
    fn init(&mut self);
    fn packing(&mut self);
    fn perturb(&mut self);
    fn keep_sol(&mut self);
    fn keep_best(&mut self);
    fn recover(&mut self);
    fn recover_best(&mut self);
    fn get_cost(&self) -> f64;
    fn size(&self) -> usize;
    fn get_area(&self) -> f64;
    fn get_total_area(&self) -> f64;
    fn get_wire_length(&self) -> f64;
    fn get_width(&self) -> f64;
    fn get_height(&self) -> f64;
    fn print_result(&self);
    fn normalize_cost(&mut self, t: usize);
}

pub struct FPlan {
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
}

impl FPlan {
    pub fn new(alpha: f64, beta: f64) -> Self {
        FPlan {
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
        }
    }

    pub fn clear(&mut self) {
        self.area = 0.0;
        self.wire_length = 0.0;
    }

    pub fn read(&mut self, fr: &str) {
        self.filename_short = fr.to_string();
        let filename = format!("./input/{}.blocks", fr);
        let file = File::open(&filename).expect("Failed to open blocks file");
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Read header
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

        // Read modules
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

        // Read terminals
        for i in 0..self.num_terminals {
            let line = lines.next().unwrap().unwrap();
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

        // Read nets
        let filename = format!("./input/{}.nets", fr);
        let file = File::open(&filename).expect("Failed to open nets file");
        let reader = BufReader::new(file);
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
                let _b = parts[1];

                // Find in modules
                let mut found = false;
                for j in 0..self.num_modules {
                    if name == self.modules[j].name {
                        net.push(j);
                        found = true;
                        break;
                    }
                }
                // Find in terminals
                if !found {
                    for j in 0..self.num_terminals {
                        if name == self.terminals[j].name {
                            net.push(self.num_modules + j); // Offset by num_modules
                            break;
                        }
                    }
                }
            }
            self.network.push(net);
        }

        // Find terminals' children
        for i in 0..self.num_nets {
            for j in 0..self.network[i].len() {
                let idx = self.network[i][j];
                if idx >= self.num_modules {
                    // It's a terminal
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

        // Sort terminals by degrees
        for i in 0..self.num_terminals {
            self.terminals[i].degree = self.terminals[i].child.len();
            self.terminal_array.push((i, self.terminals[i].degree));
        }
        self.terminal_array.sort_by(|a, b| b.1.cmp(&a.1));
    }

    pub fn calc_wire_length(&mut self) -> f64 {
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

    pub fn print_result(&self) {
        let filename = format!("./out/{}.result", self.filename_short);
        let mut frresult = File::create(&filename).expect("Failed to create result file");

        let filename_pl = format!("./out/{}.pl", self.filename_short);
        let mut frplace = File::create(&filename_pl).expect("Failed to create placement file");

        println!("{}:", self.filename_short);
        println!("#modules = {}", self.num_modules);
        println!("#nets = {}", self.num_nets);
        println!("Total wirelength = {}", self.wire_length);
        println!("Total area = {}", self.area);
        println!("Height = {}, Width = {}", self.height, self.width);

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

pub fn rand_01() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<f64>()
}

pub fn rand_bool() -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.5)
}
