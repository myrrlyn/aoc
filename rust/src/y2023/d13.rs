use std::{
	fmt,
	iter::Sum,
	ops::Not,
};

use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		take_until1,
	},
	character::complete::newline,
	combinator::{
		map,
		value,
	},
	multi::separated_list1,
	sequence::terminated,
};
use tap::Pipe;

use crate::{
	coords::spaces::DisplayGrid,
	prelude::*,
	Coord2D,
	Grid2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 13, |t| t.parse_dyn_puzzle::<Mirrors>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mirrors {
	patterns: Vec<Pattern>,
}

impl<'a> Parsed<&'a str> for Mirrors {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(newline, Pattern::parse_wyz), |patterns| {
			Self { patterns }
		})(text)
	}
}

impl Puzzle for Mirrors {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		// for pat in &self.patterns {
		// 	println!("{pat}");
		// }
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.patterns
			.iter()
			.flat_map(|pat| pat.find_reflection(None))
			.sum::<i64>()
			.pipe(Ok)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.patterns
			.iter_mut()
			.flat_map(|pat| -> Option<Reflection> {
				let orig = pat.find_reflection(None)?;
				let (min, max) = pat.grid.dimensions()?;
				// Note: it takes two points to make a reflection, and one
				// smudge has two candidate points for inversion. The found
				// points don't necessarily match the sample data, but that's
				// okay!
				for row in min.y ..= max.y {
					for col in min.x ..= max.x {
						let coord = Coord2D::new(col, row);
						pat.grid.update_default(coord, |pt| *pt = !*pt);
						if let Some(fixed) = pat.find_reflection(Some(orig)) {
							if fixed != orig {
								tracing::debug!(%coord, "smudge fixed");
								return Some(fixed);
							}
						}
						pat.grid.update_default(coord, |pt| *pt = !*pt);
					}
				}
				tracing::warn!(%pat, "never fixed");
				None
			})
			.sum::<i64>()
			.pipe(Ok)
	}
}

/// Indicates an axis of symmetry in a `Pattern`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reflection {
	/// The `Pattern` had a horizontal axis of symmetry. Contains the number of
	/// rows above the line.
	Horizontal(i64),
	/// The `Pattern` had a vertical axis of symmetry. Contains the number of
	/// rows to the left of the line.
	Vertical(i64),
}

impl Sum<Reflection> for i64 {
	fn sum<I>(iter: I) -> Self
	where I: Iterator<Item = Reflection> {
		iter.fold(0, |acc, val| {
			acc + match val {
				Reflection::Horizontal(ct) => ct * 100,
				Reflection::Vertical(ct) => ct,
			}
		})
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pattern {
	grid: Grid2D<i8, Tile>,
}

impl Pattern {
	pub fn find_reflection(
		&self,
		ignore: impl Into<Option<Reflection>>,
	) -> Option<Reflection> {
		let ignore = ignore.into();
		let (min, max) = self.grid.dimensions()?;
		// Scan through each row junction looking for a horizontal axis of
		// vertical symmetry.
		'scan_row: for row in (min.y + 1) ..= max.y {
			tracing::trace!(%row, "scanning");
			// Terminology: graphs have positive-Y pointing down. These names
			// refer to the TEXTUAL direction, so "above" is lesser than
			// "below".
			let (mut above, mut below) = (row - 1, row);
			// Beginning with the two rows on either side of the divisor, and
			// expanding outwards, scan each row-pair for difference.
			while above >= min.y && below <= max.y {
				for ((_, a), (_, b)) in
					self.grid.row(above).zip(self.grid.row(below))
				{
					if a != b {
						tracing::trace!(%above, %below, "symmetry broken");
						continue 'scan_row;
					}
				}
				tracing::trace!(%above, %below, "symmetrical row-pair");
				above -= 1;
				below += 1;
			}
			// At loop exit, no divergence has been detected.
			let horiz = Reflection::Horizontal(row as i64);
			// If the found reflection matches the ignore pattern, skip it.
			if let Some(ignore) = ignore {
				if ignore == horiz {
					tracing::trace!(?horiz, "ignoring");
					continue;
				}
			}
			tracing::debug!(upper=%(row-1), lower=%row, "symmetrical rows");
			return Some(horiz);
		}
		// Scan through each column junction looking for a vertical axis of
		// horizontal symmmetry.
		'scan_col: for col in (min.x + 1) ..= max.x {
			tracing::trace!(%col, "scanning");
			let (mut left, mut right) = (col - 1, col);
			while left >= min.x && right <= max.x {
				for ((_, a), (_, b)) in
					self.grid.column(left).zip(self.grid.column(right))
				{
					if a != b {
						tracing::trace!(%left, %right, "symmetry broken");
						continue 'scan_col;
					}
				}
				tracing::trace!(%left, %right, "symmetrical col-pair");
				left -= 1;
				right += 1;
			}
			let vert = Reflection::Vertical(col as i64);
			if let Some(ignore) = ignore {
				if ignore == vert {
					tracing::trace!(?vert, "ignoring");
					continue;
				}
			}
			tracing::debug!(left=%(col-1), right=%col, "symmetrical cols");
			return Some(vert);
		}
		// tracing::warn!(%self, "found no symmetry");
		None
	}
}

impl<'a> Parsed<&'a str> for Pattern {
	// Strictly speaking, this is not a correct parser, since it discards the
	// right side of a line if an intermediate error occurs, but we know that
	// the input is not adversarial to the *parser*.
	fn parse_wyz(mut text: &'a str) -> ParseResult<&'a str, Self> {
		let mut this = Self::default();
		let mut row = 0;
		loop {
			// Since we are not bubbling the error, Rust won't infer the error
			// type of the parser without an explicit bind.
			let maybe_line: ParseResult<&'a str, &'a str> =
				terminated(take_until1("\n"), newline)(text);
			let Ok((rest, mut line)) = maybe_line
			else {
				break;
			};
			let mut col = 0;
			while let Ok((rest, tile)) = line.parse_wyz::<Tile>() {
				this.grid.insert(crate::Coord2D::new(col, row), tile);
				col += 1;
				line = rest;
			}
			row += 1;
			text = rest;
		}
		Ok((text, this))
	}
}

impl DisplayGrid<i8, Tile> for Pattern {
	fn bounds_inclusive(&self) -> Option<(Coord2D<i8>, Coord2D<i8>)> {
		self.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i8,
		col: i8,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		match self.grid.get(Coord2D::new(col, row)) {
			Some(Tile::Ash) => symbols.middle_dot,
			Some(Tile::Rock) => symbols.full,
			None => symbols.empty,
		}
	}
}

impl fmt::Display for Pattern {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		writeln!(fmt)?;
		DisplayGrid::render(self, fmt)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tile {
	#[default]
	Ash,
	Rock,
}

impl Not for Tile {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::Ash => Self::Rock,
			Self::Rock => Self::Ash,
		}
	}
}

impl<'a> Parsed<&'a str> for Tile {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((value(Self::Ash, tag(".")), value(Self::Rock, tag("#"))))(text)
	}
}
