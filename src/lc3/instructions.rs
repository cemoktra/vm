use std::io::Read;
use std::io::Write;
use std::process;

use crate::vm::{instructions::InstructionsTrait, memory::MemoryTrait, registers::RegistersTrait};

use super::{error::Error, registers::RegistersEnum};

#[derive(Debug)]
pub enum Instructions {
    Add {
        destination: RegistersEnum,
        source1: RegistersEnum,
        source2: RegisterMode,
    },
    And {
        destination: RegistersEnum,
        source1: RegistersEnum,
        source2: RegisterMode,
    },
    Branch {
        pc_offset: u16,
        condition_flag: u16,
    },
    Not {
        destination: RegistersEnum,
        source1: RegistersEnum,
    },
    Jump {
        source: RegistersEnum,
    },
    JumpRegister(JumpType),
    Load {
        destination: RegistersEnum,
        pc_offset: u16,
    },
    LoadIndirect {
        destination: RegistersEnum,
        pc_offset: u16,
    },
    LoadRegister {
        destination: RegistersEnum,
        source1: RegistersEnum,
        offset: u16,
    },
    LoadEffectiveAddress {
        destination: RegistersEnum,
        pc_offset: u16,
    },
    Store {
        source: RegistersEnum,
        pc_offset: u16,
    },
    StoreIndirect {
        source: RegistersEnum,
        pc_offset: u16,
    },
    StoreRegister {
        source1: RegistersEnum,
        source2: RegistersEnum,
        offset: u16,
    },
    Trap(TrapRoutine),
    RES,
    RTI,
}

#[derive(Debug)]
pub enum RegisterMode {
    Immediate(u16),
    Register(RegistersEnum),
}

#[derive(Debug)]
pub enum JumpType {
    Long(u16),
    Register(RegistersEnum),
}

#[derive(Debug)]
pub enum TrapRoutine {
    GETC = 0x20,
    OUT,
    PUTS,
    IN,
    PUTSP,
    HALT,
}

impl TryFrom<u16> for TrapRoutine {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(TrapRoutine::GETC),
            0x21 => Ok(TrapRoutine::OUT),
            0x22 => Ok(TrapRoutine::PUTS),
            0x23 => Ok(TrapRoutine::IN),
            0x24 => Ok(TrapRoutine::PUTSP),
            0x25 => Ok(TrapRoutine::HALT),
            _ => Err(Error::UnknownTrapRoutine(value)),
        }
    }
}

fn sign_extend(mut x: u16, bit_count: u8) -> u16 {
    if (x >> (bit_count - 1)) & 1 != 0 {
        x |= 0xFFFF << bit_count;
    }
    x
}

impl InstructionsTrait for Instructions {
    type ValueType = u16;
    type InstructionSet = Instructions;
    type RegisterSet = RegistersEnum;
    type Error = Error;

    fn read(value: Self::ValueType) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match value >> 12 {
            0 => Ok(Instructions::Branch {
                pc_offset: sign_extend(value & 0x1FF, 9),
                condition_flag: (value >> 9) & 0x7,
            }),
            1 => {
                let r0 = ((value >> 9) & 0x7).try_into()?;
                let r1 = ((value >> 6) & 0x7).try_into()?;
                if (value >> 5) & 0x1 > 0 {
                    Ok(Instructions::Add {
                        destination: r0,
                        source1: r1,
                        source2: RegisterMode::Immediate(sign_extend(value & 0x1F, 5)),
                    })
                } else {
                    Ok(Instructions::Add {
                        destination: r0,
                        source1: r1,
                        source2: RegisterMode::Register((value & 0x7).try_into()?),
                    })
                }
            }
            2 => Ok(Instructions::Load {
                destination: ((value >> 9) & 0x7).try_into()?,
                pc_offset: sign_extend(value & 0x1FF, 9),
            }),
            3 => Ok(Instructions::Store {
                source: ((value >> 9) & 0x7).try_into()?,
                pc_offset: sign_extend(value & 0x1FF, 9),
            }),
            4 => {
                if (value >> 11) & 1 != 0 {
                    Ok(Instructions::JumpRegister(JumpType::Long(sign_extend(
                        value & 0x7FF,
                        11,
                    ))))
                } else {
                    Ok(Instructions::JumpRegister(JumpType::Register(
                        ((value >> 6) & 0x7).try_into()?,
                    )))
                }
            }
            5 => {
                let r0 = ((value >> 9) & 0x7).try_into()?;
                let r1 = ((value >> 6) & 0x7).try_into()?;
                if (value >> 5) & 0x1 > 0 {
                    Ok(Instructions::And {
                        destination: r0,
                        source1: r1,
                        source2: RegisterMode::Immediate(sign_extend(value & 0x1F, 5)),
                    })
                } else {
                    Ok(Instructions::And {
                        destination: r0,
                        source1: r1,
                        source2: RegisterMode::Register((value & 0x7).try_into()?),
                    })
                }
            }
            6 => Ok(Instructions::LoadRegister {
                destination: ((value >> 9) & 0x7).try_into()?,
                source1: ((value >> 6) & 0x7).try_into()?,
                offset: sign_extend(value & 0x3F, 6),
            }),
            7 => Ok(Instructions::StoreRegister {
                source1: ((value >> 9) & 0x7).try_into()?,
                source2: ((value >> 6) & 0x7).try_into()?,
                offset: sign_extend(value & 0x3F, 6),
            }),
            8 => Ok(Instructions::RTI),
            9 => Ok(Instructions::Not {
                destination: ((value >> 9) & 0x7).try_into()?,
                source1: ((value >> 6) & 0x7).try_into()?,
            }),
            10 => Ok(Instructions::LoadIndirect {
                destination: ((value >> 9) & 0x7).try_into()?,
                pc_offset: sign_extend(value & 0x1FF, 9),
            }),
            11 => Ok(Instructions::StoreIndirect {
                source: ((value >> 9) & 0x7).try_into()?,
                pc_offset: sign_extend(value & 0x1FF, 9),
            }),
            12 => Ok(Instructions::Jump {
                source: ((value >> 6) & 0x7).try_into()?,
            }),
            13 => Ok(Instructions::RES),
            14 => Ok(Instructions::LoadEffectiveAddress {
                destination: ((value >> 9) & 0x7).try_into()?,
                pc_offset: sign_extend(value & 0x1FF, 9),
            }),
            15 => Ok(Instructions::Trap((value & 0xFF).try_into()?)),
            _ => Err(Error::UnknownInstruction(value)),
        }
    }

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
        O: Write,
    {
        match self {
            Instructions::Add {
                destination,
                source1,
                source2,
            } => {
                match source2 {
                    RegisterMode::Immediate(source2) => {
                        let (result, _) = registers.get(*source1).overflowing_add(*source2);
                        registers.set(*destination, result)
                    }
                    RegisterMode::Register(source2) => {
                        let (result, _) = registers
                            .get(*source1)
                            .overflowing_add(registers.get(*source2));
                        registers.set(*destination, result)
                    }
                }
                registers.update_flags(*destination);
            }
            Instructions::LoadIndirect {
                destination,
                pc_offset,
            } => {
                let address = memory.read(
                    registers.get(RegistersEnum::ProgramCounter) + pc_offset,
                    input,
                );
                registers.set(*destination, memory.read(address, input));
                registers.update_flags(*destination);
            }
            Instructions::RES => unreachable!(),
            Instructions::RTI => unreachable!(),
            Instructions::And {
                destination,
                source1,
                source2,
            } => {
                match source2 {
                    RegisterMode::Immediate(source2) => {
                        registers.set(*destination, registers.get(*source1) & source2)
                    }
                    RegisterMode::Register(source2) => registers.set(
                        *destination,
                        registers.get(*source1) & registers.get(*source2),
                    ),
                }
                registers.update_flags(*destination);
            }
            Instructions::Not {
                destination,
                source1,
            } => {
                registers.set(*destination, !registers.get(*source1));
                registers.update_flags(*destination);
            }
            Instructions::Branch {
                pc_offset,
                condition_flag,
            } => {
                if condition_flag & registers.get(RegistersEnum::Condition) > 0 {
                    let (result, _) = registers
                        .get(RegistersEnum::ProgramCounter)
                        .overflowing_add(*pc_offset);
                    registers.set(RegistersEnum::ProgramCounter, result);
                }
            }
            Instructions::Jump { source } => {
                registers.set(RegistersEnum::ProgramCounter, registers.get(*source));
            }
            Instructions::JumpRegister(jump_type) => {
                registers.set(
                    RegistersEnum::R7,
                    registers.get(RegistersEnum::ProgramCounter),
                );
                match jump_type {
                    JumpType::Long(pc_offset) => {
                        let (result, _) = registers
                            .get(RegistersEnum::ProgramCounter)
                            .overflowing_add(*pc_offset);
                        registers.set(RegistersEnum::ProgramCounter, result)
                    }
                    JumpType::Register(register) => {
                        registers.set(RegistersEnum::ProgramCounter, registers.get(*register))
                    }
                }
            }
            Instructions::Load {
                destination,
                pc_offset,
            } => {
                let (address, _) = registers
                    .get(RegistersEnum::ProgramCounter)
                    .overflowing_add(*pc_offset);
                registers.set(*destination, memory.read(address, input));
                registers.update_flags(*destination);
            }
            Instructions::LoadRegister {
                destination,
                source1,
                offset,
            } => {
                let (address, _) = registers.get(*source1).overflowing_add(*offset);
                registers.set(*destination, memory.read(address, input));
                registers.update_flags(*destination);
            }
            Instructions::LoadEffectiveAddress {
                destination,
                pc_offset,
            } => {
                let (result, _) = registers
                    .get(RegistersEnum::ProgramCounter)
                    .overflowing_add(*pc_offset);
                registers.set(*destination, result);
                registers.update_flags(*destination);
            }
            Instructions::Store { source, pc_offset } => {
                let (address, _) = registers
                    .get(RegistersEnum::ProgramCounter)
                    .overflowing_add(*pc_offset);
                memory.write(address, registers.get(*source));
            }
            Instructions::StoreIndirect { source, pc_offset } => {
                let (address, _) = registers
                    .get(RegistersEnum::ProgramCounter)
                    .overflowing_add(*pc_offset);
                let address = memory.read(address, input);
                memory.write(address, registers.get(*source));
            }
            Instructions::StoreRegister {
                source1,
                source2,
                offset,
            } => {
                let (address, _) = registers.get(*source2).overflowing_add(*offset);
                memory.write(address, registers.get(*source1));
            }
            Instructions::Trap(routine) => match routine {
                TrapRoutine::GETC => {
                    let mut buffer = [0; 1];
                    input.read_exact(&mut buffer)?;
                    registers.set(RegistersEnum::R0, buffer[0] as u16);
                }
                TrapRoutine::OUT => {
                    let character = registers.get(RegistersEnum::R0) as u8 as char;
                    write!(output, "{character}")?;
                }
                TrapRoutine::PUTS => {
                    let mut address = registers.get(RegistersEnum::R0);
                    let mut byte = memory.read(address, input);
                    while byte != 0x0000 {
                        let character = byte as u8 as char;
                        write!(output, "{character}")?;
                        address += 1;
                        byte = memory.read(address, input);
                    }
                    output.flush()?;
                }
                TrapRoutine::IN => {
                    output.flush()?;
                    let character = input
                        .bytes()
                        .next()
                        .and_then(|result| result.ok())
                        .map(|byte| byte as u16)
                        .unwrap();
                    registers.set(RegistersEnum::R0, character);
                }
                TrapRoutine::PUTSP => {
                    let mut address = registers.get(RegistersEnum::R0);
                    let mut byte = memory.read(address, input);
                    while byte != 0x0000 {
                        let character = (byte & 0xFF) as u8 as char;
                        write!(output, "{character}")?;
                        let character = (byte >> 8) as u8 as char;
                        write!(output, "{character}")?;
                        address += 1;
                        byte = memory.read(address, input);
                    }
                    output.flush()?;
                }
                TrapRoutine::HALT => {
                    output.flush()?;
                    process::exit(1)
                }
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{
        lc3::{
            instructions::{JumpType, TrapRoutine},
            memory::Memory,
            registers::{Registers, RegistersEnum, PROGRAM_START},
        },
        vm::{instructions::InstructionsTrait, memory::MemoryTrait, registers::RegistersTrait},
    };

    use super::{Instructions, RegisterMode};

    #[test]
    fn test_add_immediate() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        registers.set(RegistersEnum::R1, 5);
        let instruction = Instructions::Add {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
            source2: RegisterMode::Immediate(10),
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(15, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_add_register() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        registers.set(RegistersEnum::R1, 5);
        registers.set(RegistersEnum::R2, 7);
        let instruction = Instructions::Add {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
            source2: RegisterMode::Register(RegistersEnum::R2),
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(12, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_and_immediate() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        registers.set(RegistersEnum::R1, 5);
        let instruction = Instructions::And {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
            source2: RegisterMode::Immediate(10),
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(0, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_and_register() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        registers.set(RegistersEnum::R1, 5);
        registers.set(RegistersEnum::R2, 7);
        let instruction = Instructions::And {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
            source2: RegisterMode::Register(RegistersEnum::R2),
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(5, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_not() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        registers.set(RegistersEnum::R1, 0xFF00);
        let instruction = Instructions::Not {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(0x00FF, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_load_indirect() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        let address = 1;
        let value = 23;
        memory.write(PROGRAM_START + pc_offset, address);
        memory.write(address, value);

        let instruction = Instructions::LoadIndirect {
            destination: RegistersEnum::R0,
            pc_offset: 5,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(value, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_branch() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 10;

        registers.set(RegistersEnum::Condition, 0);
        let instruction = Instructions::Branch {
            pc_offset,
            condition_flag: 1,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(PROGRAM_START, registers.get(RegistersEnum::ProgramCounter));

        registers.set(RegistersEnum::Condition, 1);
        let instruction = Instructions::Branch {
            pc_offset,
            condition_flag: 1,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(
            PROGRAM_START + pc_offset,
            registers.get(RegistersEnum::ProgramCounter)
        );
    }

    #[test]
    fn test_jump() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        registers.set(RegistersEnum::R0, PROGRAM_START + pc_offset);

        let instruction = Instructions::Jump {
            source: RegistersEnum::R0,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(
            PROGRAM_START + pc_offset,
            registers.get(RegistersEnum::ProgramCounter)
        );
    }

    #[test]
    fn test_jump_register_long() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;

        let instruction = Instructions::JumpRegister(JumpType::Long(pc_offset));
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(
            PROGRAM_START + pc_offset,
            registers.get(RegistersEnum::ProgramCounter)
        );
        assert_eq!(PROGRAM_START, registers.get(RegistersEnum::R7));
    }

    #[test]
    fn test_jump_register_register() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        registers.set(RegistersEnum::R0, PROGRAM_START + pc_offset);

        let instruction = Instructions::JumpRegister(JumpType::Register(RegistersEnum::R0));
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(
            PROGRAM_START + pc_offset,
            registers.get(RegistersEnum::ProgramCounter)
        );
        assert_eq!(PROGRAM_START, registers.get(RegistersEnum::R7));
    }

    #[test]
    fn test_load() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        let value = 10;
        memory.write(PROGRAM_START + pc_offset, value);

        let instruction = Instructions::Load {
            destination: RegistersEnum::R0,
            pc_offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(value, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_load_register() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let offset = 5;
        let address = 25;
        let value = 10;
        memory.write(address + offset, value);
        registers.set(RegistersEnum::R1, address);

        let instruction = Instructions::LoadRegister {
            destination: RegistersEnum::R0,
            source1: RegistersEnum::R1,
            offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(value, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_load_effective_address() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;

        let instruction = Instructions::LoadEffectiveAddress {
            destination: RegistersEnum::R0,
            pc_offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(PROGRAM_START + pc_offset, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_store() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        let value = 15;

        registers.set(RegistersEnum::R0, value);
        let instruction = Instructions::Store {
            source: RegistersEnum::R0,
            pc_offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(
            value,
            memory.read(PROGRAM_START + pc_offset, &mut std::io::stdin())
        );
    }

    #[test]
    fn test_store_indirect() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let pc_offset = 5;
        let value = 15;
        let address = 25;

        memory.write(PROGRAM_START + pc_offset, address);
        registers.set(RegistersEnum::R0, value);
        let instruction = Instructions::StoreIndirect {
            source: RegistersEnum::R0,
            pc_offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(value, memory.read(address, &mut std::io::stdin()));
    }

    #[test]
    fn test_store_register() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();

        let offset = 5;
        let value = 15;
        let address = 25;

        registers.set(RegistersEnum::R0, value);
        registers.set(RegistersEnum::R1, address);
        let instruction = Instructions::StoreRegister {
            source1: RegistersEnum::R0,
            source2: RegistersEnum::R1,
            offset,
        };
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(value, memory.read(address + offset, &mut std::io::stdin()));
    }

    #[test]
    fn test_trap_getc() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();
        let character = 'A' as u16;
        let mut input = Cursor::new(vec![character as u8]);

        let instruction = Instructions::Trap(TrapRoutine::GETC);
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut input,
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(character, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_trap_out() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();
        let mut output = Cursor::new(vec![0; 15]);

        let character = 'A' as u16;
        registers.set(RegistersEnum::R0, character);

        let instruction = Instructions::Trap(TrapRoutine::OUT);
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut output,
            )
            .unwrap();
        assert_eq!(character, output.get_ref()[0] as u16);
    }

    #[test]
    fn test_trap_puts() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();
        let mut output = Cursor::new(vec![0; 15]);

        let address = 20;
        let s = "Hello";
        registers.set(RegistersEnum::R0, address);
        s.char_indices().for_each(|(index, character)| {
            memory.write(address + index as u16, character as u16);
        });

        let instruction = Instructions::Trap(TrapRoutine::PUTS);
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut output,
            )
            .unwrap();

        assert_eq!(
            String::from_utf8(output.get_ref()[0..s.len()].to_vec()).unwrap(),
            s
        );
    }

    #[test]
    fn test_trap_in() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();
        let character = 'A' as u16;
        let mut input = Cursor::new(vec![character as u8]);

        let instruction = Instructions::Trap(TrapRoutine::IN);
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut input,
                &mut std::io::stdout(),
            )
            .unwrap();
        assert_eq!(character, registers.get(RegistersEnum::R0));
    }

    #[test]
    fn test_trap_putsp() {
        let mut registers = Registers::default();
        let mut memory = Memory::default();
        let mut output = Cursor::new(vec![0; 15]);

        let address = 20;
        let byte = 'V' as u16 | ('M' as u16) << 8;

        registers.set(RegistersEnum::R0, address);
        memory.write(address, byte);

        let instruction = Instructions::Trap(TrapRoutine::PUTSP);
        instruction
            .execute(
                &mut registers,
                &mut memory,
                &mut std::io::stdin(),
                &mut output,
            )
            .unwrap();

        assert_eq!(output.get_ref()[0] as char, 'V');
        assert_eq!(output.get_ref()[1] as char, 'M');
    }
}
