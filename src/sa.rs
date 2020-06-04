use crate::santa;
use rand::seq::SliceRandom;
use rand::Rng;


pub struct SA {
    families: santa::FamilyData,
    families_per_day: Vec<Vec<usize>>,
    all_fams: Vec<usize>,
    init_temperature: f32,
    temperature: f32,
}

impl SA {
    pub fn new(families: santa::FamilyData, init_temperature: f32) -> Self {
        let mut s = Self {
            families,
            families_per_day: vec![Vec::new(); 101],
            all_fams: Vec::new(),
            init_temperature,
            temperature: init_temperature,
        };
        for i in 0..5000 {
            s.all_fams.push(i);
        }
        s
    }

    pub fn optimize(&mut self, sol: santa::Solution) -> santa::Solution {
        println!("c: {}", sol.costs);
        self.init_families_per_day(&sol);
        let mut bestsol = sol.clone();
        let mut cursol = sol;
        for i in 0..100000 {
            self.improve(&mut cursol, 10);
            if cursol.costs < bestsol.costs {
                bestsol = cursol.clone();
            }
            if i % 100 == 0 {
                println!("c: {}", bestsol.costs);
                cursol = bestsol.clone();
            } else if i % 10 == 0 {
                self.temperature += self.init_temperature;
            }
        }
        return bestsol
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
        let maxlen = 5;
        loop {
            let x = m.candidates[ind];
            let xi = sol.x[x as usize];
            m.old_days.push(xi);
            let xj = self.pick_alternative(x, xi);
            m.new_days.push(xj);
            if sol.move_feasible(&self.families, &m)  {
                let delta = sol.score_move(&self.families, &m);
                if  delta > 0.0 || (ind > 1 && self.accept(delta)) {
                    sol.apply_move(&self.families, &m);
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
            println!("delta: {}", delta);
            return true;
        } else {
            return false;
        }

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