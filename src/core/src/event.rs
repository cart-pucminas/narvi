use std::fmt::Display;

use crate::ModuleId;

#[derive(Debug, Eq, PartialEq)]
pub enum EventPayload {
    HartExecute,
    MemoryLoadReq { address: u64, size: u8 },
    MemoryLoadRes { data: u64, size: u8},
    MemoryStoreReq { address: u64, data: u64, size: u8 },
}

impl EventPayload {
    fn as_str(&self) -> &'static str {
        match self {
            Self::HartExecute => "HartExecute",
            Self::MemoryLoadReq { .. } => "MemoryLoadReq",
            Self::MemoryLoadRes { .. } => "MemoryLoadRes",
            Self::MemoryStoreReq { .. } => "MemoryStoreReq",
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
