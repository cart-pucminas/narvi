#[derive(Default, Debug, Clone)]
pub struct HartJournal {
    cycles_lost: u128,
    num_cycles: u128,
    num_inst: u128,
}

impl HartJournal {
    pub fn new() -> Self {
        HartJournal {
            cycles_lost: 0,
            num_cycles: 0,
            num_inst: 0,
        }
    }

    pub fn get(self) -> (u128, u128, u128) {
        (self.cycles_lost, self.num_cycles, self.num_inst)
    }

    pub fn lost_cycle(&mut self, amount: u128) {
        self.cycles_lost += amount;
    }

    pub fn cycles_done(&mut self, amount: u128) {
        self.num_cycles += amount;
    }

    pub fn inst_done(&mut self, amount: u128) {
        self.num_inst += amount;
    }
}
