use crate::{FromBytesError, ShortMessage, ShortMessageFactory, U7};
use derive_more::Into;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// A short message implemented as a tuple of bytes.
///
/// The struct's size in memory is currently 3 bytes.
///
/// # Example
///
/// ```
/// use helgoboss_midi::{
///     Channel, KeyNumber, MessageMainCategory, MessageSuperType, RawShortMessage, ShortMessage,
///     ShortMessageFactory, ShortMessageType, U7,
/// };
///
/// let msg = RawShortMessage::note_on(Channel::new(5), KeyNumber::new(64), U7::new(123));
/// assert_eq!(std::mem::size_of_val(&msg), 3);
/// assert_eq!(msg.status_byte(), 149);
/// assert_eq!(msg.data_byte_1().get(), 64);
/// assert_eq!(msg.data_byte_2().get(), 123);
/// assert_eq!(msg.r#type(), ShortMessageType::NoteOn);
/// assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
/// assert_eq!(msg.main_category(), MessageMainCategory::Channel);
/// assert_eq!(msg.channel(), Some(Channel::new(5)));
/// assert_eq!(msg.key_number(), Some(KeyNumber::new(64)));
/// assert_eq!(msg.velocity(), Some(U7::new(123)));
/// assert_eq!(msg.controller_number(), None);
/// assert_eq!(msg.control_value(), None);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Into)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RawShortMessage((u8, U7, U7));

impl ShortMessageFactory for RawShortMessage {
    unsafe fn from_bytes_unchecked(bytes: (u8, U7, U7)) -> Self {
        Self(bytes)
    }
}

impl TryFrom<(u8, U7, U7)> for RawShortMessage {
    type Error = FromBytesError;

    fn try_from(value: (u8, U7, U7)) -> Result<Self, Self::Error> {
        RawShortMessage::from_bytes(value)
    }
}

impl ShortMessage for RawShortMessage {
    fn status_byte(&self) -> u8 {
        (self.0).0
    }

    fn data_byte_1(&self) -> U7 {
        (self.0).1
    }

    fn data_byte_2(&self) -> U7 {
        (self.0).2
    }
}
