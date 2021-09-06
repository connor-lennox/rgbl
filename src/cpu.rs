use crate::mmu::Mmu;

#[derive(Clone, Copy)]
enum Flags {
    Z = 0b10000000,
    N = 0b01000000,
    H = 0b00100000,
    C = 0b00010000,
}

impl Flags {
    fn as_bits(&self) -> u8 {
        *self as u8
    }
}

pub struct CpuRegisters {
    pub a: u8,
    pub flags: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl CpuRegisters {
    pub fn new() -> Self {
        CpuRegisters {
            a: 0x01,
            flags: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
        }
    }

    pub fn af(&self) -> u16 { ((self.a as u16) << 8) | (self.flags as u16) }
    pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | (self.c as u16) }
    pub fn de(&self) -> u16 { ((self.d as u16) << 8) | (self.e as u16) }
    pub fn hl(&self) -> u16 { ((self.h as u16) << 8) | (self.l as u16) }

    pub fn hli(&mut self) -> u16 {
        let res = self.hl();
        self.set_hl(self.hl().wrapping_add(1));
        res
    }

    pub fn hld(&mut self) -> u16 {
        let res = self.hl();
        self.set_hl(self.hl().wrapping_sub(1));
        res
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flags = (value & 0xFF) as u8;
    }

    fn get_flag(&self, flag: Flags) -> bool {
        (self.flags & flag.as_bits()) != 0
    }

    fn set_flag(&mut self, flag: Flags, value: bool) {
        let m = flag.as_bits();
        if value {
            self.flags |= m;
        } else {
            self.flags &= !m;
        }
        self.flags &= 0xF0;
    }
}

pub struct Cpu {
    pub regs: CpuRegisters,

    pub sp: u16,
    pub pc: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: CpuRegisters::new(),

            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    pub fn execute(&mut self, mmu: &mut Mmu) -> u8 {
        // Returns the number of m-cycles the opcode took
        let opcode: u8 = self.read_u8(mmu);

        match opcode {
            0x00 => { 1 }
            0x01 => { let v = self.read_u16(mmu); self.regs.set_bc(v); 3 }
            0x02 => { mmu.write(self.regs.bc(), self.regs.a); 2 }
            0x03 => { self.regs.set_bc(self.regs.bc().wrapping_add(1)); 2 }
            0x04 => { self.regs.b = self.increment(self.regs.b); 1 }
            0x05 => { self.regs.b = self.decrement(self.regs.b); 1 }
            0x06 => { self.regs.b = self.read_u8(mmu); 2 }
            0x07 => { self.regs.a = self.rlc(self.regs.a); 1 }
            0x08 => { let a = self.read_u16(mmu); mmu.write_word(a, self.sp); 5 }
            0x09 => { let v = self.add_regs(self.regs.hl(), self.regs.bc()); self.regs.set_hl(v); 2 }
            0x0A => { self.regs.a = mmu.read(self.regs.bc()); 2 }
            0x0B => { self.regs.set_bc(self.regs.bc().wrapping_sub(1)); 2 }
            0x0C => { self.regs.c = self.increment(self.regs.c); 1 }
            0x0D => { self.regs.c = self.decrement(self.regs.c); 1 }
            0x0E => { self.regs.c = self.read_u8(mmu); 2 }
            0x0F => { self.regs.a = self.rrc(self.regs.a); 1 }

            0x10 => { todo!("stop"); 1 }
            0x11 => { let v = self.read_u16(mmu); self.regs.set_de(v); 3 }
            0x12 => { mmu.write(self.regs.de(), self.regs.a); 2 }
            0x13 => { self.regs.set_de(self.regs.de().wrapping_add(1)); 2 }
            0x14 => { self.regs.d = self.increment(self.regs.d); 1 }
            0x15 => { self.regs.d = self.decrement(self.regs.d); 1 }
            0x16 => { self.regs.d = self.read_u8(mmu); 2 }
            0x17 => { self.regs.a = self.rl(self.regs.a); 1 }
            0x18 => { self.jump_rel(mmu); 3 }
            0x19 => { let v = self.add_regs(self.regs.hl(), self.regs.de()); self.regs.set_hl(v); 2 }
            0x1A => { self.regs.a = mmu.read(self.regs.de()); 2 }
            0x1B => { self.regs.set_de(self.regs.de().wrapping_sub(1)); 2 }
            0x1C => { self.regs.e = self.increment(self.regs.e); 1 }
            0x1D => { self.regs.e = self.decrement(self.regs.e); 1 }
            0x1E => { self.regs.e = self.read_u8(mmu); 2 }
            0x1F => { self.regs.a = self.rr(self.regs.a); 1 }

            0x20 => { if !self.regs.get_flag(Flags::Z) { self.jump_rel(mmu); 3 } else { self.pc += 1; 2 } }
            0x21 => { let v = self.read_u16(mmu); self.regs.set_hl(v); 3 }
            0x22 => { mmu.write(self.regs.hli(), self.regs.a); 2 }
            0x23 => { self.regs.set_hl(self.regs.hl().wrapping_add(1)); 2 }
            0x24 => { self.regs.h = self.increment(self.regs.h); 1 }
            0x25 => { self.regs.h = self.decrement(self.regs.h); 1 }
            0x26 => { self.regs.h = self.read_u8(mmu); 2 }
            0x27 => { todo!("DAA"); 1 },
            0x28 => { if self.regs.get_flag(Flags::Z) { self.jump_rel(mmu); 3 } else { self.pc += 1; 2 } }
            0x29 => { let v = self.add_regs(self.regs.hl(), self.regs.hl()); self.regs.set_hl(v); 2 }
            0x2A => { self.regs.a = mmu.read(self.regs.hli()); 2 }
            0x2B => { self.regs.set_hl(self.regs.hl().wrapping_sub(1)); 2 }
            0x2C => { self.regs.l = self.increment(self.regs.l); 1 }
            0x2D => { self.regs.l = self.decrement(self.regs.l); 1 }
            0x2E => { self.regs.l = self.read_u8(mmu); 2 }
            0x2F => { self.regs.a = !self.regs.a; self.regs.set_flag(Flags::N, true); self.regs.set_flag(Flags::H, true); 1 }

            0x30 => { if !self.regs.get_flag(Flags::C) { self.jump_rel(mmu); 3 } else { self.pc += 1; 2 } }
            0x31 => { let v = self.read_u16(mmu); self.sp = v; 3 }
            0x32 => { mmu.write(self.regs.hld(), self.regs.a); 2 }
            0x33 => { self.sp = self.sp.wrapping_add(1); 2 }
            0x34 => { let v = self.increment(mmu.read(self.regs.hl())); mmu.write(self.regs.hl(), v); 3 }
            0x35 => { let v = self.decrement(mmu.read(self.regs.hl())); mmu.write(self.regs.hl(), v); 3 }
            0x36 => { mmu.write(self.regs.hl(), self.read_u8(mmu)); 3 }            
            0x37 => { self.regs.set_flag(Flags::H, false); self.regs.set_flag(Flags::N, false); self.regs.set_flag(Flags::C, true); 1 }
            0x38 => { if self.regs.get_flag(Flags::C) { self.jump_rel(mmu); 3 } else { self.pc += 1; 2 } }
            0x39 => { let v = self.add_regs(self.regs.hl(), self.sp); self.regs.set_hl(v); 2 }
            0x3A => { self.regs.a = mmu.read(self.regs.hld()); 2 }
            0x3B => { self.sp = self.sp.wrapping_sub(1); 2 }
            0x3C => { self.regs.a = self.increment(self.regs.a); 1 }
            0x3D => { self.regs.a = self.decrement(self.regs.a); 1 }
            0x3E => { self.regs.a = self.read_u8(mmu); 2 }
            0x3F => { self.regs.set_flag(Flags::H, false); self.regs.set_flag(Flags::N, false); self.regs.set_flag(Flags::C, !self.regs.get_flag(Flags::C)); 1 }

            // HALT
            0x76 => { todo!("halt"); 1 }

            // LD dest, source
            0x40..=0x7F => {
                let mut cycles = 1;
                let source_code = opcode & 0x07;
                let dest_code = (opcode >> 3) & 0x07;

                cycles += if source_code == 0x06 {1} else {0};
                cycles += if dest_code == 0x06 {1} else {0};

                self.set_reg_from_code(mmu, dest_code, self.code_to_reg(mmu, source_code));

                cycles
            }

            // ADD A, r8
            0x80..=0x8F => {
                let carry = opcode & 0x08 == 0x08;
                self.regs.a = self.add(self.code_to_reg(mmu, opcode), carry);

                // Performing this operation with a memory access takes an extra m-cycle
                if opcode & 0x07 == 0x06 {2} else {1}
            }
            
            // SUB A, r8
            0x90..=0x9F => {
                let carry = opcode & 0x08 == 0x08;
                self.regs.a = self.sub(self.code_to_reg(mmu, opcode), carry);
                if opcode & 0x07 == 0x06 {2} else {1}
            }
            
            // AND A, r8
            0xA0..=0xA7 => {
                self.regs.a = self.and(self.code_to_reg(mmu, opcode));
                if opcode & 0x07 == 0x06 {2} else {1}
            }

            // XOR A, r8
            0xA8..=0xAF => {
                self.regs.a = self.xor(self.code_to_reg(mmu, opcode));
                if opcode & 0x07 == 0x06 {2} else {1}
            }

            // OR A, r8
            0xB0..=0xB7 => {
                self.regs.a = self.or(self.code_to_reg(mmu, opcode));
                if opcode & 0x07 == 0x06 {2} else {1}
            }

            // CP r8
            0xB8..=0xBF => {
                self.cp(self.code_to_reg(mmu, opcode));
                if opcode & 0x07 == 0x06 {2} else {1}
            }

            0xC0 => { if !self.regs.get_flag(Flags::Z) { self.ret(mmu); 5 } else { 2 } }
            0xC1 => { let v = self.pop_stack(mmu); self.regs.set_bc(v); 3 }
            0xC2 => { self.jump_imm(mmu, !self.regs.get_flag(Flags::Z)) }
            0xC3 => { self.jump_imm(mmu, true) }
            0xC4 => { self.call(mmu, !self.regs.get_flag(Flags::Z)) }
            0xC5 => { self.push_stack(mmu, self.regs.bc()); 4 }
            0xC6 => { let v = self.read_u8(mmu); self.regs.a = self.add(v, false); 2 }
            0xC7 => { self.rst(mmu, 0x00); 4 }
            0xC8 => { if self.regs.get_flag(Flags::Z) { self.ret(mmu); 5 } else { 2 } }
            0xC9 => { self.ret(mmu); 4 }
            0xCA => { self.jump_imm(mmu, self.regs.get_flag(Flags::Z)) }
            0xCB => { self.execute_cb(mmu) }
            0xCC => { self.call(mmu, self.regs.get_flag(Flags::Z)) }
            0xCD => { self.call(mmu, true) }
            0xCE => { let v = self.read_u8(mmu); self.regs.a = self.add(v, true); 2 }
            0xCF => { self.rst(mmu, 0x08); 4 }
            
            0xD0 => { if !self.regs.get_flag(Flags::C) { self.ret(mmu); 5 } else { 2 } }
            0xD1 => { let v = self.pop_stack(mmu); self.regs.set_de(v); 3 }
            0xD2 => { self.jump_imm(mmu, !self.regs.get_flag(Flags::C)) }

            0xD4 => { self.call(mmu, !self.regs.get_flag(Flags::C)) }
            0xD5 => { self.push_stack(mmu, self.regs.de()); 4 }
            0xD6 => { let v = self.read_u8(mmu); self.regs.a = self.sub(v, false); 2 }
            0xD7 => { self.rst(mmu, 0x10); 4 }
            0xD8 => { if self.regs.get_flag(Flags::C) { self.ret(mmu); 5 } else { 2 } }
            0xD9 => { self.ret(mmu); self.enable_interrupts(mmu); 4 }
            0xDA => { self.jump_imm(mmu, self.regs.get_flag(Flags::C)) }

            0xDC => { self.call(mmu, self.regs.get_flag(Flags::C)) }

            0xDE => { let v = self.read_u8(mmu); self.regs.a = self.sub(v, true); 2 }
            0xDF => { self.rst(mmu, 0x18); 4 }

            0xE0 => { mmu.write(0xFF00 + self.read_u8(mmu) as u16, self.regs.a); 3 }
            0xE1 => { let v = self.pop_stack(mmu); self.regs.set_hl(v); 3 }
            0xE2 => { mmu.write(0xFF00 + self.regs.c as u16, self.regs.a); 2 }

            0xE5 => { self.push_stack(mmu, self.regs.hl()); 4 }
            0xE6 => { let v = self.read_u8(mmu); self.regs.a = self.and(v); 2 }
            0xE7 => { self.rst(mmu, 0x20); 4 }
            0xE8 => { self.pc = self.add_imm(mmu, self.pc); 4 }
            0xE9 => { self.pc = self.regs.hl(); 1 }
            0xEA => { let a = self.read_u16(mmu); mmu.write(a, self.regs.a); 4 }

            0xEE => { let v = self.read_u8(mmu); self.regs.a = self.xor(v); 2 }
            0xEF => { self.rst(mmu, 0x28); 4 }
            
            0xF0 => { self.regs.a = mmu.read(0xFF00 + (self.read_u8(mmu) as u16)); 3 }
            0xF1 => { let v = self.pop_stack(mmu) & 0xFFF0; self.regs.set_af(v); 3 }
            0xF2 => { self.regs.a = mmu.read(0xFF00 + self.regs.c as u16); 2 }
            0xF3 => { self.disable_interrupts(mmu); 1 }

            0xF5 => { self.push_stack(mmu, self.regs.af()); 4 }
            0xF6 => { let v = self.read_u8(mmu); self.regs.a = self.or(v); 2 }
            0xF7 => { self.rst(mmu, 0x30); 4 }
            0xF8 => { let v = self.add_imm(mmu, self.sp); self.regs.set_hl(v); 3 }
            0xF9 => { self.sp = self.regs.hl(); 2 }
            0xFA => { let a = self.read_u16(mmu); self.regs.a = mmu.read(a); 4 }
            0xFB => { self.enable_interrupts(mmu); 1 }

            0xFE => { let v = self.read_u8(mmu); self.cp(v); 2 }
            0xFF => { self.rst(mmu, 0x38); 4 }

            _ => { panic!("invalid opcode {:02X}", opcode)}
        }
    }

    fn execute_cb(&mut self, mmu: &mut Mmu) -> u8 {
        let opcode: u8 = self.read_u8(mmu);
        let mem_access = opcode & 0x07 == 0x06;

        match opcode {
            // RLC
            0x00..=0x07 => { 
                let v = self.rlc(self.code_to_reg(mmu, opcode)); 
                self.set_reg_from_code(mmu, opcode, v); 
                if mem_access {4} else {2}
            }

            // RRC
            0x08..=0x0F => { 
                let v = self.rrc(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // RL
            0x10..=0x17 => {
                let v = self.rl(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // RR
            0x18..=0x1F => {
                let v = self.rr(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // SLA
            0x20..=0x27 => {
                let v = self.sla(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // SRA
            0x28..=0x2F => {
                let v = self.sra(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // SWAP
            0x30..=0x37 => {
                let v = self.swap(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // SRL
            0x38..=0x3F => { 
                let v = self.srl(self.code_to_reg(mmu, opcode));
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // BIT
            0x40..=0x7F => {
                // Decode bit to check
                let b = (opcode >> 3) & 0x07;
                self.bit(self.code_to_reg(mmu, opcode), b);
                if mem_access{3} else {2}
            }

            // RES
            0x80..=0xBF => {
                // Decode bit to reset
                let b = (opcode >> 3) & 0x07;
                let v = self.res(self.code_to_reg(mmu, opcode), b);
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }

            // SET
            0xC0..=0xFF => {
                // Decode bit to set
                let b = (opcode >> 3) & 0x07;
                let v = self.set(self.code_to_reg(mmu, opcode), b);
                self.set_reg_from_code(mmu, opcode, v);
                if mem_access {4} else {2}
            }
        }
    }

    fn code_to_reg(&self, mmu: &Mmu, opcode: u8) -> u8 {
        match opcode & 0x07 {
            0x00 => self.regs.b,
            0x01 => self.regs.c,
            0x02 => self.regs.d,
            0x03 => self.regs.e,
            0x04 => self.regs.h,
            0x05 => self.regs.l,
            0x06 => mmu.read(self.regs.hl()),
            0x07 => self.regs.a,
            _ => panic!("invalid register code")
        }
    }

    fn set_reg_from_code(&mut self, mmu: &mut Mmu, opcode: u8, v: u8) {
        match opcode & 0x07 {
            0x00 => self.regs.b = v,
            0x01 => self.regs.c = v,
            0x02 => self.regs.d = v,
            0x03 => self.regs.e = v,
            0x04 => self.regs.h = v,
            0x05 => self.regs.l = v,
            0x06 => mmu.write(self.regs.hl(), v),
            0x07 => self.regs.a = v,
            _ => panic!("invalid register code")
        }
    }

    fn read_u8(&mut self, mmu: &Mmu) -> u8 {
        // Read a u8 immediate and increment PC
        let v = mmu.read(self.pc);
        self.pc += 1;
        v
    }

    fn read_u16(&mut self, mmu: &Mmu) -> u16 {
        // Read a u16 immediate and increment PC by 2
        let v = mmu.read_word(self.pc);
        self.pc += 2;
        v
    }

    fn increment(&mut self, a: u8) -> u8 {
        // Increment a u8 value, setting flags as required
        let res = a.wrapping_add(1);
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::H, (a & 0x0F) + 1 > 0x0F);
        self.regs.set_flag(Flags::N, false);
        res
    }

    fn decrement(&mut self, a: u8) -> u8 {
        // Decrement a u8 value, setting flags as required
        let res = a.wrapping_sub(1);
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::H, (a & 0x0F) == 0);
        self.regs.set_flag(Flags::N, true);
        res
    }

    fn add(&mut self, rhs: u8, with_carry: bool) -> u8 {
        // Addition between register A and provided value. Optionally also adds value of carry flag
        let lhs = self.regs.a;
        let c = if with_carry && self.regs.get_flag(Flags::C) { 1 } else { 0 };
        let res = lhs.wrapping_add(rhs).wrapping_add(c);
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, (lhs & 0xF) + (rhs & 0xF) + c > 0xF);
        self.regs.set_flag(Flags::C, (lhs as u16) + (rhs as u16) + (c as u16) > 0xFF);
        res
    }

    fn sub(&mut self, rhs: u8, with_carry: bool) -> u8 {
        // Subtraction between register A and provided value. Optionally also subtracts value of carry flag
        let lhs = self.regs.a;
        let c = if with_carry && self.regs.get_flag(Flags::C) { 1 } else { 0 };
        let res = lhs.wrapping_sub(rhs).wrapping_sub(c);
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::N, true);
        self.regs.set_flag(Flags::H, (lhs & 0x0F) < (rhs & 0x0F) + c);
        self.regs.set_flag(Flags::C, (lhs as u16) < (rhs as u16) + (c as u16));
        res
    }

    fn and(&mut self, rhs: u8) -> u8 {
        // Bitwise AND between register A and provided value. Sets Z, H flags, clears N, C flags
        let lhs = self.regs.a;
        let res = lhs & rhs;
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, true);
        self.regs.set_flag(Flags::C, false);
        res
    }

    fn xor(&mut self, rhs: u8) -> u8 {
        // Bitwise XOR between register A and provided value. Sets Z flag, clears N, H, C flags
        let lhs = self.regs.a;
        let res = lhs ^ rhs;
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);
        self.regs.set_flag(Flags::C, false);
        res
    }

    fn or(&mut self, rhs: u8) -> u8 {
        // Bitwise OR between register A and provided value. Sets Z flag, clears N, H, C flags
        let lhs = self.regs.a;
        let res = lhs | rhs;
        self.regs.set_flag(Flags::Z, res == 0);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);
        self.regs.set_flag(Flags::C, false);
        res
    }

    fn cp(&mut self, rhs: u8) {
        // Compare register A with target value. Result goes in Z flag, implemented via subtraction with discarded results
        self.sub(rhs, false);
    }

    fn rl(&mut self, v: u8) -> u8 {
        // Rotate left, through carry (9-bit rotate)
        let old_carry = if self.regs.get_flag(Flags::C) {1} else {0};
        self.regs.set_flag(Flags::C, v & 0x80 == 0x80);
        
        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        // Shift in old carry value
        let result = (v << 1) | old_carry;
    
        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn rlc(&mut self, v: u8) -> u8 {
        // Rotate left, copying old bit 7 to carry (also becomes bit 0)
        self.regs.set_flag(Flags::C, v & 0x80 == 0x80);
        let result = v.rotate_left(1);

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn rr(&mut self, v: u8) -> u8 {
        // Rotate right, through carry (9-bit rotate)
        let old_carry = if self.regs.get_flag(Flags::C) {1} else {0};
        self.regs.set_flag(Flags::C, v & 0x01 == 0x01);
        
        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        // Shift in old carry value
        let result = (v >> 1) | (old_carry << 7);
    
        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn rrc(&mut self, v: u8) -> u8 {
        // Rotate right, copying old bit 7 to carry (also becomes bit 0)
        self.regs.set_flag(Flags::C, v & 0x01 == 0x01);
        let result = v.rotate_right(1);

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn sla(&mut self, v: u8) -> u8 {
        // Shift left into carry
        self.regs.set_flag(Flags::C, v & 0x80 == 0x80);
        let result = v << 1;

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn sra(&mut self, v: u8) -> u8 {
        // Shift right into carry, MSB remains the same
        self.regs.set_flag(Flags::C, v & 0x01 == 0x01);
        let result = (v >> 1) | (v & 0x80);

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn swap(&mut self, v: u8) -> u8 {
        // Swap upper and lower nibbles of byte
        let result = (v << 4) | (v >> 4);

        self.regs.set_flag(Flags::Z, result == 0);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);
        self.regs.set_flag(Flags::C, false);

        result
    }

    fn srl(&mut self, v: u8) -> u8 {
        // Shift right into carry, MSB becomes 0
        self.regs.set_flag(Flags::C, v & 0x01 == 0x01);
        let result = v >> 1;

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, false);

        self.regs.set_flag(Flags::Z, result == 0);
        result
    }

    fn bit(&mut self, v: u8, b: u8) {
        // Check bit b in v
        self.regs.set_flag(Flags::Z, v & (1 << b) != 0);

        // Reset flags
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, true);
    }

    fn res(&mut self, v: u8, b: u8) -> u8 {
        // Reset bit b in v
        v & !(1 << b)
    }

    fn set(&mut self, v: u8, b: u8) -> u8 {
        // Set bit b in v
        v | (1 << b)
    }

    fn add_regs(&mut self, lhs: u16, rhs: u16) -> u16 {
        // Add two u16 values, setting registers as needed
        let res = lhs.wrapping_add(rhs);
        self.regs.set_flag(Flags::H, ((lhs & 0x0FFF) + (rhs & 0x0FFF)) > 0x0FFF);
        self.regs.set_flag(Flags::C, lhs > 0xFFFF - rhs);
        self.regs.set_flag(Flags::N, false);
        res
    }

    fn add_imm(&mut self, mmu: &Mmu, lhs: u16) -> u16 {
        // Add an immediate i8 to a provided u16. Sets flags
        let rhs = self.read_u8(mmu) as i8 as i16 as u16;
        let res = lhs.wrapping_add(rhs);
        self.regs.set_flag(Flags::Z, false);
        self.regs.set_flag(Flags::N, false);
        self.regs.set_flag(Flags::H, (lhs & 0x000F) + (rhs & 0x000F) > 0x000F);
        self.regs.set_flag(Flags::C, (lhs & 0x00FF) + (rhs & 0x00FF) > 0x00FF);
        res
    }

    fn jump_imm(&mut self, mmu: &Mmu, condition: bool) -> u8 {
        // Jump to an immediate u16 address
        let addr = self.read_u16(mmu);
        if condition {
            self.pc = addr;
            4
        } else {
            3
        }
    }

    fn jump_rel(&mut self, mmu: &Mmu) {
        // Jump relative to the current PC by an immediate i8 value
        let r = self.read_u8(mmu) as i8;
        self.pc = ((self.pc as u32 as i32) + (r as i32)) as u16;
    }

    fn push_stack(&mut self, mmu: &mut Mmu, value: u16) {
        // Decrement stack pointer and write value
        self.sp -= 2;
        mmu.write_word(self.sp, value);
    }

    fn pop_stack(&mut self, mmu: &Mmu) -> u16 {
        // Pop a value off the stack, then increment stack pointer 2
        let v = mmu.read_word(self.sp);
        self.sp += 2;
        v
    }

    fn call(&mut self, mmu: &mut Mmu, condition: bool) -> u8 {
        // Call the function at an immediate address by pushing the current PC to stack and setting PC
        // Make sure to read the address before pushing the PC onto the stack!
        let addr = self.read_u16(mmu);
        if condition {
            self.push_stack(mmu, self.pc);
            self.pc = addr;
            6
        } else {
            3
        }
    }

    fn ret(&mut self, mmu: &Mmu) {
        // Return from the current function by setting the PC to the popped stack value
        self.pc = self.pop_stack(mmu)
    }

    fn rst(&mut self, mmu: &mut Mmu, addr: u16) {
        // Push the current address to the stack and reset to address
        self.push_stack(mmu, self.pc);
        self.pc = addr;
    }

    fn enable_interrupts(&mut self, mmu: &mut Mmu) {
        // TODO: does anything else have to happen here?
        mmu.write(0xFFFF, 1);
    }

    fn disable_interrupts(&mut self, mmu: &mut Mmu) {
        mmu.write(0xFFFF, 0);
    }
}