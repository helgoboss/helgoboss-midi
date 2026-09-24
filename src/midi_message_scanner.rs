use crate::{ControlChange14BitMessageScanner, MidiMessage, ParameterNumberMessageScanner, ScanOutcome, ShortMessage};

/// Scanner for detecting potential (N)RPN or 14-bit Control Change messages in a stream of
/// short messages.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MidiMessageScanner {
    pn_scanner: ParameterNumberMessageScanner,
    cc_14_bit_scanner: ControlChange14BitMessageScanner,
}

impl Default for MidiMessageScanner {
    fn default() -> Self {
        Self {
            pn_scanner: Default::default(),
            cc_14_bit_scanner: Default::default(),
        }
    }
}

impl MidiMessageScanner {
    /// Creates a new scanner.
    pub fn new() -> MidiMessageScanner {
        Default::default()
    }

    /// Feeds the scanner a single short message.
    ///
    /// Returns a (N)RPN or 14-bit CC message if one has been detected, otherwise the given short
    /// message. Returns `None` if the message was consumed by the scanner.
    pub fn feed<T: ShortMessage>(&mut self, msg: T) -> Option<MidiMessage<T>> {
        // Try (N)RPN parser
        match self.pn_scanner.feed(&msg) {
            ScanOutcome::Unhandled => {
            }
            ScanOutcome::Consumed => {
                return None;
            }
            ScanOutcome::Complete(pn_msg) => {
                return Some(MidiMessage::ParameterNumber(pn_msg));
            }
        }
        // Try 14-bit CC parser
        match self.cc_14_bit_scanner.feed(&msg) {
            ScanOutcome::Unhandled => {
                // Both unhandled. Return message as is.
                Some(MidiMessage::Short(msg))
            }
            ScanOutcome::Consumed => {
                None
            }
            ScanOutcome::Complete(cc_14_bit_msg) => {
                Some(MidiMessage::ControlChange14Bit(cc_14_bit_msg))
            }
        }
    }
}