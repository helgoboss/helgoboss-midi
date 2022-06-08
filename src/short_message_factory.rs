use crate::{
    build_status_byte, extract_type_from_status_byte, Channel, ControllerNumber,
    FuzzyMessageSuperType, KeyNumber, ShortMessage, ShortMessageType, TimeCodeQuarterFrame, U14,
    U7,
};

/// An error which can occur when trying to create a [`ShortMessage`] from raw bytes.
///
/// [`ShortMessage`]: trait.ShortMessage.html
#[derive(Clone, Eq, PartialEq, Debug, derive_more::Display)]
#[display(fmt = "invalid MIDI message bytes")]
pub struct FromBytesError(pub(crate) ());

#[cfg(feature = "std")]
impl std::error::Error for FromBytesError {}

/// Static methods for creating short MIDI messages.
///
/// This trait is supposed to be implemented for structs that represent a short MIDI message *and*
/// also support their creation. Only one method needs to be implemented, the rest is done by
/// default methods.
pub trait ShortMessageFactory: ShortMessage + Sized {
    /// Creates a MIDI message from the given bytes without checking the status byte. The tuple
    /// consists of the status byte, data byte 1 and data byte 2 in exactly this order.
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
    unsafe fn from_bytes_unchecked(bytes: (u8, U7, U7)) -> Self;

    /// Creates a MIDI message from the given bytes. The tuple consists of the status byte, data
    /// byte 1 and data byte 2 in exactly this order.
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
    fn from_bytes(bytes: (u8, U7, U7)) -> Result<Self, FromBytesError> {
        extract_type_from_status_byte(bytes.0).map_err(|_| FromBytesError(()))?;
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates this message from a MIDI message of another type.
    fn from_other(msg: &impl ShortMessage) -> Self {
        msg.to_other()
    }

    /// Creates a Channel message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a channel message type.
    fn channel_message(r#type: ShortMessageType, channel: Channel, data_1: U7, data_2: U7) -> Self {
        assert_eq!(r#type.super_type(), FuzzyMessageSuperType::Channel);
        unsafe {
            Self::from_bytes_unchecked((build_status_byte(r#type.into(), channel), data_1, data_2))
        }
    }

    /// Creates a System Common message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a System Common message type.
    fn system_common_message(r#type: ShortMessageType, data_1: U7, data_2: U7) -> Self {
        assert_eq!(r#type.super_type(), FuzzyMessageSuperType::SystemCommon);
        unsafe { Self::from_bytes_unchecked((r#type.into(), data_1, data_2)) }
    }

    /// Creates a System Real Time message.
    ///
    /// # Panics
    ///
    /// This function panics if the given type is not a System Real Time message type.
    fn system_real_time_message(r#type: ShortMessageType) -> Self {
        assert_eq!(r#type.super_type(), FuzzyMessageSuperType::SystemRealTime);
        unsafe { Self::from_bytes_unchecked((r#type.into(), U7::MIN, U7::MIN)) }
    }

    /// Creates a Note On message.
    fn note_on(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::NoteOn.into(), channel),
                key_number.into(),
                velocity,
            ))
        }
    }

    /// Creates a Note Off message.
    fn note_off(channel: Channel, key_number: KeyNumber, velocity: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::NoteOff.into(), channel),
                key_number.into(),
                velocity,
            ))
        }
    }

    /// Creates a Control Change message.
    fn control_change(
        channel: Channel,
        controller_number: ControllerNumber,
        control_value: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::ControlChange.into(), channel),
                controller_number.into(),
                control_value,
            ))
        }
    }

    /// Creates a Program Change message.
    fn program_change(channel: Channel, program_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::ProgramChange.into(), channel),
                program_number,
                U7::MIN,
            ))
        }
    }

    /// Creates a Polyphonic Key Pressure message.
    fn polyphonic_key_pressure(
        channel: Channel,
        key_number: KeyNumber,
        pressure_amount: U7,
    ) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::PolyphonicKeyPressure.into(), channel),
                key_number.into(),
                pressure_amount,
            ))
        }
    }

    /// Creates a Channel Pressure message.
    fn channel_pressure(channel: Channel, pressure_amount: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::ChannelPressure.into(), channel),
                pressure_amount,
                U7::MIN,
            ))
        }
    }

    /// Creates a Pitch Bend Change message.
    fn pitch_bend_change(channel: Channel, pitch_bend_value: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                build_status_byte(ShortMessageType::PitchBendChange.into(), channel),
                U7((pitch_bend_value.get() & 0x7f) as u8),
                U7((pitch_bend_value.get() >> 7) as u8),
            ))
        }
    }

    /// Creates the start of a System Exclusive message.
    fn system_exclusive_start() -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                ShortMessageType::SystemExclusiveStart.into(),
                U7::MIN,
                U7::MIN,
            ))
        }
    }

    /// Creates a MIDI Time Code Quarter Frame message.
    fn time_code_quarter_frame(frame: TimeCodeQuarterFrame) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                ShortMessageType::TimeCodeQuarterFrame.into(),
                frame.into(),
                U7::MIN,
            ))
        }
    }

    /// Creates a Song Position Pointer message.
    fn song_position_pointer(position: U14) -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                ShortMessageType::SongPositionPointer.into(),
                U7((position.get() & 0x7f) as u8),
                U7((position.get() >> 7) as u8),
            ))
        }
    }

    /// Creates a Song Select message.
    fn song_select(song_number: U7) -> Self {
        unsafe {
            Self::from_bytes_unchecked((ShortMessageType::SongSelect.into(), song_number, U7::MIN))
        }
    }

    /// Creates a Tune Request message.
    fn tune_request() -> Self {
        unsafe {
            Self::from_bytes_unchecked((ShortMessageType::TuneRequest.into(), U7::MIN, U7::MIN))
        }
    }

    /// Creates the end of a System Exclusive message.
    fn system_exclusive_end() -> Self {
        unsafe {
            Self::from_bytes_unchecked((
                ShortMessageType::SystemExclusiveEnd.into(),
                U7::MIN,
                U7::MIN,
            ))
        }
    }

    /// Creates a Timing Clock message.
    fn timing_clock() -> Self {
        unsafe {
            Self::from_bytes_unchecked((ShortMessageType::TimingClock.into(), U7::MIN, U7::MIN))
        }
    }

    /// Creates a Start message.
    fn start() -> Self {
        unsafe { Self::from_bytes_unchecked((ShortMessageType::Start.into(), U7::MIN, U7::MIN)) }
    }

    /// Creates a Continue message.
    fn r#continue() -> Self {
        unsafe { Self::from_bytes_unchecked((ShortMessageType::Continue.into(), U7::MIN, U7::MIN)) }
    }

    /// Creates a Stop message.
    fn stop() -> Self {
        unsafe { Self::from_bytes_unchecked((ShortMessageType::Stop.into(), U7::MIN, U7::MIN)) }
    }

    /// Creates an Active Sensing message.
    fn active_sensing() -> Self {
        unsafe {
            Self::from_bytes_unchecked((ShortMessageType::ActiveSensing.into(), U7::MIN, U7::MIN))
        }
    }

    /// Creates a System Reset message.
    fn system_reset() -> Self {
        unsafe {
            Self::from_bytes_unchecked((ShortMessageType::SystemReset.into(), U7::MIN, U7::MIN))
        }
    }
}
