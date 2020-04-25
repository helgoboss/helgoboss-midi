//! Contains convenience functions for creating messages with minimum boilerplate, intended to be
//! used in test or demo code.
//!
//! # Panics
//!
//! Most of the functions contained in this module panic if a passed value doesn't conform to the
//! value range of the corresponding type.
use crate::{
    Channel, ControlChange14BitMessage, ControllerNumber, KeyNumber, ParameterNumberMessage,
    RawShortMessage, ShortMessageFactory, TimeCodeQuarterFrame, U14, U4, U7,
};
use std::convert::TryInto;

type Msg = RawShortMessage;

use channel as ch;
use controller_number as cn;
use key_number as kn;

pub fn u4(value: u8) -> U4 {
    value.try_into().unwrap()
}

pub fn u7(value: u8) -> U7 {
    value.try_into().unwrap()
}

pub fn u14(value: u16) -> U14 {
    value.try_into().unwrap()
}

pub fn channel(value: u8) -> Channel {
    value.try_into().unwrap()
}

pub fn key_number(value: u8) -> KeyNumber {
    value.try_into().unwrap()
}

pub fn controller_number(value: u8) -> ControllerNumber {
    value.try_into().unwrap()
}

pub fn short(status_byte: u8, data_byte_1: u8, data_byte_2: u8) -> Msg {
    Msg::from_bytes((status_byte, u7(data_byte_1), u7(data_byte_2))).unwrap()
}

pub fn note_on(channel: u8, key_number: u8, velocity: u8) -> Msg {
    Msg::note_on(ch(channel), kn(key_number), u7(velocity))
}

pub fn note_off(channel: u8, key_number: u8, velocity: u8) -> Msg {
    Msg::note_off(ch(channel), kn(key_number), u7(velocity))
}

pub fn control_change(channel: u8, controller_number: u8, control_value: u8) -> Msg {
    Msg::control_change(ch(channel), cn(controller_number), u7(control_value))
}

pub fn program_change(channel: u8, program_number: u8) -> Msg {
    Msg::program_change(ch(channel), u7(program_number))
}

pub fn polyphonic_key_pressure(channel: u8, key_number: u8, pressure_amount: u8) -> Msg {
    Msg::polyphonic_key_pressure(ch(channel), kn(key_number), u7(pressure_amount))
}

pub fn channel_pressure(channel: u8, pressure_amount: u8) -> Msg {
    Msg::channel_pressure(ch(channel), u7(pressure_amount))
}
pub fn pitch_bend_change(channel: u8, pitch_bend_value: u16) -> Msg {
    Msg::pitch_bend_change(ch(channel), u14(pitch_bend_value))
}

pub fn system_exclusive_start() -> Msg {
    Msg::system_exclusive_start()
}

pub fn time_code_quarter_frame(frame: TimeCodeQuarterFrame) -> Msg {
    Msg::time_code_quarter_frame(frame)
}

pub fn song_position_pointer(position: u16) -> Msg {
    Msg::song_position_pointer(u14(position))
}

pub fn song_select(song_number: u8) -> Msg {
    Msg::song_select(u7(song_number))
}

pub fn tune_request() -> Msg {
    Msg::tune_request()
}

pub fn system_exclusive_end() -> Msg {
    Msg::system_exclusive_end()
}

pub fn timing_clock() -> Msg {
    Msg::timing_clock()
}

pub fn start() -> Msg {
    Msg::start()
}

pub fn r#continue() -> Msg {
    Msg::r#continue()
}

pub fn stop() -> Msg {
    Msg::stop()
}

pub fn active_sensing() -> Msg {
    Msg::active_sensing()
}

pub fn system_reset() -> Msg {
    Msg::system_reset()
}

pub fn control_change_14_bit(
    channel: u8,
    msb_controller_number: u8,
    value: u16,
) -> ControlChange14BitMessage {
    ControlChange14BitMessage::new(ch(channel), cn(msb_controller_number), u14(value))
}

pub fn nrpn(channel: u8, number: u16, value: u8) -> ParameterNumberMessage {
    ParameterNumberMessage::non_registered_7_bit(ch(channel), u14(number), u7(value))
}

pub fn nrpn_14_bit(channel: u8, number: u16, value: u16) -> ParameterNumberMessage {
    ParameterNumberMessage::non_registered_14_bit(ch(channel), u14(number), u14(value))
}

pub fn rpn(channel: u8, number: u16, value: u8) -> ParameterNumberMessage {
    ParameterNumberMessage::registered_7_bit(ch(channel), u14(number), u7(value))
}

pub fn rpn_14_bit(channel: u8, number: u16, value: u16) -> ParameterNumberMessage {
    ParameterNumberMessage::registered_14_bit(ch(channel), u14(number), u14(value))
}
