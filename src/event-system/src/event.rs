use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

pub enum EventType {
     
}

// #[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    instant: u64,
    priority: u8,
    event_type: EventType,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant && self.priority == other.priority 
    }

    fn ne(&self, other: &Self) -> bool {
        self.instant != other.instant || self.priority != other.priority
    }
}

impl Eq for Event { }

impl PartialOrd for Event {
    fn lt(&self, other: &Self) -> bool {
        self.instant < other.instant || (self.instant == other.instant && self.priority < other.priority)  
    }

    fn le(&self, other: &Self) -> bool {
        self.instant < other.instant || (self.instant == other.instant && self.priority <= other.priority) 
    }

    fn gt(&self, other: &Self) -> bool {
        self.instant > other.instant || (self.instant == other.instant && self.priority > other.priority)
    }

    fn ge(&self, other: &Self) -> bool {
        self.instant > other.instant || (self.instant == other.instant && self.priority >= other.priority)
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.eq(other) {
            Some(std::cmp::Ordering::Equal)
        } else if self.lt(other) {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    } 
}

impl Event {
    pub fn new(instant: u64, priority: u8, event_type: EventType) -> Event {
        Event { instant, priority, event_type }
    }

    pub fn instant(&self) -> u64 {
        self.instant
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }
}
