use crate::cpu::Cpu;
use crate::cartridge::{self};
use crate::memory::{MemoryType, DMGMemory};
use crate::mmu::Mmu;
use crate::timers::Timers;

pub struct Motherboard {
    pub cpu: Cpu,
    pub mmu: Mmu,
    pub timers: Timers,
}

impl Motherboard {
    pub fn new(cart_rom: &Vec<u8>) -> Self {
        Motherboard {
            cpu: Cpu::new(),
            mmu: Mmu::new(
                MemoryType::DMGMemory(DMGMemory::new()),
                cartridge::load_cartridge(cart_rom),
            ),
            timers: Timers::new(),
        }
    }

    pub fn tick(&mut self) {
        let mcycles = self.cpu.execute(&mut self.mmu);
        self.timers.tick(&mut self.mmu, mcycles);
    }
}