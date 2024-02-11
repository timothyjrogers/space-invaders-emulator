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
        let mut bits: u8 = 0b00000010;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_conditions() {
        let conditions = Conditions::new();
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_set_carry() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Carry, true);
        assert_eq!(conditions.carry, true);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_set_parity() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Parity, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, true);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_set_aux() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Auxillary, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, true);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_set_zero() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Zero, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, true);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_set_sign() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Sign, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, true);
    }

    #[test]
    fn test_reset_carry() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Carry, true);
        assert_eq!(conditions.carry, true);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
        conditions.set(ConditionName::Carry, false);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_reset_parity() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Parity, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, true);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
        conditions.set(ConditionName::Parity, false);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_reset_aux() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Auxillary, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, true);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
        conditions.set(ConditionName::Auxillary, false);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_reset_zero() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Zero, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, true);
        assert_eq!(conditions.sign, false);
        conditions.set(ConditionName::Zero, false);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_reset_sign() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Sign, true);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, true);
        conditions.set(ConditionName::Sign, false);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, false);
        assert_eq!(conditions.sign, false);
    }

    #[test]
    fn test_get_carry() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Carry, true);
        assert_eq!(conditions.get(ConditionName::Carry), true);
    }

    #[test]
    fn test_get_parity() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Parity, true);
        assert_eq!(conditions.get(ConditionName::Parity), true);
    }

    #[test]
    fn test_get_aux() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Auxillary, true);
        assert_eq!(conditions.get(ConditionName::Auxillary), true);
    }

    #[test]
    fn test_get_zero() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Zero, true);
        assert_eq!(conditions.get(ConditionName::Zero), true);
    }

    #[test]
    fn test_get_sign() {
        let mut conditions = Conditions::new();
        conditions.set(ConditionName::Sign, true);
        assert_eq!(conditions.get(ConditionName::Sign), true);
    }

    #[test]
    fn test_as_bits() {
        let mut conditions = Conditions::new();
        assert_eq!(conditions.as_bits(), 0b000000010);
        conditions.set(ConditionName::Sign, true);
        assert_eq!(conditions.as_bits(), 0b10000010);
    }

    #[test]
    fn test_restore_from_bits() {
        let mut conditions = Conditions::new();
        conditions.restore_from_bits(0b11000000);
        assert_eq!(conditions.carry, false);
        assert_eq!(conditions.parity, false);
        assert_eq!(conditions.aux, false);
        assert_eq!(conditions.zero, true);
        assert_eq!(conditions.sign, true);
    }
}