use crate::santa;
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Instant;

pub struct ILS {
    families: santa::FamilyData,
    families_per_day: Vec<Vec<usize>>,
    all_fams: Vec<usize>,
    move_depth: usize,
}

impl ILS {
    pub fn new(families: santa::FamilyData, move_depth: usize) -> Self {
        let mut s = Self {
            families,
            families_per_day: vec![Vec::new(); 101],
            all_fams: Vec::new(),
            move_depth,
        };
        for i in 0..5000 {
            s.all_fams.push(i);
        }
        s
    }

    pub fn optimize(&mut self, sol: santa::Solution, maxiter: usize) -> santa::Solution {
        let t = Instant::now();
        let mut bestsol = sol;
        let mut bestcosts = bestsol.costs; // TODO dont need? bestsol has costs field
        let oldcosts = bestcosts;
        let mut cursol = bestsol.clone();
        for _ in 0..maxiter {
            let sol = self.localsearch(cursol.clone(), 2, 40);
            if sol.costs < bestcosts {
                bestsol = sol;
                bestcosts = bestsol.costs;
                cursol = bestsol.clone();
            } else {
                cursol = bestsol.clone();
            }
            cursol = self.perturbate(&mut cursol);
        }
        println!(
            "[][] {} -> {}, {}s",
            oldcosts,
            bestcosts,
            t.elapsed().as_secs()
        );
        bestsol
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

    /// Seaches for moves (Ai-Bj..) -> (Aj-Bk..) -> .. that improve the given solution.
    fn find_move(&mut self, sol: &mut santa::Solution, x: u32) -> bool {
        let mut m = santa::Move::new();
        m.candidates.push(x);
        let mut ind: usize = 0;
        let maxlen = self.move_depth;
        loop {
            let x = m.candidates[ind];
            let xi = sol.x[x as usize];
            m.old_days.push(xi);
            let xj = self.pick_alternative(x, xi);
            m.new_days.push(xj);
            if sol.move_feasible(&self.families, &m) && sol.score_move(&self.families, &m) > 0.0 {
                sol.apply_move(&self.families, &m);
                self.init_families_per_day(sol);
                return true;
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

    /// change the weights of the objective function and optimize for a few iterations
    /// to escape local optimas
    fn perturbate(&mut self, sol: &mut santa::Solution) -> santa::Solution {
        let mut rng = rand::thread_rng();
        let mod_weight = rng.gen_range(0.2, 2.2);
        self.families.set_weight(mod_weight);
        let mut newsol = self.localsearch(sol.clone(), 2, 25);
        self.families.set_weight(1.0);
        newsol.costs = self.families.score(&newsol);
        newsol
    }

    fn pick_from_day(&self, xj: u32) -> u32 {
        let mut rng = rand::thread_rng();
        let fnew_ind = rng.gen_range(0, self.families_per_day[xj as usize].len());
        self.families_per_day[xj as usize][fnew_ind] as u32
    }

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
