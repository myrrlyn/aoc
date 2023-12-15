use std::{
	fmt,
	ops::{
		self,
		Neg,
		RangeInclusive,
	},
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction2D {
	North,
	South,
	West,
	East,
}

impl Direction2D {
	pub fn unit<I: Signed>(self) -> Cartesian2D<I>
	where I: Neg<Output = I> {
		match self {
			Self::North => Cartesian2D::new(I::ZERO, -I::ONE),
			Self::South => Cartesian2D::new(I::ZERO, I::ONE),
			Self::West => Cartesian2D::new(-I::ONE, I::ZERO),
			Self::East => Cartesian2D::new(I::ONE, I::ZERO),
		}
	}
}
