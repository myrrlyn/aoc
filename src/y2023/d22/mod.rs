use std::{
	iter::FusedIterator,
	sync::Arc,
};

use nom::{
	bytes::complete::tag,
	character::complete::i16 as get_i16,
	combinator::map,
	sequence::{
		separated_pair,
		tuple,
	},
};
use tap::Tap;

use crate::{
	prelude::*,
	Coord3D,
	Grid3D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 22, |t| t.parse_dyn_puzzle::<Sandbox>());

type Ordinate = i16;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sandbox {
	grid: Grid3D<Ordinate, Arc<Brick>>,
}

impl<'a> Parsed<&'a str> for Sandbox {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut grid = Grid3D::new();
		for (line, id) in text.lines().zip(1 ..) {
			let (_, mut brick): (&str, Brick) = line.parse_wyz()?;
			brick.id = id;
			let arced = Arc::new(brick);
			for pt in brick.occupies() {
				grid.insert(pt, arced.clone());
			}
		}
		Ok(("", Self { grid }))
	}
}

impl Puzzle for Sandbox {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		loop {}
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Brick {
	id:  usize,
	bgn: Coord3D<Ordinate>,
	end: Coord3D<Ordinate>,
}

impl Brick {
	pub fn occupies(
		self,
	) -> impl Iterator<Item = Coord3D<Ordinate>>
	       + DoubleEndedIterator
	       //    + ExactSizeIterator
	       + FusedIterator {
		(self.bgn.z ..= self.end.z)
			.tap(|r| {
				if r.is_empty() {
					tracing::warn!(%self.bgn.z, %self.end.z, "improper range");
				}
			})
			.flat_map(move |z| {
				(self.bgn.y ..= self.end.y)
					.tap(|r| {
						if r.is_empty() {
							tracing::warn!(%self.bgn.y, %self.end.y, "improper range");
						}
					})
					.map(move |y| (y, z))
			})
			.flat_map(move |(y, z)| {
				(self.bgn.x ..= self.end.x)
					.tap(|r| {
						if r.is_empty() {
							tracing::warn!(%self.bgn.x, %self.end.x, "improper range");
						}
					})
					.map(move |x| Coord3D::new(x, y, z))
			})
	}
}

impl<'a> Parsed<&'a str> for Brick {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_pair(coord3d, tag("~"), coord3d), |(bgn, end)| {
			Self { id: 0, bgn, end }
		})(text)
	}
}

fn coord3d<'a>(text: &'a str) -> ParseResult<&'a str, Coord3D<Ordinate>> {
	map(
		tuple((get_i16, tag(","), get_i16, tag(","), get_i16)),
		|(x, _, y, _, z)| Coord3D { x, y, z },
	)(text)
}
