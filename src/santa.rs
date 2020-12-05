#![allow(dead_code)]

extern crate csv;
extern crate serde_derive;

use std::fs::File;
use std::path::Path;

#[derive(Clone)]
pub struct FamilyData {
    pub choices: Vec<Vec<u32>>,
    pub sizes: Vec<i32>,
    candidates: Vec<i32>,
    penalty_matrix: Vec<Vec<f32>>,
    weight: f32,
    debug: bool,
}

impl FamilyData {
    pub fn new() -> FamilyData {
        FamilyData {
            choices: Vec::new(),
            sizes: Vec::new(),
            candidates: Vec::new(),
            penalty_matrix: vec![vec![0.0; 101]; 5000],
            weight: 1.0,
            debug: false,
        }
    }
    pub fn score(&self, sol: &Solution) -> f32 {
        let mut costs: f32 = 0.0;

        costs += self.score_penalty(sol);
        costs += self.score_accounting(sol) * self.weight;

        costs
    }

    pub fn split_score(&self, sol: &Solution) -> (f32, f32, f32) {
        let mut costs: f32 = 0.0;

        let pcosts = self.score_penalty(sol);
        let acosts = self.score_accounting(sol) * self.weight;

        (costs, pcosts, acosts)
    }

    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight;
    }

    pub fn peek_swap(&self, solution: &mut Solution, ida: u32, idb: u32) -> f32 {
        let delta_penalty = self.delta_penalty(solution, ida, idb);
        let delta_acc = self.delta_accounting_costs(solution, ida, idb);
        delta_penalty + delta_acc
    }

    fn delta_accounting_costs(&self, sol: &mut Solution, ida: u32, idb: u32) -> f32 {
        let fama = ida as usize;
        let famb = idb as usize;
        let daya = sol.x[fama];
        let dayb = sol.x[famb];
        let old_occa = sol.occupancies[daya as usize];
        let old_occb = sol.occupancies[dayb as usize];
        let new_occa = old_occa - self.sizes[fama] + self.sizes[famb];
        let new_occb = old_occb - self.sizes[famb] + self.sizes[fama];
        //let mut involved days = 
        //let mut delta = self.score_accounting_days(sol);
        let mut delta = self.score_accounting(sol);
        sol.occupancies[daya as usize] = new_occa;
        sol.occupancies[dayb as usize] = new_occb;
        delta -= self.score_accounting(sol);
        sol.occupancies[daya as usize] = old_occa;
        sol.occupancies[dayb as usize] = old_occb;
        delta
    }

    // greater zero means improvement
    fn delta_penalty(&self, sol: &Solution, ida: u32, idb: u32) -> f32 {
        let fama = ida as usize;
        let famb = idb as usize;
        let daya = sol.x[fama];
        let dayb = sol.x[famb];

        let mut delta: f32 = self.penalty(fama, daya) + self.penalty(famb, dayb);
        delta -= self.penalty(fama, dayb) + self.penalty(famb, daya);
        let new_occa = sol.occupancies[daya as usize] - self.sizes[fama] + self.sizes[famb];
        let new_occb = sol.occupancies[dayb as usize] - self.sizes[famb] + self.sizes[fama];
        if new_occa < 125 || new_occa > 300 {
            delta -= 3000000.0;
        }
        if new_occb < 125 || new_occb > 300 {
            delta -= 3000000.0;
        }

        delta
    }

    fn score_accounting(&self, sol: &Solution) -> f32 {
        let mut costs: f32 = 0.0;
        let mut prev = sol.occupancies[100];
        for i in (1..101).rev() {
            if self.debug && (sol.occupancies[i] > 300 || sol.occupancies[i] < 125) {
                println!(
                    "Violated occupancy constraint for day {}: {}",
                    i, sol.occupancies[i]
                );
            }
            let exp = ((sol.occupancies[i] - prev).abs()) as f32 / 50.0 + 0.5;
            let lhs = (sol.occupancies[i] - 125) as f32 / 400.0;
            let rhs = (sol.occupancies[i] as f32).powf(exp);
            costs += lhs * rhs;
            prev = sol.occupancies[i] as i32;
        }
        costs
    }

    fn score_penalty(&self, sol: &Solution) -> f32 {
        let mut costs: f32 = 0.0;

        for (i, e) in sol.x.iter().enumerate() {
            costs += self.penalty(i, *e);
        }
        costs
    }

    pub fn precalc_penalty_matrix(&mut self) {
        for fam in 0..5000 {
            for day in 0..101 {
                self.penalty_matrix[fam][day] = self.calc_penalty(fam, day as u32);
            }
        }
    }

    fn penalty(&self, fam: usize, day: u32) -> f32 {
        self.penalty_matrix[fam][day as usize]
    }

    fn calc_penalty(&self, fam: usize, day: u32) -> f32 {
        let fam_size: u32 = self.sizes[fam] as u32;
        let ret: f32 = if self.choices[fam][0] == day {
            0.0
        } else if self.choices[fam][1] == day {
            50.0
        } else if self.choices[fam][2] == day {
            50.0 + (9 * fam_size) as f32
        } else if self.choices[fam][3] == day {
            100.0 + (9 * fam_size) as f32
        } else if self.choices[fam][4] == day {
            200.0 + (9 * fam_size) as f32
        } else if self.choices[fam][5] == day {
            200.0 + (18 * fam_size) as f32
        } else if self.choices[fam][6] == day {
            300.0 + (18 * fam_size) as f32
        } else if self.choices[fam][7] == day {
            300.0 + (36 * fam_size) as f32
        } else if self.choices[fam][8] == day {
            400.0 + (36 * fam_size) as f32
        } else if self.choices[fam][9] == day {
            500.0 + (36 * fam_size + 199 * fam_size) as f32
        } else {
            500.0 + (36 * fam_size + 398 * fam_size) as f32
        };
        ret
    }
}

#[derive(Clone)]
pub struct Solution {
    pub x: Vec<u32>,
    pub costs: f32,
    pub occupancies: Vec<i32>,
}

impl Solution {
    pub fn new() -> Solution {
        Solution {
            x: Vec::new(),
            costs: 0.0,
            occupancies: vec![0i32; 101],
        }
    }

    pub fn is_feasible(&self) -> bool {
        for i in 1..101 {
            let occ = self.occupancies[i];
            if occ < 125 || occ > 300 {
                return false;
            }
        }
        true
    }

    /// value > 0 indicates improvement
    pub fn score_move(&mut self, families: &FamilyData, m: &Move) -> f32 {
        // TODO delta accounting
        let mut deltap = 0.0;
        let mut deltaa = families.score_accounting(self) * families.weight;
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            deltap += families.penalty(fam, self.x[fam]);
            deltap -= families.penalty(fam, m.new_days[ind]);
            self.occupancies[m.old_days[ind] as usize] -= families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] += families.sizes[fam];
        }
        deltaa -= families.score_accounting(self) * families.weight;
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            self.occupancies[m.old_days[ind] as usize] += families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] -= families.sizes[fam];
        }
        deltaa + deltap
    }

    /// value > 0 indicates improvement
    pub fn score_move_split(&mut self, families: &FamilyData, m: &Move) -> (f32, f32, f32) {
        // TODO delta accounting
        let mut deltap = 0.0;
        let mut deltaa = families.score_accounting(self) * families.weight;
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            deltap += families.penalty(fam, self.x[fam]);
            deltap -= families.penalty(fam, m.new_days[ind]);
            self.occupancies[m.old_days[ind] as usize] -= families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] += families.sizes[fam];
        }
        deltaa -= families.score_accounting(self) * families.weight;
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            self.occupancies[m.old_days[ind] as usize] += families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] -= families.sizes[fam];
        }
        (deltaa + deltap, deltap, deltaa)
    }

    /// returns the new score of self
    pub fn apply_move(&mut self, families: &FamilyData, m: &Move) -> f32 {
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            self.occupancies[m.old_days[ind] as usize] -= families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] += families.sizes[fam];
            self.x[fam] = m.new_days[ind];
        }
        let new_costs = families.score(self);
        self.costs = new_costs;
        new_costs
    }

    pub fn new_occs(&self, families: &FamilyData, m: &Move) -> Vec<i32> {
        let mut occs = self.occupancies.clone();
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            occs[m.old_days[ind] as usize] -= families.sizes[fam];
            occs[m.new_days[ind] as usize] += families.sizes[fam];
        }
        occs
    }

    pub fn move_feasible(&mut self, families: &FamilyData, m: &Move) -> bool {
        let mut feas: bool = true;

        for i in 0..m.candidates.len() {
            for j in 0..m.candidates.len() {
                if i != j && m.candidates[i] == m.candidates[j] {
                    return false;
                }
            }
        }

        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            self.occupancies[m.old_days[ind] as usize] -= families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] += families.sizes[fam];
        }
        for day in 1..101 {
            if self.occupancies[day] < 125 || self.occupancies[day] > 300 {
                feas = false;
                break;
            }
        }
        for ind in 0..m.candidates.len() {
            let fam: usize = m.candidates[ind] as usize;
            self.occupancies[m.old_days[ind] as usize] += families.sizes[fam];
            self.occupancies[m.new_days[ind] as usize] -= families.sizes[fam];
        }
        feas
    }

    pub fn init_occupancies(&mut self, families: &FamilyData) {
        for fam in 0..5000 {
            let day = self.x[fam] as usize;
            self.occupancies[day] += families.sizes[fam];
        }
    }
}

/// A Move describes a local search operator, that exchanges for all components of a
/// solution candidate <candidates> <old_days> by <new_days>
pub struct Move {
    pub candidates: Vec<u32>,
    pub new_days: Vec<u32>,
    pub old_days: Vec<u32>,
}

impl Move {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            new_days: Vec::new(),
            old_days: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
struct ChoicesRecord {
    #[allow(dead_code)]
    family_id: u32,
    choice_0: u32,
    choice_1: u32,
    choice_2: u32,
    choice_3: u32,
    choice_4: u32,
    choice_5: u32,
    choice_6: u32,
    choice_7: u32,
    choice_8: u32,
    choice_9: u32,
    n_people: i32,
}

pub fn read_families(path: &str) -> FamilyData {
    let mut families: FamilyData = FamilyData::new();
    let file = File::open(path);
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Cannot read file!: {}", error),
    };
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: ChoicesRecord = match result {
            Ok(result) => result,
            Err(error) => panic!("Couldnt parse file!: {}", error),
        };
        families.choices.push(vec![
            record.choice_0,
            record.choice_1,
            record.choice_2,
            record.choice_3,
            record.choice_4,
            record.choice_5,
            record.choice_6,
            record.choice_7,
            record.choice_8,
            record.choice_9,
        ]);

        families.sizes.push(record.n_people);
    }
    extend_choices(&mut families);
    families.precalc_penalty_matrix();
    families
}

fn extend_choices(families: &mut FamilyData) {
    for fam in 0..5000 {
        for day in 1..101 {
            match families.choices[fam].iter().find(|&&x| x == day) {
                Some(_) => {}
                None => families.choices[fam].push(day),
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SolutionRecord {
    #[allow(dead_code)]
    family_id: u32,
    assigned_day: u32,
}

pub fn read_score_solution(families: &FamilyData, path: &str) -> Solution {
    let mut solution: Solution = Solution::new();
    let file = File::open(path).expect("Cannot read submission file!");
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: SolutionRecord = result.expect("Couldnt parse solution file!");
        solution.x.push(record.assigned_day);
    }
    solution.init_occupancies(families);
    solution.costs = families.score(&solution);
    solution
}

pub fn write_solution(sol: &Solution, dir: &str) {
    if !Path::new(dir).exists() {
        panic!("Output directory doesn't exist, quit.")
    }
    let path = String::from(dir) + sol.costs.to_string().as_str() + ".csv";
    let mut writer = csv::Writer::from_path(path).expect("Couldnt create writer!");
    for (family_id, &assigned_day) in sol.x.iter().enumerate() {
        let rec = SolutionRecord {
            family_id: family_id as u32,
            assigned_day: assigned_day,
        };
        writer
            .serialize(rec)
            .expect("Could not write line to file!");
    }
    writer.flush().unwrap();
}
