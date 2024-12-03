use std::{
	mem,
	ops::{
		Add,
		Div,
		Mul,
	},
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		newline,
		one_of,
		space1,
	},
	combinator::{
		map,
		value,
	},
	multi::separated_list1,
	sequence::separated_pair,
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2022, 2, |t| t.parse_dyn_puzzle::<RockPaperScissors>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RockPaperScissors {
	pt1: Vec<(Action, Action)>,
	pt2: Vec<(Action, Outcome)>,
}

impl<'a> Parsed<&'a str> for RockPaperScissors {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(
				newline,
				separated_pair(Action::parse_wyz, space1, Action::parse_wyz),
			),
			|pt1| Self {
				pt1,
				pt2: Vec::new(),
			},
		)(text)
	}
}

impl Puzzle for RockPaperScissors {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.pt1
			.iter()
			.copied()
			.map(|(them, you)| (you, you * them))
			.map(|(you, res)| you + res)
			.sum::<i64>()
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.pt2 = mem::take(&mut self.pt1)
			.into_iter()
			.map(|(a, b)| (a, b.into()))
			.collect();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.pt2
			.iter()
			.copied()
			.map(|(them, res)| (res / them, res))
			.map(|(you, res)| you + res)
			.sum::<i64>()
			.pipe(Ok)
	}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
	Rock     = 1,
	Paper    = 2,
	Scissors = 3,
}

impl<'a> Parsed<&'a str> for Action {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Rock, one_of("AX")),
			value(Self::Paper, one_of("BY")),
			value(Self::Scissors, one_of("CZ")),
		))(text)
	}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Outcome {
	Win  = 6,
	Draw = 3,
	Loss = 0,
}

impl<'a> Parsed<&'a str> for Outcome {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Loss, tag("X")),
			value(Self::Draw, tag("Y")),
			value(Self::Win, tag("Z")),
		))(text)
	}
}

impl From<Action> for Outcome {
	fn from(a: Action) -> Self {
		match a {
			Action::Rock => Self::Loss,
			Action::Paper => Self::Draw,
			Action::Scissors => Self::Win,
		}
	}
}

/// Combines your Action with an opponent's Action to get your Outcome.
impl Mul<Self> for Action {
	type Output = Outcome;

	fn mul(self, other: Self) -> Self::Output {
		match (self, other) {
			// self beats other
			(Self::Rock, Self::Scissors)
			| (Self::Paper, Self::Rock)
			| (Self::Scissors, Self::Paper) => Outcome::Win,
			// other beats self
			(Self::Rock, Self::Paper)
			| (Self::Paper, Self::Scissors)
			| (Self::Scissors, Self::Rock) => Outcome::Loss,
			// draw
			_ => Outcome::Draw,
		}
	}
}

/// Adds your Action to the game Outcome for your score.
impl Add<Outcome> for Action {
	type Output = i64;

	fn add(self, outcome: Outcome) -> Self::Output {
		(self as u8 + outcome as u8) as i64
	}
}

/// Given an Outcome and an opponent's Action, produces the Action you need to
/// get your desired Outcome.
impl Div<Action> for Outcome {
	type Output = Action;

	fn div(self, action: Action) -> Self::Output {
		match (self, action) {
			(Self::Win, Action::Rock)
			| (Self::Draw, Action::Paper)
			| (Self::Loss, Action::Scissors) => Action::Paper,
			(Self::Win, Action::Paper)
			| (Self::Draw, Action::Scissors)
			| (Self::Loss, Action::Rock) => Action::Scissors,
			(Self::Win, Action::Scissors)
			| (Self::Draw, Action::Rock)
			| (Self::Loss, Action::Paper) => Action::Rock,
		}
	}
}
