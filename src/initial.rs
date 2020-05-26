use crate::santa;
use rand::seq::SliceRandom;
use rand::Rng;


/// Constructs a valid solution by looping over families and assigning (if possible)
/// to each family a day (prefering their choices). The order by which families
/// are visited is randomized.
pub fn pseudo_greedy(families: &santa::FamilyData) -> santa::Solution {
    loop {
        let sol = pseudo_greedy_impl(families);
        if sol.is_feasible() {
            return sol;
        }
    }
}

pub fn random(families: &santa::FamilyData) -> santa::Solution {
    loop {
        let sol = random_impl(families);
        if sol.is_feasible() {
            return sol;
        }
    }
}

fn random_impl(families: &santa::FamilyData) -> santa::Solution {
    let mut rng = rand::thread_rng();
    let mut fams: Vec<u32> = Vec::new();
    for fam in 0..5000 {
        fams.push(fam);
    }
    fams.shuffle(&mut rand::thread_rng());
    let mut sol = santa::Solution::new();
    sol.x = vec![0; 5000];
    for i in 0..5000 {
        let fam = fams[i] as usize;
        let s = families.sizes[fam];
        //println!("i: {} s: {}", i, s);
        loop {
            let day: usize = rng.gen_range(1, 101);
            //println!("-{}", sol.occupancies[day]);
            if sol.occupancies[day] + s < 300 {
                sol.x[fam] = day as u32;
                sol.occupancies[day] += s;
                break;
            }
        }
    }

    sol.costs = families.score(&sol);
    println!("{}", sol.costs);
    sol
}

fn pseudo_greedy_impl(families: &santa::FamilyData) -> santa::Solution {
    let mut fams: Vec<u32> = Vec::new();
    for fam in 0..5000 {
        fams.push(fam);
    }
    fams.shuffle(&mut rand::thread_rng());
    let mut sol = santa::Solution::new();
    sol.x = vec![0; 5000];
    for i in 0..5000 {
        for j in 0..100 {
            let fam = fams[i] as usize;
            let day: usize = families.choices[fam][j] as usize;
            let s = families.sizes[fam];
            if sol.occupancies[day] < 125 && sol.occupancies[day] + s < 300 {
                sol.x[fam] = day as u32;
                sol.occupancies[day] += s;
                break;
            }
        }
    }

    sol.costs = families.score(&sol);
    sol
}
