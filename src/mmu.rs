use crate::memory::{MemoryType, Memory};
use crate::cartridge::{CartridgeType, Cartridge};

pub struct Mmu {
    memory: MemoryType,
    cartridge: CartridgeType,
}

impl Mmu {
    pub fn new(memory: MemoryType, cartridge: CartridgeType) -> Self {
        Mmu {memory, cartridge}
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            // Cartridge ROM
            self.cartridge.read(addr)
        } else if addr >= 0x8000 && addr < 0xA000 {
            // Video RAM
            self.memory.read(addr)
        } else if addr >= 0xA000 && addr < 0xC000 {
            // Cartridge RAM
            self.cartridge.read(addr)
        } else if addr >= 0xC000 && addr < 0xE000 {
            // Work RAM
            self.memory.read(addr)
        } else if addr >= 0xE000 && addr < 0xFE00 {
            // Echo RAM
            self.memory.read(addr - 0x2000)  
        } else if addr >= 0xFF00 {
            // IO Registers, High RAM, and Interrupt Enable Register
            self.memory.read(addr)
        } else {
            panic!("Invalid memory read at address 0x{:02X}", addr);
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 {
            // Cartridge ROM
            self.cartridge.write(addr, value);
        } else if addr >= 0x8000 && addr < 0xA000 {
            // Video RAM
            self.memory.write(addr, value);
        } else if addr >= 0xA000 && addr < 0xC000 {
            // Cartridge RAM
            self.cartridge.write(addr, value);
        } else if addr >= 0xC000 && addr < 0xE000 {
            // Work RAM
            self.memory.write(addr, value);
        } else if addr >= 0xE000 && addr < 0xFE00 {
            // Echo RAM
            self.memory.write(addr - 0x2000, value);
        } else if addr >= 0xFF00 {
            // IO Registers, High RAM, and Interrupt Enable Register
            self.memory.write(addr, value)
        } else {
            panic!("Invalid memory write at address 0x{:02X}", addr);
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        ((self.read(addr + 1) as u16) << 8) | (self.read(addr) as u16)
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        self.write(addr + 1, (value >> 8) as u8);
        self.write(addr, (value & 0xFF) as u8);
    }
}
