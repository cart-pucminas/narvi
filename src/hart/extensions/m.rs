use crate::hart::{Hart, HartError};
use crate::util::{get_bits, sign_extend_64, sign_extend_128};

#[allow(dead_code)]
impl Hart<'_> {
    pub(super) fn execute_m(&mut self, inst: u32) -> Result<(), HartError> {
        let opcode = get_bits(6, 0, inst);
        let funct3 = get_bits(14, 12, inst);
        match (opcode, funct3) {
            (0b0110011, 0b000) => self.mul(inst),
            (0b0110011, 0b001) => self.mulh(inst),
            (0b0110011, 0b010) => self.mulhsu(inst),
            (0b0110011, 0b011) => self.mulhu(inst),
            (0b0110011, 0b100) => self.div(inst),
            (0b0110011, 0b101) => self.divu(inst),
            (0b0110011, 0b110) => self.rem(inst),
            (0b0110011, 0b111) => self.remu(inst),
            (0b0111011, 0b000) => self.mulw(inst),
            (0b0111011, 0b100) => self.divw(inst),
            (0b0111011, 0b101) => self.divuw(inst),
            (0b0111011, 0b110) => self.remw(inst),
            (0b0111011, 0b111) => self.remuw(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn mul(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)?;
        let rhs = self.get_reg(rs2)?;
        self.set_reg(rd, lhs.wrapping_mul(rhs))?;
        Ok(())
    }

    fn mulh(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = sign_extend_128(self.get_reg(rs1)? as u128, 64);
        let rhs = sign_extend_128(self.get_reg(rs2)? as u128, 64);
        let result = lhs.wrapping_mul(rhs);
        self.set_reg(rd, (result >> 64) as u64)?;
        Ok(())
    }

    fn mulhsu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = sign_extend_128(self.get_reg(rs1)? as u128, 64) as i128;
        let rhs = self.get_reg(rs2)? as i128;
        let result = lhs.wrapping_mul(rhs);
        self.set_reg(rd, (result >> 64) as u64)?;
        Ok(())
    }

    fn mulhu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as u128;
        let rhs = self.get_reg(rs2)? as u128;
        let result = lhs * rhs;
        self.set_reg(rd, (result >> 64) as u64)?;
        Ok(())
    }

    fn div(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as i64;
        let rhs = self.get_reg(rs2)? as i64;
        self.set_reg(rd, lhs.wrapping_div(rhs) as u64)?;
        Ok(())
    }

    fn divu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)?;
        let rhs = self.get_reg(rs2)?;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        self.set_reg(rd, lhs.wrapping_div(rhs))?;
        Ok(())
    }

    fn rem(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as i64;
        let rhs = self.get_reg(rs2)? as i64;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        if lhs == i64::MIN && rhs == -1 {
            todo!("don't know what happens on overflow yet");
        }

        self.set_reg(rd, (lhs % rhs) as u64)?;
        Ok(())
    }

    fn remu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as u64;
        let rhs = self.get_reg(rs2)? as u64;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        self.set_reg(rd, lhs % rhs)?;
        Ok(())
    }

    fn mulw(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        print!("{:b}\n", inst);
        print!("{}, {}", rs1, rs2);
        let lhs = self.get_reg(rs1)? as i32;
        let rhs = self.get_reg(rs2)? as i32;
        let result = lhs.wrapping_mul(rhs) as u64;
        self.set_reg(rd, sign_extend_64(result, 32))?;
        Ok(())
    }

    fn divw(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as i32;
        let rhs = self.get_reg(rs2)? as i32;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        let result = (lhs / rhs) as u64;
        self.set_reg(rd, sign_extend_64(result, 32))?;
        Ok(())
    }

    fn divuw(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as u32;
        let rhs = self.get_reg(rs2)? as u32;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        self.set_reg(rd, (lhs / rhs) as u64)?;
        Ok(())
    }

    fn remw(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as i32;
        let rhs = self.get_reg(rs2)? as i32;
        let result = (lhs % rhs) as u64;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        if lhs == i32::MIN && rhs == -1 {
            todo!("don't know what happens on overflow yet");
        }

        self.set_reg(rd, sign_extend_64(result, 32) as u64)?;
        Ok(())
    }

    fn remuw(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let lhs = self.get_reg(rs1)? as u32;
        let rhs = self.get_reg(rs2)? as u32;

        if rhs == 0 {
            todo!("don't know what to do with division by 0 yet");
        }

        self.set_reg(rd, (lhs % rhs) as u64)?;
        Ok(())
    }
}

#[cfg(test)]
mod m_tests {
    use std::u64;

    use crate::hart::extensions::Extensions;

    use crate::hart::{Hart, HartError, Reg};

    static EXTENSIONS: Extensions = Extensions {
        m: true,
        a: false,
        c: false,
        f: false,
        d: false,
    };

    #[test]
    fn mul1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, u64::MAX)?;
        hart.set_reg(Reg::s1 as u8, u64::MAX)?;

        let inst = 0x02848933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 1);

        Ok(())
    }

    #[test]
    fn mul2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, -1i64 as u64)?;
        hart.set_reg(Reg::s1 as u8, -1i64 as u64)?;

        let inst = 0x02848933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 1);

        Ok(())
    }

    #[test]
    fn mulh1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x80000000000000)?;
        hart.set_reg(Reg::s1 as u8, (-0x80000000000000i64) as u64)?;

        let inst = 0x02941933;
        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFC00000000000);

        Ok(())
    }

    #[test]
    fn mulhsu1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x80000000000000)?;
        hart.set_reg(Reg::s1 as u8, (-0x80000000000000i64) as u64)?;

        let inst = 0x02942933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x7FC00000000000);

        Ok(())
    }

    #[test]
    fn mulhsu2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x80000000000000i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0x80000000000000)?;

        let inst = 0x02942933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFC00000000000);

        Ok(())
    }

    #[test]
    fn mulhu1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x80000000000000)?;
        hart.set_reg(Reg::s1 as u8, (-0x80000000000000i64) as u64)?;

        let inst = 0x02943933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x7FC00000000000);

        Ok(())
    }

    #[test]
    fn div1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x80000000000000)?;
        hart.set_reg(Reg::s1 as u8, (-0x80000000000000i64) as u64)?;

        let inst = 0x2944933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFFFFFFFF);

        Ok(())
    }

    #[test]
    fn divu1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x80000000000000i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0xF)?;

        let inst = 0x2945933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x1108888888888888);

        Ok(())
    }

    #[test]
    fn rem1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x5)?;
        hart.set_reg(Reg::s1 as u8, 0x4)?;

        let inst = 0x2946933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x1);

        Ok(())
    }

    #[test]
    fn rem2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x5i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0x4)?;

        let inst = 0x2946933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFFFFFFFF);

        Ok(())
    }

    #[test]
    fn rem3() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x5)?;
        hart.set_reg(Reg::s1 as u8, (-0x4i64) as u64)?;

        let inst = 0x2946933;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x1);

        Ok(())
    }

    #[test]
    fn mulw1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x10000)?;
        hart.set_reg(Reg::s1 as u8, 0x10000)?;

        let inst = 0x294093B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x0);

        Ok(())
    }

    #[test]
    fn mulw2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x1000)?;
        hart.set_reg(Reg::s1 as u8, (-0x1000i64) as u64)?;

        let inst = 0x294093B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFF000000);

        Ok(())
    }

    #[test]
    fn divw1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0x1)?;
        hart.set_reg(Reg::s1 as u8, (-0x1i64) as u64)?;

        let inst = 0x294493B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFFFFFFFF);

        Ok(())
    }

    #[test]
    fn divw2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0xFFFFFFFF00000004)?;
        hart.set_reg(Reg::s1 as u8, 0xFFFFFFFF00000002)?;

        let inst = 0x294493B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x2);

        Ok(())
    }

    #[test]
    fn divuw1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x1i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0x1)?;

        let inst = 0x294593B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFF);

        Ok(())
    }

    #[test]
    fn divuw2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0xFFFFFFFFFFFFFFFF)?;
        hart.set_reg(Reg::s1 as u8, 0xFFFFFFFF00000001)?;

        let inst = 0x294593B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFF);

        Ok(())
    }

    #[test]
    fn remw1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x5i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0x4)?;

        let inst = 0x294693B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFFFFFFFF);

        Ok(())
    }

    #[test]
    fn remw2() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, (-0x5i64) as u64)?;
        hart.set_reg(Reg::s1 as u8, 0xFFFFFFFF00000004)?;

        let inst = 0x294693B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0xFFFFFFFFFFFFFFFF);

        Ok(())
    }

    #[test]
    fn remuw1() -> Result<(), HartError> {
        let mut hart = Hart::from_extensions(&EXTENSIONS, 0);

        hart.set_reg(Reg::s0 as u8, 0xFFFFFFFF00000005)?;
        hart.set_reg(Reg::s1 as u8, 0xFFFFFFFF00000004)?;

        let inst = 0x294793B;

        assert!(hart.execute_m(inst).is_ok());
        assert_eq!(hart.regs[Reg::s2 as usize], 0x1);

        Ok(())
    }
}
