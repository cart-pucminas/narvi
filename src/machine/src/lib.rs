use hart::{extensions::Extensions, Hart};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub hart_count: usize,
    pub extensions: Extensions,
    pub l1_size: usize,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            hart_count: 1,
            extensions: Extensions::default(),
            l1_size: 0xFFFF,
        }
    }
}

#[derive(Debug)]
pub struct Machine {
    harts: Vec<Hart>,
}

impl Machine {
    pub fn new(config: &MachineConfig) -> Self {
        let harts = (0..config.hart_count)
            .map(|_| Hart::from_extensions(&config.extensions, config.l1_size))
            .collect();

        Self { harts }
    }

    pub fn harts(&self) -> &[Hart] {
        &self.harts
    }
}
