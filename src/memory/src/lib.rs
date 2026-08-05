mod ram;
mod cache_config;
mod cache_level;
mod cache_controller;

use ram::RamError;
use cache_config::CacheError;

pub use ram::Ram;
pub use cache_config::CacheReturn;
pub use cache_config::{
    CacheLevelConfig, 
    CacheReplacementPolicy,
    CacheWritePolicy
};
pub use cache_level::CacheLevel;
pub use cache_controller::CacheController;

#[derive(Debug, PartialEq, Eq)]
pub enum MemError {
    Ram(RamError),
    Cache(CacheError),
    UnreachableState,
    UpdateFailed,
    MismatchedSizes,
}

/// Manages runtime state and orchestrates memory interactions between RAM
/// and cache subsystems for higher-level consumers.
#[derive(Debug)]
pub struct MemoryRuntimeContext {
    ram: Ram,
    cache_controller: CacheController
}

impl MemoryRuntimeContext {
    pub fn new(ram: Ram, cache_controller: CacheController) -> Self {
        Self {
            ram,
            cache_controller
        }
    }

    pub fn read_8(&mut self, addr: usize) -> Result<u8, MemError> {
        self.cache_controller.read_8(&mut self.ram, addr)
    }

    pub fn read_16(&mut self, addr: usize) -> Result<u16, MemError> {
        self.cache_controller.read_16(&mut self.ram, addr)
    }

    pub fn read_32(&mut self, addr: usize) -> Result<u32, MemError> {
        self.cache_controller.read_32(&mut self.ram, addr)
    }

    pub fn read_64(&mut self, addr: usize) -> Result<u64, MemError> {
        self.cache_controller.read_64(&mut self.ram, addr)
    }

    pub fn write_8(&mut self, addr: usize, data: u8) -> Result<(), MemError> {
        self.cache_controller.write_8(&mut self.ram, addr, data)
    }

    pub fn write_16(&mut self, addr: usize, data: u16) -> Result<(), MemError> {
        self.cache_controller.write_16(&mut self.ram, addr, data)
    }

    pub fn write_32(&mut self, addr: usize, data: u32) -> Result<(), MemError> {
        self.cache_controller.write_32(&mut self.ram, addr, data)
    }

    pub fn write_64(&mut self, addr: usize, data: u64) -> Result<(), MemError> {
        self.cache_controller.write_64(&mut self.ram, addr, data)
    }
}
