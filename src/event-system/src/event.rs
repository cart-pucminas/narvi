use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    instant: u64,
    priority: u8,
}

impl Event {
    pub fn new(instant: u64, priority: u8) -> Event {
        Event { instant, priority }
    }

    pub fn instant(&self) -> u64 {
        self.instant
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }
}
