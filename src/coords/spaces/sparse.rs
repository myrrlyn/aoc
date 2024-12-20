//! Implementation of a sparsely-populated 2-dimensional Cartesian space.

use std::{
	collections::{
		BTreeMap,
		BTreeSet,
		VecDeque,
	},
	fmt::{
		self,
	},
	iter::FusedIterator,
	ops::RangeInclusive,
};

use funty::Signed;
use tap::Pipe;

use crate::coords::{
	Cartesian2DPoint,
	Cartesian3DPoint,
};

/// A 2-dimensional planar grid, sparsely populated.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian2D<I: Signed, T> {
	rows:   BTreeMap<I, BTreeMap<I, T>>,
	bounds: Option<(Cartesian2DPoint<I>, Cartesian2DPoint<I>)>,
}

impl<I: Signed, T> Cartesian2D<I, T> {
	/// Creates a new, blank, 2-D grid.
	pub fn new() -> Self {
		Self::default()
	}

	/// Views the underling B-Tree storage in case the provided APIs are not
	/// sufficient.
	pub fn raw_data(&self) -> &BTreeMap<I, BTreeMap<I, T>> {
		&self.rows
	}

	/// Resets the grid.
	pub fn clear(&mut self) {
		*self = Self::new();
	}

	/// Checks if the graph is empty.
	pub fn is_empty(&self) -> bool {
		self.rows.is_empty() || self.rows.values().all(|row| row.is_empty())
	}

	pub fn len(&self) -> usize {
		self.rows.values().map(BTreeMap::len).sum::<usize>()
	}

	/// If the space is not empty, returns a pair of points describing its
	/// bounding box.
	///
	/// ## Returns
	///
	/// The first element of the tuple has the minimum X and Y values observed
	/// in the space. These values do not have to be from the same point.
	///
	/// The second element of the tuple has the maximum X and Y values observed
	/// in the space. These values do not have to be from the same point.
	pub fn dimensions(
		&self,
	) -> Option<(Cartesian2DPoint<I>, Cartesian2DPoint<I>)> {
		let mut points = self.iter().map(|(pt, _)| pt);
		let mut min = points.next()?;
		let mut max = min;
		for Cartesian2DPoint { x, y } in points {
			min.x = min.x.min(x);
			min.y = min.y.min(y);
			max.x = max.x.max(x);
			max.y = max.y.max(y);
		}
		Some((min, max))
	}

	/// Tests if the graph stores a value at a given point.
	pub fn contains(
		&self,
		Cartesian2DPoint { x, y }: Cartesian2DPoint<I>,
	) -> bool {
		self.rows
			.get(&y)
			.map(|row| row.contains_key(&x))
			.unwrap_or(false)
	}

	/// Tests if the graph's area, as bounded from the unified minimum to the
	/// unified maximum of its points, encloses the described point.
	///
	/// There does not need to be a value stored at the point.
	pub fn encloses(&self, point: Cartesian2DPoint<I>) -> bool {
		let (min, max) = match self.bounds {
			Some(points) => points,
			None => return false,
		};
		(min.x <= point.x && point.x <= max.x)
			&& (min.y <= point.y && point.y <= max.y)
	}

	/// Views a value stored at a given point.
	pub fn get(
		&self,
		Cartesian2DPoint { x, y }: Cartesian2DPoint<I>,
	) -> Option<&T> {
		self.rows.get(&y).and_then(|row| row.get(&x))
	}

	/// Gets a mutable view to a particular value.
	pub fn get_mut(
		&mut self,
		Cartesian2DPoint { x, y }: Cartesian2DPoint<I>,
	) -> Option<&mut T> {
		self.rows.get_mut(&y).and_then(|row| row.get_mut(&x))
	}

	/// Inserts a value into the graph at a given point.
	pub fn insert(&mut self, point: Cartesian2DPoint<I>, value: T) {
		self.get_or_insert_with(point, || value);
	}

	/// Removes a value from the graph at a given point.
	pub fn remove(&mut self, point: Cartesian2DPoint<I>) -> Option<T> {
		self.rows.get_mut(&point.y)?.remove(&point.x)
	}

	/// Applies a transform function to the value stored at the given point.
	///
	/// If there is no value at the point, then the default value is constructed
	/// before being passed to the update function.
	pub fn update_default(
		&mut self,
		point: Cartesian2DPoint<I>,
		func: impl FnOnce(&mut T),
	) where
		T: Default,
	{
		self.rows
			.entry(point.y)
			.or_default()
			.entry(point.x)
			.or_default()
			.pipe(func);
	}

	/// Views a value stored at a give point. If the point is not currently
	/// stored within the graph, it is emplaced by calling the provided `fill`
	/// function.
	pub fn get_or_insert_with(
		&mut self,
		point @ Cartesian2DPoint { x, y }: Cartesian2DPoint<I>,
		fill: impl FnOnce() -> T,
	) -> &mut T {
		let out = self
			.rows
			.entry(y)
			.or_default()
			.entry(x)
			.or_insert_with(fill);
		self.bounds = match self.bounds.take() {
			None => Some((point, point)),
			Some((min, max)) => {
				Some((point.min_unifying(min), point.max_unifying(max)))
			},
		};
		out
	}

	/// Iterates over all points that have a live value.
	pub fn iter(
		&self,
	) -> impl Iterator<Item = (Cartesian2DPoint<I>, &T)>
	       + DoubleEndedIterator
	       + FusedIterator
	       + Clone {
		self.rows
			.iter()
			.map(|(&y, row)| {
				row.iter()
					.map(move |(&x, val)| (Cartesian2DPoint::new(x, y), val))
			})
			.flatten()
	}

	/// Consumes the graph, iterating over its values in row-major order from
	/// origin to the full extent.
	pub fn into_iter(
		self,
	) -> impl Iterator<Item = (Cartesian2DPoint<I>, T)>
	       + DoubleEndedIterator
	       + FusedIterator {
		self.rows
			.into_iter()
			.map(|(y, row)| {
				row.into_iter()
					.map(move |(x, val)| (Cartesian2DPoint::new(x, y), val))
			})
			.flatten()
	}

	/// Yields only the values which are placed in a particular row. They are
	/// yielded in order of increasing column.
	pub fn row<'a>(
		&'a self,
		row: I,
	) -> impl 'a
	       + Iterator<Item = (Cartesian2DPoint<I>, &'a T)>
	       + DoubleEndedIterator
	       + FusedIterator {
		// Filtering rows *before* producing the column iterator results in less
		// wasted work skipping over non-matching points.
		self.rows
			.iter()
			.filter(move |&(&r, _)| r == row)
			.map(|(&r, row)| {
				row.iter()
					.map(move |(&c, val)| (Cartesian2DPoint::new(c, r), val))
			})
			.flatten()
	}

	/// Yields only the values which are placed in a particular column. They are
	/// yielded in order of increasing row.
	pub fn column<'a>(
		&'a self,
		column: I,
	) -> impl 'a
	       + Iterator<Item = (Cartesian2DPoint<I>, &'a T)>
	       + DoubleEndedIterator
	       + FusedIterator {
		// Each row has either zero or one entries in the column.
		self.rows.iter().flat_map(move |(&r, row)| {
			row.get(&column)
				.map(move |val| (Cartesian2DPoint::new(column, r), val))
		})
	}
}

impl<I: Signed, T> Default for Cartesian2D<I, T> {
	fn default() -> Self {
		Self {
			rows:   BTreeMap::new(),
			bounds: None,
		}
	}
}

/// Renders the grid in Quadrant IV, normalizing to have the minimum point set
/// at the origin.
impl<I: Signed, T> fmt::Display for Cartesian2D<I, T> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		super::DisplayGrid::render(self, fmt)
	}
}

impl<I: Signed, T> FromIterator<(Cartesian2DPoint<I>, T)> for Cartesian2D<I, T> {
	fn from_iter<II: IntoIterator<Item = (Cartesian2DPoint<I>, T)>>(
		src: II,
	) -> Self {
		src.into_iter().fold(Self::new(), |mut this, (coord, val)| {
			this.insert(coord, val);
			this
		})
	}
}

impl<I: Signed, T> super::DisplayGrid<I, T> for Cartesian2D<I, T> {
	fn bounds_inclusive(
		&self,
	) -> Option<(Cartesian2DPoint<I>, Cartesian2DPoint<I>)> {
		self.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &super::Symbols,
		row: I,
		col: I,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		if self.contains(Cartesian2DPoint::new(col, row)) {
			symbols.full
		}
		else {
			symbols.empty
		}
	}
}

/// A 3-dimensional planar grid, sparsely populated.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian3D<I: Signed, T> {
	/// A three-dimensional volume which contains any collected objects.
	planes: BTreeMap<I, Cartesian2D<I, T>>,
	/// The volume can optionally be constrained differently than the points
	/// that have been directly added to
	bounds: Option<(Cartesian3DPoint<I>, Cartesian3DPoint<I>)>,
}

impl<I: Signed, T> Cartesian3D<I, T> {
	/// Creates a new, blank, 3-D grid.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.planes.values().map(Cartesian2D::len).sum::<usize>()
	}

	/// Tests if the graph stores a value at a given point.
	pub fn contains(&self, point: Cartesian3DPoint<I>) -> bool {
		let (z, xy) = point.make_2d();
		self.planes
			.get(&z)
			.map(|plane| plane.contains(xy))
			.unwrap_or(false)
	}

	/// Tests if the graph's volume, as bounded from the unified minimum to the
	/// unified maximum of its points, encloses the described point.
	///
	/// There does not need to be a value stored at the point.
	pub fn encloses(&self, point: Cartesian3DPoint<I>) -> bool {
		let (min, max) = match self.bounds {
			Some(points) => points,
			None => return false,
		};
		(min.x <= point.x && point.x <= max.x)
			&& (min.y <= point.y && point.y <= max.y)
			&& (min.z <= point.z && point.z <= max.z)
	}

	/// Gets the minimum and maximum points which describe a bounding box
	/// containing the graph.
	pub fn bounds_inclusive(
		&self,
	) -> Option<(Cartesian3DPoint<I>, Cartesian3DPoint<I>)> {
		self.bounds
	}

	/// Views a value stored at a given point.
	pub fn get(&self, point: Cartesian3DPoint<I>) -> Option<&T> {
		let (z, xy) = point.make_2d();
		self.planes.get(&z).and_then(|plane| plane.get(xy))
	}

	/// Inserts a value into the graph at a given point.
	pub fn insert(&mut self, point: Cartesian3DPoint<I>, value: T) {
		self.get_or_insert_with(point, || value);
	}

	/// Views a value stored at a given point. If the point is not currently
	/// stored within the graph, it is emplaced by calling the provided `fill`
	/// function.
	pub fn get_or_insert_with(
		&mut self,
		point: Cartesian3DPoint<I>,
		fill: impl FnOnce() -> T,
	) -> &mut T {
		let (z, xy) = point.make_2d();
		let out = self
			.planes
			.entry(z)
			.or_default()
			.get_or_insert_with(xy, fill);
		self.bounds = match self.bounds.take() {
			None => Some((point, point)),
			Some((min, max)) => {
				Some((point.min_unifying(min), point.max_unifying(max)))
			},
		};
		out
	}

	/// Performs a breadth-first search across a graph.
	pub fn search_bfs<CS: IntoIterator<Item = Cartesian3DPoint<I>>>(
		&self,
		initial_search: impl FnOnce(&Self) -> CS,
		mut searcher: impl FnMut(
			Cartesian3DPoint<I>,
			&Self,
			&mut VecDeque<Cartesian3DPoint<I>>,
		),
	) {
		let mut visited = BTreeSet::new();
		let mut search_queue = VecDeque::new();
		search_queue.extend(initial_search(self));
		while let Some(pt) = search_queue.pop_front() {
			if !self.encloses(pt) || !visited.insert(pt) {
				continue;
			}
			searcher(pt, self, &mut search_queue);
		}
	}

	pub fn stream_volume(&self) -> impl Iterator<Item = Cartesian3DPoint<I>>
	where RangeInclusive<I>: IntoIterator<Item = I> {
		self.bounds.into_iter().flat_map(
			|(
				Cartesian3DPoint {
					x: x1,
					y: y1,
					z: z1,
				},
				Cartesian3DPoint {
					x: x2,
					y: y2,
					z: z2,
				},
			)| {
				(z1 ..= z2)
					.into_iter()
					.flat_map(move |z| {
						(y1 ..= y2).into_iter().map(move |y| (y, z))
					})
					.flat_map(move |(y, z)| {
						(x1 ..= x2).into_iter().map(move |x| (x, y, z))
					})
					.map(Cartesian3DPoint::from)
			},
		)
	}

	pub fn iter(
		&self,
	) -> impl Iterator<Item = (Cartesian3DPoint<I>, &T)>
	       + DoubleEndedIterator
	       + FusedIterator {
		self.planes.iter().flat_map(|(&z, xy)| {
			xy.iter().map(move |(pt, val)| (pt.make_3d(z), val))
		})
	}
}

impl<I: Signed, T> Default for Cartesian3D<I, T> {
	fn default() -> Self {
		Self {
			planes: BTreeMap::new(),
			bounds: None,
		}
	}
}
