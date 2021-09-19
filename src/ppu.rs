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
    reached_window: bool,
    window_line_counter: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            mode: PpuMode::OAMScan,
            line_cycles: 0,
            reached_window: false,
            window_line_counter: 0,
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

            // Indicate when window reached
            if new_ly >= mmu.read(0xFF4A) { self.reached_window = true; }

            mmu.write(0xFF44, new_ly);

            // If LYC=LY and the LYC=LY STAT interrupt source is set, request a STAT interrupt
            if (new_ly == lyc) && (stat & 0b01000000 != 0) {
                self.req_stat_interrupt(mmu);
            }

            self.line_cycles -= 456;
            self.mode = if new_ly >= 144 {
                if self.mode != PpuMode::VBlank {
                    if (stat & 0b00010000) != 0 { self.req_stat_interrupt(mmu); }
                    self.req_vblank_interrupt(mmu);
                    self.reached_window = false;
                    self.window_line_counter = 0;
                }
                PpuMode::VBlank 
            } else {
                if (stat & 0b00100000) != 0 { self.req_stat_interrupt(mmu); }
                PpuMode::OAMScan 
            };
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

        // Reset LYC=LY flag in STAT register
        let mut new_stat = if ly == lyc { stat | 0b00000100 } else { stat & 0b11111011 };
        // Set ppu mode in STAT register bits 0-1
        new_stat &= 0b11111100;
        new_stat |= match self.mode {
            PpuMode::HBlank => 0,
            PpuMode::VBlank => 1,
            PpuMode::OAMScan => 2,
            PpuMode::Drawing => 3,
        };
        mmu.write(0xFF41, new_stat);
    }

    fn req_vblank_interrupt(&self, mmu: &mut Mmu) {
        mmu.write(0xFF0F, mmu.read(0xFF0F) | 0b00000001);
    }

    fn req_stat_interrupt(&self, mmu: &mut Mmu) {
        mmu.write(0xFF0F, mmu.read(0xFF0F) | 0b00000010);
    }

    fn draw_line(&mut self, mmu: &Mmu, lcd: &mut Lcd, ly: u8, lcdc: u8) {
        let mut line: [u8; 160] = [0; 160];

        // Background and Window only drawn if bit 0 of LCDC is set
        if lcdc & 0b00000001 != 0 {
            let tile_mode_8000 = lcdc & 0b00010000 != 0;

            // Background

            // Retrieve background scroll X and scroll Y
            let (scy, scx) = (mmu.read(0xFF42), mmu.read(0xFF43));
            // Select the background tilemap location based on bit 3 of the LCDC register
            let bg_tilemap: u16 = if lcdc & 0b00001000 != 0 { 0x9C00 } else { 0x9800 };

            let bg_palette: u8 = mmu.read(0xFF47);

            for x_counter in 0..20 {
                let addr = bg_tilemap +
                                (x_counter + scx as u16 / 8) +
                                (((ly as u16 + scy as u16) & 0xFF) / 8) * 32;
                let tile_num = mmu.read(addr);
                let tile_addr: u16 = if tile_mode_8000 {
                    0x8000 + (tile_num as u16) * 16 + ((ly as u16 + scy as u16) % 8) * 2
                } else {
                    0x8800 + (tile_num as i8 as i16 + 128) as u16 * 16 + ((ly as u16 + scy as u16) % 8) * 2
                };

                let b1 = mmu.read(tile_addr);
                let b2 = mmu.read(tile_addr + 1);

                // TODO: SCX!
                for px in 0..8 {
                    let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 } | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                    let color = (bg_palette >> (px_val * 2)) & 0x3;
                    line[(x_counter * 8 + px) as usize] = color;
                }
            }


            // Window
            // if lcdc & 0b00100000 != 0 && self.reached_window && mmu.read(0xFF4B) >= 7 {
            //     for x_counter in 0..20 {
            //         let addr = bg_tilemap +
            //                         x_counter +
            //                         (self.window_line_counter / 8) * 32;
            //         let tile_num = mmu.read(addr);
            //         let tile_addr: u16 = if tile_mode_8000 {
            //             0x8000 + (tile_num as u16) * 16 + (self.window_line_counter % 8) * 2
            //         } else {
            //             0x8800 + (tile_num as i8 as i16 + 128) as u16 * 16 + (self.window_line_counter % 8) * 2
            //         };
                    
            //         let b1 = mmu.read(tile_addr);
            //         let b2 = mmu.read(tile_addr + 1);
            //         for px in 0..8 {
            //             let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 } | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
            //             let color = (bg_palette >> (px_val * 2)) & 0x3;
            //             line[(x_counter * 8 + px) as usize] = color;
            //         }

            //         self.window_line_counter += 1;
            //     }
            // }
        }


        // Sprites: iterate the OAM and draw pixels on the line that we need
        // But only if LCDC bit 1 is set: enable/disable sprites 
        if lcdc & 0b00000010 != 0 {
            let mut sprite_line: [u8; 160] = [0; 160];
            let mut priority: [u8; 160] = [0xFF; 160];

            // Sprite height based on LCDC bit 2: if set "tall-sprite" mode
            let sprite_height = if lcdc & 0b00000100 != 0 { 16 } else { 8 };
            let mut buffered_sprites = 0;
            for entry in (0xFE00..0xFEA0).step_by(4) {
                let (y, x, tidx, flags) = (
                    mmu.read(entry), 
                    mmu.read(entry+1), 
                    mmu.read(entry+2) & if sprite_height == 16 { 0xFE } else { 0xFF }, 
                    mmu.read(entry+3)
                );
                if x > 0 && (ly + 16) >= y && (ly + 16) < y + sprite_height {
                    buffered_sprites += 1;

                    let background_priority = flags & 0b10000000 != 0;
                    let yflip = flags & 0b01000000 != 0;
                    let xflip = flags & 0b00100000 != 0;
                    let sprite_palette = if flags & 0b00010000 != 0 { mmu.read(0xFF49) } else { mmu.read(0xFF48) };

                    let y_line_skew = if yflip { sprite_height - 1 - (ly + 16).wrapping_sub(y) } else { ly + 16 - y } as u16;

                    let tile_addr = 0x8000 + tidx as u16 * 16 + (y_line_skew * 2);
                    let b1 = mmu.read(tile_addr);
                    let b2 = mmu.read(tile_addr + 1);

                    for px in 0..8 {
                        let linepos = (x - 8 + px) as usize;
                        if linepos > 0 && linepos < 160 {
                            let sprite_pos = if xflip { px } else { 7 - px };
                            let px_val: u8 = if b1 & (1 << sprite_pos) != 0 { 1 } else { 0 } 
                                                | if b2 & (1 << sprite_pos) != 0 { 2 } else { 0 };
                            let color = (sprite_palette >> (px_val * 2)) & 0x3;

                            if priority[linepos] > x {
                                priority[linepos] = x;

                                if color == 0 { sprite_line[linepos] = line[linepos]; }
                                else if line[linepos] == 0 && background_priority { sprite_line[linepos] = color; }
                                else if !background_priority { sprite_line[linepos] = color; }
                            }
                        }
                    }
                }

                // Only 10 sprites can be drawn on a single scanline
                if buffered_sprites >= 10 { break; }
            }

            for linepos in 0..160 {
                if sprite_line[linepos] != 0 {
                    line[linepos] = sprite_line[linepos];
                }
            }
        }



        lcd.set_line(ly, line);
    }
}