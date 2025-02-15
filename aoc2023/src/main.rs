use glue::SolverFn;
use itertools::Itertools;
use macros::*;

use anyhow::Context;
use std::io::{Error, ErrorKind};
use std::{fs, io};

use rest::*;

use_all_days!();

fn main() {
	let mut days: Vec<u8> = Vec::new();
	for arg in std::env::args() {
		if let Some((from, to)) = arg.split_once("..=") {
			if let Ok(range) = from.parse().and_then(|from| to.parse().map(|to| from..=to)) {
				for day in range {
					days.push(day);
				}
			}
		} else if let Some((from, to)) = arg.split_once("..") {
			if let Ok(range) = from.parse().and_then(|from| to.parse().map(|to| from..to)) {
				for day in range {
					days.push(day);
				}
			}
		} else if let Ok(day) = arg.parse() {
			days.push(day);
		}
	}
	for day in days.into_iter().sorted().dedup() {
		match day {
			26.. => eprintln!("{} is not a valid number for a day.", day),
			day => {
				if let Err(e) = run_day(day) {
					eprintln!("Error when trying to run day {}: {}", day, e);
				}
			},
		}
	}
}

struct Solution {
	input: String,
}
impl Solution {
	fn run(&self, parts: &[SolverFn]) -> io::Result<()> {
		if parts.is_empty() {
			Err(Error::new(
				ErrorKind::NotFound,
				"Solution contained no solvers!",
			))
		} else {
			for (index, part) in parts.iter().enumerate() {
				println!("SOLUTION PART {}:\n{}", index + 1, part(&self.input));
			}
			Ok(())
		}
	}
}

fn run_day(day: u8) -> anyhow::Result<()> {
	let path = format!("txt_input/day_{:0>2}.txt", day);
	Solution {
		input: fs::read_to_string(&path).context(format!("could not read '{}'", path))?,
	}
	.run(map_to_part_functions!())?;
	Ok(())
}