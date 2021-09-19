use crate::{lcd::Lcd, mmu::Mmu};

#[derive(PartialEq)]
enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}

pub struct Ppu {
    mode: PpuMode,
    line_cycles: u32,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            mode: PpuMode::OAMScan,
            line_cycles: 0,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu, lcd: &mut Lcd, m_cycles: u8) {
        let t_cycles = m_cycles * 4;
        self.line_cycles += t_cycles as u32;

        // Read current LY/LYC register value
        let ly = mmu.read(0xFF44);
        let lyc = mmu.read(0xFF45);

        let stat = mmu.read(0xFF41);

        // Check for going to the next scanline
        if self.line_cycles >= 456 {
            // Write new LY register value
            let new_ly = (ly + 1) % 154;
            mmu.write(0xFF44, new_ly);

            self.line_cycles -= 456;
            self.mode = if new_ly >= 144 { 
                if (stat & 0b00010000) != 0 && self.mode != PpuMode::VBlank { self.req_stat_interrupt(mmu); }
                if self.mode != PpuMode::VBlank { self.req_vblank_interrupt(mmu); }
                PpuMode::VBlank 
            } else {
                if (stat & 0b00100000) != 0 { self.req_stat_interrupt(mmu); }
                PpuMode::OAMScan 
            };
        }

        // Reset LYC=LY flag in STAT register
        let new_stat = if ly == lyc { stat | 0b00000100 } else { stat & 0b11111011 };
        mmu.write(0xFF41, new_stat);

        // If LYC=LY and the LYC=LY STAT interrupt source is set, request a STAT interrupt
        if (ly == lyc) && (stat & 0b01000000 != 0) {
            self.req_stat_interrupt(mmu);
        }

        // Check for OAM Scan -> Drawing mode switch
        if self.line_cycles >= 80 && self.mode == PpuMode::OAMScan {
            // Push a row of pixels to the LCD (all at once, at start of mode)
            self.draw_line(mmu, lcd, ly, mmu.read(0xFF40));
            self.mode = PpuMode::Drawing;
        }

        // Check for Drawing -> HBlank mode switch
        // Normally this would be a variable number of cycles,
        // but it doesn't really matter.
        if self.line_cycles >= 252 && self.mode == PpuMode::Drawing {
            if (stat & 0b00001000) != 0 { self.req_stat_interrupt(mmu); }
            self.mode = PpuMode::HBlank;
        }
    }

    fn req_vblank_interrupt(&self, mmu: &mut Mmu) {
        mmu.write(0xFF0F, mmu.read(0xFF0F) | 0b00000001);
    }

    fn req_stat_interrupt(&self, mmu: &mut Mmu) {
        mmu.write(0xFF0F, mmu.read(0xFF0F) | 0b00000010);
    }

    fn draw_line(&self, mmu: &Mmu, lcd: &mut Lcd, ly: u8, lcdc: u8) {
        let mut line: [u8; 160] = [0; 160];

        // Background and Window only drawn if bit 0 of LCDC is set
        if lcdc & 0b00000001 != 0 {
            let tile_mode_8000 = lcdc & 0b00010000 != 0;

            // Background

            // Retrieve background scroll X and scroll Y
            let (scy, scx) = (mmu.read(0xFF42), mmu.read(0xFF43));
            // Select the background tilemap location based on bit 3 of the LCDC register
            let bg_tilemap: u16 = if lcdc & 0b00001000 != 0 { 0x9C00 } else { 0x9800 };

            for x_counter in 0..20 {
                // let addr = bg_tilemap + 
                //                 (((x_counter + ((scx as u16) & 0x1F) / 8)) + 
                //                 (32 * (((ly as u16 + scy as u16) & 0xFF) / 8)) & 0x3FF);
                let addr = 0x9800 +
                                (x_counter + scx as u16 / 8) +
                                (((ly as u16 + scy as u16) & 0xFF) / 8) * 32;
                // let addr = bg_tilemap + x_counter + 32 * (ly / 8) as u16;
                let tile_num = mmu.read(addr);
                // let tile_addr: u16 = if tile_mode_8000 { 0x8000 + (tile_num as u16) * 16 } else { 0x8800 + ((tile_num as i8 as i16 + 128) as u16 * 16) };
                let tile_addr: u16 = 0x8000 + (tile_num as u16) * 16 + ((ly as u16 + scy as u16) % 8) * 2;
                let b1 = mmu.read(tile_addr);
                let b2 = mmu.read(tile_addr + 1);

                // TODO: SCX!
                for px in 0..8 {
                    line[(x_counter * 8 + px) as usize] = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 } 
                                                            | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                }
            }
        }


        // Sprites: iterate the OAM and draw pixels on the line that we need
        // But only if LCDC bit 1 is set: enable/disable sprites 
        // if lcdc & 0b00000010 != 0 {
        //     // Sprite height based on LCDC bit 2: if set "tall-sprite" mode
        //     let sprite_height = if lcdc & 0b00000100 != 0 { 16 } else { 8 };
        //     let mut buffered_sprites = 0;
        //     for entry in (0xFE00..0xFEA0).step_by(4) {
        //         let (y, x, tidx, flags) = (mmu.read(entry), mmu.read(entry+1), mmu.read(entry+2), mmu.read(entry+3));
        //         if x > 0 && (ly + 16) >= y && (ly + 16) < y + sprite_height {
        //             buffered_sprites += 1;


        //         }

        //         // Only 10 sprites can be drawn on a single scanline
        //         if buffered_sprites > 10 { break; }
        //     }
        // }



        lcd.set_line(ly, line);
    }
}