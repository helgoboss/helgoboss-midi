use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    ControllerNumber, MidiMessageFactory, U14, U7,
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
        value: U7,
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
        value: U7,
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
        value: U7,
        is_registered: bool,
    ) -> MidiParameterNumberMessage {
        MidiParameterNumberMessage {
            channel,
            number,
            value: value.into(),
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
            if self.is_registered {
                ControllerNumber::REGISTERED_PARAMETER_NUMBER_MSB
            } else {
                ControllerNumber::NON_REGISTERED_PARAMETER_NUMBER_MSB
            },
            extract_high_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Number LSB
        messages[i] = Some(T::control_change(
            self.channel,
            if self.is_registered {
                ControllerNumber::REGISTERED_PARAMETER_NUMBER_LSB
            } else {
                ControllerNumber::NON_REGISTERED_PARAMETER_NUMBER_LSB
            },
            extract_low_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Value LSB
        if self.is_14_bit {
            messages[i] = Some(T::control_change(
                self.channel,
                ControllerNumber::DATA_ENTRY_MSB_LSB,
                extract_low_7_bit_value_from_14_bit_value(self.value),
            ));
            i += 1;
        }
        // Value MSB
        messages[i] = Some(T::control_change(
            self.channel,
            ControllerNumber::DATA_ENTRY_MSB,
            if self.is_14_bit {
                extract_high_7_bit_value_from_14_bit_value(self.value)
            } else {
                U7(u16::from(self.value) as u8)
            },
        ));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel as ch, controller_number as cn, u14, u7, RawMidiMessage};

    #[test]
    fn parameter_number_messages_14_bit() {
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
                Some(RawMidiMessage::control_change(ch(0), cn(101), u7(3))),
                Some(RawMidiMessage::control_change(ch(0), cn(100), u7(36))),
                Some(RawMidiMessage::control_change(ch(0), cn(38), u7(24))),
                Some(RawMidiMessage::control_change(ch(0), cn(6), u7(117))),
            ]
        );
    }

    #[test]
    #[should_panic]
    fn parameter_number_messages_7_bit_panic() {
        MidiParameterNumberMessage::non_registered_7_bit(ch(0), u14(420), u7(255));
    }

    #[test]
    fn parameter_number_messages_7_bit() {
        // Given
        let msg = MidiParameterNumberMessage::non_registered_7_bit(ch(2), u14(421), u7(126));
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
                Some(RawMidiMessage::control_change(ch(2), cn(99), u7(3))),
                Some(RawMidiMessage::control_change(ch(2), cn(98), u7(37))),
                Some(RawMidiMessage::control_change(ch(2), cn(6), u7(126))),
                None,
            ]
        );
    }
}
