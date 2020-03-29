use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value,
    ControlChangeData, FourteenBitValue, MidiMessage, Nibble, SevenBitValue, StructuredMidiMessage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MidiParameterNumberMessage {
    channel: Nibble,
    number: FourteenBitValue,
    value: FourteenBitValue,
    is_registered: bool,
    is_14_bit: bool,
}

impl MidiParameterNumberMessage {
    pub fn non_registered_7_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: SevenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, false)
    }

    pub fn non_registered_14_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: FourteenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, false)
    }

    pub fn registered_7_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: SevenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, true)
    }

    pub fn registered_14_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: FourteenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, true)
    }

    fn seven_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: SevenBitValue,
        is_registered: bool,
    ) -> MidiParameterNumberMessage {
        debug_assert!(channel < 16);
        debug_assert!(number < 16384);
        debug_assert!(value < 128);
        MidiParameterNumberMessage {
            channel,
            number,
            value: value as FourteenBitValue,
            is_registered,
            is_14_bit: false,
        }
    }

    fn fourteen_bit(
        channel: Nibble,
        number: FourteenBitValue,
        value: FourteenBitValue,
        is_registered: bool,
    ) -> MidiParameterNumberMessage {
        debug_assert!(channel < 16);
        debug_assert!(number < 16384);
        debug_assert!(value < 16384);
        MidiParameterNumberMessage {
            channel,
            number,
            value,
            is_registered,
            is_14_bit: true,
        }
    }

    pub fn get_channel(&self) -> Nibble {
        self.channel
    }

    pub fn get_number(&self) -> FourteenBitValue {
        self.number
    }

    pub fn get_value(&self) -> FourteenBitValue {
        self.value
    }

    pub fn is_14_bit(&self) -> bool {
        self.is_14_bit
    }

    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    // If not 14-bit, this returns only 3 messages (the last one is None)
    pub fn build_midi_messages<T: MidiMessage>(&self) -> [Option<T>; 4] {
        let mut messages = [None, None, None, None];
        let mut i = 0;
        // Number MSB
        messages[i] = Some(T::control_change(
            self.channel,
            if self.is_registered { 101 } else { 99 },
            extract_high_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Number LSB
        messages[i] = Some(T::control_change(
            self.channel,
            if self.is_registered { 100 } else { 98 },
            extract_low_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Value LSB
        if self.is_14_bit {
            messages[i] = Some(T::control_change(
                self.channel,
                38,
                extract_low_7_bit_value_from_14_bit_value(self.value),
            ));
            i += 1;
        }
        // Value MSB
        messages[i] = Some(T::control_change(
            self.channel,
            6,
            if self.is_14_bit {
                extract_high_7_bit_value_from_14_bit_value(self.value)
            } else {
                self.value as u8
            },
        ));
        messages
    }
}

pub fn could_be_part_of_parameter_number_message(msg: &impl MidiMessage) -> bool {
    match msg.to_structured() {
        StructuredMidiMessage::ControlChange(data)
            if matches!(data.controller_number, 98 | 99 | 100 | 101 | 38 | 6) =>
        {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawMidiMessage;

    #[test]
    fn fourteen_bit_parameter_number_messages() {
        // Given
        let msg = MidiParameterNumberMessage::registered_14_bit(0, 420, 15000);
        // When
        // Then
        assert_eq!(msg.get_channel(), 0);
        assert_eq!(msg.get_number(), 420);
        assert_eq!(msg.get_value(), 15000);
        assert!(msg.is_14_bit());
        assert!(msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                Some(RawMidiMessage::control_change(0, 101, 3)),
                Some(RawMidiMessage::control_change(0, 100, 36)),
                Some(RawMidiMessage::control_change(0, 38, 24)),
                Some(RawMidiMessage::control_change(0, 6, 117)),
            ]
        );
    }

    #[test]
    #[should_panic]
    fn seven_bit_parameter_number_messages_panic() {
        MidiParameterNumberMessage::non_registered_7_bit(0, 420, 255);
    }

    #[test]
    fn seven_bit_parameter_number_messages() {
        // Given
        let msg = MidiParameterNumberMessage::non_registered_7_bit(2, 421, 126);
        // When
        // Then
        assert_eq!(msg.get_channel(), 2);
        assert_eq!(msg.get_number(), 421);
        assert_eq!(msg.get_value(), 126);
        assert!(!msg.is_14_bit());
        assert!(!msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                Some(RawMidiMessage::control_change(2, 99, 3)),
                Some(RawMidiMessage::control_change(2, 98, 37)),
                Some(RawMidiMessage::control_change(2, 6, 126)),
                None,
            ]
        );
    }

    #[test]
    fn could_be_part_of_14_bit_cc_message_test() {
        // Given
        // When
        // Then
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(2, 99, 3)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(2, 98, 37)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(2, 6, 126)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 101, 3)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 100, 36)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 38, 24)
        ));
        assert!(could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 6, 117)
        ));
        assert!(!could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 5, 117)
        ));
        assert!(!could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 39, 117)
        ));
        assert!(!could_be_part_of_parameter_number_message(
            &RawMidiMessage::control_change(0, 77, 2)
        ));
    }
}
