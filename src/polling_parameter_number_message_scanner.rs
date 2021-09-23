use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, DataType, ParameterNumberMessage,
    ShortMessage, StructuredShortMessage, U14, U7,
};
use core::time::Duration;
use std::time::Instant;

/// Scanner for detecting (N)RPN messages in a stream of short messages with polling.
///
/// Supports the following message sequences (`x` and `y` represent the bytes that make up the
/// parameter number, MSB represents either a data entry MSB or an increment/decrement value):
///
/// - `[x, y, MSB]`: Interpreted as 7-bit data entry or increment/decrement message.
/// - `[x, y, MSB, LSB]`: Interpreted as 14-bit data entry message.
/// - `[x, y, LSB, MSB]`: Interpreted as 14-bit data entry message.
/// - `[x, y, MSB, MSB, ...]`: Interpreted as 7-bit data entry or increment/decrement messages.
/// - `[x, y, MSB, LSB, MSB, LSB, ...]`: Interpreted as 14-bit data entry messages.
/// - `[x, y, MSB, LSB, LSB, ...]`: Interpreted as 14-bit data entry messages.
///
/// Please note that this requires invoking the [`poll`] method on a regular basis because a
/// timeout is used to wait for potentially relevant messages that might arrive a bit later.
///
/// # Example
///
/// ```
/// use helgoboss_midi::test_util::{control_change, channel, u7, u14};
/// use helgoboss_midi::{ParameterNumberMessage, PollingParameterNumberMessageScanner};
/// use std::time::Duration;
///
/// let mut scanner = PollingParameterNumberMessageScanner::new(Duration::from_millis(0));
///
/// let result_1 = scanner.feed(&control_change(2, 99, 3));
/// let result_2 = scanner.feed(&control_change(2, 98, 37));
/// let result_3 = scanner.feed(&control_change(2, 6, 126));
/// let result_4 = scanner.poll(channel(2));
/// assert_eq!(result_1, [None, None]);
/// assert_eq!(result_2, [None, None]);
/// assert_eq!(result_3, [None, None]);
/// assert_eq!(
///     result_4,
///     Some(ParameterNumberMessage::non_registered_7_bit(
///         channel(2),
///         u14(421),
///         u7(126)
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
    ///
    /// The timeout determines how long to wait for the second value byte.    
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
    /// Returns zero, one or two (N)RPN messages. Two if the scanner was currently waiting for a
    /// data entry LSB (after receiving an MSB) and encountering a data increment or decrement. In
    /// this case we have two complete messages to emit.
    pub fn feed(&mut self, msg: &impl ShortMessage) -> [Option<ParameterNumberMessage>; 2] {
        match msg.channel() {
            None => [None, None],
            Some(channel) => self.scanner_by_channel[usize::from(channel)].feed(msg),
        }
    }

    /// Returns the (N)RPN message as soon as the timeout of waiting for the second value message
    /// has been exceeded.
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
    /// As soon as the first data entry value byte arrived.
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

    fn process_value_byte_when_waiting_for_value(
        &self,
        byte: U7,
        is_msb: bool,
    ) -> Res<Option<ParameterNumberMessage>> {
        // This is the first arriving value byte. Wait for next one.
        Res {
            next_state: State::ValuePending(ValuePendingState {
                number_state: *self,
                arrival_time: Instant::now(),
                first_value_byte: byte,
                is_msb,
            }),
            result: None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct ValuePendingState {
    number_state: NumberState,
    arrival_time: Instant,
    first_value_byte: U7,
    is_msb: bool,
}

impl ValuePendingState {
    fn resolve(&self, channel: Channel) -> Option<ParameterNumberMessage> {
        if self.is_msb {
            // [x, y, MSB]
            // We were waiting for a remaining LSB but none arrived. 7-bit!
            Some(ParameterNumberMessage::seven_bit(
                channel,
                self.number_state.number(),
                self.first_value_byte,
                self.number_state.is_registered,
                DataType::DataEntry,
            ))
        } else {
            // [x, y, LSB]
            // We were waiting for a remaining MSB but none arrived. Invalid.
            None
        }
    }

    fn process_expected_value_byte_when_pending(
        &self,
        channel: Channel,
        byte: U7,
    ) -> Res<Option<ParameterNumberMessage>> {
        let value_msb = if self.is_msb {
            self.first_value_byte
        } else {
            byte
        };
        let value_lsb = if self.is_msb {
            byte
        } else {
            self.first_value_byte
        };
        Res {
            next_state: State::FourteenBitValueComplete(FourteenBitValueCompleteState {
                number_state: self.number_state,
                value_msb,
                value_lsb,
            }),
            result: Some(ParameterNumberMessage::fourteen_bit(
                channel,
                self.number_state.number(),
                build_14_bit_value_from_two_7_bit_values(value_msb, value_lsb),
                self.number_state.is_registered,
            )),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct FourteenBitValueCompleteState {
    number_state: NumberState,
    value_msb: U7,
    value_lsb: U7,
}

struct Res<T> {
    next_state: State,
    result: T,
}

impl ScannerForOneChannel {
    pub fn feed(&mut self, msg: &impl ShortMessage) -> [Option<ParameterNumberMessage>; 2] {
        match msg.to_structured() {
            StructuredShortMessage::ControlChange {
                channel,
                controller_number,
                control_value,
            } => match controller_number.get() {
                98 => [self.process_number_lsb(control_value, false, channel), None],
                99 => [self.process_number_msb(control_value, false, channel), None],
                100 => [self.process_number_lsb(control_value, true, channel), None],
                101 => [self.process_number_msb(control_value, true, channel), None],
                38 => [self.process_value_lsb(channel, control_value), None],
                6 => [self.process_value_msb(channel, control_value), None],
                96 => self.process_value_inc_dec(channel, DataType::DataIncrement, control_value),
                97 => self.process_value_inc_dec(channel, DataType::DataDecrement, control_value),
                _ => [None, None],
            },
            _ => [None, None],
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
            Res {
                next_state: State::WaitingForFirstValueByte(state.number_state),
                result: state.resolve(channel),
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
        channel: Channel,
    ) -> Option<ParameterNumberMessage> {
        self.process_number_byte(number_msb, is_registered, true, channel)
    }

    fn process_number_lsb(
        &mut self,
        number_lsb: U7,
        is_registered: bool,
        channel: Channel,
    ) -> Option<ParameterNumberMessage> {
        self.process_number_byte(number_lsb, is_registered, false, channel)
    }

    fn process_number_byte(
        &mut self,
        byte: U7,
        is_registered: bool,
        is_msb: bool,
        channel: Channel,
    ) -> Option<ParameterNumberMessage> {
        use State::*;
        let res = match &self.state {
            WaitingForNumberCompletion(state) => {
                if let Some(state_byte) = state.first_number_byte {
                    // We received one byte already.
                    if state.is_msb == is_msb {
                        // Overwrite already existing byte.
                        Res {
                            next_state: WaitingForNumberCompletion(
                                WaitingForNumberCompletionState {
                                    first_number_byte: Some(byte),
                                    is_registered,
                                    is_msb,
                                },
                            ),
                            result: None,
                        }
                    } else {
                        // Number complete.
                        Res {
                            next_state: WaitingForFirstValueByte(NumberState {
                                msb: if state.is_msb { state_byte } else { byte },
                                lsb: if state.is_msb { byte } else { state_byte },
                                is_registered,
                            }),
                            result: None,
                        }
                    }
                } else {
                    // This is the first byte.
                    Res {
                        next_state: WaitingForNumberCompletion(WaitingForNumberCompletionState {
                            first_number_byte: Some(byte),
                            is_registered,
                            is_msb,
                        }),
                        result: None,
                    }
                }
            }
            WaitingForFirstValueByte(state)
            | FourteenBitValueComplete(FourteenBitValueCompleteState {
                number_state: state,
                ..
            }) => {
                // No pending value, everything already delivered. Change number and reset value.
                Res {
                    next_state: WaitingForFirstValueByte(NumberState {
                        lsb: if is_msb { state.lsb } else { byte },
                        msb: if is_msb { byte } else { state.msb },
                        is_registered,
                    }),
                    result: None,
                }
            }
            ValuePending(state) => {
                // Pending value. Deliver, change number, reset value.
                Res {
                    next_state: WaitingForFirstValueByte(NumberState {
                        lsb: if is_msb { state.number_state.lsb } else { byte },
                        msb: if is_msb { byte } else { state.number_state.msb },
                        is_registered,
                    }),
                    result: state.resolve(channel),
                }
            }
        };
        self.state = res.next_state;
        res.result
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
                state.process_value_byte_when_waiting_for_value(value_lsb, false)
            }
            ValuePending(state) => {
                if state.is_msb {
                    // We were waiting exactly for this byte. The value is complete!
                    state.process_expected_value_byte_when_pending(channel, value_lsb)
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
                state.process_value_byte_when_waiting_for_value(value_msb, true)
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
                            DataType::DataEntry,
                        )),
                    }
                } else {
                    // We were waiting exactly for this byte. The value is complete!
                    state.process_expected_value_byte_when_pending(channel, value_msb)
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

    fn process_value_inc_dec(
        &mut self,
        channel: Channel,
        data_type: DataType,
        value: U7,
    ) -> [Option<ParameterNumberMessage>; 2] {
        use State::*;
        let res = match &self.state {
            WaitingForNumberCompletion(_) => {
                // Invalid. Ignore.
                return [None, None];
            }
            WaitingForFirstValueByte(state) => {
                let msg = ParameterNumberMessage::seven_bit(
                    channel,
                    state.number(),
                    value,
                    state.is_registered,
                    data_type,
                );
                Res {
                    next_state: WaitingForFirstValueByte(*state),
                    result: [Some(msg), None],
                }
            }
            ValuePending(state) => {
                if state.is_msb {
                    // We were waiting for an LSB but an inc/dec arrived. That means we are looking
                    // at a complete 7-bit (N)RPN data entry message plus an already complete (N)RPN
                    // data inc/dec message. Two messages at once!
                    Res {
                        next_state: WaitingForFirstValueByte(state.number_state),
                        result: [
                            Some(ParameterNumberMessage::seven_bit(
                                channel,
                                state.number_state.number(),
                                state.first_value_byte,
                                state.number_state.is_registered,
                                DataType::DataEntry,
                            )),
                            Some(ParameterNumberMessage::seven_bit(
                                channel,
                                state.number_state.number(),
                                value,
                                state.number_state.is_registered,
                                data_type,
                            )),
                        ],
                    }
                } else {
                    // We were waiting for the MSB but an inc/dec arrived. This is invalid. Start
                    // waiting for value again.
                    Res {
                        next_state: WaitingForFirstValueByte(state.number_state),
                        result: [None, None],
                    }
                }
            }
            FourteenBitValueComplete(state) => {
                // This seems to be an inc/dec with the same parameter number as before.
                let msg = ParameterNumberMessage::seven_bit(
                    channel,
                    state.number_state.number(),
                    value,
                    state.number_state.is_registered,
                    data_type,
                );
                Res {
                    next_state: WaitingForFirstValueByte(state.number_state),
                    result: [Some(msg), None],
                }
            }
        };
        self.state = res.next_state;
        res.result
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
            [None, None]
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::note_on(ch(0), key_number(100), u7(120))),
            [None, None]
        );
        assert_eq!(
            scanner.feed(&RawShortMessage::control_change(ch(0), cn(80), u7(1))),
            [None, None]
        );
    }

    #[test]
    fn x_y_msb_entry() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
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
    fn x_y_msb_increment() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(96), u7(126)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(
            result_3,
            [
                Some(ParameterNumberMessage::non_registered_increment(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(result_4, None);
    }

    #[test]
    fn x_y_msb_decrement() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(97), u7(126)));
        let result_4 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(
            result_3,
            [
                Some(ParameterNumberMessage::non_registered_decrement(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(result_4, None);
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(result_4, None);
    }

    #[test]
    fn x_y_msb_msb_entry() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(127)));
        let result_5 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::non_registered_7_bit(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
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
    fn x_y_msb_msb_increment() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(96), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(96), u7(127)));
        let result_5 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(
            result_3,
            [
                Some(ParameterNumberMessage::non_registered_increment(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::non_registered_increment(
                    ch(2),
                    u14(421),
                    u7(127)
                )),
                None
            ]
        );
        assert_eq!(result_5, None);
    }

    #[test]
    fn x_y_msb_msb_decrement() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(97), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(97), u7(127)));
        let result_5 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(
            result_3,
            [
                Some(ParameterNumberMessage::non_registered_decrement(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::non_registered_decrement(
                    ch(2),
                    u14(421),
                    u7(127)
                )),
                None
            ]
        );
        assert_eq!(result_5, None);
    }

    #[test]
    fn x_y_msb_msb_entry_inc_dec() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(125)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(96), u7(126)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(97), u7(5)));
        let result_7 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::non_registered_7_bit(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(
            result_5,
            [
                Some(ParameterNumberMessage::non_registered_7_bit(
                    ch(2),
                    u14(421),
                    u7(125)
                )),
                Some(ParameterNumberMessage::non_registered_increment(
                    ch(2),
                    u14(421),
                    u7(126)
                ))
            ]
        );
        assert_eq!(
            result_6,
            [
                Some(ParameterNumberMessage::non_registered_decrement(
                    ch(2),
                    u14(421),
                    u7(5)
                )),
                None
            ]
        );
        assert_eq!(result_7, None);
    }

    #[test]
    fn x_y_msb_poll_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.poll(ch(2));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(127)));
        let result_6 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(126)
            ))
        );
        assert_eq!(result_5, [None, None]);
        assert_eq!(
            result_6,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(127)
            ))
        );
    }

    #[test]
    fn x_y_msb_x_y_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(126)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(99), u7(3)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(98), u7(37)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(2), cn(6), u7(125)));
        let result_7 = scanner.poll(ch(2));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::non_registered_7_bit(
                    ch(2),
                    u14(421),
                    u7(126)
                )),
                None
            ]
        );
        assert_eq!(result_5, [None, None]);
        assert_eq!(result_6, [None, None]);
        assert_eq!(
            result_7,
            Some(ParameterNumberMessage::non_registered_7_bit(
                ch(2),
                u14(421),
                u7(125)
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
        );
        assert_eq!(result_5, [None, None]);
        assert_eq!(
            result_6,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(14999)
                )),
                None
            ]
        );
    }

    #[test]
    fn x_y_msb_lsb_x_y_msb_lsb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_7 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_8 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(23)));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
        );
        assert_eq!(result_5, [None, None]);
        assert_eq!(result_6, [None, None]);
        assert_eq!(result_7, [None, None]);
        assert_eq!(
            result_8,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(14999)
                )),
                None
            ]
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
        );
        assert_eq!(
            result_5,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15001)
                )),
                None
            ]
        );
    }

    #[test]
    fn x_y_lsb_msb_x_y_lsb_msb() {
        // Given
        let mut scanner = PollingParameterNumberMessageScanner::default();
        // When
        let result_1 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_2 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_3 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(24)));
        let result_4 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        let result_5 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(101), u7(3)));
        let result_6 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(100), u7(36)));
        let result_7 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(38), u7(23)));
        let result_8 = scanner.feed(&RawShortMessage::control_change(ch(0), cn(6), u7(117)));
        // Then
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(
            result_4,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
        );
        assert_eq!(result_5, [None, None]);
        assert_eq!(result_6, [None, None]);
        assert_eq!(result_7, [None, None]);
        assert_eq!(
            result_8,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(14999)
                )),
                None
            ]
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_3, [None, None]);
        assert_eq!(result_5, [None, None]);
        assert_eq!(
            result_7,
            [
                Some(ParameterNumberMessage::registered_14_bit(
                    ch(0),
                    u14(420),
                    u14(15000)
                )),
                None
            ]
        );
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_4, [None, None]);
        assert_eq!(result_6, [None, None]);
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
        assert_eq!(result_1, [None, None]);
        assert_eq!(result_2, [None, None]);
        assert_eq!(result_3, [None, None]);
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
