use serde::{Deserialize};
use rand::{random_range};

macro_rules! mask_from {
    ($size:expr, $offset:expr) => {
        ((1 << $size) - 1) << $offset  
    };
}

pub enum CacheReturn {
    Hit(Vec<u8>),
    Miss,
    Error
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
}

#[derive(Debug, Deserialize)]
#[serde(from = "CacheConfig")]
pub struct CacheLevel{

    offset_mask: usize,
    index_mask: usize,
    tag_mask: usize,
    index_start: usize,
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

        CacheLevel {
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
    pub fn new(block_size: usize, associativity: usize, n_sets: usize) -> Self {
        let n_blocks = n_sets * associativity;

        let offset_size = block_size.ilog2() as usize;          // byte offset
        let index_size = n_blocks.ilog2() as usize; 

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; // remaining bits

        CacheLevel {
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

    // General insertion method
    pub fn insert(&mut self, addr: usize, data: Vec<u8>) -> bool {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = (tmp % self.n_sets) * self.way;
        let tag = addr & self.tag_mask;
        let offset = addr & self.offset_mask;

        // Should be changed to LRU in the future
        let i = random_range(0..self.way);

        match self.data.get(idx + i) {
            Some(_) => {
                self.data[idx + i].update(data, offset);
                self.valid[idx + i] = true;
                self.tags[idx + i] = tag;
                return true
            },
            _ => ()
        }
        false
    }

    // Insertion method for the CPU (enables dirty bit)
    pub fn update(&mut self, addr: usize, data: Vec<u8>) -> bool {
        return if self.insert(addr, data) {
            let idx = (addr & self.index_mask) % self.data.len();
            self.dirty[idx] = true;
            true

        } else {
            false
        }
    }

    // Returns the bytes in this cache level
    pub fn read_level(&self, addr: usize, bytes: usize) -> CacheReturn {
        let idx = (addr & self.index_mask) % self.n_sets;
        let tag = addr & self.tag_mask;
        let offset = addr & self.offset_mask;
        
        for i in 0..self.way {
            match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                (Some(&true), Some(&value)) => {
                    if value == tag {
                        let block = self.data[idx].clone();
                        let slice = &block.bytes[offset..bytes];
                        return CacheReturn::Hit(slice.to_vec());
                    }
                },
                (None, _) | (_, None) => return CacheReturn::Error,
                _ => (),
            }
        }
        CacheReturn::Miss
    }
}
