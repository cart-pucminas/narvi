use serde::Deserialize;

use crate::{
    Ram, 
    ram::RamError,
    cache_config::CacheError,
    cache_config::CacheReturn,
    cache_level::CacheLevel
};

#[derive(Debug)]
pub enum MemError {
    Ram(RamError),
    Cache(CacheError),
    UnreachableState,
    UpdateFailed,
}

#[derive(Debug, Deserialize)]
pub struct CacheController {
    cache_levels: Vec<CacheLevel>,
    #[serde(skip)]
    ram: Ram
}

impl CacheController {
    pub fn new (cache_levels: Vec<CacheLevel>, ram: Ram) -> Self {
        CacheController {
            cache_levels,
            ram
        }
    }

    fn read_bytes_ram (&self, addr: usize, bytes: usize) -> Result<Vec<u8>, MemError> {
        
        match bytes {
            1 => match self.ram.read_8(addr) {
                Ok(val) => Ok(val.to_le_bytes().to_vec()),
                Err(err) => Err(MemError::Ram(err))
            }
            2 => match self.ram.read_16(addr) {
                Ok(val) => Ok(val.to_le_bytes().to_vec()),
                Err(err) => Err(MemError::Ram(err))
            }
            4 => match self.ram.read_32(addr) {
                Ok(val) => Ok(val.to_le_bytes().to_vec()),
                Err(err) => Err(MemError::Ram(err))
            }
            8 => match self.ram.read_64(addr) {
                Ok(val) => Ok(val.to_le_bytes().to_vec()),
                Err(err) => Err(MemError::Ram(err))
            }
            _ => Err(MemError::UnreachableState)
        }
    }

    fn bring_upwards (&mut self,
            addr: usize,
            data: Vec<u8>,
            bytes: usize,
            mut level: usize,
            read_ram: bool
        ) -> Result<(), MemError> {

        let mut res: Vec<u8> = data;

        if read_ram {
            match self.read_bytes_ram(addr, bytes) {
                Ok(val) => res = val,
                Err(err) => return Err(err)
            }
        }

        while level > 0 {
            level -= 1;
            self.cache_levels[level as usize].new_block(addr, res.clone());
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

        let mut res = vec![];

        let hit;
        let level;

        match self.find(addr) {
            Ok(tuple) => {
                (hit, level) = tuple;
                if hit {
                    res = match self.cache_levels[level as usize].read(addr, bytes) {
                        Ok(ret) => match ret {
                            CacheReturn::Hit(val) => val,
                            CacheReturn::Miss => return Err(MemError::UnreachableState)
                        },
                        Err(err) => return Err(MemError::Cache(err))
                    };
                }
            }
            Err(err) => return Err(err)
        };

        // Update upper levels
        if level != 0 {
            match self.bring_upwards(addr, res, bytes, level, !hit) {
                Ok(_) => (),
                Err(err) => return Err(err)
            }

            // Read first level again
            match self.cache_levels[0].read(addr, bytes) {
                Ok(ret) => match ret {
                    CacheReturn::Hit(val) => return Ok(val),
                    _ => return Err(MemError::UpdateFailed)
                }
                Err(err) => return Err(MemError::Cache(err))
            };
        }
        Ok(res)
    }

    fn write (&mut self, addr: usize, data: Vec<u8>) -> Result<(), MemError> {
        
        // In first layer cache
        if let Ok(true) = self.cache_levels[0].find(addr) {
            
        } else if let Ok(x) = self.find(addr) {     // Bring to first layer
            
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

    pub fn print(&self) {
        println!("Ram: \n{:?}", self.ram);
        println!("==========================================");
        println!("Cache: ");
        
        let mut i = 0;
        for c in &self.cache_levels {
            println!("Level {i} -");
            c.print();
            i += 1;
        }
    }
}
