//! Convenience methods for creating messages with minimum boilerplate, intended to be used in tests
use crate::{
    channel as ch, controller_number as cn, key_number as kn, program_number as pn, u14, u7,
    MidiControlChange14BitMessage, MidiMessageFactory, MidiParameterNumberMessage, RawMidiMessage,
};

type Msg = RawMidiMessage;

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
    Msg::program_change(ch(channel), pn(program_number))
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

pub fn timing_clock() -> Msg {
    Msg::timing_clock()
}

pub fn start() -> Msg {
    Msg::start()
}

pub fn continue_message() -> Msg {
    Msg::continue_message()
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
) -> MidiControlChange14BitMessage {
    MidiControlChange14BitMessage::new(ch(channel), cn(msb_controller_number), u14(value))
}

pub fn nrpn(channel: u8, number: u16, value: u8) -> MidiParameterNumberMessage {
    MidiParameterNumberMessage::non_registered_7_bit(ch(channel), u14(number), u7(value))
}

pub fn nrpn_14_bit(channel: u8, number: u16, value: u16) -> MidiParameterNumberMessage {
    MidiParameterNumberMessage::non_registered_14_bit(ch(channel), u14(number), u14(value))
}

pub fn rpn(channel: u8, number: u16, value: u8) -> MidiParameterNumberMessage {
    MidiParameterNumberMessage::registered_7_bit(ch(channel), u14(number), u7(value))
}

pub fn rpn_14_bit(channel: u8, number: u16, value: u16) -> MidiParameterNumberMessage {
    MidiParameterNumberMessage::registered_14_bit(ch(channel), u14(number), u14(value))
}
