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

struct CpuRegisters {
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
            flags: 0x80,
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

    pub fn get_flag(&self, flag: Flags) -> bool {
        (self.flags & flag.as_bits()) != 0
    }

    pub fn set_flag(&mut self, flag: Flags, value: bool) {
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
    regs: CpuRegisters,

    sp: u16,
    pc: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: CpuRegisters::new(),

            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    fn execute(&mut self, mmu: &mut Mmu) -> u8 {
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
            0x07 => { self.regs.a = self.rl(self.regs.a, true); 1 }
            0x08 => { let a = self.read_u16(mmu); mmu.write_word(a, self.sp); 5 }
            0x09 => { let v = self.add_regs(self.regs.hl(), self.regs.bc()); self.regs.set_hl(v); 2 }
            0x0A => { self.regs.a = mmu.read(self.regs.bc()); 2 }
            0x0B => { self.regs.set_bc(self.regs.bc().wrapping_sub(1)); 2 }
            0x0C => { self.regs.c = self.increment(self.regs.c); 1 }
            0x0D => { self.regs.c = self.decrement(self.regs.c); 1 }
            0x0E => { self.regs.c = self.read_u8(mmu); 2 }
            0x0F => { self.regs.a = self.rr(self.regs.a, true); 1 }

            0x10 => { todo!("stop"); 1 }
            0x11 => { let v = self.read_u16(mmu); self.regs.set_de(v); 3 }
            0x12 => { mmu.write(self.regs.de(), self.regs.a); 2 }
            0x13 => { self.regs.set_de(self.regs.de().wrapping_add(1)); 2 }
            0x14 => { self.regs.d = self.increment(self.regs.d); 1 }
            0x15 => { self.regs.d = self.decrement(self.regs.d); 1 }
            0x16 => { self.regs.d = self.read_u8(mmu); 2 }
            0x17 => { self.regs.a = self.rl(self.regs.a, false); 1 }
            0x18 => { self.jump_rel(mmu); 3 }
            0x19 => { let v = self.add_regs(self.regs.hl(), self.regs.de()); self.regs.set_hl(v); 2 }
            0x1A => { self.regs.a = mmu.read(self.regs.de()); 2 }
            0x1B => { self.regs.set_de(self.regs.de().wrapping_sub(1)); 2 }
            0x1C => { self.regs.e = self.increment(self.regs.e); 1 }
            0x1D => { self.regs.e = self.decrement(self.regs.e); 1 }
            0x1E => { self.regs.e = self.read_u8(mmu); 2 }
            0x1F => { self.regs.a = self.rr(self.regs.a, false); 1 }

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
            0x32 => { mmu.write(self.regs.hli(), self.regs.a); 2 }
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

            0x40 => { self.regs.b = self.regs.b; 1 }
            0x41 => { self.regs.b = self.regs.c; 1 }
            0x42 => { self.regs.b = self.regs.d; 1 }
            0x43 => { self.regs.b = self.regs.e; 1 }
            0x44 => { self.regs.b = self.regs.h; 1 }
            0x45 => { self.regs.b = self.regs.l; 1 }
            0x46 => { self.regs.b = mmu.read(self.regs.hl()); 2 }
            0x47 => { self.regs.b = self.regs.a; 1 }
            0x48 => { self.regs.c = self.regs.b; 1 }
            0x49 => { self.regs.c = self.regs.c; 1 }
            0x4A => { self.regs.c = self.regs.d; 1 }
            0x4B => { self.regs.c = self.regs.e; 1 }
            0x4C => { self.regs.c = self.regs.h; 1 }
            0x4D => { self.regs.c = self.regs.l; 1 }
            0x4E => { self.regs.c = mmu.read(self.regs.hl()); 2 }
            0x4F => { self.regs.c = self.regs.a; 1 }

            0x50 => { self.regs.d = self.regs.b; 1 }
            0x51 => { self.regs.d = self.regs.c; 1 }
            0x52 => { self.regs.d = self.regs.d; 1 }
            0x53 => { self.regs.d = self.regs.e; 1 }
            0x54 => { self.regs.d = self.regs.h; 1 }
            0x55 => { self.regs.d = self.regs.l; 1 }
            0x56 => { self.regs.d = mmu.read(self.regs.hl()); 2 }
            0x57 => { self.regs.d = self.regs.a; 1 }
            0x58 => { self.regs.e = self.regs.b; 1 }
            0x59 => { self.regs.e = self.regs.c; 1 }
            0x5A => { self.regs.e = self.regs.d; 1 }
            0x5B => { self.regs.e = self.regs.e; 1 }
            0x5C => { self.regs.e = self.regs.h; 1 }
            0x5D => { self.regs.e = self.regs.l; 1 }
            0x5E => { self.regs.e = mmu.read(self.regs.hl()); 2 }
            0x5F => { self.regs.e = self.regs.a; 1 }

            0x60 => { self.regs.h = self.regs.b; 1 }
            0x61 => { self.regs.h = self.regs.c; 1 }
            0x62 => { self.regs.h = self.regs.d; 1 }
            0x63 => { self.regs.h = self.regs.e; 1 }
            0x64 => { self.regs.h = self.regs.h; 1 }
            0x65 => { self.regs.h = self.regs.l; 1 }
            0x66 => { self.regs.h = mmu.read(self.regs.hl()); 2 }
            0x67 => { self.regs.h = self.regs.a; 1 }
            0x68 => { self.regs.l = self.regs.b; 1 }
            0x69 => { self.regs.l = self.regs.c; 1 }
            0x6A => { self.regs.l = self.regs.d; 1 }
            0x6B => { self.regs.l = self.regs.e; 1 }
            0x6C => { self.regs.l = self.regs.h; 1 }
            0x6D => { self.regs.l = self.regs.l; 1 }
            0x6E => { self.regs.l = mmu.read(self.regs.hl()); 2 }
            0x6F => { self.regs.l = self.regs.a; 1 }
            
            0x70 => { mmu.write(self.regs.hl(), self.regs.b); 2 }
            0x71 => { mmu.write(self.regs.hl(), self.regs.c); 2 }
            0x72 => { mmu.write(self.regs.hl(), self.regs.d); 2 }
            0x73 => { mmu.write(self.regs.hl(), self.regs.e); 2 }
            0x74 => { mmu.write(self.regs.hl(), self.regs.h); 2 }
            0x75 => { mmu.write(self.regs.hl(), self.regs.l); 2 }
            0x76 => { todo!("halt"); 1 }
            0x77 => { mmu.write(self.regs.hl(), self.regs.a); 2 }
            0x78 => { self.regs.a = self.regs.b; 1 }
            0x79 => { self.regs.a = self.regs.c; 1 }
            0x7A => { self.regs.a = self.regs.d; 1 }
            0x7B => { self.regs.a = self.regs.e; 1 }
            0x7C => { self.regs.a = self.regs.h; 1 }
            0x7D => { self.regs.a = self.regs.l; 1 }
            0x7E => { self.regs.a = mmu.read(self.regs.hl()); 2 }
            0x7F => { self.regs.a = self.regs.a; 1 }

            0x80 => { self.regs.a = self.add(self.regs.b, false); 1 }
            0x81 => { self.regs.a = self.add(self.regs.c, false); 1 }
            0x82 => { self.regs.a = self.add(self.regs.d, false); 1 }
            0x83 => { self.regs.a = self.add(self.regs.e, false); 1 }
            0x84 => { self.regs.a = self.add(self.regs.h, false); 1 }
            0x85 => { self.regs.a = self.add(self.regs.l, false); 1 }
            0x86 => { self.regs.a = self.add(mmu.read(self.regs.hl()), false); 2 }
            0x87 => { self.regs.a = self.add(self.regs.a, false); 1 }
            0x88 => { self.regs.a = self.add(self.regs.b, true); 1 }
            0x89 => { self.regs.a = self.add(self.regs.c, true); 1 }
            0x8A => { self.regs.a = self.add(self.regs.d, true); 1 }
            0x8B => { self.regs.a = self.add(self.regs.e, true); 1 }
            0x8C => { self.regs.a = self.add(self.regs.h, true); 1 }
            0x8D => { self.regs.a = self.add(self.regs.l, true); 1 }
            0x8E => { self.regs.a = self.add(mmu.read(self.regs.hl()), true); 2 }
            0x8F => { self.regs.a = self.add(self.regs.a, true); 1 }

            0x90 => { self.regs.a = self.sub(self.regs.b, false); 1 }
            0x91 => { self.regs.a = self.sub(self.regs.c, false); 1 }
            0x92 => { self.regs.a = self.sub(self.regs.d, false); 1 }
            0x93 => { self.regs.a = self.sub(self.regs.e, false); 1 }
            0x94 => { self.regs.a = self.sub(self.regs.h, false); 1 }
            0x95 => { self.regs.a = self.sub(self.regs.l, false); 1 }
            0x96 => { self.regs.a = self.sub(mmu.read(self.regs.hl()), false); 2 }
            0x97 => { self.regs.a = self.sub(self.regs.a, false); 1 }
            0x98 => { self.regs.a = self.sub(self.regs.b, true); 1 }
            0x99 => { self.regs.a = self.sub(self.regs.c, true); 1 }
            0x9A => { self.regs.a = self.sub(self.regs.d, true); 1 }
            0x9B => { self.regs.a = self.sub(self.regs.e, true); 1 }
            0x9C => { self.regs.a = self.sub(self.regs.h, true); 1 }
            0x9D => { self.regs.a = self.sub(self.regs.l, true); 1 }
            0x9E => { self.regs.a = self.sub(mmu.read(self.regs.hl()), true); 2 }
            0x9F => { self.regs.a = self.sub(self.regs.a, true); 1 }
            
            0xA0 => { self.regs.a = self.and(self.regs.b); 1 }
            0xA1 => { self.regs.a = self.and(self.regs.c); 1 }
            0xA2 => { self.regs.a = self.and(self.regs.d); 1 }
            0xA3 => { self.regs.a = self.and(self.regs.e); 1 }
            0xA4 => { self.regs.a = self.and(self.regs.h); 1 }
            0xA5 => { self.regs.a = self.and(self.regs.l); 1 }
            0xA6 => { self.regs.a = self.and(mmu.read(self.regs.hl())); 2 }
            0xA7 => { self.regs.a = self.and(self.regs.a); 1 }
            0xA8 => { self.regs.a = self.xor(self.regs.b); 1 }
            0xA9 => { self.regs.a = self.xor(self.regs.c); 1 }
            0xAA => { self.regs.a = self.xor(self.regs.d); 1 }
            0xAB => { self.regs.a = self.xor(self.regs.e); 1 }
            0xAC => { self.regs.a = self.xor(self.regs.h); 1 }
            0xAD => { self.regs.a = self.xor(self.regs.l); 1 }
            0xAE => { self.regs.a = self.xor(mmu.read(self.regs.hl())); 2 }
            0xAF => { self.regs.a = self.xor(self.regs.a); 1 }

            0xB0 => { self.regs.a = self.or(self.regs.b); 1 }
            0xB1 => { self.regs.a = self.or(self.regs.c); 1 }
            0xB2 => { self.regs.a = self.or(self.regs.d); 1 }
            0xB3 => { self.regs.a = self.or(self.regs.e); 1 }
            0xB4 => { self.regs.a = self.or(self.regs.h); 1 }
            0xB5 => { self.regs.a = self.or(self.regs.l); 1 }
            0xB6 => { self.regs.a = self.or(mmu.read(self.regs.hl())); 2 }
            0xB7 => { self.regs.a = self.or(self.regs.a); 1 }
            0xB8 => { self.cp(self.regs.b); 1 }
            0xB9 => { self.cp(self.regs.c); 1 }
            0xBA => { self.cp(self.regs.d); 1 }
            0xBB => { self.cp(self.regs.e); 1 }
            0xBC => { self.cp(self.regs.h); 1 }
            0xBD => { self.cp(self.regs.l); 1 }
            0xBE => { self.cp(mmu.read(self.regs.hl())); 2 }
            0xBF => { self.cp(self.regs.a); 1 }

            0xC0 => { if !self.regs.get_flag(Flags::Z) { self.ret(mmu); 5 } else { 2 } }
            0xC1 => { let v = self.pop_stack(mmu); self.regs.set_bc(v); 3 }
            0xC2 => { if !self.regs.get_flag(Flags::Z) { self.jump_imm(mmu); 4 } else { 3 } }
            0xC3 => { self.jump_imm(mmu); 4 }
            0xC4 => { if !self.regs.get_flag(Flags::Z) { self.call(mmu); 6 } else { 3 } }
            0xC5 => { self.push_stack(mmu, self.regs.bc()); 4 }
            0xC6 => { let v = self.read_u8(mmu); self.regs.a = self.add(v, false); 2 }
            0xC7 => { self.rst(mmu, 0x00); 4 }
            0xC8 => { if self.regs.get_flag(Flags::Z) { self.ret(mmu); 5 } else { 2 } }
            0xC9 => { self.ret(mmu); 4 }
            0xCA => { if self.regs.get_flag(Flags::Z) { self.jump_imm(mmu); 4 } else { 3 } }
            0xCB => { todo!("CB") }
            0xCC => { if self.regs.get_flag(Flags::Z) { self.call(mmu); 6 } else { 3 } }
            0xCD => { self.call(mmu); 6 }
            0xCE => { let v = self.read_u8(mmu); self.regs.a = self.add(v, true); 2 }
            0xCF => { self.rst(mmu, 0x08); 4 }
            
            0xD0 => { if !self.regs.get_flag(Flags::C) { self.ret(mmu); 5 } else { 2 } }
            0xD1 => { let v = self.pop_stack(mmu); self.regs.set_de(v); 3 }
            0xD2 => { if !self.regs.get_flag(Flags::C) { self.jump_imm(mmu); 4 } else { 3 } }
            0xD3 => { panic!("invalid opcode 0xD3") }
            0xD4 => { if !self.regs.get_flag(Flags::C) { self.call(mmu); 6 } else { 3 } }
            0xD5 => { self.push_stack(mmu, self.regs.de()); 4 }
            0xD6 => { let v = self.read_u8(mmu); self.regs.a = self.sub(v, false); 2 }
            0xD7 => { self.rst(mmu, 0x10); 4 }
            0xD8 => { if self.regs.get_flag(Flags::C) { self.ret(mmu); 5 } else { 2 } }
            0xD9 => { self.ret(mmu); self.enable_interrupts(); 4 }
            0xDA => { if self.regs.get_flag(Flags::C) { self.jump_imm(mmu); 4 } else { 3 } }
            0xDB => { panic!("invalid opcode 0xDB") }
            0xDC => { if self.regs.get_flag(Flags::C) { self.call(mmu); 6 } else { 3 } }
            0xDD => { panic!("invalid opcode 0xDD") }
            0xDE => { let v = self.read_u8(mmu); self.regs.a = self.sub(v, true); 2 }
            0xDF => { self.rst(mmu, 0x18); 4 }

            0xE0 => { mmu.write(0xFF00 + self.read_u8(mmu) as u16, self.regs.a); 3 }
            0xE1 => { let v = self.pop_stack(mmu); self.regs.set_hl(v); 3 }
            0xE2 => { mmu.write(0xFF00 + self.regs.c as u16, self.regs.a); 2 }
            0xE3 => { panic!("invalid opcode 0xE3") }
            0xE4 => { panic!("invalid opcode 0xE4") }
            0xE5 => { self.push_stack(mmu, self.regs.hl()); 4 }
            0xE6 => { let v = self.read_u8(mmu); self.regs.a = self.and(v); 2 }
            0xE7 => { self.rst(mmu, 0x20); 4 }
            0xE8 => { self.pc = self.add_imm(mmu, self.pc); 4 }
            0xE9 => { self.pc = self.regs.hl(); 1 }
            0xEA => { let a = self.read_u16(mmu); mmu.write(a, self.regs.a); 4 }
            0xEB => { panic!("invalid opcode 0xEB") }
            0xEC => { panic!("invalid opcode 0xEC") }
            0xED => { panic!("invalid opcode 0xED") }
            0xEE => { let v = self.read_u8(mmu); self.regs.a = self.xor(v); 2 }
            0xEF => { self.rst(mmu, 0x28); 4 }
            
            0xF0 => { self.regs.a = mmu.read(0xFF00 + self.read_u8(mmu) as u16); 3 }
            0xF1 => { let v = self.pop_stack(mmu) & 0xFFF0; self.regs.set_af(v); 3 }
            0xF2 => { self.regs.a = mmu.read(0xFF00 + self.regs.c as u16); 2 }
            0xF3 => { self.disable_interrupts(); 1 }
            0xF4 => { panic!("invalid opcode 0xF4") }
            0xF5 => { self.push_stack(mmu, self.regs.af()); 4 }
            0xF6 => { let v = self.read_u8(mmu); self.regs.a = self.or(v); 2 }
            0xF7 => { self.rst(mmu, 0x30); 4 }
            0xF8 => { let v = self.add_imm(mmu, self.sp); self.regs.set_hl(v); 3 }
            0xF9 => { self.sp = self.regs.hl(); 2 }
            0xFA => { let a = self.read_u16(mmu); self.regs.a = mmu.read(a); 4 }
            0xFB => { self.enable_interrupts(); 1 }
            0xFC => { panic!("invalid opcode 0xFC") }
            0xFD => { panic!("invalid opcode 0xFD") }
            0xFE => { let v = self.read_u8(mmu); self.cp(v); 2 }
            0xFF => { self.rst(mmu, 0x38); 4 }
            
            _ => todo!("unimplemented opcode: {}", opcode)
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
        let v = ((mmu.read(self.pc) as u16) << 8) | (mmu.read(self.pc + 1) as u16);
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

    fn rl(&mut self, v: u8, set_carry: bool) -> u8 {
        // Rotate left, optionally set carry flag to shifted out value
        if set_carry {
            self.regs.set_flag(Flags::C, v & 0x80 != 0);
        }
        v << 1
    }

    fn rr(&mut self, v: u8, set_carry: bool) -> u8 {
        // Rotate right, optionally set carry flag to shifted out value
        if set_carry {
            self.regs.set_flag(Flags::C, v & 0x01 != 0);
        }
        v >> 1
    }

    fn add_regs(&mut self, lhs: u16, rhs: u16) -> u16 {
        // Add two u16 values, setting registers as needed
        let res = lhs.wrapping_add(rhs);
        self.regs.set_flag(Flags::H, (lhs & 0x07FF) + (rhs & 0x07FF) > 0x07FF);
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

    fn jump_imm(&mut self, mmu: &Mmu) {
        // Jump to an immediate u16 address
        self.pc = self.read_u16(mmu);
    }

    fn jump_rel(&mut self, mmu: &Mmu) {
        // Jump relative to the current PC by an immediate i8 value
        let r = self.read_u8(mmu) as i8;
        self.pc = ((self.pc as u32 as i32) + (r as i32)) as u16;
    }

    fn push_stack(&mut self, mmu: &mut Mmu, value: u16) {
        // Push a value onto the stack, then decrement stack pointer 2
        mmu.write_word(self.sp - 1, value);
        self.sp -= 2;
    }

    fn pop_stack(&mut self, mmu: &Mmu) -> u16 {
        // Pop a value off the stack, then increment stack pointer 2
        self.sp += 2;
        mmu.read_word(self.sp - 1)
    }

    fn call(&mut self, mmu: &mut Mmu) {
        // Call the function at an immediate address by pushing the current PC to stack and setting PC
        self.push_stack(mmu, self.pc);
        self.pc = self.read_u16(mmu);
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

    fn enable_interrupts(&mut self) {
        todo!("enable interrupts");
    }

    fn disable_interrupts(&mut self) {
        todo!("disable interrupts");
    }
}