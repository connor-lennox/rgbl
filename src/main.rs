extern crate enum_dispatch;

use std::io::prelude::*;
use std::io::{self, BufRead};
use std::fs::File;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

mod cartridge;
mod cpu;
mod memory;
mod motherboard;
mod mmu;

use std::fs;
use std::path::Path;

use crate::motherboard::Motherboard;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() -> Result<(), Error> {
    let cart_data = fs::read(Path::new("./carts/01-special.gb")).expect("could not open file");
    let mut mb = Motherboard::new(&cart_data);
    print!("{}", mb.mmu.read(0x0149));

    let mut logfile = File::create("./carts/logs/log.txt").expect("Could not create log file");
    let ref_file = File::open("./carts/reference_logs/blarg1.txt").expect("Could not open reference log");

    let mut ref_lines =  io::BufReader::new(ref_file).lines();

    loop {
        let log_string = format!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
                                    mb.cpu.regs.a, mb.cpu.regs.flags, mb.cpu.regs.b, mb.cpu.regs.c, mb.cpu.regs.d, mb.cpu.regs.e, mb.cpu.regs.h, mb.cpu.regs.l, mb.cpu.sp, mb.cpu.pc,
                                    mb.mmu.read(mb.cpu.pc), mb.mmu.read(mb.cpu.pc+1), mb.mmu.read(mb.cpu.pc+2), mb.mmu.read(mb.cpu.pc+3));
        writeln!(&mut logfile, "{}", log_string).expect("could not write to log");
        let ref_string = ref_lines.next().expect("error reading reference log").expect("reference log finished!");
        if !(log_string == ref_string) {
            panic!("reference log mismatch: expected\n{}\nbut got\n{}", ref_string, log_string);
        }
        mb.tick();
        if mb.mmu.read(0xFF02) == 0x81 {
            print!("{}", mb.mmu.read(0xFF01) as char);
            io::stdout().flush().ok().expect("could not flush stdout");
            mb.mmu.write(0xFF02, 0);
        }
    }


    Ok(())

    // env_logger::init();
    // let event_loop = EventLoop::new();
    // let mut input = WinitInputHelper::new();
    // let window = {
    //     let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    //     WindowBuilder::new()
    //         .with_title("Hello Pixels")
    //         .with_inner_size(size)
    //         .with_min_inner_size(size)
    //         .build(&event_loop)
    //         .unwrap()
    // };

    // let mut pixels = {
    //     let window_size = window.inner_size();
    //     let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    //     Pixels::new(WIDTH, HEIGHT, surface_texture)?
    // };
    // let mut world = World::new();

    // event_loop.run(move |event, _, control_flow| {
    //     // Draw the current frame
    //     if let Event::RedrawRequested(_) = event {
    //         world.draw(pixels.get_frame());
    //         if pixels
    //             .render()
    //             .map_err(|e| error!("pixels.render() failed: {}", e))
    //             .is_err()
    //         {
    //             *control_flow = ControlFlow::Exit;
    //             return;
    //         }
    //     }

    //     // Handle input events
    //     if input.update(&event) {
    //         // Close events
    //         if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
    //             *control_flow = ControlFlow::Exit;
    //             return;
    //         }

    //         // Resize the window
    //         if let Some(size) = input.window_resized() {
    //             pixels.resize_surface(size.width, size.height);
    //         }

    //         // Update internal state and request a redraw
    //         world.update();
    //         window.request_redraw();
    //     }
    // });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
