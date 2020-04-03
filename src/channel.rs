#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Channel(u8);

impl Channel {
    pub const MIN: Channel = Channel(0);

    pub const MAX: Channel = Channel(15);

    pub const COUNT: u8 = 16;

    pub fn new(number: u8) -> Channel {
        assert!(number < Channel::COUNT);
        Channel(number)
    }

    pub const unsafe fn new_unchecked(number: u8) -> Channel {
        Channel(number)
    }
}

impl From<Channel> for u8 {
    fn from(value: Channel) -> Self {
        value.0
    }
}

impl From<Channel> for usize {
    fn from(value: Channel) -> Self {
        value.0 as usize
    }
}

pub fn ch(number: u8) -> Channel {
    Channel::new(number)
}
