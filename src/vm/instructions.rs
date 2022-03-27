use super::{memory::MemoryTrait, registers::RegistersTrait};
use std::io::{Read, Write};

pub trait InstructionsTrait {
    type ValueType;
    type InstructionSet;
    type RegisterSet;
    type Error;

    fn execute<R, M, I, O>(
        &self,
        registers: &mut R,
        memory: &mut M,
        input: &mut I,
        output: &mut O,
    ) -> Result<(), Self::Error>
    where
        R: RegistersTrait<ValueType = Self::ValueType, RegisterSet = Self::RegisterSet>,
        M: MemoryTrait<ValueType = Self::ValueType>,
        I: Read,
        O: Write;
}
