#![doc = include_str!("spaces.md")]

use std::{
	fmt::{
		self,
		Write,
	},
	marker::PhantomData,
};

use funty::Signed;

pub mod dense;
pub mod sparse;

pub use self::{
	dense::Cartesian2D as Dense2D,
	sparse::Cartesian2D as Sparse2D,
};
pub use super::Cartesian2DPoint as Point2D;

/// A collection of symbols relevant for rendering a table to plaintext.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Symbols {
	/// A solid horizontal line.
	pub horiz:      char,
	/// A solid vertical line.
	pub vert:       char,
	/// A intersection between horizontal and vertical lines.
	pub cross:      char,
	/// A diagonal line from lower-left to upper-right.
	pub swne:       char,
	/// A dot in the center of a tile.
	pub middle_dot: char,
	/// A broken horizontal line.
	pub horiz_dots: char,
	/// A broken vertical line.
	pub vert_dots:  char,
	/// An empty tile.
	pub empty:      char,
	/// Indicates that the tile is one-quarter full.
	pub quarter_1:  char,
	/// Indicates that the tile is one-half full.
	pub quarter_2:  char,
	/// Indicatest hat the tile is three-quarters full.
	pub quarter_3:  char,
	/// A completely filled tile.
	pub full:       char,
}

impl Symbols {
	/// ASCII-only drawing symbols.
	pub const ASCII: Self = Self {
		horiz:      '-',
		vert:       '|',
		cross:      '+',
		swne:       '/',
		middle_dot: '.',
		horiz_dots: '-',
		vert_dots:  '|',
		empty:      ' ',
		quarter_1:  '_',
		quarter_2:  'm',
		quarter_3:  'M',
		full:       '#',
	};
	/// Symbols taken from the box-drawing and and geometric shapes Unicode
	/// blocks.
	pub const FANCY: Self = Self {
		horiz:      '─',
		vert:       '│',
		cross:      '┼',
		swne:       '╱',
		middle_dot: '·',
		horiz_dots: '╌',
		vert_dots:  '╎',
		empty:      ' ',
		quarter_1:  '░',
		quarter_2:  '▒',
		quarter_3:  '▓',
		full:       '█',
	};
}

pub trait DisplayGrid<I: Signed, T> {
	/// The implementor must provide a size hint for the renderer.
	fn bounds_inclusive(&self) -> Option<(Point2D<I>, Point2D<I>)>;

	/// The implementor must yield a single character for each cell in the grid.
	///
	/// The trait provides a set of suggested symbols out of the box, and
	/// implementors can use these or provide their own as they see fit.
	///
	/// The *actual* unit of display is not `char`, it is the terminal cell, but
	/// this is a close-enough approximation since I don't actually want to
	/// write a terminal renderer ... yet.
	fn print_cell(
		&self,
		symbols: &Symbols,
		row: I,
		col: I,
		row_abs: usize,
		col_abs: usize,
	) -> char;

	fn render(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let Some((min, max)) = self.bounds_inclusive()
		else {
			return Ok(());
		};
		let symbols = if fmt.alternate() {
			Symbols::FANCY
		}
		else {
			Symbols::ASCII
		};
		// We print from 0 to len, not from min to max, regardless of coördinate
		// domain.
		let (row_max_abs, col_max_abs) =
			((max.y - min.y).as_usize(), (max.x - min.x).as_usize());
		// Compute the number of digits needed to display the largest ordinate.
		let num_hex_digits =
			(row_max_abs.max(col_max_abs) as f64).log(16.0).ceil() as usize;
		let mut display_cols = vec![String::new(); num_hex_digits];
		let display_cols = display_cols.as_mut_slice();
		// For every column ordinate in the table,
		for col in 0 ..= col_max_abs {
			let mut mask = 0xF;
			// Write the least digit into the first line,
			write!(display_cols[0], "{:x}", col & mask)?;
			// Then for all other digits,
			for place in 1 .. num_hex_digits {
				// Get that place's line,
				let line = &mut display_cols[place];
				// If the column number is an even interval of the current
				// place, write the place digit,
				if col & mask == 0 {
					write!(line, "{:x}", (col >> (4 * place)) & 0xF)?;
				}
				// Otherwise, write a light filler to prevent line saturation.
				else {
					line.write_char(symbols.horiz_dots)?;
				}
				// Expand the insignificance mask.
				mask <<= 4;
				mask |= 0xF;
			}
		}
		for (line, _place) in display_cols.iter().zip(0 .. num_hex_digits).rev()
		{
			// TODO(myrrlyn): eventually figure out how to render a diagonal
			writeln!(
				fmt,
				"{fill: <pfx$}{sep}{line}",
				fill = "",
				pfx = num_hex_digits,
				sep = symbols.vert
			)?;
		}
		for _ in 0 .. num_hex_digits {
			fmt.write_char(symbols.horiz)?;
		}
		fmt.write_char(symbols.cross)?;
		for _ in 0 ..= col_max_abs {
			fmt.write_char(symbols.horiz)?;
		}
		writeln!(fmt)?;

		// Loop through each row, printing a row header and then allowing the
		// implementor to print the table contents.
		let mut row = min.y;
		for row_abs in 0 ..= row_max_abs {
			let mut mask = 0xF;
			let mut row_chars = format!("{:x}", row_abs & 0xF);
			for place in 1 .. num_hex_digits {
				mask |= 0xF;
				if row_abs & mask == 0 {
					write!(
						&mut row_chars,
						"{:x}",
						(row_abs >> (4 * place)) & 0xF
					)?;
				}
				else {
					row_chars.write_char(symbols.vert_dots)?;
				}
				mask <<= 4;
			}
			for char in row_chars.chars().rev() {
				fmt.write_char(char)?;
			}
			fmt.write_char(symbols.vert)?;
			let mut col = min.x;
			for col_abs in 0 ..= col_max_abs {
				fmt.write_char(
					self.print_cell(&symbols, row, col, row_abs, col_abs),
				)?;
				col += I::ONE;
			}
			writeln!(fmt)?;
			row += I::ONE;
		}
		Ok(())
	}

	fn display<'a>(&'a self) -> GridPrinter<'a, Self, I, T> {
		GridPrinter {
			inner: self,
			_grid: PhantomData,
		}
	}
}

pub struct GridPrinter<'a, G: 'a + ?Sized + DisplayGrid<I, T>, I: Signed, T> {
	inner: &'a G,
	_grid: PhantomData<Dense2D<I, T>>,
}

impl<'a, G: 'a + DisplayGrid<I, T>, I: Signed, T> fmt::Display
	for GridPrinter<'a, G, I, T>
{
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.inner.render(fmt)
	}
}

#[cfg(test)]
mod tests {
	use tap::Pipe;

	use super::*;

	#[test]
	fn render_table() {
		const MAX_COL: usize = 150;
		const MAX_ROW: usize = 30;
		let data = (0 .. MAX_ROW)
			.map(|_| (0 .. MAX_COL).map(|_| ()).collect::<Vec<_>>())
			.collect::<Vec<_>>()
			.pipe(|v| Dense2D::<i16, ()>::from_raw(Point2D::ZERO, v))
			.pipe(|inner| Demo { inner });

		struct Demo {
			inner: Dense2D<i16, ()>,
		}
		impl DisplayGrid<i16, ()> for Demo {
			fn bounds_inclusive(&self) -> Option<(Point2D<i16>, Point2D<i16>)> {
				self.inner.dimensions()
			}

			fn print_cell(
				&self,
				symbols: &Symbols,
				_row: i16,
				_col: i16,
				row_abs: usize,
				col_abs: usize,
			) -> char {
				if col_abs % (row_abs + 1) == 0 {
					symbols.full
				}
				else {
					symbols.empty
				}
			}
		}
		impl fmt::Display for Demo {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				DisplayGrid::render(self, fmt)
			}
		}
		eprintln!("\n{data}");
	}
}
