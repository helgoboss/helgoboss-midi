use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, ParameterNumberMessage, ShortMessage,
    StructuredShortMessage, U14, U7,
};
use std::time::{Duration, Instant};

/// Scanner for detecting (N)RPN messages in a stream of short messages with polling.
///
/// Supports the following message sequences (`x` and `y` represent the bytes that make up the
/// parameter number):
///
/// - `[x, y, MSB]`: Interpreted as 7-bit message.
/// - `[x, y, MSB, LSB]`: Interpreted as 14-bit message.
/// - `[x, y, LSB, MSB]`: Interpreted as 14-bit message.
/// - `[x, y, MSB, MSB, ...]`: Interpreted as 7-bit messages.
/// - `[x, y, MSB, LSB, MSB, LSB, ...]`: Interpreted as 14-bit messages.
/// - `[x, y, MSB, LSB, LSB, ...]`: Interpreted as 14-bit messages.
///
/// Please note that this requires invoking the [`poll`] method on a regular basis because a
/// timeout is used to wait for potentially relevant messages that might arrive a bit later.
///
/// # Example
///
/// ```
/// use helgoboss_midi::test_util::control_change;
/// use helgoboss_midi::{
///     Channel, ControllerNumber, ParameterNumberMessage, PollingParameterNumberMessageScanner, U7,
///     U14,
/// };
/// use std::time::Duration;
///
/// let mut scanner = PollingParameterNumberMessageScanner::new(Duration::from_millis(0));
///
/// let result_1 = scanner.feed(&control_change(2, 99, 3));
/// let result_2 = scanner.feed(&control_change(2, 98, 37));
/// let result_3 = scanner.feed(&control_change(2, 6, 126));
/// let result_4 = scanner.poll(ch(2));
/// assert_eq!(result_1, None);
/// assert_eq!(result_2, None);
/// assert_eq!(result_3, None);
/// assert_eq!(
///     result_4,
///     Some(ParameterNumberMessage::non_registered_7_bit(
///         Channel::new(2),
///         U14::new(421),
///         U7::new(126)
///     ))
/// );
/// ```
///
/// [`poll`]: struct.PollingParameterNumberMessageScanner.html#method.poll
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct PollingParameterNumberMessageScanner {
    scanner_by_channel: [ScannerForOneChannel; 16],
}

impl PollingParameterNumberMessageScanner {
    /// Creates a new scanner.
    pub fn new(timeout: Duration) -> PollingParameterNumberMessageScanner {
        let channel_scanner = ScannerForOneChannel {
            timeout,
            ..Default::default()
        };
        Self {
            scanner_by_channel: [channel_scanner; 16],
        }
    }

    /// Feeds the scanner a single short message.
    ///
    /// Returns the (N)RPN message if one has been detected.
    pub fn feed(&mut self, msg: &impl ShortMessage) -> Option<ParameterNumberMessage> {
        let channel = msg.channel()?;
        self.scanner_by_channel[usize::from(channel)].feed(msg)
    }

    /// Returns the (N)RPN message as soon as the timeout of waiting for the second value message
    /// has been exceeded.
    ///
    /// Only applicable to scan mode [`LsbOrMsbFirst`].
    ///
    /// [`LsbOrMsbFirst`]: enum.ValueScanMode.html#variant.LsbOrMsbFirst
    pub fn poll(&mut self, channel: Channel) -> Option<ParameterNumberMessage> {
        self.scanner_by_channel[usize::from(channel)].poll(channel)
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
    timeout: Duration,
    state: State,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    /// As long as the parameter number is not yet complete.
    ///
    /// Doesn't deal with values yet.
    WaitingForNumberCompletion(WaitingForNumberCompletionState),
    /// As soon as the number is complete.
    WaitingForFirstValueByte(NumberState),
    /// As soon as the first value byte arrived.
    ValuePending(ValuePendingState),
    /// The sequence is complete already.
    FourteenBitValueComplete(FourteenBitValueCompleteState),
}

impl Default for State {
    fn default() -> Self {
        Self::WaitingForNumberCompletion(Default::default())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
struct WaitingForNumberCompletionState {
    first_number_byte: Option<U7>,
    is_registered: bool,
    is_msb: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct NumberState {
    msb: U7,
    lsb: U7,
    is_registered: bool,
}

impl NumberState {
    fn number(&self) -> U14 {
        build_14_bit_value_from_two_7_bit_values(self.msb, self.lsb)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct ValuePendingState {
    number_state: NumberState,
    arrival_time: Instant,
    first_value_byte: U7,
    is_msb: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct FourteenBitValueCompleteState {
    number_state: NumberState,
    value_msb: U7,
    value_lsb: U7,
}

struct Res {
    next_state: State,
    result: Option<ParameterNumberMessage>,
}

impl ScannerForOneChannel {
    pub fn feed(&mut self, msg: &impl ShortMessage) -> Option<ParameterNumberMessage> {
        match msg.to_structured() {
            StructuredShortMessage::ControlChange {
                channel,
                controller_number,
                control_value,
            } => match controller_number.get() {
                98 => self.process_number_lsb(control_value, false),
                99 => self.process_number_msb(control_value, false),
                100 => self.process_number_lsb(control_value, true),
                101 => self.process_number_msb(control_value, true),
                38 => self.process_value_lsb(channel, control_value),
                6 => self.process_value_msb(channel, control_value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn poll(&mut self, channel: Channel) -> Option<ParameterNumberMessage> {
        let res = {
            let state = if let State::ValuePending(s) = &self.state {
                s
            } else {
                return None;
            };
            if state.arrival_time.elapsed() < self.timeout {
                return None;
            }
            if state.is_msb {
                // [x, y, MSB]
                // We were waiting for a remaining LSB but none arrived. 7-bit!
                Res {
                    next_state: Default::default(),
                    result: Some(ParameterNumberMessage::seven_bit(
                        channel,
                        state.number_state.number(),
                        state.first_value_byte,
                        state.number_state.is_registered,
                    )),
                }
            } else {
                // [x, y, LSB]
                // We were waiting for a remaining MSB but none arrived. Invalid. Start waiting
                // for value again.
                Res {
                    next_state: State::WaitingForFirstValueByte(state.number_state),
                    result: None,
                }
            }
        };
        self.state = res.next_state;
        res.result
    }

    pub fn reset(&mut self) {
        self.state = Default::default();
    }

    fn process_number_msb(
        &mut self,
        number_msb: U7,
        is_registered: bool,
    ) -> Option<ParameterNumberMessage> {
        self.process_number_byte(number_msb, is_registered, true)
    }

    fn process_number_lsb(
        &mut self,
        number_lsb: U7,
        is_registered: bool,
    ) -> Option<ParameterNumberMessage> {
        self.process_number_byte(number_lsb, is_registered, false)
    }

    fn process_number_byte(
        &mut self,
        byte: U7,
        is_registered: bool,
        is_msb: bool,
    ) -> Option<ParameterNumberMessage> {
        use State::*;
        let next_state = match &self.state {
            WaitingForNumberCompletion(state) => {
                if let Some(state_byte) = state.first_number_byte {
                    // We received one byte already.
                    if state.is_msb == is_msb {
                        // Overwrite already existing byte.
                        WaitingForNumberCompletion(WaitingForNumberCompletionState {
                            first_number_byte: Some(byte),
                            is_registered,
                            is_msb,
                        })
                    } else {
                        // Number complete.
                        WaitingForFirstValueByte(NumberState {
                            msb: if state.is_msb { state_byte } else { byte },
                            lsb: if state.is_msb { byte } else { state_byte },
                            is_registered,
                        })
                    }
                } else {
                    // This is the first byte.
                    WaitingForNumberCompletion(WaitingForNumberCompletionState {
                        first_number_byte: Some(byte),
                        is_registered,
                        is_msb,
                    })
                }
            }
            WaitingForFirstValueByte(state)
            | ValuePending(ValuePendingState {
                number_state: state,
                ..
            })
            | FourteenBitValueComplete(FourteenBitValueCompleteState {
                number_state: state,
                ..
            }) => {
                // Change number and reset value.
                WaitingForFirstValueByte(NumberState {
                    lsb: if is_msb { state.lsb } else { byte },
                    msb: if is_msb { byte } else { state.lsb },
                    is_registered,
                })
            }
        };
        self.state = next_state;
        None
    }

    fn process_value_lsb(
        &mut self,
        channel: Channel,
        value_lsb: U7,
    ) -> Option<ParameterNumberMessage> {
        use State::*;
        let res = match &self.state {
            WaitingForNumberCompletion(_) => {
                // Invalid. Ignore.
                return None;
            }
            WaitingForFirstValueByte(state) => {
                process_value_byte_when_waiting_for_value(state, value_lsb, false)
            }
            ValuePending(state) => {
                if state.is_msb {
                    // We were waiting exactly for this byte. The value is complete!
                    process_expected_value_byte_when_pending(state, channel, value_lsb)
                } else {
                    // We were waiting for the MSB but another LSB arrived. This is invalid. Start
                    // waiting for value again.
                    Res {
                        next_state: WaitingForFirstValueByte(state.number_state),
                        result: None,
                    }
                }
            }
            FourteenBitValueComplete(state) => {
                // Value was already complete. This is a fine adjustment.
                Res {
                    next_state: FourteenBitValueComplete(FourteenBitValueCompleteState {
                        value_lsb,
                        ..*state
                    }),
                    result: Some(ParameterNumberMessage::fourteen_bit(
                        channel,
                        state.number_state.number(),
                        build_14_bit_value_from_two_7_bit_values(state.value_msb, value_lsb),
                        state.number_state.is_registered,
                    )),
                }
            }
        };
        self.state = res.next_state;
        res.result
    }

    fn process_value_msb(
        &mut self,
        channel: Channel,
        value_msb: U7,
    ) -> Option<ParameterNumberMessage> {
        use State::*;
        let res = match &self.state {
            WaitingForNumberCompletion(_) => {
                // Invalid. Ignore.
                return None;
            }
            WaitingForFirstValueByte(state) => {
                process_value_byte_when_waiting_for_value(state, value_msb, true)
            }
            ValuePending(state) => {
                if state.is_msb {
                    // We were waiting for an LSB but another MSB arrived. That means we are looking
                    // at a complete 7-bit (N)RPN message and received already the beginning of the
                    // next one!
                    Res {
                        next_state: ValuePending(ValuePendingState {
                            number_state: state.number_state,
                            arrival_time: Instant::now(),
                            first_value_byte: value_msb,
                            is_msb: true,
                        }),
                        result: Some(ParameterNumberMessage::seven_bit(
                            channel,
                            state.number_state.number(),
                            state.first_value_byte,
                            state.number_state.is_registered,
                        )),
                    }
                } else {
                    // We were waiting exactly for this byte. The value is complete!
                    process_expected_value_byte_when_pending(state, channel, value_msb)
                }
            }
            FourteenBitValueComplete(state) => {
                // This seems to be the beginning of a new [MSB, LSB] sequence with the same
                // parameter number as before.
                Res {
                    next_state: ValuePending(ValuePendingState {
                        number_state: state.number_state,
                        arrival_time: Instant::now(),
                        first_value_byte: value_msb,
                        is_msb: true,
                    }),
                    result: None,
                }
            }
        };
        self.state = res.next_state;
        res.result
    }
}

fn process_value_byte_when_waiting_for_value(state: &NumberState, byte: U7, is_msb: bool) -> Res {
    // This is the first arriving value byte. Wait for next one.
    Res {
        next_state: State::ValuePending(ValuePendingState {
            number_state: *state,
            arrival_time: Instant::now(),
            first_value_byte: byte,
            is_msb,
        }),
        result: None,
    }
}

fn process_expected_value_byte_when_pending(
    state: &ValuePendingState,
    channel: Channel,
    byte: U7,
) -> Res {
    let value_msb = if state.is_msb {
        state.first_value_byte
    } else {
        byte
    };
    let value_lsb = if state.is_msb {
        byte
    } else {
        state.first_value_byte
    };
    Res {
        next_state: State::FourteenBitValueComplete(FourteenBitValueCompleteState {
            number_state: state.number_state,
            value_msb,
            value_lsb,
        }),
        result: Some(ParameterNumberMessage::fourteen_bit(
            channel,
            state.number_state.number(),
            build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb),
            state.number_state.is_registered,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, key_number, u14, u7};
    use crate::{RawShortMessage, ShortMessageFactory};

    #[test]
    fn should_ignore_non_contributing_short_messages() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        // Then
        assert_eq!(
            scanner.feed(&RawShortMessage::note_on(ch(0), key_number(100), u7(100))),
            None
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::note_on(ch(0), key_number(100), u7(120))),
            None
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::control_change(ch(0), cn(80), u7(1))),
            None
        );
    }

    #[test]
    fn x_y_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(126)
            ))
        );
    }

    #[test]
    fn x_y_msb_lsb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15000)
            ))
        );
    }

    #[test]
    fn x_y_lsb_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15000)
            ))
        );
    }

    #[test]
    fn x_y_lsb_invalid() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(38), u7(24)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(result_4, None);
    }

    #[test]
    fn x_y_msb_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(127)));
        let result_5 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(126)
            ))
        );
        assert_eq!(
            result_5,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(127)
            ))
        );
    }

    #[test]
    fn x_y_msb_lsb_msb_lsb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(23)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15000)
            ))
        );
        assert_eq!(result_5, None);
        assert_eq!(
            result_6,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(14999)
            ))
        );
    }

    #[test]
    fn x_y_msb_lsb_lsb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(25)));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15000)
            ))
        );
        assert_eq!(
            result_5,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15001)
            ))
        );
    }

    #[test]
    fn should_process_different_channels_independently() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_7 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_8 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_3, None);
        assert_eq!(result_5, None);
        assert_eq!(
            result_7,
            Some(ParameterNumberMessage::registered_14_bit(
                ch(0),
                u14(420),
                u14(15000)
            ))
        );
        assert_eq!(result_2, None);
        assert_eq!(result_4, None);
        assert_eq!(result_6, None);
        assert_eq!(
            result_8,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(126)
            ))
        );
    }

    #[test]
    fn should_ignore_non_contributing_short_messages_mixed() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        scanner.feed(&RawShortMessage::control_change(ch(2), cn(34), u7(5)));
        scanner.feed(&RawShortMessage::note_on(ch(2), key_number(100), u7(105)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        scanner.feed(&RawShortMessage::control_change(ch(2), cn(50), u7(6)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, None);
        assert_eq!(result_2, None);
        assert_eq!(result_3, None);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(126)
            ))
        );
    }
}
