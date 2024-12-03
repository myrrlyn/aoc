/*! Densely-filled 2-dimensional spaces

This module provides data-types which are specialized for finite spaces which
are densely packed. The current implementation requires that *all* elements in
the space boundaries are fully populated, and has no support for holes. If your
space requires holes, you must either use `Option` as the storage type, or
switch to the sparse structure.
*/

use std::{
	fmt,
	iter::{
		self,
		FusedIterator,
	},
	ops::{
		Index,
		IndexMut,
	},
};

use funty::Signed;
use tap::Tap;

use super::{
	Point2D,
	Sparse2D,
};

/// A 2-dimensional Cartesian grid where all cells within the bounds are filled
/// with some value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian2D<I: Signed, T> {
	origin: Point2D<I>,
	table:  Vec<Vec<T>>,
}

impl<I: Signed, T> Cartesian2D<I, T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_raw(origin: Point2D<I>, table: Vec<Vec<T>>) -> Self {
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

	pub fn dimensions(&self) -> Option<(Point2D<I>, Point2D<I>)> {
		let rows = self.table.len().checked_sub(1)?;
		let cols = self.table.first()?.len().checked_sub(1)?;
		let min = self.origin;
		let max =
			Point2D::new(I::try_from(cols).ok()?, I::try_from(rows).ok()?) + min;
		Some((min, max))
	}

	pub fn in_bounds(&self, point: Point2D<I>) -> bool {
		if self.is_empty() {
			return false;
		};
		let shifted = point - self.origin;
		shifted.y.as_usize() < self.table.len()
			&& shifted.x.as_usize() < self.table[0].len()
	}

	/// Gets a single tile from the grid.
	pub fn get(&self, point: Point2D<I>) -> Option<&T> {
		let point = point - self.origin;
		self.table.get(point.y.as_usize())?.get(point.x.as_usize())
	}

	/// Gets an entire row from the grid.
	pub fn get_row(&self, row: I) -> Option<&[T]> {
		let r_abs = row - self.origin.y;
		self.table.get(r_abs.as_usize()).map(Vec::as_slice)
	}

	/// Iterates through each tile in the grid, in row-major order.
	pub fn iter(
		&self,
	) -> impl Iterator<Item = (Point2D<I>, &T)> + DoubleEndedIterator + FusedIterator
	where <I as TryFrom<isize>>::Error: fmt::Debug {
		let Point2D { y, x } = self.origin;
		let ys = self.table.len();
		let xs = self.table.first().map(|v| v.len()).unwrap_or_default();
		self.table
			.iter()
			.zip(Axis::new(y, ys))
			.map(move |(row, y)| {
				row.iter()
					.zip(Axis::new(x, xs))
					.map(move |(val, x)| (Point2D::new(x, y), val))
			})
			.flatten()
	}

	/// Consumes the grid, yielding each tile in row-major order.
	pub fn into_iter(
		self,
	) -> impl Iterator<Item = (Point2D<I>, T)> + DoubleEndedIterator + FusedIterator
	where <I as TryFrom<isize>>::Error: fmt::Debug {
		let Point2D { y, x } = self.origin;
		let ys = self.table.len();
		let xs = self.table.first().map(|v| v.len()).unwrap_or(0);
		self.table
			.into_iter()
			.zip(Axis::new(y, ys))
			.map(move |(row, y)| {
				row.into_iter()
					.zip(Axis::new(x, xs))
					.map(move |(val, x)| (Point2D::new(x, y), val))
			})
			.flatten()
	}
}

impl<I: Signed, T: Default> From<Sparse2D<I, T>> for Cartesian2D<I, T> {
	fn from(sparse: Sparse2D<I, T>) -> Self {
		let Some((origin, extent)) = sparse.dimensions()
		else {
			return Self::new();
		};
		let dim = extent - origin;
		let table = iter::from_fn(|| {
			Some(
				iter::from_fn(|| Some(T::default()))
					.take(dim.x.as_usize() + 1)
					.collect::<Vec<_>>(),
			)
		})
		.take(dim.y.as_usize() + 1)
		.collect::<Vec<_>>();
		Self { origin, table }.tap_mut(|this| {
			for (point, value) in sparse.into_iter() {
				this[point] = value;
			}
		})
	}
}

impl<I: Signed, T> Default for Cartesian2D<I, T> {
	fn default() -> Self {
		Self {
			origin: Point2D::new(I::ZERO, I::ZERO),
			table:  vec![],
		}
	}
}

impl<I: Signed, T> Index<Point2D<I>> for Cartesian2D<I, T> {
	type Output = T;

	fn index(&self, index: Point2D<I>) -> &Self::Output {
		let shifted = index - self.origin;
		// tracing::debug!(%shifted, "indexing");
		&self.table[shifted.y.as_usize()][shifted.x.as_usize()]
	}
}

impl<I: Signed, T> IndexMut<Point2D<I>> for Cartesian2D<I, T> {
	fn index_mut(&mut self, index: Point2D<I>) -> &mut Self::Output {
		let shifted = index - self.origin;
		// tracing::debug!(%shifted, "indexing");
		&mut self.table[shifted.y.as_usize()][shifted.x.as_usize()]
	}
}

impl<I: Signed, T: Default + PartialEq> super::DisplayGrid<I, T>
	for Cartesian2D<I, T>
{
	fn bounds_inclusive(&self) -> Option<(Point2D<I>, Point2D<I>)> {
		self.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &super::Symbols,
		_row: I,
		_col: I,
		row_abs: usize,
		col_abs: usize,
	) -> char {
		if self.table[row_abs][col_abs] == T::default() {
			symbols.empty
		}
		else {
			symbols.full
		}
	}
}

/// Roughly equivalent to `Range<I>`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axis<I: Signed> {
	bgn: I,
	end: I,
}

impl<I: Signed> Axis<I> {
	pub fn new_inclusive(min: I, max: I) -> Self {
		let (min, max) = (min.min(max), min.max(max));
		Self {
			bgn: min,
			end: max + I::ONE,
		}
	}
}

impl<I: Signed> Axis<I>
where <I as TryFrom<isize>>::Error: fmt::Debug
{
	fn new(start: I, length: usize) -> Self {
		Self {
			bgn: start,
			end: (start.as_isize() + length as isize)
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
