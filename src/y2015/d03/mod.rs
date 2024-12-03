use nom::{
	branch::alt,
	bytes::complete::tag,
	combinator::{
		map,
		value,
	},
	multi::many1,
};

use crate::{
	prelude::*,
	Coord2D,
	Grid2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2015, 3, |t| t.parse_dyn_puzzle::<Map>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Map {
	steps: Vec<Step>,
	grid:  Grid2D<i32, i32>,
}

impl<'a> Parsed<&'a str> for Map {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(many1(Step::parse_wyz), |steps| Self {
			steps,
			grid: Grid2D::new(),
		})(text)
	}
}

impl Puzzle for Map {
	fn part_1(&mut self) -> eyre::Result<i64> {
		let mut pos = Coord2D::default();
		self.grid.insert(pos, 1);
		for step in self.steps.iter().copied() {
			pos = step.step(pos);
			self.grid.update_default(pos, |ct| *ct += 1);
		}
		Ok(self.grid.iter().filter(|(_, &ct)| ct >= 1).count() as i64)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.grid.clear();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let (mut one, mut two) = (Coord2D::default(), Coord2D::default());
		self.grid.insert(one, 2);
		for (step, alt) in self
			.steps
			.iter()
			.copied()
			.zip([true, false].into_iter().cycle())
		{
			let cur = if alt { &mut one } else { &mut two };
			*cur = step.step(*cur);
			self.grid.update_default(*cur, |ct| *ct += 1);
		}
		Ok(self.grid.iter().filter(|(_, &ct)| ct >= 1).count() as i64)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Step {
	North,
	South,
	East,
	West,
}

impl Step {
	pub fn step(self, point: Coord2D<i32>) -> Coord2D<i32> {
		match self {
			Self::North => Coord2D {
				y: point.y + 1,
				..point
			},
			Self::South => Coord2D {
				y: point.y - 1,
				..point
			},
			Self::East => Coord2D {
				x: point.x + 1,
				..point
			},
			Self::West => Coord2D {
				x: point.x - 1,
				..point
			},
		}
	}
}

impl<'a> Parsed<&'a str> for Step {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::North, tag("^")),
			value(Self::South, tag("v")),
			value(Self::East, tag(">")),
			value(Self::West, tag("<")),
		))(text)
	}
}
