use std::{
	collections::{
		BTreeMap,
		BTreeSet,
		VecDeque,
	},
	ops::RangeInclusive,
};

use funty::Signed;
use tap::Pipe;

use super::{
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

/// A 3-dimensional planar grid, sparsely populated.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian3D<I: Signed, T> {
	planes: BTreeMap<I, Cartesian2D<I, T>>,
	bounds: Option<(Cartesian3DPoint<I>, Cartesian3DPoint<I>)>,
}

impl<I: Signed, T> Cartesian2D<I, T> {
	/// Creates a new, blank, 2-D grid.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn raw_data(&self) -> &BTreeMap<I, BTreeMap<I, T>> {
		&self.rows
	}

	/// Resets the grid.
	pub fn clear(&mut self) {
		*self = Self::new();
	}

	pub fn is_empty(&self) -> bool {
		self.rows.is_empty() || self.rows.values().all(|row| row.is_empty())
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
	pub fn iter(&self) -> impl Iterator<Item = (Cartesian2DPoint<I>, &T)> {
		self.rows
			.iter()
			.map(|(&y, row)| {
				row.iter()
					.map(move |(&x, val)| (Cartesian2DPoint::new(x, y), val))
			})
			.flatten()
	}

	pub fn into_iter(self) -> impl Iterator<Item = (Cartesian2DPoint<I>, T)> {
		self.rows
			.into_iter()
			.map(|(y, row)| {
				row.into_iter()
					.map(move |(x, val)| (Cartesian2DPoint::new(x, y), val))
			})
			.flatten()
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

impl<I: Signed, T> Cartesian3D<I, T> {
	/// Creates a new, blank, 3-D grid.
	pub fn new() -> Self {
		Self::default()
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
}

impl<I: Signed, T> Default for Cartesian2D<I, T> {
	fn default() -> Self {
		Self {
			rows:   BTreeMap::new(),
			bounds: None,
		}
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
