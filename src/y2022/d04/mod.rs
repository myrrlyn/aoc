use std::{
	fmt,
	ops::RangeInclusive,
};

use nom::{
	bytes::complete::tag,
	character::complete::{
		newline,
		u64 as get_u64,
	},
	combinator::map,
	multi::separated_list1,
	sequence::separated_pair,
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2022, 4, |t| t.parse_dyn_puzzle::<Camp>());

#[derive(Clone, Debug, Default)]
pub struct Camp {
	chores: Vec<Assignments>,
}

impl<'a> Parsed<&'a str> for Camp {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(newline, Assignments::parse_wyz), |chores| {
			Self { chores }
		})(text)
	}
}

impl Puzzle for Camp {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.chores
			.iter()
			.filter(|Assignments { left, right }| {
				let lnum = [left.start(), left.end()];
				let rnum = [right.start(), right.end()];

				rnum.into_iter().all(|r| left.contains(r))
					|| lnum.into_iter().all(|l| right.contains(l))
			})
			.count()
			.pipe(|c| Ok(c as i64))
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.chores
			.iter()
			.filter(|Assignments { left, right }| {
				let lnum = [left.start(), left.end()];
				let rnum = [right.start(), right.end()];

				rnum.into_iter().any(|r| left.contains(r))
					|| lnum.into_iter().any(|l| right.contains(l))
			})
			.count()
			.pipe(|c| Ok(c as i64))
	}
}

#[derive(Clone)]
pub struct Assignments {
	left:  RangeInclusive<usize>,
	right: RangeInclusive<usize>,
}

impl<'a> Parsed<&'a str> for Assignments {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		fn get_usize<'a>(text: &'a str) -> ParseResult<&'a str, usize> {
			map(get_u64, |n| n as usize)(text)
		}
		fn range<'a>(
			text: &'a str,
		) -> ParseResult<&'a str, RangeInclusive<usize>> {
			map(
				separated_pair(get_usize, tag("-"), get_usize),
				|(bgn, end)| bgn ..= end,
			)(text)
		}
		map(separated_pair(range, tag(","), range), |(left, right)| {
			Self { left, right }
		})(text)
	}
}

impl fmt::Debug for Assignments {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(
			fmt,
			"{lbgn}{sep}..={sep}{lend} / {rbgn}{sep}..={sep}{rend}",
			lbgn = self.left.start(),
			lend = self.left.end(),
			rbgn = self.right.start(),
			rend = self.right.end(),
			sep = if fmt.alternate() { " " } else { "" },
		)
	}
}
