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
    de,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use std::fs::File;
use std::io::Write;

#[derive(Debug, PartialEq, Eq)]
pub enum MachineError {
    InvalidConfig,
}

#[derive(Debug, Clone)]
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

#[derive(Serialize, Deserialize)]
struct MachineConfigData {
    hart_count: u8,
    extensions: ExtensionsData,
    ram_size: usize,
    cache_config: Vec<CacheLevelConfigData>,
}

#[derive(Serialize, Deserialize)]
struct ExtensionsData {
    m: bool,
    a: bool,
    c: bool,
    f: bool,
    d: bool,
}

#[derive(Serialize, Deserialize)]
struct CacheLevelConfigData {
    n_blocks: usize,
    block_size: usize,
    set_size: usize,
    share_config: u8,
    replacement_policy: CacheReplacementPolicyData,
    write_policy: CacheWritePolicyData,
}

#[derive(Serialize, Deserialize)]
enum CacheReplacementPolicyData {
    LRU,
    LFU,
    FIFO,
    Random,
}

#[derive(Serialize, Deserialize)]
enum CacheWritePolicyData {
    WriteThrough,
    WriteBack,
}

impl From<&MachineConfig> for MachineConfigData {
    fn from(config: &MachineConfig) -> Self {
        Self {
            hart_count: config.hart_count,
            extensions: ExtensionsData::from(&config.extensions),
            ram_size: config.ram_size,
            cache_config: config.cache_config.iter()
                .map(CacheLevelConfigData::from)
                .collect(),
        }
    }
}

impl From<MachineConfigData> for MachineConfig {
    fn from(data: MachineConfigData) -> Self {
        Self {
            hart_count: data.hart_count,
            extensions: Extensions::from(data.extensions),
            ram_size: data.ram_size,
            cache_config: data.cache_config.into_iter()
                .map(CacheLevelConfig::from)
                .collect(),
        }
    }
}

impl From<&Extensions> for ExtensionsData {
    fn from(extensions: &Extensions) -> Self {
        Self {
            m: extensions.m,
            a: extensions.a,
            c: extensions.c,
            f: extensions.f,
            d: extensions.d,
        }
    }
}

impl From<ExtensionsData> for Extensions {
    fn from(data: ExtensionsData) -> Self {
        Self {
            m: data.m,
            a: data.a,
            c: data.c,
            f: data.f,
            d: data.d,
        }
    }
}

impl From<&CacheLevelConfig> for CacheLevelConfigData {
    fn from(config: &CacheLevelConfig) -> Self {
        Self {
            n_blocks: config.n_blocks,
            block_size: config.block_size,
            set_size: config.set_size,
            share_config: config.share_config,
            replacement_policy: CacheReplacementPolicyData::from(config.replacement_policy),
            write_policy: CacheWritePolicyData::from(config.write_policy),
        }
    }
}

impl From<CacheLevelConfigData> for CacheLevelConfig {
    fn from(data: CacheLevelConfigData) -> Self {
        Self::new(
            data.n_blocks,
            data.block_size,
            data.set_size,
            data.share_config,
            CacheReplacementPolicy::from(data.replacement_policy),
            CacheWritePolicy::from(data.write_policy),
        )
    }
}

impl From<CacheReplacementPolicy> for CacheReplacementPolicyData {
    fn from(policy: CacheReplacementPolicy) -> Self {
        match policy {
            CacheReplacementPolicy::LRU => Self::LRU,
            CacheReplacementPolicy::LFU => Self::LFU,
            CacheReplacementPolicy::FIFO => Self::FIFO,
            CacheReplacementPolicy::Random => Self::Random,
        }
    }
}

impl From<CacheReplacementPolicyData> for CacheReplacementPolicy {
    fn from(policy: CacheReplacementPolicyData) -> Self {
        match policy {
            CacheReplacementPolicyData::LRU => Self::LRU,
            CacheReplacementPolicyData::LFU => Self::LFU,
            CacheReplacementPolicyData::FIFO => Self::FIFO,
            CacheReplacementPolicyData::Random => Self::Random,
        }
    }
}

impl From<CacheWritePolicy> for CacheWritePolicyData {
    fn from(policy: CacheWritePolicy) -> Self {
        match policy {
            CacheWritePolicy::WriteThrough => Self::WriteThrough,
            CacheWritePolicy::WriteBack => Self::WriteBack,
        }
    }
}

impl From<CacheWritePolicyData> for CacheWritePolicy {
    fn from(policy: CacheWritePolicyData) -> Self {
        match policy {
            CacheWritePolicyData::WriteThrough => Self::WriteThrough,
            CacheWritePolicyData::WriteBack => Self::WriteBack,
        }
    }
}

impl Serialize for MachineConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MachineConfigData::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MachineConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = MachineConfigData::deserialize(deserializer)?;
        let config = MachineConfig::from(data);
        if config.is_valid() {
            Ok(config)
        } else {
            Err(de::Error::custom("machine config must include at least one cache level"))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_config_round_trips_through_yaml() {
        let config = MachineConfig::default();

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: MachineConfig = serde_yaml::from_str(&yaml).unwrap();

        let mut fil = File::create("test.yaml").expect("Could not open file");
        write!(fil, "{}", yaml);

        assert_eq!(deserialized.hart_count, config.hart_count);
        assert_eq!(deserialized.extensions, config.extensions);
        assert_eq!(deserialized.ram_size, config.ram_size);
        assert_eq!(deserialized.cache_config.len(), config.cache_config.len());

        for (actual, expected) in deserialized.cache_config.iter().zip(config.cache_config.iter()) {
            assert_eq!(actual.n_blocks, expected.n_blocks);
            assert_eq!(actual.block_size, expected.block_size);
            assert_eq!(actual.set_size, expected.set_size);
            assert_eq!(actual.share_config, expected.share_config);
            assert!(matches!(
                (actual.replacement_policy, expected.replacement_policy),
                (CacheReplacementPolicy::LRU, CacheReplacementPolicy::LRU)
                    | (CacheReplacementPolicy::LFU, CacheReplacementPolicy::LFU)
                    | (CacheReplacementPolicy::FIFO, CacheReplacementPolicy::FIFO)
                    | (CacheReplacementPolicy::Random, CacheReplacementPolicy::Random)
            ));
            assert!(matches!(
                (actual.write_policy, expected.write_policy),
                (CacheWritePolicy::WriteThrough, CacheWritePolicy::WriteThrough)
                    | (CacheWritePolicy::WriteBack, CacheWritePolicy::WriteBack)
            ));
        }

        Machine::new(&deserialized, Vec::new()).unwrap();
    }
}
