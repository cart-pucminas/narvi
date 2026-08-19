use crate::event::{Event, EventPayload};

pub mod event;

pub type ModuleId = usize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Size {
    Byte,
    HalfWord,
    Word,
    DoubleWord
}

impl Size {
    pub fn in_bytes(self) -> u8 {
        match self {
            Self::Byte => 1,
            Self::HalfWord => 2,
            Self::Word => 4,
            Self::DoubleWord => 8
        }
    }

    pub fn in_bits(self) -> u8 {
        match self {
            Self::Byte => 8,
            Self::HalfWord => 16,
            Self::Word => 32,
            Self::DoubleWord => 64
        }
    }
}


pub trait EngineContext {
    fn schedule(&mut self, timestamp: u64, target: Target, payload: EventPayload);
}

pub trait Module { 
    fn process_event(&mut self, event: Event, engine_context: &mut dyn EngineContext);
}
