use std::{fmt::Display, sync::Arc};

use crate::{
    ModuleId
};

pub enum JournalEvent {
    CacheHit,
    CacheMiss,
    Cycles { cycles: usize },
    CyclesLost { cycles: usize },
    HartInstruction
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Target {
    Module(ModuleId),
    Myself
}

#[derive(Debug, Eq, PartialEq)]
pub enum EventPayload {
    HartExecute,
    MemoryLoadReq { address: usize, size_in_bytes: usize, requester: Target },
    MemoryLoadRes { data: Vec<u8> },
    MemoryStoreReq { address: usize, data: Vec<u8> },
    Reset
}

impl EventPayload {
    fn as_str(&self) -> &'static str {
        match self {
            Self::HartExecute => "HartExecute",
            Self::MemoryLoadReq { .. } => "MemoryLoadReq",
            Self::MemoryLoadRes { .. } => "MemoryLoadRes",
            Self::MemoryStoreReq { .. } => "MemoryStoreReq",
            Self::Reset => "Reset"
        }
    }
}

impl Display for EventPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Event {
    timestamp: u64,
    target: ModuleId,
    payload: EventPayload
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}) @ {}", self.payload, self.target, self.timestamp)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Event {
    pub fn new(
        timestamp: u64, 
        target: ModuleId, 
        payload: EventPayload
    ) -> Self {
        Self {
            timestamp,
            target,
            payload
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn target(&self) -> usize {
        self.target
    }

    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }
}
