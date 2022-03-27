use std::io::Read;

use crate::vm::memory::MemoryTrait;

pub struct Memory([u16; u16::MAX as usize]);

pub enum MemoryMappedReg {
    Kbsr = 0xFE00,
    Kbdr = 0xFE02,
}

impl MemoryTrait for Memory {
    type ValueType = u16;

    fn read<I>(&mut self, address: Self::ValueType, input: &mut I) -> Self::ValueType
    where
        I: Read,
    {
        if address == MemoryMappedReg::Kbsr as u16 {
            self.handle_keyboard(input);
        }
        self.0[address as usize]
    }

    fn write(&mut self, address: Self::ValueType, value: Self::ValueType) {
        self.0[address as usize] = value;
    }

    fn max(&self) -> Self::ValueType {
        u16::MAX
    }
}

impl Memory {
    fn handle_keyboard<I>(&mut self, input: &mut I)
    where
        I: Read,
    {
        let mut buffer = [0u8; 2];
        input.read_exact(&mut buffer).unwrap();

        if buffer[0] != 0 {
            self.write(MemoryMappedReg::Kbsr as u16, 1 << 15);
            self.write(MemoryMappedReg::Kbdr as u16, buffer[0] as u16);
        } else {
            self.write(MemoryMappedReg::Kbsr as u16, 0)
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self([0; u16::MAX as usize])
    }
}
