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
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr),        // Cartridge ROM
            0x8000..=0x9FFF => self.memory.read(addr),           // Video RAM
            0xA000..=0xBFFF => self.cartridge.read(addr),        // Cartridge RAM
            0xC000..=0xDFFF => self.memory.read(addr),           // Work RAM
            0xE000..=0xFDFF => self.memory.read(addr - 0x2000),  // Echo RAM
            0xFE00..=0xFE9F => self.memory.read(addr),           // OAM
            0xFEA0..=0xFEFF => 0xFF,                             // Forbidden Memory
            0xFF00 => 0xFF,                                     // TODO: Temp input stub for testing
            0xFF00.. => self.memory.read(addr)                  // IO Regs, High RAM, Interrupt Enable Register
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, value),       // Cartridge ROM
            0x8000..=0x9FFF => self.memory.write(addr, value),          // Video RAM
            0xA000..=0xBFFF => self.cartridge.write(addr, value),       // Cartridge RAM
            0xC000..=0xDFFF => self.memory.write(addr, value),          // Work RAM
            0xE000..=0xFDFF => self.memory.write(addr - 0x2000, value), // Echo RAM
            0xFE00..=0xFE9F => self.memory.write(addr, value),          // OAM
            0xFEA0..=0xFEFF => (),                                      // Forbidden Memory
            0xFF00 => (),                                               // TODO: Temp input stub for testing
            0xFF00.. => self.memory.write(addr, value)                  // IO Regs, High RAM, Interrupt Enable Register
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
