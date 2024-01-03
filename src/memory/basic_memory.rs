//use std::fmt;

use crate::memory::Memory;

pub struct BasicMemory {
    memory: [u8; 65_536],   
}

impl BasicMemory {
    pub fn new() -> Self {
        Self {
            memory: [0; 65_536],
        }
    }
}

impl Memory for BasicMemory {
    fn read(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_memory() {
        let memory = BasicMemory::new();
        assert_eq!(memory.memory[0], 0);
        assert_eq!(memory.memory[65_535], 0);
    }

    #[test]
    fn test_read() {
        let memory = BasicMemory::new();
        assert_eq!(memory.read(0), 0);
    }

    #[test]
    fn test_write() {
        let mut memory = BasicMemory::new();
        memory.write(0, 1);
        assert_eq!(memory.read(0), 1);
    }
}