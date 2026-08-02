use harts::hart::{Extensions, Hart, HartError};
use memory::CacheL1;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum MachineError {
    InvalidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub hart_count: u8,
    pub extensions: Extensions,
    pub l1_share: u8,
    pub l1_size: usize,
    pub l2_share: u8,
    pub l2_size: usize,
    pub l3_share: u8,
    pub l3_size: usize,
    pub l4_share: u8,
    pub l4_size: usize,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            hart_count: 8,
            extensions: Extensions::default(),
            l1_share: 1,
            l1_size: 0xFFFF,
            l2_share: 2,
            l2_size: 0xFFFF,
            l3_share: 4,
            l3_size: 0xFFFF,
            l4_share: 0,
            l4_size: 0,
        }
    }
}

impl MachineConfig {
    pub fn is_valid(&self) -> bool {
        if self.hart_count.is_multiple_of(self.l1_share) {
           return false;
        }
        if self.l1_size == 0 {
            return false;
        }
        if self.l2_share != 0 && self.hart_count.is_multiple_of(self.l2_share) {
           return false;
        }
        if self.l3_share != 0 && self.hart_count.is_multiple_of(self.l3_share) {
           return false;
        }
        if self.l4_share != 0 && self.hart_count.is_multiple_of(self.l4_share) {
           return false;
        }
        true
    }
}

#[derive(Debug)]
pub struct Machine {
    // TODO: had to rename because there is a crate with the same name 
    // harts: Vec<Hart>,
    installed_harts: Vec<Hart>,
    // TODO: caches defined here but only some must be shared between harts
    l1_share: u8,
    l2_share: u8,
    l3_share: u8,
    l4_share: u8,
    // TODO: temporary flag to know when to stop
    done: bool
}

impl Machine {
    pub fn new(config: &MachineConfig, asm: Vec<u8>) -> Result<Self, MachineError>{
        if !config.is_valid() {
            println!("machine config is invalid, but proceeding for now anyways");
            //Err(MachineError::InvalidConfig)
        }

        let mut installed_harts = vec![Hart::from_extensions(&config.extensions, 0xFF); config.hart_count as usize];

        // TODO: overwriting entire l1 here for quick program loading, should be removed
        installed_harts[0].overwrite_l1(asm);

        Ok(Self{
            installed_harts,
            l1_share: config.l1_share, 
            l2_share: config.l2_share, 
            l3_share: config.l3_share, 
            l4_share: config.l4_share, 
            done: false
        })
    }

    /// Starts the simulation loop
    pub fn update(&mut self) -> Result<(), HartError> {
        self.done = self.installed_harts[0].update()?;
        Ok(())
    }

    // intended for visualization and debugging purposes only
    // should be safe since it returns an immutable borrow
    pub fn get_hart(&self, hart_id: usize) -> &Hart {
        if hart_id > 0 {
            todo!("multiple harts unsupported, use id 0");
        }

        &self.installed_harts[hart_id]
    }

    pub fn dump_l1(&self) -> Vec<u8> {
        self.installed_harts[0].l1_snapshot()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}
