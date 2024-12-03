//! Inflation
//!
//! Well this was certainly ... something.
//!
//! Honestly, the hardest part of today's puzzle was adding a Display impl for
//! the plane so that I could make sure I was correctly processing the map
//! expansion. It took me a while to get the padding logic nailed down (plus,
//! I'll admit, I wasted a lot of time making the axis markers pretty). Since I
//! fell asleep before the puzzle opened, and only started work after ~08:00 my
//! time, I wasn't racing anymore and figured I should take the time to make the
//! infrastructure nice.
//!
//! The expansion logic itself was fairly easy. The only note is to remember
//! that all the expansion slots in the map are *already* scaled by 1, so to
//! re-scale them to some other value, the multiplicand is `scale - 1`.

use std::{
	fmt,
	mem,
};

use tap::Pipe;

use crate::{
	prelude::*,
	Coord2D,
	Grid2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 11, |t| t.parse_dyn_puzzle::<Cosmos>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cosmos {
	map: Grid2D<i32, Galaxy>,
}

impl Cosmos {
	pub fn expand(&mut self, scalar: i32) {
		let Some((min, max)) = self.map.dimensions()
		else {
			tracing::warn!("cannot expand an empty star-map");
			return;
		};
		let empty_rows = ((min.y) ..= (max.y))
			.into_iter()
			.filter(|row| !self.map.raw_data().contains_key(row))
			.collect::<Vec<_>>();
		let empty_cols = ((min.x) ..= (max.x))
			.into_iter()
			.filter(|col| {
				self.map
					.raw_data()
					.values()
					.all(|row| !row.contains_key(col))
			})
			.collect::<Vec<_>>();
		self.map = mem::take(&mut self.map)
			.into_iter()
			.map(|(Coord2D { x, y }, galaxy)| {
				// Move the galaxies south-east for every empty column or row
				// that is north-west of them.
				let coord = Coord2D::new(
					x + ((empty_cols.iter().filter(|&&col| col < x).count()
						as i32) * (scalar - 1)),
					y + ((empty_rows.iter().filter(|&&row| row < y).count()
						as i32) * (scalar - 1)),
				);
				(coord, galaxy)
			})
			.collect();
	}

	fn distances(&self) -> i64 {
		let mut accum = 0;
		let mut walker = self.map.iter().map(|(c, _)| c);
		while let Some(this) = walker.next() {
			for that in walker.clone() {
				accum +=
					((this.x - that.x).abs() + (this.y - that.y).abs()) as i64;
			}
		}
		accum
	}
}

impl<'a> Parsed<&'a str> for Cosmos {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		text.lines()
			.enumerate()
			.map(|(row, line)| {
				line.char_indices().flat_map(move |(col, sym)| match sym {
					'#' => Some(Coord2D::new(col as i32, row as i32)),
					'.' => None,
					c => {
						tracing::warn!(?c, "unexpected character");
						None
					},
				})
			})
			.flatten()
			.map(|coord| (coord, Galaxy))
			.collect::<Grid2D<i32, Galaxy>>()
			.pipe(|map| Ok(("", Self { map })))
	}
}

impl Puzzle for Cosmos {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		// println!("{}", self);
		self.expand(2);
		// println!("{}", self);
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.distances().pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.expand(1_000_000 / 2); // factor out the prior expansion
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.distances().pipe(Ok)
	}
}

impl fmt::Display for Cosmos {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.map, fmt)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Galaxy;
