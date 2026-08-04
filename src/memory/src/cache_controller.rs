use serde::Deserialize;

use crate::{
    Ram, 
    ram::RamError,
    cache_config::CacheError,
    cache_config::CacheReturn,
    cache_level::CacheLevel
};

macro_rules! to_byte_vec {
    ($bytes:expr) => {
        $bytes.to_le_bytes().to_vec()
    };
}

#[derive(Debug)]
pub enum MemError {
    Ram(RamError),
    Cache(CacheError),
    UnreachableState,
    UpdateFailed,
    MismatchedSizes,
}

#[derive(Debug, Deserialize)]
pub struct CacheController {
    cache_levels: Vec<CacheLevel>,

    #[serde(skip)]
    ram: Ram,

    #[serde(skip)]
    block_size: usize
}

impl CacheController {
    pub fn new (cache_levels: Vec<CacheLevel>, ram: Ram, block_size: usize) -> Self {
        CacheController {
            cache_levels,
            block_size,
            ram
        }
    }

    fn write_back (&mut self, addr: usize, mut level: usize, dirty_block: Vec<u8>) -> Result<(), MemError> {

        level += 1;
        if level < self.cache_levels.len() {
            self.cache_levels[(level) as usize].update(addr, dirty_block)
            .map_err(MemError::Cache)?;
        } else {
            self.ram.write_bytes(addr, dirty_block)
            .map_err(MemError::Ram)?;
        }
        Ok(())
    }

    fn bring_upwards (&mut self, addr: usize, mut level: usize, read_ram: bool) -> Result<(), MemError> {

        let mut block = vec![];

        if read_ram {
            match self.ram.read_bytes(addr, self.block_size) {
                Ok(val) => block = val,
                Err(err) => return Err(MemError::Ram(err))
            }
        } else {
            block = match self.cache_levels[level].get_block(addr)
            .map_err(MemError::Cache)? { 
                CacheReturn::Hit(val) => val,
                CacheReturn::Miss => return Err(MemError::UnreachableState) // Value was found
            }
        }

        while level > 0 {
            level -= 1;
            let opt = self.cache_levels[level as usize].insert(addr, block.clone())
                      .map_err(MemError::Cache)?;

            if let Some((dirty_addr, dirty_block)) = opt {
                self.write_back(dirty_addr, level, dirty_block)?;
            }
        }
        Ok(())
    }

    fn find (&mut self, addr: usize) -> Result<(bool, usize), MemError> {

        let n = self.cache_levels.len();
        let mut level: i32 = 0;
        let mut hit = false;

        // Search levels
        while !hit && (level as usize) < n {
            if let Ok(true) = self.cache_levels[level as usize].find(addr) {
                hit = true;
            } else {
                level += 1;
            }
        }
        Ok((hit, level as usize))
    }

    fn read (&mut self, addr: usize, bytes: usize) -> Result<Vec<u8>, MemError> {

        let (hit, level) = self.find(addr)?;

        // Update upper levels if necessary
        if level != 0 {
            self.bring_upwards(addr, level, !hit)?;
        }

        // Read first level
        match self.cache_levels[0].read(addr, bytes)
        .map_err(MemError::Cache)? {
            CacheReturn::Hit(val) => Ok(val),
            CacheReturn::Miss => return Err(MemError::UpdateFailed)
        }
    }

    fn write (&mut self, addr: usize, data: Vec<u8>) -> Result<(), MemError> {
        
        if let Ok(true) = self.cache_levels[0].find(addr) { // In the first cache layer
            self.cache_levels[0].update(addr, data)
            .map_err(MemError::Cache)?;

        } else if let Ok((in_cache, lvl)) = self.find(addr) {
            self.bring_upwards(addr, lvl, !in_cache)?;
            self.cache_levels[0].update(addr, data)
            .map_err(MemError::Cache)?;
        }
        Ok(())
    }

    pub fn read_8 (&mut self, addr: usize) -> Result<u8, MemError> {
        match self.read(addr, 1) {
            Ok(val) => Ok(val[0]),
            Err(err) => Err(err)
        }
    }
    
    pub fn read_16 (&mut self, addr: usize) -> Result<u16, MemError> {
        match self.read(addr, 2) {
            Ok(val) => {
                let mut tmp = (val[1] as u16) << 8;
                tmp |= val[0] as u16;

                Ok(tmp)
            }
            Err(err) => Err(err)
        }
    }

    pub fn read_32 (&mut self, addr: usize) -> Result<u32, MemError> {
        match self.read(addr, 4) {
            Ok(val) => {
                let mut tmp = (val[3] as u32) << 24;
                tmp |= (val[2] as u32) << 16;
                tmp |= (val[1] as u32) << 8;
                tmp |= val[0] as u32;

                Ok(tmp)
            }
            Err(err) => Err(err)
        }
    }

    pub fn read_64 (&mut self, addr: usize) -> Result<u64, MemError> {
        match self.read(addr, 8) {
            Ok(val) => {
                let mut tmp = (val[7] as u64) << 56;
                tmp |= (val[6] as u64) << 48;
                tmp |= (val[5] as u64) << 40;
                tmp |= (val[4] as u64) << 32;
                tmp |= (val[3] as u64) << 24;
                tmp |= (val[2] as u64) << 16;
                tmp |= (val[1] as u64) << 8;
                tmp |= val[0] as u64;

                Ok(tmp)
            },
            Err(err) => Err(err)
        }
    }

    pub fn write_8 (&mut self, addr: usize, data: u8) -> Result<(), MemError> {
        self.write(addr, to_byte_vec!(data))
    }

    pub fn write_16 (&mut self, addr: usize, data: u16) -> Result<(), MemError> {
        self.write(addr, to_byte_vec!(data))
    }

    pub fn write_32 (&mut self, addr: usize, data: u32) -> Result<(), MemError> {
        self.write(addr, to_byte_vec!(data))
    }

    pub fn write_64 (&mut self, addr: usize, data: u64) -> Result<(), MemError> {
        self.write(addr, to_byte_vec!(data))
    }

    pub fn print(&self) {
        println!("Cache: ");
        
        let mut i = 0;
        for c in &self.cache_levels {
            println!("\nLevel {i} -");
            c.print();
            i += 1;
        }
        println!("==========================================");
        println!("\nRam: \n{:?}", self.ram);
        println!("==========================================");
    }
}
