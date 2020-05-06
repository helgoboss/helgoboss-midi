use crate::{FromBytesError, ShortMessage, ShortMessageFactory, U7};
use derive_more::Into;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// A short message implemented as a tuple of bytes.
///
/// The struct's size in memory is currently 3 bytes.
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
