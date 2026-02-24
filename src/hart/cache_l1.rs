use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CacheL1 {
    arr: Vec<u8>,
    size: usize
}

#[allow(dead_code)]
impl CacheL1 {
    /// Creates a new memory space initialized to zero.
    ///
    /// # Arguments
    ///
    /// * `size` - The desired total size of the memory space *in bytes*.
    pub fn new(size: usize) -> Self {
        CacheL1 {
            arr: vec![0; size],
            size
        }
    }

    /// Reads a single byte (8 bits) from the memory at the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The byte address to read from.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is out of bounds.
    pub fn get8(&self, addr: usize) -> u8 {
        self.arr[addr]
    }

    /// Returns the size of the memory
    ///
    /// # Arguments
    ///
    /// * 'none' - No arguments
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Returns a halfword (2 bytes) in little-endian format,
    /// starting from `addr`.
    ///
    /// The byte at `addr` is the Least Significant Byte (LSB).
    ///
    /// # Arguments
    ///
    /// * `addr` - The starting byte address to read from.
    pub fn get16(&self, addr: usize) -> u16 {
        (self.arr[addr + 1] as u16) << 8 | self.arr[addr] as u16
    }

    /// Returns a word (4 bytes) in little-endian form,
    /// starting from `addr`.
    ///
    /// The byte at `addr` is the Least Significant Byte (LSB).
    ///
    /// # Arguments
    ///
    /// * `addr` - The starting byte address to read from.
    pub fn get32(&self, addr: usize) -> u32 {
        (self.get16(addr + 2) as u32) << 16 | self.get16(addr) as u32
    }

    /// Returns a doubleword (8 bytes) in little-endian form,
    /// starting from `addr`.
    ///
    /// The byte at `addr` is the Least Significant Byte (LSB).
    ///
    /// # Arguments
    ///
    /// * `addr` - The starting byte address to read from.
    pub fn get64(&self, addr: usize) -> u64 {
        (self.get32(addr + 4) as u64) << 32 | self.get32(addr) as u64
    }

    /// Writes a single byte (8 bits) to the memory at the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The byte address to read from.
    /// * `val` - The value to be written in memory
    ///
    /// # Panics
    ///
    /// Panics if `addr` is out of bounds.
    pub fn set8(&mut self, addr: usize, val: u8){
        self.arr[addr] = val;
    }
    
    /// Writes a halfword (16 bits) to the memory at the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The byte address to read from.
    /// * `val` - The value to be written in memory
    ///
    /// # Panics
    ///
    /// Panics if `addr` is out of bounds.
    pub fn set16(&mut self, addr: usize, val: u16){
        let p1 = ((val & 0xFF00) >> 8) as u8;
        let p2 = (val & 0x00FF) as u8;
        self.arr[addr + 1] = p1;
        self.arr[addr] = p2;
    }
    
    /// Writes a word (32 bits) to the memory at the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The byte address to read from.
    /// * `val` - The value to be written in memory
    ///
    /// # Panics
    ///
    /// Panics if `addr` is out of bounds.
    pub fn set32(&mut self, addr: usize, val: u32){
        let p1 = ((val & 0xFF00_0000) >> 24) as u8;
        let p2 = ((val & 0x00FF_0000) >> 16) as u8;
        let p3 = ((val & 0x0000_FF00) >> 8) as u8;
        let p4 = (val & 0x0000_00FF) as u8;
        self.arr[addr + 3] = p1;
        self.arr[addr + 2] = p2;
        self.arr[addr + 1] = p3;
        self.arr[addr] = p4;
    }
    
    /// Writes a doubleword (64 bits) to the memory at the given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The byte address to read from.
    /// * `val` - The value to be written in memory
    ///
    /// # Panics
    ///
    /// Panics if `addr` is out of bounds.
    pub fn set64(&mut self, addr: usize, val: u64){
        let p1 = ((val & 0xFF00_0000_0000_0000) >> 56) as u8;
        let p2 = ((val & 0x00FF_0000_0000_0000) >> 48) as u8;
        let p3 = ((val & 0x0000_FF00_0000_0000) >> 40) as u8;
        let p4 = ((val & 0x0000_00FF_0000_0000) >> 32) as u8;
        let p5 = ((val & 0x0000_0000_FF00_0000) >> 24) as u8;
        let p6 = ((val & 0x0000_0000_00FF_0000) >> 16) as u8;
        let p7 = ((val & 0x0000_0000_0000_FF00) >> 8) as u8;
        let p8 = (val & 0x0000_0000_0000_00FF) as u8;
        self.arr[addr + 7] = p1;
        self.arr[addr + 6] = p2;
        self.arr[addr + 5] = p3;
        self.arr[addr + 4] = p4;
        self.arr[addr + 3] = p5;
        self.arr[addr + 2] = p6;
        self.arr[addr + 1] = p7;
        self.arr[addr] = p8;
    }
    
}
