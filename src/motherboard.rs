use crate::cpu::Cpu;
use crate::cartridge::{self};
use crate::memory::{MemoryType, DMGMemory};
use crate::mmu::Mmu;

pub struct Motherboard {
    pub cpu: Cpu,
    pub mmu: Mmu,
}

impl Motherboard {
    pub fn new(cart_rom: &Vec<u8>) -> Self {
        Motherboard {
            cpu: Cpu::new(),
            mmu: Mmu::new(
                MemoryType::DMGMemory(DMGMemory::new()),
                cartridge::load_cartridge(cart_rom),
            )
        }
    }

    pub fn tick(&mut self) {
        self.cpu.execute(&mut self.mmu);
    }
}