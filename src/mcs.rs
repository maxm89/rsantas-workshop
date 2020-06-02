use crate::ils;
use crate::santa::{FamilyData, Solution};
use crate::solution_queue::SolutionQueue;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct MonteCarloSearch {
    families: FamilyData,
    reps_per_sol: usize,
    nthreads: usize,
    move_depth: usize,
    outdir: String,
    nperturbations: usize,
}

impl MonteCarloSearch {
    pub fn new(
        families: FamilyData,
        reps_per_sol: usize,
        nthreads: usize,
        move_depth: usize,
        outdir: String,
        nperturbations: usize,
    ) -> Self {
        Self {
            families,
            reps_per_sol,
            nthreads,
            move_depth,
            outdir,
            nperturbations,
        }
    }

    /// Kind of parallel Monte Carlo search. Starts by optimizing an initial solution,
    /// stores the result alongside the initial solution and continues optimizing one of the
    /// already found solutions. Initial solutions are picked with a bias towards
    /// unexplored and promising solution candidates.
    pub fn optimize_multi(&mut self, sols: Vec<Solution>) {
        println!(
            "[] Starting Monte Carlo search with {} thread(s)",
            self.nthreads
        );
        let sq = Arc::new(Mutex::new(SolutionQueue::new(self.outdir.clone())));
        let mut sq_lock = sq.lock().unwrap();
        for sol in sols {
            sq_lock.insert_todo(sol, self.reps_per_sol);
        }
        std::mem::drop(sq_lock);
        let mut cnt = 10;
        loop {
            let mut thread_handles = Vec::new();
            for _ in 0..self.nthreads {
                let mut sq_lock = sq.lock().unwrap();
                let mut sol = sq_lock.select();
                std::mem::drop(sq_lock);
                // clone sq because it is moved into the thread
                let fams = self.families.clone();
                let sq = Arc::clone(&sq);
                let move_depth = self.move_depth;
                let npert = self.nperturbations;
                thread_handles.push(thread::spawn(move || {
                    loop {
                        let new_sol;
                        let mut ils = ils::ILS::new(fams.clone(), move_depth);
                        if cnt > 0 {
                            new_sol = ils.optimize(sol.clone(), npert, true);
                            cnt -= 1;
                        } else {
                            new_sol = ils.optimize(sol.clone(), npert, false);
                        }
                        // when optimization is completed, write solution to history
                        let mut sq_lock = sq.lock().unwrap();
                        sq_lock.insert_history(new_sol);
                        sol = sq_lock.select();
                    }
                }));
            }
            for thread_handle in thread_handles {
                thread_handle.join().expect("Could not join thread.?!");
            }
        }
    }

    pub fn print_config(&self) {}
}
