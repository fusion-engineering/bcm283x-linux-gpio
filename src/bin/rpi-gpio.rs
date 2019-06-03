use rpi_gpio::{
	check_bcm283x_gpio,
	GpioConfig,
	GpioPullConfig,
	Rpio,
	PinInfo,
	PinMode,
	PullMode,
};

use structopt::StructOpt;

#[derive(Clone, Debug, Default)]
struct PinCommand {
	index                 : usize,
	set_level             : Option<bool>,
	set_function          : Option<PinMode>,
	set_pull_mode         : Option<PullMode>,
	set_detect_rise       : Option<bool>,
	set_detect_fall       : Option<bool>,
	set_detect_high       : Option<bool>,
	set_detect_low        : Option<bool>,
	set_detect_async_rise : Option<bool>,
	set_detect_async_fall : Option<bool>,
}

impl PinCommand {
	fn new(index: usize) -> Self {
		Self {
			index,
			.. Default::default()
		}
	}
}

#[derive(StructOpt)]
#[structopt(max_term_width = 120)]
#[structopt(raw(setting = "structopt::clap::AppSettings::DeriveDisplayOrder"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::UnifiedHelpMessage"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Options {
	/// Show more information.
	#[structopt(long = "verbose", short = "v")]
	verbose: bool,

	/// Dangerous: skip the verification of the CPU.
	#[structopt(long = "no-verify-cpu")]
	no_verify_cpu: bool,

	/// Configure a GPIO pin.
	#[structopt(
		long = "set-pin",
		short = "s",
		value_name = "CONFIG",
		number_of_values = 1,
	)]
	pins: Vec<PinCommand>,
}

fn main() {
	let options = Options::from_args();

	if !options.no_verify_cpu {
		if let Some(error) = check_bcm283x_gpio().err() {
			eprintln!("Error: {}", error);
			eprintln!("");
			eprintln!("Failed to verify the CPU type. Make sure the program is being run on a BCM2835/7 CPU.");
			eprintln!("Alternatively, add --no-verify-cpu to the command line, but note that this could be dangerous.");
			std::process::exit(1);
		}
	}

	let mut rpio = match Rpio::new() {
		Ok(x) => x,
		Err(error) => {
			eprintln!("{}", error);
			eprintln!();
			eprintln!("Make sure to run the application as root on a BCM2835/7 CPU and that your kernel was configured properly.");
			eprintln!("You may need to disable CONFIG_IO_STRICT_DEVMEM and add iomem=relaxed to the kernel command line.");
			std::process::exit(1);
		}
	};

	if options.verbose {
		let address = rpio.control_block() as usize;
		eprintln!("mapped IO control block at: 0x{:X}", address);
	}

	if !options.pins.is_empty() {
		let (gpio, pud) = config_from_commands(&options.pins);
		gpio.apply(&mut rpio);
		unsafe {
			pud.apply(&mut rpio);
		}
	}

	for (index, pin) in rpio.read_all().pins().iter().enumerate() {
		print_pin(index, pin, options.verbose);
	}
}

fn print_pin(index: usize, pin: &PinInfo, verbose: bool) {
	use yansi::Paint;

	let level = match pin.level {
		true  => Paint::green("HIGH"),
		false => Paint::red("LOW"),
	};

	let mode = format!("{:?}", pin.mode);
	print!("pin={:<2}   level={:4}   mode={:6}", Paint::yellow(index), level, Paint::cyan(mode));

	if verbose {
		let event = match pin.level {
			true  => Paint::green("yes"),
			false => Paint::red("no "),
		};
		print!("   event={}   detect=", event);

		let mut detect_any = false;
		if pin.detect_rise {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("rise"));
		}
		if pin.detect_fall {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("fall"));
		}
		if pin.detect_high {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("high"));
		}
		if pin.detect_low {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("low"));
		}
		if pin.detect_async_rise {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("async_rise"));
		}
		if pin.detect_async_fall {
			if std::mem::replace(&mut detect_any, true) {
				print!(",");
			}
			print!("{}", Paint::cyan("async_fall"));
		}
		if !detect_any {
			print!("{}", Paint::magenta("nothing"));
		}
	}

	println!();
}

fn partition<'a>(input: &'a str, split_on: char) -> (&'a str, Option<&'a str>) {
	let mut parts = input.splitn(2, split_on);
	(parts.next().unwrap(), parts.next())
}

fn split_key_value(input: &str) -> (&str, Option<&str>) {
	let (key, value) = partition(input, '=');
	let key = key.trim();
	let value = value.map(str::trim);
	(key, value)
}

impl std::str::FromStr for PinCommand {
	type Err = String;
	fn from_str(data: &str) -> Result<Self, Self::Err> {
		let mut fields = data.split(",").map(str::trim).filter(|x| !x.is_empty());

		let index  = fields.next().unwrap();
		let index  = usize::from_str(index).map_err(|_| format!("invalid pin index: {}", index))?;
		if index > 53 {
			return Err(format!("pin index out of range [0-53]: {}", index));
		}

		let mut command = PinCommand::new(index);
		for field in fields {
			let (key, value) = split_key_value(field);

			let value = match value {
				Some(x) => x,
				None => return Err(format!("missing value for option `{}`", key)),
			};

			match key {
				"level"             => set_bool(&mut command.set_level, key, value)?,
				"function"          => set_function(&mut command.set_function, key, value)?,
				"mode"              => set_function(&mut command.set_function, key, value)?,
				"pull"              => set_pull(&mut command.set_pull_mode, key, value)?,
				"detect-rise"       => set_bool(&mut command.set_detect_rise, key, value)?,
				"detect-fall"       => set_bool(&mut command.set_detect_fall, key, value)?,
				"detect-high"       => set_bool(&mut command.set_detect_high, key, value)?,
				"detect-low"        => set_bool(&mut command.set_detect_low, key, value)?,
				"detect-async-rise" => set_bool(&mut command.set_detect_async_rise, key, value)?,
				"detect-async-fall" => set_bool(&mut command.set_detect_async_fall, key, value)?,
				_ => return Err(format!("unknown pin option: `{}`", key)),
			}
		}

		Ok(command)
	}
}

fn set_bool(dest: &mut Option<bool>, key: &str, value: &str) -> Result<(), String> {
	if dest.is_some() {
		return Err(format!("option `{}` already set", key))
	}

	dest.replace(match value {
		"on"  | "high" | "true"  | "1" => true,
		"off" | "low"  | "false" | "0" => false,
		_ => return Err(format!("invalid boolean: {}, expected on, high, true, 1, off, low, false or 0", value)),
	});

	Ok(())
}

fn set_function(dest: &mut Option<PinMode>, key: &str, value: &str) -> Result<(), String> {
	if dest.is_some() {
		return Err(format!("option `{}` already set", key))
	}

	dest.replace(match value {
		"input"  | "in"  => PinMode::Input,
		"output" | "out" => PinMode::Output,
		"alt0"           => PinMode::Alt0,
		"alt1"           => PinMode::Alt1,
		"alt2"           => PinMode::Alt2,
		"alt3"           => PinMode::Alt3,
		"alt4"           => PinMode::Alt4,
		"alt5"           => PinMode::Alt5,
		_ => return Err(format!("unknown pin function: {}, expected input, output or alt0..5", value)),
	});

	Ok(())
}

fn set_pull(dest: &mut Option<PullMode>, key: &str, value: &str) -> Result<(), String> {
	if dest.is_some() {
		return Err(format!("option `{}` already set", key))
	}

	dest.replace(match value {
		"up"   => PullMode::PullUp,
		"down" => PullMode::PullDown,
		"float"=> PullMode::Float,
		_ => return Err(format!("unknown pull up/down mode: {}, expected up, down or float", value)),
	});

	Ok(())
}

fn config_from_commands(commands: &[PinCommand]) -> (GpioConfig, GpioPullConfig) {
	let mut gpio = rpi_gpio::GpioConfig::new();
	let mut pud  = rpi_gpio::GpioPullConfig::new();

	for pin in commands {
		if let Some(value) = pin.set_level {
			gpio.set_level(pin.index, value);
		}
		if let Some(value) = pin.set_function {
			gpio.set_function(pin.index, value);
		}
		if let Some(value) = pin.set_pull_mode {
			pud.set_pull_mode(pin.index, value);
		}
		if let Some(value) = pin.set_detect_rise {
			gpio.set_detect_rise(pin.index, value);
		}
		if let Some(value) = pin.set_detect_fall {
			gpio.set_detect_fall(pin.index, value);
		}
		if let Some(value) = pin.set_detect_high {
			gpio.set_detect_high(pin.index, value);
		}
		if let Some(value) = pin.set_detect_low {
			gpio.set_detect_low(pin.index, value);
		}
		if let Some(value) = pin.set_detect_async_rise {
			gpio.set_detect_async_rise(pin.index, value);
		}
		if let Some(value) = pin.set_detect_async_fall {
			gpio.set_detect_async_fall(pin.index, value);
		}
	}

	(gpio, pud)
}
