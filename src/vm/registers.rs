pub trait RegistersTrait {
    type RegisterSet;
    type ValueType;

    fn get(&self, register: Self::RegisterSet) -> Self::ValueType;
    fn set(&mut self, register: Self::RegisterSet, value: Self::ValueType);
    fn next_instruction(&mut self) -> Self::ValueType;
    fn update_flags(&mut self, register: Self::RegisterSet);
    fn get_pc(&self) -> Self::ValueType;
    fn set_pc(&mut self, value: Self::ValueType);
}
