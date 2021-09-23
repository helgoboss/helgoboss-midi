use crate::{
    controller_numbers, extract_high_7_bit_value_from_14_bit_value,
    extract_low_7_bit_value_from_14_bit_value, Channel, ControllerNumber, ShortMessageFactory, U14,
    U7,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A MIDI Parameter Number message, either registered (RPN) or non-registered (NRPN).
///
/// MIDI systems emit those by sending up to 4 short Control Change messages in a row. The
/// [`ParameterNumberMessageScanner`] can be used to extract such messages from a stream of
/// [`ShortMessage`]s.
///
/// # Example
///
/// ```
/// use helgoboss_midi::{
///     controller_numbers, Channel, ParameterNumberMessage, RawShortMessage, U14,
///     DataEntryByteOrder::MsbFirst
/// };
///
/// let msg =
///     ParameterNumberMessage::registered_14_bit(Channel::new(0), U14::new(420), U14::new(15000));
/// assert_eq!(msg.channel().get(), 0);
/// assert_eq!(msg.number().get(), 420);
/// assert_eq!(msg.value().get(), 15000);
/// assert!(msg.is_registered());
/// assert!(msg.is_14_bit());
/// let short_messages: [Option<RawShortMessage>; 4] = msg.to_short_messages(MsbFirst);
/// use helgoboss_midi::test_util::control_change;
/// assert_eq!(
///     short_messages,
///     [
///         Some(control_change(0, 101, 3)),
///         Some(control_change(0, 100, 36)),
///         Some(control_change(0, 6, 117)),
///         Some(control_change(0, 38, 24)),
///     ]
/// );
/// ```
///
/// [`ShortMessage`]: trait.ShortMessage.html
/// [`ParameterNumberMessageScanner`]: struct.ParameterNumberMessageScanner.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParameterNumberMessage {
    channel: Channel,
    number: U14,
    value: U14,
    is_registered: bool,
    is_14_bit: bool,
    data_type: DataType,
}

impl ParameterNumberMessage {
    /// Creates an NRPN message with a 7-bit data-entry value.
    pub fn non_registered_7_bit(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, false, DataType::DataEntry)
    }

    /// Creates an NRPN message with a 14-bit data-entry value.
    pub fn non_registered_14_bit(
        channel: Channel,
        number: U14,
        value: U14,
    ) -> ParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, false)
    }

    /// Creates an NRPN message with a data decrement value.
    pub fn non_registered_decrement(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, false, DataType::DataDecrement)
    }

    /// Creates an NRPN message with a data increment value.
    pub fn non_registered_increment(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, false, DataType::DataIncrement)
    }

    /// Creates an RPN message with a 7-bit data-entry value.
    pub fn registered_7_bit(channel: Channel, number: U14, value: U7) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, true, DataType::DataEntry)
    }

    /// Creates an RPN message with a 14-bit data-entry value.
    pub fn registered_14_bit(channel: Channel, number: U14, value: U14) -> ParameterNumberMessage {
        Self::fourteen_bit(channel, number, value, true)
    }

    /// Creates an RPN message with a data decrement value.
    pub fn registered_decrement(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, true, DataType::DataDecrement)
    }

    /// Creates an RPN message with a data increment value.
    pub fn registered_increment(
        channel: Channel,
        number: U14,
        value: U7,
    ) -> ParameterNumberMessage {
        Self::seven_bit(channel, number, value, true, DataType::DataIncrement)
    }

    pub(crate) fn seven_bit(
        channel: Channel,
        number: U14,
        value: U7,
        is_registered: bool,
        data_type: DataType,
    ) -> ParameterNumberMessage {
        ParameterNumberMessage {
            channel,
            number,
            value: value.into(),
            is_registered,
            is_14_bit: false,
            data_type,
        }
    }

    pub(crate) fn fourteen_bit(
        channel: Channel,
        number: U14,
        value: U14,
        is_registered: bool,
    ) -> ParameterNumberMessage {
        ParameterNumberMessage {
            channel,
            number,
            value,
            is_registered,
            is_14_bit: true,
            // 14-bit value always means data entry.
            data_type: DataType::DataEntry,
        }
    }

    /// Returns the channel of this message.
    pub fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns the parameter number of this message.
    pub fn number(&self) -> U14 {
        self.number
    }

    /// Returns the value of this message.
    ///
    /// If it's just a 7-bit message, the value is <= 127.
    pub fn value(&self) -> U14 {
        self.value
    }

    /// Returns `true` if this message has a 14-bit value and `false` if only a 7-bit value.
    pub fn is_14_bit(&self) -> bool {
        self.is_14_bit
    }

    /// Returns whether this message uses a registered parameter number.
    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    /// Returns the data type of the value in this message.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Translates this message into up to 4 short Control Change messages, which need to be sent in
    /// a row in order to encode this (N)RPN message.
    ///
    /// If this message has a 14-bit value, all returned short messages are `Some` and the given
    /// data entry byte order is respected. If it has a 7-bit value only, the last short message is
    /// `None`.
    pub fn to_short_messages<T: ShortMessageFactory>(
        &self,
        data_entry_byte_order: DataEntryByteOrder,
    ) -> [Option<T>; 4] {
        use controller_numbers::*;
        let mut messages = [None, None, None, None];
        let mut i = 0;
        // Number MSB
        messages[i] = Some(T::control_change(
            self.channel,
            if self.is_registered {
                REGISTERED_PARAMETER_NUMBER_MSB
            } else {
                NON_REGISTERED_PARAMETER_NUMBER_MSB
            },
            extract_high_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Number LSB
        messages[i] = Some(T::control_change(
            self.channel,
            if self.is_registered {
                REGISTERED_PARAMETER_NUMBER_LSB
            } else {
                NON_REGISTERED_PARAMETER_NUMBER_LSB
            },
            extract_low_7_bit_value_from_14_bit_value(self.number),
        ));
        i += 1;
        // Value bytes
        use DataType::*;
        match self.data_type {
            DataEntry => {
                use DataEntryByteOrder::*;
                match data_entry_byte_order {
                    MsbFirst => {
                        // Value MSB
                        messages[i] = Some(self.build_data_entry_msb_msg());
                        i += 1;
                        // Value LSB
                        if self.is_14_bit {
                            messages[i] = Some(self.build_data_entry_lsb_msg());
                        }
                    }
                    LsbFirst => {
                        // Value LSB
                        if self.is_14_bit {
                            messages[i] = Some(self.build_data_entry_lsb_msg());
                            i += 1;
                        }
                        // Value MSB
                        messages[i] = Some(self.build_data_entry_msb_msg());
                    }
                };
            }
            DataIncrement => {
                messages[i] = Some(self.build_data_inc_dec_msg(controller_numbers::DATA_INCREMENT))
            }
            DataDecrement => {
                messages[i] = Some(self.build_data_inc_dec_msg(controller_numbers::DATA_DECREMENT))
            }
        }
        messages
    }

    fn build_data_entry_msb_msg<T: ShortMessageFactory>(&self) -> T {
        T::control_change(
            self.channel,
            controller_numbers::DATA_ENTRY_MSB,
            if self.is_14_bit {
                extract_high_7_bit_value_from_14_bit_value(self.value)
            } else {
                U7(self.value.get() as u8)
            },
        )
    }

    fn build_data_entry_lsb_msg<T: ShortMessageFactory>(&self) -> T {
        T::control_change(
            self.channel,
            controller_numbers::DATA_ENTRY_MSB_LSB,
            extract_low_7_bit_value_from_14_bit_value(self.value),
        )
    }

    fn build_data_inc_dec_msg<T: ShortMessageFactory>(&self, cn: ControllerNumber) -> T {
        T::control_change(
            self.channel,
            cn,
            extract_low_7_bit_value_from_14_bit_value(self.value),
        )
    }
}

/// The desired byte order of a data entry value.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DataEntryByteOrder {
    /// Most significant byte first.
    MsbFirst,
    /// Least significant byte first.
    LsbFirst,
}

/// Type of the value that is encoded in a parameter number message.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// An absolute value.
    DataEntry,
    /// An increment relative from the current value.
    DataIncrement,
    /// A decrement relative from the current value.
    DataDecrement,
}

impl<T: ShortMessageFactory> From<ParameterNumberMessage> for [Option<T>; 4] {
    fn from(msg: ParameterNumberMessage) -> Self {
        msg.to_short_messages(DataEntryByteOrder::MsbFirst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number as cn, u14, u7};
    use crate::RawShortMessage;

    #[test]
    fn parameter_number_messages_14_bit() {
        // Given
        let msg = ParameterNumberMessage::registered_14_bit(ch(0), u14(420), u14(15000));
        // When
        // Then
        assert_eq!(msg.channel(), ch(0));
        assert_eq!(msg.number(), u14(420));
        assert_eq!(msg.value(), u14(15000));
        assert!(msg.is_14_bit());
        assert!(msg.is_registered());
        assert_eq!(msg.data_type(), DataType::DataEntry);
        let lsb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::LsbFirst);
        assert_eq!(
            lsb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(0), cn(101), u7(3))),
                Some(RawShortMessage::control_change(ch(0), cn(100), u7(36))),
                Some(RawShortMessage::control_change(ch(0), cn(38), u7(24))),
                Some(RawShortMessage::control_change(ch(0), cn(6), u7(117))),
            ]
        );
        let msb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::MsbFirst);
        assert_eq!(
            msb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(0), cn(101), u7(3))),
                Some(RawShortMessage::control_change(ch(0), cn(100), u7(36))),
                Some(RawShortMessage::control_change(ch(0), cn(6), u7(117))),
                Some(RawShortMessage::control_change(ch(0), cn(38), u7(24))),
            ]
        );
    }

    #[test]
    fn parameter_number_messages_7_bit() {
        // Given
        let msg = ParameterNumberMessage::non_registered_7_bit(ch(2), u14(421), u7(126));
        // When
        // Then
        assert_eq!(msg.channel(), ch(2));
        assert_eq!(msg.number(), u14(421));
        assert_eq!(msg.value(), u14(126));
        assert!(!msg.is_14_bit());
        assert!(!msg.is_registered());
        assert_eq!(msg.data_type(), DataType::DataEntry);
        let lsb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::LsbFirst);
        assert_eq!(
            lsb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(2), cn(99), u7(3))),
                Some(RawShortMessage::control_change(ch(2), cn(98), u7(37))),
                Some(RawShortMessage::control_change(ch(2), cn(6), u7(126))),
                None,
            ]
        );
        let msb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::MsbFirst);
        assert_eq!(
            msb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(2), cn(99), u7(3))),
                Some(RawShortMessage::control_change(ch(2), cn(98), u7(37))),
                Some(RawShortMessage::control_change(ch(2), cn(6), u7(126))),
                None,
            ]
        );
    }

    #[test]
    fn parameter_number_messages_increment() {
        // Given
        let msg = ParameterNumberMessage::non_registered_increment(ch(2), u14(421), u7(126));
        // When
        // Then
        assert_eq!(msg.channel(), ch(2));
        assert_eq!(msg.number(), u14(421));
        assert_eq!(msg.value(), u14(126));
        assert!(!msg.is_14_bit());
        assert!(!msg.is_registered());
        assert_eq!(msg.data_type(), DataType::DataIncrement);
        let lsb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::LsbFirst);
        assert_eq!(
            lsb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(2), cn(99), u7(3))),
                Some(RawShortMessage::control_change(ch(2), cn(98), u7(37))),
                Some(RawShortMessage::control_change(ch(2), cn(96), u7(126))),
                None,
            ]
        );
        let msb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::MsbFirst);
        assert_eq!(
            msb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(2), cn(99), u7(3))),
                Some(RawShortMessage::control_change(ch(2), cn(98), u7(37))),
                Some(RawShortMessage::control_change(ch(2), cn(96), u7(126))),
                None,
            ]
        );
    }

    #[test]
    fn parameter_number_messages_decrement() {
        // Given
        let msg = ParameterNumberMessage::registered_decrement(ch(0), u14(420), u7(1));
        // When
        // Then
        assert_eq!(msg.channel(), ch(0));
        assert_eq!(msg.number(), u14(420));
        assert_eq!(msg.value(), u14(1));
        assert!(!msg.is_14_bit());
        assert!(msg.is_registered());
        assert_eq!(msg.data_type(), DataType::DataDecrement);
        let lsb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::LsbFirst);
        assert_eq!(
            lsb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(0), cn(101), u7(3))),
                Some(RawShortMessage::control_change(ch(0), cn(100), u7(36))),
                Some(RawShortMessage::control_change(ch(0), cn(97), u7(1))),
                None,
            ]
        );
        let msb_first_short_msgs: [Option<RawShortMessage>; 4] =
            msg.to_short_messages(DataEntryByteOrder::MsbFirst);
        assert_eq!(
            msb_first_short_msgs,
            [
                Some(RawShortMessage::control_change(ch(0), cn(101), u7(3))),
                Some(RawShortMessage::control_change(ch(0), cn(100), u7(36))),
                Some(RawShortMessage::control_change(ch(0), cn(97), u7(1))),
                None,
            ]
        );
    }
}
