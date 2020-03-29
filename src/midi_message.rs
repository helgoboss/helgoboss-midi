use crate::{
    build_14_bit_value_from_two_7_bit_values, build_byte_from_nibbles,
    extract_high_7_bit_value_from_14_bit_value, extract_high_nibble_from_byte,
    extract_low_7_bit_value_from_14_bit_value, extract_low_nibble_from_byte, with_low_nibble_added,
    Byte, FourteenBitValue, Nibble, SevenBitValue,
};
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::convert::TryInto;
#[allow(unused_imports)]
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Trait to be implemented by struct representing MIDI message. Only the three byte-returning
/// methods need to be implemented, the rest is done by default methods. The advantage of this
/// architecture is that we can have a unified API, no matter which underlying data structure is
/// used.
///
/// Please also recommend the trait `MidiMessageFactory` for your struct if creating new MIDI
/// messages programmatically should be supported.
pub trait MidiMessage {
    fn get_status_byte(&self) -> Byte;

    fn get_data_byte_1(&self) -> SevenBitValue;

    fn get_data_byte_2(&self) -> SevenBitValue;

    fn get_kind(&self) -> MidiMessageKind {
        get_midi_message_kind_from_status_byte(self.get_status_byte()).unwrap()
    }

    fn get_super_kind(&self) -> MidiMessageSuperKind {
        self.get_kind().get_super_kind()
    }

    fn get_main_category(&self) -> MidiMessageMainCategory {
        self.get_super_kind().get_main_category()
    }

    fn to_structured(&self) -> StructuredMidiMessage {
        use MidiMessageKind::*;
        match self.get_kind() {
            NoteOff => StructuredMidiMessage::NoteOff(NoteData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                key_number: self.get_data_byte_1(),
                velocity: self.get_data_byte_2(),
            }),
            NoteOn => StructuredMidiMessage::NoteOn(NoteData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                key_number: self.get_data_byte_1(),
                velocity: self.get_data_byte_2(),
            }),
            PolyphonicKeyPressure => {
                StructuredMidiMessage::PolyphonicKeyPressure(PolyphonicKeyPressureData {
                    channel: extract_low_nibble_from_byte(self.get_status_byte()),
                    key_number: self.get_data_byte_1(),
                    pressure_amount: self.get_data_byte_2(),
                })
            }
            ControlChange => StructuredMidiMessage::ControlChange(ControlChangeData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                controller_number: self.get_data_byte_1(),
                control_value: self.get_data_byte_2(),
            }),
            ProgramChange => StructuredMidiMessage::ProgramChange(ProgramChangeData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                program_number: self.get_data_byte_1(),
            }),
            ChannelPressure => StructuredMidiMessage::ChannelPressure(ChannelPressureData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                pressure_amount: self.get_data_byte_1(),
            }),
            PitchBendChange => StructuredMidiMessage::PitchBendChange(PitchBendChangeData {
                channel: extract_low_nibble_from_byte(self.get_status_byte()),
                pitch_bend_value: build_14_bit_value_from_two_7_bit_values(
                    self.get_data_byte_2(),
                    self.get_data_byte_1(),
                ),
            }),
            SystemExclusiveStart => StructuredMidiMessage::SystemExclusiveStart,
            MidiTimeCodeQuarterFrame => StructuredMidiMessage::MidiTimeCodeQuarterFrame,
            SongPositionPointer => StructuredMidiMessage::SongPositionPointer,
            SongSelect => StructuredMidiMessage::SongSelect,
            TuneRequest => StructuredMidiMessage::TuneRequest,
            SystemExclusiveEnd => StructuredMidiMessage::SystemExclusiveEnd,
            TimingClock => StructuredMidiMessage::TimingClock,
            Start => StructuredMidiMessage::Start,
            Continue => StructuredMidiMessage::Continue,
            Stop => StructuredMidiMessage::Stop,
            ActiveSensing => StructuredMidiMessage::ActiveSensing,
            SystemReset => StructuredMidiMessage::SystemReset,
        }
    }

    // Returns false if the message kind is NoteOn but the velocity is 0
    fn is_note_on(&self) -> bool {
        match self.to_structured() {
            StructuredMidiMessage::NoteOn(data) if data.velocity > 0 => true,
            _ => false,
        }
    }

    // Also returns true if the message kind is NoteOn but the velocity is 0
    fn is_note_off(&self) -> bool {
        use StructuredMidiMessage::*;
        match self.to_structured() {
            NoteOff(_) => true,
            NoteOn(data) if data.velocity == 0 => true,
            _ => false,
        }
    }

    fn is_note(&self) -> bool {
        match self.get_kind() {
            MidiMessageKind::NoteOn | MidiMessageKind::NoteOff => true,
            _ => false,
        }
    }

    fn get_channel(&self) -> Option<Nibble> {
        if self.get_main_category() != MidiMessageMainCategory::Channel {
            return None;
        }
        Some(extract_low_nibble_from_byte(self.get_status_byte()))
    }

    fn get_key_number(&self) -> Option<SevenBitValue> {
        use MidiMessageKind::*;
        match self.get_kind() {
            NoteOff | NoteOn | PolyphonicKeyPressure => Some(self.get_data_byte_1()),
            _ => None,
        }
    }

    fn get_velocity(&self) -> Option<SevenBitValue> {
        use MidiMessageKind::*;
        match self.get_kind() {
            NoteOff | NoteOn => Some(self.get_data_byte_2()),
            _ => None,
        }
    }

    fn get_controller_number(&self) -> Option<SevenBitValue> {
        if self.get_kind() != MidiMessageKind::ControlChange {
            return None;
        }
        Some(self.get_data_byte_1())
    }

    fn get_control_value(&self) -> Option<SevenBitValue> {
        if self.get_kind() != MidiMessageKind::ControlChange {
            return None;
        }
        Some(self.get_data_byte_2())
    }

    fn get_program_number(&self) -> Option<SevenBitValue> {
        if self.get_kind() != MidiMessageKind::ProgramChange {
            return None;
        }
        Some(self.get_data_byte_1())
    }

    fn get_pressure_amount(&self) -> Option<SevenBitValue> {
        use MidiMessageKind::*;
        match self.get_kind() {
            PolyphonicKeyPressure => Some(self.get_data_byte_2()),
            ChannelPressure => Some(self.get_data_byte_1()),
            _ => None,
        }
    }

    fn get_pitch_bend_value(&self) -> Option<FourteenBitValue> {
        if self.get_kind() != MidiMessageKind::PitchBendChange {
            return None;
        }
        Some(build_14_bit_value_from_two_7_bit_values(
            self.get_data_byte_2(),
            self.get_data_byte_1(),
        ))
    }
}

/// Trait to be implemented by struct representing a MIDI message if it supports creation of various
/// types of MIDI messages. Only one method needs to be implemented, the rest is done by default
/// methods. The advantage of this architecture is that we can have a unified factory API, no matter
/// which underlying data structure is used.
pub trait MidiMessageFactory: Sized {
    unsafe fn from_bytes_raw(
        status_byte: Byte,
        data_byte_1: SevenBitValue,
        data_byte_2: SevenBitValue,
    ) -> Self;

    // Although we could argue that calling this function with illegal input values is a violation
    // of its contract, this function returns a result rather than panicking. It's because - unlike
    // the convenience factory functions - this function is primarily intended to be used in
    // situations where the bytes come from somewhere else (e.g. are user-generated) and therefore
    // acts a bit like a parse function where client code should be able to recover from wrong
    // input.
    fn from_bytes(
        status_byte: Byte,
        data_byte_1: SevenBitValue,
        data_byte_2: SevenBitValue,
    ) -> Result<Self, &'static str> {
        get_midi_message_kind_from_status_byte(status_byte).map_err(|_| "Unknown status byte")?;
        if data_byte_1 >= 0x7f {
            return Err("Data byte 1 is too large");
        }
        if data_byte_2 >= 0x7f {
            return Err("Data byte 2 is too large");
        }
        Ok(unsafe { Self::from_bytes_raw(status_byte, data_byte_1, data_byte_2) })
    }

    fn from_structured(msg: &StructuredMidiMessage) -> Self {
        unsafe {
            Self::from_bytes_raw(
                msg.get_status_byte(),
                msg.get_data_byte_1(),
                msg.get_data_byte_2(),
            )
        }
    }

    fn channel_message(
        kind: MidiMessageKind,
        channel: Nibble,
        data_1: SevenBitValue,
        data_2: SevenBitValue,
    ) -> Self {
        debug_assert_eq!(
            kind.get_super_kind().get_main_category(),
            MidiMessageMainCategory::Channel
        );
        debug_assert!(channel < 16);
        debug_assert!(data_1 < 128);
        debug_assert!(data_2 < 128);
        unsafe { Self::from_bytes_raw(with_low_nibble_added(kind.into(), channel), data_1, data_2) }
    }

    // TODO Create factory methods for system-common and system-exclusive messages
    fn system_real_time_message(kind: MidiMessageKind) -> Self {
        debug_assert_eq!(kind.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
        unsafe { Self::from_bytes_raw(kind.into(), 0, 0) }
    }

    fn note_on(channel: Nibble, key_number: SevenBitValue, velocity: SevenBitValue) -> Self {
        Self::channel_message(MidiMessageKind::NoteOn, channel, key_number, velocity)
    }

    fn note_off(channel: Nibble, key_number: SevenBitValue, velocity: SevenBitValue) -> Self {
        Self::channel_message(MidiMessageKind::NoteOff, channel, key_number, velocity)
    }

    fn control_change(
        channel: Nibble,
        controller_number: SevenBitValue,
        control_value: SevenBitValue,
    ) -> Self {
        Self::channel_message(
            MidiMessageKind::ControlChange,
            channel,
            controller_number,
            control_value,
        )
    }

    fn program_change(channel: Nibble, program_number: SevenBitValue) -> Self {
        Self::channel_message(MidiMessageKind::ProgramChange, channel, program_number, 0)
    }

    fn polyphonic_key_pressure(
        channel: Nibble,
        key_number: SevenBitValue,
        pressure_amount: SevenBitValue,
    ) -> Self {
        Self::channel_message(
            MidiMessageKind::PolyphonicKeyPressure,
            channel,
            key_number,
            pressure_amount,
        )
    }

    fn channel_pressure(channel: Nibble, pressure_amount: SevenBitValue) -> Self {
        Self::channel_message(
            MidiMessageKind::ChannelPressure,
            channel,
            pressure_amount,
            0,
        )
    }
    fn pitch_bend_change(channel: Nibble, pitch_bend_value: FourteenBitValue) -> Self {
        Self::channel_message(
            MidiMessageKind::PitchBendChange,
            channel,
            (pitch_bend_value & 0x7f) as SevenBitValue,
            (pitch_bend_value >> 7) as SevenBitValue,
        )
    }
    fn timing_clock() -> Self {
        Self::system_real_time_message(MidiMessageKind::TimingClock)
    }
    fn start() -> Self {
        Self::system_real_time_message(MidiMessageKind::Start)
    }
    fn continue_message() -> Self {
        Self::system_real_time_message(MidiMessageKind::Continue)
    }
    fn stop() -> Self {
        Self::system_real_time_message(MidiMessageKind::Stop)
    }
    fn active_sensing() -> Self {
        Self::system_real_time_message(MidiMessageKind::ActiveSensing)
    }
    fn system_reset() -> Self {
        Self::system_real_time_message(MidiMessageKind::SystemReset)
    }
}

// The most low-level kind of a MIDI message
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumIter)]
#[repr(u8)]
pub enum MidiMessageKind {
    // Channel messages = channel voice messages + channel mode messages (given value represents
    // channel 0 status byte)
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyphonicKeyPressure = 0xa0,
    ControlChange = 0xb0,
    ProgramChange = 0xc0,
    ChannelPressure = 0xd0,
    PitchBendChange = 0xe0,
    // System exclusive messages
    SystemExclusiveStart = 0xf0,
    // System common messages
    MidiTimeCodeQuarterFrame = 0xf1,
    SongPositionPointer = 0xf2,
    SongSelect = 0xf3,
    TuneRequest = 0xf6,
    SystemExclusiveEnd = 0xf7,
    // System real-time messages (given value represents the complete status byte)
    TimingClock = 0xf8,
    Start = 0xfa,
    Continue = 0xfb,
    Stop = 0xfc,
    ActiveSensing = 0xfe,
    SystemReset = 0xff,
}

impl MidiMessageKind {
    pub fn get_super_kind(&self) -> MidiMessageSuperKind {
        use MidiMessageKind::*;
        match self {
            NoteOn
            | NoteOff
            | ChannelPressure
            | PolyphonicKeyPressure
            | PitchBendChange
            | ProgramChange
            | ControlChange => MidiMessageSuperKind::Channel,
            TimingClock | Start | Continue | Stop | ActiveSensing | SystemReset => {
                MidiMessageSuperKind::SystemRealTime
            }
            MidiTimeCodeQuarterFrame
            | SongPositionPointer
            | SongSelect
            | TuneRequest
            | SystemExclusiveEnd => MidiMessageSuperKind::SystemCommon,
            SystemExclusiveStart => MidiMessageSuperKind::SystemExclusive,
        }
    }
}

// A somewhat mid-level kind of a MIDI message.
// In this enum we don't distinguish between channel voice and channel mode messages because this
// difference doesn't solely depend on the MidiMessageKind (channel mode messages are just
// particular ControlChange messages).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MidiMessageSuperKind {
    Channel,
    SystemCommon,
    SystemRealTime,
    SystemExclusive,
}

impl MidiMessageSuperKind {
    pub fn get_main_category(&self) -> MidiMessageMainCategory {
        if *self == MidiMessageSuperKind::Channel {
            MidiMessageMainCategory::Channel
        } else {
            MidiMessageMainCategory::System
        }
    }
}

// At the highest level MIDI messages are put into two categories
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MidiMessageMainCategory {
    Channel,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuredMidiMessage {
    // Channel messages
    NoteOff(NoteData),
    NoteOn(NoteData),
    PolyphonicKeyPressure(PolyphonicKeyPressureData),
    ControlChange(ControlChangeData),
    ProgramChange(ProgramChangeData),
    ChannelPressure(ChannelPressureData),
    PitchBendChange(PitchBendChangeData),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteData {
    pub channel: Nibble,
    pub key_number: SevenBitValue,
    pub velocity: SevenBitValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlChangeData {
    pub channel: Nibble,
    pub controller_number: SevenBitValue,
    pub control_value: SevenBitValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramChangeData {
    pub channel: Nibble,
    pub program_number: SevenBitValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolyphonicKeyPressureData {
    pub channel: Nibble,
    pub key_number: SevenBitValue,
    pub pressure_amount: SevenBitValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelPressureData {
    pub channel: Nibble,
    pub pressure_amount: SevenBitValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PitchBendChangeData {
    pub channel: Nibble,
    pub pitch_bend_value: FourteenBitValue,
}

impl MidiMessageFactory for StructuredMidiMessage {
    unsafe fn from_bytes_raw(status_byte: u8, data_byte_1: u8, data_byte_2: u8) -> Self {
        RawMidiMessage::from_bytes_raw(status_byte, data_byte_1, data_byte_2).to_structured()
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
            NoteOff(data) => with_low_nibble_added(MidiMessageKind::NoteOff.into(), data.channel),
            NoteOn(data) => with_low_nibble_added(MidiMessageKind::NoteOn.into(), data.channel),
            PolyphonicKeyPressure(data) => {
                with_low_nibble_added(MidiMessageKind::PolyphonicKeyPressure.into(), data.channel)
            }
            ControlChange(data) => {
                with_low_nibble_added(MidiMessageKind::ControlChange.into(), data.channel)
            }
            ProgramChange(data) => {
                with_low_nibble_added(MidiMessageKind::ProgramChange.into(), data.channel)
            }
            ChannelPressure(data) => {
                with_low_nibble_added(MidiMessageKind::ChannelPressure.into(), data.channel)
            }
            PitchBendChange(data) => {
                with_low_nibble_added(MidiMessageKind::PitchBendChange.into(), data.channel)
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

    fn get_data_byte_1(&self) -> u8 {
        use StructuredMidiMessage::*;
        match self {
            NoteOff(data) => data.key_number,
            NoteOn(data) => data.key_number,
            PolyphonicKeyPressure(data) => data.key_number,
            ControlChange(data) => data.controller_number,
            ProgramChange(data) => data.program_number,
            ChannelPressure(data) => data.pressure_amount,
            PitchBendChange(data) => {
                extract_low_7_bit_value_from_14_bit_value(data.pitch_bend_value)
            }
            _ => 0,
        }
    }

    fn get_data_byte_2(&self) -> u8 {
        use StructuredMidiMessage::*;
        match self {
            NoteOff(data) => data.velocity,
            NoteOn(data) => data.velocity,
            PolyphonicKeyPressure(data) => data.pressure_amount,
            ControlChange(data) => data.control_value,
            ProgramChange(_data) => 0,
            ChannelPressure(_data) => 0,
            PitchBendChange(data) => {
                extract_high_7_bit_value_from_14_bit_value(data.pitch_bend_value)
            }
            _ => 0,
        }
    }

    // Optimization
    fn to_structured(&self) -> StructuredMidiMessage {
        self.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawMidiMessage {
    status_byte: u8,
    data_byte_1: u8,
    data_byte_2: u8,
}

impl MidiMessageFactory for RawMidiMessage {
    unsafe fn from_bytes_raw(status_byte: u8, data_byte_1: u8, data_byte_2: u8) -> Self {
        Self {
            status_byte,
            data_byte_1,
            data_byte_2,
        }
    }
}

impl MidiMessage for RawMidiMessage {
    fn get_status_byte(&self) -> u8 {
        self.status_byte
    }

    fn get_data_byte_1(&self) -> u8 {
        self.data_byte_1
    }

    fn get_data_byte_2(&self) -> u8 {
        self.data_byte_2
    }
}

fn get_midi_message_kind_from_status_byte(
    status_byte: Byte,
) -> Result<MidiMessageKind, TryFromPrimitiveError<MidiMessageKind>> {
    let high_status_byte_nibble = extract_high_nibble_from_byte(status_byte);
    if high_status_byte_nibble == 0xf {
        // System message. The complete status byte makes up the kind.
        status_byte.try_into()
    } else {
        // Channel message. Just the high nibble of the status byte makes up the kind
        // (low nibble encodes channel).
        build_byte_from_nibbles(high_status_byte_nibble, 0).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes_ok() {
        // Given
        let msg = RawMidiMessage::from_bytes(145, 64, 100).unwrap();
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), 64);
        assert_eq!(msg.get_data_byte_2(), 100);
        assert_eq!(msg.get_kind(), MidiMessageKind::NoteOn);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(1));
        assert_eq!(msg.get_key_number(), Some(64));
        assert_eq!(msg.get_velocity(), Some(100));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::NoteOn(NoteData {
                channel: 1,
                key_number: 64,
                velocity: 100
            })
        );
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn from_bytes_err() {
        // Given
        let msg = RawMidiMessage::from_bytes(2, 64, 100);
        // When
        // Then
        assert!(msg.is_err());
    }

    #[test]
    fn note_on() {
        // Given
        let msg = RawMidiMessage::note_on(1, 64, 100);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), 64);
        assert_eq!(msg.get_data_byte_2(), 100);
        assert_eq!(msg.get_kind(), MidiMessageKind::NoteOn);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(1));
        assert_eq!(msg.get_key_number(), Some(64));
        assert_eq!(msg.get_velocity(), Some(100));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::NoteOn(NoteData {
                channel: 1,
                key_number: 64,
                velocity: 100
            })
        );
        assert!(msg.is_note());
        assert!(msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn real_note_off() {
        // Given
        let msg = RawMidiMessage::note_off(2, 125, 70);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0x82);
        assert_eq!(msg.get_data_byte_1(), 125);
        assert_eq!(msg.get_data_byte_2(), 70);
        assert_eq!(msg.get_kind(), MidiMessageKind::NoteOff);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(2));
        assert_eq!(msg.get_key_number(), Some(125));
        assert_eq!(msg.get_velocity(), Some(70));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::NoteOff(NoteData {
                channel: 2,
                key_number: 125,
                velocity: 70
            })
        );
        assert!(msg.is_note());
        assert!(!msg.is_note_on());
        assert!(msg.is_note_off());
    }

    #[test]
    fn fake_note_off() {
        // Given
        let msg = RawMidiMessage::note_on(0, 5, 0);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0x90);
        assert_eq!(msg.get_data_byte_1(), 5);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::NoteOn);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(0));
        assert_eq!(msg.get_key_number(), Some(5));
        assert_eq!(msg.get_velocity(), Some(0));
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::NoteOn(NoteData {
                channel: 0,
                key_number: 5,
                velocity: 0
            })
        );
        assert!(msg.is_note());
        assert!(!msg.is_note_on());
        assert!(msg.is_note_off());
    }

    #[test]
    fn control_change() {
        // Given
        let msg = RawMidiMessage::control_change(1, 50, 2);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xb1);
        assert_eq!(msg.get_data_byte_1(), 50);
        assert_eq!(msg.get_data_byte_2(), 2);
        assert_eq!(msg.get_kind(), MidiMessageKind::ControlChange);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(1));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), Some(50));
        assert_eq!(msg.get_control_value(), Some(2));
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::ControlChange(ControlChangeData {
                channel: 1,
                controller_number: 50,
                control_value: 2
            })
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn program_change() {
        // Given
        let msg = RawMidiMessage::program_change(4, 22);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xc4);
        assert_eq!(msg.get_data_byte_1(), 22);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::ProgramChange);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(4));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), Some(22));
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::ProgramChange(ProgramChangeData {
                channel: 4,
                program_number: 22
            })
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn polyphonic_key_pressure() {
        // Given
        let msg = RawMidiMessage::polyphonic_key_pressure(15, 127, 50);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xaf);
        assert_eq!(msg.get_data_byte_1(), 127);
        assert_eq!(msg.get_data_byte_2(), 50);
        assert_eq!(msg.get_kind(), MidiMessageKind::PolyphonicKeyPressure);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(15));
        assert_eq!(msg.get_key_number(), Some(127));
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), Some(50));
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::PolyphonicKeyPressure(PolyphonicKeyPressureData {
                channel: 15,
                key_number: 127,
                pressure_amount: 50
            })
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn channel_pressure() {
        // Given
        let msg = RawMidiMessage::channel_pressure(14, 0);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xde);
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::ChannelPressure);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(14));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), None);
        assert_eq!(msg.get_pressure_amount(), Some(0));
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::ChannelPressure(ChannelPressureData {
                channel: 14,
                pressure_amount: 0
            })
        );
        assert!(!msg.is_note());
        assert!(!msg.is_note_on());
        assert!(!msg.is_note_off());
    }

    #[test]
    fn pitch_bend_change() {
        // Given
        let msg = RawMidiMessage::pitch_bend_change(1, 1278);
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xe1);
        assert_eq!(msg.get_data_byte_1(), 126);
        assert_eq!(msg.get_data_byte_2(), 9);
        assert_eq!(msg.get_kind(), MidiMessageKind::PitchBendChange);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(1));
        assert_eq!(msg.get_key_number(), None);
        assert_eq!(msg.get_velocity(), None);
        assert_eq!(msg.get_controller_number(), None);
        assert_eq!(msg.get_control_value(), None);
        assert_eq!(msg.get_pitch_bend_value(), Some(1278));
        assert_eq!(msg.get_pressure_amount(), None);
        assert_eq!(msg.get_program_number(), None);
        assert_eq!(
            msg.to_structured(),
            StructuredMidiMessage::PitchBendChange(PitchBendChangeData {
                channel: 1,
                pitch_bend_value: 1278
            })
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
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::TimingClock);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::Start);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
    fn continue_message() {
        // Given
        let msg = RawMidiMessage::continue_message();
        // When
        // Then
        assert_eq!(msg.get_status_byte(), 0xfb);
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::Continue);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::Stop);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::ActiveSensing);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
        assert_eq!(msg.get_data_byte_1(), 0);
        assert_eq!(msg.get_data_byte_2(), 0);
        assert_eq!(msg.get_kind(), MidiMessageKind::SystemReset);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
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
        let msg = StructuredMidiMessage::from_bytes(145, 64, 100).unwrap();
        // When
        // Then
        let expected_msg = StructuredMidiMessage::NoteOn(NoteData {
            channel: 1,
            key_number: 64,
            velocity: 100,
        });
        assert_eq!(msg, expected_msg);
        assert_eq!(msg.get_status_byte(), 145);
        assert_eq!(msg.get_data_byte_1(), 64);
        assert_eq!(msg.get_data_byte_2(), 100);
        assert_eq!(msg.get_kind(), MidiMessageKind::NoteOn);
        assert_eq!(msg.get_super_kind(), MidiMessageSuperKind::Channel);
        assert_eq!(msg.get_main_category(), MidiMessageMainCategory::Channel);
        assert_eq!(msg.get_channel(), Some(1));
        assert_eq!(msg.get_key_number(), Some(64));
        assert_eq!(msg.get_velocity(), Some(100));
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
        let messages: Vec<RawMidiMessage> = MidiMessageKind::iter()
            .flat_map(move |kind| match kind.get_super_kind() {
                MidiMessageSuperKind::Channel => (0..16)
                    .map(|ch| RawMidiMessage::channel_message(kind, ch, 0, 0))
                    .collect(),
                MidiMessageSuperKind::SystemRealTime => {
                    vec![RawMidiMessage::system_real_time_message(kind)]
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
        assert_eq!(first.get_kind(), second.get_kind());
        assert_eq!(first.get_super_kind(), second.get_super_kind());
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
