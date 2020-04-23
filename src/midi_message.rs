use crate::{
    build_14_bit_value_from_two_7_bit_values, Channel, ControllerNumber, KeyNumber,
    StructuredMidiMessage, U14, U4, U7,
};
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::convert::TryInto;
#[allow(unused_imports)]
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Trait to be implemented by struct representing a single primitive MIDI message. Only the three
/// byte-returning methods need to be implemented, the rest is done by default methods. The
/// advantage of this architecture is that we can have a unified API, no matter which underlying
/// data structure is used.
///
/// This trait is just for primitive MIDI messages, that is messages which use 3 bytes at a
/// maximum. It can't represent messages which are longer than 3 bytes, e.g. it can't be used to
/// represent a complete sys-ex message. This is by design. One main advantage is that
/// any implementation of this trait can easily implement Copy, which is essential if you want to
/// pass around messages by copying them instead of dealing with references. MIDI messages are
/// often processed in a real-time thread where things need to happen fast and heap allocations
/// are a no-go. If we would make this trait support arbitrarily-sized messages, we would lose Copy
/// and would have to make everything work with references or pointers - which can bring its
/// own restrictions such as not being able to use rxRust in a safe way.
///
/// Please also implement the trait `MidiMessageFactory` for your struct if creating new MIDI
/// messages programmatically should be supported.
pub trait MidiMessage {
    fn status_byte(&self) -> u8;

    fn data_byte_1(&self) -> U7;

    fn data_byte_2(&self) -> U7;

    fn r#type(&self) -> MidiMessageType {
        midi_message_type_from_status_byte(self.status_byte()).unwrap()
    }

    fn super_type(&self) -> MidiMessageSuperType {
        use MidiMessageSuperType::*;
        use MidiMessageType::*;
        match self.r#type() {
            NoteOn
            | NoteOff
            | ChannelPressure
            | PolyphonicKeyPressure
            | PitchBendChange
            | ProgramChange => ChannelVoice,
            ControlChange => {
                if ControllerNumber::from(self.data_byte_1())
                    < ControllerNumber::LOCAL_CONTROL_ON_OFF
                {
                    ChannelVoice
                } else {
                    ChannelMode
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
            MidiTimeCodeQuarterFrame
            | SongPositionPointer
            | SongSelect
            | SystemCommonUndefined1
            | SystemCommonUndefined2
            | TuneRequest
            | SystemExclusiveEnd => SystemCommon,
            SystemExclusiveStart => SystemExclusive,
        }
    }

    fn main_category(&self) -> MidiMessageMainCategory {
        self.super_type().main_category()
    }

    fn to_structured(&self) -> StructuredMidiMessage {
        use MidiMessageType::*;
        match self.r#type() {
            NoteOff => StructuredMidiMessage::NoteOff {
                channel: extract_channel_from_status_byte(self.status_byte()),
                key_number: self.data_byte_1().into(),
                velocity: self.data_byte_2(),
            },
            NoteOn => StructuredMidiMessage::NoteOn {
                channel: extract_channel_from_status_byte(self.status_byte()),
                key_number: self.data_byte_1().into(),
                velocity: self.data_byte_2(),
            },
            PolyphonicKeyPressure => StructuredMidiMessage::PolyphonicKeyPressure {
                channel: extract_channel_from_status_byte(self.status_byte()),
                key_number: self.data_byte_1().into(),
                pressure_amount: self.data_byte_2(),
            },
            ControlChange => StructuredMidiMessage::ControlChange {
                channel: extract_channel_from_status_byte(self.status_byte()),
                controller_number: self.data_byte_1().into(),
                control_value: self.data_byte_2(),
            },
            ProgramChange => StructuredMidiMessage::ProgramChange {
                channel: extract_channel_from_status_byte(self.status_byte()),
                program_number: self.data_byte_1().into(),
            },
            ChannelPressure => StructuredMidiMessage::ChannelPressure {
                channel: extract_channel_from_status_byte(self.status_byte()),
                pressure_amount: self.data_byte_1(),
            },
            PitchBendChange => StructuredMidiMessage::PitchBendChange {
                channel: extract_channel_from_status_byte(self.status_byte()),
                pitch_bend_value: build_14_bit_value_from_two_7_bit_values(
                    self.data_byte_2(),
                    self.data_byte_1(),
                ),
            },
            SystemExclusiveStart => StructuredMidiMessage::SystemExclusiveStart,
            MidiTimeCodeQuarterFrame => {
                StructuredMidiMessage::MidiTimeCodeQuarterFrame(self.data_byte_1().into())
            }
            SongPositionPointer => StructuredMidiMessage::SongPositionPointer {
                position: build_14_bit_value_from_two_7_bit_values(
                    self.data_byte_2(),
                    self.data_byte_1(),
                ),
            },
            SongSelect => StructuredMidiMessage::SongSelect {
                song_number: self.data_byte_1(),
            },
            TuneRequest => StructuredMidiMessage::TuneRequest,
            SystemExclusiveEnd => StructuredMidiMessage::SystemExclusiveEnd,
            TimingClock => StructuredMidiMessage::TimingClock,
            Start => StructuredMidiMessage::Start,
            Continue => StructuredMidiMessage::Continue,
            Stop => StructuredMidiMessage::Stop,
            ActiveSensing => StructuredMidiMessage::ActiveSensing,
            SystemReset => StructuredMidiMessage::SystemReset,
            SystemCommonUndefined1 => StructuredMidiMessage::SystemCommonUndefined1,
            SystemCommonUndefined2 => StructuredMidiMessage::SystemCommonUndefined2,
            SystemRealTimeUndefined1 => StructuredMidiMessage::SystemRealTimeUndefined1,
            SystemRealTimeUndefined2 => StructuredMidiMessage::SystemRealTimeUndefined2,
        }
    }

    // Returns false if the message type is NoteOn but the velocity is 0
    fn is_note_on(&self) -> bool {
        match self.to_structured() {
            StructuredMidiMessage::NoteOn { velocity, .. } => velocity > U7::MIN,
            _ => false,
        }
    }

    // Also returns true if the message type is NoteOn but the velocity is 0
    fn is_note_off(&self) -> bool {
        use StructuredMidiMessage::*;
        match self.to_structured() {
            NoteOff { .. } => true,
            NoteOn { velocity, .. } => velocity == U7::MIN,
            _ => false,
        }
    }

    fn is_note(&self) -> bool {
        match self.r#type() {
            MidiMessageType::NoteOn | MidiMessageType::NoteOff => true,
            _ => false,
        }
    }

    fn channel(&self) -> Option<Channel> {
        if self.main_category() != MidiMessageMainCategory::Channel {
            return None;
        }
        Some(extract_channel_from_status_byte(self.status_byte()))
    }

    fn key_number(&self) -> Option<KeyNumber> {
        use MidiMessageType::*;
        match self.r#type() {
            NoteOff | NoteOn | PolyphonicKeyPressure => Some(self.data_byte_1().into()),
            _ => None,
        }
    }

    fn velocity(&self) -> Option<U7> {
        use MidiMessageType::*;
        match self.r#type() {
            NoteOff | NoteOn => Some(self.data_byte_2()),
            _ => None,
        }
    }

    fn controller_number(&self) -> Option<ControllerNumber> {
        if self.r#type() != MidiMessageType::ControlChange {
            return None;
        }
        Some(self.data_byte_1().into())
    }

    fn control_value(&self) -> Option<U7> {
        if self.r#type() != MidiMessageType::ControlChange {
            return None;
        }
        Some(self.data_byte_2())
    }

    fn program_number(&self) -> Option<U7> {
        if self.r#type() != MidiMessageType::ProgramChange {
            return None;
        }
        Some(self.data_byte_1())
    }

    fn pressure_amount(&self) -> Option<U7> {
        use MidiMessageType::*;
        match self.r#type() {
            PolyphonicKeyPressure => Some(self.data_byte_2()),
            ChannelPressure => Some(self.data_byte_1()),
            _ => None,
        }
    }

    fn pitch_bend_value(&self) -> Option<U14> {
        if self.r#type() != MidiMessageType::PitchBendChange {
            return None;
        }
        Some(build_14_bit_value_from_two_7_bit_values(
            self.data_byte_2(),
            self.data_byte_1(),
        ))
    }
}

// The most low-level type of a MIDI message
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    IntoPrimitive,
    TryFromPrimitive,
    EnumIter,
)]
#[repr(u8)]
pub enum MidiMessageType {
    // Channel messages = channel voice messages + channel mode messages (given value represents
    // channel 0 status byte)
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyphonicKeyPressure = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBendChange = 0xE0,
    // System exclusive messages
    SystemExclusiveStart = 0xF0,
    // System common messages
    MidiTimeCodeQuarterFrame = 0xF1,
    SongPositionPointer = 0xF2,
    SongSelect = 0xF3,
    SystemCommonUndefined1 = 0xF4,
    SystemCommonUndefined2 = 0xF5,
    TuneRequest = 0xF6,
    SystemExclusiveEnd = 0xF7,
    // System real-time messages (given value represents the complete status byte)
    TimingClock = 0xF8,
    SystemRealTimeUndefined1 = 0xF9,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    SystemRealTimeUndefined2 = 0xFD,
    ActiveSensing = 0xFE,
    SystemReset = 0xFF,
}

impl MidiMessageType {
    pub fn super_type(&self) -> BlurryMidiMessageSuperType {
        use BlurryMidiMessageSuperType::*;
        use MidiMessageType::*;
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
            MidiTimeCodeQuarterFrame
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

// A somewhat mid-level type of a MIDI message.
// In this enum we don't distinguish between channel voice and channel mode messages because this
// difference doesn't solely depend on the MidiMessageType (channel mode messages are just
// particular ControlChange messages).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BlurryMidiMessageSuperType {
    Channel,
    SystemCommon,
    SystemRealTime,
    SystemExclusive,
}

// A somewhat mid-level type of a MIDI message.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MidiMessageSuperType {
    ChannelVoice,
    ChannelMode,
    SystemCommon,
    SystemRealTime,
    SystemExclusive,
}

impl BlurryMidiMessageSuperType {
    pub fn main_category(&self) -> MidiMessageMainCategory {
        use MidiMessageMainCategory::*;
        if *self == BlurryMidiMessageSuperType::Channel {
            Channel
        } else {
            System
        }
    }
}

impl MidiMessageSuperType {
    pub fn main_category(&self) -> MidiMessageMainCategory {
        use MidiMessageMainCategory::*;
        use MidiMessageSuperType::*;
        match *self {
            ChannelMode | ChannelVoice => Channel,
            SystemCommon | SystemRealTime | SystemExclusive => System,
        }
    }
}

// The MIDI spec says: "Messages are divided into two main categories: Channel and System."
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MidiMessageMainCategory {
    Channel,
    System,
}

/// Content of a MIDI time code quarter frame message. It contains a part of the current time code.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MidiTimeCodeQuarterFrame {
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

impl From<MidiTimeCodeQuarterFrame> for U7 {
    fn from(frame: MidiTimeCodeQuarterFrame) -> Self {
        use MidiTimeCodeQuarterFrame::*;
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

impl From<U7> for MidiTimeCodeQuarterFrame {
    fn from(data_byte_1: U7) -> Self {
        use MidiTimeCodeQuarterFrame::*;
        let data = u8::from(data_byte_1);
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

/// Time code type contained in the last quarter frame message
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum TimeCodeType {
    Fps24 = 0,
    Fps25 = 1,
    Fps30DropFrame = 2,
    Fps30NonDrop = 3,
}

fn extract_channel_from_status_byte(byte: u8) -> Channel {
    Channel(byte & 0x0f)
}

pub(crate) fn midi_message_type_from_status_byte(
    status_byte: u8,
) -> Result<MidiMessageType, TryFromPrimitiveError<MidiMessageType>> {
    let high_status_byte_nibble = extract_high_nibble_from_byte(status_byte);
    if high_status_byte_nibble == 0xf {
        // System message. The complete status byte makes up the type.
        status_byte.try_into()
    } else {
        // Channel message. Just the high nibble of the status byte makes up the type
        // (low nibble encodes channel).
        build_byte_from_nibbles(high_status_byte_nibble, 0).try_into()
    }
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
    use crate::{Channel, MidiMessageFactory, RawMidiMessage};

    #[test]
    fn from_bytes_ok() {
        // Given
        let msg = RawMidiMessage::from_bytes(145, u7(64), u7(100)).unwrap();
        // When
        // Then
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), MidiMessageType::NoteOn);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::NoteOn {
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
        let msg = RawMidiMessage::from_bytes(2, u7(64), u7(100));
        // When
        // Then
        assert!(msg.is_err());
    }

    #[test]
    fn note_on() {
        // Given
        let msg = RawMidiMessage::note_on(ch(1), key_number(64), u7(100));
        // When
        // Then
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), MidiMessageType::NoteOn);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::NoteOn {
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
        let msg = RawMidiMessage::note_off(ch(2), key_number(125), u7(70));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0x82);
        assert_eq!(msg.data_byte_1(), u7(125));
        assert_eq!(msg.data_byte_2(), u7(70));
        assert_eq!(msg.r#type(), MidiMessageType::NoteOff);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::NoteOff {
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
        let msg = RawMidiMessage::note_on(ch(0), key_number(5), u7(0));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0x90);
        assert_eq!(msg.data_byte_1(), u7(5));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::NoteOn);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::NoteOn {
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
        let msg = RawMidiMessage::control_change(ch(1), controller_number(50), u7(2));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xb1);
        assert_eq!(msg.data_byte_1(), u7(50));
        assert_eq!(msg.data_byte_2(), u7(2));
        assert_eq!(msg.r#type(), MidiMessageType::ControlChange);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::ControlChange {
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
        let msg = RawMidiMessage::program_change(ch(4), u7(22));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xc4);
        assert_eq!(msg.data_byte_1(), u7(22));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::ProgramChange);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::ProgramChange {
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
        let msg = RawMidiMessage::polyphonic_key_pressure(ch(15), key_number(127), u7(50));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xaf);
        assert_eq!(msg.data_byte_1(), u7(127));
        assert_eq!(msg.data_byte_2(), u7(50));
        assert_eq!(msg.r#type(), MidiMessageType::PolyphonicKeyPressure);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::PolyphonicKeyPressure {
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
        let msg = RawMidiMessage::channel_pressure(ch(14), u7(0));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xde);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::ChannelPressure);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::ChannelPressure {
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
        let msg = RawMidiMessage::pitch_bend_change(ch(1), u14(1278));
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xe1);
        assert_eq!(msg.data_byte_1(), u7(126));
        assert_eq!(msg.data_byte_2(), u7(9));
        assert_eq!(msg.r#type(), MidiMessageType::PitchBendChange);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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
            StructuredMidiMessage::PitchBendChange {
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
        let msg = RawMidiMessage::timing_clock();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xf8);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::TimingClock);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::TimingClock);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn start() {
        // Given
        let msg = RawMidiMessage::start();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfa);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::Start);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::Start);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn r#continue() {
        // Given
        let msg = RawMidiMessage::r#continue();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfb);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::Continue);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::Continue);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn stop_message() {
        // Given
        let msg = RawMidiMessage::stop();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfc);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::Stop);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::Stop);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn active_sensing() {
        // Given
        let msg = RawMidiMessage::active_sensing();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xfe);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::ActiveSensing);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::ActiveSensing);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn system_reset() {
        // Given
        let msg = RawMidiMessage::system_reset();
        // When
        // Then
        assert_eq!(msg.status_byte(), 0xff);
        assert_eq!(msg.data_byte_1(), u7(0));
        assert_eq!(msg.data_byte_2(), u7(0));
        assert_eq!(msg.r#type(), MidiMessageType::SystemReset);
        assert_eq!(msg.super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.channel(), None);
        assert_eq!(msg.key_number(), None);
        assert_eq!(msg.velocity(), None);
        assert_eq!(msg.controller_number(), None);
        assert_eq!(msg.control_value(), None);
        assert_eq!(msg.pitch_bend_value(), None);
        assert_eq!(msg.pressure_amount(), None);
        assert_eq!(msg.program_number(), None);
        assert_eq!(msg.to_structured(), StructuredMidiMessage::SystemReset);
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn structured() {
        // Given
        let msg = StructuredMidiMessage::from_bytes(145, u7(64), u7(100)).unwrap();
        // When
        // Then
        let expected_msg = StructuredMidiMessage::NoteOn {
            channel: ch(1),
            key_number: key_number(64),
            velocity: u7(100),
        };
        assert_eq!(msg, expected_msg);
        assert_eq!(msg.status_byte(), 145);
        assert_eq!(msg.data_byte_1(), u7(64));
        assert_eq!(msg.data_byte_2(), u7(100));
        assert_eq!(msg.r#type(), MidiMessageType::NoteOn);
        assert_eq!(msg.super_type(), MidiMessageSuperType::ChannelVoice);
        assert_eq!(msg.main_category(), MidiMessageMainCategory::Channel);
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

    #[test]
    fn structured_and_back() {
        // Given
        let messages: Vec<RawMidiMessage> = MidiMessageType::iter()
            .flat_map(move |t| match t.super_type() {
                BlurryMidiMessageSuperType::Channel => (0..Channel::COUNT)
                    .map(|c| RawMidiMessage::channel_message(t, ch(c), U7::MIN, U7::MIN))
                    .collect(),
                BlurryMidiMessageSuperType::SystemRealTime => {
                    vec![RawMidiMessage::system_real_time_message(t)]
                }
                _ => vec![],
            })
            .collect();
        for msg in messages {
            // When
            let structured = msg.to_structured();
            let restored = RawMidiMessage::from_structured(&structured);
            // Then
            assert_equal_results(&msg, &structured);
            assert_equal_results(&msg, &restored);
        }
    }

    fn assert_equal_results(first: &impl MidiMessage, second: &impl MidiMessage) {
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
