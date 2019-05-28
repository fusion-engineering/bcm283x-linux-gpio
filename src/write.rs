use crate::{PinMode, PullMode, Register, Rpio};

/// Wait for one clock cycle.
fn nop() {
	unsafe { asm!("nop") }
}

/// Wait for a number of clock cycles.
///
/// This function will probably wait for a bit more,
/// since it is implemented using a nop-loop.
fn wait_cycles(cycles: usize) {
	for _ in 0..cycles {
		nop();
	}
}

/// A GPIO config that can be applied at once.
///
/// The configuration will only change the bits associated with the settings to apply.
/// For example, setting the function of pin 1 will not change the function of pin 2.
#[derive(Clone)]
pub struct GpioConfig {
	pub function          : [Option<PinMode>; 54],
	pub level             : [Option<bool>; 54],
	pub detect_rise       : [Option<bool>; 54],
	pub detect_fall       : [Option<bool>; 54],
	pub detect_high       : [Option<bool>; 54],
	pub detect_low        : [Option<bool>; 54],
	pub detect_async_rise : [Option<bool>; 54],
	pub detect_async_fall : [Option<bool>; 54],
}

/// The configuration for GPIO pull up/down modes.
///
/// These are seperate from the regular GPIO configuration
/// because they can not be set atomatically.
///
/// Because of that, the [`apply`] function is unsafe.
#[derive(Clone)]
pub struct GpioPullConfig {
	pub pull_mode : [Option<PullMode>; 54],
}

impl GpioConfig {
	pub fn new() -> Self {
		Self {
			function          : [None; 54],
			level             : [None; 54],
			detect_rise       : [None; 54],
			detect_fall       : [None; 54],
			detect_high       : [None; 54],
			detect_low        : [None; 54],
			detect_async_fall : [None; 54],
			detect_async_rise : [None; 54],
		}
	}

	pub fn set_function(&mut self, pin: usize, mode: PinMode) {
		self.function[pin] = Some(mode);
	}

	pub fn set_level(&mut self, pin: usize, level: bool) {
		self.level[pin] = Some(level);
	}

	pub fn set_detect_rise(&mut self, pin: usize, detect: bool) {
		self.detect_rise[pin] = Some(detect);
	}

	pub fn set_detect_fall(&mut self, pin: usize, detect: bool) {
		self.detect_fall[pin] = Some(detect);
	}

	pub fn set_detect_high(&mut self, pin: usize, detect: bool) {
		self.detect_high[pin] = Some(detect);
	}

	pub fn set_detect_low(&mut self, pin: usize, detect: bool) {
		self.detect_low[pin] = Some(detect);
	}

	pub fn set_detect_async_rise(&mut self, pin: usize, detect: bool) {
		self.detect_async_rise[pin] = Some(detect);
	}

	pub fn set_detect_async_fall(&mut self, pin: usize, detect: bool) {
		self.detect_async_fall[pin] = Some(detect);
	}

	/// Apply the configuration.
	pub fn apply(&self, rpio: &mut Rpio) {
		unsafe {
			self.apply_functions(rpio);

			apply_registers(rpio, Register::ren,  &self.detect_rise);
			apply_registers(rpio, Register::fen,  &self.detect_fall);
			apply_registers(rpio, Register::hen,  &self.detect_high);
			apply_registers(rpio, Register::len,  &self.detect_low);
			apply_registers(rpio, Register::aren, &self.detect_async_rise);
			apply_registers(rpio, Register::afen, &self.detect_async_fall);
		}
	}

	unsafe fn apply_functions(&self, rpio: &mut Rpio) {
		let mut mask  = [0u32; 6];
		let mut value = [0u32; 6];

		for (pin, function) in self.function.iter().enumerate() {
			if let Some(function) = function {
				let reg   = pin / 10;
				let index = pin % 10;
				mask[reg]  |= 0b111 << (index * 3);
				value[reg] |= u32::from(function.to_bits()) << (index * 3);
			}
		}

		for i in 0..6 {
			// Zero all pins that we're chaning.
			// This will set them to inputs, but that should be safe.
			rpio.and_register(Register::fsel(i), !mask[i]);

			// Then set the actual functions.
			rpio.or_register(Register::fsel(i), value[i]);
		}
	}
}

impl GpioPullConfig {
	pub fn new() -> Self {
		Self {
			pull_mode: [None; 54],
		}
	}

	pub fn set_pull_mode(&mut self, pin: usize, mode: PullMode) {
		self.pull_mode[pin] = Some(mode);
	}

	/// Apply the configuration.
	///
	/// This is not atomic.
	/// If another process or the kernel is trying to change pull up/down
	/// settings at the same time, the wrong type of pull up/down may be applied to pins.
	pub unsafe fn apply(&self, rpio: &mut Rpio) {
		let mut float_clk     = [0u32; 2];
		let mut pull_up_clk   = [0u32; 2];
		let mut pull_down_clk = [0u32; 2];

		for (i, mode) in self.pull_mode.iter().enumerate() {
			match mode {
				None => (),
				Some(PullMode::Float)    =>     float_clk[i / 32] |= 1 << (i % 32),
				Some(PullMode::PullUp)   =>   pull_up_clk[i / 32] |= 1 << (i % 32),
				Some(PullMode::PullDown) => pull_down_clk[i / 32] |= 1 << (i % 32),
			}
		}

		Self::apply_pull_mode(rpio, 0b00, float_clk);
		Self::apply_pull_mode(rpio, 0b10, pull_up_clk);
		Self::apply_pull_mode(rpio, 0b01, pull_down_clk);
	}

	unsafe fn apply_pull_mode(rpio: &mut Rpio, mode: u32, pins: [u32; 2]) {
		// Do nothing if not necessary.
		if pins[0] == 0 && pins[1] == 0 {
			return;
		}

		// Set the pull up/down bits and wait for 150 cycles.
		rpio.write_register(Register::GPPUDCLK0, 0);
		rpio.write_register(Register::GPPUDCLK1, 0);
		rpio.write_register(Register::GPPUD, mode);
		wait_cycles(150);

		// Set the clock for the pins to modify and wait 150 cycles.
		rpio.write_register(Register::GPPUDCLK0, pins[0]);
		rpio.write_register(Register::GPPUDCLK1, pins[1]);
		wait_cycles(150);

		// Clear the signal and the clocks.
		rpio.write_register(Register::GPPUDCLK0, 0);
		rpio.write_register(Register::GPPUDCLK1, 0);
		rpio.write_register(Register::GPPUD,     0);
	}
}

unsafe fn apply_registers<F>(rpio: &mut Rpio, register: F, values: &[Option<bool>; 54])
where
	F: Fn(usize) -> Register,
{
	let mut out_l = [0u32; 2];
	let mut out_h = [0u32; 2];

	for (pin, value) in values.iter().enumerate() {
		if let Some(bit) = value {
			let reg_i =  pin / 32;
			let index = (pin % 32) as u32;
			out_l[reg_i] |= 1 << index;
			out_h[reg_i] |= u32::from(*bit) << index;
		}
	}

	for i in 0..2 {
		// Zero all bits that we're changing.
		rpio.and_register(register(i), !out_l[i]);

		// Then or the ones into them.
		rpio.or_register(register(i), out_h[i]);
	}
}
