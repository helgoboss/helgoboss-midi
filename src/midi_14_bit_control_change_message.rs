use crate::{
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value, Channel,
    ControllerNumber, MidiMessage, MidiMessageFactory, StructuredMidiMessage, U14,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Midi14BitControlChangeMessage {
    channel: Channel,
    msb_controller_number: ControllerNumber,
    value: U14,
}

impl Midi14BitControlChangeMessage {
    pub fn new(
        channel: Channel,
        msb_controller_number: ControllerNumber,
        value: U14,
    ) -> Midi14BitControlChangeMessage {
        assert!(msb_controller_number.can_act_as_14_bit_msb());
        Midi14BitControlChangeMessage {
            channel,
            msb_controller_number,
            value,
        }
    }

    pub fn get_channel(&self) -> Channel {
        self.channel
    }

    pub fn get_msb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
    }

    pub fn get_lsb_controller_number(&self) -> ControllerNumber {
        self.msb_controller_number
            .get_corresponding_14_bit_lsb()
            .unwrap()
    }

    pub fn get_value(&self) -> U14 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel as ch, controller_number as cn, u14, u7, RawMidiMessage};

    #[test]
    fn basics() {
        // Given
        let msg = Midi14BitControlChangeMessage::new(ch(5), cn(2), u14(1057));
        // When
        // Then
        assert_eq!(msg.get_channel(), ch(5));
        assert_eq!(msg.get_msb_controller_number(), cn(2));
        assert_eq!(msg.get_lsb_controller_number(), cn(34));
        assert_eq!(msg.get_value(), u14(1057));
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
