use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    MidiMessage, MidiMessageFactory, SevenBitValue, StructuredMidiMessage, SEVEN_BIT_VALUE_MAX,
    U14,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiParameterNumberMessage {
    channel: Channel,
    number: U14,
    value: U14,
    is_registered: bool,
    is_14_bit: bool,
}

impl MidiParameterNumberMessage {
    pub fn non_registered_7_bit(
        channel: Channel,
        number: U14,
        value: SevenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, false)
    }

    pub fn non_registered_14_bit(
        channel: Channel,
        number: U14,
        value: U14,
    ) -> MidiParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, false)
    }

    pub fn registered_7_bit(
        channel: Channel,
        number: U14,
        value: SevenBitValue,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, true)
    }

    pub fn registered_14_bit(
        channel: Channel,
        number: U14,
        value: U14,
    ) -> MidiParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, true)
    }

    fn seven_bit(
        channel: Channel,
        number: U14,
        value: SevenBitValue,
        is_registered: bool,
    ) -> MidiParameterNumberMessage {
        debug_assert!(value <= SEVEN_BIT_VALUE_MAX);
        MidiParameterNumberMessage {
            channel,
            number,
            value: U14(value as u16),
            is_registered,
            is_14_bit: false,
        }
    }

    fn fourteen_bit(
        channel: Channel,
        number: U14,
        value: U14,
        is_registered: bool,
    ) -> MidiParameterNumberMessage {
        MidiParameterNumberMessage {
            channel,
            number,
            value,
            is_registered,
            is_14_bit: true,
        }
    }

    pub fn get_channel(&self) -> Channel {
        self.channel
    }

    pub fn get_number(&self) -> U14 {
        self.number
    }

    pub fn get_value(&self) -> U14 {
        self.value
    }

    pub fn is_14_bit(&self) -> bool {
        self.is_14_bit
    }

    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    // If not 14-bit, this returns only 3 messages (the last one is None)
    pub fn build_midi_messages<T: MidiMessageFactory>(&self) -> [Option<T>; 4] {
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
                u16::from(self.value) as SevenBitValue
            },
        ));
        messages
    }
}

pub fn msg_could_be_part_of_parameter_number_msg(msg: &impl MidiMessage) -> bool {
    match msg.to_structured() {
        StructuredMidiMessage::ControlChange {
            controller_number, ..
        } => ctrl_number_could_be_part_of_parameter_number_msg(controller_number),
        _ => false,
    }
}

pub fn ctrl_number_could_be_part_of_parameter_number_msg(controller_number: SevenBitValue) -> bool {
    matches!(controller_number, 98 | 99 | 100 | 101 | 38 | 6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel as ch, u14, RawMidiMessage};

    #[test]
    fn fourteen_bit_parameter_number_messages() {
        // Given
        let msg = MidiParameterNumberMessage::registered_14_bit(ch(0), u14(420), u14(15000));
        // When
        // Then
        assert_eq!(msg.get_channel(), ch(0));
        assert_eq!(msg.get_number(), u14(420));
        assert_eq!(msg.get_value(), u14(15000));
        assert!(msg.is_14_bit());
        assert!(msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                Some(RawMidiMessage::control_change(ch(0), 101, 3)),
                Some(RawMidiMessage::control_change(ch(0), 100, 36)),
                Some(RawMidiMessage::control_change(ch(0), 38, 24)),
                Some(RawMidiMessage::control_change(ch(0), 6, 117)),
            ]
        );
    }

    #[test]
    #[should_panic]
    fn seven_bit_parameter_number_messages_panic() {
        MidiParameterNumberMessage::non_registered_7_bit(ch(0), u14(420), 255);
    }

    #[test]
    fn seven_bit_parameter_number_messages() {
        // Given
        let msg = MidiParameterNumberMessage::non_registered_7_bit(ch(2), u14(421), 126);
        // When
        // Then
        assert_eq!(msg.get_channel(), ch(2));
        assert_eq!(msg.get_number(), u14(421));
        assert_eq!(msg.get_value(), u14(126));
        assert!(!msg.is_14_bit());
        assert!(!msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                Some(RawMidiMessage::control_change(ch(2), 99, 3)),
                Some(RawMidiMessage::control_change(ch(2), 98, 37)),
                Some(RawMidiMessage::control_change(ch(2), 6, 126)),
                None,
            ]
        );
    }

    #[test]
    fn could_be_part_of_14_bit_cc_message_test() {
        // Given
        // When
        // Then
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(2), 99, 3)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(2), 98, 37)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(2), 6, 126)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 101, 3)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 100, 36)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 38, 24)
        ));
        assert!(msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 6, 117)
        ));
        assert!(!msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 5, 117)
        ));
        assert!(!msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 39, 117)
        ));
        assert!(!msg_could_be_part_of_parameter_number_msg(
            &RawMidiMessage::control_change(ch(0), 77, 2)
        ));
    }
}
