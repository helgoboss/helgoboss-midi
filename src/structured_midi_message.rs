use crate::{
    build_status_byte, extract_high_7_bit_value_from_14_bit_value,
    extract_low_7_bit_value_from_14_bit_value, Channel, ControllerNumber, KeyNumber, MidiMessage,
    MidiMessageFactory, MidiMessageKind, ProgramNumber, RawMidiMessage, U14, U7,
};

/// MIDI message implemented as an enum where each variant contains exactly the data which is
/// relevant for the particular MIDI message kind. This enum is primarily intended for read-only
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
        program_number: ProgramNumber,
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
    MidiTimeCodeQuarterFrame,
    SongPositionPointer,
    SongSelect,
    TuneRequest,
    SystemExclusiveEnd,
    // System real-time messages
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
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
            NoteOff { channel, .. } => build_status_byte(MidiMessageKind::NoteOff.into(), *channel),
            NoteOn { channel, .. } => build_status_byte(MidiMessageKind::NoteOn.into(), *channel),
            PolyphonicKeyPressure { channel, .. } => {
                build_status_byte(MidiMessageKind::PolyphonicKeyPressure.into(), *channel)
            }
            ControlChange { channel, .. } => {
                build_status_byte(MidiMessageKind::ControlChange.into(), *channel)
            }
            ProgramChange { channel, .. } => {
                build_status_byte(MidiMessageKind::ProgramChange.into(), *channel)
            }
            ChannelPressure { channel, .. } => {
                build_status_byte(MidiMessageKind::ChannelPressure.into(), *channel)
            }
            PitchBendChange { channel, .. } => {
                build_status_byte(MidiMessageKind::PitchBendChange.into(), *channel)
            }
            SystemExclusiveStart => MidiMessageKind::SystemExclusiveStart.into(),
            MidiTimeCodeQuarterFrame => MidiMessageKind::MidiTimeCodeQuarterFrame.into(),
            SongPositionPointer => MidiMessageKind::SongPositionPointer.into(),
            SongSelect => MidiMessageKind::SongSelect.into(),
            TuneRequest => MidiMessageKind::TuneRequest.into(),
            SystemExclusiveEnd => MidiMessageKind::SystemExclusiveEnd.into(),
            TimingClock => MidiMessageKind::TimingClock.into(),
            Start => MidiMessageKind::Start.into(),
            Continue => MidiMessageKind::Continue.into(),
            Stop => MidiMessageKind::Stop.into(),
            ActiveSensing => MidiMessageKind::ActiveSensing.into(),
            SystemReset => MidiMessageKind::SystemReset.into(),
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
            _ => U7::MIN,
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
            _ => U7::MIN,
        }
    }

    // Optimization
    fn to_structured(&self) -> StructuredMidiMessage {
        self.clone()
    }
}
