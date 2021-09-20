use enum_dispatch::enum_dispatch;

#[enum_dispatch(MemoryType)]
pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

#[enum_dispatch]
pub enum MemoryType {
    DMGMemory,
}

pub struct DMGMemory {
    vram: [u8; 8192],
    wram: [u8; 8192],
    oam: [u8; 160],
    io_regs: [u8; 512],
    high_ram: [u8; 512],
}

impl DMGMemory {
    pub fn new() -> Self {
        let mut mem = DMGMemory { vram: [0; 8192], wram: [0; 8192], oam: [0; 160], io_regs: [0xFF; 512], high_ram: [0; 512] };
        mem.io_regs[0x0040] = 0x85;
        mem.io_regs[0x0042] = 0;
        mem.io_regs[0x0043] = 0;
        mem.io_regs[0x0044] = 0;
        mem
    }
}

impl Memory for DMGMemory {
    fn read(&self, addr: u16) -> u8 {
        if addr >= 0x8000 && addr < 0xA000 {
            self.vram[addr as usize - 0x8000]
        } else if addr >= 0xC000 && addr < 0xE000 {
            self.wram[addr as usize - 0xC000]
        } else if addr >= 0xFE00 && addr < 0xFEA0 {
            self.oam[addr as usize - 0xFE00]
        } else if addr >= 0xFF00 && addr < 0xFF80 {
            self.io_regs[addr as usize - 0xFF00]
        } else if addr >= 0xFF80 {
            self.high_ram[addr as usize - 0xFF80]
        } else {
            panic!("invalid DMGMemory read address: {}", addr);
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr >= 0x8000 && addr < 0xA000 {
            self.vram[addr as usize - 0x8000] = value;
        } else if addr >= 0xC000 && addr < 0xE000 {
            self.wram[addr as usize - 0xC000] = value;
        } else if addr >= 0xFE00 && addr < 0xFEA0 {
            self.oam[addr as usize - 0xFE00] = value;  
        } else if addr >= 0xFF00 && addr < 0xFF80 {
            self.io_regs[addr as usize - 0xFF00] = value;
            if addr == 0xFF46 {
                self.do_dma(value);
            }
        } else if addr >= 0xFF80 {
            self.high_ram[addr as usize - 0xFF80] = value;
        } else {
            panic!("invalid DMGMemory read address: {}", addr);
        }
    }
}

impl DMGMemory {
    fn do_dma(&mut self, src: u8) {
        for i in 0..=0x9F {
            self.write(0xFE00 | i, self.read((src as u16) << 8 | i));
        }
    }
}
