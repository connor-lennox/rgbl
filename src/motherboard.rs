use crate::cpu::Cpu;
use crate::cartridge::{self, CartridgeType, Cartridge};
use crate::memory::{MemoryType, DMGMemory, Memory};

pub struct Motherboard {
    pub cpu: Cpu,
    pub memory: MemoryType,
    pub cartridge: CartridgeType,
}

impl Motherboard {
    pub fn new(cart_rom: &Vec<u8>) -> Self {
        Motherboard {
            cpu: Cpu::new(),
            memory: MemoryType::DMGMemory(DMGMemory::new()),
            cartridge: cartridge::load_cartridge(cart_rom),
        }
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
        } else {
            panic!("Invalid memory read at address {}", addr);
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
        } else {
            panic!("Invalid memory write at address {}", addr);
        }
    }
}