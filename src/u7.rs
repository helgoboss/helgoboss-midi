#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct U7(pub(crate) u8);

impl U7 {
    pub const MIN: U7 = U7(0);

    pub const MAX: U7 = U7(127);

    pub const COUNT: u8 = 128;

    pub fn new(number: u8) -> U7 {
        assert!(number < U7::COUNT);
        U7(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> U7 {
        U7(number)
    }
}

impl From<U7> for u8 {
    fn from(value: U7) -> Self {
        value.0
    }
}

impl From<U7> for u16 {
    fn from(value: U7) -> Self {
        value.0 as u16
    }
}

impl From<U7> for usize {
    fn from(value: U7) -> Self {
        value.0 as usize
    }
}

pub fn u7(number: u8) -> U7 {
    U7::new(number)
}
