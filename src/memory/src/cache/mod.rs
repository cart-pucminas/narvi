mod cache_controller;
mod cache_level;
mod test;

pub use cache_controller::CacheController;
pub use cache_level::CacheLevel;

use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CacheReplacementPolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CacheWritePolicy {
    WriteThrough,
    WriteBack
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLevelConfig {
    pub n_blocks: usize,
    pub block_size: usize,
    pub set_size: usize,
    pub share_config: u8,
    pub replacement_policy: CacheReplacementPolicy,
    pub write_policy: CacheWritePolicy
}

impl CacheLevelConfig {
    pub fn new(
        n_blocks: usize, 
        block_size: usize, 
        set_size: usize, 
        share_config: u8, 
        replacement_policy: CacheReplacementPolicy, 
        write_policy: CacheWritePolicy
    ) -> Self {
        Self {
            n_blocks,
            block_size,
            set_size,
            share_config,
            replacement_policy,
            write_policy
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    OutOfBounds,
    PolicyFailed,
    UnreachableState,
}

#[derive(Debug, Clone)]
pub(crate) enum CacheReturn {
    Hit(Vec<u8>),
    Miss,
}

impl From<CacheReturn> for Vec<u8> {
    fn from (ret: CacheReturn) -> Self {
        return match ret {
            CacheReturn::Hit(value) => value,
            CacheReturn::Miss => vec![]
        }
    }
}

