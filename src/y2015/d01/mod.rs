use nom::{
	character::complete::{
		newline,
		one_of,
	},
	combinator::{
		map,
		recognize,
	},
	multi::many1,
	sequence::terminated,
};

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2015, 1, |t| t.parse_dyn_puzzle::<Elevator>());

pub struct Elevator {
	sequence: String,
}

impl<'a> Parsed<&'a str> for Elevator {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			terminated(recognize(many1(one_of("()"))), newline),
			|seq: &str| Self {
				sequence: seq.to_owned(),
			},
		)(text)
	}
}

impl Puzzle for Elevator {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.sequence
			.chars()
			.map(|c| match c {
				'(' => 1,
				')' => -1,
				c => {
					tracing::error!(%c, "unexpected character in input");
					0
				},
			})
			.reduce(|a, b| a + b)
			.ok_or_else(|| eyre::eyre!("no characters in input"))
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut current = 0;
		for (c, pos) in self.sequence.chars().zip(1 ..) {
			match c {
				'(' => current += 1,
				')' => current -= 1,
				c => {
					tracing::error!(%c, "unexpected character in input");
					continue;
				},
			}
			if current == -1 {
				return Ok(pos);
			}
		}
		eyre::bail!("never reached the basement");
	}
}
