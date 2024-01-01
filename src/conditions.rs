use std::fmt;

pub enum ConditionName {
    Carry,
    Auxillary,
    Sign,
    Zero,
    Parity,
}

pub struct Conditions {
    carry: bool,
    aux: bool,
    sign: bool,
    zero: bool,
    parity: bool,
}

impl Conditions {
    pub fn new() -> Self {
        Conditions{
            carry: false,
            aux: false,
            sign: false,
            zero: false,
            parity: false,
        }
    }

    pub fn set(&mut self, register: ConditionName) {
        match register {
            ConditionName::Carry => self.carry = true,
            ConditionName::Auxillary => self.aux = true,
            ConditionName::Sign => self.sign = true,
            ConditionName::Zero => self.zero = true,
            ConditionName::Parity => self.parity = true,
        }
    }

    pub fn get(&self, register: ConditionName) -> bool {
        match register {
            ConditionName::Carry => self.carry,
            ConditionName::Auxillary => self.aux,
            ConditionName::Sign => self.sign,
            ConditionName::Zero => self.zero,
            ConditionName::Parity => self.parity,
        }
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "carry: {}, aux: {}, sign: {}, zero: {}, parity: {}", self.carry, self.aux, self.sign, self.zero, self.parity)
    }
}