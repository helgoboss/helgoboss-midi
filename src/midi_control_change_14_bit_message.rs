use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    ControllerNumber, MidiMessageFactory, U14,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 14-bit MIDI Control Change message.
///
/// Unlike a [`MidiMessage`] of type [`MidiMessageType::ControlChange`], this one supports 14-bit
/// resolution, that means 16384 different values instead of only 128. MIDI systems emit those by
/// sending 2 single Control Change messages in a row. The [`MidiControlChange14BitMessageScanner`]
/// can be used to extract such messages from a stream of [`MidiMessage`]s.
///
/// [`MidiMessage`]: trait.MidiMessage.html
/// [`MidiMessageType::ControlChange`]: enum.MidiMessageType.html#variant.ControlChange
/// [`MidiControlChange14BitMessageScanner`]: struct.MidiControlChange14BitMessageScanner.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MidiControlChange14BitMessage {
    channel: Channel,
    msb_controller_number: ControllerNumber,
    value: U14,
}

impl MidiControlChange14BitMessage {
    /// Creates a 14-bit Control Change message.
    ///
    /// # Panics
    ///
    /// This function panics if `msb_controller_number` can't serve as controller number for
    /// transmitting the most significant byte of a 14-bit Control Change message.
    pub fn new(
        channel: Channel,
        msb_controller_number: ControllerNumber,
        value: U14,
    ) -> MidiControlChange14BitMessage {
        assert!(
            msb_controller_number
                .corresponding_14_bit_lsb_controller_number()
                .is_some()
        );
        MidiControlChange14BitMessage {
            channel,
            msb_controller_number,
            value,
        }
    }

    /// Returns the channel of this message.
    pub fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the controller number for transmitting the most significant byte of this message.
    pub fn msb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
    }

    /// Returns the controller number for transmitting the least significant byte of this message.
    pub fn lsb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
            .corresponding_14_bit_lsb_controller_number()
            .unwrap()
    }

    /// Returns the 14-bit value of this message.
    pub fn value(&self) -> U14 {
        self.value
    }

    /// Translates this message into 2 single 7-bit Control Change MIDI messages, which need to be
    /// sent in a row in order to encode this 14-bit Control Change message.
    pub fn to_midi_messages<T: MidiMessageFactory>(&self) -> [T; 2] {
        [
            T::control_change(
                self.channel,
                self.msb_controller_number(),
                extract_high_7_bit_value_from_14_bit_value(self.value),
            ),
            T::control_change(
                self.channel,
                self.lsb_controller_number(),
                extract_low_7_bit_value_from_14_bit_value(self.value),
            ),
        ]
    }
}

impl<T: MidiMessageFactory> From<MidiControlChange14BitMessage> for [T; 2] {
    fn from(msg: MidiControlChange14BitMessage) -> Self {
        msg.to_midi_messages()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, u14, u7};
    use crate::RawMidiMessage;

    #[test]
    fn basics() {
        // Given
        let msg = MidiControlChange14BitMessage::new(ch(5), cn(2), u14(1057));
        // When
        // Then
        assert_eq!(msg.channel(), ch(5));
        assert_eq!(msg.msb_controller_number(), cn(2));
        assert_eq!(msg.lsb_controller_number(), cn(34));
        assert_eq!(msg.value(), u14(1057));
        let midi_msgs = msg.to_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                RawMidiMessage::control_change(ch(5), cn(2), u7(8)),
                RawMidiMessage::control_change(ch(5), cn(34), u7(33))
            ]
        );
        let midi_msgs_2: [RawMidiMessage; 2] = msg.into();
        assert_eq!(midi_msgs_2, midi_msgs);
    }
}
