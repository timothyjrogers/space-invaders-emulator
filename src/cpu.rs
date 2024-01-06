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

enum Register16 {
    BC,
    DE,
    HL,
    PSW,
    PC,
    SP,
}

pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    conditions: conditions::Conditions,
    interrupt_enabled: bool,
    memory: Box<dyn Memory>,
    wait_cycles: usize,
    interrupt_opcode: Option<u8>,
    devices: [u8; 256],
    output: Option<(u8, u8)>,
    halted: bool,
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
            pc: 0,
            sp: 0,
            conditions: conditions::Conditions::new(),
            interrupt_enabled: true,
            memory,
            wait_cycles: 0,
            interrupt_opcode: None,
            devices: [0; 256],
            output: None,
            halted: false,
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

        let instruction: u8;
        match self.interrupt_opcode {
            Some(x) => {
                self.halted = false;
                instruction = x;
            },
            None => {
                if self.halted {
                    return;
                }
                instruction = self.memory.read(self.pc);
                self.pc = self.pc + 1;
            }
        }

       self.wait_cycles = self.dispatch(instruction);
    }

    fn dispatch(&mut self, instruction: u8) -> usize {
        let mut wait_cycles;
        match instruction {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => wait_cycles = self.nop(),
            0x1 => wait_cycles = self.lxi(Register16::BC),
            0x2 => wait_cycles = self.stax(Register16::BC),
            0x3 => wait_cycles = self.inx(Register16::BC),
            0x4 => wait_cycles = self.inr(Register::B),
            0x5 => wait_cycles = self.dcr(Register::B),
            0x6 => wait_cycles = self.mvi(Register::B),
            0x7 => wait_cycles = self.rlc(),
            0x9 => wait_cycles = self.dad(Register16::BC),
            0xa => wait_cycles = self.ldax(Register16::BC),
            0xb => wait_cycles = self.dcx(Register16::BC),
            0xc => wait_cycles = self.inr(Register::C),
            0xd => wait_cycles = self.dcr(Register::C),
            0xe => wait_cycles = self.mvi(Register::C),
            0xf => wait_cycles = self.rrc(),
            0x11 => wait_cycles = self.lxi(Register16::DE),
            0x12 => wait_cycles = self.stax(Register16::DE),
            0x13 => wait_cycles = self.inx(Register16::DE),
            0x14 => wait_cycles = self.inr(Register::D),
            0x15 => wait_cycles = self.dcr(Register::D),
            0x16 => wait_cycles = self.mvi(Register::D),
            0x17 => wait_cycles = self.ral(),
            0x19 => wait_cycles = self.dad(Register16::DE),
            0x1a => wait_cycles = self.ldax(Register16::DE),
            0x1b => wait_cycles = self.dcx(Register16::DE),
            0x1c => wait_cycles = self.inr(Register::E),
            0x1d => wait_cycles = self.dcr(Register::E),
            0x1e => wait_cycles = self.mvi(Register::E),
            0x1f => wait_cycles = self.rar(),
            0x21 => wait_cycles = self.lxi(Register16::HL),
            0x22 => wait_cycles = self.shld(),
            0x23 => wait_cycles = self.inx(Register16::HL),
            0x24 => wait_cycles = self.inr(Register::H),
            0x25 => wait_cycles = self.dcr(Register::H),
            0x26 => wait_cycles = self.mvi(Register::H),
            0x27 => wait_cycles = self.daa(),
            0x29 => wait_cycles = self.dad(Register16::HL),
            0x2a => wait_cycles = self.lhld(),
            0x2b => wait_cycles = self.dcx(Register16::HL),
            0x2c => wait_cycles = self.inr(Register::L),
            0x2d => wait_cycles = self.dcr(Register::L),
            0x2e => wait_cycles = self.mvi(Register::L),
            0x2f => wait_cycles = self.cma(),
            0x31 => wait_cycles = self.lxi(Register16::SP),
            0x32 => wait_cycles = self.sta(),
            0x33 => wait_cycles = self.inx(Register16::SP),
            0x34 => wait_cycles = self.inrm(),
            0x35 => wait_cycles = self.dcrm(),
            0x36 => wait_cycles = self.mvim(),
            0x37 => wait_cycles = self.stc(),
            0x39 => wait_cycles = self.dad(Register16::SP),
            0x3a => wait_cycles = self.lda(),
            0x3b => wait_cycles = self.dcx(Register16::SP),
            0x3c => wait_cycles = self.inr(Register::A),
            0x3d => wait_cycles = self.dcr(Register::A),
            0x3e => wait_cycles = self.mvi(Register::A),
            0x3f => wait_cycles = self.cmc(),
            0x40 => wait_cycles = self.mov(Register::B, Register::B),
            0x41 => wait_cycles = self.mov(Register::C, Register::B),
            0x42 => wait_cycles = self.mov(Register::D, Register::B),
            0x43 => wait_cycles = self.mov(Register::E, Register::B),
            0x44 => wait_cycles = self.mov(Register::H, Register::B),
            0x45 => wait_cycles = self.mov(Register::L, Register::B),
            0x46 => wait_cycles = self.movm_load(Register::B),
            0x47 => wait_cycles = self.mov(Register::A, Register::B),
            0x48 => wait_cycles = self.mov(Register::B, Register::C),
            0x49 => wait_cycles = self.mov(Register::C, Register::C),
            0x4a => wait_cycles = self.mov(Register::D, Register::C),
            0x4b => wait_cycles = self.mov(Register::E, Register::C),
            0x4c => wait_cycles = self.mov(Register::H, Register::C),
            0x4d => wait_cycles = self.mov(Register::L, Register::C),
            0x4e => wait_cycles = self.movm_load(Register::C),
            0x4f => wait_cycles = self.mov(Register::A, Register::C),
            0x50 => wait_cycles = self.mov(Register::B, Register::D),
            0x51 => wait_cycles = self.mov(Register::C, Register::D),
            0x52 => wait_cycles = self.mov(Register::D, Register::D),
            0x53 => wait_cycles = self.mov(Register::E, Register::D),
            0x54 => wait_cycles = self.mov(Register::H, Register::D),
            0x55 => wait_cycles = self.mov(Register::L, Register::D),
            0x56 => wait_cycles = self.movm_load(Register::D),
            0x57 => wait_cycles = self.mov(Register::A, Register::D),
            0x58 => wait_cycles = self.mov(Register::B, Register::E),
            0x59 => wait_cycles = self.mov(Register::C, Register::E),
            0x5a => wait_cycles = self.mov(Register::D, Register::E),
            0x5b => wait_cycles = self.mov(Register::E, Register::E),
            0x5c => wait_cycles = self.mov(Register::H, Register::E),
            0x5d => wait_cycles = self.mov(Register::L, Register::E),
            0x5e => wait_cycles = self.movm_load(Register::E),
            0x5f => wait_cycles = self.mov(Register::A, Register::E),
            0x60 => wait_cycles = self.mov(Register::B, Register::H),
            0x61 => wait_cycles = self.mov(Register::C, Register::H),
            0x62 => wait_cycles = self.mov(Register::D, Register::H),
            0x63 => wait_cycles = self.mov(Register::E, Register::H),
            0x64 => wait_cycles = self.mov(Register::H, Register::H),
            0x65 => wait_cycles = self.mov(Register::L, Register::H),
            0x66 => wait_cycles = self.movm_load(Register::H),
            0x67 => wait_cycles = self.mov(Register::A, Register::H),
            0x68 => wait_cycles = self.mov(Register::B, Register::L),
            0x69 => wait_cycles = self.mov(Register::C, Register::L),
            0x6a => wait_cycles = self.mov(Register::D, Register::L),
            0x6b => wait_cycles = self.mov(Register::E, Register::L),
            0x6c => wait_cycles = self.mov(Register::H, Register::L),
            0x6d => wait_cycles = self.mov(Register::L, Register::L),
            0x6e => wait_cycles = self.movm_load(Register::L),
            0x6f => wait_cycles = self.mov(Register::A, Register::L),
            0x70 => wait_cycles = self.movm(Register::B),
            0x71 => wait_cycles = self.movm(Register::C),
            0x72 => wait_cycles = self.movm(Register::D),
            0x73 => wait_cycles = self.movm(Register::E),
            0x74 => wait_cycles = self.movm(Register::H),
            0x75 => wait_cycles = self.movm(Register::L),
            0x76 => wait_cycles = self.halt(),
            0x77 => wait_cycles = self.movm(Register::A),
            0x78 => wait_cycles = self.mov(Register::B, Register::A),
            0x79 => wait_cycles = self.mov(Register::C, Register::A),
            0x7a => wait_cycles = self.mov(Register::D, Register::A),
            0x7b => wait_cycles = self.mov(Register::E, Register::A),
            0x7c => wait_cycles = self.mov(Register::H, Register::A),
            0x7d => wait_cycles = self.mov(Register::L, Register::A),
            0x7e => wait_cycles = self.movm_load(Register::A),
            0x7f => wait_cycles = self.mov(Register::A, Register::A),
            0x80 => wait_cycles = self.add(Register::B),
            0x81 => wait_cycles = self.add(Register::C),
            0x82 => wait_cycles = self.add(Register::D),
            0x83 => wait_cycles = self.add(Register::E),
            0x84 => wait_cycles = self.add(Register::H),
            0x85 => wait_cycles = self.add(Register::L),
            0x86 => wait_cycles = self.addm(),
            0x87 => wait_cycles = self.add(Register::A),
            0x88 => wait_cycles = self.adc(Register::B),
            0x89 => wait_cycles = self.adc(Register::C),
            0x8a => wait_cycles = self.adc(Register::D),
            0x8b => wait_cycles = self.adc(Register::E),
            0x8c => wait_cycles = self.adc(Register::H),
            0x8d => wait_cycles = self.adc(Register::L),
            0x8e => wait_cycles = self.adcm(),
            0x8f => wait_cycles = self.adc(Register::A),
            0x90 => wait_cycles = self.sub(Register::B),
            0x91 => wait_cycles = self.sub(Register::C),
            0x92 => wait_cycles = self.sub(Register::D),
            0x93 => wait_cycles = self.sub(Register::E),
            0x94 => wait_cycles = self.sub(Register::H),
            0x95 => wait_cycles = self.sub(Register::L),
            0x96 => wait_cycles = self.subm(),
            0x97 => wait_cycles = self.sub(Register::A),
            0x98 => wait_cycles = self.sbb(Register::B),
            0x99 => wait_cycles = self.sbb(Register::C),
            0x9a => wait_cycles = self.sbb(Register::D),
            0x9b => wait_cycles = self.sbb(Register::E),
            0x9c => wait_cycles = self.sbb(Register::H),
            0x9d => wait_cycles = self.sbb(Register::L),
            0x9e => wait_cycles = self.sbbm(),
            0x9f => wait_cycles = self.sbb(Register::A),
            0xa0 => wait_cycles = self.ana(Register::B),
            0xa1 => wait_cycles = self.ana(Register::C),
            0xa2 => wait_cycles = self.ana(Register::D),
            0xa3 => wait_cycles = self.ana(Register::E),
            0xa4 => wait_cycles = self.ana(Register::H),
            0xa5 => wait_cycles = self.ana(Register::L),
            0xa6 => wait_cycles = self.anam(),
            0xa7 => wait_cycles = self.ana(Register::A),
            0xa8 => wait_cycles = self.xra(Register::B),
            0xa9 => wait_cycles = self.xra(Register::C),
            0xaa => wait_cycles = self.xra(Register::D),
            0xab => wait_cycles = self.xra(Register::E),
            0xac => wait_cycles = self.xra(Register::H),
            0xad => wait_cycles = self.xra(Register::L),
            0xae => wait_cycles = self.xram(),
            0xaf => wait_cycles = self.xra(Register::A),
            0xb0 => wait_cycles = self.ora(Register::B),
            0xb1 => wait_cycles = self.ora(Register::C),
            0xb2 => wait_cycles = self.ora(Register::D),
            0xb3 => wait_cycles = self.ora(Register::E),
            0xb4 => wait_cycles = self.ora(Register::H),
            0xb5 => wait_cycles = self.ora(Register::L),
            0xb6 => wait_cycles = self.oram(),
            0xb7 => wait_cycles = self.ora(Register::A),
            0xb8 => wait_cycles = self.cmp(Register::B),
            0xb9 => wait_cycles = self.cmp(Register::C),
            0xba => wait_cycles = self.cmp(Register::D),
            0xbb => wait_cycles = self.cmp(Register::E),
            0xbc => wait_cycles = self.cmp(Register::H),
            0xbd => wait_cycles = self.cmp(Register::L),
            0xbe => wait_cycles = self.cmpm(),
            0xbf => wait_cycles = self.cmp(Register::A),
            0xc0 => wait_cycles = self.ret_false(conditions::ConditionName::Zero),
            0xc1 => wait_cycles = self.pop(Register16::BC),
            0xc2 => wait_cycles = self.jmp_false(conditions::ConditionName::Zero),
            0xc3 | 0xcB => wait_cycles = self.jmp(),
            0xc4 => wait_cycles = self.call_false(conditions::ConditionName::Zero),
            0xc5 => wait_cycles = self.push(Register16::BC),
            0xc6 => wait_cycles = self.adi(),
            0xc7 => wait_cycles = self.rst(0),
            0xc8 => wait_cycles = self.ret_true(conditions::ConditionName::Zero),
            0xc9 | 0xd9 => wait_cycles = self.ret(),
            0xca => wait_cycles = self.jmp_true(conditions::ConditionName::Zero),
            0xcc => wait_cycles = self.call_true(conditions::ConditionName::Zero),
            0xcd | 0xdd | 0xed | 0xfd => wait_cycles = self.call(),
            0xce => wait_cycles = self.aci(),
            0xcf => wait_cycles = self.rst(1),
            0xd0 => wait_cycles = self.ret_false(conditions::ConditionName::Carry),
            0xd1 => wait_cycles = self.pop(Register16::DE),
            0xd2 => wait_cycles = self.jmp_false(conditions::ConditionName::Carry),
            0xd3 => wait_cycles = self.device_out(),
            0xd4 => wait_cycles = self.call_false(conditions::ConditionName::Carry),
            0xd5 => wait_cycles = self.push(Register16::DE),
            0xd6 => wait_cycles = self.sui(),
            0xd7 => wait_cycles = self.rst(2),
            0xd8 => wait_cycles = self.ret_true(conditions::ConditionName::Carry),
            0xda => wait_cycles = self.jmp_true(conditions::ConditionName::Carry),
            0xdb => wait_cycles = self.device_in(),
            0xdc => wait_cycles = self.call_true(conditions::ConditionName::Carry),
            0xde => wait_cycles = self.sbi(),
            0xdf => wait_cycles = self.rst(3),
            0xe0 => wait_cycles = self.ret_false(conditions::ConditionName::Parity),
            0xe1 => wait_cycles = self.pop(Register16::HL),
            0xe2 => wait_cycles = self.jmp_false(conditions::ConditionName::Parity),
            0xe3 => wait_cycles = self.xthl(),
            0xe4 => wait_cycles = self.call_false(conditions::ConditionName::Parity),
            0xe5 => wait_cycles = self.push(Register16::HL),
            0xe6 => wait_cycles = self.ani(),
            0xe7 => wait_cycles = self.rst(4),
            0xe8 => wait_cycles = self.ret_true(conditions::ConditionName::Parity),
            0xe9 => wait_cycles = self.pchl(),
            0xea => wait_cycles = self.jmp_true(conditions::ConditionName::Parity),
            0xeb => wait_cycles = self.xchg(),
            0xec => wait_cycles = self.call_true(conditions::ConditionName::Parity),
            0xee => wait_cycles = self.xri(),
            0xef => wait_cycles = self.rst(5),
            0xf0 => wait_cycles = self.ret_false(conditions::ConditionName::Sign),
            0xf1 => wait_cycles = self.pop(Register16::PSW),
            0xf2 => wait_cycles = self.jmp_false(conditions::ConditionName::Sign),
            0xf3 => wait_cycles = self.di(),
            0xf4 => wait_cycles = self.call_false(conditions::ConditionName::Sign),
            0xf5 => wait_cycles = self.push(Register16::PSW),
            0xf6 => wait_cycles = self.ori(),
            0xf7 => wait_cycles = self.rst(6),
            0xf8 => wait_cycles = self.ret_true(conditions::ConditionName::Sign),
            0xf9 => wait_cycles = self.sphl(),
            0xfa => wait_cycles = self.jmp_true(conditions::ConditionName::Sign),
            0xfb => wait_cycles = self.ei(),
            0xfc => wait_cycles = self.call_true(conditions::ConditionName::Sign),
            0xfe => wait_cycles = self.cpi(),
            0xff => wait_cycles = self.rst(7)        
        }
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

    /* Length: 3, Cycles: 10, Flags: None*/
    fn lxi(&mut self, register: Register16) -> usize {
        let lsb = self.memory.read(self.pc);
        let msb = self.memory.read(self.pc + 1);
        match register {
            Register16::BC => {
                self.c = lsb;
                self.b = msb;
            },
            Register16::DE => {
                self.e = lsb;
                self.d = msb;
            },
            Register16::HL => {
                self.l = lsb;
                self.h = msb;
            },
            Register16::SP => {
                self.sp = ((msb as u16) << 8) + lsb as u16;
            },
            _ => panic!("Invalid LXI register, exiting.")
        }
        self.pc = self.pc + 2;
        return 9;
    }

    /* Length: 1, Cycles: 7, Flags: None*/
    fn stax(&mut self, register: Register16) -> usize {
        let addr = match register {
            Register16::BC => concat_u8(self.b, self.c),
            Register16::DE => concat_u8(self.d, self.e),
            _ => panic!("Invalid STAX register, exiting.")
        };
        self.memory.write(addr, self.a);
        return 6; // 7 - 1
    }

    /* Length: 3, Cycles: 13, Flags: None*/
    fn sta(&mut self) -> usize {
        let addr = ((self.memory.read(self.pc + 1) as u16) << 8) + self.memory.read(self.pc) as u16;
        self.memory.write(addr, self.a);
        self.pc = self.pc + 2;
        return 12; // 13 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None*/
    fn inx(&mut self, register: Register16) -> usize {
        match register {
            Register16::BC => (self.b, self.c) = split_u16(concat_u8(self.b, self.c).wrapping_add(1)),
            Register16::DE => (self.d, self.e) = split_u16(concat_u8(self.d, self.e).wrapping_add(1)),
            Register16::HL => (self.h, self.l) = split_u16(concat_u8(self.h, self.l).wrapping_add(1)),
            Register16::SP => self.sp = self.sp.wrapping_add(1),
            _ => panic!("Invalid INX register, exiting.")
        };
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 5, Flags: SZAP */
    fn inr(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let v = value.wrapping_add(1);
        self.conditions.set(conditions::ConditionName::Zero, v == 0);
        self.conditions.set(conditions::ConditionName::Sign, v & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, v.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(value, 1));
        match register {
            Register::B => self.b = v,
            Register::C => self.c = v,
            Register::D => self.d = v,
            Register::E => self.e = v,
            Register::H => self.h = v,
            Register::L => self.l = v,
            Register::A => self.a = v,
        }
       return 4; // 5 - 1
    }

    /* Length 1, Cycles: 10, Flags SZAP */
    fn inrm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let value = self.memory.read(addr);
        let v = value.wrapping_add(1);
        self.conditions.set(conditions::ConditionName::Zero, v == 0);
        self.conditions.set(conditions::ConditionName::Sign, v & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, v.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(value, 1));
        self.memory.write(addr, v);
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 5, Flags: SZAP */
    fn dcr(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let v = value.wrapping_sub(1);
        self.conditions.set(conditions::ConditionName::Zero, v == 0);
        self.conditions.set(conditions::ConditionName::Sign, v & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, v.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(value, 1));
        match register {
            Register::B => self.b = v,
            Register::C => self.c = v,
            Register::D => self.d = v,
            Register::E => self.e = v,
            Register::H => self.h = v,
            Register::L => self.l = v,
            Register::A => self.a = v,
        }
        return 4; // 5 - 1
    }

    /* Length 1, Cycles: 10, Flags: SZAP */
    fn dcrm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let value = self.memory.read(addr);
        let v = value.wrapping_sub(1);
        self.conditions.set(conditions::ConditionName::Zero, v == 0);
        self.conditions.set(conditions::ConditionName::Sign, v & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, v.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(value, 1));
        self.memory.write(addr, v);
        return 9; // 10 - 1
    }

    /* Length 2, Cycles: 7, Flags: None */
    fn mvi(&mut self, register: Register) -> usize {
        let val = self.memory.read(self.pc);
        match register {
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.d = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
            Register::A => self.a = val,
        }
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 10, Flags: None */
    fn mvim(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        let addr = concat_u8(self.h, self.l);
        self.memory.write(addr, val);
        self.pc = self.pc + 1;
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rlc(&mut self) -> usize {
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x80 == 0x80);
        self.a = self.a.rotate_left(1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rrc(&mut self) -> usize {
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x01 == 0x01);
        self.a = self.a.rotate_right(1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn ral(&mut self) -> usize {
        let mut carry = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            carry = 1;
        }
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x80 == 0x80);
        self.a = self.a.rotate_left(1);
        if carry == 1 {
            self.a = self.a | 0x01;
        } else {
            self.a = self.a & 0xFE;
        }
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rar(&mut self) -> usize {
        let mut carry = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            carry = 1;
        }
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x01 == 0x01);
        self.a = self.a.rotate_right(1);
        if carry == 1 {
            self.a = self.a | 0x80;
        } else {
            self.a = self.a & 0x8F;
        }
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 10, Flags: C */
    fn dad(&mut self, register: Register16) -> usize {
        let val = match register {
            Register16::BC => concat_u8(self.b, self.c),
            Register16::DE => concat_u8(self.d, self.e),
            Register16::HL => concat_u8(self.h, self.l),
            Register16::SP => self.sp,
            _ => panic!("Invalid DAD register, exiting.")
        };
        let hl = concat_u8(self.h, self.l);
        let (res, overflow) = hl.overflowing_add(val);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        (self.h, self.l) = split_u16(res);
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */
    fn ldax(&mut self, register: Register16) -> usize {
        let addr = match register {
            Register16::BC => concat_u8(self.b, self.c),
            Register16::DE => concat_u8(self.d, self.e),
            _ => panic!("Invalid LDAX register, exiting.")
        };
        self.a = self.memory.read(addr);
        return 6; // 7 - 1
    }

    /* Length: 3, Cycles: 13, Flags: None */
    fn lda(&mut self) -> usize {
        let addr = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        self.a = self.memory.read(addr);
        self.pc = self.pc + 2;
        return 12; // 13 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None*/
    fn dcx(&mut self, register: Register16) -> usize {
        match register {
            Register16::BC => (self.b, self.c) = split_u16(concat_u8(self.b, self.c).wrapping_sub(1)),
            Register16::DE => (self.d, self.e) = split_u16(concat_u8(self.d, self.e).wrapping_sub(1)),
            Register16::HL => (self.h, self.l) = split_u16(concat_u8(self.h, self.l).wrapping_sub(1)),
            Register16::SP => self.sp = self.sp.wrapping_sub(1),
            _ => panic!("Invalid DCX register, exiting.")
        };
        return 4; // 5 - 1
    }

    /* Length: 3, Cycles: 16, Flags: None */
    fn shld(&mut self) -> usize {
        let addr = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        self.memory.write(addr, self.l);
        self.memory.write(addr + 1, self.h);
        self.pc = self.pc + 2;
        return 15; // 16 - 1
    }

    /* Length: 3, Cycles: 16, Flags: None */
    fn lhld(&mut self) -> usize {
        let addr = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        self.l = self.memory.read(addr);
        self.h = self.memory.read(addr + 1);
        self.pc = self.pc + 2;
        return 15; // 16 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn cma(&mut self) -> usize {
        self.a = !self.a;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn stc(&mut self) -> usize {
        self.conditions.set(conditions::ConditionName::Carry, true);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: None */
    fn cmc(&mut self) -> usize {
        self.conditions.set(conditions::ConditionName::Carry, !self.conditions.get(conditions::ConditionName::Carry));
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None */
    fn mov(&mut self, source: Register, destination: Register) -> usize {
        let val = match source {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        match destination {
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.e = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
            Register::A => self.a = val,
        }
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */ 
    fn movm_load(&mut self, register: Register) -> usize {
        let addr = concat_u8(self.h, self.l);
        match register {
            Register::B => self.b = self.memory.read(addr),
            Register::C => self.c = self.memory.read(addr),
            Register::D => self.d = self.memory.read(addr),
            Register::E => self.e = self.memory.read(addr),
            Register::H => self.h = self.memory.read(addr),
            Register::L => self.l = self.memory.read(addr),
            Register::A => self.a = self.memory.read(addr),
        }
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */ 
    fn movm(&mut self, register: Register) -> usize {
        let addr = concat_u8(self.h, self.l);
        match register {
            Register::B => self.memory.write(addr, self.b),
            Register::C => self.memory.write(addr, self.c),
            Register::D => self.memory.write(addr, self.d),
            Register::E => self.memory.write(addr, self.e),
            Register::H => self.memory.write(addr, self.h),
            Register::L => self.memory.write(addr, self.l),
            Register::A => self.memory.write(addr, self.a),
        }
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn add(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let (res, overflow) = self.a.overflowing_add(value);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, value));
        self.a = res;
        return 3; // 4 - 1
    }

    /* Length 1, Cycles: 7, Flags: SZAPC */
    fn addm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let (a, of) = self.a.overflowing_add(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
        self.a = a;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn adc(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let v = value.wrapping_add(cy);
        let (res, overflow) = self.a.overflowing_add(v);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, v));
        self.a = res;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn adcm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let v = val.wrapping_add(cy);
        let (a, of) = self.a.overflowing_add(v);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, v));
        self.a = a;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn sub(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let (res, overflow) = self.a.overflowing_sub(value);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, value));
        self.a = res;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn subm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let (a, of) = self.a.overflowing_sub(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
        self.a = a;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn sbb(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let v = value.wrapping_sub(cy);
        let (res, overflow) = self.a.overflowing_sub(v);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, v));
        self.a = res;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn sbbm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let v = val.wrapping_sub(cy);
        let (a, of) = self.a.overflowing_sub(v);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 != 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, v));
        self.a = a;
        return 7; // 6 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn ana(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let aux = ((self.a & 0x0A) >> 3) | ((value & 0x0A) >> 3) == 0x1;
        self.a = self.a & value;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, aux);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn anam(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let aux = ((self.a & 0x0A) >> 3) | ((val & 0x0A) >> 3) == 0x1;
        self.a = self.a & val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, aux);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn xra(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        self.a = self.a ^ value;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn xram(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        self.a = self.a ^ val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn ora(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        self.a = self.a | value;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        return 3; // 4 - 1
    }
    
    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn oram(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        self.a = self.a | val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn cmp(&mut self, register: Register) -> usize {
        let value = match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        };
        let (res, overflow) = self.a.overflowing_sub(value);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, value));
        return 3; // 4 - 1
    }
    
    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn cmpm(&mut self) -> usize {
        let addr = concat_u8(self.h, self.l);
        let val = self.memory.read(addr);
        let (a, of) = self.a.overflowing_sub(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
        return 7; // 6 - 1
    }

    /* Length: 1, Cycles: 10, Flags: None */
    fn pop(&mut self, register: Register16) -> usize {
        match register {
            Register16::BC => {
                self.c = self.memory.read(self.sp);
                self.b = self.memory.read(self.sp + 1);
            },
            Register16::DE => {
                self.e = self.memory.read(self.sp);
                self.d = self.memory.read(self.sp + 1);
            },
            Register16::HL => {
                self.l = self.memory.read(self.sp);
                self.h = self.memory.read(self.sp + 1);
            },
            Register16::PSW => {
                self.conditions.restore_from_bits(self.memory.read(self.sp));
                self.a = self.memory.read(self.sp + 1);
            },
            _ => panic!("Invalid POP register, exiting.")
        }
        self.sp = self.sp + 2;
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 11, Flags: None */
    fn push(&mut self, register: Register16) -> usize {
        match register {
            Register16::BC => {
                self.memory.write(self.sp - 2, self.c);
                self.memory.write(self.sp - 1, self.b);
            },
            Register16::DE => {
                self.memory.write(self.sp - 2, self.e);
                self.memory.write(self.sp - 1, self.d);
            },
            Register16::HL => {
                self.memory.write(self.sp - 2, self.l);
                self.memory.write(self.sp - 1, self.h);
            },
            Register16::PSW => {
                self.memory.write(self.sp - 2, self.conditions.as_bits());
                self.memory.write(self.sp - 1, self.a);
            },
            _ => panic!("Invalid PUSH register, exiting.")
        }
        self.sp = self.sp - 2;
        return 10; // 11 - 1
    }
    
    /* Length: 1, Cycles: 11, Flags: None */
    fn rst(&mut self, destination: u16) -> usize {
        self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
        self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
        self.sp = self.sp - 2;
        self.pc = destination;
        return 10; // 11 - 1
    }

    /* Length: 3, Cycles: 17, Flags: None */
    fn call(&mut self) -> usize {
        let immediate = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self. pc));
        self.pc = self.pc + 2;
        self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
        self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
        self.sp = self.sp - 2;
        self.pc = immediate;
        return 16; // 17 - 1
    }

    /* Length: 3, Cycles: 17/11, Flags: None */
    fn call_true(&mut self, condition: conditions::ConditionName) -> usize {
        if self.conditions.get(condition) {
            let immediate = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self. pc));
            self.pc = self.pc + 2;
            self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
            self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
            self.sp = self.sp - 2;
            self.pc = immediate;
            return 16; // 17 - 1
        }
        return 10; // 11 - 1
    }

    /* Length: 3, Cycles: 17, Flags: None */
    fn call_false(&mut self, condition: conditions::ConditionName) -> usize {
        if !self.conditions.get(condition) {
            let immediate = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self. pc));
            self.pc = self.pc + 2;
            self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
            self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
            self.sp = self.sp - 2;
            self.pc = immediate;
            return 16; // 17 - 1
        }
        return 10; // 11 - 1
    }

    /* Length: 1, Cycles: 10, Flags: None */
    fn ret(&mut self) -> usize {
        self.pc = concat_u8(self.memory.read(self.sp + 1), self.memory.read(self.sp));
        self.sp = self.sp + 2;
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 11/5, Flags: None */
    fn ret_true(&mut self, condition: conditions::ConditionName) -> usize {
        if self.conditions.get(condition) {
            self.pc = concat_u8(self.memory.read(self.sp + 1), self.memory.read(self.sp));
            self.sp = self.sp + 2;
            return 10; // 11 - 1
        }
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 11/5, Flags: None */
    fn ret_false(&mut self, condition: conditions::ConditionName) -> usize {
        if !self.conditions.get(condition) {
            self.pc = concat_u8(self.memory.read(self.sp + 1), self.memory.read(self.sp));
            self.sp = self.sp + 2;
            return 10; // 11 - 1
        }
        return 4; // 5 - 1
    }

    /* Length: 3, Cycles: 10, Flags: None */
    fn jmp(&mut self) -> usize {
        self.pc = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        return 9; // 10 - 1
    }

    /* Length: 3, Cycles: 10, Flags: None */
    fn jmp_true(&mut self, condition: conditions::ConditionName) -> usize {
        if self.conditions.get(condition) {
            self.pc = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        }
        return 9; // 10 - 1
    }

    /* Length: 3, Cycles: 10, Flags: None */
    fn jmp_false(&mut self, condition: conditions::ConditionName) -> usize {
        if !self.conditions.get(condition) {
            self.pc = concat_u8(self.memory.read(self.pc + 1), self.memory.read(self.pc));
        }
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 18, Flags: None */
    fn xthl(&mut self) -> usize {
        let lval = self.memory.read(self.sp);
        let hval = self.memory.read(self.sp + 1);
        self.memory.write(self.sp, self.l);
        self.memory.write(self.sp + 1, self.h);
        self.l = lval;
        self.h = hval;
        return 17; // 18 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None */
    fn pchl(&mut self) -> usize {
        self.pc = concat_u8(self.h, self.l);
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None */
    fn sphl(&mut self) -> usize {
        self.sp = concat_u8(self.h, self.l);
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None */
    fn xchg(&mut self) -> usize {
        let hval = self.d;
        let lval = self.e;
        self.d = self.h;
        self.e = self.l;
        self.h = hval;
        self.l = lval;
        return 4; // 5 - 1
    }

    /* Length: 2, Cycles: 10, Flags: None */
    fn device_in(&mut self) -> usize {
        let device = self.memory.read(self.pc);
        self.a = self.devices[device as usize];
        self.pc = self.pc + 1;
        return 9; // 10 - 1
    }

    /* Length: 2, Cycles: 10, Flags: None */
    fn device_out(&mut self) -> usize {
        let device = self.memory.read(self.pc);
        self.output = Some((device, self.a));
        self.pc = self.pc + 1;
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 4, Flags: None */
    fn ei(&mut self) -> usize {
        self.enable_interrupts();
        return 3;
    }

    /* Length: 1, Cycles: 4, Flags: None */
    fn di(&mut self) -> usize {
        self.disable_interrupts();
        return 3;
    }
    
    /* Length: 1, Cycles: 7, Flags: None */
    fn halt(&mut self) -> usize {
        self.halted = true;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn adi(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        let (res, overflow) = self.a.overflowing_add(val);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
        self.a = res;
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn aci(&mut self) -> usize {
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let val = self.memory.read(self.pc).wrapping_add(cy);
        let (res, overflow) = self.a.overflowing_add(val);
        self.conditions.set(conditions::ConditionName::Carry, overflow);
        self.conditions.set(conditions::ConditionName::Zero, res == 0);
        self.conditions.set(conditions::ConditionName::Sign, res & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, res.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, val));
        self.a = res;
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn sui(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        let (a, of) = self.a.overflowing_sub(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
        self.a = a;
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn sbi(&mut self) -> usize {
        let mut cy = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            cy = 1;
        }
        let val = self.memory.read(self.pc).wrapping_sub(cy);
        let (a, of) = self.a.overflowing_sub(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
        self.a = a;
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn ani(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        let aux = ((self.a & 0x0A) >> 3) | ((val & 0x0A) >> 3) == 0x1;
        self.a = self.a & val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, aux);
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn xri(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        self.a = self.a ^ val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn ori(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        self.a = self.a | val;
        self.conditions.set(conditions::ConditionName::Carry, false);
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, false);
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn cpi(&mut self) -> usize {
        let val = self.memory.read(self.pc);
        let (a, of) = self.a.overflowing_sub(val);
        self.conditions.set(conditions::ConditionName::Carry, of);
        self.conditions.set(conditions::ConditionName::Zero, a == 0);
        self.conditions.set(conditions::ConditionName::Sign, a & 0x80 == 0x80);
        self.conditions.set(conditions::ConditionName::Parity, a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, val));
        self.pc = self.pc + 1;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn daa(&mut self) -> usize {
        //TODO DAA
        return 3; // 4 - 1
    }

    fn enable_interrupts(&mut self) {
        self.interrupt_enabled = true;
    }

    fn disable_interrupts(&mut self) {
        self.interrupt_enabled = false;
    }

    pub fn set_input(&mut self, device: u8, value: u8) {
        self.devices[device as usize] = value;
    }

    pub fn get_output(&self) -> Option<(u8, u8)> {
        return self.output;
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

fn concat_u8(high: u8, low: u8) -> u16 {
    return ((high as u16) << 8) | low as u16;
}

fn split_u16(val: u16) -> (u8, u8) {
    let high = (val >> 8) as u8;
    let low = (val & 0xFF) as u8;
    return (high, low);
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
        let cpu = Cpu::new(memory);
        let wait_cycles = cpu.nop();
        assert_eq!(wait_cycles, 3);
    }

    #[test]
    fn test_lxi_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.c, 0);
        let wait_cycles = cpu.lxi(Register16::BC);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.b, 2);
        assert_eq!(cpu.c, 1);
        assert_eq!(wait_cycles, 9);
    }

    #[test]
    fn test_lxi_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
        assert_eq!(cpu.d, 0);
        assert_eq!(cpu.e, 0);
        let wait_cycles = cpu.lxi(Register16::DE);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.d, 2);
        assert_eq!(cpu.e, 1);
        assert_eq!(wait_cycles, 9);
    }

    #[test]
    fn test_lxi_hl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
        assert_eq!(cpu.h, 0);
        assert_eq!(cpu.l, 0);
        let wait_cycles = cpu.lxi(Register16::HL);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.h, 2);
        assert_eq!(cpu.l, 1);
        assert_eq!(wait_cycles, 9);
    }

    #[test]
    fn test_lxi_sp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
        assert_eq!(cpu.sp, 0);
        let wait_cycles = cpu.lxi(Register16::SP);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.sp, 0x0201);
        assert_eq!(wait_cycles, 9);
    }

    #[test]
    fn test_stax_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 1;
        cpu.b = 1;
        cpu.c = 1;
        let addr = concat_u8(cpu.b, cpu.c);
        assert_eq!(cpu.memory.read(addr), 0);
        let wait_cycles = cpu.stax(Register16::BC);
        assert_eq!(cpu.memory.read(addr), cpu.a);
        assert_eq!(cpu.pc, 0);
        assert_eq!(wait_cycles, 6);
    }

    #[test]
    fn test_stax_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 1;
        cpu.d = 1;
        cpu.e = 1;
        let addr = concat_u8(cpu.d, cpu.e);
        assert_eq!(cpu.memory.read(addr), 0);
        let wait_cycles = cpu.stax(Register16::DE);
        assert_eq!(cpu.memory.read(addr), 1);
        assert_eq!(cpu.pc, 0);
        assert_eq!(wait_cycles, 6);
    }

    #[test]
    fn test_inx_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.inx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0001);
    }

    #[test]
    fn test_inx_bc_half_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.c = 0xFF;
        let mut wait_cycles = cpu.inx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0100);
    }

    #[test]
    fn test_inx_bc_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        cpu.c = 0xFF;
        let mut wait_cycles = cpu.inx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0000);
    }

    #[test]
    fn test_inx_sp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.inx(Register16::SP);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.sp, 0x0001);
    }

    #[test]
    fn test_inx_sp_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.sp = 0xFFFF;
        let mut wait_cycles = cpu.inx(Register16::SP);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.sp, 0x0000);
    }

    #[test]
    fn test_inr_b(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_b_half_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0b00001111;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0b00010000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_b_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_inr_b_negative() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0b01111111;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0b10000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_b_positive() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_b_zero() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
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

    #[test]
    fn test_concat_u8() {
        let v1 = 0x01;
        let v2 = 0x01;
        let v3 = concat_u8(v1, v2);
        assert_eq!(v3, 0x0101);
    }
}