use nom::{
	branch::alt,
	bytes::complete::tag,
	combinator::{
		map,
		value,
	},
	sequence::{
		delimited,
		preceded,
		separated_pair,
	},
};
use tap::Pipe;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2024, 3, |t| t.parse_dyn_puzzle::<Instructions>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Instructions {
	found: Vec<Instruction>,
}

impl Puzzle for Instructions {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.found
			.iter()
			.map(|insn| match insn {
				&Instruction::Mul { left, right } => left as i64 * right as i64,
				_ => 0,
			})
			.sum::<i64>()
			.pipe(Ok)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut enabled = true;
		let mut sum = 0;
		for insn in &self.found {
			match insn {
				&Instruction::Enable => enabled = true,
				&Instruction::Disable => enabled = false,
				&Instruction::Mul { left, right } => {
					if enabled {
						sum += left as i64 * right as i64;
					}
				},
			}
		}
		Ok(sum)
	}
}

impl<'a> Parsed<&'a str> for Instructions {
	fn parse_wyz(mut src: &'a str) -> ParseResult<&'a str, Self> {
		let mut found = Vec::new();
		while !src.is_empty() {
			match src.parse_wyz::<Instruction>() {
				Ok((rest, insn)) => {
					found.push(insn);
					src = rest;
				},
				Err(_) => {
					src = match src.chars().next() {
						None => "",
						Some(c) => {
							let mut bytes = [0u8; 4];
							let len = c.encode_utf8(&mut bytes).len();
							&src[len ..]
						},
					}
				},
			}
		}
		Ok(("", Self { found }))
	}
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Instruction {
	Mul { left: i32, right: i32 },
	Enable,
	Disable,
}

impl<'a> Parsed<&'a str> for Instruction {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			map(
				preceded(
					tag("mul"),
					delimited(
						tag("("),
						separated_pair(
							parse_number::<i32>,
							tag(","),
							parse_number::<i32>,
						),
						tag(")"),
					),
				),
				|(left, right)| Self::Mul { left, right },
			),
			value(Self::Enable, tag("do()")),
			value(Self::Disable, tag("don't()")),
		))(src)
	}
}
