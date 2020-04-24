#![doc(html_root_url = "https://docs.rs/helgoboss-midi/0.1.0")]
//! Data structures and utilities for dealing with MIDI messages according to the MIDI 1.0
//! specification.
//!
//! The most important type in here is [`MidiMessage`].
//!
//! [`MidiMessage`]: trait.MidiMessage.html

#[macro_use]
mod newtype_macros;
pub use newtype_macros::*;

mod midi_message;
pub use midi_message::*;

mod midi_message_factory;
pub use midi_message_factory::*;

mod structured_midi_message;
pub use structured_midi_message::*;

mod raw_midi_message;
pub use raw_midi_message::*;

mod midi_control_change_14_bit_message;
pub use midi_control_change_14_bit_message::*;

mod midi_control_change_14_bit_message_parser;
pub use midi_control_change_14_bit_message_parser::*;

mod midi_parameter_number_message;
pub use midi_parameter_number_message::*;

mod midi_parameter_number_message_parser;
pub use midi_parameter_number_message_parser::*;

// I added the _mod suffix because of intellij-rust issue 4992
mod channel_mod;
pub use channel_mod::*;

mod key_number_mod;
pub use key_number_mod::*;

mod controller_number_mod;
pub use controller_number_mod::*;

mod u4_mod;
pub use u4_mod::*;

mod u7_mod;
pub use u7_mod::*;

mod u14_mod;
pub use u14_mod::*;

mod bit_util;
pub(crate) use bit_util::*;

pub mod test_util;
