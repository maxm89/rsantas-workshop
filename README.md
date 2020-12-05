# (.rs)antas-workshop tour 2019

Rust code that scored me rank 90 on the final leaderboard in the 2019 kaggle christmas competition (https://www.kaggle.com/c/santa-workshop-tour-2019/overview).
The competition was basically a combinatorial multiobjective optimization problem, similiar to the assignment problem with an additional nasty nonlinear constraint.

I came up with a meta heuristic approach that builds upon an Iteration Local Search (ILS) implementation. Starting from a set of initial solutions, the ILS is repeatedly used to generate a diverse set of solution candidates. From this set, solutions are drawn proportional to their objective function value and number of visits, to trade off exploitation and exploration of the search. 

Running the program with four threads from 500 random initial solutions, one should get results scoring < 70,000 in a day.
With reasonable parameters for a long run, the program can be started like:
```
cargo run --release -- --nthreads 4 --ninit 500 --nreps 1 --npert 25
```
The number of threads (--nthreads) should equal or less to the number of cores in your machine.
