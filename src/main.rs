#![allow(dead_code)]
// macro_use also imports the macros of a crate
#[macro_use]
extern crate serde_derive;
mod santa;
use santa::read_families;
mod ils;
mod solution_queue;
use std::env;
mod mcs;
use mcs::MonteCarloSearch;
mod initial;
use std::path::Path;

fn welcome() {
    println!("");
    println!("...Bad scheduler at work...");
    println!("");
}

struct CmdArgs {
    args: Vec<String>,
}

impl CmdArgs {
    fn new() -> CmdArgs {
        CmdArgs {
            args: env::args().collect(),
        }
    }
    fn parse_presence(&self, long: &str, short: &str) -> bool {
        let ddlong = self.append_ddash(long);
        let dshort = self.append_dash(short);

        self.find_str(ddlong.as_str(), dshort.as_str()) != None
    }
    fn parse_int_or(&self, long: &str, short: &str, default: usize) -> usize {
        let ddlong = self.append_ddash(long);
        let dshort = self.append_dash(short);
        let pos = self.find_str(ddlong.as_str(), dshort.as_str());
        let pos = match pos {
            Some(p) => p,
            None => return default,
        };
        if pos < self.args.len() - 1 {
            let arg = match self.args[pos + 1].parse::<usize>() {
                Ok(arg) => arg,
                Err(_) => default,
            };
            return arg;
        }
        default
    }

    fn parse_int(&self, long: &str, short: &str) -> usize {
        let ddlong = self.append_ddash(long);
        let dshort = self.append_dash(short);
        let pos = self
            .find_str(ddlong.as_str(), dshort.as_str())
            .expect("Argument not in argument list but mandatory. Quit.");
        let res: usize = self.args[pos + 1]
            .parse()
            .expect("Could not parse integer from argument. Quit.");
        res
    }
    fn parse_string_or(&self, long: &str, short: &str, default: &str) -> String {
        let ddlong = self.append_ddash(long);
        let dshort = self.append_dash(short);
        let pos = self.find_str(ddlong.as_str(), dshort.as_str());
        let pos = match pos {
            Some(p) => p,
            None => return String::from(default),
        };
        if pos < self.args.len() - 1 {
            let arg: String = match self.args[pos + 1].parse() {
                Ok(arg) => arg,
                Err(_) => String::from(default),
            };
            return arg;
        }
        String::from(default)
    }

    fn parse_string(&self, long: &str, short: &str) -> Option<String> {
        let ddlong = self.append_ddash(long);
        let dshort = self.append_dash(short);
        let pos = self.find_str(ddlong.as_str(), dshort.as_str());
        let pos = match pos {
            Some(p) => p,
            None => return None,
        };
        if pos < self.args.len() - 1 {
            let arg: Option<String> = match self.args[pos + 1].parse() {
                Ok(arg) => Some(arg),
                Err(_) => None,
            };
            return arg;
        }
        None
    }
    fn append_dash(&self, short: &str) -> String {
        String::from("-") + short
    }
    fn append_ddash(&self, long: &str) -> String {
        String::from("--") + long
    }

    fn find_str(&self, long: &str, short: &str) -> Option<usize> {
        self.args.iter().position(|x| x == long || x == short)
    }

    fn print_help(&self) {
        for arg in self.args.iter() {
            if arg == "help" {
                println!("USAGE:");
                println!("\t cargo run --release -- [OPTIONS]\n");
                println!("OPTIONS:");
                println!("\t-d, --depth <uint>\tMax depth of search moves (Default: 3)");
                println!("\t-n, --nthreads <uint>\tNumber of threads (Default: 1)");
                println!("\t-i, --ninit <uint>\tNumber of initial solutions to start from");
                println!(
                    "\t-s, --sol <path>\tInitial solution from file (use either --ninit or --sol)"
                );
                println!(
                    "\t-r, --nreps <uint>\tNumber of times to repeat optimizing initial solutions"
                );
                println!("\t-p, --npert <uint>\tNumber of perturbations per ILS run (Default: 15)");
                println!("\t-o, --outdir <path>\tOutput directory (Default: ./data/output/)");
                println!("");
                std::process::exit(0);
            }
        }
    }
}

fn main() {
    welcome();
    let cmd = CmdArgs::new();
    cmd.print_help();
    let families_path = cmd.parse_string_or("fam", "f", "./data/input/family_data.csv");
    let nthreads = cmd.parse_int_or("nthreads", "n", 1);
    // if no initial solution given: generate <ninit> random solutions
    let ninit = cmd.parse_int_or("ninit", "i", nthreads);
    // how often to optimize each initial sol
    let reps_per_sol = cmd.parse_int_or("nreps", "r", nthreads);
    let families = read_families(families_path.as_str());
    let solfile = cmd.parse_string("sol", "s");
    // max. depth of moves
    let depth = cmd.parse_int_or("depth", "d", 3);
    let nperturbations = cmd.parse_int_or("npert", "p", 15);
    let outdir = cmd.parse_string_or("outdir", "o", "./data/output/");
    if !Path::new(&outdir).exists() {
        panic!("Output directory doesn't exist, quit.")
    }
    // TODO print cfg
    let sols: Vec<santa::Solution> = match solfile {
        Some(path) => vec![santa::read_score_solution(&families, path.as_str())],
        None => {
            let mut sols: Vec<santa::Solution> = Vec::new();
            for _ in 0..ninit {
                sols.push(initial::pseudo_greedy(&families))
            }
            sols
        }
    };
    let mut mcs = MonteCarloSearch::new(
        families,
        reps_per_sol,
        nthreads,
        depth,
        outdir,
        nperturbations,
    );
    mcs.optimize_multi(sols);
}
