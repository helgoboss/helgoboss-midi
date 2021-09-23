// Basic newtype definition
newtype! {
    #[doc = r"A controller number (0 - 127) of a MIDI Control Change message."]
    name = ControllerNumber, repr = u8, max = 127
}

// From related newtype to this newtype and back
impl_from_newtype_to_newtype!(ControllerNumber, crate::U7);
impl_from_newtype_to_newtype!(crate::U7, ControllerNumber);

// From lower primitives to this newtype
// -

// From this newtype to higher primitives
impl_from_newtype_to_primitive!(ControllerNumber, u8);
impl_from_newtype_to_primitive!(ControllerNumber, i8);
impl_from_newtype_to_primitive!(ControllerNumber, u16);
impl_from_newtype_to_primitive!(ControllerNumber, i16);
impl_from_newtype_to_primitive!(ControllerNumber, u32);
impl_from_newtype_to_primitive!(ControllerNumber, i32);
impl_from_newtype_to_primitive!(ControllerNumber, u64);
impl_from_newtype_to_primitive!(ControllerNumber, i64);
impl_from_newtype_to_primitive!(ControllerNumber, u128);
impl_from_newtype_to_primitive!(ControllerNumber, i128);
impl_from_newtype_to_primitive!(ControllerNumber, usize);
impl_from_newtype_to_primitive!(ControllerNumber, isize);

// TryFrom higher primitives to this newtype
impl_try_from_primitive_to_newtype!(u8, ControllerNumber);
impl_try_from_primitive_to_newtype!(u16, ControllerNumber);
impl_try_from_primitive_to_newtype!(i16, ControllerNumber);
impl_try_from_primitive_to_newtype!(u32, ControllerNumber);
impl_try_from_primitive_to_newtype!(i32, ControllerNumber);
impl_try_from_primitive_to_newtype!(u64, ControllerNumber);
impl_try_from_primitive_to_newtype!(i64, ControllerNumber);
impl_try_from_primitive_to_newtype!(u128, ControllerNumber);
impl_try_from_primitive_to_newtype!(i128, ControllerNumber);
impl_try_from_primitive_to_newtype!(usize, ControllerNumber);
impl_try_from_primitive_to_newtype!(isize, ControllerNumber);

impl ControllerNumber {
    /// Returns whether this controller number can be used to make up a 14-bit Control Change
    /// message.
    pub fn can_be_part_of_14_bit_control_change_message(&self) -> bool {
        self.0 < 64
    }

    /// If this controller number can be used to send the most significant byte of a 14-bit
    /// Control Change message, this function returns the corresponding controller number that would
    /// be used to send the least significant byte of it.
    pub fn corresponding_14_bit_lsb_controller_number(&self) -> Option<ControllerNumber> {
        if self.0 >= 32 {
            return None;
        }
        Some(ControllerNumber(self.0 + 32))
    }

    /// Returns whether this controller number is intended to be used to send part of a (N)RPN
    /// message.
    pub fn is_parameter_number_message_controller_number(&self) -> bool {
        matches!(self.0, 98 | 99 | 100 | 101 | 38 | 6 | 96 | 97)
    }

    /// Returns whether this controller number is intended to be used to send Channel Mode
    /// messages.
    pub fn is_channel_mode_message_controller_number(&self) -> bool {
        *self >= controller_numbers::RESET_ALL_CONTROLLERS
    }
}

/// Contains predefined controller numbers.
///
/// # Design
///
/// These are not associated constants of [`ControllerNumber`] because then we could only access
/// them prefixed with `ControllerNumber::`. Making [`ControllerNumber`] an enum would have been the
/// alternative, but this has other downsides such as having to introduce a special variant for
/// undefined controllers and unnecessary conversion from and to integers. From the MIDI spec
/// perspective, a controller number seems closer to a plain 7-bit integer than to an enum with
/// well-defined values. Not all of the controller numbers have a special meaning, and if they do,
/// this meaning is not necessarily important. In practice, controller numbers are often used for
/// other things than they were intended for, especially the exotic ones.
///
/// [`ControllerNumber`]: struct.ControllerNumber.html
pub mod controller_numbers {
    use crate::ControllerNumber;

    pub const BANK_SELECT: ControllerNumber = ControllerNumber(0x00);
    pub const MODULATION_WHEEL: ControllerNumber = ControllerNumber(0x01);
    pub const BREATH_CONTROLLER: ControllerNumber = ControllerNumber(0x02);
    pub const FOOT_CONTROLLER: ControllerNumber = ControllerNumber(0x04);
    pub const PORTAMENTO_TIME: ControllerNumber = ControllerNumber(0x05);
    pub const DATA_ENTRY_MSB: ControllerNumber = ControllerNumber(0x06);
    pub const CHANNEL_VOLUME: ControllerNumber = ControllerNumber(0x07);
    pub const BALANCE: ControllerNumber = ControllerNumber(0x08);
    pub const PAN: ControllerNumber = ControllerNumber(0x0A);
    pub const EXPRESSION_CONTROLLER: ControllerNumber = ControllerNumber(0x0B);
    pub const EFFECT_CONTROL_1: ControllerNumber = ControllerNumber(0x0C);
    pub const EFFECT_CONTROL_2: ControllerNumber = ControllerNumber(0x0D);
    pub const GENERAL_PURPOSE_CONTROLLER_1: ControllerNumber = ControllerNumber(0x10);
    pub const GENERAL_PURPOSE_CONTROLLER_2: ControllerNumber = ControllerNumber(0x11);
    pub const GENERAL_PURPOSE_CONTROLLER_3: ControllerNumber = ControllerNumber(0x12);
    pub const GENERAL_PURPOSE_CONTROLLER_4: ControllerNumber = ControllerNumber(0x13);
    pub const BANK_SELECT_LSB: ControllerNumber = ControllerNumber(0x20);
    pub const MODULATION_WHEEL_LSB: ControllerNumber = ControllerNumber(0x21);
    pub const BREATH_CONTROLLER_LSB: ControllerNumber = ControllerNumber(0x22);
    pub const FOOT_CONTROLLER_LSB: ControllerNumber = ControllerNumber(0x24);
    pub const PORTAMENTO_TIME_LSB: ControllerNumber = ControllerNumber(0x25);
    pub const DATA_ENTRY_MSB_LSB: ControllerNumber = ControllerNumber(0x26);
    pub const CHANNEL_VOLUME_LSB: ControllerNumber = ControllerNumber(0x27);
    pub const BALANCE_LSB: ControllerNumber = ControllerNumber(0x28);
    pub const PAN_LSB: ControllerNumber = ControllerNumber(0x2A);
    pub const EXPRESSION_CONTROLLER_LSB: ControllerNumber = ControllerNumber(0x2B);
    pub const EFFECT_CONTROL_1_LSB: ControllerNumber = ControllerNumber(0x2C);
    pub const EFFECT_CONTROL_2_LSB: ControllerNumber = ControllerNumber(0x2D);
    pub const GENERAL_PURPOSE_CONTROLLER_1_LSB: ControllerNumber = ControllerNumber(0x30);
    pub const GENERAL_PURPOSE_CONTROLLER_2_LSB: ControllerNumber = ControllerNumber(0x31);
    pub const GENERAL_PURPOSE_CONTROLLER_3_LSB: ControllerNumber = ControllerNumber(0x32);
    pub const GENERAL_PURPOSE_CONTROLLER_4_LSB: ControllerNumber = ControllerNumber(0x33);
    pub const DAMPER_PEDAL_ON_OFF: ControllerNumber = ControllerNumber(0x40);
    pub const PORTAMENTO_ON_OFF: ControllerNumber = ControllerNumber(0x41);
    pub const SOSTENUTO_ON_OFF: ControllerNumber = ControllerNumber(0x42);
    pub const SOFT_PEDAL_ON_OFF: ControllerNumber = ControllerNumber(0x43);
    pub const LEGATO_FOOTSWITCH: ControllerNumber = ControllerNumber(0x44);
    pub const HOLD_2: ControllerNumber = ControllerNumber(0x45);
    pub const SOUND_CONTROLLER_1: ControllerNumber = ControllerNumber(0x46);
    pub const SOUND_CONTROLLER_2: ControllerNumber = ControllerNumber(0x47);
    pub const SOUND_CONTROLLER_3: ControllerNumber = ControllerNumber(0x48);
    pub const SOUND_CONTROLLER_4: ControllerNumber = ControllerNumber(0x49);
    pub const SOUND_CONTROLLER_5: ControllerNumber = ControllerNumber(0x4A);
    pub const SOUND_CONTROLLER_6: ControllerNumber = ControllerNumber(0x4B);
    pub const SOUND_CONTROLLER_7: ControllerNumber = ControllerNumber(0x4C);
    pub const SOUND_CONTROLLER_8: ControllerNumber = ControllerNumber(0x4D);
    pub const SOUND_CONTROLLER_9: ControllerNumber = ControllerNumber(0x4E);
    pub const SOUND_CONTROLLER_10: ControllerNumber = ControllerNumber(0x4F);
    pub const GENERAL_PURPOSE_CONTROLLER_5: ControllerNumber = ControllerNumber(0x50);
    pub const GENERAL_PURPOSE_CONTROLLER_6: ControllerNumber = ControllerNumber(0x51);
    pub const GENERAL_PURPOSE_CONTROLLER_7: ControllerNumber = ControllerNumber(0x52);
    pub const GENERAL_PURPOSE_CONTROLLER_8: ControllerNumber = ControllerNumber(0x53);
    pub const PORTAMENTO_CONTROL: ControllerNumber = ControllerNumber(0x54);
    pub const HIGH_RESOLUTION_VELOCITY_PREFIX: ControllerNumber = ControllerNumber(0x58);
    pub const EFFECTS_1_DEPTH: ControllerNumber = ControllerNumber(0x5B);
    pub const EFFECTS_2_DEPTH: ControllerNumber = ControllerNumber(0x5C);
    pub const EFFECTS_3_DEPTH: ControllerNumber = ControllerNumber(0x5D);
    pub const EFFECTS_4_DEPTH: ControllerNumber = ControllerNumber(0x5E);
    pub const EFFECTS_5_DEPTH: ControllerNumber = ControllerNumber(0x5F);
    pub const DATA_INCREMENT: ControllerNumber = ControllerNumber(0x60);
    pub const DATA_DECREMENT: ControllerNumber = ControllerNumber(0x61);
    pub const NON_REGISTERED_PARAMETER_NUMBER_LSB: ControllerNumber = ControllerNumber(0x62);
    pub const NON_REGISTERED_PARAMETER_NUMBER_MSB: ControllerNumber = ControllerNumber(0x63);
    pub const REGISTERED_PARAMETER_NUMBER_LSB: ControllerNumber = ControllerNumber(0x64);
    pub const REGISTERED_PARAMETER_NUMBER_MSB: ControllerNumber = ControllerNumber(0x65);
    pub const ALL_SOUND_OFF: ControllerNumber = ControllerNumber(0x78);
    pub const RESET_ALL_CONTROLLERS: ControllerNumber = ControllerNumber(0x79);
    pub const LOCAL_CONTROL_ON_OFF: ControllerNumber = ControllerNumber(0x7A);
    pub const ALL_NOTES_OFF: ControllerNumber = ControllerNumber(0x7B);
    pub const OMNI_MODE_OFF: ControllerNumber = ControllerNumber(0x7C);
    pub const OMNI_MODE_ON: ControllerNumber = ControllerNumber(0x7D);
    pub const MONO_MODE_ON: ControllerNumber = ControllerNumber(0x7E);
    pub const POLY_MODE_ON: ControllerNumber = ControllerNumber(0x7F);
}
