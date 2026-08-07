use harts::hart::{
    Extensions, 
    Hart, 
    HartError
};

use memory::{
    CacheController,
    CacheLevelConfig,
    CacheReplacementPolicy,
    CacheWritePolicy,
    MemError,
    MemoryRuntimeContext,
    Ram
};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, PartialEq, Eq)]
pub enum MachineError {
    InvalidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub hart_count: u8,
    pub extensions: Extensions,
    pub ram_size: usize,
    pub cache_config: Vec<CacheLevelConfig>
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            hart_count: 8,
            extensions: Extensions::default(),
            ram_size: 16384,
            cache_config: {
                let r = CacheReplacementPolicy::LRU;
                let w = CacheWritePolicy::WriteBack;
                vec![
                    CacheLevelConfig::new(4, 64, 4, 1, r, w),
                    CacheLevelConfig::new(16, 64, 8, 2, r, w),
                    CacheLevelConfig::new(64, 64, 16, 4, r, w)
                ]
            }
        }
    }
}

impl MachineConfig {
    pub fn is_valid(&self) -> bool {
        !self.cache_config.is_empty()
    }
}

#[derive(Debug)]
pub struct Machine {
    // TODO: had to rename because there is a crate with the same name 
    // harts: Vec<Hart>,
    installed_harts: Vec<Hart>,
    mem_ctx: MemoryRuntimeContext,
    // TODO: temporary flag to know when to stop
    done: bool
}

impl Machine {
    pub fn new(
        config: &MachineConfig, 
        asm: Vec<u8>
    ) -> Result<Self, MachineError>{
        if !config.is_valid() {
            return Err(MachineError::InvalidConfig)
        }

        let mut installed_harts = vec![
            Hart::from_extensions(&config.extensions); 
            config.hart_count as usize
        ];

        let mem_ctx = MemoryRuntimeContext::new(
            Ram::new(config.ram_size),
            match CacheController::new(&config.cache_config) {
                Ok(cfg) => Ok(cfg),
                Err(err) => Err(MachineError::InvalidConfig)
            }?
        );

        Ok(Self{
            installed_harts,
            mem_ctx,
            done: false
        })
    }

    /// Starts the simulation loop
    pub fn update(&mut self) -> Result<(), HartError> {
        self.done = self.installed_harts[0].update(&mut self.mem_ctx)?;
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

    pub fn done(&self) -> bool {
        self.done
    }
}
