use crate::{MidiMessage, MidiMessageFactory, U7};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RawMidiMessage(u8, U7, U7);

impl MidiMessageFactory for RawMidiMessage {
    unsafe fn from_bytes_unchecked(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Self {
        Self(status_byte, data_byte_1, data_byte_2)
    }
}

impl MidiMessage for RawMidiMessage {
    fn status_byte(&self) -> u8 {
        self.0
    }

    fn data_byte_1(&self) -> U7 {
        self.1
    }

    fn data_byte_2(&self) -> U7 {
        self.2
    }
}
