use crate::{
    build_status_byte, extract_high_7_bit_value_from_14_bit_value,
    extract_low_7_bit_value_from_14_bit_value, Channel, ControllerNumber, KeyNumber, MidiMessage,
    MidiMessageFactory, MidiMessageType, MidiTimeCodeQuarterFrame, RawMidiMessage, U14, U4, U7,
};

/// MIDI message implemented as an enum where each variant contains exactly the data which is
/// relevant for the particular MIDI message type. This enum is primarily intended for read-only
/// usage via pattern matching. For that reason each variant is a struct-like enum, which is ideal
/// for pattern matching while it is less ideal for reuse (the data contained in the variant can't
/// be passed around in one piece).
#[derive(Clone, Debug, Eq, PartialEq)]
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
    // System exclusive messages
    SystemExclusiveStart,
    // System common messages
    MidiTimeCodeQuarterFrame(MidiTimeCodeQuarterFrame),
    SongPositionPointer {
        position: U14,
    },
    SongSelect {
        song_number: U7,
    },
    TuneRequest,
    SystemExclusiveEnd,
    // System real-time messages
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
        RawMidiMessage::from_bytes_unchecked(status_byte, data_byte_1, data_byte_2).to_structured()
    }

    // Optimization (although probably not used anyway)
    fn from_structured(msg: &StructuredMidiMessage) -> Self {
        msg.clone()
    }
}

impl MidiMessage for StructuredMidiMessage {
    fn get_status_byte(&self) -> u8 {
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

    fn get_data_byte_1(&self) -> U7 {
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

    fn get_data_byte_2(&self) -> U7 {
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
            MidiTimeCodeQuarterFrame(frame) => U7::MIN,
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

    // Optimization
    fn to_structured(&self) -> StructuredMidiMessage {
        self.clone()
    }
}
