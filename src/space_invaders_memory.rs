//use std::fmt;

use crate::memory::Memory;

pub struct SpaceInvadersMemory {
    memory: [u8; 65_536],   
}

impl SpaceInvadersMemory {
    pub fn new() -> Self {
        Self {
            memory: [0; 65_536],
        }
    }
}

impl Memory for SpaceInvadersMemory {
    fn read(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

/*
impl fmt::Display for SpaceInvadersMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.memory[0])
    }
}
*/