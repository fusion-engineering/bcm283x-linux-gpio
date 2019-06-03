#![feature(asm)]
#![feature(core_intrinsics)]

use nix::sys::mman;
use std::fmt::Display;
use std::io::Read;

const CONTROL_BLOCK_SIZE : usize = 0x00000100;

mod read;
mod register;
mod write;

use nix::errno::Errno;

pub use read::GpioState;
pub use read::PinInfo;
pub use register::Register;
pub use write::GpioConfig;
pub use write::GpioPullConfig;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
	message: String,
	errno: Option<Errno>,
}

impl Error {
	fn new(message: impl std::string::ToString, errno: Option<Errno>) -> Self {
		Self { message: message.to_string(), errno }
	}

	fn from_nix(message: impl std::string::ToString, error: nix::Error) -> Self {
		Self::new(message, error.as_errno())
	}

	fn from_io(message: impl std::string::ToString, error: std::io::Error) -> Self {
		let errno = error.raw_os_error().map(Errno::from_i32);
		Self::new(message, errno)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.errno {
			None => write!(f, "{}", self.message),
			Some(errno) => write!(f, "{}: {}", self.message, errno),
		}
	}
}

impl std::error::Error for Error {}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PinFunction {
	Input,
	Output,
	Alt0,
	Alt1,
	Alt2,
	Alt3,
	Alt4,
	Alt5,
}

/// A pull up/down mode for a GPIO pin.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PullMode {
	Float,
	PullDown,
	PullUp,
}

impl PinFunction {
	pub fn try_from_bits(bits: u8) -> Result<Self, ()> {
		match bits {
			0b000 => Ok(PinFunction::Input),
			0b001 => Ok(PinFunction::Output),
			0b100 => Ok(PinFunction::Alt0),
			0b101 => Ok(PinFunction::Alt1),
			0b110 => Ok(PinFunction::Alt2),
			0b111 => Ok(PinFunction::Alt3),
			0b011 => Ok(PinFunction::Alt4),
			0b010 => Ok(PinFunction::Alt5),
			_     => Err(())
		}
	}

	pub fn to_bits(self) -> u8 {
		match self {
			PinFunction::Input  => 0b000,
			PinFunction::Output => 0b001,
			PinFunction::Alt0   => 0b100,
			PinFunction::Alt1   => 0b101,
			PinFunction::Alt2   => 0b110,
			PinFunction::Alt3   => 0b111,
			PinFunction::Alt4   => 0b011,
			PinFunction::Alt5   => 0b010,
		}
	}
}

pub struct Rpio {
	control_block: *mut std::ffi::c_void,
}

impl Rpio {
	/// Create a new handle to the GPIO peripheral.
	///
	/// This will attempt to map a portion of /dev/mem,
	/// in order to access the memory mapped GPIO peripheral.
	///
	/// This may fail if:
	///  - we don't have root permission.
	///  - the kernel was compiled with CONFIG_IO_STRICT_DEVMEM.
	///  - the kernel was compiled with CONFIG_STRICT_DEVMEM,
	///    and not started with `iomem=relaxed` on the kernel command line.
	pub fn new() -> Result<Rpio, Error> {
		use std::os::unix::io::AsRawFd;

		let gpio_address = read_gpio_address()?;

		let file = open_rw("/dev/mem")?;
		let fd   = file.file.as_raw_fd();
		let control_block = unsafe {
			mman::mmap(std::ptr::null_mut(), CONTROL_BLOCK_SIZE, mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE, mman::MapFlags::MAP_SHARED, fd, gpio_address)
				.map_err(|e| Error::from_nix(format!("failed to map GPIO memory (0x{:08X}) from /dev/mem", gpio_address), e))?
		};

		Ok(Self { control_block })
	}

	/// Get the pointer to the mapped control block.
	pub fn control_block(&self) -> *mut std::ffi::c_void {
		self.control_block
	}

	/// Read the entire current GPIO state.
	pub fn read_all(&self) -> GpioState {
		let address = self.control_block as *const [u32; 0x100];
		GpioState::from_data(unsafe { std::ptr::read_volatile(address) })
	}

	/// Read a value from a register.
	pub fn read_register(&self, reg: Register) -> u32 {
		unsafe { std::ptr::read_volatile(self.register_address(reg)) }
	}

	/// Write a value to a register.
	pub unsafe fn write_register(&mut self, reg: Register, value: u32) {
		std::ptr::write_volatile(self.register_address_mut(reg), value)
	}

	/// Perform an atomaic bitwise AND on the contents of a register.
	pub unsafe fn and_register(&mut self, reg: Register, value: u32) {
		*self.register_address_mut(reg) &= value;
	}

	/// Perform an atomic bitwise OR on the contents of a register.
	pub unsafe fn or_register(&mut self, reg: Register, value: u32) {
		*self.register_address_mut(reg) |= value;
	}

	/// Perform an atomic bitwise XOR on the contents of a register.
	pub unsafe fn xor_register(&mut self, reg: Register, value: u32) {
		*self.register_address_mut(reg) ^= value;
	}

	/// Read the current level of a GPIO pin.
	pub fn read_level(&self, index: usize) -> bool {
		assert_pin_index(index);
		let value = self.read_register(Register::lev(index / 32));
		let value = value >> (index % 32);
		value & 1 == 1
	}

	/// Atomically set the level of a single GPIO pin.
	pub fn set_level(&mut self, index: usize, value: bool) {
		let bits = 1 << (index % 32);
		let register = match value {
			true  => Register::set(index / 32),
			false => Register::clr(index / 32),
		};
		unsafe { self.write_register(register, bits) }
	}

	fn register_address(&self, reg: Register) -> *const u32 {
		self.control_block.wrapping_add(reg as usize) as *const u32
	}

	fn register_address_mut(&self, reg: Register) -> *mut u32 {
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

fn assert_pin_index(index: usize) {
	assert!(index <= 53, "gpio pin index out of range, expected a value in the range [0-53], got {}", index);
}

fn partition(data: &[u8], split_on: u8) -> Result<(&[u8], &[u8]), ()> {
	let mut iterator = data.splitn(2, |c| *c == split_on);

	let a = match iterator.next() {
		Some(x) => x,
		None    => return Err(()),
	};

	let b = match iterator.next() {
		Some(x) => x,
		None    => return Err(()),
	};

	Ok((a, b))
}

fn is_whitespace(c: u8) -> bool {
	c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

fn trim(data: &[u8]) -> &[u8] {
	let first = match data.iter().position(|x| !is_whitespace(*x)) {
		None => return &data[0..0],
		Some(x) => x,
	};

	let last = match data.iter().rposition(|x| !is_whitespace(*x)) {
		None => return &data[0..0],
		Some(x) => x,
	};

	&data[first..last+1]
}

struct FileWithPath {
	pub path: std::path::PathBuf,
	pub file: std::fs::File,
}

fn open(path: impl Into<std::path::PathBuf>) -> Result<FileWithPath, Error> {
	let path = path.into();
	let file = std::fs::File::open(&path).map_err(|e| Error::from_io(format!("failed to open {}", path.display()), e))?;
	Ok(FileWithPath {
		path,
		file,
	})
}

fn open_rw(path: impl Into<std::path::PathBuf>) -> Result<FileWithPath, Error> {
	let path = path.into();
	let file = std::fs::OpenOptions::new().create(false).read(true).write(true).open(&path)
		.map_err(|e| Error::from_io(format!("failed to open {}", path.display()), e))?;

	Ok(FileWithPath {
		path,
		file,
	})
}

fn read_all(file: FileWithPath) -> Result<Vec<u8>, Error> {
	let mut file = file;
	let mut data = Vec::new();
	file.file.read_to_end(&mut data).map_err(|e| Error::from_io(format!("failed to read from {}", file.path.display()), e))?;
	Ok(data)
}

/// Check whether the current platform has a bcm2835-gpio peripheral at the expected bus address.
pub fn check_bcm283x_gpio() -> Result<(), Error> {
	const EXPECTED: &str = "brcm,bcm2835-gpio";

	let file = open("/proc/device-tree/soc/gpio@7e200000/compatible")?;
	let mut data = read_all(file)?;
	if data[data.len() - 1] == 0 {
		data.pop();
	}

	if data == EXPECTED.as_bytes() {
		Ok(())
	} else {
		Err(Error::new(format!("invalid gpio peripheral type, expected {}, got {:?}", EXPECTED, String::from_utf8_lossy(&data)), None))
	}
}

/// Read the GPIO peripheral base address from /proc/iomem.
fn read_gpio_address() -> Result<i64, Error> {
	let file = open("/proc/iomem")?;
	let data = read_all(file)?;

	// Loop over lines.
	for (i, line) in data.split(|c| *c == b'\n').enumerate().filter(|(_, line)| !line.is_empty()) {
		// Split kernel range from peripheral name.
		let (range, peripheral) = partition(line, b':').map_err(|_| Error::new(format!("malformed entry in /proc/iomem on line {}", i), None))?;
		let range = trim(range);
		let peripheral = trim(peripheral);

		if peripheral.ends_with(b".gpio") {
			let (start, _end) = partition(range, b'-').map_err(|_| Error::new(format!("malformed entry in /proc/iomem on line {}", i), None))?;
			let start = std::str::from_utf8(start).map_err(|_| Error::new(format!("malformed entry in /proc/iomem on line {}", i), None))?;
			let start = i64::from_str_radix(start, 16).map_err(|_| Error::new(format!("invalid start address in /proc/iomem on line {}: {}", i, start), None))?;
			return Ok(start);
		}
	}

	Err(Error::new(&"failed to find GPIO peripheral in /proc/iomem", None))
}
