use crate::mmu::Mmu;

pub struct Timers {
    div_partial: u8,
    tima_partial: u16,
}

impl Timers {
    pub fn new() -> Timers {
        Timers { div_partial: 0, tima_partial: 0 }
    }

    pub fn tick(&mut self, mmu: &mut Mmu, mcycles: u8) {
        // Given an amount of m-cycles, do timer-related tasks and set interrupts
        let t_cycles = mcycles * 4;

        // Do DIV register
        let (res, inc_div) = self.div_partial.overflowing_add(t_cycles);
        self.div_partial = res;
        if inc_div {
            let prev_div = mmu.read(0xFF04);
            mmu.write(0xFF04, prev_div.wrapping_add(1));
        }

        // Do TIMA register
        let tac = mmu.read(0xFF07);
        // Check if the timer is enabled
        if tac & 0b100 != 0 {
            // Do partial timer ticks according to cpu progress
            self.tima_partial += t_cycles as u16;
            let timer_step = match tac & 0b011 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _ => panic!("invalid TAC speed")
            };

            // Check partial tick progress compared to threshold
            if self.tima_partial > timer_step {
                // Increment TIMA register, throw interrupt if wrapping
                let prev_tima = mmu.read(0xFF05);
                let (new_tima, overflow) = prev_tima.overflowing_add(1);
                // If TIMA overflowed, reset it to TMA and throw interrupt
                if overflow {
                    let tma = mmu.read(0xFF06);
                    mmu.write(0xFF05, tma);
                    mmu.write(0xFF0F, mmu.read(0xFF0F) | 0b00100);
                } else {
                    mmu.write(0xFF05, new_tima);
                }

                self.tima_partial -= timer_step;
            }
        }
    }
}
