#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Register {
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

	GPEDS0 = 0x40,
	GPEDS1 = 0x44,

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

impl Register {
	pub fn fsel(index: usize) -> Self {
		match index {
			0 => Register::GPFSEL0,
			1 => Register::GPFSEL1,
			2 => Register::GPFSEL2,
			3 => Register::GPFSEL3,
			4 => Register::GPFSEL4,
			5 => Register::GPFSEL5,
			_ => panic!("GPFSEL register index must be in the range [0..6), got {}", index),
		}
	}

	pub fn set(index: usize) -> Self {
		match index {
			0 => Register::GPSET0,
			1 => Register::GPSET1,
			_ => panic!("GPSET register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn clr(index: usize) -> Self {
		match index {
			0 => Register::GPCLR0,
			1 => Register::GPCLR1,
			_ => panic!("GPCLR register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn lev(index: usize) -> Self {
		match index {
			0 => Register::GPLEV0,
			1 => Register::GPLEV1,
			_ => panic!("GPLEV register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn eds(index: usize) -> Self {
		match index {
			0 => Register::GPEDS0,
			1 => Register::GPEDS1,
			_ => panic!("GPEDS register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn ren(index: usize) -> Self {
		match index {
			0 => Register::GPREN0,
			1 => Register::GPREN1,
			_ => panic!("GPREN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn fen(index: usize) -> Self {
		match index {
			0 => Register::GPFEN0,
			1 => Register::GPFEN1,
			_ => panic!("GPFEN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn hen(index: usize) -> Self {
		match index {
			0 => Register::GPHEN0,
			1 => Register::GPHEN1,
			_ => panic!("GPHEN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn len(index: usize) -> Self {
		match index {
			0 => Register::GPLEN0,
			1 => Register::GPLEN1,
			_ => panic!("GPLEN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn aren(index: usize) -> Self {
		match index {
			0 => Register::GPAREN0,
			1 => Register::GPAREN1,
			_ => panic!("GPAREN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn afen(index: usize) -> Self {
		match index {
			0 => Register::GPAFEN0,
			1 => Register::GPAFEN1,
			_ => panic!("GPAFEN register index must be in the range [0..2), got {}", index),
		}
	}

	pub fn pud() -> Self {
		Register::GPPUD
	}

	pub fn pudclk(index: usize) -> Self {
		match index {
			0 => Register::GPPUDCLK0,
			1 => Register::GPPUDCLK1,
			_ => panic!("GPPUDCLK register index must be in the range [0..2), got {}", index),
		}
	}
}
