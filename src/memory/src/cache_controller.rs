#[derive(Debug, Deserialize)]
struct CacheConfig {
    n_blocks: usize,
    block_size: usize,
    set_size: usize,
}

enum CacheState {
    Idle,
    CompareTag,
    WriteBack,
    Allocate,
}

struct CacheLevel{
    data: Vec<usize>,
    valid: Vec<bool>,
    dirty: Vec<bool>,
    cstate: CacheState,
    index_start: usize,
    tag_start: usize,
    way: usize,
}

impl CacheLevel {
    pub fn new(block_size: usize, associativity: usize, n_sets: usize) -> Self {
        let cache_size = n_sets * block_size * associativity;
        
    }
}
