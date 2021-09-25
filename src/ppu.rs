use crate::{lcd::Lcd};

#[derive(PartialEq)]
enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    Drawing,
}

pub struct Ppu {
    vram: [u8; 8192],
    oam: [u8; 160],
    io_regs: [u8; 512],

    mode: PpuMode,
    line_cycles: u32,
    reached_window: bool,
    window_line_counter: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        let mut ppu = Ppu {
            vram: [0; 8192],
            oam: [0; 160],
            io_regs: [0; 512],

            mode: PpuMode::OAMScan,
            line_cycles: 0,
            reached_window: false,
            window_line_counter: 0,
        };
        ppu.io_regs[0x0040] = 0x85;
        ppu.io_regs[0x0042] = 0;
        ppu.io_regs[0x0043] = 0;
        ppu.io_regs[0x0044] = 0;

        ppu
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // 0x8000..=0x9FFF => if self.mode != PpuMode::Drawing { self.vram[(addr - 0x8000) as usize] } else { 0xFF },
            // 0xFE00..=0xFE9F => if self.mode == PpuMode::HBlank || self.mode == PpuMode::VBlank { self.oam[(addr - 0xFE00) as usize] } else { 0xFF }, 
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => self.io_regs[(addr - 0xFF00) as usize],
            _ => panic!("invalid memory read on ppu: {}", addr)
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // VRAM/OAM blocking temporarily disabled
            // 0x8000..=0x9FFF => if self.mode != PpuMode::Drawing { self.vram[(addr - 0x8000) as usize] = value; },
            // 0xFE00..=0xFE9F => if self.mode == PpuMode::HBlank || self.mode == PpuMode::VBlank { self.oam[(addr - 0xFE00) as usize] = value; }
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFF00..=0xFF7F => self.io_regs[(addr - 0xFF00) as usize] = value,
            _ => panic!("invalid memory write on ppu: {}", addr)
        }
    }

    pub fn dma(&mut self, data: &[u8]) {
        self.oam.copy_from_slice(data);
    }

    pub fn tick(&mut self, lcd: &mut Lcd, m_cycles: u8) {
        let t_cycles = m_cycles * 4;
        self.line_cycles += t_cycles as u32;

        // Read current LY/LYC register value
        let ly = self.io_regs[0x0044];
        let lyc = self.io_regs[0x0045];

        let stat = self.io_regs[0x0041];

        // Check for going to the next scanline
        if self.line_cycles >= 456 {
            // Write new LY register value
            let new_ly = (ly + 1) % 154;

            // Indicate when window reached
            if new_ly == self.io_regs[0x004A] { self.reached_window = true; }

            self.io_regs[0x0044] = new_ly;

            // If LYC=LY and the LYC=LY STAT interrupt source is set, request a STAT interrupt
            if (new_ly == lyc) && (stat & 0b01000000 != 0) {
                self.req_stat_interrupt();
            }

            self.line_cycles -= 456;
            self.mode = if new_ly >= 144 {
                if self.mode != PpuMode::VBlank {
                    if (stat & 0b00010000) != 0 { self.req_stat_interrupt(); }
                    self.req_vblank_interrupt();
                    self.reached_window = false;
                    self.window_line_counter = 0;
                }
                PpuMode::VBlank 
            } else {
                if (stat & 0b00100000) != 0 { self.req_stat_interrupt(); }
                PpuMode::OAMScan 
            };
        }

        // Check for OAM Scan -> Drawing mode switch
        if self.line_cycles >= 80 && self.mode == PpuMode::OAMScan {
            // Push a row of pixels to the LCD (all at once, at start of mode)
            self.draw_line(lcd, ly, self.io_regs[0x0040]);
            self.mode = PpuMode::Drawing;
        }

        // Check for Drawing -> HBlank mode switch
        // Normally this would be a variable number of cycles,
        // but it doesn't really matter.
        if self.line_cycles >= 252 && self.mode == PpuMode::Drawing {
            if (stat & 0b00001000) != 0 { self.req_stat_interrupt(); }
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
        self.io_regs[0x0041] = new_stat;
    }

    fn req_vblank_interrupt(&mut self) {
        self.io_regs[0x000F] = self.io_regs[0x000F] | 0b00000001;
    }

    fn req_stat_interrupt(&mut self) {
        self.io_regs[0x000F] = self.io_regs[0x000F] | 0b00000010;
    }

    fn draw_line(&mut self, lcd: &mut Lcd, ly: u8, lcdc: u8) {
        let mut line: [u8; 160] = [0; 160];

        // Background and Window only drawn if bit 0 of LCDC is set
        if lcdc & 0b00000001 != 0 {
            let tile_mode_8000 = lcdc & 0b00010000 != 0;

            // Background

            // Retrieve background scroll X and scroll Y
            let (scy, scx) = (self.io_regs[0x0042], self.io_regs[0x0043]);
            // Select the background tilemap location based on bit 3 of the LCDC register
            let bg_tilemap: u16 = if (lcdc & 0b00001000) != 0 { 0x1C00 } else { 0x1800 };

            let bg_palette: u8 = self.io_regs[0x0047];

            for x_counter in 0..21 {
                let tile_y = (ly as u16 + scy as u16) & 0xFF;
                let addr = (bg_tilemap +
                                ((x_counter + (scx as u16 / 8)) & 0x1F) +
                                (((tile_y / 8) & 0x1F) * 32)) as usize;
                let tile_num = self.vram[addr];
                let tile_addr: usize = (if tile_mode_8000 {
                    (tile_num as u16) * 16
                } else {
                    let normed_tile_num = (tile_num as i8 as i16 + 128) as u16;
                    0x0800 + normed_tile_num * 16
                } + (tile_y % 8) * 2) as usize;

                let b1 = self.vram[tile_addr];
                let b2 = self.vram[tile_addr + 1];

                for px in 0..8 {
                    if (x_counter * 8 + px) > (scx % 8) as u16 {
                        let linepos = (x_counter * 8 + px - (scx % 8) as u16) as usize;
                        if linepos < 160 {
                            let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 } | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                            let color = (bg_palette >> (px_val * 2)) & 0x3;
                            line[linepos] = color;
                        }
                    }
                }
            }


            // Window
            let window_tilemap = if lcdc & 0b01000000 != 0 { 0x1C00 } else { 0x1800 };
            let wx = self.io_regs[0x004B];
            let wy = self.io_regs[0x004A];
            if lcdc & 0b00100000 != 0 && ly >= wy && wx >= 7 && wx < 167 {
                for x_counter in 0..20 {
                    let addr = (window_tilemap +
                                    (x_counter as u16) +
                                    (self.window_line_counter / 8) * 32) as usize;
                    let tile_num = self.vram[addr];
                    let tile_addr: usize = if tile_mode_8000 {
                        (tile_num as u16) * 16 + (self.window_line_counter % 8) * 2
                    } else {
                        0x0800 + (tile_num as i8 as i16 + 128) as u16 * 16 + (self.window_line_counter % 8) * 2
                    } as usize;
                    
                    let b1 = self.vram[tile_addr];
                    let b2 = self.vram[tile_addr + 1];
                    for px in 0..8 {
                        let linepos = x_counter as u16 * 8 + (px + wx - 7) as u16;
                        if linepos < 160 {
                            let px_val = if b1 & (1 << 7 - px) != 0 { 1 } else { 0 } | if b2 & (1 << 7 - px) != 0 { 2 } else { 0 };
                            let color = (bg_palette >> (px_val * 2)) & 0x3;
                            line[linepos as usize] = color;
                        }
                    }
                }

                self.window_line_counter += 1;
            }
        }


        // Sprites: iterate the OAM and draw pixels on the line that we need
        // But only if LCDC bit 1 is set: enable/disable sprites 
        if lcdc & 0b00000010 != 0 {
            let mut sprite_line: [u8; 160] = [0; 160];
            let mut priority: [u8; 160] = [0xFF; 160];

            // Sprite height based on LCDC bit 2: if set "tall-sprite" mode
            let sprite_height = if lcdc & 0b00000100 != 0 { 16 } else { 8 };
            let mut buffered_sprites = 0;
            for entry in (0x0000..0x00A0).step_by(4) {
                let (y, x, tidx, flags) = (
                    self.oam[entry], 
                    self.oam[entry+1], 
                    self.oam[entry+2] & if sprite_height == 16 { 0xFE } else { 0xFF }, 
                    self.oam[entry+3]
                );
                if x > 0 && (ly + 16) >= y && (ly + 16) < y + sprite_height {
                    buffered_sprites += 1;

                    let background_priority = flags & 0b10000000 != 0;
                    let yflip = flags & 0b01000000 != 0;
                    let xflip = flags & 0b00100000 != 0;
                    let sprite_palette = if flags & 0b00010000 != 0 { self.io_regs[0x0049] } else { self.io_regs[0x0048] };

                    let y_line_skew = if yflip { sprite_height - 1 - (ly + 16).wrapping_sub(y) } else { ly + 16 - y } as u16;

                    let tile_addr = (tidx as u16 * 16 + (y_line_skew * 2)) as usize;
                    let b1 = self.vram[tile_addr];
                    let b2 = self.vram[tile_addr + 1];

                    for px in 0..8 {
                        if x + px >= 8 {
                            let linepos = (x + px - 8) as usize;
                            if linepos > 0 && linepos < 160 {
                                let sprite_pos = if xflip { px } else { 7 - px };
                                let px_val: u8 = if b1 & (1 << sprite_pos) != 0 { 1 } else { 0 } 
                                                    | if b2 & (1 << sprite_pos) != 0 { 2 } else { 0 };
                                let color = (sprite_palette >> (px_val * 2)) & 0x3;

                                if priority[linepos] > x {
                                    priority[linepos] = x;

                                    if color == 0 { sprite_line[linepos] = line[linepos]; }
                                    else if line[linepos] == 0 || !background_priority { sprite_line[linepos] = color; }
                                }
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