use std::{
	ops::Index,
	sync::atomic::Ordering,
};

use im::OrdSet;
use radium::{
	Atom,
	Radium,
};
use rayon::Scope;

use crate::{
	coords::{
		points::{
			Cartesian2D as Point2D,
			Direction2D,
		},
		spaces::{
			Dense2D,
			DisplayGrid,
		},
	},
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 17, |t| t.parse_dyn_puzzle::<Coldtown>());

const RUN: u8 = 2;

const ORDER: Ordering = Ordering::SeqCst;

#[derive(Debug, Default)]
pub struct Coldtown {
	grid:       Dense2D<i16, Tile>,
	best_score: Atom<i64>,
	start:      Point2D<i16>,
	end:        Point2D<i16>,
}

impl<'a> Parsed<&'a str> for Coldtown {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let grid = Dense2D::from_raw(
			Point2D::ZERO,
			text.lines()
				.map(|line| {
					line.chars().map(|c| Tile::new(c as u8 - b'0')).collect()
				})
				.collect(),
		);
		let Some((start, end)) = grid.dimensions()
		else {
			tracing::error!("cannot process an empty grid");
			return Err(nom::Err::Failure(nom::error::Error::new(
				text,
				nom::error::ErrorKind::Digit,
			)));
		};
		Ok(("", Self {
			grid,
			best_score: Atom::new(i64::MAX),
			start,
			end,
		}))
	}
}

impl Puzzle for Coldtown {
	fn part_1(&mut self) -> eyre::Result<i64> {
		rayon::scope(|s| {
			s.spawn(|s| {
				Pathfinder::new(&*self, self.start, Direction2D::East).search(s);
			});
			Pathfinder::new(&*self, self.start, Direction2D::South).search(s);
		});
		return Ok(self.best_score.load(ORDER));
	}
}

#[derive(Clone, Debug)]
pub struct Pathfinder<'a> {
	past:    OrdSet<Point2D<i16>>,
	town:    &'a Coldtown,
	score:   i64,
	pos:     Point2D<i16>,
	dir:     Direction2D,
	to_turn: u8,
}

impl<'a> Pathfinder<'a> {
	pub fn new(town: &'a Coldtown, pos: Point2D<i16>, dir: Direction2D) -> Self {
		let mut out = Self {
			town,
			pos,
			dir,
			past: OrdSet::new(),
			to_turn: RUN,
			score: 0,
		};
		out.past.insert(pos);
		out
	}

	pub fn search<'s>(mut self, scope: &Scope<'s>)
	where 'a: 's {
		loop {
			let best = self.town.best_score.load(ORDER);
			// If we are not better than the current best path, quit.
			if self.score >= best {
				// tracing::debug!(%self.pos, %self.score, %best, "quit");
				break;
			}
			// If we are not the cheapest path to get to this tile in this
			// direction, quit.
			if self.town.grid[self.pos][self.dir].fetch_min(self.score, ORDER)
				< self.score
			{
				break;
			}
			if self.pos == self.town.end {
				if self.town.best_score.fetch_min(self.score, ORDER) > self.score
				{
					tracing::debug!("\n{:#}", self.display());
					tracing::info!(%self.score, "new winner");
					break;
				}
			}
			let turns = match self.dir {
				Direction2D::North | Direction2D::South => {
					[Direction2D::East, Direction2D::West]
				},
				Direction2D::West | Direction2D::East => {
					[Direction2D::South, Direction2D::North]
				},
			}
			.map(|d| (d, RUN));
			let ahead = self.to_turn.checked_sub(1).map(|t| (self.dir, t));
			let mut steps = ahead
				.into_iter()
				.chain(turns)
				.map(|(d, t)| (self.pos + d.unit(), d, t))
				.filter(|&(pt, ..)| self.town.grid.in_bounds(pt))
				.filter_map(|(pt, dir, to_turn)| {
					let mut this = self.clone();
					// Move to the next point
					this.pos = pt;
					// Set the direction of travel.
					this.dir = dir;
					// And the duration.
					this.to_turn = to_turn;
					// Accumulate the new tile's cost into the current score.
					this.score += self.town.grid[this.pos].cost as i64;
					if this.past.insert(pt).is_none() {
						Some(this)
					}
					else {
						None
					}
				});
			let Some(mine) = steps.next()
			else {
				break;
			};
			for other in steps {
				scope.spawn(move |s| other.search(s));
			}
			self = mine;
		}
	}
}

impl DisplayGrid<i16, u8> for Pathfinder<'_> {
	fn bounds_inclusive(&self) -> Option<(Point2D<i16>, Point2D<i16>)> {
		self.town.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i16,
		col: i16,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		let pt = Point2D::new(col, row);
		if self.past.contains(&pt) {
			symbols.full
		}
		else {
			(self.town.grid[pt].cost + b'0') as char
		}
	}
}

#[derive(Debug, Default)]
pub struct Tile {
	cost: u8,
	n:    Atom<i64>,
	s:    Atom<i64>,
	w:    Atom<i64>,
	e:    Atom<i64>,
}

impl Tile {
	pub fn new(cost: u8) -> Self {
		Self {
			cost,
			n: Atom::new(i64::MAX),
			s: Atom::new(i64::MAX),
			w: Atom::new(i64::MAX),
			e: Atom::new(i64::MAX),
		}
	}
}

impl Clone for Tile {
	fn clone(&self) -> Self {
		Self {
			cost: self.cost,
			n:    Atom::new(self.n.load(ORDER)),
			s:    Atom::new(self.s.load(ORDER)),
			w:    Atom::new(self.w.load(ORDER)),
			e:    Atom::new(self.e.load(ORDER)),
		}
	}
}

impl Index<Direction2D> for Tile {
	type Output = Atom<i64>;

	fn index(&self, dir: Direction2D) -> &Self::Output {
		match dir {
			Direction2D::North => &self.n,
			Direction2D::South => &self.s,
			Direction2D::West => &self.w,
			Direction2D::East => &self.e,
		}
	}
}
