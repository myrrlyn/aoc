use std::{
	collections::BTreeSet,
	fmt,
	mem,
};

use tap::Pipe;
use wyz::BidiIterator;

use crate::{
	coords::{
		points::Direction2D,
		spaces::Dense2D,
	},
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 14, |t| t.parse_dyn_puzzle::<Tilting>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tilting {
	table: Dense2D<i16, Rock>,
}

impl Tilting {
	pub fn tilt(&mut self, direction: Direction2D) -> eyre::Result<()> {
		let (min, max) = self
			.table
			.dimensions()
			.ok_or_else(|| eyre::eyre!("cannot tilt an empty table"))?;
		let step = direction.unit();
		for row in ((min.y) ..= max.y).bidi(direction == Direction2D::South) {
			for col in (min.x ..= max.x).bidi(direction == Direction2D::East) {
				let current = Coord2D::new(col, row);
				let mut next = current + step;
				if self.table[current] != Rock::Sphere {
					continue;
				}
				while self.table.in_bounds(next) {
					if self.table[next] != Rock::Void {
						break;
					}
					next += step;
				}
				next -= step;
				self.table[next] = mem::take(&mut self.table[current]);
			}
		}
		Ok(())
	}

	pub fn applied_load(&self) -> eyre::Result<i64> {
		let (min, max) = self
			.table
			.dimensions()
			.ok_or_else(|| eyre::eyre!("cannot process an empty table"))?;
		let max = max - min;
		self.table
			.iter()
			.filter(|&(_, &rock)| rock == Rock::Sphere)
			.map(|(Coord2D { y, .. }, _)| (max.y - y + 1) as i64)
			.sum::<i64>()
			.pipe(Ok)
	}
}

impl<'a> Parsed<&'a str> for Tilting {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		Dense2D::from_raw(
			Coord2D::new(0, 0),
			text.lines()
				.map(|line| line.chars().map(Rock::from).collect())
				.collect(),
		)
		.pipe(|table| Ok(("", Self { table })))
	}
}

impl Puzzle for Tilting {
	/// Tilt the table so that the rocks slide north as far as they'll go.
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.tilt(Direction2D::North)?;
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.applied_load()
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		const CYCLES: usize = 1_000_000_000;
		self.tilt(Direction2D::West)?;
		self.tilt(Direction2D::South)?;
		self.tilt(Direction2D::East)?;
		let mut observed_patterns = BTreeSet::new();
		let mut cycle_found = false;
		let mut step_count = 1;
		while step_count < CYCLES {
			self.tilt(Direction2D::North)?;
			self.tilt(Direction2D::West)?;
			self.tilt(Direction2D::South)?;
			self.tilt(Direction2D::East)?;
			step_count += 1;
			// We need to be able to produce the pattern twice in the event of
			// a cache-clear.
			let pattern = || {
				self.table
					.iter()
					.filter(|&(_, &r)| r == Rock::Sphere)
					.map(|(p, _)| p)
					.collect::<Vec<_>>()
			};
			// A cycle occurs when a pattern is observed a second time.
			if !observed_patterns.insert(pattern()) {
				tracing::debug!("found a repeated observation!");
				// The first time a pattern repeats, we have *at least* a cycle,
				// plus a potential introductory path that got us here. Discard
				// all observed patterns, then insert the duplicate again.
				if !cycle_found {
					tracing::debug!(%step_count, "first such repetition; resetting tracker");
					cycle_found = true;
					observed_patterns.clear();
					observed_patterns.insert(pattern());
					continue;
				}
				// Now, we know that the observation has *only* cyclic patterns.
				tracing::debug!(%step_count, cycle_len=%observed_patterns.len(), "second such repetition; fast-forwarding");
				let jump = (CYCLES - step_count) / observed_patterns.len();
				step_count += jump * observed_patterns.len();
				tracing::debug!(%step_count, "fast-forwarded");
				// Discard the observations; there are fewer steps remaining
				// than the cycle length, so we know this branch will not enter
				// a third time.
				observed_patterns.clear();
			}
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.applied_load()
	}
}

impl fmt::Display for Tilting {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.table.render(fmt, |_, rock| match rock {
			Rock::Sphere => 'O', // '○',
			Rock::Cube => '#',   // '■',
			Rock::Void => ' ',
		})
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Rock {
	Sphere,
	Cube,
	#[default]
	Void,
}

impl From<char> for Rock {
	fn from(c: char) -> Self {
		match c {
			'O' => Self::Sphere,
			'#' => Self::Cube,
			'.' => Self::Void,
			c => {
				tracing::warn!(?c, "unknown void character");
				Self::Void
			},
		}
	}
}
