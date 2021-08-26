use crate::cartridge::CartridgeType;
use crate::memory::MemoryType;

pub struct Motherboard {
    pub memory: MemoryType,
    pub cartridge: CartridgeType,
}