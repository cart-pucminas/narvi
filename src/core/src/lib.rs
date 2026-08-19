use crate::event::{Event, EventPayload};

pub mod event;

pub type ModuleId = usize;

pub enum Target {
    Module(ModuleId),
    Myself
}

pub trait EngineContext {
    fn schedule(&mut self, timestamp: u64, target: Target, payload: EventPayload);
}

pub trait Module { 
    fn process_event(&mut self, event: Event, engine_context: &mut dyn EngineContext);
}
