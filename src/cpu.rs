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
    SP,
}

impl fmt::Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match self {
            Register16::BC => "BC",
            Register16::DE => "DE",
            Register16::HL => "HL",
            Register16::PSW => "PSW",
            Register16::SP => "SP",
        };
        write!(f, "{}", val)
    }
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
            sp: 0x2400,
            conditions: conditions::Conditions::new(),
            interrupt_enabled: false,
            memory,
            wait_cycles: 0,
            interrupt_opcode: None,
            devices: [0; 256],
            output: None,
            halted: false,
        }
    }

    pub fn tick(&mut self) {
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
                self.disable_interrupts();
                self.interrupt_opcode = None;
                instruction = x;
            },
            None => {
                if self.halted {
                    return;
                }
                instruction = self.fetch_byte();
            }
        }
        self.wait_cycles = self.dispatch(instruction);
    }

    fn dispatch(&mut self, instruction: u8) -> usize {
        match instruction {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => self.nop(),
            0x1 => self.lxi(Register16::BC),
            0x2 => self.stax(Register16::BC),
            0x3 => self.inx(Register16::BC),
            0x4 => self.inr(Register::B),
            0x5 => self.dcr(Register::B),
            0x6 => self.mvi(Register::B),
            0x7 => self.rlc(),
            0x9 => self.dad(Register16::BC),
            0xa => self.ldax(Register16::BC),
            0xb => self.dcx(Register16::BC),
            0xc => self.inr(Register::C),
            0xd => self.dcr(Register::C),
            0xe => self.mvi(Register::C),
            0xf => self.rrc(),
            0x11 => self.lxi(Register16::DE),
            0x12 => self.stax(Register16::DE),
            0x13 => self.inx(Register16::DE),
            0x14 => self.inr(Register::D),
            0x15 => self.dcr(Register::D),
            0x16 => self.mvi(Register::D),
            0x17 => self.ral(),
            0x19 => self.dad(Register16::DE),
            0x1a => self.ldax(Register16::DE),
            0x1b => self.dcx(Register16::DE),
            0x1c => self.inr(Register::E),
            0x1d => self.dcr(Register::E),
            0x1e => self.mvi(Register::E),
            0x1f => self.rar(),
            0x21 => self.lxi(Register16::HL),
            0x22 => self.shld(),
            0x23 => self.inx(Register16::HL),
            0x24 => self.inr(Register::H),
            0x25 => self.dcr(Register::H),
            0x26 => self.mvi(Register::H),
            0x27 => self.daa(),
            0x29 => self.dad(Register16::HL),
            0x2a => self.lhld(),
            0x2b => self.dcx(Register16::HL),
            0x2c => self.inr(Register::L),
            0x2d => self.dcr(Register::L),
            0x2e => self.mvi(Register::L),
            0x2f => self.cma(),
            0x31 => self.lxi(Register16::SP),
            0x32 => self.sta(),
            0x33 => self.inx(Register16::SP),
            0x34 => self.inrm(),
            0x35 => self.dcrm(),
            0x36 => self.mvim(),
            0x37 => self.stc(),
            0x39 => self.dad(Register16::SP),
            0x3a => self.lda(),
            0x3b => self.dcx(Register16::SP),
            0x3c => self.inr(Register::A),
            0x3d => self.dcr(Register::A),
            0x3e => self.mvi(Register::A),
            0x3f => self.cmc(),
            0x40 => self.mov(Register::B, Register::B),
            0x41 => self.mov(Register::C, Register::B),
            0x42 => self.mov(Register::D, Register::B),
            0x43 => self.mov(Register::E, Register::B),
            0x44 => self.mov(Register::H, Register::B),
            0x45 => self.mov(Register::L, Register::B),
            0x46 => self.movm_load(Register::B),
            0x47 => self.mov(Register::A, Register::B),
            0x48 => self.mov(Register::B, Register::C),
            0x49 => self.mov(Register::C, Register::C),
            0x4a => self.mov(Register::D, Register::C),
            0x4b => self.mov(Register::E, Register::C),
            0x4c => self.mov(Register::H, Register::C),
            0x4d => self.mov(Register::L, Register::C),
            0x4e => self.movm_load(Register::C),
            0x4f => self.mov(Register::A, Register::C),
            0x50 => self.mov(Register::B, Register::D),
            0x51 => self.mov(Register::C, Register::D),
            0x52 => self.mov(Register::D, Register::D),
            0x53 => self.mov(Register::E, Register::D),
            0x54 => self.mov(Register::H, Register::D),
            0x55 => self.mov(Register::L, Register::D),
            0x56 => self.movm_load(Register::D),
            0x57 => self.mov(Register::A, Register::D),
            0x58 => self.mov(Register::B, Register::E),
            0x59 => self.mov(Register::C, Register::E),
            0x5a => self.mov(Register::D, Register::E),
            0x5b => self.mov(Register::E, Register::E),
            0x5c => self.mov(Register::H, Register::E),
            0x5d => self.mov(Register::L, Register::E),
            0x5e => self.movm_load(Register::E),
            0x5f => self.mov(Register::A, Register::E),
            0x60 => self.mov(Register::B, Register::H),
            0x61 => self.mov(Register::C, Register::H),
            0x62 => self.mov(Register::D, Register::H),
            0x63 => self.mov(Register::E, Register::H),
            0x64 => self.mov(Register::H, Register::H),
            0x65 => self.mov(Register::L, Register::H),
            0x66 => self.movm_load(Register::H),
            0x67 => self.mov(Register::A, Register::H),
            0x68 => self.mov(Register::B, Register::L),
            0x69 => self.mov(Register::C, Register::L),
            0x6a => self.mov(Register::D, Register::L),
            0x6b => self.mov(Register::E, Register::L),
            0x6c => self.mov(Register::H, Register::L),
            0x6d => self.mov(Register::L, Register::L),
            0x6e => self.movm_load(Register::L),
            0x6f => self.mov(Register::A, Register::L),
            0x70 => self.movm(Register::B),
            0x71 => self.movm(Register::C),
            0x72 => self.movm(Register::D),
            0x73 => self.movm(Register::E),
            0x74 => self.movm(Register::H),
            0x75 => self.movm(Register::L),
            0x76 => self.halt(),
            0x77 => self.movm(Register::A),
            0x78 => self.mov(Register::B, Register::A),
            0x79 => self.mov(Register::C, Register::A),
            0x7a => self.mov(Register::D, Register::A),
            0x7b => self.mov(Register::E, Register::A),
            0x7c => self.mov(Register::H, Register::A),
            0x7d => self.mov(Register::L, Register::A),
            0x7e => self.movm_load(Register::A),
            0x7f => self.mov(Register::A, Register::A),
            0x80 => self.add(Register::B),
            0x81 => self.add(Register::C),
            0x82 => self.add(Register::D),
            0x83 => self.add(Register::E),
            0x84 => self.add(Register::H),
            0x85 => self.add(Register::L),
            0x86 => self.addm(),
            0x87 => self.add(Register::A),
            0x88 => self.adc(Register::B),
            0x89 => self.adc(Register::C),
            0x8a => self.adc(Register::D),
            0x8b => self.adc(Register::E),
            0x8c => self.adc(Register::H),
            0x8d => self.adc(Register::L),
            0x8e => self.adcm(),
            0x8f => self.adc(Register::A),
            0x90 => self.sub(Register::B),
            0x91 => self.sub(Register::C),
            0x92 => self.sub(Register::D),
            0x93 => self.sub(Register::E),
            0x94 => self.sub(Register::H),
            0x95 => self.sub(Register::L),
            0x96 => self.subm(),
            0x97 => self.sub(Register::A),
            0x98 => self.sbb(Register::B),
            0x99 => self.sbb(Register::C),
            0x9a => self.sbb(Register::D),
            0x9b => self.sbb(Register::E),
            0x9c => self.sbb(Register::H),
            0x9d => self.sbb(Register::L),
            0x9e => self.sbbm(),
            0x9f => self.sbb(Register::A),
            0xa0 => self.ana(Register::B),
            0xa1 => self.ana(Register::C),
            0xa2 => self.ana(Register::D),
            0xa3 => self.ana(Register::E),
            0xa4 => self.ana(Register::H),
            0xa5 => self.ana(Register::L),
            0xa6 => self.anam(),
            0xa7 => self.ana(Register::A),
            0xa8 => self.xra(Register::B),
            0xa9 => self.xra(Register::C),
            0xaa => self.xra(Register::D),
            0xab => self.xra(Register::E),
            0xac => self.xra(Register::H),
            0xad => self.xra(Register::L),
            0xae => self.xram(),
            0xaf => self.xra(Register::A),
            0xb0 => self.ora(Register::B),
            0xb1 => self.ora(Register::C),
            0xb2 => self.ora(Register::D),
            0xb3 => self.ora(Register::E),
            0xb4 => self.ora(Register::H),
            0xb5 => self.ora(Register::L),
            0xb6 => self.oram(),
            0xb7 => self.ora(Register::A),
            0xb8 => self.cmp(Register::B),
            0xb9 => self.cmp(Register::C),
            0xba => self.cmp(Register::D),
            0xbb => self.cmp(Register::E),
            0xbc => self.cmp(Register::H),
            0xbd => self.cmp(Register::L),
            0xbe => self.cmpm(),
            0xbf => self.cmp(Register::A),
            0xc0 => self.ret_conditional(conditions::ConditionName::Zero, false),
            0xc1 => self.pop(Register16::BC),
            0xc2 => self.jmp_conditional(conditions::ConditionName::Zero, false),
            0xc3 | 0xcB => self.jmp(),
            0xc4 => self.call_conditional(conditions::ConditionName::Zero, false),
            0xc5 => self.push(Register16::BC),
            0xc6 => self.adi(),
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => self.rst(instruction),
            0xc8 => self.ret_conditional(conditions::ConditionName::Zero, true),
            0xc9 | 0xd9 => self.ret(),
            0xca => self.jmp_conditional(conditions::ConditionName::Zero, true),
            0xcc => self.call_conditional(conditions::ConditionName::Zero, true),
            0xcd | 0xdd | 0xed | 0xfd => self.call(),
            0xce => self.aci(),
            0xd0 => self.ret_conditional(conditions::ConditionName::Carry, false),
            0xd1 => self.pop(Register16::DE),
            0xd2 => self.jmp_conditional(conditions::ConditionName::Carry, false),
            0xd3 => self.device_out(),
            0xd4 => self.call_conditional(conditions::ConditionName::Carry, false),
            0xd5 => self.push(Register16::DE),
            0xd6 => self.sui(),
            0xd8 => self.ret_conditional(conditions::ConditionName::Carry, true),
            0xda => self.jmp_conditional(conditions::ConditionName::Carry, true),
            0xdb => self.device_in(),
            0xdc => self.call_conditional(conditions::ConditionName::Carry, true),
            0xde => self.sbi(),
            0xe0 => self.ret_conditional(conditions::ConditionName::Parity, false),
            0xe1 => self.pop(Register16::HL),
            0xe2 => self.jmp_conditional(conditions::ConditionName::Parity, false),
            0xe3 => self.xthl(),
            0xe4 => self.call_conditional(conditions::ConditionName::Parity, false),
            0xe5 => self.push(Register16::HL),
            0xe6 => self.ani(),
            0xe8 => self.ret_conditional(conditions::ConditionName::Parity, true),
            0xe9 => self.pchl(),
            0xea => self.jmp_conditional(conditions::ConditionName::Parity, true),
            0xeb => self.xchg(),
            0xec => self.call_conditional(conditions::ConditionName::Parity, true),
            0xee => self.xri(),
            0xf0 => self.ret_conditional(conditions::ConditionName::Sign, false),
            0xf1 => self.pop(Register16::PSW),
            0xf2 => self.jmp_conditional(conditions::ConditionName::Sign, false),
            0xf3 => self.di(),
            0xf4 => self.call_conditional(conditions::ConditionName::Sign, false),
            0xf5 => self.push(Register16::PSW),
            0xf6 => self.ori(),
            0xf8 => self.ret_conditional(conditions::ConditionName::Sign, true),
            0xf9 => self.sphl(),
            0xfa => self.jmp_conditional(conditions::ConditionName::Sign, true),
            0xfb => self.ei(),
            0xfc => self.call_conditional(conditions::ConditionName::Sign, true),
            0xfe => self.cpi(),
        }
    }

    /* Length: 1, Cycles: 4, Flags: None*/
    fn nop(&self) -> usize {
        return 3;
    }

    /* Length: 3, Cycles: 10, Flags: None*/
    fn lxi(&mut self, register: Register16) -> usize {
        let immediate = self.fetch_two_bytes();
        self.set_two_byte_register(immediate, &register);
        return 9;
    }

    /* Length: 1, Cycles: 7, Flags: None*/
    fn stax(&mut self, register: Register16) -> usize {
        let addr = self.get_two_byte_register(&register);
        self.memory.write(addr, self.a);
        return 6; // 7 - 1
    }

    /* Length: 3, Cycles: 13, Flags: None*/
    fn sta(&mut self) -> usize {
        let immediate = self.fetch_two_bytes();
        self.memory.write(immediate, self.a);
        return 12; // 13 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None*/
    fn inx(&mut self, register: Register16) -> usize {
        let value = self.get_two_byte_register(&register);
        self.set_two_byte_register(value.wrapping_add(1), &register);
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 5, Flags: SZAP */
    fn inr(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let result = self.add_sub_8bit(value, 1);
        self.set_one_byte_register(result as u8, &register);
       return 4; // 5 - 1
    }

    /* Length 1, Cycles: 10, Flags SZAP */
    fn inrm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let result = self.add_sub_8bit(value, 1);
        self.memory.write(addr, result as u8);
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 5, Flags: SZAP */
    fn dcr(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let result = self.add_sub_8bit(value, (1 as u8).wrapping_neg());
        self.set_one_byte_register(result as u8, &register);
        return 4; // 5 - 1
    }

    /* Length 1, Cycles: 10, Flags: SZAP */
    fn dcrm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let result = self.add_sub_8bit(value, (1 as u8).wrapping_neg());
        self.memory.write(addr, result as u8);
        return 9; // 10 - 1
    }

    /* Length 2, Cycles: 7, Flags: None */
    fn mvi(&mut self, register: Register) -> usize {
        let value = self.fetch_byte();
        self.set_one_byte_register(value, &register);
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 10, Flags: None */
    fn mvim(&mut self) -> usize {
        let value = self.fetch_byte();
        let addr = self.get_two_byte_register(&Register16::HL);
        self.memory.write(addr, value);
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rlc(&mut self) -> usize {
        self.a = self.a.rotate_left(1);
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x1 == 1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rrc(&mut self) -> usize {
        self.conditions.set(conditions::ConditionName::Carry, self.a & 0x1 == 1);
        self.a = self.a.rotate_right(1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn ral(&mut self) -> usize {
        let carry = self.a & 0x1;
        self.a = self.a << 1;
        let carry_bit: u8 = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        self.a = self.a | carry_bit;
        self.conditions.set(conditions::ConditionName::Carry, carry == 1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 4, Flags: C */
    fn rar(&mut self) -> usize {
        let carry = self.a & 0x1;
        self.a = self.a >> 1;
        let carry_bit: u8 = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        self.a = self.a | (carry_bit << 7);
        self.conditions.set(conditions::ConditionName::Carry, carry == 1);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 10, Flags: C */
    fn dad(&mut self, register: Register16) -> usize {
        let value = self.get_two_byte_register(&register);
        let hl = self.get_two_byte_register(&Register16::HL);
        let result = (hl as u32) + (value as u32);
        self.conditions.set(conditions::ConditionName::Carry, result > u16::MAX.into());
        self.set_two_byte_register(result as u16, &Register16::HL);
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */
    fn ldax(&mut self, register: Register16) -> usize {
        let addr = self.get_two_byte_register(&register);
        self.a = self.memory.read(addr);
        return 6; // 7 - 1
    }

    /* Length: 3, Cycles: 13, Flags: None */
    fn lda(&mut self) -> usize {
        let immediate = self.fetch_two_bytes();
        self.a = self.memory.read(immediate);
        return 12; // 13 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None*/
    fn dcx(&mut self, register: Register16) -> usize {
        let value = self.get_two_byte_register(&register);
        self.set_two_byte_register(value.wrapping_sub(1), &register);
        return 4; // 5 - 1
    }

    /* Length: 3, Cycles: 16, Flags: None */
    fn shld(&mut self) -> usize {
        let immediate = self.fetch_two_bytes();
        self.memory.write(immediate, self.l);
        self.memory.write(immediate + 1, self.h);
        return 15; // 16 - 1
    }

    /* Length: 3, Cycles: 16, Flags: None */
    fn lhld(&mut self) -> usize {
        let immediate = self.fetch_two_bytes();
        self.l = self.memory.read(immediate);
        self.h = self.memory.read(immediate + 1);
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
        let value = self.get_one_byte_register(&source);
        self.set_one_byte_register(value, &destination);
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */ 
    fn movm_load(&mut self, register: Register) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        self.set_one_byte_register(value, &register);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 7, Flags: None */ 
    fn movm(&mut self, register: Register) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.get_one_byte_register(&register);
        self.memory.write(addr, value);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn add(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let result = self.add_sub_8bit(self.a, value);
        self.conditions.set(conditions::ConditionName::Carry, result > u8::MAX.into());
        self.a = result as u8;
        return 3; // 4 - 1
    }

    /* Length 1, Cycles: 7, Flags: SZAPC */
    fn addm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let result = self.add_sub_8bit(self.a, value);
        self.conditions.set(conditions::ConditionName::Carry, result > u8::MAX.into());
        self.a = result as u8;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn adc(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let carry = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        let result = self.add_sub_8bit(self.a, value + carry);
        self.conditions.set(conditions::ConditionName::Carry, result > u8::MAX.into());
        self.a = result as u8;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn adcm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let carry = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        let result = self.add_sub_8bit(self.a, value + carry);
        self.conditions.set(conditions::ConditionName::Carry, result > u8::MAX.into());
        self.a = result as u8;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn sub(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let result = self.add_sub_8bit(self.a, value.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = result as u8;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn subm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let result = self.add_sub_8bit(self.a, value.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = result as u8;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn sbb(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let carry: u8 = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        let result = self.add_sub_8bit(self.a, value.wrapping_neg() + carry.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = result as u8;
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn sbbm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let mut carry: u8 = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            carry = 1;
        }
        let result = self.add_sub_8bit(self.a, value.wrapping_neg() + carry.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = result as u8;
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn ana(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        self.a = self.a & value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn anam(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        self.a = self.a & value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn xra(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        self.a = self.a ^ value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 3; // 4 - 1
    }

    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn xram(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        self.a = self.a ^ value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn ora(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        self.a = self.a | value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 3; // 4 - 1
    }
    
    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn oram(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        self.a = self.a | value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn cmp(&mut self, register: Register) -> usize {
        let value = self.get_one_byte_register(&register);
        let _result = self.add_sub_8bit(self.a, value.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        return 3; // 4 - 1
    }
    
    /* Length: 1, Cycles: 7, Flags: SZAPC */
    fn cmpm(&mut self) -> usize {
        let addr = self.get_two_byte_register(&Register16::HL);
        let value = self.memory.read(addr);
        let _result = self.add_sub_8bit(self.a, value.wrapping_neg());
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 10, Flags: None */
    fn pop(&mut self, register: Register16) -> usize {
        let lsb = self.memory.read(self.sp);
        let msb = self.memory.read(self.sp + 1);
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
            Register16::PSW => {
                self.conditions.restore_from_bits(lsb);
                self.a = msb;
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

    fn rst(&mut self, opcode: u8) -> usize {
        let destination = ((opcode & 0b00111000)) as u16;
        self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
        self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
        self.sp = self.sp - 2;
        self.pc = destination;
        return 10; // 11 - 1
    }

    /* Length: 3, Cycles: 17, Flags: None */
    fn call(&mut self) -> usize {
        let immediate = self.fetch_two_bytes();
        self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
        self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
        self.sp = self.sp - 2;
        self.pc = immediate;
        return 16; // 17 - 1
    }

    fn call_conditional(&mut self, condition: conditions::ConditionName, value: bool) -> usize {
        if self.conditions.get(condition) == value {
            let immediate = self.fetch_two_bytes();
            self.memory.write(self.sp - 1, (self.pc >> 8) as u8);
            self.memory.write(self.sp - 2, (self.pc & 0xFF) as u8);
            self.sp = self.sp - 2;
            self.pc = immediate;
            return 16; // 17 - 1
        } else {
            self.pc = self.pc + 2;
            return 10; // 11 - 1
        }
    }

    /* Length: 1, Cycles: 10, Flags: None */
    fn ret(&mut self) -> usize {
        self.pc = concat_u8(self.memory.read(self.sp + 1), self.memory.read(self.sp));
        self.sp = self.sp + 2;
        return 9; // 10 - 1
    }

    /* Length: 1, Cycles: 11/5, Flags: None */
    fn ret_conditional(&mut self, condition: conditions::ConditionName, value: bool) -> usize {
        if self.conditions.get(condition) == value {
            self.pc = concat_u8(self.memory.read(self.sp + 1), self.memory.read(self.sp));
            self.sp = self.sp + 2;
            return 10; // 11 - 1
        }
        return 4; // 5 - 1
    }

    /* Length: 3, Cycles: 10, Flags: None */
    fn jmp(&mut self) -> usize {
        self.pc = self.fetch_two_bytes();
        return 9; // 10 - 1
    }

    fn jmp_conditional(&mut self, condition: conditions::ConditionName, value: bool) -> usize {
        if self.conditions.get(condition) == value {
            self.pc = self.fetch_two_bytes();
        } else {
            self.pc = self.pc + 2;
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
        self.pc = self.get_two_byte_register(&Register16::HL);
        return 4; // 5 - 1
    }

    /* Length: 1, Cycles: 5, Flags: None */
    fn sphl(&mut self) -> usize {
        self.sp = self.get_two_byte_register(&Register16::HL);
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
        let device = self.fetch_byte();
        self.a = self.devices[device as usize];
        return 9; // 10 - 1
    }

    /* Length: 2, Cycles: 10, Flags: None */
    fn device_out(&mut self) -> usize {
        let device = self.fetch_byte();
        self.output = Some((device, self.a));
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
        let value = self.fetch_byte();
        let result = (self.a as u16) + (value as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, value));
        self.conditions.set(conditions::ConditionName::Carry, result > 0xFF);
        self.a = lsb;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn aci(&mut self) -> usize {
        let carry = if self.conditions.get(conditions::ConditionName::Carry) {
            1
        } else {
            0
        };
        let value = self.fetch_byte();
        let result = (self.a as u16) + ((value + carry) as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, value));
        self.conditions.set(conditions::ConditionName::Carry, result > 0xFF);
        self.a = lsb;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn sui(&mut self) -> usize {
        let value = self.fetch_byte();
        let result = (self.a as u16) + (value.wrapping_neg() as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, value));
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = lsb;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn sbi(&mut self) -> usize {
        let mut carry: u8 = 0;
        if self.conditions.get(conditions::ConditionName::Carry) {
            carry = 1;
        }
        let value = self.fetch_byte();
        let result = (self.a as u16) + (value.wrapping_neg() as u16 + carry.wrapping_neg() as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, value));
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        self.a = lsb;
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn ani(&mut self) -> usize {
        let value = self.fetch_byte();
        self.a = self.a & value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn xri(&mut self) -> usize {
        let value = self.fetch_byte();
        self.a = self.a ^ value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn ori(&mut self) -> usize {
        let value = self.fetch_byte();
        self.a = self.a | value;
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Carry, false);
        return 6; // 7 - 1
    }

    /* Length: 2, Cycles: 7, Flags: SZAPC */
    fn cpi(&mut self) -> usize {
        let value = self.fetch_byte();
        let result = (self.a as u16) + (value.wrapping_neg() as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_sub(self.a, value));
        self.conditions.set(conditions::ConditionName::Carry, self.a < value);
        return 6; // 7 - 1
    }

    /* Length: 1, Cycles: 4, Flags: SZAPC */
    fn daa(&mut self) -> usize {
        if self.a & 0x0F > 9 || self.conditions.get(conditions::ConditionName::Auxillary) {
            self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(self.a, 6));
            self.a = self.a + 6;
        }
        if (self.a & 0xF0) >> 4 > 9 || self.conditions.get(conditions::ConditionName::Carry) {
            let mut upper_nibble = (self.a & 0xF0) >> 4;
            upper_nibble += 6;
            self.conditions.set(conditions::ConditionName::Carry, upper_nibble > 0xF);
            self.a = (upper_nibble << 4) | (self.a & 0x0F);
        }
        self.conditions.set(conditions::ConditionName::Zero, self.a == 0);
        self.conditions.set(conditions::ConditionName::Sign, self.a >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, self.a.count_ones() % 2 == 0);
        return 3; // 4 - 1
    }

    fn enable_interrupts(&mut self) {
        self.interrupt_enabled = true;
    }

    fn disable_interrupts(&mut self) {
        self.interrupt_enabled = false;
    }

    fn fetch_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc = self.pc + 1;
        return self.memory.read(pc);
    }

    fn fetch_two_bytes(&mut self) -> u16 {
        let lsb = self.memory.read(self.pc);
        let msb = self.memory.read(self.pc + 1);
        self.pc = self.pc + 2;
        return concat_u8(msb, lsb);
    }

    fn get_one_byte_register(&mut self, register: &Register) -> u8 {
        match register {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        }
    }

    fn set_one_byte_register(&mut self, value: u8, register: &Register) {
        match register {
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
            Register::A => self.a = value,
        };
    }

    fn get_two_byte_register(&mut self, register: &Register16) -> u16 {
        match register {
            Register16::BC => concat_u8(self.b, self.c),
            Register16::DE => concat_u8(self.d, self.e),
            Register16::HL => concat_u8(self.h, self.l),
            Register16::SP => self.sp,
            _ => panic!("Invalid LXI register, exiting.")
        }
    }

    fn set_two_byte_register(&mut self, value: u16, register: &Register16) {
        let (msb, lsb) = split_u16(value);
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
                self.sp = value;
            },
            _ => panic!("Invalid LXI register, exiting.")
        }
    }

    fn add_sub_8bit(&mut self, v1: u8, v2: u8) -> u16 {
        let result = (v1 as u16) + (v2 as u16);
        let lsb = result as u8;
        self.conditions.set(conditions::ConditionName::Zero, lsb == 0);
        self.conditions.set(conditions::ConditionName::Sign, lsb >= 0x80);
        self.conditions.set(conditions::ConditionName::Parity, lsb.count_ones() % 2 == 0);
        self.conditions.set(conditions::ConditionName::Auxillary, check_half_carry_add(v1, v2));
        return result;
    }

    pub fn receive_interrupt(&mut self, interrupt: u8) {
        self.interrupt_opcode = Some(interrupt);
    }

    pub fn set_input(&mut self, device: u8, value: u8) {
        self.devices[device as usize] = value;
    }

    pub fn get_output(&mut self) -> Option<(u8, u8)> {
        let output = self.output;
        self.output = None;
        return output;
    }

    pub fn get_vram(&self) -> [u8; 7_168] {
        let mut vram: [u8; 7_168] = [0; 7_168];
        for i in 0..7_168 {
            vram[i] = self.memory.read((0x2400 + i) as u16);
        }
        return vram;    
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
    //let result = (v1_masked + v2_masked) & 0x10;
    let result = v1_masked.overflowing_add(v2_masked).0 & 0x10;
    return result == 0x10;
}

fn check_half_carry_sub(v1: u8, v2: u8) -> bool {
    let v1_masked = v1 & 0x0F;
    let v2_masked = v2 & 0x0F;
    //let result = (v1_masked - v2_masked) & 0x10;
    let result = v1_masked.overflowing_sub(v2_masked).0 & 0x10;
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
        assert_eq!(cpu.conditions.as_bits(), 0b00000010);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp,  0x2400);
        assert_eq!(cpu.interrupt_enabled, false);
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
    fn test_daa() {
        let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0x9B;
        let wait_cycles = cpu.daa();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.conditions.get(conditions::ConditionName::Carry), true);
        assert_eq!(cpu.conditions.get(conditions::ConditionName::Auxillary), true);
    }

    #[test]
    fn test_lxi_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
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
        let wait_cycles = cpu.stax(Register16::DE);
        assert_eq!(cpu.memory.read(addr), 1);
        assert_eq!(cpu.pc, 0);
        assert_eq!(wait_cycles, 6);
    }

    #[test]
    fn test_sta() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 2);
        let mut cpu = Cpu::new(memory);
        cpu.a = 1;
        let wait_cycles = cpu.sta();
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.memory.read(0x0201), 1);
        assert_eq!(wait_cycles, 12);
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
    fn test_inx_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.inx(Register16::DE);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.d, cpu.e), 0x0001);
    }

    #[test]
    fn test_inx_hl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.inx(Register16::HL);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.h, cpu.l), 0x0001);
    }

    #[test]
    fn test_inx_half_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.c = 0xFF;
        let mut wait_cycles = cpu.inx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0100);
    }

    #[test]
    fn test_inx_sp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.inx(Register16::SP);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.sp, 0x2401);
    }

    #[test]
    fn test_inx_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        cpu.c = 0xFF;
        let mut wait_cycles = cpu.inx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0000);
    }

    #[test]
    fn test_inr_b(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_c(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::C);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.c, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_d(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::D);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.d, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_e(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::E);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.e, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_h(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::H);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.h, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_l(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::L);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.l, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_a(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::A);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_inr_negative() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0b01111111;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0b10000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_positive() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_inr_zero() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        let wait_cycles = cpu.inr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_inrm() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.inrm();
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.memory.read(0), 2);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_b(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 2;
        let wait_cycles = cpu.dcr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_c(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.c = 2;
        let wait_cycles = cpu.dcr(Register::C);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.c, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_d(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.d = 2;
        let wait_cycles = cpu.dcr(Register::D);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.d, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_e(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.e = 2;
        let wait_cycles = cpu.dcr(Register::E);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.e, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_h(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.h = 2;
        let wait_cycles = cpu.dcr(Register::H);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.h, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_l(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.l = 2;
        let wait_cycles = cpu.dcr(Register::L);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.l, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_a(){
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 2;
        let wait_cycles = cpu.dcr(Register::A);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.dcr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0xFF);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_negative() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.dcr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0xFF);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_positive() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 2;
        let wait_cycles = cpu.dcr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_dcr_zero() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 1;
        let wait_cycles = cpu.dcr(Register::B);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.b, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_dcrm() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 2);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.dcrm();
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.memory.read(0),1);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_mvi_b() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::B);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.b, 1);
    }

    #[test]
    fn test_mvi_c() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::C);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.c, 1);
    }

    #[test]
    fn test_mvi_d() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::D);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.d, 1);
    }

    #[test]
    fn test_mvi_e() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::E);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.e, 1);
    }

    #[test]
    fn test_mvi_h() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::H);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.h, 1);
    }

    #[test]
    fn test_mvi_l() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::L);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.l, 1);
    }

    #[test]
    fn test_mvi_a() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.mvi(Register::A);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn test_mvim() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        let mut cpu = Cpu::new(memory);
        cpu.h = 1;
        cpu.l = 1;
        let wait_cycles = cpu.mvim();
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.memory.read(0x0101), 1);
    }

    #[test]
    fn test_rlc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b00000001;
        let wait_cycles = cpu.rlc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b00000010);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_rlc_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b10000000;
        let wait_cycles = cpu.rlc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b00000001);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_rrc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b10000000;
        let wait_cycles = cpu.rrc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b01000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_rrc_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b00000001;
        let wait_cycles = cpu.rrc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b10000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_ral() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b01000000;
        let wait_cycles = cpu.ral();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b10000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_ral_carry_out() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b10000001;
        let wait_cycles = cpu.ral();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b00000010);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_ral_carry_in() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.a = 0b01000000;
        let wait_cycles = cpu.ral();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b10000001);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_rar() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b00000010;
        let wait_cycles = cpu.rar();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b00000001);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_rar_carry_out() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b00000001;
        let wait_cycles = cpu.rar();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b00000000);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_rar_carry_in() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.a = 0b00000010;
        let wait_cycles = cpu.rar();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b10000001);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_dad_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 1;
        cpu.c = 1;
        cpu.h = 1;
        cpu.l = 1;
        let wait_cycles = cpu.dad(Register16::BC);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 0x02);
        assert_eq!(cpu.l, 0x02);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_dad_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.d = 1;
        cpu.e = 1;
        cpu.h = 1;
        cpu.l = 1;
        let wait_cycles = cpu.dad(Register16::DE);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 0x02);
        assert_eq!(cpu.l, 0x02);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_dad_hl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.h = 1;
        cpu.l = 1;
        let wait_cycles = cpu.dad(Register16::HL);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 0x02);
        assert_eq!(cpu.l, 0x02);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
    fn test_dad_sp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.sp = 0x0101;
        cpu.h = 1;
        cpu.l = 1;
        let wait_cycles = cpu.dad(Register16::SP);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 0x02);
        assert_eq!(cpu.l, 0x02);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);

    }

    #[test]
    fn test_dad_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0xFF;
        cpu.c = 0xFF;
        cpu.h = 0x00;
        cpu.l = 0x01;
        let wait_cycles = cpu.dad(Register16::BC);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 0);
        assert_eq!(cpu.l, 0);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_ldax_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x0101, 1);
        let mut cpu = Cpu::new(memory);
        cpu.b = 1;
        cpu.c = 1;
        let wait_cycles = cpu.ldax(Register16::BC);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn test_ldax_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x0101, 1);
        let mut cpu = Cpu::new(memory);
        cpu.d = 1;
        cpu.e = 1;
        let wait_cycles = cpu.ldax(Register16::DE);
        assert_eq!(wait_cycles, 6);
        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn test_lda() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 1);
        memory.write(0x0101, 1);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.lda();
        assert_eq!(wait_cycles, 12);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.a, 1);
    }

    #[test]
    fn test_dcx_bc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.b = 0x01;
        cpu.c = 0x01;
        let mut wait_cycles = cpu.dcx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0x0100);
    }

    #[test]
    fn test_dcx_de() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.d = 0x01;
        cpu.e = 0x01;
        let mut wait_cycles = cpu.dcx(Register16::DE);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.d, cpu.e), 0x0100);
    }

    #[test]
    fn test_dcx_hl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.h = 0x01;
        cpu.l = 0x01;
        let mut wait_cycles = cpu.dcx(Register16::HL);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.h, cpu.l), 0x0100);
    }

    #[test]
    fn test_dcx_sp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.dcx(Register16::SP);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.sp, 0x23FF);
    }

    #[test]
    fn test_dcx_carry() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let mut wait_cycles = cpu.dcx(Register16::BC);
        assert_eq!(wait_cycles, 4);
        assert_eq!(concat_u8(cpu.b, cpu.c), 0xFFFF);
    }

    #[test]
    fn test_shld() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 1);
        let mut cpu = Cpu::new(memory);
        cpu.h = 0x0A;
        cpu.l = 0x0B;
        let wait_cycles = cpu.shld();
        assert_eq!(wait_cycles, 15);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.memory.read(0x0101), 0x0B);
        assert_eq!(cpu.memory.read(0x0102), 0x0A);
    }

    #[test]
    fn test_lhld() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 1);
        memory.write(1, 1);
        memory.write(0x0101, 0x0B);
        memory.write(0x0102, 0x0A);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.lhld();
        assert_eq!(wait_cycles, 15);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.l, 0x0B);
        assert_eq!(cpu.h, 0x0A);
    }

    #[test]
    fn test_cma() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 0b01010101;
        let wait_cycles = cpu.cma();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.a, 0b10101010);
    }

    #[test]
    fn test_stc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.stc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
    }

    #[test]
    fn test_cmc() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.cmc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
        let wait_cycles = cpu.cmc();
        assert_eq!(wait_cycles, 3);
        assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
    }

    #[test]
	fn test_mov_b_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_mov_b_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::B);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.b, 1);
	}

	#[test]
	fn test_movm_load_b() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::B);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.b, 0x1);
	}

	#[test]
	fn test_mov_c_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_mov_c_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::C);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.c, 1);
	}

	#[test]
	fn test_movm_load_c() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::C);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.c, 0x1);
	}

	#[test]
	fn test_mov_d_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_mov_d_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::D);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.d, 1);
	}

	#[test]
	fn test_movm_load_d() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::D);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.d, 0x1);
	}

	#[test]
	fn test_mov_e_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_mov_e_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::E);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.e, 1);
	}

	#[test]
	fn test_movm_load_e() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::E);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.e, 0x1);
	}

	#[test]
	fn test_mov_h_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_mov_h_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::H);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.h, 1);
	}

	#[test]
	fn test_movm_load_h() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::H);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.h, 0x1);
	}

	#[test]
	fn test_mov_l_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_mov_l_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::L);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.l, 1);
	}

	#[test]
	fn test_movm_load_l() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::L);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.l, 0x1);
	}

	#[test]
	fn test_mov_a_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.mov(Register::B, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.mov(Register::C, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.mov(Register::D, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.mov(Register::E, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.mov(Register::H, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.mov(Register::L, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_mov_a_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.mov(Register::A, Register::A);
		assert_eq!(wait_cycles, 4);
		assert_eq!(cpu.a, 1);
	}

	#[test]
	fn test_movm_load_a() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.movm_load(Register::A);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.a, 0x1);
	}

	#[test]
	fn test_movm_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.movm(Register::B);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0), 1);
	}

	#[test]
	fn test_movm_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.movm(Register::C);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0), 1);
	}

	#[test]
	fn test_movm_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.movm(Register::D);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0), 1);
	}

	#[test]
	fn test_movm_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.movm(Register::E);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0), 1);
	}

	#[test]
	fn test_movm_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.movm(Register::H);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0x0100), 1);
	}

	#[test]
	fn test_movm_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.movm(Register::L);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0x0001), 1);
	}

	#[test]
	fn test_movm_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.movm(Register::A);
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.memory.read(0), 1);
	}

	#[test]
	fn test_add_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		let wait_cycles = cpu.add(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		let wait_cycles = cpu.add(Register::C);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		let wait_cycles = cpu.add(Register::D);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		let wait_cycles = cpu.add(Register::E);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		let wait_cycles = cpu.add(Register::H);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		let wait_cycles = cpu.add(Register::L);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_add_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		let wait_cycles = cpu.add(Register::A);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_addm() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		let wait_cycles = cpu.addm();
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

    #[test]
    fn test_add_carry_halfcarry_zero() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
        cpu.a = 0x01;
		cpu.b = 0xFF;
		let wait_cycles = cpu.add(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 0);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_add_negative() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
        cpu.a = 0x01;
		cpu.b = 0x7F;
		let wait_cycles = cpu.add(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 0x80);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
	fn test_adc_b() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.b = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_c() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.c = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::C);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_d() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.d = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::D);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_e() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.e = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::E);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_h() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.h = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::H);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_l() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.l = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::L);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adc_a() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
		cpu.a = 1;
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adc(Register::A);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 3);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

	#[test]
	fn test_adcm() {
		let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		memory.write(0, 1);
		let mut cpu = Cpu::new(memory);
		cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
		let wait_cycles = cpu.adcm();
		assert_eq!(wait_cycles, 6);
		assert_eq!(cpu.a, 2);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
	}

    #[test]
    fn test_adc_carry_halfcarry_zero() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.a = 0x01;
		cpu.b = 0xFE;
		let wait_cycles = cpu.adc(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 0);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
    }

    #[test]
    fn test_adc_negative() {
		let memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
		let mut cpu = Cpu::new(memory);
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.a = 0x01;
		cpu.b = 0x7E;
		let wait_cycles = cpu.adc(Register::B);
		assert_eq!(wait_cycles, 3);
		assert_eq!(cpu.a, 0x80);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
    }

    #[test]
    fn test_pop_b() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.pop(Register16::BC);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.b, 2);
        assert_eq!(cpu.c, 1);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_pop_d() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.pop(Register16::DE);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.d, 2);
        assert_eq!(cpu.e, 1);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_pop_h() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.pop(Register16::HL);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.h, 2);
        assert_eq!(cpu.l, 1);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_pop_psw_set() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 0xFF);
        memory.write(0x23FF, 1);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.pop(Register16::PSW);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), true);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), true);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_pop_psw_unset() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 0x00);
        memory.write(0x23FF, 1);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.conditions.set(crate::conditions::ConditionName::Auxillary, true);
        cpu.conditions.set(crate::conditions::ConditionName::Parity, true);
        cpu.conditions.set(crate::conditions::ConditionName::Sign, true);
        cpu.conditions.set(crate::conditions::ConditionName::Zero, true);
        let wait_cycles = cpu.pop(Register16::PSW);
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.a, 1);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Carry), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Auxillary), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Parity), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Sign), false);
		assert_eq!(cpu.conditions.get(crate::conditions::ConditionName::Zero), false);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_push_b() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.sp = 2;
        cpu.b = 1;
        cpu.c = 2;
        let wait_cycles = cpu.push(Register16::BC);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.memory.read(0), 2);
        assert_eq!(cpu.memory.read(1), 1);
    }

    #[test]
    fn test_push_d() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.sp = 2;
        cpu.d = 1;
        cpu.e = 2;
        let wait_cycles = cpu.push(Register16::DE);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.memory.read(0), 2);
        assert_eq!(cpu.memory.read(1), 1);
    }

    #[test]
    fn test_push_h() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.sp = 2;
        cpu.h = 1;
        cpu.l = 2;
        let wait_cycles = cpu.push(Register16::HL);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.memory.read(0), 2);
        assert_eq!(cpu.memory.read(1), 1);
    }

    #[test]
    fn test_push_psw() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.a = 1;
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        cpu.conditions.set(crate::conditions::ConditionName::Auxillary, true);
        cpu.conditions.set(crate::conditions::ConditionName::Parity, true);
        cpu.conditions.set(crate::conditions::ConditionName::Sign, true);
        cpu.conditions.set(crate::conditions::ConditionName::Zero, true);
        let wait_cycles = cpu.push(Register16::PSW);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.memory.read(0x23FE), 0b11010111);
        assert_eq!(cpu.memory.read(0x23FF), 1);
    }

    #[test]
    fn test_rnz_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Zero, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Zero, false);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rnz_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Zero, false);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rz_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Zero, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Zero, true);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);

    }

    #[test]
    fn test_rz_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Zero, true);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rnc_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Carry, false);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rnc_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Carry, false);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rc_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Carry, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Carry, true);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rc_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Carry, true);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rpo_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Parity, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Parity, false);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rpo_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Parity, false);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rpe_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Parity, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Parity, true);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rpe_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Parity, true);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rp_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Sign, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Sign, false);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_rp_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Sign, false);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rm_true() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.conditions.set(crate::conditions::ConditionName::Sign, true);
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Sign, true);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_rm_false() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret_conditional(crate::conditions::ConditionName::Sign, true);
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x23FE);
    }

    #[test]
    fn test_ret() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 1);
        memory.write(0x23FF, 2);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        let wait_cycles = cpu.ret();
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(cpu.sp, 0x2400);
    }

    #[test]
    fn test_jmp() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0, 0xCD);
        memory.write(1, 0xAB);
        let mut cpu = Cpu::new(memory);
        let wait_cycles = cpu.jmp();
        assert_eq!(wait_cycles, 9);
        assert_eq!(cpu.pc, 0xABCD);
    }

    #[test]
    fn test_rst_0() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b00000000);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_rst_1() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11001111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 8);
    }

    #[test]
    fn test_rst_2() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11010111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 16);
    }

    #[test]
    fn test_rst_3() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11011111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 24);
    }

    #[test]
    fn test_rst_4() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11100111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 32);
    }

    #[test]
    fn test_rst_5() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11101111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 40);
    }

    #[test]
    fn test_rst_6() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11110111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 48);
    }

    #[test]
    fn test_rst_7() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.pc = 0x0102;
        let wait_cycles = cpu.rst(0b11111111);
        assert_eq!(wait_cycles, 10);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.sp, 0x23FE);
        assert_eq!(cpu.pc, 56);
    }

    #[test]
    fn test_xthl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        memory.write(0x23FE, 0x0A);
        memory.write(0x23FF, 0x0B);
        let mut cpu = Cpu::new(memory);
        cpu.sp = cpu.sp - 2;
        cpu.h = 0x01;
        cpu.l = 0x02;
        let wait_cycles = cpu.xthl();
        assert_eq!(wait_cycles, 17);
        assert_eq!(cpu.memory.read(0x23FE), 0x02);
        assert_eq!(cpu.memory.read(0x23FF), 0x01);
        assert_eq!(cpu.h, 0x0B);
        assert_eq!(cpu.l, 0x0A);
    }

    #[test]
    fn test_pchl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.h = 0x01;
        cpu.l = 0x02;
        let wait_cycles = cpu.pchl();
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.pc, 0x0102);
    }

    #[test]
    fn test_sphl() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.h = 0x01;
        cpu.l = 0x02;
        let wait_cycles = cpu.sphl();
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.sp, 0x0102);
    }

    #[test]
    fn test_xchg() {
        let mut memory = Box::new(crate::memory::basic_memory::BasicMemory::new());
        let mut cpu = Cpu::new(memory);
        cpu.d = 0x01;
        cpu.e = 0x02;
        cpu.h = 0x0A;
        cpu.l = 0x0B;
        let wait_cycles = cpu.xchg();
        assert_eq!(wait_cycles, 4);
        assert_eq!(cpu.d, 0x0A);
        assert_eq!(cpu.e, 0x0B);
        assert_eq!(cpu.h, 0x01);
        assert_eq!(cpu.l, 0x02);
    }

    #[test]
    fn test_receive_interrupt() {
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
        let high = 0x0A;
        let low = 0x0B;
        let val = concat_u8(high, low);
        assert_eq!(val, 0x0A0B);
    }

    #[test]
    fn test_split_u16() {
        let val = 0x0A0B;
        let (high, low) = split_u16(val);
        assert_eq!(high, 0x0A);
        assert_eq!(low, 0x0B);
    }
}