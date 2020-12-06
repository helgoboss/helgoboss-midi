#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![doc(html_root_url = "https://docs.rs/helgoboss-midi/0.1.0")]

//! Interfaces, data structures and utilities for dealing with MIDI messages.
//!
//! # Features
//!
//! - Complete support for the following message types:
//!     - Short messages (3 bytes)
//!     - 14-bit Control Change messages
//!     - (N)RPN messages
//! - Scanners for extracting 14-bit Control Change and (N)RPN messages from a stream of short
//!   messages
//! - Suitable for real-time usage (no heap allocation, no dynamic dispatch, no locking)
//! - Unified API to work with different short message data structures (see
//!   [`ShortMessage`](trait.ShortMessage.html) trait)
//! - Uses wording which is as close as possible to the [MIDI 1.0 specification](https://www.midi.org/specifications-old/category/midi-1-0-detailed-specifications)
//!
//! # Not yet implemented
//!
//! Data structures and utilities for dealing with System Exclusive messages are not yet
//! implemented. They will be added eventually as separate structures on top of the
//! existing ones (similar to (N)RPN and 14-bit Control Change).
//!
//! # Examples
//!
//! See how to ...
//!
//! - [Create and inspect short messages](struct.RawShortMessage.html#example)
//! - [Easily match short messages](enum.StructuredShortMessage.html#example)
//! - [Create and inspect 14-bit Control Change
//!   messages](struct.ControlChange14BitMessage.html#example)
//! - [Create and inspect (N)RPN messages](struct.ParameterNumberMessage.html#example)
//! - [Create MIDI messages with minimum boilerplate](test_util/index.html#example)
//! - [Scan stream for 14-bit Control Change
//!   messages](struct.ControlChange14BitMessageScanner.html#example)
//! - [Scan stream for (N)RPN messages](struct.ParameterNumberMessageScanner.html#example)
#[macro_use]
mod newtype_macros;
pub use newtype_macros::*;

mod short_message;
pub use short_message::*;

mod short_message_factory;
pub use short_message_factory::*;

mod structured_short_message;
pub use structured_short_message::*;

mod raw_short_message;
pub use raw_short_message::*;

mod control_change_14_bit_message;
pub use control_change_14_bit_message::*;

mod control_change_14_bit_message_scanner;
pub use control_change_14_bit_message_scanner::*;

mod parameter_number_message;
pub use parameter_number_message::*;

mod parameter_number_message_scanner;
pub use parameter_number_message_scanner::*;

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
