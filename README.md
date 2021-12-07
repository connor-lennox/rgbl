# rgbl

A Gameboy DMG emulator written in Rust.

The code in the `src` directory is organized as per the individual components of the Gameboy itself. 
This preserves how components interacted with each other in the original hardware and also makes it easier to understand what is responsible for what.
The main component of the emulator itself (beside the included `main.rs` which runs the emulation) is the [Motherboard](https://github.com/connor-lennox/rgbl/blob/master/src/motherboard.rs), which houses all of the other components.

For graphics, I am using the [pixels](https://crates.io/crates/pixels) crate, and my input loop comes from [winit](https://crates.io/crates/winit).

A common technique in Gameboy emulation is to divide all of the clock cycle counts by 4 (as all opcodes happen to require a number of clock cycles that is a multiple of 4).
If you see a reference to an "m-cycle" in the code, that is actually 4 "t-cycles" (one t-cycle being a clock tick from the internal 4 MHz clock).
In order to emulate games at the proper speed, I only allow the emulator to operate at a maximum of 60 frames per second, where each frame is denoted by about 70000 m-cycles.
While this doesn't allow me to get accurately emulate sub-opcode timings, this doesn't actually make a noticeable difference for the vast majority of titles.

If you're interested in trying my emulator out for yourself, you can clone this repository and use a rust toolchain to compile and run.
To load a cartridge, pass the path to the cartridge file as a command line argument.
While I don't provide cartridge files here, there are several test carts available freely online (beyond that, you're on your own).
