use serde::{Deserialize};
use rand::{random_range};

macro_rules! mask_from {
    ($size:expr, $offset:expr) => {
        ((1 << $size) - 1) << $offset  
    };
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum CachePolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

pub enum CacheReturn {
    Hit(Vec<u8>),
    Miss,
    Error
}

impl From<CacheReturn> for Vec<u8> {
    fn from(ret: CacheReturn) -> Self {
        return match ret {
            CacheReturn::Hit(value) => value,
            _ => vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheLine {
    pub bytes: Vec<u8>
}

impl CacheLine {
    fn new(block_size: usize) -> Self {
        CacheLine { bytes: vec![0; block_size] }
    }

    pub fn update(&mut self, vec: Vec<u8>, offset: usize) {
        let len = std::cmp::min(self.bytes.len(), vec.len());
        self.bytes[offset..len+offset].copy_from_slice(&vec[..len]);
    }

    pub fn print(&self) {
        println!("Line: {:?}", self.bytes);
    }
}

#[derive(Debug, Clone)]
pub struct CacheSet {
    data: Vec<CacheLine>,
    policy: CachePolicy,
    policy_list: Vec<usize>,    // List used for LRU, LFU and FIFO logic 
    way: usize,
}

impl CacheSet {
    pub fn new (block_size: usize, 
                set_size: usize,
                policy: CachePolicy
        ) -> Self {

        let policy_list : Vec<usize> = match policy {
            CachePolicy::LRU => (0..set_size).rev().collect(),
            CachePolicy::LFU => vec![0; set_size],
            CachePolicy::FIFO => vec![],
            CachePolicy::Random => vec![],
        };

        CacheSet {
            data: vec![CacheLine::new(block_size); set_size],
            policy,
            policy_list,
            way: set_size
        }
    }
    
    pub fn insert(&mut self, data: Vec<u8>, offset: usize) -> usize {
        let sub = self.policy_next();
        println!("sub: {}", sub);
        self.data[sub].update(data, offset);
        self.policy_update(sub, true);
        sub
    }

    pub fn update(&mut self, idx: usize, data: Vec<u8>, offset: usize) {
        self.data[idx].update(data, offset);
        self.policy_update(idx, false);
    }

    fn policy_next(&mut self) -> usize {
        match self.policy {
            CachePolicy::LRU => { 
                match self.policy_list.last() {
                    Some(&res) => res,
                    None => {
                        eprintln!("Finding next position from the policy list has failed. 
                            Defaulting to 0");
                        0
                    }
                }
            },
            CachePolicy::LFU => {
                match self.policy_list
                          .iter()
                          .enumerate()
                          .min_by_key( |(_, val)| *val)
                          .map( |(index, _)| index) {
                    Some(min_idx) => min_idx,
                    None => {
                        eprintln!("Finding next position from the policy list has failed. 
                            Defaulting to 0");
                        0
                    }
                }
            },
            CachePolicy::FIFO => {
                match self.policy_list.pop() {
                    Some(res) => res,
                    None => {
                        if self.policy_list.len() != 0 {
                            eprintln!("Finding next position from the policy list has failed. 
                                Defaulting to 0");
                        }
                        0
                    }
                }
            },
            CachePolicy::Random => random_range(0..self.way)
        }
    }

    fn policy_update(&mut self, idx: usize, new_insertion: bool) {
        match self.policy {
            CachePolicy::LRU => { 
                let old_idx = match self.policy_list.iter().position(|x| *x == idx) {
                    Some(idx) => idx,
                    None => {
                        eprintln!("Policy update failed! Could not find element {}", idx);
                        0
                    }
                };
                self.policy_list.remove(old_idx);
                self.policy_list.insert(0, idx);
            },
            CachePolicy::LFU => {
                self.policy_list[idx] += 1;
            },
            CachePolicy::FIFO => {
                if new_insertion {
                    self.policy_list.insert(0, idx);
                }
            },
            CachePolicy::Random => (),
        }
    }

    pub fn print (&self) {
        for line in &self.data {
            line.print();
        }
        println!("policy list: {:?}", self.policy_list);
    }
}

#[derive(Debug, Deserialize)]
struct CacheConfig {
    n_blocks: usize,
    block_size: usize,
    set_size: usize,
    policy: CachePolicy
}

#[derive(Debug, Deserialize)]
#[serde(from = "CacheConfig")]
pub struct CacheLevel{

    #[serde(skip)]
    offset_mask: usize,
    #[serde(skip)]
    index_mask: usize,
    #[serde(skip)]
    tag_mask: usize,

    #[serde(skip)]
    index_start: usize,
    #[serde(skip)]
    tag_start: usize,

    #[serde(skip)]
    pub sets: Vec<CacheSet>,
    #[serde(skip)]
    tags: Vec<usize>,
    #[serde(skip)]
    valid: Vec<bool>,
    #[serde(skip)]
    dirty: Vec<bool>,

    #[serde(skip)]
    way: usize,
    #[serde(skip)]
    n_sets: usize,
}

impl From<CacheConfig> for CacheLevel {
    fn from(config: CacheConfig) -> Self {

        let offset_size = config.block_size.ilog2() as usize;
        let index_size = config.n_blocks.ilog2() as usize;

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; // remaining bits

        let n_sets = config.n_blocks / config.set_size;
        let base_set = CacheSet::new(config.block_size, config.set_size, config.policy);

        CacheLevel {
            index_mask, 
            tag_mask, 
            offset_mask,
            index_start: offset_size,
            tag_start: offset_size + index_size,
            sets: vec![base_set; n_sets],
            tags: vec![0; config.n_blocks],
            valid: vec![false ; config.n_blocks], 
            dirty: vec![false ; config.n_blocks], 
            way: config.set_size,
            n_sets
        }
    }
}

impl CacheLevel {
    pub fn new(block_size: usize, associativity: usize, n_sets: usize, policy: CachePolicy) -> Self {
        let n_blocks = n_sets * associativity;

        let offset_size = block_size.ilog2() as usize;          // byte offset
        let index_size = n_blocks.ilog2() as usize; 

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; // remaining bits

        let base_set = CacheSet::new(block_size, associativity, policy);

        CacheLevel {
            index_mask,
            tag_mask,
            offset_mask,
            index_start: offset_size,
            tag_start: index_size + offset_size,
            sets: vec![base_set; n_sets],
            tags: vec![0; n_blocks],
            valid: vec![false ; n_blocks], 
            dirty: vec![false ; n_blocks], 
            way: associativity,
            n_sets
        }
    }

    // General insertion method
    pub fn insert(&mut self, addr: usize, data: Vec<u8>, set_dirty: bool) -> bool {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;
        println!("idx for {:?} is: {}", data, idx);
        println!("offset is: {}", addr & self.offset_mask);

        match self.sets.get(idx) {
            Some(_) => {
                let tag = (addr & self.tag_mask) >> self.tag_start;
                let _ = self.sets[idx].insert(data, addr & self.offset_mask);

                self.tags[idx] = tag;
                self.dirty[idx] = set_dirty;
                self.valid[idx] = true;
                true
            },
            _ => {
                eprintln!("Insertion failed! Could not index the set at position: {}", idx);
                false
            }
        }
    }

    // Returns the bytes in this cache level
    pub fn read_level(&mut self, addr: usize, bytes: usize) -> CacheReturn {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        let tag = (addr & self.tag_mask) >> self.tag_start;
        
        for i in 0..self.way {
            match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                (Some(&true), Some(&value)) if value == tag => {
                    let offset = addr & self.offset_mask;
                    let block = self.sets[idx].data[i].clone();
                    let slice = &block.bytes[offset..bytes];

                    return CacheReturn::Hit(slice.to_vec());
                },
                (None, _) | (_, None) => return CacheReturn::Error,
                _ => (),
            }
        }
        CacheReturn::Miss
    }

    pub fn print(&self) {
        let mut i = 0;
        for set in &self.sets {
            println!("\n=> set {}", i);
            set.print();
            i+=1;
        }
    }
}
