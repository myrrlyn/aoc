use std::fmt::{
	self,
	Write as _,
};

use bitflags::bitflags;
use nom::{
	character::complete::{
		newline,
		one_of,
	},
	combinator::map,
	multi::{
		many1,
		separated_list1,
	},
};
use tap::Pipe;

use crate::{
	coords::{
		points::Direction2D,
		spaces::{
			DisplayGrid,
			Symbols,
		},
		Dense2DSpace,
	},
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2024, 4, |t| t.parse_dyn_puzzle::<WordSearch>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WordSearch {
	contents: Dense2DSpace<i16, Cell>,
}

impl WordSearch {
}

impl Puzzle for WordSearch {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		let starts = self
			.contents
			.iter()
			.filter(|(_, c)| c.symbol == 'X')
			.map(|(p, _)| p)
			.collect::<Vec<_>>();
		for pos in starts {
			for main in Direction2D::all() {
				let alt = main.turn_right();

				let step_main = main.unit();
				let step_diag = step_main + alt.unit();

				let mut found_main = true;
				let mut found_diag = true;
				let mut pos_main = pos;
				let mut pos_diag = pos;
				for letter in ['M', 'A', 'S'] {
					pos_main += step_main;
					pos_diag += step_diag;

					if self
						.contents
						.get(pos_main)
						.copied()
						.unwrap_or_default()
						.symbol != letter
					{
						found_main = false;
					}
					if self
						.contents
						.get(pos_diag)
						.copied()
						.unwrap_or_default()
						.symbol != letter
					{
						found_diag = false;
					}
				}

				pos_main = pos;
				pos_diag = pos;
				for _ in 0 .. 4 {
					if found_main {
						if let Some(cell) = self.contents.get_mut(pos_main) {
							cell.in_word.mark(step_main);
						}
					}
					if found_diag {
						if let Some(cell) = self.contents.get_mut(pos_diag) {
							cell.in_word.mark(step_diag);
						}
					}

					pos_main += step_main;
					pos_diag += step_diag;
				}
			}
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.contents
			.iter()
			.filter(|(_, c)| c.symbol == 'X' && c.in_word != Directions::empty())
			.map(|(_, c)| c.in_word.bits().count_ones())
			.sum::<u32>()
			.pipe(|ct| Ok(ct as i64))
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for (_, c) in self.contents.iter_mut() {
			c.in_word = Directions::empty();
		}
		let centers = self
			.contents
			.iter()
			.filter(|(_, c)| c.symbol == 'A')
			.map(|(p, _)| p)
			.collect::<Vec<_>>();
		for pos in centers {
			for main in Direction2D::all() {
				let turn = main.turn_right();
				let step = main.unit() + turn.unit();
				let one = pos + step;
				let two = pos - step;

				if self.contents.get(one).copied().unwrap_or_default().symbol
					== 'M' && self
					.contents
					.get(two)
					.copied()
					.unwrap_or_default()
					.symbol == 'S'
				{
					if let Some(c) = self.contents.get_mut(pos) {
						c.in_word.mark(step);
					}
				}
			}
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.contents
			.iter()
			.filter(|(_, c)| {
				c.symbol == 'A'
					&& (c.in_word.contains(Directions::NE)
						|| c.in_word.contains(Directions::SW))
					&& (c.in_word.contains(Directions::NW)
						|| c.in_word.contains(Directions::SE))
			})
			.count()
			.pipe(|ct| Ok(ct as i64))
	}
}

impl DisplayGrid<i16, Cell> for WordSearch {
	fn bounds_inclusive(&self) -> Option<(Coord2D<i16>, Coord2D<i16>)> {
		self.contents.dimensions()
	}

	fn print_cell(
		&self,
		_: &Symbols,
		row: i16,
		col: i16,
		_: usize,
		_: usize,
	) -> char {
		self.contents[Coord2D::new(col, row)].into()
	}
}

impl<'a> Parsed<&'a str> for WordSearch {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(newline, many1(map(one_of("XMAS"), Cell::from))),
			|grid| WordSearch {
				contents: Dense2DSpace::from_raw(
					crate::Coord2D::new(0, 0),
					grid,
				),
			},
		)(src)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cell {
	pub symbol:  char,
	pub in_word: Directions,
}

impl From<char> for Cell {
	fn from(src: char) -> Self {
		Self {
			symbol:  src,
			in_word: Directions::empty(),
		}
	}
}

impl Into<char> for Cell {
	fn into(self) -> char {
		if self.in_word == Directions::empty() {
			' '
		}
		else {
			self.symbol
		}
	}
}

impl fmt::Display for Cell {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_char((*self).into())
	}
}

bitflags! {
	#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Directions: u8 {
		const N  = 1;
		const NE = 2;
		const E  = 4;
		const SE = 8;
		const S  = 16;
		const SW = 32;
		const W  = 64;
		const NW = 128;
	}
}

impl Directions {
	pub fn mark(&mut self, point: Coord2D<i16>) {
		match (point.y.signum(), point.x.signum()) {
			(-1, 0) => *self |= Self::N,
			(-1, 1) => *self |= Self::NE,
			(0, 1) => *self |= Self::E,
			(1, 1) => *self |= Self::SE,
			(1, 0) => *self |= Self::S,
			(1, -1) => *self |= Self::SW,
			(0, -1) => *self |= Self::W,
			(-1, -1) => *self |= Self::NW,
			_ => {},
		}
	}
}

impl Default for Directions {
	fn default() -> Self {
		Self::empty()
	}
}
