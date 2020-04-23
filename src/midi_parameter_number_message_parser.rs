use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, MidiMessage, MidiParameterNumberMessage,
    StructuredMidiMessage, U7,
};

#[derive(Default)]
pub struct MidiParameterNumberMessageParser {
    parser_by_channel: [ParserForOneChannel; Channel::COUNT as usize],
}

impl MidiParameterNumberMessageParser {
    pub fn new() -> MidiParameterNumberMessageParser {
        Default::default()
    }

    pub fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiParameterNumberMessage> {
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
    number_msb: Option<U7>,
    number_lsb: Option<U7>,
    is_registered: bool,
    value_lsb: Option<U7>,
}

impl ParserForOneChannel {
    fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiParameterNumberMessage> {
        match msg.to_structured() {
            StructuredMidiMessage::ControlChange {
                channel,
                controller_number,
                control_value,
            } => match u8::from(controller_number) {
                98 => self.process_number_lsb(control_value, false),
                99 => self.process_number_msb(control_value, false),
                100 => self.process_number_lsb(control_value, true),
                101 => self.process_number_msb(control_value, true),
                38 => self.process_value_lsb(control_value),
                6 => self.process_value_msb(channel, control_value),
                _ => None,
            },
            _ => return None,
        }
    }

    fn reset(&mut self) {
        self.number_msb = None;
        self.number_lsb = None;
        self.is_registered = false;
        self.reset_value();
    }

    fn process_number_lsb(
        &mut self,
        number_lsb: U7,
        is_registered: bool,
    ) -> Option<MidiParameterNumberMessage> {
        self.reset_value();
        self.number_lsb = Some(number_lsb);
        self.is_registered = is_registered;
        None
    }

    fn process_number_msb(
        &mut self,
        number_msb: U7,
        is_registered: bool,
    ) -> Option<MidiParameterNumberMessage> {
        self.reset_value();
        self.number_msb = Some(number_msb);
        self.is_registered = is_registered;
        None
    }

    fn process_value_lsb(&mut self, value_lsb: U7) -> Option<MidiParameterNumberMessage> {
        self.value_lsb = Some(value_lsb);
        None
    }

    fn process_value_msb(
        &mut self,
        channel: Channel,
        value_msb: U7,
    ) -> Option<MidiParameterNumberMessage> {
        let number_lsb = self.number_lsb?;
        let number_msb = self.number_msb?;
        let number = build_14_bit_value_from_two_7_bit_values(number_msb, number_lsb);
        let msg = if self.is_registered {
            match self.value_lsb {
                Some(value_lsb) => MidiParameterNumberMessage::registered_14_bit(
                    channel,
                    number,
                    build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb),
                ),
                None => MidiParameterNumberMessage::registered_7_bit(channel, number, value_msb),
            }
        } else {
            match self.value_lsb {
                Some(value_lsb) => MidiParameterNumberMessage::non_registered_14_bit(
                    channel,
                    number,
                    build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb),
                ),
                None => {
                    MidiParameterNumberMessage::non_registered_7_bit(channel, number, value_msb)
                }
            }
        };
        Some(msg)
    }

    fn reset_value(&mut self) {
        self.value_lsb = None;
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
        let mut parser = MidiParameterNumberMessageParser::new();
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
    fn should_return_parameter_number_result_message_on_fourth_midi_message() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(38), u7(24)));
        let result_4 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(6), u7(117)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.channel(), ch(0));
        assert_eq!(result_4.number(), u14(420));
        assert_eq!(result_4.value(), u14(15000));
        assert!(result_4.is_registered());
        assert!(result_4.is_14_bit());
    }

    #[test]
    fn should_return_parameter_number_result_message_on_third_midi_message() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(6), u7(126)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.channel(), ch(2));
        assert_eq!(result_3.number(), u14(421));
        assert_eq!(result_3.value(), u14(126));
        assert!(!result_3.is_registered());
        assert!(!result_3.is_14_bit());
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(99), u7(3)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(100), u7(36)));
        let result_4 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(98), u7(37)));
        let result_5 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(38), u7(24)));
        let result_6 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(6), u7(126)));
        let result_7 = parser.feed(&RawMidiMessage::control_change(ch(0), cn(6), u7(117)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_3, None);
        assert_eq!(result_5, None);
        let result_7 = result_7.unwrap();
        assert_eq!(result_7.channel(), ch(0));
        assert_eq!(result_7.number(), u14(420));
        assert_eq!(result_7.value(), u14(15000));
        assert!(result_7.is_registered());
        assert!(result_7.is_14_bit());
        assert_eq!(result_2, None);
        assert_eq!(result_4, None);
        let result_6 = result_6.unwrap();
        assert_eq!(result_6.channel(), ch(2));
        assert_eq!(result_6.number(), u14(421));
        assert_eq!(result_6.value(), u14(126));
        assert!(!result_6.is_registered());
        assert!(!result_6.is_14_bit());
    }

    #[test]
    fn should_ignore_non_contributing_midi_messages_mixed() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(99), u7(3)));
        parser.feed(&RawMidiMessage::control_change(ch(2), cn(34), u7(5)));
        parser.feed(&RawMidiMessage::note_on(ch(2), key_number(100), u7(105)));
        let result_2 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(98), u7(37)));
        parser.feed(&RawMidiMessage::control_change(ch(2), cn(50), u7(6)));
        let result_3 = parser.feed(&RawMidiMessage::control_change(ch(2), cn(6), u7(126)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.channel(), ch(2));
        assert_eq!(result_3.number(), u14(421));
        assert_eq!(result_3.value(), u14(126));
        assert!(!result_3.is_registered());
        assert!(!result_3.is_14_bit());
    }
}
