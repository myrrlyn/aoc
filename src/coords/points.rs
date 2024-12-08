use std::{
	fmt::{
		self,
		Write as _,
	},
	iter::FusedIterator,
	ops::{
		self,
		BitOr,
		BitOrAssign,
		Neg,
		RangeInclusive,
	},
};

use bitvec::{
	array::BitArray,
	BitArr,
};
use funty::Signed;
use tap::Tap;

/// An integral co-ordinate on a two-dimensional gridded plane.
///
/// These are sorted by Y, then X.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian2D<I: Signed> {
	pub y: I,
	pub x: I,
}

/// An integral co-ordinate in a three-dimensional gridded volume.
///
/// These are sorted by Z, then Y, then X.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cartesian3D<I: Signed> {
	pub z: I,
	pub y: I,
	pub x: I,
}

impl<I: Signed> Cartesian2D<I> {
	pub const ZERO: Self = Self {
		x: I::ZERO,
		y: I::ZERO,
	};

	pub const fn new(x: I, y: I) -> Self {
		Self { x, y }
	}

	pub fn axial_distance(self, other: Self) -> I {
		let Self { x: x1, y: y1 } = self;
		let Self { x: x2, y: y2 } = other;

		let [x1, x2] = [x1, x2].tap_mut(|xs| xs.sort());
		let [y1, y2] = [y1, y2].tap_mut(|ys| ys.sort());

		(x2 - x1) + (y2 - y1)
	}

	pub fn nearby(self, radius: I, max_axial_distance: I) -> Vec<Self>
	where RangeInclusive<I>: IntoIterator<Item = I> {
		let mut out = Vec::new();
		for x in radius.wrapping_neg() ..= radius {
			for y in radius.wrapping_neg() ..= radius {
				let dist = x.abs() + y.abs();
				if dist == I::ZERO || dist > max_axial_distance {
					continue;
				}
				out.push(Self {
					x: self.x + x,
					y: self.y + y,
				});
			}
		}
		out
	}

	pub fn make_3d(self, z: I) -> Cartesian3D<I> {
		Cartesian3D {
			x: self.x,
			y: self.y,
			z,
		}
	}

	pub fn min_unifying(self, other: Self) -> Self {
		Self {
			x: self.x.min(other.x),
			y: self.y.min(other.y),
		}
	}

	pub fn max_unifying(self, other: Self) -> Self {
		Self {
			x: self.x.max(other.x),
			y: self.y.max(other.y),
		}
	}
}

impl<I: Signed + Neg<Output = I>> Cartesian2D<I> {
	/// Computes the four immediate neighbors of this point.
	///
	/// They are returned in `[N, S, W, E]` order.
	pub fn direct_neighbors(self) -> [Self; 4] {
		[
			self + Direction2D::North.unit(),
			self + Direction2D::South.unit(),
			self + Direction2D::West.unit(),
			self + Direction2D::East.unit(),
		]
	}

	pub fn abs_manhattan(self) -> I {
		self.x.abs() + self.y.abs()
	}
}

impl<I: Signed> Cartesian3D<I> {
	pub const ZERO: Self = Self {
		x: I::ZERO,
		y: I::ZERO,
		z: I::ZERO,
	};

	pub fn new(x: I, y: I, z: I) -> Self {
		Self { x, y, z }
	}

	pub fn axial_distance(self, other: Self) -> I {
		let Self {
			x: x1,
			y: y1,
			z: z1,
		} = self;
		let Self {
			x: x2,
			y: y2,
			z: z2,
		} = other;

		let [x1, x2] = [x1, x2].tap_mut(|xs| xs.sort());
		let [y1, y2] = [y1, y2].tap_mut(|ys| ys.sort());
		let [z1, z2] = [z1, z2].tap_mut(|zs| zs.sort());

		(x2 - x1) + (y2 - y1) + (z2 - z1)
	}

	pub fn nearby(self, radius: I, max_axial_distance: I) -> Vec<Self>
	where RangeInclusive<I>: IntoIterator<Item = I> {
		let mut out = Vec::new();
		for x in radius.wrapping_neg() ..= radius {
			for y in radius.wrapping_neg() ..= radius {
				for z in radius.wrapping_neg() ..= radius {
					let dist = x.abs() + y.abs() + z.abs();
					if dist == I::ZERO || dist > max_axial_distance {
						continue;
					}
					out.push(Self {
						x: self.x + x,
						y: self.y + y,
						z: self.z + z,
					});
				}
			}
		}
		out
	}

	pub fn make_2d(self) -> (I, Cartesian2D<I>) {
		(self.z, Cartesian2D {
			x: self.x,
			y: self.y,
		})
	}

	pub fn min_unifying(self, other: Self) -> Self {
		Self {
			x: self.x.min(other.x),
			y: self.y.min(other.y),
			z: self.z.min(other.z),
		}
	}

	pub fn max_unifying(self, other: Self) -> Self {
		Self {
			x: self.x.max(other.x),
			y: self.y.max(other.y),
			z: self.z.max(other.z),
		}
	}
}

impl<I: Signed> From<(I, I)> for Cartesian2D<I> {
	fn from((x, y): (I, I)) -> Self {
		Self { x, y }
	}
}

impl<I: Signed> From<(I, I, I)> for Cartesian3D<I> {
	fn from((x, y, z): (I, I, I)) -> Self {
		Self { x, y, z }
	}
}

impl<I: Signed> fmt::Display for Cartesian2D<I> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "({}, {})", self.x, self.y)
	}
}

impl<I: Signed> fmt::Display for Cartesian3D<I> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "({}, {}, {})", self.x, self.y, self.z)
	}
}

impl<I: Signed> ops::Add<Self> for Cartesian2D<I> {
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		self += rhs;
		self
	}
}

impl<I: Signed> ops::AddAssign<Self> for Cartesian2D<I> {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl<I: Signed> ops::Mul<I> for Cartesian2D<I> {
	type Output = Self;

	fn mul(mut self, rhs: I) -> Self {
		self *= rhs;
		self
	}
}

impl<I: Signed> ops::MulAssign<I> for Cartesian2D<I> {
	fn mul_assign(&mut self, rhs: I) {
		self.x *= rhs;
		self.y *= rhs;
	}
}

impl<I: Signed> ops::Sub<Self> for Cartesian2D<I> {
	type Output = Self;

	fn sub(mut self, rhs: Self) -> Self {
		self -= rhs;
		self
	}
}

impl<I: Signed> ops::SubAssign<Self> for Cartesian2D<I> {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl<I: Signed> ops::Add<Self> for Cartesian3D<I> {
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		self += rhs;
		self
	}
}

impl<I: Signed> ops::AddAssign<Self> for Cartesian3D<I> {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl<I: Signed> ops::Sub<Self> for Cartesian3D<I> {
	type Output = Self;

	fn sub(mut self, rhs: Self) -> Self {
		self -= rhs;
		self
	}
}

impl<I: Signed> ops::SubAssign<Self> for Cartesian3D<I> {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

/// A direction in a 2-D plane.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction2D {
	North,
	South,
	West,
	East,
}

impl Direction2D {
	/// Produces a point in the 2-D plane that is one unit away from origin in
	/// the specified direction.
	///
	/// North and West are -1 on their respective axes; South and East are
	/// positive.
	pub fn unit<I: Signed + Neg<Output = I>>(self) -> Cartesian2D<I> {
		match self {
			Self::North => Cartesian2D::new(I::ZERO, -I::ONE),
			Self::South => Cartesian2D::new(I::ZERO, I::ONE),
			Self::West => Cartesian2D::new(-I::ONE, I::ZERO),
			Self::East => Cartesian2D::new(I::ONE, I::ZERO),
		}
	}

	/// Yields each direction in the group.
	pub const fn all() -> [Self; 4] {
		[Self::North, Self::East, Self::South, Self::West]
	}

	/// Turns the direction one step clockwise.
	pub fn turn_right(self) -> Self {
		match self {
			Self::North => Self::East,
			Self::East => Self::South,
			Self::South => Self::West,
			Self::West => Self::North,
		}
	}
}

impl fmt::Display for Direction2D {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str(match (self, fmt.alternate()) {
			(Self::North, false) => "N",
			(Self::North, true) => "North",
			(Self::South, false) => "S",
			(Self::South, true) => "South",
			(Self::West, false) => "W",
			(Self::West, true) => "West",
			(Self::East, false) => "E",
			(Self::East, true) => "East",
		})
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DirectionSet2D {
	inner: BitArr![for 4, in u8],
}

impl DirectionSet2D {
	pub fn new() -> Self {
		Self {
			inner: BitArray::ZERO,
		}
	}

	pub fn insert(&mut self, direction: Direction2D) {
		self.inner.set(
			match direction {
				Direction2D::North => 0,
				Direction2D::South => 1,
				Direction2D::West => 2,
				Direction2D::East => 3,
			},
			true,
		);
	}

	pub fn contains(&self, direction: Direction2D) -> bool {
		self.inner[match direction {
			Direction2D::North => 0,
			Direction2D::South => 1,
			Direction2D::West => 2,
			Direction2D::East => 3,
		}]
	}

	pub fn contents<'a>(
		&'a self,
	) -> impl 'a + Iterator<Item = Direction2D> + FusedIterator + DoubleEndedIterator
	{
		Direction2D::all().into_iter().filter(|&d| self.contains(d))
	}
}

impl fmt::Display for DirectionSet2D {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		for (dir, sym) in Direction2D::all().into_iter().zip("NSWE".chars()) {
			fmt.write_char(if self.contains(dir) { sym } else { ' ' })?;
		}
		Ok(())
	}
}

impl BitOr<Direction2D> for DirectionSet2D {
	type Output = Self;

	fn bitor(mut self, rhs: Direction2D) -> Self::Output {
		self |= rhs;
		self
	}
}

impl BitOrAssign<Direction2D> for DirectionSet2D {
	fn bitor_assign(&mut self, rhs: Direction2D) {
		self.insert(rhs);
	}
}
