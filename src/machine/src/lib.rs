use harts::hart::{extensions::Extensions, Hart};
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
    harts: Vec<Hart>,
    l1: (),
    l2: (),
    l3: (),
    l4: (),
    l1_share: u8,
    l2_share: u8,
    l3_share: u8,
    l4_share: u8,
}

impl Machine {

    pub fn new(config: &MachineConfig) -> Result<Self, MachineError>{
        if !config.is_valid() {
            Err(MachineError::InvalidConfig)
        } else {
            let l1 = ();
            let l2 = ();
            let l3 = ();
            let l4 = ();
            // let harts = vec![Hart::from_extensions(&config.extensions, 0xFFFF); config.hart_count as usize];
            let harts: Vec<Hart> = Vec::new(); 
            Ok(Self{
                harts,
                l1, l2, l3, l4,
                l1_share: config.l1_share, 
                l2_share: config.l2_share, 
                l3_share: config.l3_share, 
                l4_share: config.l4_share, 
            })
        }
    }

}
