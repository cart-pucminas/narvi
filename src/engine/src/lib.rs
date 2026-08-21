use std::{collections::BinaryHeap, default};
use narvi_core::{
    event::Event,
    Module
};

use harts::hart::Hart;

pub struct Engine {
    modules : Vec<Box<dyn Module>>,
    event_queue: BinaryHeap<Event>,
    time: u64
}

impl Engine {
    pub fn new() -> Self {
        Self {
            modules: Default::default(),
            event_queue: Default::default(),
            time: 0
        }
    }
    
    fn register_module(&mut self, module: Box<dyn Module>) -> usize {
        let id = self.modules.len();
        self.modules.push(module);
        id
    }

    pub fn schedule(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub fn update(&mut self) -> Option<Event> {
        if let Some(event) = self.event_queue.pop() {

            self.time = event.timestamp();

            Some(event)
        } else {
            None
        }
    }
}

