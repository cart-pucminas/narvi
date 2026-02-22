use super::{Hart, HartError};
use crate::util::{get_bits, sign_extend_32, sign_extend_64};

#[allow(dead_code, unused_variables)]
impl Hart<'_> {
    pub(super) fn execute_rv64i(&mut self, inst: u32) -> Result<(), HartError> {
        let opcode = get_bits(6, 0, inst);
        match opcode {
            0x37 => self.lui(inst),
            0x17 => self.auipc(inst),
            0x6F => self.jal(inst),
            0x67 => self.jalr(inst),
            0x63 => self.branch(inst),
            0x03 => self.load(inst),
            0x23 => self.store(inst),
            0x13 => self.al_imm(inst),
            0x33 => self.al(inst),
            0x0F => self.fence(inst),
            0x73 => self.environment(inst),
            0x1B => self.al_imm_w(inst),
            0x3B => self.al_w(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn branch(&mut self, inst: u32) -> Result<(), HartError> {
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => self.beq(inst),
            1 => self.bne(inst),
            4 => self.blt(inst),
            5 => self.bge(inst),
            6 => self.bltu(inst),
            7 => self.bgeu(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn load(&mut self, inst: u32) -> Result<(), HartError> {
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => self.lb(inst),
            1 => self.lh(inst),
            2 => self.lw(inst),
            4 => self.lbu(inst),
            5 => self.lhu(inst),
            6 => self.lwu(inst),
            3 => self.ld(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn store(&mut self, inst: u32) -> Result<(), HartError> {
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => self.sb(inst),
            1 => self.sh(inst),
            2 => self.sw(inst),
            3 => self.sd(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn al_imm(&mut self, inst: u32) -> Result<(), HartError> {
        let funct7 = get_bits(31, 25, inst);
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => self.addi(inst),
            2 => self.slti(inst),
            3 => self.sltiu(inst),
            4 => self.xori(inst),
            6 => self.ori(inst),
            7 => self.andi(inst),
            1 => self.slli(inst),
            5 => if funct7 == 0 {
                self.srli(inst)
            } else if funct7 == 0b100000 {
                self.srai(inst)
            } else {
                Err(HartError::ExecutionError)
            },
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn al(&mut self, inst: u32) -> Result<(), HartError> {
        let funct7 = get_bits(31, 25, inst);
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => if funct7 == 0 {
                self.add(inst)   
            } else if funct7 == 0b100000 {
                self.sub(inst)
            } else {
                Err(HartError::InstructionNotFound) 
            },
            1 => self.sll(inst),
            2 => self.slt(inst),
            3 => self.sltu(inst),
            4 => self.xor(inst),
            5 => if funct7 == 0 {
                self.srl(inst)
            } else if funct7 == 0b100000 {
                self.sra(inst)
            } else {
                Err(HartError::InstructionNotFound) 
            },
            6 => self.or(inst),
            7 => self.and(inst),
            _ => Err(HartError::InstructionNotFound),
        }
    }

    fn environment(&mut self, inst: u32) -> Result<(), HartError> {
        let func12 = get_bits(31, 20, inst);
        if func12 == 0 {
            self.ecall(inst)
        } else if func12 == 1 {
            self.ebreak(inst)
        } else {
            Err(HartError::ExecutionError)
        }
    }

    fn al_imm_w(&mut self, inst: u32) -> Result<(), HartError> {
        let funct7 = get_bits(31, 25, inst);
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => self.addiw(inst),
            1 => self.slliw(inst),
            5 => if funct7 == 0 {
                self.srliw(inst)
            } else if funct7 == 0b10000 {
                self.sraiw(inst)
            } else {
                Err(HartError::ExecutionError)
            },
            _ => Err(HartError::ExecutionError),
        }
    }

    fn al_w(&mut self, inst: u32) -> Result<(), HartError> {
        let funct7 = get_bits(31, 25, inst);
        let funct3 = get_bits(14, 12, inst);
        match funct3 {
            0 => if funct7 == 0 {
                self.addw(inst)
            } else if funct7 == 0b100000 {
                self.subw(inst)
            } else {
                Err(HartError::ExecutionError)
            },
            1 => self.sllw(inst),
            5 => if funct7 == 0 {
                self.srlw(inst)
            } else if funct7 == 0b100000 {
                self.sraw(inst)
            } else {
                Err(HartError::ExecutionError)
            },
            _ => Err(HartError::ExecutionError),
        }
    }

    fn fence(&mut self, inst: u32) -> Result<(), HartError> {
        todo!("fence")
    }

    fn lui(&mut self, inst: u32) -> Result<(), HartError> {
        // let imm = get_bits(31, 12, inst) << 12;
        let imm = inst & 0xFFFFF000;
        let rd = get_bits(11, 7, inst) as u8;
        self.set_reg(rd, sign_extend_64(imm as u64, 32))?;
        Ok(())
    }

    fn auipc(&mut self, inst: u32) -> Result<(), HartError> {
        let imm = inst & 0xFFFFF000;
        let rd = get_bits(11, 7, inst) as u8;
        let val:u64 = imm as u64 + self.pc;
        self.set_reg(rd, val)?;
        Ok(())
    }

    fn jal(&mut self, inst: u32) -> Result<(), HartError> {
        let imm_19_12 = (get_bits(19, 12, inst) << 12) as u64;
        let imm_11 = (get_bits(20, 20, inst) << 11) as u64;
        let imm_10_1 = (get_bits(30, 21, inst) << 1) as u64;
        let imm_20 = (get_bits(31, 31, inst) << 20) as u64;
        let rd = get_bits(11, 7, inst) as u8;
        let offset = sign_extend_64(imm_20 | imm_19_12 | imm_11 | imm_10_1, 21); 
        self.set_reg(rd, self.pc + 4)?;
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn jalr(&mut self, inst: u32) -> Result<(), HartError> {
        let imm = sign_extend_64(get_bits(31, 20, inst) as u64, 12);
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let source_val = self.get_reg(rs1)?;
        self.set_reg(rd, self.pc + 4)?;

        // TODO: this addition sometimes leads to an overflow, but I'm not sure how it should be treated yet
        self.pc = (source_val + imm) & !1u64;
        Ok(())
    }

    fn beq(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if self.get_reg(rs1)? != self.get_reg(rs2)? {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn bne(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if self.get_reg(rs1)? == self.get_reg(rs2)? {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn blt(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if (self.get_reg(rs1)? as i64) >= (self.get_reg(rs2)? as i64) {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn bge(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if (self.get_reg(rs1)? as i64) < (self.get_reg(rs2)? as i64) {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn bltu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if self.get_reg(rs1)? >= self.get_reg(rs2)? {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn bgeu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;

        if self.get_reg(rs1)? < self.get_reg(rs2)? {
            return Ok(())
        }

        let imm_11 = (get_bits(7, 7, inst) << 11) as u64;
        let imm_4_1 = (get_bits(11, 8, inst) << 1) as u64;
        let imm_10_5 = (get_bits(30, 25, inst) << 5) as u64;
        let imm_12 = (get_bits(31, 31, inst) << 12) as u64;
        let offset = sign_extend_64(imm_12 | imm_11 | imm_10_5 | imm_4_1, 13);
        self.pc += offset - 4; // considering that PC will move +4 after instruction
        Ok(())
    }

    fn lb(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = sign_extend_64(self.l1.get8(addr as usize) as u64, 8);
        self.set_reg(rd, resulting_value)
    }

    fn lh(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = sign_extend_64(self.l1.get16(addr as usize) as u64, 16);
        self.set_reg(rd, resulting_value)
    }

    fn lw(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = sign_extend_64(self.l1.get32(addr as usize) as u64, 32);
        self.set_reg(rd, resulting_value)
    }

    fn lwu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = self.l1.get32(addr as usize) as u64;
        self.set_reg(rd, resulting_value)
    }

    fn lbu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = self.l1.get8(addr as usize) as u64;
        self.set_reg(rd, resulting_value)
    }

    fn lhu(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = self.l1.get16(addr as usize) as u64;
        self.set_reg(rd, resulting_value)
    }

    fn ld(&mut self, inst: u32) -> Result<(), HartError> {
        let rs1 = get_bits(19, 15, inst);
        let rd = get_bits(11, 7, inst) as u8;
        let imm = sign_extend_32(get_bits(31, 20, inst), 12);
        let addr = rs1 + imm;
        let resulting_value = self.l1.get64(addr as usize);
        self.set_reg(rd, resulting_value)
    }

    fn sb(&mut self, inst: u32) -> Result<(), HartError> { 
        let rs1 = get_bits(19, 15, inst);
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_32(imm_bits, 12);
        let addr = rs1 + imm;
        let rs2 = get_bits(19, 15, inst) as u8;
        let reg_val = self.get_reg(rs2)? as u8;
        self.l1.set8(addr as usize, reg_val);
        Ok(())
    }

    fn sh(&mut self, inst: u32) -> Result<(), HartError> { 
        let rs1 = get_bits(19, 15, inst);
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_32(imm_bits, 12);
        let addr = rs1 + imm;
        let rs2 = get_bits(19, 15, inst) as u8;
        let reg_val = self.get_reg(rs2)? as u16;
        self.l1.set16(addr as usize, reg_val);
        Ok(())
    }

    fn sw(&mut self, inst: u32) -> Result<(), HartError> { 
        let rs1 = get_bits(19, 15, inst);
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_32(imm_bits, 12);
        let addr = rs1 + imm;
        let rs2 = get_bits(19, 15, inst) as u8;
        let reg_val = self.get_reg(rs2)? as u32;
        self.l1.set32(addr as usize, reg_val);
        Ok(())
    }

    fn sd(&mut self, inst: u32) -> Result<(), HartError> { 
        let rs1 = get_bits(19, 15, inst);
        let imm_bits = get_bits(11, 7, inst) | ( get_bits(31, 25, inst) << 5 );
        let imm = sign_extend_32(imm_bits, 12);
        let addr = rs1 + imm;
        let rs2 = get_bits(19, 15, inst) as u8;
        let reg_val = self.get_reg(rs2)? as u64;
        self.l1.set64(addr as usize, reg_val);
        Ok(())
    }

    fn addi(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source.wrapping_add(sign_extend_64(imm as u64, 12)))?;
        Ok(())
    }

    fn slti(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        let value = (source as i64) < (sign_extend_64(imm as u64, 12) as i64);
        self.set_reg(rd, value as u64)?;
        Ok(())
    }

    fn sltiu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        let value = source < sign_extend_64(imm as u64, 12);
        self.set_reg(rd, value as u64)?;
        Ok(())
    }

    fn xori(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source ^ sign_extend_64(imm as u64, 12))?; 
        Ok(())
    }

    fn ori(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source | sign_extend_64(imm as u64, 12))?; 
        Ok(())
    }

    fn andi(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source & sign_extend_64(imm as u64, 12))?; 
        Ok(())
    }

    fn slli(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let shamt = get_bits(24, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source << shamt)?;
        Ok(())
    }

    fn srli(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let shamt = get_bits(24, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source >> shamt)?;
        Ok(())
    }

    fn srai(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        // let rs2 = get_bits(24, 20, inst) as u8;
        // let shamt = self.get_reg(rs2)?;
        let shamt = get_bits(25, 20, inst);
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, ((source as i64) >> shamt) as u64)?;
        Ok(())
    }

    fn add(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        self.set_reg(rd, source1.wrapping_add(source2))?;
        Ok(())
    }

    fn sub(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        self.set_reg(rd, source1.wrapping_sub(source2))?;
        Ok(())
    }

    fn sll(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source << shamt)?;
        Ok(())
    }

    fn slt(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        let value = (source1 as i64) < (source2 as i64);
        self.set_reg(rd, value as u64)?;
        Ok(())
    }

    fn sltu(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        let value = source1 < source2;
        self.set_reg(rd, value as u64)?;
        Ok(())
    }

    fn xor(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        self.set_reg(rd, source1 ^ source2)?;
        Ok(())
    }

    fn srl(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, source >> shamt)?;
        Ok(())
    }

    fn sra(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)?;
        self.set_reg(rd, 
            (source as i64 >> shamt) as u64
        )?;
        Ok(())
    }

    fn or(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        self.set_reg(rd, source1 | source2)?;
        Ok(())
    }

    fn and(&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)?;
        let source2 = self.get_reg(rs2)?;
        self.set_reg(rd, source1 & source2)?;
        Ok(())
    }

    fn ecall(&mut self, inst: u32) -> Result<(), HartError> {
        todo!("ecall")
    }

    fn ebreak(&mut self, inst: u32) -> Result<(), HartError> {
        todo!("ebreak")
    }

    fn addiw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let imm = get_bits(31, 20, inst);
        let source = self.get_reg(rs1)? & 0x0000_0000_FFFF_FFFF;
        self.set_reg(rd, source.wrapping_add(sign_extend_64(imm as u64, 12)))?;
        Ok(())
    }

    fn slliw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let shamt = get_bits(24, 20, inst);
        let source = self.get_reg(rs1)?;
        let res: u32 = (source as u32) << shamt; // Cast to 32 bit, then sign extend
        self.set_reg(rd, sign_extend_64(res as u64, 32) )?;
        Ok(())
    }

    fn srliw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let shamt = get_bits(24, 20, inst);
        let source = self.get_reg(rs1)?;
        let res: u32 = (source as u32) >> shamt; // Cast to 32 bit, then sign extend
        self.set_reg(rd, sign_extend_64(res as u64, 32) )?;
        Ok(())
    }
    
    fn sraiw (&mut self, inst: u32) -> Result<(), HartError> {
        if (inst & 0x0200_0000) == 0x0200_0000 {
            return Err(HartError::ReservedInstruction(String::from("SRAIW With 25th bit as 1 is a reserved instruction.")));
        }
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let shamt = get_bits(24, 20, inst);
        let source = ((self.get_reg(rs1)?) & 0x0000_0000_FFFF_FFFF) as u32;
        self.set_reg(rd, 
            sign_extend_64(((source as i32) >> shamt) as u64, 32)
        )?;
        Ok(())
    }

    fn addw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)? as u32;
        let source2 = self.get_reg(rs2)? as u32;
        self.set_reg(rd, 
            sign_extend_64(source1.wrapping_add(source2) as u64, 32) 
        )?;
        Ok(())
    }

    fn subw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = get_bits(24, 20, inst) as u8;
        let source1 = self.get_reg(rs1)? as u32;
        let source2 = self.get_reg(rs2)? as u32;
        self.set_reg(rd, 
            sign_extend_64(source1.wrapping_sub(source2) as u64, 32) 
        )?;
        Ok(())
    }

    fn sllw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = (get_bits(24, 20, inst) as u8) & 0x0F;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)? as u32;
        self.set_reg(rd, 
            sign_extend_64((source << shamt) as u64, 32)
        )?;
        Ok(())
    }

    fn srlw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = (get_bits(24, 20, inst) as u8) & 0x0F;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)? as u32;
        self.set_reg(rd, 
            sign_extend_64((source >> shamt) as u64, 32)
        )?;
        Ok(())
    }
    
    fn sraw (&mut self, inst: u32) -> Result<(), HartError> {
        let rd = get_bits(11, 7, inst) as u8;
        let rs1 = get_bits(19, 15, inst) as u8;
        let rs2 = (get_bits(24, 20, inst) as u8) & 0x0F;
        let shamt = self.get_reg(rs2)?;
        let source = self.get_reg(rs1)? as u32;
        self.set_reg(rd, 
            sign_extend_64((source as i32 >> shamt) as u64, 32)
        )?;
        Ok(())
    }
}
