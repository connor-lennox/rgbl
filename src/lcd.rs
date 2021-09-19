pub struct Lcd {
    pub pixels: [u8; 23200]
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            pixels: [0; 23200],
        }
    }

    pub fn set_line(&mut self, ly: u8, line: [u8; 160]) {
        let line_num = ly as usize;
        self.pixels[line_num*160..(line_num+1)*160].copy_from_slice(&line);
    }
}