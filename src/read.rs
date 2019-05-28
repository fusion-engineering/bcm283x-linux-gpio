use crate::{PinMode, Reg};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinInfo {
	pub mode: PinMode,
	pub level: bool,
	pub event: bool,
	pub detect_rise: bool,
	pub detect_fall: bool,
	pub detect_high: bool,
	pub detect_low: bool,
	pub detect_async_rise: bool,
	pub detect_async_fall: bool,
}

#[derive(Clone)]
pub struct RpioState {
	data: [u32; 0x100],
}

impl RpioState {
	pub fn from_data(data: [u32; 0x100]) -> Self {
		Self { data }
	}

	pub fn data(&self) -> &[u32; 0x100] {
		&self.data
	}

	pub fn into_data(self) -> [u32; 0x100] {
		self.data
	}

	pub fn pin_mode(&self, index: u32) -> PinMode {
		PinMode::try_from_bits(self.read_pin_bits(index, Reg::GPFSEL0, 10, 3) as u8).unwrap()
	}

	pub fn pin_level(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPLEV0, 32, 1) != 0
	}

	pub fn pin_event(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPPEDS0, 32, 1) != 0
	}

	pub fn pin_detect_rise(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPREN0, 32, 1) != 0
	}

	pub fn pin_detect_fall(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPFEN0, 32, 1) != 0
	}

	pub fn pin_detect_high(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPHEN0, 32, 1) != 0
	}

	pub fn pin_detect_low(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPLEN0, 32, 1) != 0
	}

	pub fn pin_detect_async_rise(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPAREN0, 32, 1) != 0
	}

	pub fn pin_detect_async_fall(&self, index: u32) -> bool {
		self.read_pin_bits(index, Reg::GPAFEN0, 32, 1) != 0
	}

	pub fn pin(&self, index: u32) -> PinInfo {
		PinInfo {
			mode:              self.pin_mode(index),
			level:             self.pin_level(index),
			event:             self.pin_event(index),
			detect_rise:       self.pin_detect_rise(index),
			detect_fall:       self.pin_detect_fall(index),
			detect_high:       self.pin_detect_high(index),
			detect_low:        self.pin_detect_low(index),
			detect_async_rise: self.pin_detect_async_rise(index),
			detect_async_fall: self.pin_detect_async_fall(index),
		}
	}

	pub fn pins(&self) -> Vec<PinInfo> {
		(0..53).map(|i| self.pin(i)).collect()
	}

	fn read_pin_bits(&self, index: u32, base: Reg, pins_per_register: u8, bits_per_pin: u8) -> u32 {
		assert!(index <= 53, "gpio pin index out of range, expected a value in the range [0-53], got {}", index);

		let pins_per_register = pins_per_register as u32;
		let bits_per_pin      = bits_per_pin      as u32;

		// Register has a relative byte address,
		// but registers are 32 bit.
		let base           = base as u32 / 4;
		let register_index = base + index / pins_per_register;
		let index          = index % pins_per_register;

		let value = self.data[register_index as usize] >> (bits_per_pin * index);
		let mask  = !(std::u32::MAX << bits_per_pin);
		value & mask
	}
}
