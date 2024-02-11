use crate::memory::Memory;

pub struct SpaceInvadersMemory {
    memory: [u8; 65_536],   
}

impl SpaceInvadersMemory {
    pub fn new(rom: [u8; 8_192]) -> Self {
        let mut memory = [0; 65_536];
        for addr in 0..8_192 {
            memory[addr] = rom[addr];
        }
        Self {
            memory,
        }
    }
}

impl Memory for SpaceInvadersMemory {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => return self.memory[addr as usize],
            0x4000..=0x5FFF => return  self.memory[(addr - 0x2000) as usize],
            0x6000..=0x7FFF => return  self.memory[(addr - 0x4000) as usize],
            0x8000..=0x9FFF => return  self.memory[(addr - 0x6000) as usize],
            0xA000..=0xBFFF => return  self.memory[(addr - 0x8000) as usize],
            0xC000..=0xDFFF => return  self.memory[(addr - 0xA000) as usize],
            0xE000..=0xFFFF => return  self.memory[(addr - 0xC000) as usize],
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => return,
            0x2000..=0x3FFF => self.memory[addr as usize] = data,
            0x4000..=0x5FFF => return,
            0x6000..=0x7FFF => self.memory[(addr - 0x4000) as usize] = data,
            0x8000..=0x9FFF => return,
            0xA000..=0xBFFF => self.memory[(addr - 0x8000) as usize] = data,
            0xC000..=0xDFFF => return,
            0xE000..=0xFFFF => self.memory[(addr - 0xC000) as usize] = data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut memory = SpaceInvadersMemory::new([0; 8_192]);
        assert_eq!(memory.read(0x0000), 0x0);
    }

    #[test]
    fn test_write() {
        let mut memory = SpaceInvadersMemory::new([0; 8_192]);
        memory.write(0x2000, 0x1);
        assert_eq!(memory.read(0x2000), 0x1);
    }

    #[test]
    fn test_write_readonly() {
        let mut memory = SpaceInvadersMemory::new([0; 8_192]);
        memory.write(0x0, 0x1);
        assert_eq!(memory.read(0x0), 0x0);
    }

    #[test]
    fn test_mirror() {
        let mut memory = SpaceInvadersMemory::new([0; 8_192]);
        memory.write(0x2000, 0x1);
        assert_eq!(memory.read(0x6000), 0x1);
    }
}