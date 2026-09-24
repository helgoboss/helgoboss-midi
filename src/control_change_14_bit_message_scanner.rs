use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, ControlChange14BitMessage, ControllerNumber,
    ScanOutcome, ShortMessage, StructuredShortMessage, U7,
};

/// Scanner for detecting 14-bit Control Change messages in a stream of short MIDI messages.
///
/// # Example
///
/// ```
/// use helgoboss_midi::test_util::control_change;
/// use helgoboss_midi::{Channel, ControlChange14BitMessage, ControlChange14BitMessageScanner, ControllerNumber, ScanOutcome, U14};
///
/// let mut scanner = ControlChange14BitMessageScanner::new();
/// let result_1 = scanner.feed(&control_change(5, 2, 8));
/// let result_2 = scanner.feed(&control_change(5, 34, 33));
/// assert_eq!(result_1, ScanOutcome::Consumed);
/// assert_eq!(
///     result_2,
///     ScanOutcome::Complete(ControlChange14BitMessage::new(
///         Channel::new(5),
///         ControllerNumber::new(2),
///         U14::new(1057)
///     ))
/// );
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ControlChange14BitMessageScanner {
    scanner_by_channel: [ScannerForOneChannel; 16],
}

impl ControlChange14BitMessageScanner {
    /// Creates a new scanner.
    pub fn new() -> ControlChange14BitMessageScanner {
        Default::default()
    }

    /// Feeds the scanner a single short message.
    ///
    /// Returns the 14-bit Control Change message if one has been detected.  
    pub fn feed(&mut self, msg: &impl ShortMessage) -> ScanOutcome<ControlChange14BitMessage> {
        let Some(channel) = msg.channel() else {
            return ScanOutcome::Unhandled;
        };
        self.scanner_by_channel[usize::from(channel)].feed(msg)
    }

    /// Resets the scanner discarding all intermediate scanning progress.
    pub fn reset(&mut self) {
        for p in self.scanner_by_channel.iter_mut() {
            p.reset();
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
struct ScannerForOneChannel {
    msb_controller_number: Option<ControllerNumber>,
    value_msb: Option<U7>,
}

impl ScannerForOneChannel {
    fn feed(&mut self, msg: &impl ShortMessage) -> ScanOutcome<ControlChange14BitMessage> {
        match msg.to_structured() {
            StructuredShortMessage::ControlChange {
                controller_number,
                channel,
                control_value,
            } => match controller_number.get() {
                (0..=31) => self.process_value_msb(controller_number, control_value),
                (32..=63) => self.process_value_lsb(channel, controller_number, control_value),
                _ => ScanOutcome::Unhandled,
            },
            _ => ScanOutcome::Unhandled,
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
    ) -> ScanOutcome<ControlChange14BitMessage> {
        self.msb_controller_number = Some(msb_controller_number);
        self.value_msb = Some(value_msb);
        ScanOutcome::Consumed
    }

    fn process_value_lsb(
        &mut self,
        channel: Channel,
        lsb_controller_number: ControllerNumber,
        value_lsb: U7,
    ) -> ScanOutcome<ControlChange14BitMessage> {
        let Some(msb_controller_number) = self.msb_controller_number else {
            return ScanOutcome::Unhandled;
        };
        let Some(value_msb) = self.value_msb else {
            return ScanOutcome::Unhandled;
        };
        let corresponding_14_bit_lsb_cc_number = msb_controller_number
            .corresponding_14_bit_lsb_controller_number()
            .expect("corresponding 14-bit LSB CC number should be set");
        if lsb_controller_number != corresponding_14_bit_lsb_cc_number {
            return ScanOutcome::Unhandled;
        }
        let value = build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb);
        let msg = ControlChange14BitMessage::new(channel, msb_controller_number, value);
        ScanOutcome::Complete(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, key_number, u14, u7};
    use crate::{RawShortMessage, ShortMessageFactory};

    #[test]
    fn should_ignore_non_contributing_messages() {
        // Given
        let mut scanner = ControlChange14BitMessageScanner::new();
        // When
        // Then
        assert_eq!(
            scanner.feed(&RawShortMessage::note_on(ch(0), key_number(100), u7(100))),
            ScanOutcome::Unhandled
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::note_on(ch(0), key_number(100), u7(120))),
            ScanOutcome::Unhandled
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::control_change(ch(0), cn(80), u7(1))),
            ScanOutcome::Unhandled
        );
    }

    #[test]
    fn should_return_14_bit_result_message_on_second_lsb_short_message() {
        // Given
        let mut scanner = ControlChange14BitMessageScanner::new();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(34), u7(33)));
        // Then
        assert_eq!(result_1, ScanOutcome::Consumed);
        let ScanOutcome::Complete(result_2) = result_2 else {
            panic!("result_2 should be complete");
        };
        assert_eq!(result_2.channel(), ch(5));
        assert_eq!(result_2.msb_controller_number(), cn(2));
        assert_eq!(result_2.lsb_controller_number(), cn(34));
        assert_eq!(result_2.value(), u14(1057));
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut scanner = ControlChange14BitMessageScanner::new();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(6), cn(3), u7(8)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(34), u7(33)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(6), cn(35), u7(34)));
        // Then
        assert_eq!(result_1, ScanOutcome::Consumed);
        assert_eq!(result_2, ScanOutcome::Consumed);
        let ScanOutcome::Complete(result_3) = result_3 else {
            panic!("result_3 should be complete");
        };
        assert_eq!(result_3.channel(), ch(5));
        assert_eq!(result_3.msb_controller_number(), cn(2));
        assert_eq!(result_3.lsb_controller_number(), cn(34));
        assert_eq!(result_3.value(), u14(1057));
        let ScanOutcome::Complete(result_4) = result_4 else {
            panic!("result_4 should be complete");
        };
        assert_eq!(result_4.channel(), ch(6));
        assert_eq!(result_4.msb_controller_number(), cn(3));
        assert_eq!(result_4.lsb_controller_number(), cn(35));
        assert_eq!(result_4.value(), u14(1058));
    }

    #[test]
    fn should_ignore_non_contributing_short_messages_mixed() {
        // Given
        let mut scanner = ControlChange14BitMessageScanner::new();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(77), u7(9)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(34), u7(33)));
        // Then
        assert_eq!(result_1, ScanOutcome::Consumed);
        assert_eq!(result_2, ScanOutcome::Unhandled);
        let ScanOutcome::Complete(result_3) = result_3 else {
            panic!("result_3 should be complete");
        };
        assert_eq!(result_3.channel(), ch(5));
        assert_eq!(result_3.msb_controller_number(), cn(2));
        assert_eq!(result_3.lsb_controller_number(), cn(34));
        assert_eq!(result_3.value(), u14(1057));
    }

    #[test]
    fn should_only_consider_last_incoming_msb() {
        // Given
        let mut scanner = ControlChange14BitMessageScanner::new();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(2), u7(8)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(3), u7(8)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(34), u7(33)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(5), cn(35), u7(34)));
        // Then
        assert_eq!(result_1, ScanOutcome::Consumed);
        assert_eq!(result_2, ScanOutcome::Consumed);
        assert_eq!(result_3, ScanOutcome::Unhandled);
        let ScanOutcome::Complete(result_4) = result_4 else {
            panic!("result_4 should be complete");
        };
        assert_eq!(result_4.channel(), ch(5));
        assert_eq!(result_4.msb_controller_number(), cn(3));
        assert_eq!(result_4.lsb_controller_number(), cn(35));
        assert_eq!(result_4.value(), u14(1058));
    }
}
