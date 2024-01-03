use std::fmt;
use crate::conditions;
use crate::memory::Memory;

enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum RegisterPair {
    BC,
    DE,
    HL,
    PSW,
}

pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    conditions: conditions::Conditions,
    pc: u16,
    sp: u16,
    interrupt_enabled: bool,
    memory: Box<dyn Memory>,
    wait_cycles: usize,
    interrupt_opcode: Option<u8>,
}

impl Cpu {
    pub fn new(memory: Box<dyn Memory>) -> Self {
        Cpu {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            conditions: conditions::Conditions::new(),
            pc: 0,
            sp: 0,
            interrupt_enabled: true,
            memory,
            wait_cycles: 0,
            interrupt_opcode: None,
        }
    }

    pub fn tick(&mut self) {
        /*
            1. Check if cycles are pending from last executed instruction, if so decrement wait_cycles and return
            2. If wait_cycles == 0, Check if an interrupt has been received from the wrapper system; if so load that interrupt as the current instruction and begin processing. Do not change PC etc.
            3. If no interrupt is present, load the next insruction from memory[PC] and begin processing.
        */
        if self.wait_cycles > 0 {
            self.wait_cycles = self.wait_cycles - 1;
            return;
        }
        
        if !self.interrupt_enabled {
            self.interrupt_opcode = None;
        }

        let instruction = match self.interrupt_opcode {
            Some(x) => {
                x
            },
            None => {
                let i = self.memory.read(self.pc);
                self.pc = self.pc + 1;
                i
            }
        };

       self.wait_cycles = self.dispatch(instruction);
    }

    fn dispatch(&mut self, instruction: u8) -> usize {
        let mut wait_cycles = 0;
        match instruction {
            0x0 => {
                wait_cycles = 3; // 4 - 1
            },
            0x1 => {
                let lsb = self.memory.read(self.pc);
                let msb = self.memory.read(self.pc + 1);
                self.c = lsb;
                self.b = msb;
                self.pc = self.pc + 2;
                wait_cycles = 9; // 10 - 1
            },
            0x2 => {
                let addr: u16 = ((self.b as u16) << 8) + self.c as u16;
                self.memory.write(addr, self.a);
                wait_cycles = 6; // 7 - 1
            },
            0x3 => {
                let (c, cof) = self.c.overflowing_add(1);
                self.c = c;
                if cof {
                    let (b, _bof) = self.b.overflowing_add(1);
                    self.b = b;
                }
                wait_cycles = 4; // 5 - 1
            },
            0x4 => {
                let b = self.b.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, b == 0);
                self.conditions.set(conditions::ConditionName::Sign, b & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, b.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.b, 1));
                self.b = b;
                wait_cycles = 4; // 5 - 1
            },
            0x5 => {
                let b = self.b.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, b == 0);
                self.conditions.set(conditions::ConditionName::Sign, b & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, b.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.b, 1));
                self.b = b;
                wait_cycles = 4; // 5 - 1
            },
            0x6 => {
                let val = self.memory.read(self.pc);
                self.b = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x7 => {
                self.conditions.set(conditions::ConditionName::Carry, self.a & 0xA0 == 0xA0);
                self.a = self.a.rotate_left(1);
                wait_cycles = 3; // 4 - 1
            },
            0x8 => {
                wait_cycles = 3; // 4 - 1
            },
            0x9 => {
                let bc = ((self.b as u16) << 8) + self.c as u16;
                let hl = ((self.h as u16) << 8) + self.l as u16;
                let (res, of) = hl.overflowing_add(bc);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.h = (res >> 8) as u8;
                self.l = (res & 0xFF) as u8;
                wait_cycles = 9; // 10 - 1
            },
            0xa => {
                let addr: u16 = ((self.b as u16) << 8) + self.c as u16;
                self.a = self.memory.read(addr);
                wait_cycles = 6; // 7 - 1
            },
            0xb => {
                let mut val = ((self.b as u16) << 8) + self.c as u16;
                val = val.wrapping_sub(1);
                self.b = (val >> 8) as u8;
                self.c = (val & 0xFF) as u8;
                wait_cycles = 4; // 5 - 1
            },
            0xc => {
                let c = self.c.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, c == 0);
                self.conditions.set(conditions::ConditionName::Sign, c & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, c.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.c, 1));
                self.c = c;
                wait_cycles = 4; // 5 - 1
            },
            0xd => {
                let c = self.c.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, c == 0);
                self.conditions.set(conditions::ConditionName::Sign, c & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, c.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.c, 1));
                self.c = c;
                wait_cycles = 4; // 5 - 1
            },
            0xe => {
                let val = self.memory.read(self.pc);
                self.c = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0xf => {
                self.conditions.set(conditions::ConditionName::Carry, self.a & 0x01 == 0x01);
                self.a = self.a.rotate_right(1);
                wait_cycles = 3; // 4 - 1
            },
            0x10 => {
                wait_cycles = 3; // 4 - 1
            },
            0x11 => {
                let lsb = self.memory.read(self.pc);
                let msb = self.memory.read(self.pc + 1);
                self.e = lsb;
                self.d = msb;
                self.pc = self.pc + 2;
                wait_cycles = 9; // 10 - 1
            },
            0x12 => {
                let addr = ((self.d as u16) << 8) + self.e as u16;
                self.memory.write(addr, self.a);
                wait_cycles = 6; // 7 - 1
            },
            0x13 => {
                let (e, eof) = self.e.overflowing_add(1);
                self.e = e;
                if eof {
                    let (d, _dof) = self.d.overflowing_add(1);
                    self.d = d;
                }
                wait_cycles = 4; // 5 - 1
            },
            0x14 => {
                let d = self.d.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, d == 0);
                self.conditions.set(conditions::ConditionName::Sign, d & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, d.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.d, 1));
                self.d = d;
                wait_cycles = 4; // 5 - 1
            },
            0x15 => {
                let d = self.d.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, d == 0);
                self.conditions.set(conditions::ConditionName::Sign, d & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, d.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.d, 1));
                self.d = d;
                wait_cycles = 4; // 5 - 1
            },
            0x16 => {
                let val = self.memory.read(self.pc);
                self.d = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x17 => {
                let mut carry = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    carry = 1;
                }
                self.conditions.set(conditions::ConditionName::Carry, self.a & 0xA0 == 0xA0);
                self.a = self.a.rotate_left(1);
                if carry == 1 {
                    self.a = self.a | 0x01;
                } else {
                    self.a = self.a & 0xFE;
                }
                wait_cycles = 3; // 4 - 1
            },
            0x18 => {
                wait_cycles = 3; // 4 - 1
            },
            0x19 => {
                let de = ((self.d as u16) << 8) + self.e as u16;
                let hl = ((self.h as u16) << 8) + self.l as u16;
                let (res, of) = hl.overflowing_add(de);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.h = (res >> 8) as u8;
                self.l = (res & 0xFF) as u8;
                wait_cycles = 9; // 10 - 1
            },
            0x1a => {
                let addr: u16 = ((self.d as u16) << 8) + self.e as u16;
                self.a = self.memory.read(addr);
                wait_cycles = 6; // 7 - 1
            },
            0x1b => {
                let mut val = ((self.d as u16) << 8) + self.e as u16;
                val = val.wrapping_sub(1);
                self.d = (val >> 8) as u8;
                self.e = (val & 0xFF) as u8;
                wait_cycles = 4; // 5 - 1
            },
            0x1c => {
                let e = self.e.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, e == 0);
                self.conditions.set(conditions::ConditionName::Sign, e & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, e.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.e, 1));
                self.e = e;
                wait_cycles = 4; // 5 - 1
            },
            0x1d => {
                let e = self.e.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, e == 0);
                self.conditions.set(conditions::ConditionName::Sign, e & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, e.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.e, 1));
                self.e = e;
                wait_cycles = 4; // 5 - 1
            },
            0x1e => {
                let val = self.memory.read(self.pc);
                self.e = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x1f => {
                let mut carry = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    carry = 1;
                }
                self.conditions.set(conditions::ConditionName::Carry, self.a & 0x01 == 0x01);
                self.a = self.a.rotate_right(1);
                if carry == 1 {
                    self.a = self.a | 0xA0;
                } else {
                    self.a = self.a & 0x8F;
                }
                wait_cycles = 3; // 4 - 1
            },
            0x20 => {
                wait_cycles = 3; // 4 - 1
            },
            0x21 => {
                let lsb = self.memory.read(self.pc);
                let msb = self.memory.read(self.pc + 1);
                self.l = lsb;
                self.h = msb;
                self.pc = self.pc + 2;
                wait_cycles = 9; // 10 - 1
            },
            0x22 => {
                let addr = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                self.memory.write(addr, self.l);
                self.memory.write(addr + 1, self.h);
                self.pc = self.pc + 2;
                wait_cycles = 15; // 16 - 1
            },
            0x23 => {
                let (l, lof) = self.l.overflowing_add(1);
                self.l = l;
                if lof {
                    let (h, _hof) = self.h.overflowing_add(1);
                    self.h = h;
                }
                wait_cycles = 4; // 5 - 1
            },
            0x24 => {
                let h = self.h.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, h == 0);
                self.conditions.set(conditions::ConditionName::Sign, h & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, h.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.h, 1));
                self.h = h;
                wait_cycles = 4; // 5 - 1
            },
            0x25 => {
                let h = self.h.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, h == 0);
                self.conditions.set(conditions::ConditionName::Sign, h & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, h.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.h, 1));
                self.h = h;
                wait_cycles = 4; // 5 - 1
            },
            0x26 => {
                let val = self.memory.read(self.pc);
                self.h = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x27 => {
                //TODO DAA
            },
            0x28 => {
                wait_cycles = 3; // 4 - 1
            },
            0x29 => {
                let hl = ((self.h as u16) << 8) + self.l as u16;
                let (res, of) = hl.overflowing_add(hl);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.h = (res >> 8) as u8;
                self.l = (res & 0xFF) as u8;
                wait_cycles = 9; // 10 - 1
            },
            0x2a => {
                let addr = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                self.l = self.memory.read(addr);
                self.h = self.memory.read(addr + 1);
                self.pc = self.pc + 2;
                wait_cycles = 15; // 16 - 1
            },
            0x2b => {
                let mut val = ((self.h as u16) << 8) + self.l as u16;
                val = val.wrapping_sub(1);
                self.h = (val >> 8) as u8;
                self.l = (val & 0xFF) as u8;
                wait_cycles = 4; // 5 - 1
            },
            0x2c => {
                let l = self.l.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, l == 0);
                self.conditions.set(conditions::ConditionName::Sign, l & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, l.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.l, 1));
                self.l = l;
                wait_cycles = 4; // 5 - 1
            },
            0x2d => {
                let l = self.l.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, l == 0);
                self.conditions.set(conditions::ConditionName::Sign, l & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, l.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.l, 1));
                self.l = l;
                wait_cycles = 4; // 5 - 1
            },
            0x2e => {
                let val = self.memory.read(self.pc);
                self.l = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x2f => {
                self.a = !self.a;
                wait_cycles = 3; // 4 - 1
            },
            0x30 => {
                wait_cycles = 3; // 4 - 1
            },
            0x31 => {
                let lsb = self.memory.read(self.pc);
                let msb = self.memory.read(self.pc + 1);
                self.sp = ((msb as u16) << 8) + lsb as u16; // construct 16 bit int from the two 8-bit memory values
                self.pc = self.pc + 2;
                wait_cycles = 9; // 10 - 1
            },
            0x32 => {
                let addr = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                self.memory.write(addr, self.a);
                self.pc = self.pc + 2;
                wait_cycles = 12; // 13 - 1
            },
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                wait_cycles = 4; // 5 - 1
            },
            0x34 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr).wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, val == 0);
                self.conditions.set(conditions::ConditionName::Sign, val & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, val.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.memory.read(addr), 1));
                self.memory.write(addr, val);
                wait_cycles = 9; // 10 - 1
            },
            0x35 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr).wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, val == 0);
                self.conditions.set(conditions::ConditionName::Sign, val & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, val.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.memory.read(addr), 1));
                self.memory.write(addr, val);
                wait_cycles = 9; // 10 - 1
            },
            0x36 => {
                let val = self.memory.read(self.pc);
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, val);
                self.pc = self.pc + 1;
                wait_cycles = 9; // 10 - 1
            },
            0x37 => {
                self.conditions.set(conditions::ConditionName::Carry, true);
                wait_cycles = 3; // 4 - 1
            },
            0x38 => {
                wait_cycles = 3; // 4 - 1
            },
            0x39 => {
                let hl = ((self.h as u16) << 8) + self.l as u16;
                let (res, of) = hl.overflowing_add(self.sp);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.h = (res >> 8) as u8;
                self.l = (res & 0xFF) as u8;
                wait_cycles = 9; // 10 - 1
            },
            0x3a => {
                let addr = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                self.a = self.memory.read(addr);
                self.pc = self.pc + 2;
                wait_cycles = 12; // 13 - 1
            },
            0x3b => {
                self.sp = self.sp.wrapping_sub(1);
                wait_cycles = 4; // 5 - 1
            },
            0x3c => {
                let a = self.a.wrapping_add(1);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, 1));
                self.a = a;
                wait_cycles = 4; // 5 - 1
            },
            0x3d => {
                let a = self.a.wrapping_sub(1);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, 1));
                self.a = a;
                wait_cycles = 4; // 5 - 1
            },
            0x3e => {
                let val = self.memory.read(self.pc);
                self.a = val;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0x3f => {
                self.conditions.set(conditions::ConditionName::Carry, !self.conditions.get(conditions::ConditionName::Carry));
                wait_cycles = 3; // 4 - 1
            },
            0x40 => {
                self.b = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x41 => {
                self.b = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x42 => {
                self.b = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x43 => {
                self.b = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x44 => {
                self.b = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x45 => {
                self.b = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x46 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.b = val;
                wait_cycles = 6; // 7 - 1
            },
            0x47 => {
                self.b = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x48 => {
                self.c = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x49 => {
                self.c = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x4a => {
                self.c = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x4b => {
                self.c = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x4c => {
                self.c = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x4d => {
                self.c = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x4e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.c = val;
                wait_cycles = 6; // 7 - 1
            },
            0x4f => {
                self.c = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x50 => {
                self.d = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x51 => {
                self.d = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x52 => {
                self.d = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x53 => {
                self.d = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x54 => {
                self.d = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x55 => {
                self.d = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x56 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.d = val;
                wait_cycles = 6; // 7 - 1
            },
            0x57 => {
                self.d = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x58 => {
                self.e = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x59 => {
                self.e = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x5a => {
                self.e = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x5b => {
                self.e = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x5c => {
                self.e = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x5d => {
                self.e = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x5e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.e = val;
                wait_cycles = 6; // 7 - 1
            },
            0x5f => {
                self.e = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x60 => {
                self.h = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x61 => {
                self.h = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x62 => {
                self.h = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x63 => {
                self.h = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x64 => {
                self.h = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x65 => {
                self.h = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x66 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.h = val;
                wait_cycles = 6; // 7 - 1
            },
            0x67 => {
                self.h = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x68 => {
                self.l = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x69 => {
                self.l = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x6a => {
                self.l = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x6b => {
                self.l = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x6c => {
                self.l = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x6d => {
                self.l = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x6e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.l = val;
                wait_cycles = 6; // 7 - 1
            },
            0x6f => {
                self.l = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x70 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.b);
                wait_cycles = 6; // 7 - 1
            },
            0x71 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.c);
                wait_cycles = 6; // 7 - 1
            },
            0x72 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.d);
                wait_cycles = 6; // 7 - 1
            },
            0x73 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.e);
                wait_cycles = 6; // 7 - 1
            },
            0x74 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.h);
                wait_cycles = 6; // 7 - 1
            },
            0x75 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.l);
                wait_cycles = 6; // 7 - 1
            },
            0x76 => {
                //TODO HLT
            },
            0x77 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                self.memory.write(addr, self.a);
                wait_cycles = 6; // 7 - 1
            },
            0x78 => {
                self.a = self.b;
                wait_cycles = 4; // 5 - 1
            },
            0x79 => {
                self.a = self.c;
                wait_cycles = 4; // 5 - 1
            },
            0x7a => {
                self.a = self.d;
                wait_cycles = 4; // 5 - 1
            },
            0x7b => {
                self.a = self.e;
                wait_cycles = 4; // 5 - 1
            },
            0x7c => {
                self.a = self.h;
                wait_cycles = 4; // 5 - 1
            },
            0x7d => {
                self.a = self.l;
                wait_cycles = 4; // 5 - 1
            },
            0x7e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.a = val;
                wait_cycles = 6; // 7 - 1
            },
            0x7f => {
                self.a = self.a;
                wait_cycles = 4; // 5 - 1
            },
            0x80 => {
                let (a, of) = self.a.overflowing_add(self.b);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.b));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x81 => {
                let (a, of) = self.a.overflowing_add(self.c);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.c));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x82 => {
                let (a, of) = self.a.overflowing_add(self.d);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.d));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x83 => {
                let (a, of) = self.a.overflowing_add(self.e);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.e));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x84 => {
                let (a, of) = self.a.overflowing_add(self.h);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.h));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x85 => {
                let (a, of) = self.a.overflowing_add(self.l);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.l));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x86 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let (a, of) = self.a.overflowing_add(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
                self.a = a;
                wait_cycles = 6; // 7 - 1
            },
            0x87 => {
                let (a, of) = self.a.overflowing_add(self.a);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, self.a));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x88 => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let b = self.b.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(b);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, b));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x89 => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let c = self.c.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(c);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, c));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x8a => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let d = self.d.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(d);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, d));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x8b => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let e = self.e.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(e);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, e));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x8c => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let h = self.h.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(h);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, h));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x8d => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let l = self.l.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(l);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, l));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x8e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let v = val.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(v);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, v));
                self.a = a;
                wait_cycles = 7; // 6 - 1
            },
            0x8f => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let acy = self.a.wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(acy);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, acy));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x90 => {
                let (a, of) = self.a.overflowing_sub(self.b);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.b));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x91 => {
                let (a, of) = self.a.overflowing_sub(self.c);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.c));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x92 => {
                let (a, of) = self.a.overflowing_sub(self.d);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.d));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x93 => {
                let (a, of) = self.a.overflowing_sub(self.e);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.e));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x94 => {
                let (a, of) = self.a.overflowing_sub(self.h);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.h));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x95 => {
                let (a, of) = self.a.overflowing_sub(self.l);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.l));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x96 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let (a, of) = self.a.overflowing_sub(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
                self.a = a;
                wait_cycles = 6; // 7 - 1
            },
            0x97 => {
                let (a, of) = self.a.overflowing_sub(self.a);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.a));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x98 => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let b = self.b.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(b);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, b));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x99 => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let c = self.c.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(c);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, c));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x9a => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let d = self.d.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(d);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, d));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x9b => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let e = self.e.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(e);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, e));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x9c => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let h = self.h.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(h);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, h));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x9d => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let l = self.l.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(l);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, l));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0x9e => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let v = val.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(v);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, v));
                self.a = a;
                wait_cycles = 7; // 6 - 1
            },
            0x9f => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let acy = self.a.wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(acy);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, acy));
                self.a = a;
                wait_cycles = 3; // 4 - 1
            },
            0xa0 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.b & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.b;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa1 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.c & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.c;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa2 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.d & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.d;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa3 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.e & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.e;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa4 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.h & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.h;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa5 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.l & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.l;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa6 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let aux = ((self.a & 0x0A) >> 3) | ((val & 0x0A) >> 3) == 0x1;
                self.a = self.a & val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 6; // 7 - 1
            },
            0xa7 => {
                let aux = ((self.a & 0x0A) >> 3) | ((self.a & 0x0A) >> 3) == 0x1;
                self.a = self.a & self.a;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                wait_cycles = 3; // 4 - 1
            },
            0xa8 => {
                self.a = self.a ^ self.b;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xa9 => {
                self.a = self.a ^ self.c;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xaa => {
                self.a = self.a ^ self.d;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xab => {
                self.a = self.a ^ self.e;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xac => {
                self.a = self.a ^ self.h;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xad => {
                self.a = self.a ^ self.l;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xae => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.a = self.a ^ val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 6; // 7 - 1
            },
            0xaf => {
                self.a = self.a ^ self.a;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb0 => {
                self.a = self.a |self.b;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb1 => {
                self.a = self.a |self.c;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb2 => {
                self.a = self.a |self.d;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb3 => {
                self.a = self.a |self.e;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb4 => {
                self.a = self.a |self.h;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb5 => {
                self.a = self.a |self.l;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb6 => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                self.a = self.a | val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 6; // 7 - 1
            },
            0xb7 => {
                self.a = self.a |self.a;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                wait_cycles = 3; // 4 - 1
            },
            0xb8 => {
                let (a, of) = self.a.overflowing_sub(self.b);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.b));
                wait_cycles = 3; // 4 - 1
            },
            0xb9 => {
                let (a, of) = self.a.overflowing_sub(self.c);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.c));
                wait_cycles = 3; // 4 - 1
            },
            0xba => {
                let (a, of) = self.a.overflowing_sub(self.d);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.d));
                wait_cycles = 3; // 4 - 1
            },
            0xbb => {
                let (a, of) = self.a.overflowing_sub(self.e);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.e));
                wait_cycles = 3; // 4 - 1
            },
            0xbc => {
                let (a, of) = self.a.overflowing_sub(self.h);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.h));
                wait_cycles = 3; // 4 - 1
            },
            0xbd => {
                let (a, of) = self.a.overflowing_sub(self.l);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.l));
                wait_cycles = 3; // 4 - 1
            },
            0xbe => {
                let addr = ((self.h as u16) << 8) + self.l as u16;
                let val = self.memory.read(addr);
                let (a, of) = self.a.overflowing_sub(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
                wait_cycles = 7; // 6 - 1
            },
            0xbf => {
                let (a, of) = self.a.overflowing_sub(self.a);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, self.a));
                wait_cycles = 3; // 4 - 1
            },
            0xc0 => {
                if !self.conditions.get(conditions::ConditionName::Zero) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xc1 => {
                self.c = self.memory.read(self.sp);
                self.b = self.memory.read(self.sp + 1);
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xc2 => {
                if !self.conditions.get(conditions::ConditionName::Zero) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xc3 => {
                self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                wait_cycles = 9; // 10 - 1
            },
            0xc4 => {
                if !self.conditions.get(conditions::ConditionName::Zero) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xc5 => {
                self.memory.write(self.sp - 2, self.c);
                self.memory.write(self.sp - 1, self.b);
                self.sp = self.sp - 2;
            },
            0xc6 => {
                let val = self.memory.read(self.pc);
                let (a, of) = self.a.overflowing_add(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
                self.a = a;
                self.pc = self.pc + 1;
                wait_cycles = 3; // 4 - 1
            },
            0xc7 => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0000;
                wait_cycles = 10; // 11 - 1
            },
            0xc8 => {
                if self.conditions.get(conditions::ConditionName::Zero) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xc9 => {
                self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xca => {
                if self.conditions.get(conditions::ConditionName::Zero) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xcb => {
                self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                wait_cycles = 9; // 10 - 1
            },
            0xcc => {
                if self.conditions.get(conditions::ConditionName::Zero) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xcd => {
                let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                self.pc = self.pc + 2;
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = immediate;
                wait_cycles = 16; // 17 - 1
            },
            0xce => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let val = self.memory.read(self.pc).wrapping_add(cy);
                let (a, of) = self.a.overflowing_add(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
                self.a = a;
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0xcf => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0008;
                wait_cycles = 10; // 11 - 1
            },
            0xd0 => {
                if !self.conditions.get(conditions::ConditionName::Carry) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xd1 => {
                self.d = self.memory.read(self.sp);
                self.e = self.memory.read(self.sp + 1);
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xd2 => {
                if !self.conditions.get(conditions::ConditionName::Carry) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xd3 => {
                //TODO IN d8
            },
            0xd4 => {
                if !self.conditions.get(conditions::ConditionName::Carry) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xd5 => {
                self.memory.write(self.sp - 2, self.d);
                self.memory.write(self.sp - 1, self.e);
                self.sp = self.sp - 2;
            },
            0xd6 => {
                let val = self.memory.read(self.pc);
                let (a, of) = self.a.overflowing_sub(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
                self.a = a;
                self.pc = self.pc + 1;
                wait_cycles = 3; // 4 - 1
            },
            0xd7 => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0010;
                wait_cycles = 10; // 11 - 1
            },
            0xd8 => {
                if self.conditions.get(conditions::ConditionName::Carry) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xd9 => {
                self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xda => {
                if self.conditions.get(conditions::ConditionName::Carry) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xdb => {
                //TODO OUT d8
            },
            0xdc => {
                if self.conditions.get(conditions::ConditionName::Carry) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xdd => {
                let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                self.pc = self.pc + 2;
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = immediate;
                wait_cycles = 16; // 17 - 1
            },
            0xde => {
                let mut cy = 0;
                if self.conditions.get(conditions::ConditionName::Carry) {
                    cy = 1;
                }
                let val = self.memory.read(self.pc).wrapping_sub(cy);
                let (a, of) = self.a.overflowing_sub(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
                self.a = a;
                self.pc = self.pc + 1;
                wait_cycles = 3; // 4 - 1
            },
            0xdf => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0018;
                wait_cycles = 10; // 11 - 1
            },
            0xe0 => {
                if !self.conditions.get(conditions::ConditionName::Parity) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xe1 => {
                self.h = self.memory.read(self.sp);
                self.l = self.memory.read(self.sp + 1);
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xe2 => {
                if !self.conditions.get(conditions::ConditionName::Parity) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xe3 => {
                let lval = self.memory.read(self.sp);
                let hval = self.memory.read(self.sp + 1);
                self.memory.write(self.sp, self.l);
                self.memory.write(self.sp + 1, self.h);
                self.l = lval;
                self.h = hval;
                wait_cycles = 17; // 18 - 1
            },
            0xe4 => {
                if !self.conditions.get(conditions::ConditionName::Parity) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xe5 => {
                self.memory.write(self.sp - 2, self.h);
                self.memory.write(self.sp - 1, self.l);
                self.sp = self.sp - 2;
            },
            0xe6 => {
                let val = self.memory.read(self.pc);
                let aux = ((self.a & 0x0A) >> 3) | ((val & 0x0A) >> 3) == 0x1;
                self.a = self.a & val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, aux);
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0xe7 => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0020;
                wait_cycles = 10; // 11 - 1
            },
            0xe8 => {
                if self.conditions.get(conditions::ConditionName::Parity) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xe9 => {
                self.pc = ((self.h as u16) << 8) + self.l as u16;
                wait_cycles = 4; // 5 - 1
            },
            0xea => {
                if self.conditions.get(conditions::ConditionName::Parity) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xeb => {
                let hval = self.d;
                let lval = self.e;
                self.d = self.h;
                self.e = self.l;
                self.h = hval;
                self.l = lval;
                wait_cycles = 4; // 5 - 1
            },
            0xec => {
                if self.conditions.get(conditions::ConditionName::Parity) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xed => {
                let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                self.pc = self.pc + 2;
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = immediate;
                wait_cycles = 16; // 17 - 1
            },
            0xee => {
                let val = self.memory.read(self.pc);
                self.a = self.a ^ val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                self.pc = self.pc + 1;
                wait_cycles = 3; // 4 - 1
            },
            0xef => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0028;
                wait_cycles = 10; // 11 - 1
            },
            0xf0 => {
                if !self.conditions.get(conditions::ConditionName::Sign) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xf1 => {
                let flags = self.memory.read(self.sp);
                self.conditions.restore_from_bits(flags);
                self.a = self.memory.read(self.sp + 1);
                self.sp = self.sp + 2;
                wait_cycles = 9; // 10 - 1
            },
            0xf2 => {
                if !self.conditions.get(conditions::ConditionName::Sign) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xf3 => {
                self.interrupt_enabled = false;
                wait_cycles = 3; // 4 - 1
            },
            0xf4 => {
                if !self.conditions.get(conditions::ConditionName::Sign) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xf5 => {
                let flags = self.conditions.as_bits();
                self.memory.write(self.sp - 2, flags);
                self.memory.write(self.sp - 1, self.a);
                self.sp = self.sp - 2;
                wait_cycles = 10; // 11 - 1
            },
            0xf6 => {
                let val = self.memory.read(self.pc);
                self.a = self.a | val;
                self.conditions.set(conditions::ConditionName::Carry, false);
                self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
                self.conditions.set(conditions::ConditionName::Sign, self.a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, false);
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0xf7 => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0030;
                wait_cycles = 10; // 11 - 1
            },
            0xf8 => {
                if self.conditions.get(conditions::ConditionName::Sign) {
                    self.pc = ((self.memory.read(self.sp + 1) as u16) << 8) + self.memory.read(self.sp) as u16;
                    self.sp = self.sp + 2;
                    wait_cycles = 10; // 11 - 1
                } else {
                    wait_cycles = 4; // 5 - 1
                }
            },
            0xf9 => {
                self.sp = ((self.h as u16) << 8) + self.l as u16;
                wait_cycles = 4; // 5 - 1
            },
            0xfa => {
                if self.conditions.get(conditions::ConditionName::Sign) {
                    self.pc = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
                }
                wait_cycles = 9; // 10 - 1
            },
            0xfb => {
                self.interrupt_enabled = true;
                wait_cycles = 3; // 4 - 1
            },
            0xfc => {
                if self.conditions.get(conditions::ConditionName::Sign) {
                    let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                    self.pc = self.pc + 2;
                    self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                    self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                    self.sp = self.sp - 2;
                    self.pc = immediate;
                    wait_cycles = 16; // 17 - 1
                } else {
                    wait_cycles = 10; // 11 - 1
                }
            },
            0xfd => {
                let immediate = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self. pc) as u16;
                self.pc = self.pc + 2;
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = immediate;
                wait_cycles = 16; // 17 - 1
            },
            0xfe => {
                let val = self.memory.read(self.pc);
                let (a, of) = self.a.overflowing_sub(val);
                self.conditions.set(conditions::ConditionName::Carry, of);
                self.conditions.set(conditions::ConditionName::Zero, a == 0);
                self.conditions.set(conditions::ConditionName::Sign, a & 0xA0 == 0xA0);
                self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
                self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
                self.pc = self.pc + 1;
                wait_cycles = 6; // 7 - 1
            },
            0xff => {
                self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
                self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
                self.sp = self.sp - 2;
                self.pc = 0x0038;
                wait_cycles = 10; // 11 - 1
            }        }
        return wait_cycles;
    }

    pub fn receive_interrupt(&mut self, interrupt: u8) {
        if self.interrupt_enabled {
            self.interrupt_opcode = Some(interrupt);
        }
    }

    /* Length: 1, Cycles: 4, Flags: None*/
    fn nop(&self) -> usize {
        return 3;
    }

    fn enable_interrupts(&mut self) {
        self.interrupt_enabled = true;
    }

    fn disable_interrupts(&mut self) {
        self.interrupt_enabled = false;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\ta: {}\n\tb: {}\n\tc: {}\n\td: {}\n\te: {}\n\th: {}\n\tl: {}\n\tconditions: {}\n\tpc: {}\n\tsp: {}\n\tmemory[0]: {}\n", self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.conditions, self.pc, self.sp, self.memory.read(0))
    }
}

fn check_half_carry_add(v1: u8, v2: u8) -> bool {
    let v1_masked = v1 & 0x0F;
    let v2_masked = v2 & 0x0F;
    let result = (v1_masked + v2_masked) & 0x10;
    return result == 0x10;
}

fn check_half_carry_sub(v1: u8, v2: u8) -> bool {
    let v1_masked = v1 & 0x0F;
    let v2_masked = v2 & 0x0F;
    let result = (v1_masked - v2_masked) & 0x10;
    return result == 0x10;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cpu() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let cpu = Cpu::new(memory);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.c, 0);
        assert_eq!(cpu.d, 0);
        assert_eq!(cpu.e, 0);
        assert_eq!(cpu.h, 0);
        assert_eq!(cpu.l, 0);
        assert_eq!(cpu.conditions.as_bits(), 0b00000000);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp,  0);
        assert_eq!(cpu.interrupt_enabled, true);
        assert_eq!(cpu.memory.read(0), 0);
        assert_eq!(cpu.wait_cycles, 0);
        assert_eq!(cpu.interrupt_opcode, None);
    }

    #[test]
    fn test_nop() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let cycles = cpu.nop();
        assert_eq!(cycles, 3);
    }

    #[test]
    fn test_receive_interrupt_enabled() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.receive_interrupt(1);
        let interrupt_opcode = match cpu.interrupt_opcode {
            Some(x) => x,
            None => 0b00000000
        };
        assert_eq!(interrupt_opcode, 0b00000001);
    }

    #[test]
    fn test_receive_interrupt_disabled() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.interrupt_enabled = false;
        cpu.receive_interrupt(1);
        let interrupt_opcode = match cpu.interrupt_opcode {
            Some(x) => x,
            None => 0b00000000
        };
        assert_eq!(interrupt_opcode, 0);
    }

    #[test]
    fn test_disable_interrupt() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.disable_interrupts();
        assert_eq!(cpu.interrupt_enabled, false);
    }

    #[test]
    fn test_enable_interrupt() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.disable_interrupts();
        assert_eq!(cpu.interrupt_enabled, false);
        cpu.enable_interrupts();
        assert_eq!(cpu.interrupt_enabled, true);
    }
}