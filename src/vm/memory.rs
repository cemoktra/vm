use std::io::Read;

pub trait MemoryTrait {
    type ValueType;

    fn read<I>(&mut self, address: Self::ValueType, input: &mut I) -> Self::ValueType
    where
        I: Read;
    fn write(&mut self, address: Self::ValueType, value: Self::ValueType);

    fn max(&self) -> Self::ValueType;
}
