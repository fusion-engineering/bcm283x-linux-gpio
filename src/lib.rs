use nix::sys::mman;

const CONTROL_BLOCK_ADDRESS : i64   = 0x3f200000;
const CONTROL_BLOCK_SIZE    : usize = 0x00000100;

pub enum Reg {
	GPFSEL0 = 0x00,
	GPFSEL1 = 0x04,
	GPFSEL2 = 0x08,
	GPFSEL3 = 0x0C,
	GPFSEL4 = 0x10,
	GPFSEL5 = 0x14,

	GPSET0  = 0x1C,
	GPSET1  = 0x20,

	GPCLR0  = 0x28,
	GPCLR1  = 0x2C,

	GPLEV0  = 0x34,
	GPLEV1  = 0x38,

	GPPEDS0 = 0x40,
	GPPEDS1 = 0x44,

	GPREN0  = 0x4C,
	GPREN1  = 0x50,

	GPFEN0  = 0x58,
	GPFEN1  = 0x5C,

	GPHEN0  = 0x64,
	GPHEN1  = 0x68,

	GPLEN0  = 0x70,
	GPLEN1  = 0x74,

	GPAREN0 = 0x7C,
	GPAREN1 = 0x80,

	GPAFEN0 = 0x88,
	GPAFEN1 = 0x8C,

	GPPUD     = 0x94,
	GPPUDCLK0 = 0x98,
	GPPUDCLK1 = 0x9C,
}

pub struct Rpio {
	control_block: *mut std::ffi::c_void,
}

impl Rpio {
	pub fn new() -> nix::Result<Rpio> {
		use nix::{fcntl::OFlag, sys::stat::Mode};

		let fd = nix::fcntl::open("/dev/mem", OFlag::O_CLOEXEC | OFlag::O_RDONLY, Mode::empty())?;
		let control_block = unsafe { mman::mmap(std::ptr::null_mut(), CONTROL_BLOCK_SIZE, mman::ProtFlags::PROT_READ, mman::MapFlags::MAP_SHARED, fd, CONTROL_BLOCK_ADDRESS)? };
		drop(nix::unistd::close(fd));

		Ok(Self {
			control_block
		})
	}

	pub fn read_all(&self) -> RpioState {
		let address = self.control_block as *const [u32; 0x100];
		RpioState::from_data(unsafe { std::ptr::read_volatile(address) })
	}

	pub fn read_register(&self, reg: Reg) -> u32 {
		unsafe { std::ptr::read_volatile(self.register_address(reg)) }
	}

	pub fn write_register(&mut self, reg: Reg, value: u32) {
		unsafe { std::ptr::write_volatile(self.register_address_mut(reg), value) }
	}

	fn register_address(&self, reg: Reg) -> *const u32 {
		self.control_block.wrapping_add(reg as usize) as *const u32
	}

	fn register_address_mut(&self, reg: Reg) -> *mut u32 {
		self.control_block.wrapping_add(reg as usize) as *mut u32
	}
}

impl Drop for Rpio {
	fn drop(&mut self) {
		unsafe {
			drop(mman::munmap(self.control_block, CONTROL_BLOCK_SIZE))
		}
	}
}

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PinMode {
	Input,
	Output,
	Alt0,
	Alt1,
	Alt2,
	Alt3,
	Alt4,
	Alt5,
}

impl PinMode {
	pub fn try_from_bits(bits: u8) -> Result<Self, ()> {
		match bits {
			0b000 => Ok(PinMode::Input),
			0b001 => Ok(PinMode::Output),
			0b100 => Ok(PinMode::Alt0),
			0b101 => Ok(PinMode::Alt1),
			0b110 => Ok(PinMode::Alt2),
			0b111 => Ok(PinMode::Alt3),
			0b011 => Ok(PinMode::Alt4),
			0b010 => Ok(PinMode::Alt5),
			_     => Err(())
		}
	}
}

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
