use std::{
	collections::BTreeSet,
	fmt,
	mem,
	time::{
		Duration,
		SystemTime,
	},
};

use tap::{
	Pipe,
	Tap,
};
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
		// tracing::debug!!("\n{self:#}");
		Ok(())
	}

	pub fn applied_load(&self) -> eyre::Result<i64> {
		let (min, max) = self
			.table
			.dimensions()
			.ok_or_else(|| eyre::eyre!("cannot process an empty table"))?;
		let max = max - min;
		// tracing::debug!(%max);
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
		.pipe(|table| {
			Ok(("", Self { table }.tap(|t| tracing::debug!("\n{t:#}"))))
		})
	}
}

impl Puzzle for Tilting {
	/// Tilt the table so that the rocks slide north as far as they'll go.
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.tilt(Direction2D::North)?;
		tracing::debug!("\n{self:#}");
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
		tracing::debug!("cycle 1\n{self:#}");
		// let mut cycles = BTreeSet::new();
		let mut now = SystemTime::now();
		let mut print = now + Duration::from_secs(1);
		let mut prev = 0;
		let mut old = self.clone();
		for n in 1 .. CYCLES {
			self.tilt(Direction2D::North)?;
			self.tilt(Direction2D::West)?;
			self.tilt(Direction2D::South)?;
			self.tilt(Direction2D::East)?;
			if *self == old {
				tracing::debug!(%n, "became circular\n{self:#}");
				break;
			}
			old.clone_from(self);
			now = SystemTime::now();
			if now > print {
				tracing::debug!(%n, cycles=%(n - prev), "spinning");
				print = now + Duration::from_secs(1);
				prev = n;
			}
			// if cycles.len() > 100 {
			// 	cycles.clear();
			// }
			// if !cycles.insert(
			// 	self.table
			// 		.iter()
			// 		.filter(|&(_, &r)| r == Rock::Sphere)
			// 		.map(|(p, _)| p)
			// 		.collect::<BTreeSet<_>>(),
			// ) {
			// 	tracing::debug!(%n, "became circular\n{self:#}");
			// 	break;
			// }
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
