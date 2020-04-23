use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, ControllerNumber,
    MidiControlChange14BitMessage, MidiMessage, StructuredMidiMessage, U7,
};

#[derive(Default)]
pub struct MidiControlChange14BitMessageParser {
    parser_by_channel: [ParserForOneChannel; Channel::COUNT as usize],
}

impl MidiControlChange14BitMessageParser {
    pub fn new() -> MidiControlChange14BitMessageParser {
        Default::default()
    }

    pub fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiControlChange14BitMessage> {
        let channel = msg.channel()?;
        self.parser_by_channel[usize::from(channel)].feed(msg)
    }

    pub fn reset(&mut self) {
        for p in self.parser_by_channel.iter_mut() {
            p.reset();
        }
    }
}

#[derive(Default, Clone, Copy)]
struct ParserForOneChannel {
    msb_controller_number: Option<ControllerNumber>,
    value_msb: Option<U7>,
}

impl ParserForOneChannel {
    fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiControlChange14BitMessage> {
        match msg.to_structured() {
            StructuredMidiMessage::ControlChange {
                controller_number,
                channel,
                control_value,
            } => match u8::from(controller_number) {
                (0..=31) => self.process_value_msb(controller_number, control_value),
                (32..=63) => self.process_value_lsb(channel, controller_number, control_value),
                _ => None,
            },
            _ => return None,
        }
    }

    fn reset(&mut self) {
        self.msb_controller_number = None;
        self.value_msb = None;
    }

    fn process_value_msb(
        &mut self,
        msb_controller_number: ControllerNumber,
        value_msb: U7,
    ) -> Option<MidiControlChange14BitMessage> {
        self.msb_controller_number = Some(msb_controller_number);
        self.value_msb = Some(value_msb);
        None
    }

    fn process_value_lsb(
        &mut self,
        channel: Channel,
        lsb_controller_number: ControllerNumber,
        value_lsb: U7,
    ) -> Option<MidiControlChange14BitMessage> {
        let msb_controller_number = self.msb_controller_number?;
        let value_msb = self.value_msb?;
        if lsb_controller_number != msb_controller_number.corresponding_14_bit_lsb().unwrap() {
            return None;
        }
        let value = build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb);
        Some(MidiControlChange14BitMessage::new(
            channel,
            msb_controller_number,
            value,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, key_number, u14, u7};
    use crate::{MidiMessageFactory, RawMidiMessage};

    #[test]
    fn should_ignore_non_contributing_midi_messages() {
        // Given
        let mut parser = MidiControlChange14BitMessageParser::new();
        // When
        // Then
        assert_eq!(
            parser.feed(&RawMidiMessage::note_on(ch(0), key_number(100), u7(100))),
            None
        );
        assert_eq!(
            parser.feed(&RawMidiMessage::note_on(ch(0), key_number(100), u7(120))),
            None
        );
        assert_eq!(
            parser.feed(&RawMidiMessage::control_change(ch(0), cn(80), u7(1))),
            None
        );
    }

    #[test]
    fn should_return_14_bit_result_message_on_second_lsb_midi_message() {
        // Given
        let mut parser = MidiControlChange14BitMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(34), u7(33)));
        // Then
        assert_eq!(result_1, None);
        let result_2 = result_2.unwrap();
        assert_eq!(result_2.channel(), ch(5));
        assert_eq!(result_2.msb_controller_number(), cn(2));
        assert_eq!(result_2.lsb_controller_number(), cn(34));
        assert_eq!(result_2.value(), u14(1057));
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut parser = MidiControlChange14BitMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(6), cn(3), u7(8)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(34), u7(33)));
        let result_4 = parser.feed(&RawMidiMessage::control_change(ch(6), cn(35), u7(34)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.channel(), ch(5));
        assert_eq!(result_3.msb_controller_number(), cn(2));
        assert_eq!(result_3.lsb_controller_number(), cn(34));
        assert_eq!(result_3.value(), u14(1057));
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.channel(), ch(6));
        assert_eq!(result_4.msb_controller_number(), cn(3));
        assert_eq!(result_4.lsb_controller_number(), cn(35));
        assert_eq!(result_4.value(), u14(1058));
    }

    #[test]
    fn should_ignore_non_contributing_midi_messages_mixed() {
        // Given
        let mut parser = MidiControlChange14BitMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(77), u7(9)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(34), u7(33)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.channel(), ch(5));
        assert_eq!(result_3.msb_controller_number(), cn(2));
        assert_eq!(result_3.lsb_controller_number(), cn(34));
        assert_eq!(result_3.value(), u14(1057));
    }

    #[test]
    fn should_only_consider_last_incoming_msb() {
        // Given
        let mut parser = MidiControlChange14BitMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(3), u7(8)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(34), u7(33)));
        let result_4 = parser.feed(&RawMidiMessage::control_change(ch(5), cn(35), u7(34)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.channel(), ch(5));
        assert_eq!(result_4.msb_controller_number(), cn(3));
        assert_eq!(result_4.lsb_controller_number(), cn(35));
        assert_eq!(result_4.value(), u14(1058));
    }
}
