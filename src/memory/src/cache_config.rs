use serde::{Deserialize};

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
    Error(&'static str)
}

impl From<CacheReturn> for Vec<u8> {
    fn from(ret: CacheReturn) -> Self {
        return match ret {
            CacheReturn::Hit(value) => value,
            CacheReturn::Error(err) => {
                println!("{}", err);
                vec![]
            },
            _ => vec![]
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
