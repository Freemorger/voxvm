use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

use crate::vm::RegTypes;

#[derive(Debug, Clone, Copy)]
pub enum Register {
    uint(u64),
    int(i64),
    float(f64),
    StrAddr(u64),
    address(u64),
    ds_addr(u64),
}

impl PartialEq for Register {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => a == b,
            (Register::int(a), Register::int(b)) => a == b,
            (Register::float(a), Register::float(b)) => a == b,
            (Register::StrAddr(a), Register::StrAddr(b)) => a == b,
            (Register::address(a), Register::address(b)) => a == b,
            (Register::ds_addr(a), Register::ds_addr(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => a.partial_cmp(b),
            (Register::int(a), Register::int(b)) => a.partial_cmp(b),
            (Register::float(a), Register::float(b)) => a.partial_cmp(b),
            (Register::StrAddr(a), Register::StrAddr(b)) => a.partial_cmp(b),
            (Register::address(a), Register::address(b)) => a.partial_cmp(b),
            (Register::ds_addr(a), Register::ds_addr(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for Register {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a + b),
            (Register::int(a), Register::int(b)) => Register::int(a + b),
            (Register::float(a), Register::float(b)) => Register::float(a + b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a + b),
            (Register::address(a), Register::address(b)) => Register::address(a + b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a + b),
            _ => panic!(
                "Cannot add different register types: {:?} + {:?}",
                self, other
            ),
        }
    }
}

impl AddAssign for Register {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Register {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a - b),
            (Register::int(a), Register::int(b)) => Register::int(a - b),
            (Register::float(a), Register::float(b)) => Register::float(a - b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a - b),
            (Register::address(a), Register::address(b)) => Register::address(a - b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a - b),
            _ => panic!(
                "Cannot subtract different register types: {:?} - {:?}",
                self, other
            ),
        }
    }
}

impl SubAssign for Register {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul for Register {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a * b),
            (Register::int(a), Register::int(b)) => Register::int(a * b),
            (Register::float(a), Register::float(b)) => Register::float(a * b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a * b),
            (Register::address(a), Register::address(b)) => Register::address(a * b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a * b),
            _ => panic!(
                "Cannot multiply different register types: {:?} * {:?}",
                self, other
            ),
        }
    }
}

impl MulAssign for Register {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Div for Register {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a / b),
            (Register::int(a), Register::int(b)) => Register::int(a / b),
            (Register::float(a), Register::float(b)) => Register::float(a / b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a / b),
            (Register::address(a), Register::address(b)) => Register::address(a / b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a / b),
            _ => panic!(
                "Cannot divide different register types: {:?} / {:?}",
                self, other
            ),
        }
    }
}

impl DivAssign for Register {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl Rem for Register {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a % b),
            (Register::int(a), Register::int(b)) => Register::int(a % b),
            (Register::float(a), Register::float(b)) => Register::float(a % b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a % b),
            (Register::address(a), Register::address(b)) => Register::address(a % b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a % b),
            _ => panic!(
                "Cannot modulo different register types: {:?} % {:?}",
                self, other
            ),
        }
    }
}

impl RemAssign for Register {
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other;
    }
}

impl BitAnd for Register {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a & b),
            (Register::int(a), Register::int(b)) => Register::int(a & b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a & b),
            (Register::address(a), Register::address(b)) => Register::address(a & b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a & b),
            _ => panic!(
                "Bitwise AND not supported for these types: {:?} & {:?}",
                self, other
            ),
        }
    }
}

impl BitAndAssign for Register {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOr for Register {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a | b),
            (Register::int(a), Register::int(b)) => Register::int(a | b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a | b),
            (Register::address(a), Register::address(b)) => Register::address(a | b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a | b),
            _ => panic!(
                "Bitwise OR not supported for these types: {:?} | {:?}",
                self, other
            ),
        }
    }
}

impl BitOrAssign for Register {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXor for Register {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a ^ b),
            (Register::int(a), Register::int(b)) => Register::int(a ^ b),
            (Register::StrAddr(a), Register::StrAddr(b)) => Register::StrAddr(a ^ b),
            (Register::address(a), Register::address(b)) => Register::address(a ^ b),
            (Register::ds_addr(a), Register::ds_addr(b)) => Register::ds_addr(a ^ b),
            _ => panic!(
                "Bitwise XOR not supported for these types: {:?} ^ {:?}",
                self, other
            ),
        }
    }
}

impl BitXorAssign for Register {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Shl for Register {
    type Output = Self;

    fn shl(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a << b),
            (Register::int(a), Register::uint(b)) => Register::int(a << b),
            (Register::StrAddr(a), Register::uint(b)) => Register::StrAddr(a << b),
            (Register::address(a), Register::uint(b)) => Register::address(a << b),
            (Register::ds_addr(a), Register::uint(b)) => Register::ds_addr(a << b),
            _ => panic!(
                "Shift left not supported for these types: {:?} << {:?}",
                self, other
            ),
        }
    }
}

impl ShlAssign for Register {
    fn shl_assign(&mut self, other: Self) {
        *self = *self << other;
    }
}

impl Shr for Register {
    type Output = Self;

    fn shr(self, other: Self) -> Self {
        match (self, other) {
            (Register::uint(a), Register::uint(b)) => Register::uint(a >> b),
            (Register::int(a), Register::uint(b)) => Register::int(a >> b),
            (Register::StrAddr(a), Register::uint(b)) => Register::StrAddr(a >> b),
            (Register::address(a), Register::uint(b)) => Register::address(a >> b),
            (Register::ds_addr(a), Register::uint(b)) => Register::ds_addr(a >> b),
            _ => panic!(
                "Shift right not supported for these types: {:?} >> {:?}",
                self, other
            ),
        }
    }
}

impl ShrAssign for Register {
    fn shr_assign(&mut self, other: Self) {
        *self = *self >> other;
    }
}

impl Not for Register {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Register::uint(a) => Register::uint(!a),
            Register::int(a) => Register::int(!a),
            Register::StrAddr(a) => Register::StrAddr(!a),
            Register::address(a) => Register::address(!a),
            Register::ds_addr(a) => Register::ds_addr(!a),
            Register::float(_) => panic!("Bitwise NOT not supported for float"),
        }
    }
}

impl Neg for Register {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Register::uint(a) => Register::uint((a as i64).wrapping_neg() as u64),
            Register::int(a) => Register::int(-a),
            Register::float(a) => Register::float(-a),
            Register::StrAddr(a) => Register::StrAddr((a as i64).wrapping_neg() as u64),
            Register::address(a) => Register::address((a as i64).wrapping_neg() as u64),
            Register::ds_addr(a) => Register::ds_addr((a as i64).wrapping_neg() as u64),
        }
    }
}

impl Register {
    pub fn from_u64_bits(val: u64, to_type: RegTypes) -> Register {
        match to_type {
            RegTypes::uint64 => Register::uint(val),
            RegTypes::int64 => Register::int(val as i64),
            RegTypes::float64 => Register::float(f64::from_bits(val)),
            RegTypes::StrAddr => Register::StrAddr(val),
            RegTypes::address => Register::address(val),
            RegTypes::ds_addr => Register::ds_addr(val),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Register::uint(val) => *val,
            Register::int(val) => *val as u64,
            Register::float(val) => *val as u64,
            Register::StrAddr(val) => *val,
            Register::address(val) => *val,
            Register::ds_addr(val) => *val,
        }
    }

    pub fn as_u64_bitwise(&self) -> u64 {
        match self {
            Register::uint(val) => *val,
            Register::int(val) => *val as u64,
            Register::float(val) => val.to_bits(),
            Register::StrAddr(val) => *val,
            Register::address(val) => *val,
            Register::ds_addr(val) => *val,
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Register::uint(val) => *val as i64,
            Register::int(val) => *val,
            Register::float(val) => *val as i64,
            Register::StrAddr(val) => *val as i64,
            Register::address(val) => *val as i64,
            Register::ds_addr(val) => *val as i64,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Register::uint(val) => *val as f64,
            Register::int(val) => *val as f64,
            Register::float(val) => *val,
            Register::StrAddr(val) => *val as f64,
            Register::address(val) => *val as f64,
            Register::ds_addr(val) => *val as f64,
        }
    }

    pub fn logical_not(self) -> Register {
        match self {
            Register::uint(val) => Register::uint(if val == 0 { 1 } else { 0 }),
            Register::int(val) => Register::int(if val == 0 { 1 } else { 0 }),
            Register::float(val) => Register::float(if val == 0.0 { 1.0 } else { 0.0 }),
            Register::StrAddr(val) => Register::StrAddr(if val == 0 { 1 } else { 0 }),
            Register::address(val) => Register::address(if val == 0 { 1 } else { 0 }),
            Register::ds_addr(val) => Register::ds_addr(if val == 0 { 1 } else { 0 }),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::uint(val) => write!(f, "{}", val),
            Register::int(val) => write!(f, "{}", val),
            Register::float(val) => write!(f, "{}", val),
            Register::StrAddr(val) => write!(f, "StrAddr({:#x})", val),
            Register::address(val) => write!(f, "VM Heap addr ({:#x})", val),
            Register::ds_addr(val) => write!(f, "VM Data segment addr ({:#x})", val),
        }
    }
}
