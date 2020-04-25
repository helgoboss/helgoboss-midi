use crate::{
    build_14_bit_value_from_two_7_bit_values, extract_channel_from_status_byte, Channel,
    ControllerNumber, KeyNumber, ShortMessageFactory, StructuredShortMessage, U14, U4, U7,
};
use derive_more::{Display, Error};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::{TryFrom, TryInto};

/// A single short MIDI message, where *short* means it's made up by a maximum of 3 bytes.
///
/// This trait is supposed to be implemented for structs that represent a short MIDI message. Only
/// the three byte-returning methods need to be implemented, the rest is done by default methods.
/// Optimizations can be applied by overwriting default methods.
///
/// Please also implement the trait [`ShortMessageFactory`] for your struct if creating new short
/// messages should be supported.
///
/// # Design
///
/// ## Why a trait and not just a data structure?
///
/// The advantage of using a trait is that a unified API can be used to work with short MIDI
/// messages regardless of the underlying data structure. This crate comes with two implementations:
/// the "no-fuzz" byte-based data structure [`RawShortMessage`] and the match-friendly data
/// structure [`StructuredShortMessage`]. Each one has its own subtle strengths, yet both are really
/// just MIDI messages and should have the same capabilities. This fact is reflected by having a
/// common trait.
///
/// If there wouldn't be a trait, we would probably keep the [`StructuredShortMessage`]
/// implementation and ditch [`RawShortMessage`]. But this would come at a small cost. In real-world
/// applications, short MIDI messages are constructed from raw bytes. When creating a
/// [`StructuredShortMessage`] from raw bytes, a small amount of conversion is required - even if
/// the consumer doesn't need the matching capabilities at all. With [`RawShortMessage`], it's just
/// a matter of copying the bytes. The conversion can happen at a later point when we have to
/// inspect the message. However, keeping only [`RawShortMessage`] is also not a good idea because
/// then we lose the matching capabilities.
///
/// Another benefit is flexibility. We might have an existing struct (e.g. an FFI struct) which
/// already represents a short MIDI message. Okay, in this case we could just eagerly copy those 3
/// bytes to a [`RawShortMessage`], that's cheap. But what if that existing struct represents more
/// than a short MIDI message, e.g. it also supports System Exclusive messages or carries MIDI
/// event information such as a frame? Then simply copying it a dozen times can decrease
/// performance. So we might want to pass it around as a reference instead. Then we can just
/// implement the trait for it and it will look and behave like a short MIDI message.
///
/// ## Why doesn't this trait support System Exclusive messages?
///
/// This trait is not designed to represent messages that are longer than 3 bytes, such as complete
/// System Exclusive messages. One benefit is that implementations of this trait can easily get by
/// without doing heap allocations. This is important because MIDI messages are often processed in a
/// real-time thread where things need to happen fast and heap allocations are a no-go. Also,
/// implementations can be made copyable just by deriving `Copy`, which is essential for passing
/// around messages by copying rather then dealing with references.
///
/// For the majority of use cases, System Exclusive messages are not necessary. Support for them can
/// be built as a separate data structure on top of this trait and will probably added to
/// `helgoboss-midi` in future.
///
/// ## Why doesn't this trait support 14-bit Control Change or (N)RPN messages?
///
/// This trait is not used to represent MIDI messages made up by multiple short messages, such as
/// (N)RPN messages. Those are implemented in separate structs in order to follow the
/// single-responsibility principle.
///
/// [`ShortMessageFactory`]: trait.ShortMessageFactory.html
/// [`RawShortMessage`]: struct.RawShortMessage.html
/// [`StructuredShortMessage`]: enum.StructuredShortMessage.html
pub trait ShortMessage {
    /// Returns the status byte.
    fn status_byte(&self) -> u8;

    /// Returns the first data byte.
    fn data_byte_1(&self) -> U7;

    /// Returns the second data byte.
    fn data_byte_2(&self) -> U7;

    /// Returns the status byte and the two data bytes as a tuple.
    ///
    /// Implementations can override this default implementation if it's cheaper to get all bytes
    /// in one go.
    fn to_bytes(&self) -> (u8, U7, U7) {
        (self.status_byte(), self.data_byte_1(), self.data_byte_2())
    }

    /// Converts this message to a short message of another type.
    fn to_other<O: ShortMessageFactory>(&self) -> O {
        let bytes = self.to_bytes();
        unsafe { O::from_bytes_unchecked(bytes) }
    }

    /// Converts this message to a [`StructuredShortMessage`], which is ideal for matching.
    fn to_structured(&self) -> StructuredShortMessage {
        self.to_other()
    }

    /// Returns the type of this message.
    fn r#type(&self) -> ShortMessageType {
        extract_type_from_status_byte(self.status_byte()).unwrap()
    }

    /// Returns the super type of this message.
    fn super_type(&self) -> MessageSuperType {
        use MessageSuperType::*;
        use ShortMessageType::*;
        match self.r#type() {
            NoteOn
            | NoteOff
            | ChannelPressure
            | PolyphonicKeyPressure
            | PitchBendChange
            | ProgramChange => ChannelVoice,
            ControlChange => {
                if ControllerNumber::from(self.data_byte_1())
                    .is_channel_mode_message_controller_number()
                {
                    ChannelMode
                } else {
                    ChannelVoice
                }
            }
            TimingClock
            | SystemRealTimeUndefined1
            | Start
            | Continue
            | Stop
            | SystemRealTimeUndefined2
            | ActiveSensing
            | SystemReset => SystemRealTime,
            TimeCodeQuarterFrame
            | SongPositionPointer
            | SongSelect
            | SystemCommonUndefined1
            | SystemCommonUndefined2
            | TuneRequest
            | SystemExclusiveEnd => SystemCommon,
            SystemExclusiveStart => SystemExclusive,
        }
    }

    /// Returns the main category of this message.
    fn main_category(&self) -> MessageMainCategory {
        self.super_type().main_category()
    }

    /// Returns whether this message is a note-on in a practical sense. That means, it also returns
    /// `false` if the message type is [`NoteOn`] but the velocity is zero.
    ///
    /// [`NoteOn`]: enum.ShortMessageType.html#variant.NoteOn
    fn is_note_on(&self) -> bool {
        match self.to_structured() {
            StructuredShortMessage::NoteOn { velocity, .. } => velocity > U7::MIN,
            _ => false,
        }
    }

    /// Returns whether this message is a note-off in a practical sense. That means, it also returns
    /// `true` if the message type is [`NoteOn`] but the velocity is zero.
    ///
    /// [`NoteOn`]: enum.ShortMessageType.html#variant.NoteOn
    fn is_note_off(&self) -> bool {
        use StructuredShortMessage::*;
        match self.to_structured() {
            NoteOff { .. } => true,
            NoteOn { velocity, .. } => velocity == U7::MIN,
            _ => false,
        }
    }

    /// Returns whether this message is a note-on or note-off.
    fn is_note(&self) -> bool {
        match self.r#type() {
            ShortMessageType::NoteOn | ShortMessageType::NoteOff => true,
            _ => false,
        }
    }

    /// Returns the channel of this message if applicable.
    fn channel(&self) -> Option<Channel> {
        if self.main_category() != MessageMainCategory::Channel {
            return None;
        }
        Some(extract_channel_from_status_byte(self.status_byte()))
    }

    /// Returns the key number of this message if applicable.
    fn key_number(&self) -> Option<KeyNumber> {
        use ShortMessageType::*;
        match self.r#type() {
            NoteOff | NoteOn | PolyphonicKeyPressure => Some(self.data_byte_1().into()),
            _ => None,
        }
    }

    /// Returns the velocity of this message if applicable.
    fn velocity(&self) -> Option<U7> {
        use ShortMessageType::*;
        match self.r#type() {
            NoteOff | NoteOn => Some(self.data_byte_2()),
            _ => None,
        }
    }

    /// Returns the controller number of this message if applicable.
    fn controller_number(&self) -> Option<ControllerNumber> {
        if self.r#type() != ShortMessageType::ControlChange {
            return None;
        }
        Some(self.data_byte_1().into())
    }

    /// Returns the control value of this message if applicable.
    fn control_value(&self) -> Option<U7> {
        if self.r#type() != ShortMessageType::ControlChange {
            return None;
        }
        Some(self.data_byte_2())
    }

    /// Returns the program number of this message if applicable.
    fn program_number(&self) -> Option<U7> {
        if self.r#type() != ShortMessageType::ProgramChange {
            return None;
        }
        Some(self.data_byte_1())
    }

    /// Returns the pressure amount of this message if applicable.
    fn pressure_amount(&self) -> Option<U7> {
        use ShortMessageType::*;
        match self.r#type() {
            PolyphonicKeyPressure => Some(self.data_byte_2()),
            ChannelPressure => Some(self.data_byte_1()),
            _ => None,
        }
    }

    /// Returns the pitch bend value of this message if applicable.
    fn pitch_bend_value(&self) -> Option<U14> {
        if self.r#type() != ShortMessageType::PitchBendChange {
            return None;
        }
        Some(build_14_bit_value_from_two_7_bit_values(
            self.data_byte_2(),
            self.data_byte_1(),
        ))
    }
}

/// The most fine-grained classification of short MIDI messages.
///
/// Variants can be converted to and from `u8`. In case of channel messages, the `u8` value
/// corresponds to the status byte with channel 0. In case of system messages, the `u8` value
/// corresponds to the complete status byte.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, IntoPrimitive, TryFromPrimitive,
)]
#[cfg_attr(feature = "serde", derive(Serialize_repr, Deserialize_repr))]
#[repr(u8)]
pub enum ShortMessageType {
    // Channel messages = channel voice messages + channel mode messages
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyphonicKeyPressure = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBendChange = 0xE0,
    // System Exclusive messages
    SystemExclusiveStart = 0xF0,
    // System Common messages
    TimeCodeQuarterFrame = 0xF1,
    SongPositionPointer = 0xF2,
    SongSelect = 0xF3,
    SystemCommonUndefined1 = 0xF4,
    SystemCommonUndefined2 = 0xF5,
    TuneRequest = 0xF6,
    SystemExclusiveEnd = 0xF7,
    // System Real Time messages
    TimingClock = 0xF8,
    SystemRealTimeUndefined1 = 0xF9,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    SystemRealTimeUndefined2 = 0xFD,
    ActiveSensing = 0xFE,
    SystemReset = 0xFF,
}

impl ShortMessageType {
    /// `u8` representation of the first message type.
    pub const MIN: u8 = 0x80;

    /// `u8` representation of the last message type.
    pub const MAX: u8 = 0xFF;

    /// Returns the corresponding fuzzy super type.
    pub fn super_type(&self) -> FuzzyMessageSuperType {
        use FuzzyMessageSuperType::*;
        use ShortMessageType::*;
        match self {
            NoteOn
            | NoteOff
            | ChannelPressure
            | PolyphonicKeyPressure
            | PitchBendChange
            | ProgramChange
            | ControlChange => Channel,
            TimingClock
            | SystemRealTimeUndefined1
            | Start
            | Continue
            | Stop
            | SystemRealTimeUndefined2
            | ActiveSensing
            | SystemReset => SystemRealTime,
            TimeCodeQuarterFrame
            | SongPositionPointer
            | SongSelect
            | SystemCommonUndefined1
            | SystemCommonUndefined2
            | TuneRequest
            | SystemExclusiveEnd => SystemCommon,
            SystemExclusiveStart => SystemExclusive,
        }
    }
}

/// Like [`MessageSuperType`] but without distinction between different channel messages.
///
/// This enum exists because in some cases it can be helpful to obtain the message super type just
/// from a [`ShortMessageType`] - at least to some degree. In order to accurately determine the
/// [`MessageSuperType`], it's necessary to have the actual MIDI message at hand, because the
/// distinction between channel voice and channel mode messages depends on the controller number.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FuzzyMessageSuperType {
    /// See [`MessageSuperType::ChannelVoice`] and [`MessageSuperType::ChannelMode`]
    Channel,
    /// See [`MessageSuperType::SystemCommon`]
    SystemCommon,
    /// See [`MessageSuperType::SystemRealTime`]
    SystemRealTime,
    /// See [`MessageSuperType::SystemExclusive`]
    SystemExclusive,
}

/// A more coarse-grained classification of MIDI messages than [`ShortMessageType`].
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MessageSuperType {
    /// Channel Voice messages are used to send musical performance information.
    ChannelVoice,
    /// Channel Mode messages affect the way a synthesizer responds to MIDI data.
    ChannelMode,
    /// System Common messages are intended for all receivers in the system.
    SystemCommon,
    /// System Real Time messages are used for synchronization between clock-based MIDI components.
    SystemRealTime,
    /// System Exclusive messages are used to transfer any number of data bytes in a format
    /// specified by the referenced manufacturer.
    SystemExclusive,
}

impl FuzzyMessageSuperType {
    /// Returns the corresponding main category.
    pub fn main_category(&self) -> MessageMainCategory {
        use MessageMainCategory::*;
        if *self == FuzzyMessageSuperType::Channel {
            Channel
        } else {
            System
        }
    }
}

impl MessageSuperType {
    /// Returns the corresponding main category.
    pub fn main_category(&self) -> MessageMainCategory {
        use MessageMainCategory::*;
        use MessageSuperType::*;
        match *self {
            ChannelMode | ChannelVoice => Channel,
            SystemCommon | SystemRealTime | SystemExclusive => System,
        }
    }
}

/// The most high-level classification of MIDI messages.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MessageMainCategory {
    /// Channel Messages apply to a specific channel.
    Channel,
    /// System Messages are not channel-specific.
    System,
}

/// Possible contents of a MIDI Time Code Quarter Frame message. Each frame is part of the MIDI Time
/// Code information used for synchronization of MIDI equipment and other equipment, such as audio
/// or video tape machines.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TimeCodeQuarterFrame {
    FrameCountLsNibble(U4),
    FrameCountMsNibble(U4),
    SecondsCountLsNibble(U4),
    SecondsCountMsNibble(U4),
    MinutesCountLsNibble(U4),
    MinutesCountMsNibble(U4),
    HoursCountLsNibble(U4),
    Last {
        hours_count_ms_bit: bool,
        time_code_type: TimeCodeType,
    },
}

impl From<TimeCodeQuarterFrame> for U7 {
    fn from(frame: TimeCodeQuarterFrame) -> Self {
        use TimeCodeQuarterFrame::*;
        match frame {
            FrameCountLsNibble(v) => build_mtc_quarter_frame_data_byte(0, v),
            FrameCountMsNibble(v) => build_mtc_quarter_frame_data_byte(1, v),
            SecondsCountLsNibble(v) => build_mtc_quarter_frame_data_byte(2, v),
            SecondsCountMsNibble(v) => build_mtc_quarter_frame_data_byte(3, v),
            MinutesCountLsNibble(v) => build_mtc_quarter_frame_data_byte(4, v),
            MinutesCountMsNibble(v) => build_mtc_quarter_frame_data_byte(5, v),
            HoursCountLsNibble(v) => build_mtc_quarter_frame_data_byte(6, v),
            Last {
                hours_count_ms_bit,
                time_code_type,
            } => {
                let bit_0 = hours_count_ms_bit as u8;
                let bit_1_and_2 = u8::from(time_code_type) << 1;
                build_mtc_quarter_frame_data_byte(7, U4(bit_1_and_2 | bit_0))
            }
        }
    }
}

fn build_mtc_quarter_frame_data_byte(frame_type: u8, data: U4) -> U7 {
    U7((frame_type << 4) | u8::from(data))
}

impl From<U7> for TimeCodeQuarterFrame {
    fn from(data_byte_1: U7) -> Self {
        use TimeCodeQuarterFrame::*;
        let data = data_byte_1.get();
        match extract_high_nibble_from_byte(data) {
            0 => FrameCountLsNibble(extract_low_nibble_from_byte(data)),
            1 => FrameCountMsNibble(extract_low_nibble_from_byte(data)),
            2 => SecondsCountLsNibble(extract_low_nibble_from_byte(data)),
            3 => SecondsCountMsNibble(extract_low_nibble_from_byte(data)),
            4 => MinutesCountLsNibble(extract_low_nibble_from_byte(data)),
            5 => MinutesCountMsNibble(extract_low_nibble_from_byte(data)),
            6 => HoursCountLsNibble(extract_low_nibble_from_byte(data)),
            7 => Last {
                hours_count_ms_bit: (data & 0b0000001) != 0,
                time_code_type: ((data & 0b0000110) >> 1).try_into().unwrap(),
            },
            _ => unreachable!(),
        }
    }
}

/// Possible time code types of a MIDI Time Code Quarter Frame message.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, IntoPrimitive, TryFromPrimitive,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum TimeCodeType {
    Fps24 = 0,
    Fps25 = 1,
    Fps30DropFrame = 2,
    Fps30NonDrop = 3,
}

/// An error which can be returned when trying to create a [`ShortMessage`] from raw bytes.
///
/// [`ShortMessage`]: trait.ShortMessage.html
#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "invalid status byte")]
pub struct InvalidStatusByteError;

pub(crate) fn extract_type_from_status_byte(
    status_byte: u8,
) -> Result<ShortMessageType, InvalidStatusByteError> {
    let high_status_byte_nibble = extract_high_nibble_from_byte(status_byte);
    let relevant_part = if high_status_byte_nibble == 0xf {
        // System message. The complete status byte makes up the type.
        status_byte
    } else {
        // Channel message. Just the high nibble of the status byte makes up the type
        // (low nibble encodes channel).
        build_byte_from_nibbles(high_status_byte_nibble, 0)
    };
    ShortMessageType::try_from(relevant_part).map_err(|_| InvalidStatusByteError)
}

fn extract_low_nibble_from_byte(value: u8) -> U4 {
    U4(value & 0x0f)
}

fn extract_high_nibble_from_byte(byte: u8) -> u8 {
    (byte >> 4) & 0x0f
}

fn build_byte_from_nibbles(high_nibble: u8, low_nibble: u8) -> u8 {
    debug_assert!(high_nibble <= 0xf);
    debug_assert!(low_nibble <= 0xf);
    (high_nibble << 4) | low_nibble
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{channel as ch, controller_number, key_number, u14, u7};
    use crate::{RawShortMessage, ShortMessageFactory};
    #[cfg(feature = "serde")]
    use serde_json::json;

    #[test]
    fn from_bytes_ok() {
        // Given
        let msg = RawShortMessage::from_bytes((145, u7(64), u7(100))).unwrap();
        // When
        // Then
        assert_eq!(std::mem::size_of::<RawShortMessage>(), 3);
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), ShortMessageType::NoteOn);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(1)));
        assert_eq!(msg.key_number(), Some(key_number(64)));
        assert_eq!(msg.velocity(), Some(u7(100)));
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::NoteOn {
                channel: ch(1),
                key_number: key_number(64),
                velocity: u7(100),
            }
        );
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn from_bytes_err() {
        // Given
        let msg = RawShortMessage::from_bytes((2, u7(64), u7(100)));
        // When
        // Then
        assert!(msg.is_err());
    }

    #[test]
    fn note_on() {
        // Given
        let msg = RawShortMessage::note_on(ch(1), key_number(64), u7(100));
        // When
        // Then
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), ShortMessageType::NoteOn);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(1)));
        assert_eq!(msg.key_number(), Some(key_number(64)));
        assert_eq!(msg.velocity(), Some(u7(100)));
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::NoteOn {
                channel: ch(1),
                key_number: key_number(64),
                velocity: u7(100),
            }
        );
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn real_note_off() {
        // Given
        let msg = RawShortMessage::note_off(ch(2), key_number(125), u7(70));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0x82);
        assert_eq!(msg.data_byte_1(), u7(125));
        assert_eq!(msg.data_byte_2(), u7(70));
        assert_eq!(msg.r#type(), ShortMessageType::NoteOff);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(2)));
        assert_eq!(msg.key_number(), Some(key_number(125)));
        assert_eq!(msg.velocity(), Some(u7(70)));
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::NoteOff {
                channel: ch(2),
                key_number: key_number(125),
                velocity: u7(70),
            }
        );
        assert!(msg.is_note());
        assert!(!msg.is_note_on());
        assert!(msg.is_note_off());
    }

    #[test]
    fn fake_note_off() {
        // Given
        let msg = RawShortMessage::note_on(ch(0), key_number(5), u7(0));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0x90);
        assert_eq!(msg.data_byte_1(), u7(5));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::NoteOn);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(0)));
        assert_eq!(msg.key_number(), Some(key_number(5)));
        assert_eq!(msg.velocity(), Some(u7(0)));
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::NoteOn {
                channel: ch(0),
                key_number: key_number(5),
                velocity: u7(0),
            }
        );
        assert!(msg.is_note());
        assert!(!msg.is_note_on());
        assert!(msg.is_note_off());
    }

    #[test]
    fn control_change() {
        // Given
        let msg = RawShortMessage::control_change(ch(1), controller_number(50), u7(2));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xb1);
        assert_eq!(msg.data_byte_1(), u7(50));
        assert_eq!(msg.data_byte_2(), u7(2));
        assert_eq!(msg.r#type(), ShortMessageType::ControlChange);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(1)));
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), Some(controller_number(50)));
        assert_eq!(msg.control_value(), Some(u7(2)));
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::ControlChange {
                channel: ch(1),
                controller_number: controller_number(50),
                control_value: u7(2),
            }
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn program_change() {
        // Given
        let msg = RawShortMessage::program_change(ch(4), u7(22));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xc4);
        assert_eq!(msg.data_byte_1(), u7(22));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::ProgramChange);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(4)));
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), Some(u7(22)));
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::ProgramChange {
                channel: ch(4),
                program_number: u7(22),
            }
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn polyphonic_key_pressure() {
        // Given
        let msg = RawShortMessage::polyphonic_key_pressure(ch(15), key_number(127), u7(50));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xaf);
        assert_eq!(msg.data_byte_1(), u7(127));
        assert_eq!(msg.data_byte_2(), u7(50));
        assert_eq!(msg.r#type(), ShortMessageType::PolyphonicKeyPressure);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(15)));
        assert_eq!(msg.key_number(), Some(key_number(127)));
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), Some(u7(50)));
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::PolyphonicKeyPressure {
                channel: ch(15),
                key_number: key_number(127),
                pressure_amount: u7(50),
            }
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn channel_pressure() {
        // Given
        let msg = RawShortMessage::channel_pressure(ch(14), u7(0));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xde);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::ChannelPressure);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(14)));
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), Some(u7(0)));
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::ChannelPressure {
                channel: ch(14),
                pressure_amount: u7(0),
            }
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn pitch_bend_change() {
        // Given
        let msg = RawShortMessage::pitch_bend_change(ch(1), u14(1278));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xe1);
        assert_eq!(msg.data_byte_1(), u7(126));
        assert_eq!(msg.data_byte_2(), u7(9));
        assert_eq!(msg.r#type(), ShortMessageType::PitchBendChange);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(1)));
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), Some(u14(1278)));
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredShortMessage::PitchBendChange {
                channel: ch(1),
                pitch_bend_value: u14(1278),
            }
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn timing_clock() {
        // Given
        let msg = RawShortMessage::timing_clock();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xf8);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::TimingClock);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::TimingClock);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn start() {
        // Given
        let msg = RawShortMessage::start();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfa);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::Start);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::Start);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn r#continue() {
        // Given
        let msg = RawShortMessage::r#continue();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfb);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::Continue);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::Continue);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn stop_message() {
        // Given
        let msg = RawShortMessage::stop();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfc);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::Stop);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::Stop);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn active_sensing() {
        // Given
        let msg = RawShortMessage::active_sensing();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfe);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::ActiveSensing);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::ActiveSensing);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn system_reset() {
        // Given
        let msg = RawShortMessage::system_reset();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xff);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), ShortMessageType::SystemReset);
        assert_eq!(msg.super_type(), MessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredShortMessage::SystemReset);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn structured() {
        // Given
        let msg = StructuredShortMessage::from_bytes((145, u7(64), u7(100))).unwrap();
        // When
        // Then
        let expected_msg = StructuredShortMessage::NoteOn {
            channel: ch(1),
            key_number: key_number(64),
            velocity: u7(100),
        };
        assert_eq!(msg, expected_msg);
        assert_eq!(std::mem::size_of::<StructuredShortMessage>(), 4);
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), ShortMessageType::NoteOn);
        assert_eq!(msg.super_type(), MessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MessageMainCategory::Channel);
        assert_eq!(msg.channel(), Some(ch(1)));
        assert_eq!(msg.key_number(), Some(key_number(64)));
        assert_eq!(msg.velocity(), Some(u7(100)));
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), expected_msg);
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn structured_serialize() {
        // Given
        let msg = StructuredShortMessage::from_bytes((145, u7(64), u7(100))).unwrap();
        // When
        let j = serde_json::to_value(&msg).unwrap();
        // Then
        assert_eq!(
            j,
            json! {
                {
                    "NoteOn": {
                        "channel": 1,
                        "key_number": 64,
                        "velocity": 100
                    }
                }
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn raw_serialize() {
        // Given
        let msg = RawShortMessage::from_bytes((145, u7(64), u7(100))).unwrap();
        // When
        let j = serde_json::to_value(&msg).unwrap();
        // Then
        assert_eq!(
            j,
            json! {
                [145, 64, 100]
            }
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn type_serialize() {
        // Given
        let r#type = RawShortMessage::note_on(ch(4), key_number(50), u7(100)).r#type();
        // When
        let j = serde_json::to_value(&r#type).unwrap();
        // Then
        assert_eq!(
            j,
            json! {
                144
            }
        );
    }

    #[test]
    fn structured_and_back() {
        // Given
        let messages: Vec<RawShortMessage> = (ShortMessageType::MIN..=ShortMessageType::MAX)
            .flat_map(|repr| ShortMessageType::try_from(repr))
            .flat_map(move |t| match t.super_type() {
                FuzzyMessageSuperType::Channel => (0..16)
                    .map(|c| RawShortMessage::channel_message(t, ch(c), U7::MIN, U7::MIN))
                    .collect(),
                FuzzyMessageSuperType::SystemRealTime => {
                    vec![RawShortMessage::system_real_time_message(t)]
                }
                _ => vec![],
            })
            .collect();
        for msg in messages {
            // When
            let structured = msg.to_structured();
            let restored = RawShortMessage::from_other(&structured);
            // Then
            assert_equal_results(&msg, &structured);
            assert_equal_results(&msg, &restored);
        }
    }

    fn assert_equal_results(first: &impl ShortMessage, second: &impl ShortMessage) {
        assert_eq!(first.status_byte(), second.status_byte());
        assert_eq!(first.data_byte_1(), second.data_byte_1());
        assert_eq!(first.data_byte_2(), second.data_byte_2());
        assert_eq!(first.r#type(), second.r#type());
        assert_eq!(first.super_type(), second.super_type());
        assert_eq!(first.main_category(), second.main_category());
        assert_eq!(first.channel(), second.channel());
        assert_eq!(first.key_number(), second.key_number());
        assert_eq!(first.velocity(), second.velocity());
        assert_eq!(first.controller_number(), second.controller_number());
        assert_eq!(first.control_value(), second.control_value());
        assert_eq!(first.pitch_bend_value(), second.pitch_bend_value());
        assert_eq!(first.pressure_amount(), second.pressure_amount());
        assert_eq!(first.program_number(), second.program_number());
        assert_eq!(first.is_note(), second.is_note());
        assert_eq!(first.is_note_on(), second.is_note_on());
        assert_eq!(first.is_note_off(), second.is_note_off());
    }
}
