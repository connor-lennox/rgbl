extern crate enum_dispatch;

use std::io::prelude::*;
use std::io::{self, BufRead};
use std::fs::File;
use std::time::Instant;

use joypad::{Joypad, JoypadButton};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const TARGET_FPS: u32 = 60;

mod cartridge;
mod cpu;
mod memory;
mod motherboard;
mod mmu;
mod timers;
mod ppu;
mod lcd;
mod joypad;

use std::{env, fs};
use std::path::Path;

use crate::lcd::Lcd;
use crate::motherboard::Motherboard;

fn get_log_string(mb: &Motherboard) -> String {
    format!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
        mb.cpu.regs.a, mb.cpu.regs.flags, mb.cpu.regs.b, mb.cpu.regs.c, mb.cpu.regs.d, mb.cpu.regs.e, mb.cpu.regs.h, mb.cpu.regs.l, mb.cpu.sp, mb.cpu.pc,
        mb.mmu.read(mb.cpu.pc), mb.mmu.read(mb.cpu.pc+1), mb.mmu.read(mb.cpu.pc+2), mb.mmu.read(mb.cpu.pc+3))
}

static CONTROLS: [VirtualKeyCode; 8] = [VirtualKeyCode::Z, VirtualKeyCode::X, VirtualKeyCode::Return, VirtualKeyCode::RShift, 
                    VirtualKeyCode::Left, VirtualKeyCode::Right, VirtualKeyCode::Up, VirtualKeyCode::Down];

fn control(key: VirtualKeyCode) -> JoypadButton {
    match key {
        VirtualKeyCode::Z => JoypadButton::A,
        VirtualKeyCode::X => JoypadButton::B,
        VirtualKeyCode::Return => JoypadButton::Start,
        VirtualKeyCode::RShift => JoypadButton::Select,
        VirtualKeyCode::Left => JoypadButton::Left,
        VirtualKeyCode::Right => JoypadButton::Right,
        VirtualKeyCode::Up => JoypadButton::Up,
        VirtualKeyCode::Down => JoypadButton::Down,
        _ => panic!("invalid control keycode")
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let cart_path = if args.len() > 1 {Path::new(&args[1])} else {Path::new("./carts/blargg_roms/01-special.gb")};

    let cart_data = fs::read(cart_path).expect("could not open file");
    let mut mb = Motherboard::new(&cart_data);

    // let mut logfile = File::create("./carts/logs/log.txt").expect("Could not create log file");
    // let ref_file = File::open("./carts/reference_logs/blargg11.txt").expect("Could not open reference log");

    // let mut ref_lines =  io::BufReader::new(ref_file).lines();

    // loop {
    //     // Reference logs skip all execution below PC 0x0100
    //     // if mb.cpu.pc >= 0x0100 {
    //     //     let log_string = get_log_string(&mb);
    //     //     writeln!(&mut logfile, "{}", log_string).expect("could not write to log");
    //         // let ref_string = ref_lines.next().expect("error reading reference log").expect("reference log finished!");
    //         // if !(log_string == ref_string) {
    //         //     mb.tick();
    //         //     writeln!(&mut logfile, "{}", get_log_string(&mb)).expect("");
    //         //     panic!("reference log mismatch: expected\n{}\nbut got\n{}", ref_string, log_string);
    //         // }
    //     // }

    //     mb.tick();
    //     if mb.mmu.read(0xFF02) == 0x81 {
    //         print!("{}", mb.mmu.read(0xFF01) as char);
    //         io::stdout().flush().ok().expect("could not flush stdout");
    //         mb.mmu.write(0xFF02, 0);
    //     }
    // }

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("RGBL")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut cycle_count:u32 = 0;

    let mut frame_start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        cycle_count += mb.tick() as u32;
        if mb.mmu.read(0xFF02) == 0x81 {
            print!("{}", mb.mmu.read(0xFF01) as char);
            io::stdout().flush().ok().expect("could not flush stdout");
            mb.mmu.write(0xFF02, 0);
        }

        if cycle_count >= 70224 {
            draw_lcd(&mb.lcd, pixels.get_frame(), &mb);
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            cycle_count -= 70224;

            // Wait to conserve framerate
            let elapsed_time = Instant::now().duration_since(frame_start).as_millis() as u32;
            let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
                true => 1000 / TARGET_FPS - elapsed_time,
                false => 0,
            };
            let new_inst = frame_start + std::time::Duration::from_millis(wait_millis as u64);
            *control_flow = ControlFlow::WaitUntil(new_inst);
            frame_start = Instant::now();
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            for ctr in CONTROLS {
                if input.key_pressed(ctr) { mb.joypad.press(control(ctr)) }
                if input.key_released(ctr) { mb.joypad.release(control(ctr)) }
            }
        };
    });

}


fn draw_lcd(lcd: &Lcd, frame: &mut [u8], mb: &Motherboard) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {

        // let (x, y) = ((i % 256) as u16, (i / 256) as u16);
        // let tile_num = mb.mmu.read(0x9800 + ((x / 8) + (y / 8) * 32)) as u16;

        // let addr = 0x8000 + tile_num * 16 + (y % 8) * 2;
        // let tile1 = mb.mmu.read(addr as u16);
        // let tile2 = mb.mmu.read((addr+1) as u16);
        
        // let pxidx = 7 - i % 8;
        // let px = if tile1 & (1 << pxidx) != 0 { 1 } else { 0 } 
        //                 | if tile2 & (1 << pxidx) != 0 { 2 } else { 0 };

        // let c = match px {
        //     3 => [0, 0, 0, 255],
        //     2 => [100, 100, 100, 255],
        //     1 => [175, 175, 175, 255],
        //     0 => [255, 255, 255, 255],
        //     _ => panic!("invalid color code")
        // };

        let c = match lcd.pixels[i] {
            3 => [0, 0, 0, 255],
            2 => [100, 100, 100, 255],
            1 => [175, 175, 175, 255],
            0 => [255, 255, 255, 255],
            _ => panic!("invalid color code")
        };

        pixel.copy_from_slice(&c);
    }
}
