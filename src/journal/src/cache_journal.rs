#[derive(Default, Debug, Clone)]
pub struct CacheJournal {
    cache_miss: Vec<u128>,
    cache_hit: Vec<u128>,
}

impl CacheJournal {

    pub fn new(cache_levels: usize) -> Self {
        CacheJournal {
            cache_miss: vec![0; cache_levels],
            cache_hit: vec![0; cache_levels],
        }
    }

    pub fn get(self) -> (Vec<u128>, Vec<u128>) {
        (self.cache_miss, self.cache_hit)
    }

    pub fn miss(&mut self, level: usize) {
        self.cache_miss[level] += 1;
    }

    pub fn hit(&mut self, level: usize) {
        self.cache_hit[level] += 1;
    }
}
