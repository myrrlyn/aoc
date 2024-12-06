use nom::{
	self,
	branch::alt,
	bytes::complete::tag,
	combinator::value,
};

use crate::{
	coords::{
		points::{
			Direction2D,
			DirectionSet2D,
		},
		spaces::DisplayGrid,
		Dense2DSpace,
	},
	prelude::*,
	Coord2D,
	Grid2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 6, |t| t.parse_dyn_puzzle::<Patrol>());

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Patrol {
	grid:      Dense2DSpace<i16, Square>,
	cursor:    Coord2D<i16>,
	direction: Direction2D,
}

impl Patrol {
	pub fn step_guard(&mut self) -> eyre::Result<()> {
		let this = &mut self.grid[self.cursor];
		if this.has_visited(self.direction) {
			eyre::bail!("cycle detected");
		}
		this.visit(self.direction);
		let next = self.cursor + self.direction.unit();
		// If the next coord is out of bounds, move the cursor to it, do not
		// update the board, and quit.
		if !self.grid.in_bounds(next) {
			self.cursor = next;
			return Ok(());
		}
		// Otherwise, if the next coord is in bounds and obstracted, change
		// directions but do not advance the cursor.
		if let Square::Obstructed = self.grid[next] {
			self.direction = self.direction.turn_right();
			return Ok(());
		}
		// If the next coord is in bounds and not obstructed, advance to it.
		self.cursor = next;
		Ok(())
	}
}

impl Puzzle for Patrol {
	fn after_parse(&mut self) -> eyre::Result<()> {
		self.grid.set_origin(Coord2D::ZERO - self.cursor);
		self.cursor = Coord2D::ZERO;
		Ok(())
	}

	fn prepare_1(&mut self) -> eyre::Result<()> {
		let begin = std::time::Instant::now();
		while self.grid.in_bounds(self.cursor) {
			if std::time::Instant::now() - begin
				> std::time::Duration::from_secs(5)
			{
				eyre::bail!("probably found a cycle in the guard walk");
			}
			self.step_guard()?;
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		println!("{:#}", self.display());
		Ok(self
			.grid
			.iter()
			.map(|(_, &s)| s)
			.filter(Square::is_visited)
			.count() as i64)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for (_, sq) in self.grid.iter_mut() {
			if sq.is_visited() {
				*sq = Square::Open;
			}
		}
		self.grid[Coord2D::ZERO] = Square::Guard(Direction2D::North);
		self.direction = Direction2D::North;
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut cycles = 0;
		while self.grid.in_bounds(self.cursor) {
			let next = self.cursor + self.direction.unit();
			let mut snapshot = self.clone();
			if !snapshot.grid.in_bounds(next) {
				break;
			}
			// A cell can only be obstructed if it has not yet been walked
			// through in any direction.
			if !snapshot.grid[next].is_visited() {
				snapshot.grid[next] = Square::Obstructed;
				while snapshot.grid.in_bounds(snapshot.cursor) {
					if snapshot.step_guard().is_err() {
						println!("{:#}", snapshot.display());
						cycles += 1;
						break;
					}
				}
			}
			self.step_guard()?;
		}
		Ok(cycles)
	}
}

impl DisplayGrid<i16, Square> for Patrol {
	fn bounds_inclusive(&self) -> Option<(Coord2D<i16>, Coord2D<i16>)> {
		self.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i16,
		col: i16,
		_: usize,
		_: usize,
	) -> char {
		match self.grid[Coord2D::new(col, row)] {
			Square::Open => symbols.empty,
			Square::Obstructed => symbols.full,
			Square::Visited(_) => symbols.cross,
			Square::Guard(Direction2D::North) => '△',
			Square::Guard(Direction2D::South) => '▽',
			Square::Guard(Direction2D::West) => '◁',
			Square::Guard(Direction2D::East) => '▷',
		}
	}
}

impl<'a> Parsed<&'a str> for Patrol {
	fn parse_wyz(mut src: &'a str) -> ParseResult<&'a str, Self> {
		let mut cursor = Coord2D::ZERO;
		let mut direction = Direction2D::North;
		let mut grid = Grid2D::new();
		let mut row = 0;
		while let Some((line, rest)) = src.split_once('\n') {
			for (col, c) in line.chars().enumerate() {
				let square = c.try_into().map_err(|_| {
					nom::Err::Failure(nom::error::Error::new(
						&src[col ..],
						nom::error::ErrorKind::Alt,
					))
				})?;
				let coord = Coord2D::new(col as i16, row);
				if let Square::Guard(dir) = square {
					cursor = coord;
					direction = dir;
				}
				grid.insert(coord, square);
			}
			row += 1;
			src = rest;
		}
		Ok(("", Self {
			grid: grid.into(),
			cursor,
			direction,
		}))
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Square {
	#[default]
	Open,
	Obstructed,
	Guard(Direction2D),
	Visited(DirectionSet2D),
}

impl Square {
	pub fn visit(&mut self, direction: Direction2D) {
		if let Self::Visited(dirs) = self {
			dirs.insert(direction);
		}
		else {
			*self = Self::Visited(DirectionSet2D::new() | direction);
		}
	}

	pub fn is_visited(&self) -> bool {
		if let Self::Visited(_) = self {
			true
		}
		else {
			false
		}
	}

	pub fn has_visited(&self, direction: Direction2D) -> bool {
		if let Self::Visited(dirs) = self {
			dirs.contains(direction)
		}
		else {
			false
		}
	}
}

impl TryFrom<char> for Square {
	type Error = eyre::Error;

	fn try_from(c: char) -> Result<Self, Self::Error> {
		match c {
			'.' => Ok(Self::Open),
			'#' => Ok(Self::Obstructed),
			'^' => Ok(Self::Guard(Direction2D::North)),
			'v' => Ok(Self::Guard(Direction2D::South)),
			'<' => Ok(Self::Guard(Direction2D::West)),
			'>' => Ok(Self::Guard(Direction2D::East)),
			'X' => Ok(Self::Visited(DirectionSet2D::new())),
			_ => eyre::bail!("invalid symbol: {c}"),
		}
	}
}

impl<'a> Parsed<&'a str> for Square {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Open, tag(".")),
			value(Self::Obstructed, tag("#")),
			value(Self::Guard(Direction2D::North), tag("^")),
			value(Self::Guard(Direction2D::South), tag("v")),
			value(Self::Guard(Direction2D::West), tag("<")),
			value(Self::Guard(Direction2D::East), tag(">")),
			value(Self::Visited(DirectionSet2D::new()), tag("X")),
		))(src)
	}
}
