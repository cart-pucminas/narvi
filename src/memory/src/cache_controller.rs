use serde::{Deserialize};
use rand::{random_range};

macro_rules! mask_from {
    ($size:expr, $offset:expr) => {
        ((1 << $size) - 1) << $offset  
    };
}

#[derive(Debug, Deserialize)]
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

    fn update(&mut self, vec: Vec<u8>, offset: usize) {
        let len = std::cmp::min(self.bytes.len(), vec.len());
        self.bytes[offset..len+offset].copy_from_slice(&vec[..len]);
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

    policy: CachePolicy,

    #[serde(skip)]
    policy_list: Vec<usize>,    // List used for LRU, LFU and FIFO logic 

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
    pub data: Vec<CacheLine>,
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

        let policy_list : Vec<usize> = match config.policy {
            CachePolicy::LRU => (0..config.set_size).rev().collect(),
            CachePolicy::LFU => vec![0; config.set_size],
            CachePolicy::FIFO => vec![],
            CachePolicy::Random => vec![],
        };

        CacheLevel {
            policy: config.policy,
            policy_list,
            index_mask, 
            tag_mask, 
            offset_mask,
            index_start: offset_size,
            tag_start: offset_size + index_size,
            data: vec![CacheLine::new(config.block_size); config.n_blocks], 
            tags: vec![0; config.n_blocks],
            valid: vec![false ; config.n_blocks], 
            dirty: vec![false ; config.n_blocks], 
            way: config.set_size,
            n_sets: config.n_blocks / config.set_size
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

        let policy_list : Vec<usize> = match policy {
            CachePolicy::LRU => (0..associativity).rev().collect(),
            CachePolicy::LFU => vec![0; associativity],
            CachePolicy::FIFO => vec![],
            CachePolicy::Random => vec![],
        };

        CacheLevel {
            policy,
            policy_list,
            index_mask,
            tag_mask,
            offset_mask,
            index_start: offset_size,
            tag_start: index_size + offset_size,
            data: vec![CacheLine::new(block_size); n_blocks], 
            tags: vec![0; n_blocks],
            valid: vec![false ; n_blocks], 
            dirty: vec![false ; n_blocks], 
            way: associativity,
            n_sets
        }
    }

    fn policy_next(&mut self) -> usize {
        match self.policy {
            CachePolicy::LRU => { 
                match self.policy_list.pop() {
                    Some(res) => {
                        self.policy_list.insert(0, res);
                        res
                    }
                    None => {
                        eprintln!("Finding next position from the policy list has failed. 
                            Defaulting to 0");
                        0
                    }
                }
            },
            CachePolicy::LFU => {
                match self.policy_list.iter().min() {
                    Some(&min) => min,
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

    fn policy_update(&mut self, idx: usize) {
        todo!();
    }

    // General insertion method
    pub fn insert(&mut self, addr: usize, data: Vec<u8>, set_dirty: bool) -> bool {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = (tmp % self.n_sets) * self.way;

        let i = self.policy_next();
        
        match self.data.get(idx + i) {
            Some(_) => {
                self.data[idx + i].update(data, addr & self.offset_mask);
                self.valid[idx + i] = true;
                self.tags[idx + i] = (addr & self.tag_mask) >> self.tag_start;
                if set_dirty {
                    self.dirty[idx + i] = true;
                }
                self.policy_update(idx + i);
                return true
            },
            _ => eprintln!("Insertion failed! Could not index position: {}", idx + i)
        }
        false
    }

    // Returns the bytes in this cache level
    pub fn read_level(&mut self, addr: usize, bytes: usize) -> CacheReturn {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = (tmp % self.n_sets) * self.way;

        let tag = (addr & self.tag_mask) >> self.tag_start;
        
        for i in 0..self.way {
            match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                (Some(&true), Some(&value)) if value == tag => {
                    let offset = addr & self.offset_mask;
                    let block = self.data[idx + i].clone();
                    let slice = &block.bytes[offset..bytes];

                    self.policy_update(idx + i);

                    return CacheReturn::Hit(slice.to_vec());
                },
                (None, _) | (_, None) => return CacheReturn::Error,
                _ => (),
            }
        }
        CacheReturn::Miss
    }
}
