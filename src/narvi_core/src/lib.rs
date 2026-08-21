use crate::event::{Event, EventPayload, Target};

pub mod event;
pub mod bytes;
pub mod serialization;

pub type ModuleId = usize;

pub trait EngineContext {
    fn schedule(&mut self, timestamp: u64, target: Target, payload: EventPayload);
}

pub trait Module { 
    fn process_event(&mut self, event: Event, engine_context: &mut dyn EngineContext);
}

#[derive(Debug, Clone, Copy)]
pub enum CacheReplacementPolicy {
    LRU,
    LFU,
    FIFO,
    Random,
}

#[derive(Debug, Copy, Clone)]
pub enum CacheWritePolicy {
    WriteThrough,
    WriteBack
}

#[derive(Debug, Clone)]
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

#[allow(dead_code, unused_variables)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extensions {
    pub m: bool,
    pub a: bool,
    pub c: bool,
    pub f: bool,
    pub d: bool,
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

impl Extensions {

    pub fn new() -> Extensions {
        Extensions { m: false, a: false, c: false, f: false, d: false }
    }
}
