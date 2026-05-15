pub enum RamError {
    OutOfBounds
}

/// Simple byte-addressable little-endian RAM implementation
pub struct Ram(Vec<u8>);

impl Ram {
    pub fn with_size(size: usize) -> Self {
        Ram(vec![0; size])
    }

    pub fn read_8(&self, addr: usize) -> Result<u8, RamError> {
        if let Some(v) = self.0.get(addr).copied() {
            return Ok(v);
        }

        Err(RamError::OutOfBounds)
    }

    pub fn read_16(&self, addr: usize) -> Result<u16, RamError> {
        let byte0 = (self.read_8(addr)?) as u16;
        let byte1 = (self.read_8(addr + 1)?) as u16;
        
        Ok(byte0 | (byte1 << 8))
    }

    pub fn read_32(&self, addr: usize) -> Result<u32, RamError> {
        let byte0 = (self.read_8(addr)?) as u32;
        let byte1 = (self.read_8(addr + 1)?) as u32;
        let byte2 = (self.read_8(addr + 2)?) as u32;
        let byte3 = (self.read_8(addr + 3)?) as u32;
        
        Ok(byte0 | (byte1 << 8) | (byte2 << 16) | (byte3 << 24))
    }

    pub fn read_64(&self, addr: usize) -> Result<u64, RamError> {
        let byte0 = (self.read_8(addr)?) as u64;
        let byte1 = (self.read_8(addr + 1)?) as u64;
        let byte2 = (self.read_8(addr + 2)?) as u64;
        let byte3 = (self.read_8(addr + 3)?) as u64;
        let byte4 = (self.read_8(addr + 4)?) as u64;
        let byte5 = (self.read_8(addr + 5)?) as u64;
        let byte6 = (self.read_8(addr + 6)?) as u64;
        let byte7 = (self.read_8(addr + 7)?) as u64;
        
        Ok(byte0 | (byte1 << 8) | (byte2 << 16) | (byte3 << 24)
            | (byte4 << 32) | (byte5 << 40) | (byte6 << 48) | (byte7 << 56))
    }

    pub fn write_8(&mut self, addr: usize, val: u8) -> Result<(), RamError> {
        if let Some(cell) = self.0.get_mut(addr) {
            *cell = val;
            return Ok(());
        }

        Err(RamError::OutOfBounds)
    }

    pub fn write_16(&mut self, addr: usize, val: u16) -> Result<(), RamError> {
        self.write_8(addr, val as u8)?;
        self.write_8(addr + 1, (val >> 8) as u8)?;
        Ok(())
    }

    pub fn write_32(&mut self, addr: usize, val: u32) -> Result<(), RamError> {
        self.write_8(addr, val as u8)?;
        self.write_8(addr + 1, (val >> 8) as u8)?;
        self.write_8(addr + 2, (val >> 16) as u8)?;
        self.write_8(addr + 3, (val >> 24) as u8)?;
        Ok(())
    }

    pub fn write_64(&mut self, addr: usize, val: u64) -> Result<(), RamError> {
        self.write_8(addr, val as u8)?;
        self.write_8(addr + 1, (val >> 8) as u8)?;
        self.write_8(addr + 2, (val >> 16) as u8)?;
        self.write_8(addr + 3, (val >> 24) as u8)?;
        self.write_8(addr + 4, (val >> 32) as u8)?;
        self.write_8(addr + 5, (val >> 40) as u8)?;
        self.write_8(addr + 6, (val >> 48) as u8)?;
        self.write_8(addr + 7, (val >> 56) as u8)?;
        Ok(())
    }
}
