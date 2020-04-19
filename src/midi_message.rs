use crate::{
    build_14_bit_value_from_two_7_bit_values, extract_high_7_bit_value_from_14_bit_value,
    extract_low_7_bit_value_from_14_bit_value, Channel, ControllerNumber, KeyNumber,
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
    fn get_status_byte(&self) -> u8;

    fn get_data_byte_1(&self) -> U7;

    fn get_data_byte_2(&self) -> U7;

    fn get_type(&self) -> MidiMessageType {
        get_midi_message_type_from_status_byte(self.get_status_byte()).unwrap()
    }

    fn get_super_type(&self) -> MidiMessageSuperType {
        self.get_type().get_super_type()
    }

    fn get_main_category(&self) -> MidiMessageMainCategory {
        self.get_super_type().get_main_category()
    }

    fn to_structured(&self) -> StructuredMidiMessage {
        use MidiMessageType::*;
        match self.get_type() {
            NoteOff => StructuredMidiMessage::NoteOff {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                key_number: self.get_data_byte_1().into(),
                velocity: self.get_data_byte_2(),
            },
            NoteOn => StructuredMidiMessage::NoteOn {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                key_number: self.get_data_byte_1().into(),
                velocity: self.get_data_byte_2(),
            },
            PolyphonicKeyPressure => StructuredMidiMessage::PolyphonicKeyPressure {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                key_number: self.get_data_byte_1().into(),
                pressure_amount: self.get_data_byte_2(),
            },
            ControlChange => StructuredMidiMessage::ControlChange {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                controller_number: self.get_data_byte_1().into(),
                control_value: self.get_data_byte_2(),
            },
            ProgramChange => StructuredMidiMessage::ProgramChange {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                program_number: self.get_data_byte_1().into(),
            },
            ChannelPressure => StructuredMidiMessage::ChannelPressure {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                pressure_amount: self.get_data_byte_1(),
            },
            PitchBendChange => StructuredMidiMessage::PitchBendChange {
                channel: extract_channel_from_status_byte(self.get_status_byte()),
                pitch_bend_value: build_14_bit_value_from_two_7_bit_values(
                    self.get_data_byte_2(),
                    self.get_data_byte_1(),
                ),
            },
            SystemExclusiveStart => StructuredMidiMessage::SystemExclusiveStart,
            MidiTimeCodeQuarterFrame => {
                StructuredMidiMessage::MidiTimeCodeQuarterFrame(self.get_data_byte_1().into())
            }
            SongPositionPointer => StructuredMidiMessage::SongPositionPointer {
                position: build_14_bit_value_from_two_7_bit_values(
                    self.get_data_byte_2(),
                    self.get_data_byte_1(),
                ),
            },
            SongSelect => StructuredMidiMessage::SongSelect {
                song_number: self.get_data_byte_1(),
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
        match self.get_type() {
            MidiMessageType::NoteOn | MidiMessageType::NoteOff => true,
            _ => false,
        }
    }

    fn get_channel(&self) -> Option<Channel> {
        if self.get_main_category() != MidiMessageMainCategory::Channel {
            return None;
        }
        Some(extract_channel_from_status_byte(self.get_status_byte()))
    }

    fn get_key_number(&self) -> Option<KeyNumber> {
        use MidiMessageType::*;
        match self.get_type() {
            NoteOff | NoteOn | PolyphonicKeyPressure => Some(self.get_data_byte_1().into()),
            _ => None,
        }
    }

    fn get_velocity(&self) -> Option<U7> {
        use MidiMessageType::*;
        match self.get_type() {
            NoteOff | NoteOn => Some(self.get_data_byte_2()),
            _ => None,
        }
    }

    fn get_controller_number(&self) -> Option<ControllerNumber> {
        if self.get_type() != MidiMessageType::ControlChange {
            return None;
        }
        Some(self.get_data_byte_1().into())
    }

    fn get_control_value(&self) -> Option<U7> {
        if self.get_type() != MidiMessageType::ControlChange {
            return None;
        }
        Some(self.get_data_byte_2())
    }

    fn get_program_number(&self) -> Option<U7> {
        if self.get_type() != MidiMessageType::ProgramChange {
            return None;
        }
        Some(self.get_data_byte_1())
    }

    fn get_pressure_amount(&self) -> Option<U7> {
        use MidiMessageType::*;
        match self.get_type() {
            PolyphonicKeyPressure => Some(self.get_data_byte_2()),
            ChannelPressure => Some(self.get_data_byte_1()),
            _ => None,
        }
    }

    fn get_pitch_bend_value(&self) -> Option<U14> {
        if self.get_type() != MidiMessageType::PitchBendChange {
            return None;
        }
        Some(build_14_bit_value_from_two_7_bit_values(
            self.get_data_byte_2(),
            self.get_data_byte_1(),
        ))
    }
}

// The most low-level type of a MIDI message
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumIter)]
#[repr(u8)]
// TODO Page 35 of MIDI spec PDF contains good classification
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
    pub fn get_super_type(&self) -> MidiMessageSuperType {
        use MidiMessageType::*;
        match self {
            NoteOn
            | NoteOff
            | ChannelPressure
            | PolyphonicKeyPressure
            | PitchBendChange
            | ProgramChange
            | ControlChange => MidiMessageSuperType::Channel,
            TimingClock
            | SystemRealTimeUndefined1
            | Start
            | Continue
            | Stop
            | SystemRealTimeUndefined2
            | ActiveSensing
            | SystemReset => MidiMessageSuperType::SystemRealTime,
            MidiTimeCodeQuarterFrame
            | SongPositionPointer
            | SongSelect
            | SystemCommonUndefined1
            | SystemCommonUndefined2
            | TuneRequest
            | SystemExclusiveEnd => MidiMessageSuperType::SystemCommon,
            SystemExclusiveStart => MidiMessageSuperType::SystemExclusive,
        }
    }
}

// A somewhat mid-level type of a MIDI message.
// In this enum we don't distinguish between channel voice and channel mode messages because this
// difference doesn't solely depend on the MidiMessageType (channel mode messages are just
// particular ControlChange messages).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MidiMessageSuperType {
    // TODO Distinguish between ChannelMode and ChannelVoice
    Channel,
    SystemCommon,
    SystemRealTime,
    SystemExclusive,
}

impl MidiMessageSuperType {
    pub fn get_main_category(&self) -> MidiMessageMainCategory {
        if *self == MidiMessageSuperType::Channel {
            MidiMessageMainCategory::Channel
        } else {
            MidiMessageMainCategory::System
        }
    }
}

// The MIDI spec says: "Messages are divided into two main categories: Channel and System."
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MidiMessageMainCategory {
    Channel,
    System,
}

/// Content of a MIDI time code quarter frame message. It contains a part of the current time code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
            7 => {
                use TimeCodeType::*;
                Last {
                    hours_count_ms_bit: (data & 0b0000001) != 0,
                    time_code_type: ((data & 0b0000110) >> 1).try_into().unwrap(),
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Time code type contained in the last quarter frame message
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
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

pub(crate) fn get_midi_message_type_from_status_byte(
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
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), u7(64));
        assert_eq!(msg.get_data_byte_2(), u7(100));
        assert_eq!(msg.get_type(), MidiMessageType::NoteOn);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(1)));
        assert_eq!(msg.get_key_number(), Some(key_number(64)));
        assert_eq!(msg.get_velocity(), Some(u7(100)));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), u7(64));
        assert_eq!(msg.get_data_byte_2(), u7(100));
        assert_eq!(msg.get_type(), MidiMessageType::NoteOn);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(1)));
        assert_eq!(msg.get_key_number(), Some(key_number(64)));
        assert_eq!(msg.get_velocity(), Some(u7(100)));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0x82);
        assert_eq!(msg.get_data_byte_1(), u7(125));
        assert_eq!(msg.get_data_byte_2(), u7(70));
        assert_eq!(msg.get_type(), MidiMessageType::NoteOff);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(2)));
        assert_eq!(msg.get_key_number(), Some(key_number(125)));
        assert_eq!(msg.get_velocity(), Some(u7(70)));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0x90);
        assert_eq!(msg.get_data_byte_1(), u7(5));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::NoteOn);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(0)));
        assert_eq!(msg.get_key_number(), Some(key_number(5)));
        assert_eq!(msg.get_velocity(), Some(u7(0)));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xb1);
        assert_eq!(msg.get_data_byte_1(), u7(50));
        assert_eq!(msg.get_data_byte_2(), u7(2));
        assert_eq!(msg.get_type(), MidiMessageType::ControlChange);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(1)));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), Some(controller_number(50)));
        assert_eq!(msg.get_control_value(), Some(u7(2)));
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xc4);
        assert_eq!(msg.get_data_byte_1(), u7(22));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::ProgramChange);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(4)));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), Some(u7(22)));
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
        assert_eq!(msg.get_status_byte(), 0xaf);
        assert_eq!(msg.get_data_byte_1(), u7(127));
        assert_eq!(msg.get_data_byte_2(), u7(50));
        assert_eq!(msg.get_type(), MidiMessageType::PolyphonicKeyPressure);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(15)));
        assert_eq!(msg.get_key_number(), Some(key_number(127)));
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), Some(u7(50)));
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xde);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::ChannelPressure);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(14)));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), Some(u7(0)));
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xe1);
        assert_eq!(msg.get_data_byte_1(), u7(126));
        assert_eq!(msg.get_data_byte_2(), u7(9));
        assert_eq!(msg.get_type(), MidiMessageType::PitchBendChange);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(1)));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), Some(u14(1278)));
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xf8);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::TimingClock);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xfa);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::Start);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xfb);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::Continue);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xfc);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::Stop);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xfe);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::ActiveSensing);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 0xff);
        assert_eq!(msg.get_data_byte_1(), u7(0));
        assert_eq!(msg.get_data_byte_2(), u7(0));
        assert_eq!(msg.get_type(), MidiMessageType::SystemReset);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::SystemRealTime);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::System);
        assert_eq!(msg.get_channel(), None);
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
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
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), u7(64));
        assert_eq!(msg.get_data_byte_2(), u7(100));
        assert_eq!(msg.get_type(), MidiMessageType::NoteOn);
        assert_eq!(msg.get_super_type(), MidiMessageSuperType::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(ch(1)));
        assert_eq!(msg.get_key_number(), Some(key_number(64)));
        assert_eq!(msg.get_velocity(), Some(u7(100)));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(msg.to_structured(), expected_msg);
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn structured_and_back() {
        // Given
        let messages: Vec<RawMidiMessage> = MidiMessageType::iter()
            .flat_map(move |t| match t.get_super_type() {
                MidiMessageSuperType::Channel => (0..Channel::COUNT)
                    .map(|c| RawMidiMessage::channel_message(t, ch(c), U7::MIN, U7::MIN))
                    .collect(),
                MidiMessageSuperType::SystemRealTime => {
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
        assert_eq!(first.get_status_byte(), second.get_status_byte());
        assert_eq!(first.get_data_byte_1(), second.get_data_byte_1());
        assert_eq!(first.get_data_byte_2(), second.get_data_byte_2());
        assert_eq!(first.get_type(), second.get_type());
        assert_eq!(first.get_super_type(), second.get_super_type());
        assert_eq!(first.get_main_category(), second.get_main_category());
        assert_eq!(first.get_channel(), second.get_channel());
        assert_eq!(first.get_key_number(), second.get_key_number());
        assert_eq!(first.get_velocity(), second.get_velocity());
        assert_eq!(
            first.get_controller_number(),
            second.get_controller_number()
        );
        assert_eq!(first.get_control_value(), second.get_control_value());
        assert_eq!(first.get_pitch_bend_value(), second.get_pitch_bend_value());
        assert_eq!(first.get_pressure_amount(), second.get_pressure_amount());
        assert_eq!(first.get_program_number(), second.get_program_number());
        assert_eq!(first.is_note(), second.is_note());
        assert_eq!(first.is_note_on(), second.is_note_on());
        assert_eq!(first.is_note_off(), second.is_note_off());
    }
}
