use std::io::Read;

use crate::vm::{instructions::InstructionsTrait, memory::MemoryTrait, registers::RegistersTrait};

use super::{
    error::Error,
    instructions::Instructions,
    memory::Memory,
    registers::{Registers, RegistersEnum},
};
// use crate::vm::machine::VirtualMachine;

#[derive(Default)]
pub struct LittleComputer3 {
    memory: Memory,
    registers: Registers,
}

impl LittleComputer3 {
    pub fn load_program(&mut self, mut source: impl Read) -> Result<(), Error> {
        let mut buffer = [0u8; 2];

        source.read_exact(&mut buffer)?;
        let mut address = u16::from_be_bytes(buffer);
        loop {
            match source.read_exact(&mut buffer) {
                Ok(_) => {
                    self.memory.write(address, u16::from_be_bytes(buffer));
                    address += 1;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    } else {
                        return Err(Error::IoError(e));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn execute_program(&mut self, debug: bool) -> Result<(), Error> {
        let mut input = std::io::stdin();
        let mut output = std::io::stdout();

        while self.registers.get(RegistersEnum::ProgramCounter) < u16::MAX {
            let instruction: Instructions = self
                .memory
                .read(
                    self.registers.get(RegistersEnum::ProgramCounter),
                    &mut input,
                )
                .try_into()?;
            self.registers.set(
                RegistersEnum::ProgramCounter,
                self.registers.get(RegistersEnum::ProgramCounter) + 1,
            );
            if debug {
                println!(" => {instruction:?}");
                println!(" => {:?}", self.registers);
            }
            instruction.execute(
                &mut self.registers,
                &mut self.memory,
                &mut input,
                &mut output,
            )?;
        }

        Ok(())
    }
}
