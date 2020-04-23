use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    ControllerNumber, MidiMessageFactory, U14,
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MidiControlChange14BitMessage {
    channel: Channel,
    msb_controller_number: ControllerNumber,
    value: U14,
}

impl MidiControlChange14BitMessage {
    pub fn new(
        channel: Channel,
        msb_controller_number: ControllerNumber,
        value: U14,
    ) -> MidiControlChange14BitMessage {
        assert!(msb_controller_number.can_act_as_14_bit_msb());
        MidiControlChange14BitMessage {
            channel,
            msb_controller_number,
            value,
        }
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn msb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
    }

    pub fn lsb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
            .corresponding_14_bit_lsb()
            .unwrap()
    }

    pub fn value(&self) -> U14 {
        self.value
    }

    pub fn build_midi_messages<T: MidiMessageFactory>(&self) -> [T; 2] {
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
        msg.build_midi_messages()
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
        let midi_msgs: [RawMidiMessage; 2] = msg.build_midi_messages();
        assert_eq!(
            midi_msgs,
            [
                RawMidiMessage::control_change(ch(5), cn(2), u7(8)),
                RawMidiMessage::control_change(ch(5), cn(34), u7(33))
            ]
        );
    }
}
