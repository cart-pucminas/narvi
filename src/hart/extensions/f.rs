use crate::hart::{Hart, HartError};
use crate::util::{
    get_bits,
    sign_extend_32, sign_extend_64,
    rounding_modes::*,
};

#[allow(dead_code, unused_variables)]
impl Hart<'_> {
    pub(super) fn execute_f(&mut self, inst: u32) -> Result<(), HartError> {
        let opcode = get_bits(6, 0, inst);
        match opcode {
            0x0 => todo!("instruction decoding"),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn flw(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(get_bits(19, 15, inst) as u8)?;
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_64(get_bits(31, 20, inst) as u64, 12);
        let addr = rs1.wrapping_add(imm);
        let resulting_value = self.l1.get32(addr as usize);
        self.set_fp_reg_32_bits(rd, resulting_value)
    }

    fn fsw(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(get_bits(19, 15, inst) as u8)?;
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_64(imm_bits as u64, 12);
        let addr = rs1.wrapping_add(imm);
        let rs2 = get_bits(19, 15, inst) as u8;
        let reg_val = self.get_fp_reg_32_bits(rs2)?;
        self.l1.set32(addr as usize, reg_val);
        Ok(())
    }

    fn fmadd_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f32 = self.get_fp_reg_32(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_fma(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                            // get RM from FCSR
            _ => float_fma(rs1, rs2, rs3, rm),
        };
        if (rs1.is_infinite() && rs2 == 0.0f32) | (rs2.is_infinite() && rs1 == 0.0f32) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fmsub_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f32 = self.get_fp_reg_32(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f32) | (rs2.is_infinite() && rs1 == 0.0f32) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => float_fms(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                            // get RM from FCSR
            _ => float_fms(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fnmsub_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f32 = self.get_fp_reg_32(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f32) | (rs2.is_infinite() && rs1 == 0.0f32) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => float_fnms(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                            // get RM from FCSR
            _ => float_fnms(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fnmadd_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f32 = self.get_fp_reg_32(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f32) | (rs2.is_infinite() && rs1 == 0.0f32) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => float_fnma(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                            // get RM from FCSR
            _ => float_fnma(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fadd_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_add(rs1, rs2, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_add(rs1, rs2, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fsub_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_sub(rs1, rs2, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_sub(rs1, rs2, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fmul_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f32) | (rs2.is_infinite() && rs1 == 0.0f32) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => float_mul(rs1, rs2, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_mul(rs1, rs2, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fdiv_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_div(rs1, rs2, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_div(rs1, rs2, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fsqrt_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_sqrt(rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_sqrt(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fsgnj_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs2.is_sign_negative() {
            f32::from_bits( rs1.to_bits() | 0x8000_0000 )
        } else {
            f32::from_bits( rs1.to_bits() & 0x7FFF_FFFF )
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fsgnjn_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs2.is_sign_positive() {
            f32::from_bits( rs1.to_bits() | 0x8000_0000 )
        } else {
            f32::from_bits( rs1.to_bits() & 0x7FFF_FFFF )
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fsgnjx_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs1.is_sign_negative() ^ rs2.is_sign_negative() {
            f32::from_bits( rs1.to_bits() | 0x8000_0000 )
        } else {
            f32::from_bits( rs1.to_bits() & 0x7FFF_FFFF )
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fmin_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        if rs1.is_nan() | rs2.is_nan() { self.fcsr |= 0x0000_0010; } // If any operand is NaN, set
                                                                        // Invalid Operation flag
        self.set_fp_reg_32(rd, f32::min(rs1, rs2))?;
        Ok(())
    }

    fn fmax_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        if rs1.is_nan() | rs2.is_nan() { self.fcsr |= 0x0000_0010; } // If any operand is NaN, set
                                                                        // Invalid Operation flag
        self.set_fp_reg_32(rd, f32::max(rs1, rs2))?;
        Ok(())
    }

    fn fcvt_w_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_to_i32 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_to_i32(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_wu_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_to_u32 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_to_u32 (rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fmv_x_w(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1: u32 = self.get_fp_reg_32_bits(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.");
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = sign_extend_64(rs1 as u64, 32);
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn feq_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 == rs2) as u64 };
        if (rs1.is_nan() && rs1.is_sign_positive() ) || (rs2.is_nan() && rs2.is_sign_positive() ) { 
            self.fcsr |= 0x0000_0010; } // If any operand is signaling NaN, set
                                        // Invalid Operation flag
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn flt_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 < rs2) as u64 };
        if rs1.is_nan() || rs2.is_nan() { self.fcsr |= 0x0000_0010; } // If any operand is NaN, set
                                                                        // Invalid Operation flag
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fle_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f32 = self.get_fp_reg_32(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 <= rs2) as u64 };
        if rs1.is_nan() || rs2.is_nan() { self.fcsr |= 0x0000_0010; } // If any operand is NaN, set
                                                                        // Invalid Operation flag
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fclass_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1: f32 = self.get_fp_reg_32(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.");
        let rd = get_bits(11, 7, inst) as u8;
        let mut reg_val: u64 = 0;
        if rs1.is_sign_positive() {
            if rs1.is_infinite() { reg_val |= 0b00_1000_0000; }
            else if rs1.is_nan() { reg_val |= 0b01_0000_0000; }
            else if rs1 == 0.0 { reg_val |= 0b00_0001_0000; }
            else if rs1.is_subnormal() { reg_val |= 0b00_0010_0000; }
            else { reg_val |= 0b00_0100_0000; }
        }
        // Else is sign negative
        else if rs1.is_infinite() { reg_val |= 0b00_0000_0001; }
        else if rs1.is_nan() { reg_val |= 0b10_0000_0000; }
        else if rs1 == 0.0 { reg_val |= 0b00_0000_1000; }
        else if rs1.is_subnormal() { reg_val |= 0b00_0000_0100; }
        else { reg_val |= 0b00_0000_0010; }
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_s_w(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as i32;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => i32_to_float (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => i32_to_float(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_s_wu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as u32;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => u32_to_float (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => u32_to_float(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fmv_w_x(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1: u32 = self.get_reg(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.") as u32;
        let rd = get_bits(11, 7, inst) as u8;
        self.set_fp_reg_32_bits(rd, rs1)?;
        Ok(())
    }

    fn fcvt_l_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_to_i64 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_to_i64(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_lu_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => float_to_u64 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => float_to_u64(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_s_l(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as i64;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => i64_to_float (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => i64_to_float(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_s_lu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap();

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => u64_to_float (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => u64_to_float(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }
}
