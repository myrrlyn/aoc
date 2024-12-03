use std::collections::BTreeMap;

use nom::{
	character::complete::{
		newline,
		space1,
	},
	multi::many1,
	sequence::{
		separated_pair,
		terminated,
	},
};
use tap::Pipe;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 1, |t| t.parse_dyn_puzzle::<Idents>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Idents {
	left:        Vec<i32>,
	right:       Vec<i32>,
	count_left:  BTreeMap<i32, usize>,
	count_right: BTreeMap<i32, usize>,
}

impl<'a> Parsed<&'a str> for Idents {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		many1(terminated(
			separated_pair(parse_number::<i32>, space1, parse_number::<i32>),
			newline,
		))(src)
		.map(|(rest, list)| (rest, list.into_iter().collect()))
	}
}

impl Puzzle for Idents {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.left.sort();
		self.right.sort();
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.left
			.iter()
			.zip(self.right.iter())
			.map(|(&left, &right)| (left as i64 - right as i64).abs())
			.sum::<i64>()
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for (&left, &right) in self.left.iter().zip(self.right.iter()) {
			*self.count_left.entry(left).or_default() += 1;
			*self.count_right.entry(right).or_default() += 1;
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.count_left
			.iter()
			.map(|(&num, &count)| {
				(num as i64 * count as i64)
					* (*self.count_right.entry(num).or_default() as i64)
			})
			.sum::<i64>()
			.pipe(Ok)
	}
}

impl FromIterator<(i32, i32)> for Idents {
	fn from_iter<T: IntoIterator<Item = (i32, i32)>>(iter: T) -> Self {
		let mut out = Idents::default();
		for (left, right) in iter {
			out.left.push(left);
			out.right.push(right);
		}
		out
	}
}
