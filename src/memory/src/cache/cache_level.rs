use serde::Deserialize;
use rand::random_range;

use super::{
    CacheLevelConfig, 
    CacheError, 
    CacheReplacementPolicy, 
    CacheReturn
};

macro_rules! mask_from {
    ($size:expr, $offset:expr) => {
        ((1 << $size) - 1) << $offset  
    };
}

#[derive(Debug, Clone)]
pub struct CacheLine {
    pub bytes: Vec<u8>
}

impl CacheLine {
    pub fn new(block_size: usize) -> Self {
        CacheLine { bytes: vec![0; block_size] }
    }

    pub fn update(&mut self, vec: Vec<u8>, offset: usize) -> Result<(), CacheError> {
        let len = std::cmp::min(self.bytes.len(), vec.len());

        if let Some(slice) = self.bytes.get_mut(offset..len+offset) {
            slice.copy_from_slice(&vec[..len]);
            return Ok(())
        } 
        Err(CacheError::OutOfBounds)
    }

    pub fn print(&self) {
        println!("Line: {:?}", self.bytes);
    }
}

#[derive(Debug, Clone)]
pub struct CacheSet {
    cache_lines: Vec<CacheLine>,
    policy: CacheReplacementPolicy,
    policy_list: Vec<usize>,    // List used for LRU, LFU and FIFO logic 
    way: usize,
}

impl CacheSet {
    pub fn new (block_size: usize, 
                set_size: usize,
                policy: CacheReplacementPolicy
        ) -> Self {

        let policy_list : Vec<usize> = match policy {
            CacheReplacementPolicy::LRU => (0..set_size).rev().collect(),
            CacheReplacementPolicy::LFU => vec![0; set_size],
            CacheReplacementPolicy::FIFO => vec![],
            CacheReplacementPolicy::Random => vec![],
        };

        CacheSet {
            cache_lines: vec![CacheLine::new(block_size); set_size],
            policy,
            policy_list,
            way: set_size
        }
    }
    
    pub fn insert(&mut self, data: Vec<u8>, replace_i: usize) -> Result<usize, CacheError> {
        self.cache_lines[replace_i].update(data, 0)?;
        self.policy_update(replace_i, true)?;
        Ok(replace_i)
    }

    pub fn update(&mut self, data: Vec<u8>, offset: usize, set: usize) -> Result<(), CacheError> {
        self.cache_lines[set].update(data, offset)?;
        self.policy_update(set, false)?;
        Ok(())
    }

    fn policy_next(&mut self) -> Result<usize, CacheError> {
        match self.policy {
            CacheReplacementPolicy::LRU => { 
                match self.policy_list.last() {
                    Some(&res) => Ok(res),
                    None => Err(CacheError::PolicyFailed)
                }
            },
            CacheReplacementPolicy::LFU => {
                match self.policy_list
                          .iter()
                          .enumerate()
                          .min_by_key( |(_, val)| *val)
                          .map( |(index, _)| index) {
                    Some(min_idx) => Ok(min_idx),
                    None => Err(CacheError::PolicyFailed)
                }
            },
            CacheReplacementPolicy::FIFO => {
                match self.policy_list.pop() {
                    Some(res) => Ok(res),
                    None => {
                        if self.policy_list.len() != 0 {
                            Err(CacheError::PolicyFailed)
                        } else {
                            Ok(0)   // First insertion
                        }
                    }
                }
            },
            CacheReplacementPolicy::Random => Ok(random_range(0..self.way))
        }
    }

    fn policy_update(&mut self, idx: usize, insertion: bool) -> Result<(), CacheError> {
        match self.policy {
            CacheReplacementPolicy::LRU => { 
                let old_pos = match self.policy_list.iter().position(|x| *x == idx) {
                    Some(pos) => pos,
                    None => return Err(CacheError::PolicyFailed)
                };
                self.policy_list.remove(old_pos);
                self.policy_list.insert(0, idx);
            },
            CacheReplacementPolicy::LFU => {
                if insertion {  // Reset on new block
                    self.policy_list[idx] = 0;
                }
                self.policy_list[idx] += 1;
            },
            CacheReplacementPolicy::FIFO => {
                if insertion {
                    self.policy_list.insert(0, idx);
                }
            },
            CacheReplacementPolicy::Random => (),
        }
        Ok(())
    }

    pub fn print (&self) {
        for line in &self.cache_lines {
            line.print();
        }
        println!("policy list: {:?}", self.policy_list);
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "CacheLevelConfig")]
pub struct CacheLevel {
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
    sets: Vec<CacheSet>,
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
    
    block_size: usize,
}

impl From<CacheLevelConfig> for CacheLevel {
    fn from(config: CacheLevelConfig) -> Self {

        let offset_size = config.block_size.ilog2() as usize;
        let index_size = config.n_blocks.ilog2() as usize;

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; // remaining bits

        let n_sets = config.n_blocks / config.set_size;
        let base_set = CacheSet::new(
            config.block_size, 
            config.set_size, 
            config.replacement_policy
        );

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
            n_sets,
            block_size: config.block_size
        }
    }
}

impl CacheLevel {
    pub fn new(block_size: usize, associativity: usize, n_sets: usize, policy: CacheReplacementPolicy) -> Self {
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
            n_sets,
            block_size
        }
    }

    fn get_old (&mut self, index: usize, set_i: usize) -> Option<(usize, Vec<u8>)> {

        let cache_lines = self.sets.get(index)?;
        let set = cache_lines.cache_lines.get(set_i)?;

        let old_block = set.bytes.clone();
        
        let tag = self.tags.get(index + set_i)? << self.tag_start;
        let old_addr = tag | (index << self.index_start);

        Some((old_addr, old_block))
    }

    // Used for new blocks. Does not set the dirty bit
    pub fn insert(&mut self, addr: usize, data: Vec<u8>) -> Result<Option<(usize, Vec<u8>)>, CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        match self.sets.get(idx) {
            Some(_) => {

                let tag = (addr & self.tag_mask) >> self.tag_start;

                let i = match self.sets[idx].policy_next() {
                    Ok(sub) => sub,
                    Err(err) => { println!("{:?}. Defaulting to 0", err); 0 }
                };

                let mut res: Option<(usize, Vec<u8>)> = None;

                if self.dirty[idx + i] {
                    res = Some(self.get_old(idx, i)
                        .ok_or(CacheError::OutOfBounds)?);

                    self.dirty[idx + i] = false;
                }

                self.sets[idx].insert(data, i)?;

                self.tags[idx + i] = tag;
                self.valid[idx + i] = true;

                Ok(res)
            },
            _ => Err(CacheError::OutOfBounds)
        }
    }

    // Used for blocks that are already in the cache. Sets the dirty bit
    pub fn update(&mut self, addr: usize, data: Vec<u8>) -> Result<(), CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        match self.sets.get(idx) {
            Some(_) => {
                let tag = (addr & self.tag_mask) >> self.tag_start;
                let offset = addr & self.offset_mask;

                for i in 0..self.way {
                    match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                        (Some(&true), Some(&value)) if value == tag => {
                            self.sets[idx].update(data, offset, i)?;
                            self.dirty[idx] = true;
                            return Ok(());
                        },
                        (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                        _ => (),
                    }
                }
                Err(CacheError::UnreachableState) // Updated always comes after a find
            },
            None => Err(CacheError::OutOfBounds)
        }
    }

    // Returns a amount of bytes in this cache level
    pub fn read(&mut self, addr: usize, bytes: usize) -> Result<CacheReturn, CacheError> {

        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;
        let tag = (addr & self.tag_mask) >> self.tag_start;
        
        for i in 0..self.way {
            match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                (Some(&true), Some(&value)) if value == tag => {
                    let offset = addr & self.offset_mask;
                    let block = self.sets[idx].cache_lines[i].clone();
                    let slice = &block.bytes[offset..bytes];

                    self.sets[idx].policy_update(i, false)?;
                    return Ok(CacheReturn::Hit(slice.to_vec()))
                },
                (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                _ => (),
            }
        }
        Ok(CacheReturn::Miss)
    }

    pub fn get_block(&mut self, addr: usize) -> Result<CacheReturn, CacheError> {
        self.read(addr, self.block_size)
    }

    pub fn find(&mut self, addr: usize) -> Result<bool, CacheError> {

        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;
        let tag = (addr & self.tag_mask) >> self.tag_start;
        
        for i in 0..self.way {
            match (self.valid.get(idx + i), self.tags.get(idx + i)) {
                (Some(&true), Some(&value)) if value == tag => {

                    self.sets[idx].policy_update(i, false)?;
                    return Ok(true);
                },
                (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                _ => (),
            }
        }
        Ok(false)
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
