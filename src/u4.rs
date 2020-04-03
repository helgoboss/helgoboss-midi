#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct U4(pub(crate) u8);

impl U4 {
    pub const MIN: U4 = U4(0);

    pub const MAX: U4 = U4(15);

    pub const COUNT: u8 = 16;

    pub fn new(number: u8) -> U4 {
        assert!(number < U4::COUNT);
        U4(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> U4 {
        U4(number)
    }
}

impl From<U4> for u8 {
    fn from(value: U4) -> Self {
        value.0
    }
}

impl From<U4> for u16 {
    fn from(value: U4) -> Self {
        value.0 as u16
    }
}

impl From<U4> for usize {
    fn from(value: U4) -> Self {
        value.0 as usize
    }
}

pub fn u4(number: u8) -> U4 {
    U4::new(number)
}
