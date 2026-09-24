use crate::{
    Channel, ControlChange14BitMessage, DataEntryByteOrder, ParameterNumberMessage, ShortMessage,
    ShortMessageFactory, U14, U7,
};

/// A MIDI message in the widest sense.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MidiMessage<T: ShortMessage> {
    /// Short MIDI message.
    Short(T),
    /// MIDI Parameter Number message.
    ParameterNumber(ParameterNumberMessage),
    /// 14-bit MIDI Control Change message.
    ControlChange14Bit(ControlChange14BitMessage),
}

impl<T: ShortMessage> MidiMessage<T> {
    /// Returns the channel of this message if applicable.
    pub fn channel(&self) -> Option<Channel> {
        match self {
            MidiMessage::Short(msg) => msg.channel(),
            MidiMessage::ParameterNumber(msg) => Some(msg.channel()),
            MidiMessage::ControlChange14Bit(msg) => Some(msg.channel()),
        }
    }

    /// Returns a new message with the channel replaced.
    ///
    /// Returns `None` if this is not a channel message.
    pub fn with_channel(&self, new_channel: Channel) -> Option<Self>
    where
        T: ShortMessageFactory + Sized + Clone,
    {
        match self {
            Self::Short(msg) => msg.with_channel(new_channel).map(Self::Short),
            Self::ParameterNumber(msg) => Some(Self::ParameterNumber(msg.with_channel(new_channel))),
            Self::ControlChange14Bit(msg) => Some(Self::ControlChange14Bit(msg.with_channel(new_channel))),
        }
    }

    /// Returns the generic number for MIDI messages that have one.
    pub fn number(&self) -> Option<U14> {
        match self {
            MidiMessage::Short(msg) => msg.number().map(|n| n.into()),
            MidiMessage::ParameterNumber(msg) => Some(msg.number()),
            MidiMessage::ControlChange14Bit(msg) => Some(msg.msb_controller_number().get().into()),
        }
    }

    /// Returns the generic value for MIDI messages that have one.
    pub fn value(&self) -> Option<U14> {
        match self {
            MidiMessage::Short(msg) => msg.value(),
            MidiMessage::ParameterNumber(msg) => Some(msg.value()),
            MidiMessage::ControlChange14Bit(msg) => Some(msg.value()),
        }
    }

    /// Returns a new message with the value replaced.
    ///
    /// Returns `None` if this is a message without value or the value is too high.
    pub fn with_value(&self, new_value: U14) -> Option<Self>
    where
        T: ShortMessageFactory + Sized + Clone,
    {
        match self {
            Self::Short(msg) => msg.with_value(new_value).map(Self::Short),
            Self::ParameterNumber(msg) => Some(Self::ParameterNumber(msg.with_value(new_value))),
            Self::ControlChange14Bit(msg) => Some(Self::ControlChange14Bit(msg.with_value(new_value))),
        }
    }

    /// Translates this message into up to 4 short Control Change messages, which need to be sent in
    /// a row in order to encode this message.
    pub fn to_short_messages<R: ShortMessageFactory + Copy>(
        &self,
        pn_data_entry_byte_order: DataEntryByteOrder,
    ) -> [Option<R>; 4] {
        match self {
            MidiMessage::Short(msg) => [Some(R::from_other(msg)), None, None, None],
            MidiMessage::ParameterNumber(msg) => msg.to_short_messages(pn_data_entry_byte_order),
            MidiMessage::ControlChange14Bit(msg) => {
                let short_msgs = msg.to_short_messages::<R>();
                [Some(short_msgs[0]), Some(short_msgs[1]), None, None]
            }
        }
    }
}
