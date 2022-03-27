use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    UnknownRegister(u16),
    UnknownInstruction(u16),
    UnknownTrapRoutine(u16),
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownRegister(register) => write!(f, "'{}' is not a known register", register),
            Error::UnknownInstruction(instruction) => {
                write!(f, "'{:#X}' is not a known instruction", instruction)
            }
            Error::UnknownTrapRoutine(routine) => {
                write!(f, "'{:#X}' is not a known trap routine", routine)
            }
            Error::IoError(io_error) => write!(f, "IO error: {}", io_error),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
