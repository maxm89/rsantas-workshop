use crate::santa;
use crate::santa::Move;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

pub struct SA {
    families: santa::FamilyData,
    families_per_day: Vec<Vec<usize>>,
    all_fams: Vec<usize>,
    init_temperature: f32,
    temperature: f32,
    pub tabu: TabuList,
    maxiter: usize,
}

impl SA {
    pub fn new(families: santa::FamilyData, init_temperature: f32, maxiter: usize) -> Self {
        let mut s = Self {
            families,
            families_per_day: vec![Vec::new(); 101],
            all_fams: Vec::new(),
            init_temperature,
            temperature: init_temperature,
            tabu: TabuList::new(1),
            maxiter,
        };
        for i in 0..5000 {
            s.all_fams.push(i);
        }
        s
    }

    pub fn optimize(&mut self, sol: santa::Solution) -> santa::Solution {
        self.init_families_per_day(&sol);
        let mut bestsol = sol.clone();
        let mut cursol = sol;
        let mut no_improve_cnt = 0;
        for i in 0..self.maxiter {
            self.improve(&mut cursol, 10);
            if cursol.costs < bestsol.costs {
                bestsol = cursol.clone();
                santa::write_solution(&bestsol, "./data/output/");
                no_improve_cnt = 0;
            } else {
                no_improve_cnt += 1;
                if no_improve_cnt >= 20 {
                    //println!("reset");
                    self.temperature = self.init_temperature;
                    cursol = bestsol.clone();
                    self.perturbate(&mut cursol);
                    no_improve_cnt = 0;
                }
            }
            if i % 100 == 0 {
                println!("---> c: {}", bestsol.costs);
            }
        }
        return bestsol;
    }

    pub fn set_tabu(&mut self, tabu: TabuList) {
        self.tabu = tabu
    }

    fn improve(&mut self, sol: &mut santa::Solution, tries_per_fam: usize) -> bool {
        self.all_fams.shuffle(&mut rand::thread_rng());
        let c_before = sol.costs;
        for i in 0..5000 {
            let x = self.all_fams[i];
            for _try in 0..tries_per_fam {
                if self.find_move(sol, x as u32) {
                    break;
                }
            }
        }
        c_before != sol.costs
    }

    /// Searches for moves (Ai-Bj..) -> (Aj-Bk..) -> .. that improve the given solution.
    fn find_move(&mut self, sol: &mut santa::Solution, x: u32) -> bool {
        let mut m = santa::Move::new();
        m.candidates.push(x);
        let mut ind: usize = 0;
        let maxlen = 3;
        loop {
            let x = m.candidates[ind];
            let xi = sol.x[x as usize];
            m.old_days.push(xi);
            let xj = self.pick_alternative(x, xi);
            m.new_days.push(xj);
            if sol.move_feasible(&self.families, &m) {
                let delta = sol.score_move(&self.families, &m);
                if delta > 0.0 || (ind > 1 && self.accept(delta)) {
                    let new_occs = sol.new_occs(&self.families, &m);
                    if self.tabu.already_visited(&new_occs) {
                        return false;
                    }
                    sol.apply_move(&self.families, &m);
                    self.tabu.add(&sol.occupancies);
                    self.init_families_per_day(sol);
                    if delta <= 0.0 {
                        self.temperature *= 0.9;
                    }
                    return true;
                }
            }

            if ind + 1 >= maxlen {
                break;
            } else {
                m.candidates.push(self.pick_from_day(xj));
            }

            ind += 1;
        }
        return false;
    }

    fn accept(&self, mut delta: f32) -> bool {
        let r: f32 = rand::thread_rng().gen();
        delta /= self.temperature;
        let prob = delta.exp();
        if r < prob {
            //println!("delta: {}", delta);
            return true;
        } else {
            return false;
        }
    }

    fn perturbate(&mut self, sol: &mut santa::Solution) -> santa::Solution {
        let mut rng = rand::thread_rng();
        let mod_weight = rng.gen_range(0.2, 2.2);
        println!("perturbate with {}", mod_weight);
        self.families.set_weight(mod_weight);
        let mut newsol = self.localsearch(sol.clone(), 2, 10);
        self.families.set_weight(1.0);
        newsol.costs = self.families.score(&newsol);
        newsol
    }

    fn localsearch(
        &mut self,
        mut sol: santa::Solution,
        break_after: usize,
        tries_per_fam: usize,
    ) -> santa::Solution {
        self.init_families_per_day(&sol);
        let mut count_unchanged = 0;
        loop {
            let changed = self.improve(&mut sol, tries_per_fam);
            count_unchanged = if changed { 0 } else { count_unchanged + 1 };
            if count_unchanged >= break_after {
                break;
            }
        }
        sol
    }

    fn perturbate2(&mut self, sol: &mut santa::Solution) {
        // perturbate such that at least one type of costs improve!
        let mut rng = rand::thread_rng();
        loop {
            let fama = rng.gen_range(0, 5000);
            let famb = rng.gen_range(0, 5000);
            let mut m = santa::Move::new();
            m.candidates.push(fama as u32);
            m.old_days.push(sol.x[fama]);
            m.new_days.push(sol.x[famb]);
            m.candidates.push(famb as u32);
            m.old_days.push(sol.x[famb]);
            m.new_days.push(sol.x[fama]);
            // return if feasible and either costs improve
            if sol.move_feasible(&self.families, &m) {
                let (costs, pcosts, acosts) = sol.score_move_split(&self.families, &m);
                if costs > -200.0 {
                    if pcosts > 0.0 || acosts > 0.0 {
                        sol.apply_move(&self.families, &m);
                        self.init_families_per_day(sol);
                        return;
                    }
                }
            }
        }
    }

    fn pick_from_day(&self, xj: u32) -> u32 {
        let mut rng = rand::thread_rng();
        let fnew_ind = rng.gen_range(0, self.families_per_day[xj as usize].len());
        self.families_per_day[xj as usize][fnew_ind] as u32
    }

    // pick alternative from choices or from adjacent days
    fn pick_alternative(&self, x: u32, xi: u32) -> u32 {
        let mut rng = rand::thread_rng();
        loop {
            let xji = rng.gen_range(0, 5);
            let xnew = self.families.choices[x as usize][xji];
            if xnew != xi {
                return xnew;
            }
        }
    }

    fn init_families_per_day(&mut self, sol: &santa::Solution) {
        self.families_per_day = vec![Vec::new(); 101];
        for fam in 0..5000 {
            let day = sol.x[fam] as usize;
            self.families_per_day[day].push(fam);
        }
    }
}

#[derive(Clone)]
pub struct TabuList {
    visited: VecDeque<TabuItem>,
    nmax: usize,
}

impl TabuList {
    pub fn new(nmax: usize) -> Self {
        Self {
            visited: VecDeque::new(),
            nmax,
        }
    }

    // TODO entire solutions and approximate
    fn add(&mut self, item: TabuItem) {
        if !self.already_visited(&item) {
            self.visited.push_front(item);
            //println!("Added to tabu list");
            if self.visited.len() > self.nmax {
                self.visited.pop_back();
            }
        }
    }

    // TODO occupancies archive!!!
    fn already_visited(&self, item: &TabuItem) -> bool {
        // TODO binary search
        let visited = self.visited.iter().any(|x| x == item);
        if visited {
            println!("Been here before");
        }
        visited
    }
}
