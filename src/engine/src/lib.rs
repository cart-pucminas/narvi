use std::collections::{
    BinaryHeap,
    HashMap
};

use journal::{CacheJournal, HartJournal, Journal};

use memory::{CacheLevel, Ram};
use narvi_core::{
    EngineContext, Module, ModuleId, event::{Event, EventPayload, JournalEvent, Target}, serialization::MachineConfig
};

use harts::hart::Hart;

trait ProxyResolver {
    fn resolve_requester(self, id: ModuleId) -> Self;
}

impl ProxyResolver for EventPayload {
    fn resolve_requester(mut self, id: ModuleId) -> Self {
        match &mut self {
            EventPayload::MemoryLoadReq { requester, .. } => {
                if matches!(Target::Myself, requester) {
                    *requester = Target::Module(id);
                }
            },
            _ => {}
        }

        self
    }
}

pub struct ActiveContext<'a> {
    current_time: u64,
    current_module_id: ModuleId,
    event_queue: &'a mut BinaryHeap<Event>,
    cache_level_map: &'a mut HashMap<ModuleId, usize>,
    journal: &'a mut Journal
}

impl<'a> EngineContext for ActiveContext<'a> {
    fn schedule(&mut self, delay: u64, target: Target, payload: EventPayload) {
        let actual_target = match target {
            Target::Myself => self.current_module_id,
            Target::Module(id) => id
        };

        let actual_payload = payload.resolve_requester(self.current_module_id);

        self.event_queue.push(Event::new(
            self.current_time + delay, 
            actual_target, 
            actual_payload
        ));
    }

    fn record_journal(&mut self, event: narvi_core::event::JournalEvent) {
        // TODO: no need for HartJournal and CacheJournal
        // keeping for now because I'm lazy

        let cache_level = self.cache_level_map.get(&self.current_module_id)
            .expect("could not find level");

        let mut hart_journal = HartJournal::new();
        let mut cache_journal = CacheJournal::new(self.cache_level_map.len());

        match event {
            JournalEvent::CacheHit => {
                cache_journal.hit(*cache_level);
            },
            JournalEvent::CacheMiss => {
                cache_journal.miss(*cache_level);
            },
            JournalEvent::Cycles { cycles } => {
                hart_journal.cycles_done(cycles as u128);
            },
            JournalEvent::CyclesLost { cycles } => {
                hart_journal.lost_cycle(cycles as u128);
            },
            JournalEvent::HartInstruction => {
                hart_journal.inst_done(1);
            }
        }

        self.journal.merge_cache(cache_journal);
        self.journal.merge_hart(hart_journal);
    }
}

pub struct Engine {
    modules : Vec<Box<dyn Module>>,
    event_queue: BinaryHeap<Event>,
    time: u64,
    cache_level_map: HashMap<ModuleId, usize>,
    pub journal: Journal,

}

impl Engine {
    pub fn build_from_config(config: MachineConfig) -> Self {
        let mut modules: Vec<Box<dyn Module>> = Vec::new();
        let mut cache_level_map: HashMap<ModuleId, usize> = HashMap::new();

        for _ in 0..config.hart_count {
            let mut hart = Hart::from_extensions(&config.extensions);
            modules.push(Box::new(hart));
        }

        let ram = Ram::new(config.ram_size);
        let ram_id = modules.len();
        modules.push(Box::new(ram));

        let mut previous_store_id = ram_id;

        for (level, cache_conf) in config.cache_config.into_iter().rev().enumerate() {
            let mut cache = CacheLevel::from(cache_conf);
            cache.set_backing_store(previous_store_id);
            previous_store_id = modules.len();
            modules.push(Box::new(cache));
        }

        let cache_levels = cache_level_map.len();

        Self {
            modules,
            event_queue: Default::default(),
            time: 0,
            cache_level_map,
            journal: Journal::new(cache_levels)
        }
    }

    pub fn update(&mut self) -> bool {
        if let Some(event) = self.event_queue.pop() {
            self.time = event.timestamp();
            
            let target_id = event.target();

            let mut ctx = ActiveContext {
                current_time: self.time,
                current_module_id: target_id,
                event_queue: &mut self.event_queue,
                cache_level_map: &mut self.cache_level_map,
                journal: &mut self.journal
            };

            if let Some(module) = self.modules.get_mut(target_id) {
                module.process_event(event, &mut ctx);
            }

            true
        } else {
            false
        }
    }
}
