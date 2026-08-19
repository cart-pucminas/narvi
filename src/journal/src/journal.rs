use crate::{
    hart_journal::HartJournal,
    cache_journal::CacheJournal
};

#[derive(Default, Debug, Clone)]
pub struct Journal {
    cache_miss: Vec<u128>,
    cache_hit: Vec<u128>,
    cycles_lost: u128,
    num_cycles: u128,
    num_inst: u128,
}

impl Journal {
    pub fn new(cache_levels: usize) -> Self {
        Journal {
            cache_miss: vec![0; cache_levels],
            cache_hit: vec![0; cache_levels],
            cycles_lost: 0,
            num_cycles: 0,
            num_inst: 0,
        }
    }
    
    pub fn merge_hart(&mut self, hart: HartJournal) {
        let (cycles_lost, num_cycles, num_inst) = hart.get();
        self.cycles_lost += cycles_lost;
        self.num_cycles += num_cycles;
        self.num_inst += num_inst;
    }
    
    pub fn merge_cache(&mut self, cache: CacheJournal) {
        let (cache_miss, cache_hit) = cache.get();

        let n = cache_miss.len();

        for i in 0..n {
            self.cache_miss[i] += cache_miss[i];
            self.cache_hit[i] += cache_hit[i];
        }
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
