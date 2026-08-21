mod ram;
mod cache;

use ram::RamError;

pub use ram::Ram;
pub use cache::{
    CacheLevel,
    CacheError,
};

#[derive(Debug, PartialEq, Eq)]
pub enum MemError {
    Ram(RamError),
    Cache(CacheError),
    UnreachableState,
    UpdateFailed,
    MismatchedSizes,
}
