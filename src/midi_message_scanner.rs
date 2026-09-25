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
                }
            }
            ScanOutcome::Complete(pn_msg) => {
                return MidiMessageScannerOutcome {
                    leftover: None,
                    message: Some(MidiMessage::ParameterNumber(pn_msg))
                }
            }
        }
        // Try 14-bit CC parser
        match self.cc_14_bit_scanner.feed(&msg) {
            ScanOutcome::Unhandled => {
                // Both unhandled. Return message as is.
                MidiMessageScannerOutcome {
                    leftover: last_consumed_payload,
                    message: Some(MidiMessage::Short(msg))
                }
            }
            ScanOutcome::Pending => {
                self.last_consumed_payload = Some(payload);
                MidiMessageScannerOutcome {
                    leftover: last_consumed_payload,
                    message: None,
                }
            }
            ScanOutcome::Complete(cc_14_bit_msg) => {
                MidiMessageScannerOutcome {
                    leftover: None,
                    message: Some(MidiMessage::ControlChange14Bit(cc_14_bit_msg)),
                }
            }
        }
    }

    /// Resets the scanners and returns a maybe not yet processed previously-fed payload.
    pub fn reset(&mut self) -> Option<T> {
        self.pn_scanner.reset();
        self.cc_14_bit_scanner.reset();
        self.last_consumed_payload.take()
    }
}


pub struct MidiMessageScannerOutcome<S: ShortMessage, T> {
    /// Previously consumed short message that was either superseded by a new one or didn't
    /// end up in a (N)RPN or 14-bit message.
    pub leftover: Option<T>,
    /// The detected message.
    pub message: Option<MidiMessage<S>>,
}