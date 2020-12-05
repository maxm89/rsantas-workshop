use crate::santa;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::thread_rng;

struct SolutionNode {
    sol: santa::Solution,
    visited: usize,
    id: usize,
    not_improved_for: usize,
}

impl SolutionNode {
    fn new(sol: santa::Solution) -> Self {
        Self { sol, visited: 0, id:0, not_improved_for:0 }
    }
    fn new_with_id(sol: santa::Solution, id: usize) -> Self {
        Self { sol, visited: 0, id, not_improved_for:0 }
    }
}

struct OptimizationJob {
    sol: santa::Solution,
    reps: usize, // how often to optimize sol
}

impl OptimizationJob {
    pub fn new(sol: santa::Solution, reps: usize) -> Self {
        Self { sol, reps }
    }
}

pub struct SolutionQueue {
    todo_queue: Vec<OptimizationJob>,
    hist2: Vec<SolutionNode>,
    fmin: f32,
    outdir: String,
    id_cnt: usize,
}

///
impl SolutionQueue {
    pub fn new(outdir: String) -> Self {
        Self {
            todo_queue: Vec::new(),
            hist2: Vec::new(),
            fmin: 0.0,
            outdir,
            id_cnt: 0,
        }
    }
    /// Draw a solution from past solution candidates proportional to fitness rank and
    /// times visited to trade of exploitation and exploration.
    fn sample_history(&mut self) -> santa::Solution {
        let ind: Vec<usize> = (0..self.hist2.len()).collect();
        let visits: Vec<usize> = self.hist2.iter().map(|node| node.visited).collect();

        // argsort visits st lower visits have higher rank; scores are already ranked
        let mut visit_ranks: Vec<(&usize, usize)> = ind.iter().zip(visits).collect();
        visit_ranks.sort_by(|(_, lhs), (_, rhs)| rhs.partial_cmp(lhs).unwrap());
        let mut sums = Vec::new();
        // build squared sum of ranks for each solution to decrease the tail
        for (rankvisits, (&rankscore, _)) in visit_ranks.iter().enumerate() {
            let mut sm = rankscore + rankvisits;
            if sm == 0 {
                sm = 1;
            }
            sm *= sm;
            sums.push(sm);
        }
        let dist = WeightedIndex::new(&sums).unwrap();
        let rnd_idx = dist.sample(&mut thread_rng());
        let &rnd_idx = visit_ranks[rnd_idx].0;

        self.hist2[rnd_idx].visited += 1;
        self.hist2[rnd_idx].sol.clone()
    }

    /// Insert obtained solutions to continue from.
    /// Ignoring very similiar solutions amounts to a kind of tabu search.
    pub fn insert_history(&mut self, new_sol: santa::Solution) {
        let similarities = self.similarity(&new_sol);
        let new_costs = new_sol.costs;
        let mut new_node = SolutionNode::new(new_sol);
        let mut similiar_node = (0, 0);
        let mut found_similiar = false;
        for i in 0..self.hist2.len() {
            let sim = similarities[i];
            if sim > self.threshold(new_costs) && sim > similiar_node.0 {
                similiar_node = (sim, i);
                found_similiar = true;
            }
        }

        if found_similiar {
            let similiar_node_costs = self.hist2[similiar_node.1].sol.costs;
            if new_costs < similiar_node_costs {
                new_node.visited = self.hist2[similiar_node.1].visited;
                if new_costs < self.fmin {
                    santa::write_solution(&new_node.sol, &self.outdir);
                    self.fmin = new_costs;
                }
                self.hist2[similiar_node.1] = new_node;
                // now hist2 may not be sorted anymore
                self.hist2
                    .sort_by(|lhs, rhs| rhs.sol.costs.partial_cmp(&lhs.sol.costs).unwrap());

                self.print_stats();
            }
        } else {
            for i in 0..self.hist2.len() {
                if self.hist2[i].sol.costs < new_costs {
                    self.hist2.insert(i, new_node);
                    self.print_stats();
                    return;
                }
            }
            self.fmin = new_costs;
            santa::write_solution(&new_node.sol, &self.outdir);
            self.hist2.push(new_node);
            self.print_stats();
        }
    }

    pub fn insert_todo(&mut self, sol: santa::Solution, reps: usize) {
        let job = OptimizationJob::new(sol, reps);
        self.todo_queue.push(job);
    }
    /// Select an initial solution to start a new search with.
    /// If todo queue is empty, fill it with promising initial solutions from
    /// past searches and pick from them.
    pub fn select(&mut self) -> santa::Solution {
        let ntodo = self.todo_queue.len();
        if ntodo > 0 {
            let mut job = &mut self.todo_queue[ntodo - 1];
            job.reps -= 1;
            let todo = job.sol.clone();
            if job.reps <= 0 {
                self.todo_queue.pop();
            }
            return todo;
        } else {
            return self.sample_history();
        }
    }
    /// Determines the difference in days between sol and all solutions
    /// in history vec
    fn similarity(&self, sol: &santa::Solution) -> Vec<usize> {
        let mut res: Vec<usize> = Vec::new();
        for rhs in self.hist2.iter() {
            let mut fams_differ = 0;
            for fam in 0..5000 {
                if sol.x[fam] == rhs.sol.x[fam] {
                    fams_differ += 1;
                }
            }
            res.push(fams_differ);
        }
        res
    }

    fn threshold(&self, fitness: f32) -> usize {
        let thres: usize;
        if fitness < 70000.0 {
            thres = 4970;
        } else if fitness < 71000.0 {
            thres = 4960;
        } else {
            thres = 4800;
        }
        thres
    }
}

/// Aux. functions
impl SolutionQueue {
    fn print_stats(&self) {
        println!("[][] fmin: {}, nsols: {}", self.fmin, self.hist2.len());
    }
}
