use std::hint::unreachable_unchecked;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpFlag {
    Never = 0b000,
    IfZero = 0b001,
    IfNegative = 0b010,
    IfNotPositive = 0b011,
    IfPositive = 0b100,
    IfNotNegative = 0b101,
    IfNotZero = 0b110,
    Always = 0b111,
}

#[allow(dead_code)]
impl JumpFlag {
    fn from_u8(x: u8) -> Self {
        match x {
            0b000 => JumpFlag::Never,
            0b001 => JumpFlag::IfZero,
            0b010 => JumpFlag::IfNegative,
            0b011 => JumpFlag::IfNotPositive,
            0b100 => JumpFlag::IfPositive,
            0b101 => JumpFlag::IfNotZero,
            0b110 => JumpFlag::IfNotNegative,
            0b111 => JumpFlag::Always,
            _ => panic!("invalid jump flag: {}", x),
        }
    }
    
    unsafe fn from_u8_unchecked(x: u8) -> Self {
        match x {
            0b000 => JumpFlag::Never,
            0b001 => JumpFlag::IfZero,
            0b010 => JumpFlag::IfNegative,
            0b011 => JumpFlag::IfNotPositive,
            0b100 => JumpFlag::IfPositive,
            0b101 => JumpFlag::IfNotZero,
            0b110 => JumpFlag::IfNotNegative,
            0b111 => JumpFlag::Always,
            _ => unreachable_unchecked(),
        }
    }
}

impl std::ops::Not for JumpFlag {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::from_u8(!(self as u8) & 0b111)
    }
}

impl std::ops::BitAnd for JumpFlag {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_u8(self as u8 & rhs as u8)
    }
}

impl std::ops::BitAndAssign for JumpFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::BitOr for JumpFlag {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_u8(self as u8 | rhs as u8)
    }
}

impl std::ops::BitOrAssign for JumpFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
