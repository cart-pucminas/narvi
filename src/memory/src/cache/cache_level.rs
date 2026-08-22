use narvi_core::{
    CacheLevelConfig, 
    CacheReplacementPolicy, 
    CacheWritePolicy,
    EngineContext,
    Module, 
    ModuleId, 
    event::{
        Event, EventPayload, JournalEvent, Target
    }
};

use rand::{random_range};

use super::{
    CacheError, 
    CacheReturn
};

macro_rules! mask_from {
    ($size:expr, $offset:expr) => {
        ((1 << $size) - 1) << $offset  
    };
}

macro_rules! idx2to1 {
    ($i:expr, $j:expr, $cols:expr) => {
        $j + $i * $cols 
    };
}


#[derive(Debug, Clone)]
struct CacheLine {
    bytes: Vec<u8>
}

impl CacheLine {
    fn new(block_size: usize) -> Self {
        CacheLine { bytes: vec![0; block_size] }
    }

    fn update(&mut self, vec: Vec<u8>, offset: usize) -> Result<(), CacheError> {
        let len = std::cmp::min(self.bytes.len(), vec.len());

        if let Some(slice) = self.bytes.get_mut(offset..len+offset) {
            slice.copy_from_slice(&vec[..len]);
            return Ok(())
        } 
        Err(CacheError::OutOfBounds)
    }

    fn print(&self) {
        println!("Line: {:?}", self.bytes);
    }
}

#[derive(Debug, Clone)]
struct CacheSet {
    cache_lines: Vec<CacheLine>,
    policy: CacheReplacementPolicy,
    policy_list: Vec<usize>,    // List used for LRU, LFU and FIFO logic 
    way: usize,
}

impl CacheSet {
    fn new (
        block_size: usize, 
        set_size: usize,
        policy: CacheReplacementPolicy
    ) -> Self {

        let policy_list: Vec<usize> = if set_size == 1 {
                vec![]
            } else {
                match policy {
                    CacheReplacementPolicy::LRU => (0..set_size).rev().collect(),
                    CacheReplacementPolicy::LFU => vec![0; set_size],
                    CacheReplacementPolicy::FIFO => vec![],
                    CacheReplacementPolicy::Random => vec![],
                }
        };

        CacheSet {
            cache_lines: vec![CacheLine::new(block_size); set_size],
            policy,
            policy_list,
            way: set_size
        }
    }
    
    fn insert(
        &mut self, 
        data: Vec<u8>, 
        replace_i: usize
    ) -> Result<usize, CacheError> {
        self.cache_lines[replace_i].update(data, 0)?;
        self.policy_update(replace_i, true)?;
        Ok(replace_i)
    }

    fn update(
        &mut self, 
        data: Vec<u8>, 
        offset: usize, 
        i: usize
    ) -> Result<(), CacheError> {
        self.cache_lines[i].update(data, offset)?;
        self.policy_update(i, false)?;
        Ok(())
    }

    fn policy_next(&mut self) -> Result<usize, CacheError> {

        if self.way == 1 {
            return Ok(0);
        }

        match self.policy {
            CacheReplacementPolicy::LRU => { 
                match self.policy_list.last() {
                    Some(&res) => Ok(res),
                    None => Err(CacheError::PolicyFailed)
                }
            },
            CacheReplacementPolicy::LFU => {
                match self.policy_list
                          .iter()
                          .enumerate()
                          .min_by_key( |(_, val)| *val)
                          .map( |(index, _)| index) {
                    Some(min_idx) => Ok(min_idx),
                    None => Err(CacheError::PolicyFailed)
                }
            },
            CacheReplacementPolicy::FIFO => {
                match self.policy_list.pop() {
                    Some(res) => Ok(res),
                    None => {
                        if self.policy_list.len() != 0 {
                            Err(CacheError::PolicyFailed)
                        } else {
                            Ok(0)   // First insertion
                        }
                    }
                }
            },
            CacheReplacementPolicy::Random => Ok(random_range(0..self.way))
        }
    }

    fn policy_update(
        &mut self, 
        idx: usize, 
        insertion: bool
    ) -> Result<(), CacheError> {
        if self.way == 1 {
            return Ok(());
        }
        match self.policy {
            CacheReplacementPolicy::LRU => { 
                let old_pos = 
                    match self.policy_list.iter().position(|x| *x == idx) {
                        Some(pos) => pos,
                        None => return Err(CacheError::PolicyFailed)
                    };
                self.policy_list.remove(old_pos);
                self.policy_list.insert(0, idx);
            },
            CacheReplacementPolicy::LFU => {
                if insertion {  // Reset on new block
                    self.policy_list[idx] = 0;
                }
                self.policy_list[idx] += 1;
            },
            CacheReplacementPolicy::FIFO => {
                if insertion {
                    self.policy_list.insert(0, idx);
                }
            },
            CacheReplacementPolicy::Random => (),
        }
        Ok(())
    }

    pub fn print (&self) {
        for line in &self.cache_lines {
            line.print();
        }
        println!("policy list: {:?}", self.policy_list);
    }
}

#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
}

#[derive(Debug)]
enum PendingRequest {
    Load { requester: Target, size: usize },
    Store { data: Vec<u8> }
}

#[derive(Debug)]
pub struct CacheLevel {
    backing_store: Option<ModuleId>,

    pending_request: Option<(usize, PendingRequest)>,

    offset_mask: usize,
    index_mask: usize,
    tag_mask: usize,

    index_start: usize,
    tag_start: usize,

    sets: Vec<CacheSet>,
    tags: Vec<usize>,
    valid: Vec<bool>,
    dirty: Vec<bool>,

    way: usize,
    n_sets: usize,
    
    block_size: usize,

    stats: CacheStats,
    pub(super) write_policy: CacheWritePolicy,
}

impl Module for CacheLevel {
    fn process_event(&mut self, event: Event, engine_context: &mut dyn EngineContext) {
        match event.payload() {
            EventPayload::MemoryLoadReq { address, size_in_bytes, requester } => {
                match self.read(*address, *size_in_bytes as usize) {
                    Ok(CacheReturn::Hit(data)) => {
                        engine_context.schedule(1, *requester, EventPayload::MemoryLoadRes { data: data });
                        engine_context.record_journal(JournalEvent::CacheHit);
                    },
                    Ok(CacheReturn::Miss) | Err(_) => {
                        self.pending_request = Some((
                            *address, 
                            PendingRequest::Load {
                                requester: *requester,
                                size: *size_in_bytes
                            }
                        ));

                        let block_base_address = *address & (self.offset_mask ^ usize::MAX);

                        engine_context.schedule(
                            1,
                            Target::Module(self.backing_store.unwrap()),
                            EventPayload::MemoryLoadReq { 
                                address: block_base_address, 
                                size_in_bytes: self.block_size.div_ceil(8), 
                                requester: Target::Myself 
                            }
                        );

                        engine_context.record_journal(JournalEvent::CacheMiss);
                    }
                }
            },
            EventPayload::MemoryStoreReq { address, data } => {
                if let Ok(true) = self.find(*address) {
                    self.update(*address, data.clone()).expect("failed to update block");
                    
                    if matches!(self.write_policy, CacheWritePolicy::WriteThrough) {
                        engine_context.schedule(
                            1,
                            Target::Module(self.backing_store.unwrap()), 
                            EventPayload::MemoryStoreReq { 
                                address: *address,
                                data: data.clone()
                            }
                        );
                    }

                    engine_context.record_journal(JournalEvent::CacheHit);
                } else {
                    self.pending_request = Some((*address, PendingRequest::Store { data: data.clone() }));

                    let block_base_address = *address & (self.offset_mask ^ usize::MAX);

                    engine_context.schedule(
                        1,
                        Target::Module(self.backing_store.unwrap()), 
                        EventPayload::MemoryLoadReq { 
                            address: block_base_address, 
                            size_in_bytes: self.block_size.div_ceil(8),
                            requester: Target::Myself 
                        }
                    );

                    engine_context.record_journal(JournalEvent::CacheMiss);
                }
            },
            EventPayload::MemoryLoadRes { data } => {
                let (orig_addr, req_type) = self.pending_request.take()
                    .expect("received memory without a pending request");

                let insert_result = self.insert(orig_addr, data.clone()).expect("failed to insert block");

                if let Some((dirty_addr, dirty_bytes)) = insert_result {
                    if matches!(self.write_policy, CacheWritePolicy::WriteBack) {
                        engine_context.schedule(
                            1,
                            Target::Module(self.backing_store.unwrap()), 
                            EventPayload::MemoryStoreReq { 
                                address: dirty_addr, 
                                data: dirty_bytes, 
                            }
                        );
                    }
                }

                match req_type {
                    PendingRequest::Load { requester, size } => {
                        if let Ok(CacheReturn::Hit(requested_data)) = self.read(orig_addr, size) {
                            engine_context.schedule(
                                1,
                                requester,
                                EventPayload::MemoryLoadRes { data: requested_data }
                            );
                        }
                    },
                    PendingRequest::Store { data: store_data } => {
                        self.update(orig_addr, store_data.clone()).expect("failed to apply pending store");

                        if matches!(self.write_policy, CacheWritePolicy::WriteThrough) {
                            engine_context.schedule(
                                1,
                                Target::Module(self.backing_store.unwrap()),
                                EventPayload::MemoryStoreReq {
                                    address: orig_addr,
                                    data: store_data,
                                }
                            );
                        }
                    }
                }
            },
            EventPayload::Reset => {},
            _ => panic!("unable to process {event}")
        }
    }
}

impl From<&CacheLevelConfig> for CacheLevel {
    fn from(config: &CacheLevelConfig) -> Self {
        let offset_size = config.block_size.ilog2() as usize;
        let index_size = config.n_blocks.ilog2() as usize;

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        // remaining bits
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; 

        let n_sets = config.n_blocks / config.set_size;
        let base_set = CacheSet::new(
            config.block_size, 
            config.set_size, 
            config.replacement_policy
        );

        CacheLevel {
            backing_store: None,
            pending_request: None,
            index_mask, 
            tag_mask, 
            offset_mask,
            index_start: offset_size,
            tag_start: offset_size + index_size,
            sets: vec![base_set; n_sets],
            tags: vec![0; config.n_blocks],
            valid: vec![false ; config.n_blocks], 
            dirty: vec![false ; config.n_blocks], 
            way: config.set_size,
            n_sets,
            block_size: config.block_size,
            stats: Default::default(),
            write_policy: config.write_policy
        }
    }
}

impl CacheLevel {
    pub fn new(
        block_size: usize,
        associativity: usize,
        n_sets: usize,
        replacement_policy: CacheReplacementPolicy,
        write_policy: CacheWritePolicy
    ) -> Self {
        let n_blocks = n_sets * associativity;

        let offset_size = block_size.ilog2() as usize; // byte offset
        let index_size = n_sets.ilog2() as usize; 

        // Create masks
        let offset_mask = mask_from!(offset_size, 0);
        let index_mask = mask_from!(index_size, offset_size);
        // remaining bits
        let tag_mask = (offset_mask | index_mask) ^ usize::MAX; 

        let base_set = CacheSet::new(block_size, associativity, replacement_policy);

        CacheLevel {
            backing_store: None,
            pending_request: None,
            index_mask,
            tag_mask,
            offset_mask,
            index_start: offset_size,
            tag_start: index_size + offset_size,
            sets: vec![base_set; n_sets],
            tags: vec![0; n_blocks],
            valid: vec![false ; n_blocks], 
            dirty: vec![false ; n_blocks], 
            way: associativity,
            n_sets,
            block_size,
            stats: Default::default(),
            write_policy
        }
    }

    pub fn set_backing_store(&mut self, backing_store: ModuleId) {
        self.backing_store = Some(backing_store);
    }

    fn get_old (
        &mut self,
        index: usize,
        set_i: usize
    ) -> Option<(usize, Vec<u8>)> {
        let set = self.sets.get(index)?;
        let line = set.cache_lines.get(set_i)?;

        let old_block = line.bytes.clone();
        
        let tag = { 
            let block_idx = idx2to1!(index, set_i, self.way);
            self.tags.get(block_idx)? << self.tag_start
        };

        let old_addr = tag | (index << self.index_start);

        Some((old_addr, old_block))
    }

    // Used for new blocks. Does not set the dirty bit
    pub fn insert(
        &mut self, 
        addr: usize,
        data: Vec<u8>
    ) -> Result<Option<(usize, Vec<u8>)>, CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        match self.sets.get(idx) {
            Some(_) => {
                let tag = (addr & self.tag_mask) >> self.tag_start;

                let i = match self.sets[idx].policy_next() {
                    Ok(sub) => sub,
                    Err(err) => { println!("{:?}. Defaulting to 0", err); 0 }
                };

                let mut res: Option<(usize, Vec<u8>)> = None;

                let block_idx = idx2to1!(idx, i, self.way);

                if self.valid[block_idx] {
                    self.stats.evictions += 1;

                    if self.dirty[block_idx] {
                        res = Some(self.get_old(idx, i)
                            .ok_or(CacheError::OutOfBounds)?);
                        self.dirty[block_idx] = false;
                    }
                }

                self.sets[idx].insert(data, i)?;

                self.tags[block_idx] = tag;
                self.valid[block_idx] = true;

                Ok(res)
            },
            _ => Err(CacheError::OutOfBounds)
        }
    }

    // Used for blocks that are already in the cache. Sets the dirty bit
    pub fn update(
        &mut self,
        addr: usize,
        data: Vec<u8>
    ) -> Result<(), CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;

        match self.sets.get(idx) {
            Some(_) => {
                let tag = (addr & self.tag_mask) >> self.tag_start;
                let offset = addr & self.offset_mask;

                for i in 0..self.way {
                    let block_idx = idx2to1!(idx, i, self.way);

                    let control_bits = (self.valid.get(block_idx), self.tags.get(block_idx));

                    match control_bits {
                        (Some(&true), Some(&value)) if value == tag => {
                            self.sets[idx].update(data, offset, i)?;
                            self.dirty[block_idx] = true;
                            return Ok(());
                        },
                        (None, _) | (_, None) => 
                            return Err(CacheError::OutOfBounds),
                        _ => (),
                    }
                }
                // Updated always comes after a find
                Err(CacheError::UnreachableState) 
            },
            None => Err(CacheError::OutOfBounds)
        }
    }

    // Returns an amount of bytes in this cache level
    pub fn read(
        &mut self, 
        addr: usize,
        bytes: usize
    ) -> Result<CacheReturn, CacheError> {
        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;
        let tag = (addr & self.tag_mask) >> self.tag_start;
        
        for i in 0..self.way {
            let control_bits = {
                let block_idx = idx2to1!(idx, i, self.way);
                (self.valid.get(block_idx), self.tags.get(block_idx))
            };

            match control_bits {
                (Some(&true), Some(&value)) if value == tag => {
                    let offset = addr & self.offset_mask;
                    let block = self.sets[idx].cache_lines[i].clone();
                    let slice = &block.bytes[offset..offset + bytes];

                    self.sets[idx].policy_update(i, false)?;
                    return Ok(CacheReturn::Hit(slice.to_vec()))
                },
                (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                _ => (),
            }
        }
        Ok(CacheReturn::Miss)
    }

    pub fn get_block(
        &mut self,
        addr: usize
    ) -> Result<CacheReturn, CacheError> {
        self.read(addr, self.block_size)
    }

    pub fn find(&mut self, addr: usize) -> Result<bool, CacheError> {

        let tmp = (addr & self.index_mask) >> self.index_start;
        let idx = tmp % self.n_sets;
        let tag = (addr & self.tag_mask) >> self.tag_start;

        
        for i in 0..self.way {
            let control_bits = {
                let block_idx = idx2to1!(idx, i, self.way);
                (self.valid.get(block_idx), self.tags.get(block_idx))
            };

            match control_bits {
                (Some(&true), Some(&value)) if value == tag => {
                    self.stats.hits += 1;
                    self.sets[idx].policy_update(i, false)?;
                    return Ok(true);
                },
                (None, _) | (_, None) => return Err(CacheError::OutOfBounds),
                _ => (),
            }
        }

        self.stats.misses += 1;
        Ok(false)
    }

    pub fn print(&self) {
        let mut i = 0;
        for set in &self.sets {
            println!("\n=> set {}", i);
            set.print();
            i+=1;
        }
    }

    pub fn stats(&self) -> CacheStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn line_update_success() {
        let mut cache_line = CacheLine::new(4);
        cache_line.update(
            vec![0xEE, 0xFF],
            1
        ).unwrap();
        let mut iterator = cache_line.bytes.iter();
        assert_eq!(iterator.next(), Some(&0x00u8));
        assert_eq!(iterator.next(), Some(&0xEEu8));
        assert_eq!(iterator.next(), Some(&0xFFu8));
        assert_eq!(iterator.next(), Some(&0x00u8));
    }

    #[test]
    #[should_panic(expected = "OutOfBounds")]
    fn line_update_fail() {
        let mut cache_line = CacheLine::new(4);
        cache_line.update(
            vec![0xEE, 0xFF],
            3
        ).unwrap();
    }

    #[test]
    fn set_lru() {
        let mut cache_set = CacheSet::new(4, 4, CacheReplacementPolicy::LRU);
        cache_set.update(vec![0], 0, 1).unwrap(); 
        cache_set.update(vec![0], 0, 3).unwrap(); 
        cache_set.update(vec![0], 0, 1).unwrap(); 
        cache_set.update(vec![0], 0, 0).unwrap(); 
        cache_set.update(vec![0], 0, 2).unwrap(); 
        assert_eq!(cache_set.policy_next(), Ok(3usize));
    }

    #[test]
    fn set_lfu() {
        let mut cache_set = CacheSet::new(4, 4, CacheReplacementPolicy::LFU);
        cache_set.update(vec![0], 0, 2).unwrap(); 
        cache_set.update(vec![0], 0, 1).unwrap(); 
        cache_set.update(vec![0], 0, 3).unwrap(); 
        cache_set.update(vec![0], 0, 3).unwrap(); 
        cache_set.update(vec![0], 0, 0).unwrap(); 
        cache_set.update(vec![0], 0, 1).unwrap(); 
        cache_set.update(vec![0], 0, 3).unwrap(); 
        cache_set.update(vec![0], 0, 1).unwrap(); 
        cache_set.update(vec![0], 0, 2).unwrap(); 
        assert_eq!(cache_set.policy_next(), Ok(0usize));
    }

    #[test]
    fn set_fifo() {
        let mut cache_set = CacheSet::new(4, 4, CacheReplacementPolicy::FIFO);
        cache_set.insert(vec![0], 1).unwrap(); 
        cache_set.insert(vec![0], 3).unwrap(); 
        cache_set.insert(vec![0], 0).unwrap(); 
        cache_set.insert(vec![0], 2).unwrap(); 
        assert_eq!(cache_set.policy_next(), Ok(1usize));
    }

    #[test]
    fn level_lru_eviction() {
        let mut cache_level = 
            CacheLevel::new(64, 2, 1, CacheReplacementPolicy::LRU, CacheWritePolicy::WriteThrough);

        let _ = cache_level.find(0x00);
        assert_eq!(cache_level.stats.misses, 1);
        let _ = cache_level.insert(0x00, vec![0; 64]);

        let _ = cache_level.find(0x40);
        assert_eq!(cache_level.stats.misses, 2);
        let _ = cache_level.insert(0x40, vec![0; 64]);

        assert_eq!(cache_level.stats.evictions, 0);

        let _ = cache_level.find(0x80);
        assert_eq!(cache_level.stats.misses, 3);
        let _ = cache_level.insert(0x80, vec![0; 64]);

        assert_eq!(cache_level.stats.evictions, 1);
    }
}
