# (.rs)antas-workshop tour 2019

Rust code that scored me rank 90 on the final leaderboard in the 2019 kaggle christmas competition (https://www.kaggle.com/c/santa-workshop-tour-2019/overview).
The competition was basically a combinatorial multiobjective optimization problem, similiar to the assignment problem with an additional nasty nonlinear constraint.

I didn't manage to get satisfactory results with free solvers out of my linear programming model so i came up with a meta heuristic approach that works as follows.
A Iteration Local Search (ILS) method constitutes the core of the program. Starting from a set of initial solutions, the ILS is repeatedly used to further improve obtained solutions. A set of diverse solutions is maintained from which new searches are started. From this set, solutions are drawn proportional to their objective function value and number of visits, trading off exploitation and exploration of the search. 

To compile and run the code, a functioning installation of cargo is required (https://github.com/rust-lang/cargo).

Running the program with four threads from 500 random initial solutions, one should get results scoring < 7000 in a day.
With reasonable parameters for a long run, the program can be started like:
```
cargo run --release -- --nthreads 4 --ninit 500 --nreps 1 --npert 25
```
The number of threads (--nthreads) should equal the number of cores in your machine.
