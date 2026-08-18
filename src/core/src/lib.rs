use crate::event::Event;

pub mod event;

pub type ModuleId = usize;

pub trait EngineContext {
    fn schedule(&mut self, event: Event);
}

pub trait Module { 
    fn process_event(&mut self, event: Event, engine_context: &mut dyn EngineContext);
}
