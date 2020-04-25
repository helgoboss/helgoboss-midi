use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    ControllerNumber, MidiMessageFactory, U14, U7,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A MIDI Parameter Number message, either registered (RPN) or non-registered (NRPN).
///
/// MIDI systems emit those by sending up to 4 single Control Change messages in a row. The
/// [`MidiParameterNumberMessageScanner`] can be used to extract such messages from a stream of
/// [`MidiMessage`]s.
///
/// [`MidiMessage`]: trait.MidiMessage.html
/// [`MidiParameterNumberMessageScanner`]: struct.MidiParameterNumberMessageScanner.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MidiParameterNumberMessage {
    channel: Channel,
    number: U14,
    value: U14,
    is_registered: bool,
    is_14_bit: bool,
}

impl MidiParameterNumberMessage {
    /// Creates an NRPN message with a 7-bit value.
    pub fn non_registered_7_bit(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, false)
    }

    /// Creates an NRPN message with a 14-bit value.
    pub fn non_registered_14_bit(
        channel: Channel,
        number: U14,
        value: U14,
    ) -> MidiParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, false)
    }

    /// Creates an RPN message with a 7-bit value.
    pub fn registered_7_bit(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> MidiParameterNumberMessage {
        Self::seven_bit(channel, number, value, true)
    }

    /// Creates an RPN message with a 14-bit value.
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

    /// Returns the channel of this message.
    pub fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the parameter number of this message.
    pub fn number(&self) -> U14 {
        self.number
    }

    /// Returns the value of this message.
    ///
    /// If it's just a 7-bit message, the value is <= 127.
    pub fn value(&self) -> U14 {
        self.value
    }

    /// Returns `true` if this message has a 14-bit value and `false` if only a 7-bit value.
    pub fn is_14_bit(&self) -> bool {
        self.is_14_bit
    }

    /// Returns whether this message uses a registered parameter number.
    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    /// Translates this message into up to 4 single 7-bit Control Change MIDI messages, which need
    /// to be sent in a row in order to encode this (N)RPN message.
    ///
    /// If this message has a 14-bit value, all returned messages are `Some`. If it has a 7-bit
    /// value only, the last one is `None`.
    pub fn to_midi_messages<T: MidiMessageFactory>(&self) -> [Option<T>; 4] {
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
                U7(self.value.get() as u8)
            },
        ));
        messages
    }
}

impl<T: MidiMessageFactory> From<MidiParameterNumberMessage> for [Option<T>; 4] {
    fn from(msg: MidiParameterNumberMessage) -> Self {
        msg.to_midi_messages()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, u14, u7};
    use crate::RawMidiMessage;

    #[test]
    fn parameter_number_messages_14_bit() {
        // Given
        let msg = MidiParameterNumberMessage::registered_14_bit(ch(0), u14(420), u14(15000));
        // When
        // Then
        assert_eq!(msg.channel(), ch(0));
        assert_eq!(msg.number(), u14(420));
        assert_eq!(msg.value(), u14(15000));
        assert!(msg.is_14_bit());
        assert!(msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.to_midi_messages();
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
        assert_eq!(msg.channel(), ch(2));
        assert_eq!(msg.number(), u14(421));
        assert_eq!(msg.value(), u14(126));
        assert!(!msg.is_14_bit());
        assert!(!msg.is_registered());
        let midi_msgs: [Option<RawMidiMessage>; 4] = msg.to_midi_messages();
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
