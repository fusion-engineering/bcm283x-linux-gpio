use crate::{PinFunction, Register};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinInfo {
	pub function: PinFunction,
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
pub struct GpioState {
	data: [u32; 0x100],
}

impl GpioState {
	pub fn from_data(data: [u32; 0x100]) -> Self {
		Self { data }
	}

	pub fn data(&self) -> &[u32; 0x100] {
		&self.data
	}

	pub fn into_data(self) -> [u32; 0x100] {
		self.data
	}

	pub fn pin_function(&self, index: usize) -> PinFunction {
		PinFunction::try_from_bits(self.read_pin_bits(index, Register::GPFSEL0, 10, 3) as u8).unwrap()
	}

	pub fn pin_level(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPLEV0, 32, 1) != 0
	}

	pub fn pin_event(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPEDS0, 32, 1) != 0
	}

	pub fn pin_detect_rise(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPREN0, 32, 1) != 0
	}

	pub fn pin_detect_fall(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPFEN0, 32, 1) != 0
	}

	pub fn pin_detect_high(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPHEN0, 32, 1) != 0
	}

	pub fn pin_detect_low(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPLEN0, 32, 1) != 0
	}

	pub fn pin_detect_async_rise(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPAREN0, 32, 1) != 0
	}

	pub fn pin_detect_async_fall(&self, index: usize) -> bool {
		self.read_pin_bits(index, Register::GPAFEN0, 32, 1) != 0
	}

	pub fn pin(&self, index: usize) -> PinInfo {
		PinInfo {
			function:          self.pin_function(index),
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

	fn read_pin_bits(&self, index: usize, base: Register, pins_per_register: u8, bits_per_pin: u8) -> u32 {
		crate::assert_pin_index(index);

		let pins_per_register = pins_per_register as usize;
		let bits_per_pin      = bits_per_pin      as usize;

		// Register has a relative byte address,
		// but registers are 32 bit.
		let base           = base as usize / 4;
		let register_index = base + index / pins_per_register;
		let index          = index % pins_per_register;

		let value = self.data[register_index] >> (bits_per_pin * index);
		let mask  = !(std::u32::MAX << bits_per_pin);
		value & mask
	}
}
