use crate::{MidiMessage, MidiMessageFactory, U7};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawMidiMessage {
    status_byte: u8,
    data_byte_1: U7,
    data_byte_2: U7,
}

impl MidiMessageFactory for RawMidiMessage {
    unsafe fn from_bytes_unchecked(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Self {
        Self {
            status_byte,
            data_byte_1,
            data_byte_2,
        }
    }
}

impl MidiMessage for RawMidiMessage {
    fn get_status_byte(&self) -> u8 {
        self.status_byte
    }

    fn get_data_byte_1(&self) -> U7 {
        self.data_byte_1
    }

    fn get_data_byte_2(&self) -> U7 {
        self.data_byte_2
    }
}
