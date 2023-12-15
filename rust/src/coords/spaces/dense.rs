/*! Densely-filled 2-dimensional spaces

This module provides data-types which are specialized for finite spaces which
are densely packed. The current implementation requires that *all* elements in
the space boundaries are fully populated, and has no support for holes. If your
space requires holes, you must either use `Option` as the storage type, or
switch to the sparse structure.
*/

use std::{
	fmt::{
		self,
		Write as _,
	},
	iter::FusedIterator,
	ops::{
		Index,
		IndexMut,
	},
};

use funty::Signed;

use crate::coords::Cartesian2DPoint as Point;

/// A 2-dimensional Cartesian grid where all cells within the bounds are filled
/// with some value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian2D<I: Signed, T> {
	origin: Point<I>,
	table:  Vec<Vec<T>>,
}

impl<I: Signed, T> Cartesian2D<I, T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_raw(origin: Point<I>, table: Vec<Vec<T>>) -> Self {
		if let Some(len) = table.first().map(Vec::len) {
			assert!(
				table.iter().all(|v| v.len() == len),
				"Dense storage cannot be jagged"
			);
		}
		Self { origin, table }
	}

	pub fn raw_data(&self) -> &Vec<Vec<T>> {
		&self.table
	}

	pub fn clear(&mut self) {
		*self = Self::new();
	}

	pub fn is_empty(&self) -> bool {
		self.table.is_empty() || self.table.iter().all(|row| row.is_empty())
	}

	pub fn dimensions(&self) -> Option<(Point<I>, Point<I>)> {
		let rows = self.table.len().checked_sub(1)?;
		let cols = self.table.first()?.len().checked_sub(1)?;
		let min = self.origin;
		let max =
			Point::new(I::try_from(cols).ok()?, I::try_from(rows).ok()?) + min;
		Some((min, max))
	}

	pub fn in_bounds(&self, point: Point<I>) -> bool {
		if self.is_empty() {
			return false;
		};
		let shifted = point - self.origin;
		shifted.y.as_usize() < self.table.len()
			&& shifted.x.as_usize() < self.table[0].len()
	}

	pub fn iter(
		&self,
	) -> impl Iterator<Item = (Point<I>, &T)> + DoubleEndedIterator + FusedIterator
	where <I as TryFrom<usize>>::Error: fmt::Debug {
		let Point { y, x } = self.origin;
		let ys = self.table.len();
		let xs = self.table.first().map(|v| v.len()).unwrap_or_default();
		self.table
			.iter()
			.zip(Axis::new(y, ys))
			.map(move |(row, y)| {
				row.iter()
					.zip(Axis::new(x, xs))
					.map(move |(val, x)| (Point::new(x, y), val))
			})
			.flatten()
	}

	pub fn into_iter(
		self,
	) -> impl Iterator<Item = (Point<I>, T)> + DoubleEndedIterator + FusedIterator
	where <I as TryFrom<usize>>::Error: fmt::Debug {
		let Point { y, x } = self.origin;
		let ys = self.table.len();
		let xs = self.table.first().map(|v| v.len()).unwrap_or(0);
		self.table
			.into_iter()
			.zip(Axis::new(y, ys))
			.map(move |(row, y)| {
				row.into_iter()
					.zip(Axis::new(x, xs))
					.map(move |(val, x)| (Point::new(x, y), val))
			})
			.flatten()
	}
}

impl<I: Signed, T> Cartesian2D<I, T>
where <I as TryFrom<usize>>::Error: fmt::Debug
{
	pub fn render(
		&self,
		fmt: &mut fmt::Formatter,
		mut per_cell: impl FnMut(Point<I>, &T) -> char,
	) -> fmt::Result {
		if self.is_empty() {
			return Ok(());
		}
		// The axis-drawing characters are:
		// 0. horizontal bar
		// 1. vertical bar
		// 2. intersection
		// 3. SW-to-NE diagonal fill
		let drawings = if fmt.alternate() {
			['─', '│', '┼', '▟']
		}
		else {
			['-', '|', '+', '/']
		};
		let cols_width = self.table[0].len();
		let max_col = cols_width
			.checked_sub(1)
			.expect("cannot render a zero-column table");
		let max_row = self
			.table
			.len()
			.checked_sub(1)
			.expect("cannot render a zero-row table");

		if fmt.alternate()
			&& (self.origin.x != I::ZERO || self.origin.y != I::ZERO)
		{
			writeln!(
				fmt,
				"{:^w$}",
				&format!("Translated from {}", self.origin),
				w = cols_width,
			)?;
		}
		let mut places = [String::new(), String::new(), String::new()];
		for col in 0 ..= max_col {
			let h = (col / 256) % 16;
			let m = (col / 16) % 16;
			let l = col % 16;
			if col % 256 == 0 {
				if fmt.alternate() {
					write!(&mut places[0], "{h:─<256x}")?;
				}
				else {
					write!(&mut places[0], "{h:-<256x}")?;
				}
			}
			if l == 0 {
				if fmt.alternate() {
					write!(&mut places[1], "{m:─<16x}")?;
				}
				else {
					write!(&mut places[1], "{m:-<16x}")?;
				}
			}
			write!(&mut places[2], "{l:x}")?;
		}
		// Truncate each line.
		for line in &mut places[.. 2] {
			if let Some(snip) =
				line.char_indices().nth(cols_width).map(|(idx, _)| idx)
			{
				line.truncate(snip);
			}
		}
		let huge = max_row > 255;
		let big = max_row > 15;
		let pfx_cols = if huge {
			3
		}
		else if big {
			2
		}
		else {
			1
		};
		if huge {
			writeln!(
				fmt,
				"{: <pfx$}{sep}{line}",
				"",
				sep = drawings[1],
				line = places[0],
				pfx = pfx_cols,
			)?;
		}
		if big {
			writeln!(
				fmt,
				"{: <pfx$}{sep}{line}",
				"",
				sep = drawings[1],
				line = places[1],
				pfx = pfx_cols,
			)?;
		}
		writeln!(
			fmt,
			"{: <pfx$}{sep}{line}",
			drawings[3],
			sep = drawings[1],
			line = places[2],
			pfx = pfx_cols,
		)?;
		if fmt.alternate() {
			writeln!(
				fmt,
				"{:─<pfx$}┼{:─<cols$}",
				"",
				"",
				pfx = pfx_cols,
				cols = cols_width,
			)?;
		}
		else {
			writeln!(
				fmt,
				"{:-<pfx$}+{:-<cols$}",
				"",
				"",
				pfx = pfx_cols,
				cols = cols_width,
			)?;
		}
		for row in Axis::new(I::ZERO, max_row + 1) {
			write!(fmt, "{row: >w$x}{}", drawings[1], w = pfx_cols)?;

			for col in Axis::new(I::ZERO, max_col + 1) {
				let sym = per_cell(
					Point::new(col, row),
					&self.table[row.as_usize()][col.as_usize()],
				);
				fmt.write_char(sym)?;
			}
			writeln!(fmt)?;
		}
		Ok(())
	}
}

impl<I: Signed, T> Default for Cartesian2D<I, T> {
	fn default() -> Self {
		Self {
			origin: Point::new(I::ZERO, I::ZERO),
			table:  vec![],
		}
	}
}

impl<I: Signed, T> Index<Point<I>> for Cartesian2D<I, T> {
	type Output = T;

	fn index(&self, index: Point<I>) -> &Self::Output {
		let shifted = index - self.origin;
		// tracing::debug!(%shifted, "indexing");
		&self.table[shifted.y.as_usize()][shifted.x.as_usize()]
	}
}

impl<I: Signed, T> IndexMut<Point<I>> for Cartesian2D<I, T> {
	fn index_mut(&mut self, index: Point<I>) -> &mut Self::Output {
		let shifted = index - self.origin;
		// tracing::debug!(%shifted, "indexing");
		&mut self.table[shifted.y.as_usize()][shifted.x.as_usize()]
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Axis<I: Signed> {
	bgn: I,
	end: I,
}

impl<I: Signed> Axis<I>
where <I as TryFrom<usize>>::Error: fmt::Debug
{
	fn new(start: I, length: usize) -> Self {
		Self {
			bgn: start,
			end: (start.as_usize() + length)
				.try_into()
				.expect("out of bounds for axis unit"),
		}
	}
}

impl<I: Signed> Iterator for Axis<I> {
	type Item = I;

	fn next(&mut self) -> Option<Self::Item> {
		if self.bgn >= self.end {
			return None;
		}
		let out = self.bgn;
		self.bgn += I::ONE;
		Some(out)
	}
}

impl<I: Signed> DoubleEndedIterator for Axis<I> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.bgn >= self.end {
			return None;
		}
		self.end -= I::ONE;
		Some(self.end)
	}
}

impl<I: Signed> ExactSizeIterator for Axis<I> {
	fn len(&self) -> usize {
		(self.end - self.bgn).as_usize()
	}
}

impl<I: Signed> FusedIterator for Axis<I> {
}
