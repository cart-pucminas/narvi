pub mod extensions;
mod rv64i;
mod cache_l1;

use extensions::{
    Extensions,
};

use cache_l1::CacheL1;

use serde::ser::{Serialize, Serializer, SerializeStruct};

#[allow(dead_code, unused_variables, non_camel_case_types)]
#[derive(Debug)]
enum Reg {
    /*
    * ra = Return Adress
    * sp = Stack Pointer
    * gp = Global Pointer
    * tp = Thread Pointer
    * x8/s0 is also known as fp = Frame Pointer
    */
    zero = 0, ra = 1, sp = 2, gp = 3, tp = 4,
    t0 = 5, t1 = 6, t2 = 7,
    s0 = 8, s1 = 9,
    a0 = 10, a1 = 11, a2 = 12, a3 = 13, a4 = 14, a5 = 15, a6 = 16, a7 = 17,
    s2 = 18, s3 = 19, s4 = 20, s5 = 21, s6 = 22, s7 = 23, s8 = 24, s9 = 25, s10 = 26, s11 = 27,
    t3 = 28, t4 = 29, t5 = 30, t6 = 31,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HartError {
    RegisterNotFound,
    InstructionNotFound,
    ExecutionError,
    ReservedInstruction(String),
    InstructionAddressMisaligned,
    FLENMisalligned,
    FLENTooShort,
}

#[derive(Debug)]
enum FRegs {
    F(Vec<f32>),
    D(Vec<f64>),
    None,
}

impl FRegs {
    pub fn new(f: bool, d: bool) -> Self {
        match (f, d) {
            (true, false) => FRegs::F(vec![0.0; 32]),
            (_, true) => FRegs::D(vec![0.0; 32]),
            (false, false) => FRegs::None,
        }
    }
}

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
pub struct Hart <'a>{
    extensions: &'a Extensions,
    // Registers
    regs: Vec<u64>,
    pc: u64,
    l1: CacheL1,

    // __Floating Point__
    f_regs: FRegs,
    flen: u8,
    fcsr: u32,
}

impl Serialize for Hart<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer, 
    {
        let mut state = serializer.serialize_struct("Hart", 6)?;
        state.serialize_field("M_extension", &self.extensions.m)?;
        state.serialize_field("A_extension", &self.extensions.a)?;
        state.serialize_field("C_extension", &self.extensions.c)?;
        state.serialize_field("F_extension", &self.extensions.f)?;
        state.serialize_field("D_extension", &self.extensions.d)?;
        state.serialize_field("L1_size", &self.l1.size())?;
        state.end()
    }
}

impl Hart<'_> {
    pub fn from_extensions(extensions: &Extensions, cache_size: usize) -> Hart<'_> {
        Hart {
            extensions,
            regs: vec![0; 32],
            pc: 0,
            l1: CacheL1::new(cache_size),
            f_regs: FRegs::new(extensions.f, extensions.d),
            flen: match (extensions.f, extensions.d) {
                (true, false) => 32,
                (_, true) => 64,
                (false, false) => 0,
            },
            fcsr: 0,
        }
    }

    fn get_reg(&self, x: u8) -> Result<u64, HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else { 
            Ok(self.regs[x as usize]) 
        }
    }

    fn set_reg(&mut self, x: u8, value: u64) -> Result<(), HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else if x == 0 {
            Ok(())
        }
        else {
            self.regs[x as usize] = value;
            Ok(())
        }
    }

    pub fn get_fp_reg_32_bits(&self, x:u8) -> Result<u32, HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else {
            let res = match self.flen {
                32 => {
                    if let FRegs::F(ref v) = self.f_regs { v[x as usize].to_bits() }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                64 => {
                    if let FRegs::D(ref v) = self.f_regs { 
                        let whole = v[x as usize].to_bits();
                        if (whole & 0xFFFF_FFFF_0000_0000) == 0xFFFF_FFFF_0000_0000 { // Is properly boxed
                            // valid float: bottom 32 bits
                        ( v[x as usize].to_bits() &  0x0000_0000_FFFF_FFFF ) as u32
                        } else {
                            // not properly boxed: return Canonical NaN
                            0x7fc00000
                        }
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(res)
        }
    }

    pub fn set_fp_reg_32_bits(&mut self, x:u8, value:u32) -> Result<(), HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else {
            match self.flen {
                32 => {
                    if let FRegs::F(ref mut v) = self.f_regs {
                        v[x as usize] = f32::from_bits(value);
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                64 => {
                    if let FRegs::D(ref mut v) = self.f_regs {
                        let val:f64 = f64::from_bits( 0xFFFF_FFFF_0000_0000 | (value as u64) );
                        v[x as usize] = val;
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(())
        }
    }

    pub fn get_fp_reg_32(&self, x:u8) -> Result<f32, HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else {
            let res = match self.flen {
                32 => {
                    if let FRegs::F(ref v) = self.f_regs { v[x as usize] }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                64 => {
                    if let FRegs::D(ref v) = self.f_regs {
                        let whole = v[x as usize].to_bits();
                        if (whole & 0xFFFF_FFFF_0000_0000) == 0xFFFF_FFFF_0000_0000 { // Is properly boxed
                            // valid float: bottom 32 bits
                            f32::from_bits(whole as u32)
                        } else {
                            // not properly boxed: return Canonical NaN
                            f32::from_bits(0x7fc00000) 
                        }
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(res)
        }
    }

    pub fn get_fp_reg_64(&self, x:u8) -> Result<f64, HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else if self.flen < 64 {
            Err(HartError::FLENTooShort)
        }
        else {
            let res = match self.flen {
                64 => {
                    if let FRegs::D(ref v) = self.f_regs { v[x as usize] }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(res)
        }
    }

    pub fn set_fp_reg_32(&mut self, x:u8, value: f32) -> Result<(), HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else { 
            match self.flen {
                32 => {
                    if let FRegs::F(ref mut v) = self.f_regs {
                        v[x as usize] = value;
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                64 => {
                    if let FRegs::D(ref mut v) = self.f_regs {
                        let val:f64 = f64::from_bits( 0xFFFF_FFFF_0000_0000 | (value.to_bits() as u64) );
                        v[x as usize] = val;
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(())
        }
    }

    pub fn set_fp_reg_64(&mut self, x:u8, value: f64) -> Result<(), HartError> {
        if x > 31 {
            Err(HartError::RegisterNotFound)
        }
        else if self.flen < 64 {
            Err(HartError::FLENTooShort)
        }
        else {
            match self.flen {
                64 => {
                    if let FRegs::D(ref mut v) = self.f_regs {
                        v[x as usize] = value;
                    }
                    else { unreachable!("FLEN not matching FRegs len"); }
                }
                _ => todo!("Other floating point lengths")
            };
            Ok(())
        }
    }

    fn execute(&mut self, inst: u32) {
        if self.execute_rv64i(inst).is_ok() { return; }
        if self.extensions.m { todo!("m extension") }
        if self.extensions.a { todo!("a extension") }
        if self.extensions.f { todo!("f extension") }
    }
}
