use crate::U7;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct KeyNumber(pub(crate) u8);

impl KeyNumber {
    pub const MIN: KeyNumber = KeyNumber(0);

    pub const MAX: KeyNumber = KeyNumber(127);

    pub const COUNT: u8 = 128;

    pub fn new(number: u8) -> KeyNumber {
        assert!(number < KeyNumber::COUNT);
        KeyNumber(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> KeyNumber {
        KeyNumber(number)
    }
}

impl From<U7> for KeyNumber {
    fn from(value: U7) -> Self {
        KeyNumber(value.into())
    }
}

impl From<KeyNumber> for U7 {
    fn from(value: KeyNumber) -> Self {
        U7(value.into())
    }
}

impl From<KeyNumber> for u8 {
    fn from(value: KeyNumber) -> Self {
        value.0
    }
}

impl From<KeyNumber> for usize {
    fn from(value: KeyNumber) -> Self {
        value.0 as usize
    }
}

pub fn key_number(number: u8) -> KeyNumber {
    KeyNumber::new(number)
}
