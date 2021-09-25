use crate::lcd::Lcd;
use crate::memory::{MemoryType, Memory};
use crate::cartridge::{CartridgeType, Cartridge};
use crate::ppu::Ppu;

pub struct Mmu {
    ppu: Ppu,
    memory: MemoryType,
    cartridge: CartridgeType,
}

impl Mmu {
    pub fn new(memory: MemoryType, cartridge: CartridgeType) -> Self {
        Mmu {ppu: Ppu::new(), memory, cartridge}
    }

    pub fn tick(&mut self, lcd: &mut Lcd, m_cycles: u8) {
        self.ppu.tick(lcd, m_cycles);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr),        // Cartridge ROM
            0x8000..=0x9FFF => self.ppu.read(addr),              // Video RAM
            0xA000..=0xBFFF => self.cartridge.read(addr),        // Cartridge RAM
            0xC000..=0xDFFF => self.memory.read(addr),           // Work RAM
            0xE000..=0xFDFF => self.memory.read(addr - 0x2000),  // Echo RAM
            0xFE00..=0xFE9F => self.ppu.read(addr),              // OAM
            0xFEA0..=0xFEFF => 0xFF,                             // Forbidden Memory
            0xFF00 => 0xFF,                                      // TODO: Temp input stub for testing
            0xFF00..=0xFF7F => self.ppu.read(addr),              // IO Regs
            0xFF80.. => self.memory.read(addr)                   // High RAM, Interrupt Enable Register
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, value),       // Cartridge ROM
            0x8000..=0x9FFF => self.ppu.write(addr, value),             // Video RAM
            0xA000..=0xBFFF => self.cartridge.write(addr, value),       // Cartridge RAM
            0xC000..=0xDFFF => self.memory.write(addr, value),          // Work RAM
            0xE000..=0xFDFF => self.memory.write(addr - 0x2000, value), // Echo RAM
            0xFE00..=0xFE9F => self.ppu.write(addr, value),             // OAM
            0xFEA0..=0xFEFF => (),                                      // Forbidden Memory
            0xFF00 => (),                                               // TODO: Temp input stub for testing
            0xFF00..=0xFF7F => {                                        // IO Regs
                self.ppu.write(addr, value);
                if addr == 0xFF46 {
                    let mut data: [u8; 160] = [0; 160];
                    let value_base = (value as u16) << 8;
                    for i in 0x00..=0x9F {
                        data[i as usize] = self.read(value_base | i);
                    }
                    self.ppu.dma(&data);
                }
            }
            0xFF80.. => self.memory.write(addr, value)                  // High RAM, Interrupt Enable Register
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
