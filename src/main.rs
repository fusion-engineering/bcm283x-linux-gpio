pub mod lib;
use lib::{Rpio, PinInfo};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
	verbose: bool,
}

fn main() {
	let options = Options::from_args();

	let rpio = Rpio::new().unwrap();

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

	print!("{:02}: {:4} ({:?})", index, level, Paint::cyan(pin.mode));

	if verbose {
		let event = match pin.level {
			true  => Paint::green("yes"),
			false => Paint::red("no "),
		};
		print!(" (event: {}", event);

		if pin.detect_rise {
			print!(", detect_rise");
		}
		if pin.detect_fall {
			print!(", detect_fall");
		}
		if pin.detect_high {
			print!(", detect_high");
		}
		if pin.detect_low {
			print!(", detect_low");
		}
		if pin.detect_async_rise {
			print!(", detect_async_rise");
		}
		if pin.detect_async_fall {
			print!(", detect_async_fall");
		}
		print!(")");
	}

	println!();
}
