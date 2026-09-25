use crate::{
    ControlChange14BitMessageScanner, MidiMessage, ParameterNumberMessageScanner, ScanOutcome,
    ShortMessage,
};

/// Scanner for detecting potential (N)RPN or 14-bit Control Change messages in a stream of
/// short messages.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MidiMessageScanner<T> {
    pn_scanner: ParameterNumberMessageScanner,
    cc_14_bit_scanner: ControlChange14BitMessageScanner,
    last_consumed_payload: Option<T>,
}

impl<T> Default for MidiMessageScanner<T> {
    fn default() -> Self {
        Self {
            pn_scanner: Default::default(),
            cc_14_bit_scanner: Default::default(),
            last_consumed_payload: None,
        }
    }
}

impl<T> MidiMessageScanner<T> {
    /// Creates a new scanner.
    pub fn new() -> Self {
        Default::default()
    }

    /// Feeds the scanner a single short message.
    ///
    /// Behavior:
    ///
    /// - If the given message can be the first message in a potential (N)RPN or 14-bit message pair,
    ///   the function normally returns nothing (consuming the message). But if a previous
    ///   message was consumed already by the same scanner and is now superseded, the function
    ///   returns this previously consumed message.
    /// - If the given message completes a pending message pair, the function returns the
    ///   corresponding (N)RPN or 14-bit message.
    /// - If the given message can't be part of a potential (N)RPN or 14-bit message pair,
    ///   the function returns that same message and any previously consumed message.
    pub fn feed<S: ShortMessage>(&mut self, msg: S, payload: T) -> MidiMessageScannerOutcome<S, T> {
        let last_consumed_payload = self.last_consumed_payload.take();
        // Try (N)RPN parser
        match self.pn_scanner.feed(&msg) {
            ScanOutcome::Unhandled => {}
            ScanOutcome::Pending => {
                self.last_consumed_payload = Some(payload);
                return MidiMessageScannerOutcome {
                    leftover: last_consumed_payload,
                    message: None,
                };
            }
            ScanOutcome::Complete(pn_msg) => {
                return MidiMessageScannerOutcome {
                    leftover: None,
                    message: Some(MidiMessage::ParameterNumber(pn_msg)),
                }
            }
        }
        // Try 14-bit CC parser
        match self.cc_14_bit_scanner.feed(&msg) {
            ScanOutcome::Unhandled => {
                // Both unhandled. Return message as is.
                MidiMessageScannerOutcome {
                    leftover: last_consumed_payload,
                    message: Some(MidiMessage::Short(msg)),
                }
            }
            ScanOutcome::Pending => {
                self.last_consumed_payload = Some(payload);
                MidiMessageScannerOutcome {
                    leftover: last_consumed_payload,
                    message: None,
                }
            }
            ScanOutcome::Complete(cc_14_bit_msg) => MidiMessageScannerOutcome {
                leftover: None,
                message: Some(MidiMessage::ControlChange14Bit(cc_14_bit_msg)),
            },
        }
    }

    /// Resets the scanners and returns a maybe not yet processed previously-fed payload.
    pub fn reset(&mut self) -> Option<T> {
        self.pn_scanner.reset();
        self.cc_14_bit_scanner.reset();
        self.last_consumed_payload.take()
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct MidiMessageScannerOutcome<S: ShortMessage, T> {
    /// Previously consumed short message that was either superseded by a new one or didn't
    /// end up in a (N)RPN or 14-bit message.
    pub leftover: Option<T>,
    /// The detected message.
    pub message: Option<MidiMessage<S>>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{
        channel as ch, channel, control_change, control_change_14_bit, controller_number as cn,
        controller_number, key_number, u14, u7,
    };
    use crate::{ParameterNumberMessage, RawShortMessage, ShortMessageFactory};

    #[test]
    fn should_detect_short_messages() {
        // Given
        let mut scanner = MidiMessageScanner::<RawShortMessage>::new();
        let msg_1 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x39));
        let msg_2 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x6d));
        let msg_3 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x5a));
        // When
        // Then
        assert_eq!(
            scanner.feed(msg_1, msg_1),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_2, msg_2),
            MidiMessageScannerOutcome {
                leftover: Some(msg_1),
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_3, msg_3),
            MidiMessageScannerOutcome {
                leftover: Some(msg_2),
                message: None,
            }
        );
        assert_eq!(scanner.reset(), Some(msg_3));
    }

    /// Resetting the scanner after the MIDI pulse increases is a common scenario. Because
    /// in saved MIDI sequences, short messages that in combination are a (N)RPN or 14-bit
    /// message, usually must occur on the same MIDI pulse.
    #[test]
    fn should_detect_short_messages_with_reset() {
        // Given
        let mut scanner = MidiMessageScanner::<RawShortMessage>::new();
        let msg_1 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x39));
        let msg_2 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x6d));
        let msg_3 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x5a));
        // When
        // Then
        assert_eq!(
            scanner.feed(msg_1, msg_1),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(scanner.reset(), Some(msg_1));
        assert_eq!(
            scanner.feed(msg_2, msg_2),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(scanner.reset(), Some(msg_2));
        assert_eq!(
            scanner.feed(msg_3, msg_3),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(scanner.reset(), Some(msg_3));
        assert_eq!(scanner.reset(), None);
    }

    #[test]
    fn should_detect_cc_14_bit_messages() {
        // Given
        let mut scanner = MidiMessageScanner::<RawShortMessage>::new();
        let msg_1 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x39));
        let msg_2 = RawShortMessage::control_change(ch(0), controller_number(33), u7(0x6d));
        let msg_3 = RawShortMessage::control_change(ch(0), controller_number(1), u7(0x5a));
        // When
        // Then
        assert_eq!(
            scanner.feed(msg_1, msg_1),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );

        assert_eq!(
            scanner.feed(msg_2, msg_2),
            MidiMessageScannerOutcome {
                leftover: None,
                message: Some(MidiMessage::ControlChange14Bit(control_change_14_bit(
                    0, 1, 7405
                ))),
            }
        );
        assert_eq!(
            scanner.feed(msg_3, msg_3),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(scanner.reset(), Some(msg_3));
    }

    #[test]
    fn should_detect_nrpn_messages() {
        // Given
        let mut scanner = MidiMessageScanner::<RawShortMessage>::new();
        let msg_1 = control_change(0, 101, 3);
        let msg_2 = control_change(0, 100, 36);
        let msg_3 = control_change(0, 38, 24);
        let msg_4 = control_change(0, 8, 5);
        let msg_5 = control_change(0, 6, 117);
        // When
        // Then
        assert_eq!(
            scanner.feed(msg_1, msg_1),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_2, msg_2),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_3, msg_3),
            MidiMessageScannerOutcome {
                leftover: None,
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_4, msg_4),
            MidiMessageScannerOutcome {
                leftover: Some(msg_4),
                message: None,
            }
        );
        assert_eq!(
            scanner.feed(msg_5, msg_5),
            MidiMessageScannerOutcome {
                leftover: None,
                message: Some(MidiMessage::ParameterNumber(
                    ParameterNumberMessage::registered_14_bit(channel(0), u14(420), u14(15000))
                )),
            }
        );
        assert_eq!(scanner.reset(), None);
    }
}
