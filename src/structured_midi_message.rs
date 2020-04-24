use crate::{
    build_14_bit_value_from_two_7_bit_values, build_status_byte, extract_channel_from_status_byte,
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value,
    extract_type_from_status_byte, Channel, ControllerNumber, KeyNumber, MidiMessage,
    MidiMessageFactory, MidiMessageType, MidiTimeCodeQuarterFrame, U14, U7,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single MIDI message implemented as an enum where each variant contains exactly the data which
/// is relevant for the particular message type. This enum is primarily intended for read-only usage
/// via pattern matching. For that reason each variant is a struct-like enum, which is ideal for
/// pattern matching while it is less ideal for reuse (the data contained in the variant can't
/// be passed around in one piece).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StructuredMidiMessage {
    // Channel messages
    NoteOff {
        channel: Channel,
        key_number: KeyNumber,
        velocity: U7,
    },
    NoteOn {
        channel: Channel,
        key_number: KeyNumber,
        velocity: U7,
    },
    PolyphonicKeyPressure {
        channel: Channel,
        key_number: KeyNumber,
        pressure_amount: U7,
    },
    ControlChange {
        channel: Channel,
        controller_number: ControllerNumber,
        control_value: U7,
    },
    ProgramChange {
        channel: Channel,
        program_number: U7,
    },
    ChannelPressure {
        channel: Channel,
        pressure_amount: U7,
    },
    PitchBendChange {
        channel: Channel,
        pitch_bend_value: U14,
    },
    // System Exclusive messages
    SystemExclusiveStart,
    // System Common messages
    MidiTimeCodeQuarterFrame(MidiTimeCodeQuarterFrame),
    SongPositionPointer {
        position: U14,
    },
    SongSelect {
        song_number: U7,
    },
    TuneRequest,
    SystemExclusiveEnd,
    // System Real Time messages
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
    SystemCommonUndefined1,
    SystemCommonUndefined2,
    SystemRealTimeUndefined1,
    SystemRealTimeUndefined2,
}

impl MidiMessageFactory for StructuredMidiMessage {
    unsafe fn from_bytes_unchecked(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Self {
        use MidiMessageType::*;
        let r#type = extract_type_from_status_byte(status_byte).unwrap();
        match r#type {
            NoteOff => StructuredMidiMessage::NoteOff {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                velocity: data_byte_2,
            },
            NoteOn => StructuredMidiMessage::NoteOn {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                velocity: data_byte_2,
            },
            PolyphonicKeyPressure => StructuredMidiMessage::PolyphonicKeyPressure {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                pressure_amount: data_byte_2,
            },
            ControlChange => StructuredMidiMessage::ControlChange {
                channel: extract_channel_from_status_byte(status_byte),
                controller_number: data_byte_1.into(),
                control_value: data_byte_2,
            },
            ProgramChange => StructuredMidiMessage::ProgramChange {
                channel: extract_channel_from_status_byte(status_byte),
                program_number: data_byte_1.into(),
            },
            ChannelPressure => StructuredMidiMessage::ChannelPressure {
                channel: extract_channel_from_status_byte(status_byte),
                pressure_amount: data_byte_1,
            },
            PitchBendChange => StructuredMidiMessage::PitchBendChange {
                channel: extract_channel_from_status_byte(status_byte),
                pitch_bend_value: build_14_bit_value_from_two_7_bit_values(
                    data_byte_2,
                    data_byte_1,
                ),
            },
            SystemExclusiveStart => StructuredMidiMessage::SystemExclusiveStart,
            MidiTimeCodeQuarterFrame => {
                StructuredMidiMessage::MidiTimeCodeQuarterFrame(data_byte_1.into())
            }
            SongPositionPointer => StructuredMidiMessage::SongPositionPointer {
                position: build_14_bit_value_from_two_7_bit_values(data_byte_2, data_byte_1),
            },
            SongSelect => StructuredMidiMessage::SongSelect {
                song_number: data_byte_1,
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
}

impl MidiMessage for StructuredMidiMessage {
    fn status_byte(&self) -> u8 {
        use StructuredMidiMessage::*;
        match self {
            NoteOff { channel, .. } => build_status_byte(MidiMessageType::NoteOff.into(), *channel),
            NoteOn { channel, .. } => build_status_byte(MidiMessageType::NoteOn.into(), *channel),
            PolyphonicKeyPressure { channel, .. } => {
                build_status_byte(MidiMessageType::PolyphonicKeyPressure.into(), *channel)
            }
            ControlChange { channel, .. } => {
                build_status_byte(MidiMessageType::ControlChange.into(), *channel)
            }
            ProgramChange { channel, .. } => {
                build_status_byte(MidiMessageType::ProgramChange.into(), *channel)
            }
            ChannelPressure { channel, .. } => {
                build_status_byte(MidiMessageType::ChannelPressure.into(), *channel)
            }
            PitchBendChange { channel, .. } => {
                build_status_byte(MidiMessageType::PitchBendChange.into(), *channel)
            }
            SystemExclusiveStart => MidiMessageType::SystemExclusiveStart.into(),
            MidiTimeCodeQuarterFrame(_) => MidiMessageType::MidiTimeCodeQuarterFrame.into(),
            SongPositionPointer { .. } => MidiMessageType::SongPositionPointer.into(),
            SongSelect { .. } => MidiMessageType::SongSelect.into(),
            TuneRequest => MidiMessageType::TuneRequest.into(),
            SystemExclusiveEnd => MidiMessageType::SystemExclusiveEnd.into(),
            TimingClock => MidiMessageType::TimingClock.into(),
            Start => MidiMessageType::Start.into(),
            Continue => MidiMessageType::Continue.into(),
            Stop => MidiMessageType::Stop.into(),
            ActiveSensing => MidiMessageType::ActiveSensing.into(),
            SystemReset => MidiMessageType::SystemReset.into(),
            SystemCommonUndefined1 => MidiMessageType::SystemCommonUndefined1.into(),
            SystemCommonUndefined2 => MidiMessageType::SystemCommonUndefined2.into(),
            SystemRealTimeUndefined1 => MidiMessageType::SystemRealTimeUndefined1.into(),
            SystemRealTimeUndefined2 => MidiMessageType::SystemRealTimeUndefined2.into(),
        }
    }

    fn data_byte_1(&self) -> U7 {
        use StructuredMidiMessage::*;
        match self {
            NoteOff { key_number, .. } => (*key_number).into(),
            NoteOn { key_number, .. } => (*key_number).into(),
            PolyphonicKeyPressure { key_number, .. } => (*key_number).into(),
            ControlChange {
                controller_number, ..
            } => (*controller_number).into(),
            ProgramChange { program_number, .. } => (*program_number).into(),
            ChannelPressure {
                pressure_amount, ..
            } => *pressure_amount,
            PitchBendChange {
                pitch_bend_value, ..
            } => extract_low_7_bit_value_from_14_bit_value(*pitch_bend_value),
            SystemExclusiveStart => U7::MIN,
            MidiTimeCodeQuarterFrame(frame) => (*frame).into(),
            SongPositionPointer { position } => {
                extract_low_7_bit_value_from_14_bit_value(*position)
            }
            SongSelect { song_number } => *song_number,
            TuneRequest => U7::MIN,
            SystemExclusiveEnd => U7::MIN,
            TimingClock => U7::MIN,
            Start => U7::MIN,
            Continue => U7::MIN,
            Stop => U7::MIN,
            ActiveSensing => U7::MIN,
            SystemReset => U7::MIN,
            SystemCommonUndefined1 => U7::MIN,
            SystemCommonUndefined2 => U7::MIN,
            SystemRealTimeUndefined1 => U7::MIN,
            SystemRealTimeUndefined2 => U7::MIN,
        }
    }

    fn data_byte_2(&self) -> U7 {
        use StructuredMidiMessage::*;
        match self {
            NoteOff { velocity, .. } => *velocity,
            NoteOn { velocity, .. } => *velocity,
            PolyphonicKeyPressure {
                pressure_amount, ..
            } => *pressure_amount,
            ControlChange { control_value, .. } => *control_value,
            ProgramChange { .. } => U7::MIN,
            ChannelPressure { .. } => U7::MIN,
            PitchBendChange {
                pitch_bend_value, ..
            } => extract_high_7_bit_value_from_14_bit_value(*pitch_bend_value),
            SystemExclusiveStart => U7::MIN,
            MidiTimeCodeQuarterFrame(_) => U7::MIN,
            SongPositionPointer { position } => {
                extract_high_7_bit_value_from_14_bit_value(*position)
            }
            SongSelect { .. } => U7::MIN,
            TuneRequest => U7::MIN,
            SystemExclusiveEnd => U7::MIN,
            TimingClock => U7::MIN,
            Start => U7::MIN,
            Continue => U7::MIN,
            Stop => U7::MIN,
            ActiveSensing => U7::MIN,
            SystemReset => U7::MIN,
            SystemCommonUndefined1 => U7::MIN,
            SystemCommonUndefined2 => U7::MIN,
            SystemRealTimeUndefined1 => U7::MIN,
            SystemRealTimeUndefined2 => U7::MIN,
        }
    }

    // Slight optimization
    fn to_structured(&self) -> StructuredMidiMessage {
        self.clone()
    }
}
