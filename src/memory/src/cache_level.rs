use serde::{Deserialize};
use rand::{random_range};

use crate::cache_config::{
    CacheConfig, CacheError, CachePolicy, CacheReturn
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
    
    pub fn insert(&mut self, data: Vec<u8>) -> Result<usize, CacheError> {

        let sub = match self.policy_next() { 
            Ok(val) => val,
            Err(err) => {
                eprintln!("Finding next in the policy has failed. Defaulting to 0");
                eprintln!("{:?}", err);
                0
            }
        };

        self.data[sub].update(data, 0);

        match self.policy_update(sub, true) {
            Ok(_) => Ok(sub),
            Err(err) => Err(err)
        }
    }

    pub fn update(&mut self, data: Vec<u8>, offset: usize, set: usize) -> Result<usize, CacheError> {

        self.data[set].update(data, offset);
        match self.policy_update(set, false) {
            Ok(_) => Ok(set),
            Err(err) => Err(err)
        }
    }

    fn policy_next(&mut self) -> Result<usize, CacheError> {
        match self.policy {
            CachePolicy::LRU => { 
                match self.policy_list.last() {
                    Some(&res) => Ok(res),
                    None => {
                        Err(CacheError::PolicyFailed)
                    }
                }
            },
            CachePolicy::LFU => {
                match self.policy_list
                          .iter()
                          .enumerate()
                          .min_by_key( |(_, val)| *val)
                          .map( |(index, _)| index) {
                    Some(min_idx) => Ok(min_idx),
                    None => {
                        Err(CacheError::PolicyFailed)
                    }
                }
            },
            CachePolicy::FIFO => {
                match self.policy_list.pop() {
                    Some(res) => Ok(res),
                    None => {
                        if self.policy_list.len() != 0 {
                            Err(CacheError::PolicyFailed)
                        } else {
                            Ok(0)
                        }
                    }
                }
            },
            CachePolicy::Random => Ok(random_range(0..self.way))
        }
    }

    fn policy_update(&mut self, idx: usize, insertion: bool) -> Result<(), CacheError> {
        match self.policy {
            CachePolicy::LRU => { 
                let old_idx = match self.policy_list.iter().position(|x| *x == idx) {
                    Some(idx) => idx,
                    None => return Err(CacheError::PolicyFailed)
                };
                self.policy_list.remove(old_idx);
                self.policy_list.insert(0, idx);
                Ok(())
            },
            CachePolicy::LFU => {
                self.policy_list[idx] += 1;
                Ok(())
            },
            CachePolicy::FIFO => {
                if insertion {
                    self.policy_list.insert(0, idx);
                }
                Ok(())
            },
            CachePolicy::Random => Ok(()),
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
            n_sets,
            block_size: config.block_size
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
            n_sets,
            block_size
        }
    }

    // General insertion method
    pub fn insert(&mut self, addr: usize, data: Vec<u8>) -> Result<(), CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        match self.sets.get(idx) {
            Some(_) => {
                let tag = (addr & self.tag_mask) >> self.tag_start;

                match self.sets[idx].insert(data) {
                    Ok(_) => {
                        self.tags[idx] = tag;
                        self.dirty[idx] = false;
                        self.valid[idx] = true;
                        Ok(())
                    },
                    Err(err) => Err(err)
                }
            },
            _ => {
                Err(CacheError::OutOfBounds)
            }
        }
    }

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
                            return match self.sets[idx].update(data.clone(), offset, i) {
                                Ok(_) => {
                                    self.dirty[idx] = true;
                                    Ok(())
                                },
                                Err(err) => Err(err)
                            }
                        },
                        (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                        _ => (),
                    }
                }
                Err(CacheError::UnreachableState) // Updated always comes after a find
            },
            _ => {
                Err(CacheError::OutOfBounds)
            }
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
                    let block = self.sets[idx].data[i].clone();
                    let slice = &block.bytes[offset..bytes];

                    return match self.sets[idx].policy_update(i, false) {
                        Ok(_) => Ok(CacheReturn::Hit(slice.to_vec())),
                        Err(err) => Err(err)
                    };
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
                    return match self.sets[idx].policy_update(i, false) {
                        Ok(_) => Ok(true),
                        Err(err) => Err(err)
                    };
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
