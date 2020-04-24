use crate::{
    build_status_byte, extract_type_from_status_byte, BlurryMidiMessageSuperType, Channel,
    ControllerNumber, InvalidStatusByteError, KeyNumber, MidiMessage, MidiMessageType,
    MidiTimeCodeQuarterFrame, U14, U7,
};

/// Static methods for creating MIDI messages.
///
/// This trait is supposed to be implemented for structs that represent a single MIDI message *and*
/// also support their creation. Only one method needs to be implemented, the rest is done by
/// default methods.
pub trait MidiMessageFactory: MidiMessage + Sized {
    /// Creates a MIDI message from the given bytes without checking the given status byte.
    ///
    /// # Safety
    ///
    /// Callers must make sure that the given status byte is valid, otherwise an invalid MIDI
    /// message will be created.
    ///
    /// Implementations can therefore assume that the given status byte is valid. This method is
    /// usually called by [`from_bytes`], which checks the necessary preconditions.
    ///
    /// [`from_bytes`]: #method.from_bytes
    unsafe fn from_bytes_unchecked(status_byte: u8, data_byte_1: U7, data_byte_2: U7) -> Self;

    /// Creates a MIDI message from the given bytes.
    ///
    /// # Errors
    ///
    /// If the given status byte is invalid, an error will be returned.
    ///
    /// # Design
    ///
    /// Although one could argue that calling such a function with illegal input values is a
    /// violation of its contract, this function returns a result rather than panicking. It's
    /// because - unlike the functions in [`test_util`] - this function is primarily intended
    /// to be used in real-world situations where the bytes come from somewhere else (e.g. from a
    /// DAW) and therefore acts a bit like a parse function where client code should be able to
    /// recover from wrong input.
    ///
    /// [`test_util`]: test_util/index.html
    fn from_bytes(
        status_byte: u8,
        data_byte_1: U7,
        data_byte_2: U7,
    ) -> Result<Self, InvalidStatusByteError> {
        extract_type_from_status_byte(status_byte)?;
        Ok(unsafe { Self::from_bytes_unchecked(status_byte, data_byte_1, data_byte_2) })
    }

    /// Creates this message from a MIDI message of another type.
    fn from_other(msg: &impl MidiMessage) -> Self {
        msg.to_other()
    }

    /// Creates a Channel message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a channel message type.
    fn channel_message(r#type: MidiMessageType, channel: Channel, data_1: U7, data_2: U7) -> Self {
        assert_eq!(r#type.super_type(), BlurryMidiMessageSuperType::Channel);
        unsafe {
            Self::from_bytes_unchecked(build_status_byte(r#type.into(), channel), data_1, data_2)
        }
    }

    /// Creates a System Common message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a System Common message type.
    fn system_common_message(r#type: MidiMessageType, data_1: U7, data_2: U7) -> Self {
        assert_eq!(
            r#type.super_type(),
            BlurryMidiMessageSuperType::SystemCommon
        );
        unsafe { Self::from_bytes_unchecked(r#type.into(), data_1, data_2) }
    }

    /// Creates a System Real Time message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a System Real Time message type.
    fn system_real_time_message(r#type: MidiMessageType) -> Self {
        assert_eq!(
            r#type.super_type(),
            BlurryMidiMessageSuperType::SystemRealTime
        );
        unsafe { Self::from_bytes_unchecked(r#type.into(), U7::MIN, U7::MIN) }
    }

    /// Creates a Note On message.
    fn note_on(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::NoteOn.into(), channel),
                key_number.into(),
                velocity,
            )
        }
    }

    /// Creates a Note Off message.
    fn note_off(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::NoteOff.into(), channel),
                key_number.into(),
                velocity,
            )
        }
    }

    /// Creates a Control Change message.
    fn control_change(
        channel: Channel,
        controller_number: ControllerNumber,
        control_value: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::ControlChange.into(), channel),
                controller_number.into(),
                control_value,
            )
        }
    }

    /// Creates a Program Change message.
    fn program_change(channel: Channel, program_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::ProgramChange.into(), channel),
                program_number,
                U7::MIN,
            )
        }
    }

    /// Creates a Polyphonic Key Pressure message.
    fn polyphonic_key_pressure(
        channel: Channel,
        key_number: KeyNumber,
        pressure_amount: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::PolyphonicKeyPressure.into(), channel),
                key_number.into(),
                pressure_amount,
            )
        }
    }

    /// Creates a Channel Pressure message.
    fn channel_pressure(channel: Channel, pressure_amount: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::ChannelPressure.into(), channel),
                pressure_amount,
                U7::MIN,
            )
        }
    }

    /// Creates a Pitch Bend Change message.
    fn pitch_bend_change(channel: Channel, pitch_bend_value: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                build_status_byte(MidiMessageType::PitchBendChange.into(), channel),
                U7((pitch_bend_value.get() & 0x7f) as u8),
                U7((pitch_bend_value.get() >> 7) as u8),
            )
        }
    }

    /// Creates the start of a System Exclusive message.
    fn system_exclusive_start() -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageType::SystemExclusiveStart.into(),
                U7::MIN,
                U7::MIN,
            )
        }
    }

    /// Creates a MIDI Time Code Quarter Frame message.
    fn midi_time_code_quarter_frame(frame: MidiTimeCodeQuarterFrame) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageType::MidiTimeCodeQuarterFrame.into(),
                frame.into(),
                U7::MIN,
            )
        }
    }

    /// Creates a Song Position Pointer message.
    fn song_position_pointer(position: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked(
                MidiMessageType::SongPositionPointer.into(),
                U7((position.get() & 0x7f) as u8),
                U7((position.get() >> 7) as u8),
            )
        }
    }

    /// Creates a Song Select message.
    fn song_select(song_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageType::SongSelect.into(), song_number, U7::MIN)
        }
    }

    /// Creates a Tune Request message.
    fn tune_request() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::TuneRequest.into(), U7::MIN, U7::MIN) }
    }

    /// Creates the end of a System Exclusive message.
    fn system_exclusive_end() -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageType::SystemExclusiveEnd.into(), U7::MIN, U7::MIN)
        }
    }

    /// Creates a Timing Clock message.
    fn timing_clock() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::TimingClock.into(), U7::MIN, U7::MIN) }
    }

    /// Creates a Start message.
    fn start() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::Start.into(), U7::MIN, U7::MIN) }
    }

    /// Creates a Continue message.
    fn r#continue() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::Continue.into(), U7::MIN, U7::MIN) }
    }

    /// Creates a Stop message.
    fn stop() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::Stop.into(), U7::MIN, U7::MIN) }
    }

    /// Creates an Active Sensing message.
    fn active_sensing() -> Self {
        unsafe {
            Self::from_bytes_unchecked(MidiMessageType::ActiveSensing.into(), U7::MIN, U7::MIN)
        }
    }

    /// Creates a System Reset message.
    fn system_reset() -> Self {
        unsafe { Self::from_bytes_unchecked(MidiMessageType::SystemReset.into(), U7::MIN, U7::MIN) }
    }
}
