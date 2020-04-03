use crate::U7;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ProgramNumber(pub(crate) u8);

impl ProgramNumber {
    pub const MIN: ProgramNumber = ProgramNumber(0);

    pub const MAX: ProgramNumber = ProgramNumber(127);

    pub const COUNT: u8 = 128;

    pub fn new(number: u8) -> ProgramNumber {
        assert!(number < ProgramNumber::COUNT);
        ProgramNumber(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> ProgramNumber {
        ProgramNumber(number)
    }
}

impl From<U7> for ProgramNumber {
    fn from(value: U7) -> Self {
        ProgramNumber(value.into())
    }
}

impl From<ProgramNumber> for U7 {
    fn from(value: ProgramNumber) -> Self {
        U7(value.into())
    }
}

impl From<ProgramNumber> for u8 {
    fn from(value: ProgramNumber) -> Self {
        value.0
    }
}

impl From<ProgramNumber> for usize {
    fn from(value: ProgramNumber) -> Self {
        value.0 as usize
    }
}

pub fn program_number(number: u8) -> ProgramNumber {
    ProgramNumber::new(number)
}
