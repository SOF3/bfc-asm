use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Code {
    MemInc,
    MemDec,
    PtrInc,
    PtrDec,
    SysWrite,
    SysRead,
    LoopStart,
    LoopEnd,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MemInc => '+',
                Self::MemDec => '-',
                Self::PtrInc => '>',
                Self::PtrDec => '<',
                Self::SysWrite => '.',
                Self::SysRead => ',',
                Self::LoopStart => '[',
                Self::LoopEnd => ']',
            }
        )
    }
}

impl TryFrom<char> for Code {
    type Error = ();
    fn try_from(from: char) -> Result<Self, ()> {
        let c = match from {
            '+' => Self::MemInc,
            '-' => Self::MemDec,
            '>' => Self::PtrInc,
            '<' => Self::PtrDec,
            '.' => Self::SysWrite,
            ',' => Self::SysRead,
            '[' => Self::LoopStart,
            ']' => Self::LoopEnd,
            _ => return Err(()),
        };
        Ok(c)
    }
}
