use crate::{
    build_status_byte, get_midi_message_kind_from_status_byte, Channel, ControllerNumber,
    KeyNumber, MidiMessage, MidiMessageKind, MidiMessageMainCategory, MidiMessageSuperKind,
    MidiTimeCodeQuarterFrame, StructuredMidiMessage, U14, U4, U7,
};

/// Trait to be implemented by struct representing a MIDI message if it supports creation of various
/// kinds of MIDI messages. Only one method needs to be implemented, the rest is done by default
/// methods. The advantage of this architecture is that we can have a unified factory API, no matter
/// which underlying data structure is used.
pub trait MidiMessageFactory: Sized {
    unsafe fn from_bytes_unchecked(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Self;

    // Although we could argue that calling this function with illegal input values is a violation
    // of its contract, this function returns a result rather than panicking. It's because - unlike
    // the convenience factory functions - this function is primarily intended to be used in
    // situations where the bytes come from somewhere else (e.g. are user-generated) and therefore
    // acts a bit like a parse function where client code should be able to recover from wrong
    // input.
    fn from_bytes(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Result<Self, &'static str> {
        get_midi_message_kind_from_status_byte(status_byte).map_err(|_| "Invalid status byte")?;
        Ok(unsafe { Self::from_bytes_unchecked(status_byte, data_byte_1, data_byte_2) })
    }

    fn from_structured(msg: &StructuredMidiMessage) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                msg.get_status_byte(),
                msg.get_data_byte_1(),
                msg.get_data_byte_2(),
            )
        }
    }

    fn channel_message(kind: MidiMessageKind, channel: Channel, data_1: U7, data_2: U7) -> Self {
        assert_eq!(
            kind.get_super_kind().get_main_category(),
            MidiMessageMainCategory::Channel
        );
        unsafe {
            Self::from_bytes_unchecked(build_status_byte(kind.into(), channel), data_1, data_2)
        }
    }

    fn system_common_message(kind: MidiMessageKind, data_1: U7, data_2: U7) -> Self {
        assert_eq!(kind.get_super_kind(), MidiMessageSuperKind::SystemCommon);
        unsafe { Self::from_bytes_unchecked(kind.into(), data_1, data_2) }
    }

    fn system_real_time_message(kind: MidiMessageKind) -> Self {
        assert_eq!(kind.get_super_kind(), MidiMessageSuperKind::SystemRealTime);
        unsafe { Self::from_bytes_unchecked(kind.into(), U7::MIN, U7::MIN) }
    }

    fn note_on(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::NoteOn.into(), channel),
                key_number.into(),
                velocity,
            )
        }
    }

    fn note_off(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::NoteOff.into(), channel),
                key_number.into(),
                velocity,
            )
        }
    }

    fn control_change(
        channel: Channel,
        controller_number: ControllerNumber,
        control_value: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::ControlChange.into(), channel),
                controller_number.into(),
                control_value,
            )
        }
    }

    fn program_change(channel: Channel, program_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::ProgramChange.into(), channel),
                program_number,
                U7::MIN,
            )
        }
    }

    fn polyphonic_key_pressure(
        channel: Channel,
        key_number: KeyNumber,
        pressure_amount: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::PolyphonicKeyPressure.into(), channel),
                key_number.into(),
                pressure_amount,
            )
        }
    }

    fn channel_pressure(channel: Channel, pressure_amount: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::ChannelPressure.into(), channel),
                pressure_amount,
                U7::MIN,
            )
        }
    }
    fn pitch_bend_change(channel: Channel, pitch_bend_value: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageKind::PitchBendChange.into(), channel),
                U7((u16::from(pitch_bend_value) & 0x7f) as u8),
                U7((u16::from(pitch_bend_value) >> 7) as u8),
            )
        }
    }

    fn system_exclusive_start() -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageKind::SystemExclusiveStart.into(),
                U7::MIN,
                U7::MIN,
            )
        }
    }

    fn midi_time_code_quarter_frame(frame: MidiTimeCodeQuarterFrame) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageKind::MidiTimeCodeQuarterFrame.into(),
                frame.into(),
                U7::MIN,
            )
        }
    }

    fn song_position_pointer(position: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageKind::SongPositionPointer.into(),
                U7((u16::from(position) & 0x7f) as u8),
                U7((u16::from(position) >> 7) as u8),
            )
        }
    }

    fn song_select(song_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageKind::SongSelect.into(), song_number, U7::MIN)
        }
    }

    fn tune_request() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::TuneRequest.into(), U7::MIN, U7::MIN) }
    }

    fn system_exclusive_end() -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageKind::SystemExclusiveEnd.into(), U7::MIN, U7::MIN)
        }
    }

    fn timing_clock() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::TimingClock.into(), U7::MIN, U7::MIN) }
    }

    fn start() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::Start.into(), U7::MIN, U7::MIN) }
    }

    fn r#continue() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::Continue.into(), U7::MIN, U7::MIN) }
    }

    fn stop() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::Stop.into(), U7::MIN, U7::MIN) }
    }

    fn active_sensing() -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageKind::ActiveSensing.into(), U7::MIN, U7::MIN)
        }
    }

    fn system_reset() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageKind::SystemReset.into(), U7::MIN, U7::MIN) }
    }
}
