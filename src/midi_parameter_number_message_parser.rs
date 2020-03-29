use crate::{
    build_14_bit_value_from_two_7_bit_values, FourteenBitValue, Midi14BitCcMessage, MidiMessage,
    MidiParameterNumberMessage, Nibble, SevenBitValue, StructuredMidiMessage,
};

pub struct MidiParameterNumberMessageParser {
    parser_by_channel: [ParserForOneChannel; 16],
}

impl MidiParameterNumberMessageParser {
    pub fn new() -> MidiParameterNumberMessageParser {
        MidiParameterNumberMessageParser {
            parser_by_channel: [ParserForOneChannel::new(); 16],
        }
    }

    pub fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiParameterNumberMessage> {
        let channel = msg.get_channel()?;
        self.parser_by_channel[channel as usize].feed(msg)
    }

    pub fn reset(&mut self) {
        for p in self.parser_by_channel.iter_mut() {
            p.reset();
        }
    }
}

#[derive(Clone, Copy)]
struct ParserForOneChannel {
    number_msb: Option<SevenBitValue>,
    number_lsb: Option<SevenBitValue>,
    is_registered: bool,
    value_lsb: Option<SevenBitValue>,
}

impl ParserForOneChannel {
    fn new() -> ParserForOneChannel {
        ParserForOneChannel {
            number_msb: None,
            number_lsb: None,
            is_registered: false,
            value_lsb: None,
        }
    }

    fn feed(&mut self, msg: &impl MidiMessage) -> Option<MidiParameterNumberMessage> {
        let data = match msg.to_structured() {
            StructuredMidiMessage::ControlChange(d) => d,
            _ => return None,
        };
        match data.controller_number {
            98 => self.process_number_lsb(data.control_value, false),
            99 => self.process_number_msb(data.control_value, false),
            100 => self.process_number_lsb(data.control_value, true),
            101 => self.process_number_msb(data.control_value, true),
            38 => self.process_value_lsb(data.control_value),
            6 => self.process_value_msb(data.channel, data.control_value),
            _ => None,
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
        number_lsb: SevenBitValue,
        is_registered: bool,
    ) -> Option<MidiParameterNumberMessage> {
        self.reset_value();
        self.number_lsb = Some(number_lsb);
        self.is_registered = is_registered;
        None
    }

    fn process_number_msb(
        &mut self,
        number_msb: SevenBitValue,
        is_registered: bool,
    ) -> Option<MidiParameterNumberMessage> {
        self.reset_value();
        self.number_msb = Some(number_msb);
        self.is_registered = is_registered;
        None
    }

    fn process_value_lsb(
        &mut self,
        value_lsb: SevenBitValue,
    ) -> Option<MidiParameterNumberMessage> {
        self.value_lsb = Some(value_lsb);
        None
    }

    fn process_value_msb(
        &mut self,
        channel: Nibble,
        value_msb: SevenBitValue,
    ) -> Option<MidiParameterNumberMessage> {
        let number_lsb = self.number_lsb?;
        let number_msb = self.number_msb?;
        let number = build_14_bit_value_from_two_7_bit_values(number_msb, number_lsb);
        let value = match self.value_lsb {
            Some(value_lsb) => build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb),
            None => value_msb as FourteenBitValue,
        };
        let msg = if self.is_registered {
            if self.value_lsb.is_some() {
                MidiParameterNumberMessage::registered_14_bit(channel, number, value)
            } else {
                MidiParameterNumberMessage::registered_7_bit(
                    channel,
                    number,
                    value as SevenBitValue,
                )
            }
        } else {
            if self.value_lsb.is_some() {
                MidiParameterNumberMessage::non_registered_14_bit(channel, number, value)
            } else {
                MidiParameterNumberMessage::non_registered_7_bit(
                    channel,
                    number,
                    value as SevenBitValue,
                )
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
    use crate::RawMidiMessage;

    #[test]
    fn should_ignore_non_contributing_midi_messages() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        // Then
        assert_eq!(parser.feed(&RawMidiMessage::note_on(0, 100, 100)), None);
        assert_eq!(parser.feed(&RawMidiMessage::note_on(0, 100, 120)), None);
        assert_eq!(parser.feed(&RawMidiMessage::control_change(0, 80, 1)), None);
    }

    #[test]
    fn should_return_parameter_number_result_message_on_fourth_midi_message() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(0, 101, 3));
        let result_2 = parser.feed(&RawMidiMessage::control_change(0, 100, 36));
        let result_3 = parser.feed(&RawMidiMessage::control_change(0, 38, 24));
        let result_4 = parser.feed(&RawMidiMessage::control_change(0, 6, 117));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.get_channel(), 0);
        assert_eq!(result_4.get_number(), 420);
        assert_eq!(result_4.get_value(), 15000);
        assert!(result_4.is_registered());
        assert!(result_4.is_14_bit());
    }

    #[test]
    fn should_return_parameter_number_result_message_on_third_midi_message() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(2, 99, 3));
        let result_2 = parser.feed(&RawMidiMessage::control_change(2, 98, 37));
        let result_3 = parser.feed(&RawMidiMessage::control_change(2, 6, 126));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.get_channel(), 2);
        assert_eq!(result_3.get_number(), 421);
        assert_eq!(result_3.get_value(), 126);
        assert!(!result_3.is_registered());
        assert!(!result_3.is_14_bit());
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(0, 101, 3));
        let result_2 = parser.feed(&RawMidiMessage::control_change(2, 99, 3));
        let result_3 = parser.feed(&RawMidiMessage::control_change(0, 100, 36));
        let result_4 = parser.feed(&RawMidiMessage::control_change(2, 98, 37));
        let result_5 = parser.feed(&RawMidiMessage::control_change(0, 38, 24));
        let result_6 = parser.feed(&RawMidiMessage::control_change(2, 6, 126));
        let result_7 = parser.feed(&RawMidiMessage::control_change(0, 6, 117));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_3, None);
        assert_eq!(result_5, None);
        let result_7 = result_7.unwrap();
        assert_eq!(result_7.get_channel(), 0);
        assert_eq!(result_7.get_number(), 420);
        assert_eq!(result_7.get_value(), 15000);
        assert!(result_7.is_registered());
        assert!(result_7.is_14_bit());
        assert_eq!(result_2, None);
        assert_eq!(result_4, None);
        let result_6 = result_6.unwrap();
        assert_eq!(result_6.get_channel(), 2);
        assert_eq!(result_6.get_number(), 421);
        assert_eq!(result_6.get_value(), 126);
        assert!(!result_6.is_registered());
        assert!(!result_6.is_14_bit());
    }

    #[test]
    fn should_ignore_non_contributing_midi_messages_mixed() {
        // Given
        let mut parser = MidiParameterNumberMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(2, 99, 3));
        parser.feed(&RawMidiMessage::control_change(2, 34, 5));
        parser.feed(&RawMidiMessage::note_on(2, 100, 105));
        let result_2 = parser.feed(&RawMidiMessage::control_change(2, 98, 37));
        parser.feed(&RawMidiMessage::control_change(2, 50, 6));
        let result_3 = parser.feed(&RawMidiMessage::control_change(2, 6, 126));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.get_channel(), 2);
        assert_eq!(result_3.get_number(), 421);
        assert_eq!(result_3.get_value(), 126);
        assert!(!result_3.is_registered());
        assert!(!result_3.is_14_bit());
    }
}
