use std::fmt;
use crate::conditions;

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
}

impl Cpu {
    pub fn new() -> Self {
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
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\ta: {}\n\tb: {}\n\tc: {}\n\td: {}\n\te: {}\n\th: {}\n\tl: {}\n\tconditions: {}\n\tpc: {}\n\tsp: {}\n", self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.conditions, self.pc, self.sp)
    }
}