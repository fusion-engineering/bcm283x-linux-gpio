#![feature(asm)]
#![feature(core_intrinsics)]

use nix::sys::mman;

const CONTROL_BLOCK_ADDRESS : i64   = 0x3f200000;
const CONTROL_BLOCK_SIZE    : usize = 0x00000100;

mod read;
mod register;
mod write;

pub use read::GpioState;
pub use read::PinInfo;
pub use register::Register;
pub use write::GpioConfig;
pub use write::GpioPullConfig;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

/// A pull up/down mode for a GPIO pin.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PullMode {
	Float,
	PullDown,
	PullUp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct MaskedValue {
	value : u32,
	mask  : u32,
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

	pub fn to_bits(self) -> u8 {
		match self {
			PinMode::Input  => 0b000,
			PinMode::Output => 0b001,
			PinMode::Alt0   => 0b100,
			PinMode::Alt1   => 0b101,
			PinMode::Alt2   => 0b110,
			PinMode::Alt3   => 0b111,
			PinMode::Alt4   => 0b011,
			PinMode::Alt5   => 0b010,
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
	pub fn new() -> nix::Result<Rpio> {
		use nix::{fcntl::OFlag, sys::stat::Mode};

		let fd = nix::fcntl::open("/dev/mem", OFlag::O_CLOEXEC | OFlag::O_RDONLY, Mode::empty())?;
		let control_block = unsafe { mman::mmap(std::ptr::null_mut(), CONTROL_BLOCK_SIZE, mman::ProtFlags::PROT_READ, mman::MapFlags::MAP_SHARED, fd, CONTROL_BLOCK_ADDRESS)? };
		drop(nix::unistd::close(fd));

		Ok(Self {
			control_block
		})
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
	pub unsafe fn and_register(&mut self, reg: Register, value: u32) -> u32 {
		std::intrinsics::atomic_and(self.register_address_mut(reg), value)
	}

	/// Perform an atomic bitwise OR on the contents of a register.
	pub unsafe fn or_register(&mut self, reg: Register, value: u32) -> u32 {
		std::intrinsics::atomic_or(self.register_address_mut(reg), value)
	}

	/// Perform an atomic bitwise XOR on the contents of a register.
	pub unsafe fn xor_register(&mut self, reg: Register, value: u32) -> u32 {
		std::intrinsics::atomic_xor(self.register_address_mut(reg), value)
	}

	/// Perform an atomic bitwise NAND on the contents of a register.
	pub unsafe fn nand_register(&mut self, reg: Register, value: u32) -> u32 {
		std::intrinsics::atomic_xor(self.register_address_mut(reg), value)
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
