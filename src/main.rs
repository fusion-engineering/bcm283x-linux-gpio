pub mod lib;
use lib::{Rpio, PinInfo};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
	#[structopt(long = "verbose", short = "v")]
	/// Show more information.
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
