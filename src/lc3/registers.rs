use crate::vm::registers::RegistersTrait;

use super::error::Error;

pub const PROGRAM_START: u16 = 0x3000;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum RegistersEnum {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    ProgramCounter,
    Condition,
}

impl TryFrom<u16> for RegistersEnum {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegistersEnum::R0),
            1 => Ok(RegistersEnum::R1),
            2 => Ok(RegistersEnum::R2),
            3 => Ok(RegistersEnum::R3),
            4 => Ok(RegistersEnum::R4),
            5 => Ok(RegistersEnum::R5),
            6 => Ok(RegistersEnum::R6),
            7 => Ok(RegistersEnum::R7),
            8 => Ok(RegistersEnum::ProgramCounter),
            9 => Ok(RegistersEnum::Condition),
            _ => Err(Error::UnknownRegister(value)),
        }
    }
}

enum ConditionFlag {
    Positive = 1 << 0,
    Zero = 1 << 1,
    Negative = 1 << 2,
}

#[derive(Debug)]
pub struct Registers([u16; 10]);

impl Default for Registers {
    fn default() -> Self {
        let mut registers = Self([0; 10]);
        registers.set(RegistersEnum::ProgramCounter, PROGRAM_START);
        registers
    }
}

impl RegistersTrait for Registers {
    type RegisterSet = RegistersEnum;
    type ValueType = u16;

    fn get(&self, register: Self::RegisterSet) -> Self::ValueType {
        self.0[register as usize]
    }

    fn set(&mut self, register: Self::RegisterSet, value: Self::ValueType) {
        self.0[register as usize] = value;
    }

    fn next_instruction(&mut self) -> Self::ValueType {
        let address = self.get(RegistersEnum::ProgramCounter);
        self.set(RegistersEnum::ProgramCounter, address + 1);
        address
    }

    fn update_flags(&mut self, register: Self::RegisterSet) {
        let value = self.get(register);
        if value == 0 {
            self.set(RegistersEnum::Condition, ConditionFlag::Zero as u16);
        } else if value >> 15 != 0 {
            self.set(RegistersEnum::Condition, ConditionFlag::Negative as u16);
        } else {
            self.set(RegistersEnum::Condition, ConditionFlag::Positive as u16);
        }
    }

    fn get_pc(&self) -> Self::ValueType {
        self.get(RegistersEnum::ProgramCounter)
    }

    fn set_pc(&mut self, value: Self::ValueType) {
        self.set(RegistersEnum::ProgramCounter, value);
    }
}

#[cfg(test)]
mod test {
    use crate::vm::registers::RegistersTrait;

    #[test]
    fn test_default() {
        let registers = super::Registers::default();
        assert_eq!(
            super::PROGRAM_START,
            registers.get(super::RegistersEnum::ProgramCounter)
        );
    }

    #[test]
    fn test_set_get() {
        let mut registers = super::Registers::default();
        assert_eq!(0, registers.get(super::RegistersEnum::R0));
        registers.set(super::RegistersEnum::R0, 12);
        assert_eq!(12, registers.get(super::RegistersEnum::R0));
    }
}
