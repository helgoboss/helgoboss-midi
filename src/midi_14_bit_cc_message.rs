use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value,
    FourteenBitValue, MidiMessage, MidiMessageFactory, Nibble, SevenBitValue,
    StructuredMidiMessage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Midi14BitCcMessage {
    channel: Nibble,
    msb_controller_number: SevenBitValue,
    value: FourteenBitValue,
}

impl Midi14BitCcMessage {
    pub fn new(
        channel: Nibble,
        msb_controller_number: SevenBitValue,
        value: FourteenBitValue,
    ) -> Midi14BitCcMessage {
        debug_assert!(channel < 16);
        debug_assert!(msb_controller_number < 32);
        debug_assert!(value < 16384);
        Midi14BitCcMessage {
            channel,
            msb_controller_number,
            value,
        }
    }

    pub fn get_channel(&self) -> Nibble {
        self.channel
    }

    pub fn get_msb_controller_number(&self) -> SevenBitValue {
        self.msb_controller_number
    }

    pub fn get_lsb_controller_number(&self) -> SevenBitValue {
        self.msb_controller_number + 32
    }

    pub fn get_value(&self) -> FourteenBitValue {
        self.value
    }

    pub fn build_midi_messages<T: MidiMessageFactory>(&self) -> [T; 2] {
        [
            T::control_change(
                self.channel,
                self.get_msb_controller_number(),
                extract_high_7_bit_value_from_14_bit_value(self.value),
            ),
            T::control_change(
                self.channel,
                self.get_lsb_controller_number(),
                extract_low_7_bit_value_from_14_bit_value(self.value),
            ),
        ]
    }
}

pub fn could_be_part_of_14_bit_cc_message(msg: &impl MidiMessage) -> bool {
    match msg.to_structured() {
        StructuredMidiMessage::ControlChange(data) if data.controller_number < 64 => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawMidiMessage;

    #[test]
    fn basics() {
        // Given
        let msg = Midi14BitCcMessage::new(5, 2, 1057);
        // When
        // Then
        assert_eq!(msg.get_channel(), 5);
        assert_eq!(msg.get_msb_controller_number(), 2);
        assert_eq!(msg.get_lsb_controller_number(), 34);
        assert_eq!(msg.get_value(), 1057);
        let midi_msgs: [RawMidiMessage; 2] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                RawMidiMessage::control_change(5, 2, 8),
                RawMidiMessage::control_change(5, 34, 33)
            ]
        );
    }

    #[test]
    fn could_be_part_of_14_bit_cc_message_test() {
        // Given
        // When
        // Then
        assert!(could_be_part_of_14_bit_cc_message(
            &RawMidiMessage::control_change(5, 2, 8)
        ));
        assert!(could_be_part_of_14_bit_cc_message(
            &RawMidiMessage::control_change(5, 34, 33)
        ));
        assert!(!could_be_part_of_14_bit_cc_message(
            &RawMidiMessage::control_change(5, 67, 8)
        ));
    }
}
