use crate::{
    hart_journal::HartJournal,
    cache_journal::CacheJournal
};

#[derive(Default, Debug, Clone)]
pub struct Journal {
    pub cache_miss: Vec<u128>,
    pub cache_hit: Vec<u128>,
    pub cycles_lost: u128,
    pub num_cycles: u128,
    pub num_inst: u128,
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

    #[rustfmt::skip]
    pub fn dump(&self) -> String {
        let ipc = self.num_inst / self.num_cycles;
        let mut dump = format!("___ CPU Results ___
Total instructions: {}
Total cycles: {}
Total cycles lost: {}
IPC: {}\n
___ Memory Results ___\n",
            self.num_inst,
            self.num_cycles,
            self.cycles_lost,
            ipc
        );
        let mut miss_total: u128 = 0;
        let mut hit_total: u128 = 0;
        for i in 0..self.cache_miss.len() {
            dump.push_str(format!(" __ L{} __
Hits: {}
Misses: {}
Total accesses: {}
Miss rate: {}%\n",
                i+1, self.cache_hit[i], self.cache_miss[i],
                self.cache_hit[i] + self.cache_miss[i],
                (self.cache_miss[i] as f64 / (self.cache_hit[i] + self.cache_miss[i]) as f64) * 100.0
            ).as_str());
            miss_total += self.cache_miss[i];
            hit_total += self.cache_hit[i];
        }
        dump.push_str(format!(" __ Total __
Hits: {}
Misses: {}
Total accesses: {}
Miss rate: {}%\n",
            hit_total, miss_total,
            hit_total + miss_total,
            (miss_total as f64 / (hit_total + miss_total) as f64) * 100.0
        ).as_str());
        dump
    }
}

/*
    cache_miss: Vec<u128>,
    cache_hit: Vec<u128>,
    cycles_lost: u128,
    num_cycles: u128,
    num_inst: u128,
*/
