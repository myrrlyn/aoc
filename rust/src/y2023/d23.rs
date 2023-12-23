#![doc = include_str!("d23.md")]

use std::sync::atomic::Ordering;

use im::{
	OrdSet,
	Vector,
};
use radium::{
	Atom,
	Radium,
};
use rayon::Scope;
use tap::{
	Conv,
	TapOptional,
};

use crate::{
	coords::{
		points::Direction2D,
		spaces::DisplayGrid,
		Dense2DSpace,
	},
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 23, |t| t.parse_dyn_puzzle::<Trails>());

#[derive(Clone, Debug, Default)]
pub struct Trails {
	topo: Dense2DSpace<i16, Tile>,
	bgn:  Coord2D<i16>,
	end:  Coord2D<i16>,
}

impl<'a> Parsed<&'a str> for Trails {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let topo = Dense2DSpace::from_raw(
			Coord2D::ZERO,
			text.lines()
				.map(|l| {
					l.chars().map(|c| c.conv::<Kind>().conv::<Tile>()).collect()
				})
				.collect(),
		);
		let Some((bgn, end)) = (|| -> Option<(_, _)> {
			let (mut min, mut max) = topo.dimensions()?;
			min.x = topo
				.raw_data()
				.first()
				.tap_none(|| tracing::error!("cannot process empty map"))?
				.iter()
				.zip(0 ..)
				.find(|(t, _)| t.kind == Kind::Path)
				.map(|(_, i)| i)
				.tap_none(|| {
					tracing::error!("no path tile found on first line");
				})?;
			max.x = topo
				.raw_data()
				.last()?
				.iter()
				.zip(0 ..)
				.find(|(t, _)| t.kind == Kind::Path)
				.map(|(_, i)| i)
				.tap_none(|| {
					tracing::error!("no path tile found on last line");
				})?;
			Some((min, max))
		})()
		else {
			return Err(nom::Err::Failure(nom::error::Error::new(
				text,
				nom::error::ErrorKind::Char,
			)));
		};
		Ok(("", Self { topo, bgn, end }))
	}
}

impl Puzzle for Trails {
	fn part_1(&mut self) -> eyre::Result<i64> {
		let score = Atom::new(0);
		rayon::scope(|s| Hiker::new(&self, false).search_par(s, &score));
		Ok(score.load(Ordering::Relaxed) as i64 - 1)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for (_, tile) in self.topo.iter() {
			tile.best.store(0, Ordering::Relaxed);
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let score = Atom::new(0);
		rayon::scope(|s| Hiker::new(&self, true).search_par(s, &score));
		Ok(score.load(Ordering::Relaxed) as i64 - 1)
	}
}

#[derive(Clone, Debug)]
pub struct Hiker<'a> {
	topo:    &'a Trails,
	walked:  OrdSet<Coord2D<i16>>,
	path:    Vector<Coord2D<i16>>,
	current: Coord2D<i16>,
	tough:   bool,
}

impl<'a> Hiker<'a> {
	pub fn new(topo: &'a Trails, tough: bool) -> Self {
		Self {
			topo,
			walked: OrdSet::new(),
			path: Vector::new(),
			current: topo.bgn,
			tough,
		}
	}

	/// Runs a BFS using a thread-pool for collecting additional workers.
	pub fn search_par<'s>(mut self, scope: &Scope<'s>, score: &'a Atom<i32>)
	where 'a: 's {
		loop {
			// Cannot leave the park.
			let Some(cell) = self.topo.topo.get(self.current)
			else {
				break;
			};
			// Cannot walk into the woods.
			if cell.kind == Kind::Forest {
				break;
			}
			// Re-visits quit immediately.
			if self.walked.insert(self.current).is_some() {
				break;
			}
			self.path.push_back(self.current);
			let len = self.path.len() as i32;
			// If we are not the longest route to get to this point, quit.
			// NOTE: this needs to be directional, same as last time!
			// if cell.best.load(Ordering::Relaxed) > len {
			// 	break;
			// }

			// Reaching the goal updates the best score.
			if self.current == self.topo.end {
				if score.fetch_max(len, Ordering::Relaxed) == len {
					tracing::debug!(score=%len, "new best\n{:#}", self.display());
					for (cell, step) in self.path.iter().copied().zip(0 ..) {
						self.topo.topo[cell].best.store(step, Ordering::Relaxed);
					}
					break;
				}
			}
			let next = match (cell.kind, self.tough) {
				(Kind::Forest, _) => {
					unreachable!("we don't walk into the woods")
				},
				(Kind::Slope(dir), false) => self.current + dir.unit(),
				// we can always walk a path, and we can walk a slope in any
				// direction when strong
				(Kind::Path, _) | (Kind::Slope(_), true) => {
					let mut pts = self
						.current
						.direct_neighbors()
						.into_iter()
						.filter(|&p| self.topo.topo.in_bounds(p))
						.filter(|&p| self.topo.topo[p].kind != Kind::Forest);
					let Some(mine) = pts.next()
					else {
						continue;
					};
					for pt in pts {
						let mut other = self.clone();
						other.current = pt;
						scope.spawn(move |s| other.search_par(s, score));
					}
					mine
				},
			};
			self.current = next;
		}
	}
}

impl DisplayGrid<i16, Kind> for Hiker<'_> {
	fn bounds_inclusive(&self) -> Option<(Coord2D<i16>, Coord2D<i16>)> {
		self.topo.topo.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i16,
		col: i16,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		let pt = Coord2D::new(col, row);
		if self.walked.contains(&pt) {
			symbols.middle_dot
		}
		else {
			match self.topo.topo[pt].kind {
				Kind::Forest => symbols.full,
				Kind::Path => '▢',
				Kind::Slope(Direction2D::North) => '△',
				Kind::Slope(Direction2D::South) => '▽',
				Kind::Slope(Direction2D::West) => '◁',
				Kind::Slope(Direction2D::East) => '▷',
			}
		}
	}
}

#[derive(Debug, Default)]
pub struct Tile {
	kind: Kind,
	best: Atom<i32>,
}

impl Clone for Tile {
	fn clone(&self) -> Self {
		Self {
			kind: self.kind,
			best: Atom::new(self.best.load(Ordering::Relaxed)),
		}
	}
}

impl From<Kind> for Tile {
	fn from(kind: Kind) -> Self {
		Self {
			kind,
			best: Atom::new(0),
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Kind {
	Path,
	#[default]
	Forest,
	Slope(Direction2D),
}

impl From<char> for Kind {
	fn from(c: char) -> Self {
		match c {
			'.' => Self::Path,
			'#' => Self::Forest,
			'^' => Self::Slope(Direction2D::North),
			'v' => Self::Slope(Direction2D::South),
			'<' => Self::Slope(Direction2D::West),
			'>' => Self::Slope(Direction2D::East),
			c => {
				tracing::warn!(%c, "unknown forest character");
				Self::Forest
			},
		}
	}
}
