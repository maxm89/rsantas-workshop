use crate::initial;
use crate::santa;
use crate::SA;
use crate::sa::TabuList;

pub struct TabuSearch {
    families: santa::FamilyData
}

impl TabuSearch {
    pub fn new(families: santa::FamilyData) -> Self {
        Self {
            families
        }
    }

    // TODO tabu when close to current optimums

    pub fn optimize(&mut self) {
        // init sol
        // optimize and copy tabu list
        // start with new sol and old tabu list
        let mut tabu = TabuList::new(1000);
        loop {
            let mut sol = initial::random(&self.families);
            let mut sa = SA::new(self.families.clone(), 5.0, 1000);
            sa.set_tabu(tabu);
            sa.optimize(sol);
            tabu = sa.tabu;
        }
    }
}