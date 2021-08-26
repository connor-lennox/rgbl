extern crate enum_dispatch;

mod cartridge;
mod cpu;
mod memory;
mod motherboard;

use std::fs;
use std::path::Path;

fn main() {
    println!("Hello, world!");

    let cart_data = fs::read(Path::new("./carts/ram_256kb.gb")).expect("could not open file");
    let cart = cartridge::load_cartridge(&cart_data);
    print!("complete");
}
