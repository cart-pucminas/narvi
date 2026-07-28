use serde::{Deserialize};

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum CachePolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

#[derive(Debug, Clone)]
pub enum CacheError {
    OutOfBounds,
    PolicyFailed,
    UnreachableState,
}

#[derive(Debug, Clone)]
pub enum CacheReturn {
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

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub n_blocks: usize,
    pub block_size: usize,
    pub set_size: usize,
    pub policy: CachePolicy
}
