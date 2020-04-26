use santa;

// Pareto Search?

struct NSGA2 {
    pop: Vec<santa::Solution>,
    npop: usize,
    pmut: f32,
}

impl NSGA2 {
    /// sorts the pop into non-dominated fronts, returns idx into self.pop
    /// TODO: cutoff
    fn dom_sort() -> Vec<Vec<usize>> {}
}
