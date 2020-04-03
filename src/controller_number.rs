use crate::U7;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ControllerNumber(pub(crate) u8);

impl ControllerNumber {
    pub const MIN: ControllerNumber = ControllerNumber(0);

    pub const MAX: ControllerNumber = ControllerNumber(127);

    pub const COUNT: u8 = 128;

    pub fn new(number: u8) -> ControllerNumber {
        assert!(number < ControllerNumber::COUNT);
        ControllerNumber(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> ControllerNumber {
        ControllerNumber(number)
    }
}

impl From<U7> for ControllerNumber {
    fn from(value: U7) -> Self {
        ControllerNumber(value.into())
    }
}

impl From<ControllerNumber> for U7 {
    fn from(value: ControllerNumber) -> Self {
        U7(value.into())
    }
}

impl From<ControllerNumber> for u8 {
    fn from(value: ControllerNumber) -> Self {
        value.0
    }
}

impl From<ControllerNumber> for usize {
    fn from(value: ControllerNumber) -> Self {
        value.0 as usize
    }
}

pub fn controller_number(number: u8) -> ControllerNumber {
    ControllerNumber::new(number)
}
