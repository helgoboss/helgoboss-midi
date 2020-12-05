//! Contains convenience functions for creating messages with minimum boilerplate
//!
//! Intended to be used primarily in test or demo code.
//!
//! # Example
//!
//! ```
//! use helgoboss_midi::test_util::*;
//!
//! let msg_1 = short(0x90, 4, 5);
//! let msg_2 = note_on(3, 40, 100);
//! let msg_3 = pitch_bend_change(4, 2478);
//! let msg_4 = nrpn_14_bit(4, 380, 12000);
//! let msg_5 = control_change_14_bit(2, 2, 2056);
//! let ch = channel(3);
//! let kn = key_number(64);
//! ```
use crate::{
    Channel, ControlChange14BitMessage, ControllerNumber, KeyNumber, ParameterNumberMessage,
    RawShortMessage, ShortMessageFactory, TimeCodeQuarterFrame, U14, U4, U7,
};
use core::convert::TryInto;

type Msg = RawShortMessage;

use channel as ch;
use controller_number as cn;
use key_number as kn;

/// Creates a 4-bit integer value.
///
/// # Panics
///
/// Panics if the given value is greater than 15.
pub fn u4(value: u8) -> U4 {
    value.try_into().expect("not a valid 4-bit integer")
}

/// Creates a 7-bit integer value.
///
/// # Panics
///
/// Panics if the given value is greater than 127.
pub fn u7(value: u8) -> U7 {
    value.try_into().expect("not a valid 7-bit integer")
}

/// Creates a 14-bit integer value.
///
/// # Panics
///
/// Panics if the given value is higher than 16383.
pub fn u14(value: u16) -> U14 {
    value.try_into().expect("not a valid 14-bit integer")
}

/// Creates a channel.
///
/// # Panics
///
/// Panics if the given value is higher than 15.
pub fn channel(value: u8) -> Channel {
    value.try_into().expect("not a valid channel")
}

/// Creates a key number.
///
/// # Panics
///
/// Panics if the given value is higher than 127.
pub fn key_number(value: u8) -> KeyNumber {
    value.try_into().expect("not a valid key number")
}

/// Creates a controller number.
///
/// # Panics
///
/// Panics if the given value is higher than 127.
pub fn controller_number(value: u8) -> ControllerNumber {
    value.try_into().expect("not a valid controller number")
}

/// Creates a short message from raw bytes.
///
/// # Panics
///
/// Panics if one of the given values is out of range or if the status byte is invalid.
pub fn short(status_byte: u8, data_byte_1: u8, data_byte_2: u8) -> Msg {
    Msg::from_bytes((status_byte, u7(data_byte_1), u7(data_byte_2))).expect("invalid status byte")
}

/// Creates a Note On message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn note_on(channel: u8, key_number: u8, velocity: u8) -> Msg {
    Msg::note_on(ch(channel), kn(key_number), u7(velocity))
}

/// Creates a Note Off message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn note_off(channel: u8, key_number: u8, velocity: u8) -> Msg {
    Msg::note_off(ch(channel), kn(key_number), u7(velocity))
}

/// Creates a Control Change message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn control_change(channel: u8, controller_number: u8, control_value: u8) -> Msg {
    Msg::control_change(ch(channel), cn(controller_number), u7(control_value))
}

/// Creates a Program Change message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn program_change(channel: u8, program_number: u8) -> Msg {
    Msg::program_change(ch(channel), u7(program_number))
}

/// Creates a Polyphonic Key Pressure message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn polyphonic_key_pressure(channel: u8, key_number: u8, pressure_amount: u8) -> Msg {
    Msg::polyphonic_key_pressure(ch(channel), kn(key_number), u7(pressure_amount))
}

/// Creates a Channel Pressure message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn channel_pressure(channel: u8, pressure_amount: u8) -> Msg {
    Msg::channel_pressure(ch(channel), u7(pressure_amount))
}

/// Creates a Pitch Bend Change message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn pitch_bend_change(channel: u8, pitch_bend_value: u16) -> Msg {
    Msg::pitch_bend_change(ch(channel), u14(pitch_bend_value))
}

/// Creates a System Exclusive Start message.
pub fn system_exclusive_start() -> Msg {
    Msg::system_exclusive_start()
}

/// Creates a Time Code Quarter Frame message.
pub fn time_code_quarter_frame(frame: TimeCodeQuarterFrame) -> Msg {
    Msg::time_code_quarter_frame(frame)
}

/// Creates a Song Position Pointer message.
///
/// # Panics
///
/// Panics if the given value is out of range.
pub fn song_position_pointer(position: u16) -> Msg {
    Msg::song_position_pointer(u14(position))
}

/// Creates a Song Select message.
///
/// # Panics
///
/// Panics if the given value is out of range.
pub fn song_select(song_number: u8) -> Msg {
    Msg::song_select(u7(song_number))
}

/// Creates a Tune Request message.
pub fn tune_request() -> Msg {
    Msg::tune_request()
}

/// Creates a System Exclusive End message.
pub fn system_exclusive_end() -> Msg {
    Msg::system_exclusive_end()
}

/// Creates a Timing Clock message.
pub fn timing_clock() -> Msg {
    Msg::timing_clock()
}

/// Creates a start message.
pub fn start() -> Msg {
    Msg::start()
}

/// Creates a continue message.
pub fn r#continue() -> Msg {
    Msg::r#continue()
}

/// Creates a stop message.
pub fn stop() -> Msg {
    Msg::stop()
}

/// Creates an Active Sensing message.
pub fn active_sensing() -> Msg {
    Msg::active_sensing()
}

/// Creates a System Reset message.
pub fn system_reset() -> Msg {
    Msg::system_reset()
}

/// Creates a 14-bit Control Change message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn control_change_14_bit(
    channel: u8,
    msb_controller_number: u8,
    value: u16,
) -> ControlChange14BitMessage {
    ControlChange14BitMessage::new(ch(channel), cn(msb_controller_number), u14(value))
}

/// Creates a non-registered 7-bit Parameter Number message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn nrpn(channel: u8, number: u16, value: u8) -> ParameterNumberMessage {
    ParameterNumberMessage::non_registered_7_bit(ch(channel), u14(number), u7(value))
}

/// Creates an non-registered 14-bit Parameter Number message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn nrpn_14_bit(channel: u8, number: u16, value: u16) -> ParameterNumberMessage {
    ParameterNumberMessage::non_registered_14_bit(ch(channel), u14(number), u14(value))
}

/// Creates an registered 7-bit Parameter Number message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn rpn(channel: u8, number: u16, value: u8) -> ParameterNumberMessage {
    ParameterNumberMessage::registered_7_bit(ch(channel), u14(number), u7(value))
}

/// Creates an registered 14-bit Parameter Number message.
///
/// # Panics
///
/// Panics if one of the given values is out of range.
pub fn rpn_14_bit(channel: u8, number: u16, value: u16) -> ParameterNumberMessage {
    ParameterNumberMessage::registered_14_bit(ch(channel), u14(number), u14(value))
}
