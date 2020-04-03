use crate::U7;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct U14(pub(crate) u16);

impl U14 {
    pub const MIN: U14 = U14(0);

    pub const MAX: U14 = U14(16383);

    pub const COUNT: u16 = 16384;

    pub fn new(number: u16) -> U14 {
        assert!(number < U14::COUNT);
        U14(number)
    }

    pub const unsafe fn new_unchecked(number: u16) -> U14 {
        U14(number)
    }
}

impl From<U14> for u16 {
    fn from(value: U14) -> Self {
        value.0
    }
}

impl From<U7> for U14 {
    fn from(value: U7) -> Self {
        U14(value.into())
    }
}

impl From<u8> for U14 {
    fn from(value: u8) -> Self {
        U14(value as u16)
    }
}

impl From<U14> for usize {
    fn from(value: U14) -> Self {
        value.0 as usize
    }
}

pub fn u14(number: u16) -> U14 {
    U14::new(number)
}
