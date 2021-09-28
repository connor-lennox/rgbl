use crate::cpu::Cpu;
use crate::cartridge::{self};
use crate::joypad::{self, Joypad};
use crate::lcd::Lcd;
use crate::memory::{MemoryType, DMGMemory};
use crate::mmu::Mmu;
use crate::timers::Timers;
use crate::ppu::Ppu;

pub struct Motherboard {
    pub cpu: Cpu,
    pub mmu: Mmu,
    pub timers: Timers,
    pub lcd: Lcd,
    pub joypad: Joypad,
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
            lcd: Lcd::new(),
            joypad: Joypad::new(),
        }
    }

    pub fn tick(&mut self) -> u8 {
        self.joypad.tick(&mut self.mmu);
        let mcycles = self.cpu.execute(&mut self.mmu);
        self.timers.tick(&mut self.mmu, mcycles);
        self.mmu.tick(&mut self.lcd, mcycles);
        mcycles
    }
}