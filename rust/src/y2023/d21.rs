use std::{ops::Index, sync::atomic::Ordering};

use radium::{Atom, Radium};
use rayon::Scope;

use crate::{
	coords::{spaces::DisplayGrid, Dense2DSpace},
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 21, |t| t.parse_dyn_puzzle::<Garden>());

type Ordinate = i32;

#[derive(Debug, Default)]
pub struct Garden {
	grid: Dense2DSpace<Ordinate, Tile>,
}

impl<'a> Parsed<&'a str> for Garden {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut origin = Coord2D::ZERO;
		let grid = text
			.lines()
			.zip(0..)
			.map(|(l, r)| {
				l.chars()
					.zip(0..)
					.map(|(t, c)| {
						let t = Tile::new(t.into());
						if t.kind == Kind::Start {
							origin = Coord2D::new(-c, -r);
						}
						t
					})
					.collect()
			})
			.collect();
		let grid = Dense2DSpace::from_raw(origin, grid);
		Ok(("", Self { grid }))
	}
}

impl Puzzle for Garden {
	fn part_1(&mut self) -> eyre::Result<i64> {
		const STEP_COUNT: i32 = 64;
		for (_, tile) in self
			.grid
			.iter()
			.filter(|(pt, _)| (1 ..= STEP_COUNT).contains(&pt.abs_manhattan()))
			.filter(|(_, tile)| tile.kind != Kind::Stone)
			// .filter(|(pt, _)| 64 % pt.abs_manhattan() == 0)
			.filter(|(pt, _)| pt.abs_manhattan() % 2 == STEP_COUNT % 2)
		{
			tile.visited.store(true, Ordering::Relaxed);
		}
		tracing::debug!("\n{:#}", self.display());
		Ok(self
			.grid
			.iter()
			.filter(|&(_, t)| t.visited.load(Ordering::Relaxed))
			.count() as i64)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		const STEP_COUNT: i32 = 26_501_365;
		todo!()
	}
}

impl Index<Coord2D<Ordinate>> for Garden {
	type Output = Tile;

	fn index(&self, mut coord: Coord2D<Ordinate>) -> &Self::Output {
		let Some((min, max)) = self.grid.dimensions() else {
			panic!("cannot index an empty grid");
		};
		if min == max {
			return &self.grid[min];
		}
		let size = max - min;
		// Adjust each dimension to count from the minimum
		coord.x -= min.x;
		coord.x %= size.x;
		coord.x += min.x;
		coord.y -= min.y;
		coord.y %= min.y;
		coord.y += min.y;
		&self.grid[coord]
	}
}

impl DisplayGrid<Ordinate, Tile> for Garden {
	fn bounds_inclusive(
		&self,
	) -> Option<(Coord2D<Ordinate>, Coord2D<Ordinate>)> {
		self.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: Ordinate,
		col: Ordinate,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		let pt = Coord2D::new(col, row);
		let tile = &self.grid[pt];
		if tile.visited.load(Ordering::Relaxed) {
			symbols.middle_dot
		} else {
			match tile.kind {
				Kind::Empty => symbols.empty,
				Kind::Stone => symbols.full,
				Kind::Start => 'S',
			}
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Walker<'a> {
	plot: &'a Garden,
	curr: Coord2D<Ordinate>,
	dist: u8,
	walked: u8,
}

impl<'a> Walker<'a> {
	pub fn new(plot: &'a Garden, start: Coord2D<Ordinate>, dist: u8) -> Self {
		Self {
			plot,
			curr: start,
			dist,
			walked: 0,
		}
	}

	pub fn search<'s>(mut self, scope: &Scope<'s>)
	where
		'a: 's,
	{
		while self.dist != 0 {
			if self.walked % 2 == 0 {
				self.plot.grid[self.curr]
					.visited
					.store(true, Ordering::Relaxed);
			}
			let mut next = self
				.curr
				.direct_neighbors()
				.into_iter()
				.filter(|&pt| self.plot.grid.in_bounds(pt))
				.filter(|&pt| self.plot.grid[pt].kind != Kind::Stone)
				.filter(|&pt| pt.abs_manhattan() > self.curr.abs_manhattan())
				.map(|pt| {
					let mut this = self.clone();
					this.curr = pt;
					this.dist -= 1;
					this.walked += 1;
					this
				});
			let Some(mine) = next.next() else {
				break;
			};
			for step in next {
				scope.spawn(move |s| step.search(s));
			}
			self = mine;
		}
		self.plot.grid[self.curr]
			.visited
			.store(true, Ordering::Relaxed);
	}
}

#[derive(Debug)]
pub struct Tile {
	kind: Kind,
	visited: Atom<bool>,
}

impl Tile {
	pub fn new(kind: Kind) -> Self {
		Self {
			kind,
			visited: Atom::new(false),
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Kind {
	#[default]
	Empty,
	Stone,
	Start,
}

impl From<char> for Kind {
	fn from(c: char) -> Self {
		match c {
			'.' => Self::Empty,
			'#' => Self::Stone,
			'S' => Self::Start,
			c => {
				tracing::warn!(%c, "unknown character, treating as void");
				Self::Empty
			},
		}
	}
}
