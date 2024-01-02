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

    pub fn set(&mut self, register: ConditionName, value: bool) {
        match register {
            ConditionName::Carry => self.carry = value,
            ConditionName::Auxillary => self.aux = value,
            ConditionName::Sign => self.sign = value,
            ConditionName::Zero => self.zero = value,
            ConditionName::Parity => self.parity = value,
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

    pub fn as_bits(&self) -> u8 {
        let mut bits: u8 = 0b00000000;
        if self.carry {
            bits = bits | 0b00000001;
        }
        if self.parity {
            bits = bits | 0b00000100;
        }
        if self.aux {
            bits = bits | 0b00010000;
        }
        if self.zero {
            bits = bits | 0b01000000;
        }
        if self.sign {
            bits = bits | 0b10000000;
        }
        return bits;
    }

    pub fn restore_from_bits(&mut self, bits: u8) {
        if bits & 0b00000001 == 0b00000001 {
            self.carry = true;
        } else {
            self.carry = false;
        }
        if bits & 0b00000100 == 0b00000100 {
            self.parity = true;
        } else {
            self.parity = false;
        }
        if bits & 0b00010000 == 0b00010000 {
            self.aux = true;
        } else {
            self.aux = false;
        }
        if bits & 0b01000000 == 0b01000000 {
            self.zero = true;
        } else {
            self.zero = false;
        }
        if bits & 0b10000000 == 0b10000000 {
            self.sign = true;
        } else {
            self.sign = false;
        }
    }
}

impl fmt::Display for Conditions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "carry: {}, aux: {}, sign: {}, zero: {}, parity: {}", self.carry, self.aux, self.sign, self.zero, self.parity)
    }
}