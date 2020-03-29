use crate::{
    build_14_bit_value_from_two_7_bit_values, Midi14BitCcMessage, MidiMessage, Nibble,
    SevenBitValue, StructuredMidiMessage,
};

pub struct Midi14BitCcMessageParser {
    parser_by_channel: [ParserForOneChannel; 16],
}

impl Midi14BitCcMessageParser {
    pub fn new() -> Midi14BitCcMessageParser {
        Midi14BitCcMessageParser {
            parser_by_channel: [ParserForOneChannel::new(); 16],
        }
    }

    pub fn feed(&mut self, msg: &impl MidiMessage) -> Option<Midi14BitCcMessage> {
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
    msb_controller_number: Option<SevenBitValue>,
    value_msb: Option<SevenBitValue>,
}

impl ParserForOneChannel {
    fn new() -> ParserForOneChannel {
        ParserForOneChannel {
            msb_controller_number: None,
            value_msb: None,
        }
    }

    fn feed(&mut self, msg: &impl MidiMessage) -> Option<Midi14BitCcMessage> {
        let data = match msg.to_structured() {
            StructuredMidiMessage::ControlChange(d) => d,
            _ => return None,
        };
        match data.controller_number {
            (0..=31) => self.process_value_msb(data.controller_number, data.control_value),
            (32..=63) => {
                self.process_value_lsb(data.channel, data.controller_number, data.control_value)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.msb_controller_number = None;
        self.value_msb = None;
    }

    fn process_value_msb(
        &mut self,
        msb_controller_number: SevenBitValue,
        value_msb: SevenBitValue,
    ) -> Option<Midi14BitCcMessage> {
        self.msb_controller_number = Some(msb_controller_number);
        self.value_msb = Some(value_msb);
        None
    }

    fn process_value_lsb(
        &mut self,
        channel: Nibble,
        lsb_controller_number: SevenBitValue,
        value_lsb: SevenBitValue,
    ) -> Option<Midi14BitCcMessage> {
        let msb_controller_number = self.msb_controller_number?;
        let value_msb = self.value_msb?;
        if lsb_controller_number != msb_controller_number + 32 {
            return None;
        }
        let value = build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb);
        Some(Midi14BitCcMessage::new(
            channel,
            msb_controller_number,
            value,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MidiMessageFactory, RawMidiMessage};

    #[test]
    fn should_ignore_non_contributing_midi_messages() {
        // Given
        let mut parser = Midi14BitCcMessageParser::new();
        // When
        // Then
        assert_eq!(parser.feed(&RawMidiMessage::note_on(0, 100, 100)), None);
        assert_eq!(parser.feed(&RawMidiMessage::note_on(0, 100, 120)), None);
        assert_eq!(parser.feed(&RawMidiMessage::control_change(0, 80, 1)), None);
    }

    #[test]
    fn should_return_14_bit_result_message_on_second_lsb_midi_message() {
        // Given
        let mut parser = Midi14BitCcMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(5, 2, 8));
        let result_2 = parser.feed(&RawMidiMessage::control_change(5, 34, 33));
        // Then
        assert_eq!(result_1, None);
        let result_2 = result_2.unwrap();
        assert_eq!(result_2.get_channel(), 5);
        assert_eq!(result_2.get_msb_controller_number(), 2);
        assert_eq!(result_2.get_lsb_controller_number(), 34);
        assert_eq!(result_2.get_value(), 1057);
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut parser = Midi14BitCcMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(5, 2, 8));
        let result_2 = parser.feed(&RawMidiMessage::control_change(6, 3, 8));
        let result_3 = parser.feed(&RawMidiMessage::control_change(5, 34, 33));
        let result_4 = parser.feed(&RawMidiMessage::control_change(6, 35, 34));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.get_channel(), 5);
        assert_eq!(result_3.get_msb_controller_number(), 2);
        assert_eq!(result_3.get_lsb_controller_number(), 34);
        assert_eq!(result_3.get_value(), 1057);
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.get_channel(), 6);
        assert_eq!(result_4.get_msb_controller_number(), 3);
        assert_eq!(result_4.get_lsb_controller_number(), 35);
        assert_eq!(result_4.get_value(), 1058);
    }

    #[test]
    fn should_ignore_non_contributing_midi_messages_mixed() {
        // Given
        let mut parser = Midi14BitCcMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(5, 2, 8));
        let result_2 = parser.feed(&RawMidiMessage::control_change(5, 77, 9));
        let result_3 = parser.feed(&RawMidiMessage::control_change(5, 34, 33));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        let result_3 = result_3.unwrap();
        assert_eq!(result_3.get_channel(), 5);
        assert_eq!(result_3.get_msb_controller_number(), 2);
        assert_eq!(result_3.get_lsb_controller_number(), 34);
        assert_eq!(result_3.get_value(), 1057);
    }

    #[test]
    fn should_only_consider_last_incoming_msb() {
        // Given
        let mut parser = Midi14BitCcMessageParser::new();
        // When
        let result_1 = parser.feed(&RawMidiMessage::control_change(5, 2, 8));
        let result_2 = parser.feed(&RawMidiMessage::control_change(5, 3, 8));
        let result_3 = parser.feed(&RawMidiMessage::control_change(5, 34, 33));
        let result_4 = parser.feed(&RawMidiMessage::control_change(5, 35, 34));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        let result_4 = result_4.unwrap();
        assert_eq!(result_4.get_channel(), 5);
        assert_eq!(result_4.get_msb_controller_number(), 3);
        assert_eq!(result_4.get_lsb_controller_number(), 35);
        assert_eq!(result_4.get_value(), 1058);
    }
}
