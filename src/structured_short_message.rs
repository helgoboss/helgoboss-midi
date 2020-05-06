use crate::{
    build_14_bit_value_from_two_7_bit_values, build_status_byte, extract_channel_from_status_byte,
    extract_high_7_bit_value_from_14_bit_value, extract_low_7_bit_value_from_14_bit_value,
    extract_type_from_status_byte, Channel, ControllerNumber, KeyNumber, ShortMessage,
    ShortMessageFactory, ShortMessageType, TimeCodeQuarterFrame, U14, U7,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A short message implemented as an enum where each variant contains exactly the data which is
/// relevant for the particular message type.
///
/// This enum is primarily intended for read-only usage via pattern matching. For that reason each
/// variant is a struct-like enum, which is ideal for pattern matching while it is less ideal for
/// reuse (the data contained in the variant can't be passed around in one piece).
///
/// The enum's size in memory is currently 4 bytes.
///
/// # Example
///
/// ```
/// use helgoboss_midi::{
///     controller_numbers, Channel, RawShortMessage, ShortMessage, ShortMessageFactory,
///     StructuredShortMessage, U7,
/// };
///
/// let msg = RawShortMessage::control_change(
///     Channel::new(5),
///     controller_numbers::DAMPER_PEDAL_ON_OFF,
///     U7::new(100),
/// );
/// match msg.to_structured() {
///     StructuredShortMessage::ControlChange {
///         channel,
///         controller_number,
///         control_value,
///     } => {
///         assert_eq!(channel.get(), 5);
///         assert_eq!(controller_number.get(), 64);
///         assert_eq!(control_value.get(), 100);
///     }
///     _ => panic!("wrong type"),
/// };
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StructuredShortMessage {
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
    TimeCodeQuarterFrame(TimeCodeQuarterFrame),
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

impl ShortMessageFactory for StructuredShortMessage {
    unsafe fn from_bytes_unchecked((status_byte, data_byte_1, data_byte_2): (u8, U7, U7)) -> Self {
        use ShortMessageType::*;
        let r#type = extract_type_from_status_byte(status_byte).expect("invalid status byte");
        match r#type {
            NoteOff => StructuredShortMessage::NoteOff {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                velocity: data_byte_2,
            },
            NoteOn => StructuredShortMessage::NoteOn {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                velocity: data_byte_2,
            },
            PolyphonicKeyPressure => StructuredShortMessage::PolyphonicKeyPressure {
                channel: extract_channel_from_status_byte(status_byte),
                key_number: data_byte_1.into(),
                pressure_amount: data_byte_2,
            },
            ControlChange => StructuredShortMessage::ControlChange {
                channel: extract_channel_from_status_byte(status_byte),
                controller_number: data_byte_1.into(),
                control_value: data_byte_2,
            },
            ProgramChange => StructuredShortMessage::ProgramChange {
                channel: extract_channel_from_status_byte(status_byte),
                program_number: data_byte_1.into(),
            },
            ChannelPressure => StructuredShortMessage::ChannelPressure {
                channel: extract_channel_from_status_byte(status_byte),
                pressure_amount: data_byte_1,
            },
            PitchBendChange => StructuredShortMessage::PitchBendChange {
                channel: extract_channel_from_status_byte(status_byte),
                pitch_bend_value: build_14_bit_value_from_two_7_bit_values(
                    data_byte_2,
                    data_byte_1,
                ),
            },
            SystemExclusiveStart => StructuredShortMessage::SystemExclusiveStart,
            TimeCodeQuarterFrame => {
                StructuredShortMessage::TimeCodeQuarterFrame(data_byte_1.into())
            }
            SongPositionPointer => StructuredShortMessage::SongPositionPointer {
                position: build_14_bit_value_from_two_7_bit_values(data_byte_2, data_byte_1),
            },
            SongSelect => StructuredShortMessage::SongSelect {
                song_number: data_byte_1,
            },
            TuneRequest => StructuredShortMessage::TuneRequest,
            SystemExclusiveEnd => StructuredShortMessage::SystemExclusiveEnd,
            TimingClock => StructuredShortMessage::TimingClock,
            Start => StructuredShortMessage::Start,
            Continue => StructuredShortMessage::Continue,
            Stop => StructuredShortMessage::Stop,
            ActiveSensing => StructuredShortMessage::ActiveSensing,
            SystemReset => StructuredShortMessage::SystemReset,
            SystemCommonUndefined1 => StructuredShortMessage::SystemCommonUndefined1,
            SystemCommonUndefined2 => StructuredShortMessage::SystemCommonUndefined2,
            SystemRealTimeUndefined1 => StructuredShortMessage::SystemRealTimeUndefined1,
            SystemRealTimeUndefined2 => StructuredShortMessage::SystemRealTimeUndefined2,
        }
    }
}

impl ShortMessage for StructuredShortMessage {
    fn status_byte(&self) -> u8 {
        use StructuredShortMessage::*;
        match self {
            NoteOff { channel, .. } => {
                build_status_byte(ShortMessageType::NoteOff.into(), *channel)
            }
            NoteOn { channel, .. } => build_status_byte(ShortMessageType::NoteOn.into(), *channel),
            PolyphonicKeyPressure { channel, .. } => {
                build_status_byte(ShortMessageType::PolyphonicKeyPressure.into(), *channel)
            }
            ControlChange { channel, .. } => {
                build_status_byte(ShortMessageType::ControlChange.into(), *channel)
            }
            ProgramChange { channel, .. } => {
                build_status_byte(ShortMessageType::ProgramChange.into(), *channel)
            }
            ChannelPressure { channel, .. } => {
                build_status_byte(ShortMessageType::ChannelPressure.into(), *channel)
            }
            PitchBendChange { channel, .. } => {
                build_status_byte(ShortMessageType::PitchBendChange.into(), *channel)
            }
            SystemExclusiveStart => ShortMessageType::SystemExclusiveStart.into(),
            TimeCodeQuarterFrame(_) => ShortMessageType::TimeCodeQuarterFrame.into(),
            SongPositionPointer { .. } => ShortMessageType::SongPositionPointer.into(),
            SongSelect { .. } => ShortMessageType::SongSelect.into(),
            TuneRequest => ShortMessageType::TuneRequest.into(),
            SystemExclusiveEnd => ShortMessageType::SystemExclusiveEnd.into(),
            TimingClock => ShortMessageType::TimingClock.into(),
            Start => ShortMessageType::Start.into(),
            Continue => ShortMessageType::Continue.into(),
            Stop => ShortMessageType::Stop.into(),
            ActiveSensing => ShortMessageType::ActiveSensing.into(),
            SystemReset => ShortMessageType::SystemReset.into(),
            SystemCommonUndefined1 => ShortMessageType::SystemCommonUndefined1.into(),
            SystemCommonUndefined2 => ShortMessageType::SystemCommonUndefined2.into(),
            SystemRealTimeUndefined1 => ShortMessageType::SystemRealTimeUndefined1.into(),
            SystemRealTimeUndefined2 => ShortMessageType::SystemRealTimeUndefined2.into(),
        }
    }

    fn data_byte_1(&self) -> U7 {
        use StructuredShortMessage::*;
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
            TimeCodeQuarterFrame(frame) => (*frame).into(),
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
        use StructuredShortMessage::*;
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
            TimeCodeQuarterFrame(_) => U7::MIN,
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
    fn to_structured(&self) -> StructuredShortMessage {
        self.clone()
    }
}
