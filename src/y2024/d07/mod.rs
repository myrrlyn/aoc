use nom::{
	bytes::complete::tag,
	character::complete::{
		newline,
		space1,
	},
	combinator::map,
	multi::separated_list1,
	sequence::separated_pair,
};
use tap::Pipe as _;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2024, 7, |t| t.parse_dyn_puzzle::<CalibrationSet>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CalibrationSet {
	data: Vec<Calibration>,
}

impl Puzzle for CalibrationSet {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.data
			.iter()
			.filter(|c| c.is_solvable(false))
			.map(|c| c.goal)
			.sum::<i64>()
			.pipe(Ok)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.data
			.iter()
			.filter(|c| c.is_solvable(true))
			.map(|c| c.goal)
			.sum::<i64>()
			.pipe(Ok)
	}
}

impl<'a> Parsed<&'a str> for CalibrationSet {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(newline, Calibration::parse_wyz), |data| {
			Self { data }
		})(src)
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Calibration {
	pub goal:     i64,
	pub operands: Vec<i64>,
}

impl Calibration {
	pub fn is_solvable(&self, use_concat: bool) -> bool {
		let mut accumulations = Vec::new();
		let mut operands = self.operands.iter().copied().peekable();
		let Some(first) = operands.next()
		else {
			return false;
		};
		accumulations.push(first);
		while let Some(next) = operands.next() {
			let is_last = operands.peek().is_none();
			for n in 0 .. accumulations.len() {
				let old = accumulations[n];
				let add = old + next;
				let mul = old * next;
				let cat = if use_concat {
					let Ok(cat) = format!("{old}{next}").parse::<i64>()
					else {
						return false;
					};
					cat
				}
				else {
					old
				};
				let mut valid = add == self.goal || mul == self.goal;
				if use_concat {
					valid |= cat == self.goal;
				}
				if is_last && valid {
					return true;
				}
				accumulations[n] = add;
				accumulations.push(mul);
				if use_concat {
					accumulations.push(cat);
				}
			}
		}
		return false;
	}
}

impl<'a> Parsed<&'a str> for Calibration {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_pair(
				parse_number::<i64>,
				tag(": "),
				separated_list1(space1, parse_number::<i64>),
			),
			|(goal, operands)| Self { goal, operands },
		)(src)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_sequences() -> eyre::Result<()> {
		let text = include_str!("sample.txt");
		let (_, cals) = text.parse_wyz::<CalibrationSet>()?;
		assert!(cals.data[1].is_solvable(false));
		assert!(!cals.data[2].is_solvable(false));
		Ok(())
	}
}
