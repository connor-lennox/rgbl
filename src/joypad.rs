use std::collections::HashSet;

use crate::mmu::Mmu;

#[derive(PartialEq, Eq, Hash)]
pub enum JoypadButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select
}

pub struct Joypad {
    pressed: HashSet<JoypadButton>
}

impl Joypad {
    pub fn new() -> Self { Joypad { pressed: HashSet::new() } }

    pub fn press(&mut self, button: JoypadButton) {
        self.pressed.insert(button);
    }

    pub fn release(&mut self, button: JoypadButton) {
        self.pressed.remove(&button);
    }

    pub fn tick(&self, mmu: &mut Mmu) {
        let mut joyp = mmu.read(0xFF00);
        let action = joyp & 0b00100000 == 0;
        let direction = joyp & 0b00010000 == 0;

        // Set all pressed flags to 1 (unpressed)
        joyp |= 0x0F;
        if action {
            if self.pressed.contains(&JoypadButton::Start) { joyp &= 0b11110111 }
            if self.pressed.contains(&JoypadButton::Select) { joyp &= 0b11111011 }
            if self.pressed.contains(&JoypadButton::B) { joyp &= 0b11111101 }
            if self.pressed.contains(&JoypadButton::A) { joyp &= 0b11111110 }
        }

        if direction {
            if self.pressed.contains(&JoypadButton::Down) { joyp &= 0b11110111 }
            if self.pressed.contains(&JoypadButton::Up) { joyp &= 0b11111011 }
            if self.pressed.contains(&JoypadButton::Left) { joyp &= 0b11111101 }
            if self.pressed.contains(&JoypadButton::Right) { joyp &= 0b11111110 }
        }

        mmu.write(0xFF00, joyp);
    }
}