use crate::hart::{Hart, HartError};
use crate::util::{
    get_bits,
    sign_extend_64,
    rounding_modes::*,
};

#[allow(dead_code, unused_variables)]
impl Hart<'_> {
    pub(super) fn execute_d(&mut self, inst: u32) -> Result<(), HartError> {
        let opcode = get_bits(6, 0, inst);
        let funct7 = get_bits(31, 25, inst);
        let funct2 = get_bits(26, 25, inst);
        let rs2 = get_bits(24, 20, inst);
        let funct3 = get_bits(14, 12, inst);
        match (funct7, funct2, rs2, funct3, opcode) {
            (_, _, _, 0b011, 0b0000111) => self.fld(inst),
            (_, _, _, 0b011, 0b0100111) => self.fsd(inst),
            (_, 0b01, _, _, 0b1000011) => self.fmadd_d(inst),
            (_, 0b01, _, _, 0b1000111) => self.fmsub_d(inst),
            (_, 0b01, _, _, 0b1001011) => self.fnmsub_d(inst),
            (_, 0b01, _, _, 0b1001111) => self.fnmadd_d(inst),
            (0b0000001, _, _, _, 0b1010011) => self.fadd_d(inst),
            (0b0000101, _, _, _, 0b1010011) => self.fsub_d(inst),
            (0b0001001, _, _, _, 0b1010011) => self.fmul_d(inst),
            (0b0001101, _, _, _, 0b1010011) => self.fdiv_d(inst),
            (0b0101101, _, 0b00000, _, 0b1010011) => self.fsqrt_d(inst),
            (0b0010001, _, _, 0b000, 0b1010011) => self.fsgnj_d(inst),
            (0b0010001, _, _, 0b001, 0b1010011) => self.fsgnjn_d(inst),
            (0b0010001, _, _, 0b010, 0b1010011) => self.fsgnjx_d(inst),
            (0b0010101, _, _, 0b000, 0b1010011) => self.fmin_d(inst),
            (0b0010101, _, _, 0b001, 0b1010011) => self.fmax_d(inst),
            (0b0100000, _, 0b00000, _, 0b1010011) => self.fcvt_s_d(inst),
            (0b0100001, _, 0b00001, _, 0b1010011) => self.fcvt_d_s(inst),
            (0b1010001, _, 0b00000, 0b010, 0b1010011) => self.feq_d(inst),
            (0b1010001, _, _, 0b001, 0b1010011) => self.flt_d(inst),
            (0b1010001, _, _, 0b000, 0b1010011) => self.fle_d(inst),
            (0b1110001, _, _, 0b001, 0b1010011) => self.fclass_d(inst),
            (0b1100001, _, 0b00000, _, 0b1010011) => self.fcvt_w_d(inst),
            (0b1100001, _, 0b00001, _, 0b1010011) => self.fcvt_wu_d(inst),
            (0b1101001, _, 0b00000, _, 0b1010011) => self.fcvt_d_w(inst),
            (0b1101001, _, 0b00001, _, 0b1010011) => self.fcvt_d_wu(inst),
            (0b1100001, _, 0b00010, _, 0b1010011) => self.fcvt_l_d(inst),
            (0b1100001, _, 0b00011, _, 0b1010011) => self.fcvt_lu_d(inst),
            (0b1110001, _, 0b00000, 0b000, 0b1010011) => self.fmv_x_d(inst),
            (0b1101001, _, 0b00010, _, 0b1010011) => self.fcvt_d_l(inst),
            (0b1101001, _, 0b00011, _, 0b1010011) => self.fcvt_d_lu(inst),
            (0b1111001, _, 0b00000, 0b000, 0b1010011) => self.fmv_d_x(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn fld(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(get_bits(19, 15, inst) as u8)?;
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_64(get_bits(31, 20, inst) as u64, 12);
        let addr = rs1.wrapping_add(imm);
        let resulting_value = self.l1.get64(addr as usize);
        self.set_fp_reg_64(rd, f64::from_bits(resulting_value))
    }

    fn fsd(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(get_bits(19, 15, inst) as u8)?;
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_64(imm_bits as u64, 12);
        let addr = rs1.wrapping_add(imm);
        let rs2 = get_bits(19, 15, inst) as u8; // Wait, rs2 is 24-20
        let rs2 = get_bits(24, 20, inst) as u8;
        let reg_val = self.get_fp_reg_64(rs2)?.to_bits();
        self.l1.set64(addr as usize, reg_val);
        Ok(())
    }

    fn fmadd_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f64 = self.get_fp_reg_64(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_fma(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8),
            _ => double_fma(rs1, rs2, rs3, rm),
        };
        if (rs1.is_infinite() && rs2 == 0.0f64) | (rs2.is_infinite() && rs1 == 0.0f64) { self.fcsr |= 0x0000_0010; }
        self.set_fp_reg_64(rd, reg_val)?;
        // Set Invalid Operation flag if multiplying 0 by infinity
        Ok(())
    }

    fn fmsub_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f64 = self.get_fp_reg_64(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f64) | (rs2.is_infinite() && rs1 == 0.0f64) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => double_fms(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8),
            _ => double_fms(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fnmsub_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f64 = self.get_fp_reg_64(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f64) | (rs2.is_infinite() && rs1 == 0.0f64) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => double_fnms(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8),
            _ => double_fnms(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fnmadd_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rs3:f64 = self.get_fp_reg_64(
            get_bits(31, 27, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f64) | (rs2.is_infinite() && rs1 == 0.0f64) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => double_fnma(rs1, rs2, rs3, get_bits(7, 5, self.fcsr) as u8),
            _ => double_fnma(rs1, rs2, rs3, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fadd_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_add(rs1, rs2, get_bits(7, 5, self.fcsr) as u8),
            _ => double_add(rs1, rs2, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fsub_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_sub(rs1, rs2, get_bits(7, 5, self.fcsr) as u8),
            _ => double_sub(rs1, rs2, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fmul_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        if (rs1.is_infinite() && rs2 == 0.0f64) | (rs2.is_infinite() && rs1 == 0.0f64) { self.fcsr |= 0x0000_0010; }
        // Set Invalid Operation flag if multiplying 0 by infinity
        let reg_val = match rm {
            0b111 => double_mul(rs1, rs2, get_bits(7, 5, self.fcsr) as u8),
            _ => double_mul(rs1, rs2, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fdiv_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_div(rs1, rs2, get_bits(7, 5, self.fcsr) as u8),
            _ => double_div(rs1, rs2, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fsqrt_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_sqrt(rs1, get_bits(7, 5, self.fcsr) as u8),
            _ => double_sqrt(rs1, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fsgnj_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs2.is_sign_negative() {
            f64::from_bits( rs1.to_bits() | 0x8000_0000_0000_0000 )
        } else {
            f64::from_bits( rs1.to_bits() & 0x7FFF_FFFF_FFFF_FFFF )
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fsgnjn_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs2.is_sign_positive() {
            f64::from_bits( rs1.to_bits() | 0x8000_0000_0000_0000 )
        } else {
            f64::from_bits( rs1.to_bits() & 0x7FFF_FFFF_FFFF_FFFF )
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fsgnjx_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = if rs1.is_sign_negative() ^ rs2.is_sign_negative() {
            f64::from_bits( rs1.to_bits() | 0x8000_0000_0000_0000 )
        } else {
            f64::from_bits( rs1.to_bits() & 0x7FFF_FFFF_FFFF_FFFF )
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fmin_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        if rs1.is_nan() | rs2.is_nan() { self.fcsr |= 0x0000_0010; }
        self.set_fp_reg_64(rd, f64::min(rs1, rs2))?;
        Ok(())
    }

    fn fmax_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        if rs1.is_nan() | rs2.is_nan() { self.fcsr |= 0x0000_0010; }
        self.set_fp_reg_64(rd, f64::max(rs1, rs2))?;
        Ok(())
    }

    fn fcvt_s_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let rm = get_bits(14, 12, inst) as u8;
        let reg_val = match rm {
            0b111 => double_to_float (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                            // get RM from FCSR
            _ => double_to_float(rs1, rm),
        };
        self.set_fp_reg_32(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_d_s(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f32 = self.get_fp_reg_32(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        // Casting from an f32 to an f64 is perfect and lossless
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast
        self.set_fp_reg_64(rd, rs1 as f64)?;
        Ok(())
    }

    fn feq_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 == rs2) as u64 };
        if (rs1.is_nan() && rs1.is_sign_positive() ) || (rs2.is_nan() && rs2.is_sign_positive() ) { 
            self.fcsr |= 0x0000_0010; }
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn flt_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 < rs2) as u64 };
        if rs1.is_nan() || rs2.is_nan() { self.fcsr |= 0x0000_0010; }
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fle_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1:f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8
            ).unwrap();
        let rs2:f64 = self.get_fp_reg_64(
            get_bits(24, 20, inst) as u8
            ).unwrap();
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = 
            if rs1.is_nan() || rs2.is_nan() { 0 }
            else { (rs1 <= rs2) as u64 };
        if rs1.is_nan() || rs2.is_nan() { self.fcsr |= 0x0000_0010; }
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fclass_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1: f64 = self.get_fp_reg_64(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.");
        let rd = get_bits(11, 7, inst) as u8;
        let mut reg_val: u64 = 0;
        if rs1.is_sign_positive() {
            if rs1.is_infinite() { reg_val |= 0b00_1000_0000; }
            else if rs1.is_nan() { reg_val |= 0b01_0000_0000; }
            else if rs1 == 0.0 { reg_val |= 0b00_0001_0000; }
            else if rs1.is_subnormal() { reg_val |= 0b00_0010_0000; }
            else { reg_val |= 0b00_0100_0000; }
        }
        else if rs1.is_infinite() { reg_val |= 0b00_0000_0001; }
        else if rs1.is_nan() { reg_val |= 0b10_0000_0000; }
        else if rs1 == 0.0 { reg_val |= 0b00_0000_1000; }
        else if rs1.is_subnormal() { reg_val |= 0b00_0000_0100; }
        else { reg_val |= 0b00_0000_0010; }
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_w_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_to_i32 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => double_to_i32(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_wu_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_to_u32 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => double_to_u32(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_d_w(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as i32;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => i32_to_double (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => i32_to_double(rs1, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_d_wu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as u32;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => u32_to_double (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => u32_to_double(rs1, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_l_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_to_i64 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => double_to_i64(rs1, rm),
        };
        self.set_reg(rd, reg_val as u64)?;
        Ok(())
    }

    fn fcvt_lu_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 : f64 = self.get_fp_reg_64(
            get_bits(19, 15, inst) as u8   
        ).unwrap();
        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => double_to_u64 (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => double_to_u64(rs1, rm),
        };
        self.set_reg(rd, reg_val)?;
        Ok(())
    }

    fn fmv_x_d(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_fp_reg_64(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.").to_bits();
        let rd = get_bits(11, 7, inst) as u8;
        self.set_reg(rd, rs1)?;
        Ok(())
    }

    fn fcvt_d_l(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap() as i64;

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => i64_to_double (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => i64_to_double(rs1, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fcvt_d_lu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(
            get_bits(19, 15, inst) as u8   
        ).unwrap();

        let rm = get_bits(14, 12, inst) as u8;
        let rd = get_bits(11, 7, inst) as u8;
        let reg_val = match rm {
            0b111 => u64_to_double (rs1, get_bits(7, 5, self.fcsr) as u8), // if RM == DYN,
                                                                        // get RM from FCSR
            _ => u64_to_double(rs1, rm),
        };
        self.set_fp_reg_64(rd, reg_val)?;
        Ok(())
    }

    fn fmv_d_x(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = self.get_reg(get_bits(19, 15, inst) as u8).expect("should be unreachable, rs1 field is only 4 bits.");
        let rd = get_bits(11, 7, inst) as u8;
        self.set_fp_reg_64(rd, f64::from_bits(rs1))?;
        Ok(())
    }
}
