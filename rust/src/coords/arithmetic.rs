use std::ops::Sub;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point2D {
	x: f64,
	y: f64,
}

impl Point2D {
	pub fn new(x: f64, y: f64) -> Self {
		Self { x, y }
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector2D {
	dx: f64,
	dy: f64,
}

impl Vector2D {
	pub fn new(dx: f64, dy: f64) -> Self {
		Self { dx, dy }
	}

	pub fn slope(self) -> f64 {
		self.dy / self.dx
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Particle2D {
	p0: Point2D,
	v:  Vector2D,
}

impl Particle2D {
	pub fn new(p0: Point2D, v: Vector2D) -> Self {
		Self { p0, v }
	}

	pub fn y_intercept(self) -> f64 {
		// y = mx + b
		// b = y - mx
		self.p0.y - self.v.slope() * self.p0.x
	}

	pub fn intersection(self, them: Self) -> Option<((f64, f64), Point2D)> {
		let m_a = self.v.slope();
		let m_b = them.v.slope();
		if (m_a - m_b).abs() < 0.001 {
			return None;
		}
		let x_i = (self.p0.y - them.p0.y) / (m_a - m_b);
		let y_ia = m_a * x_i + self.p0.y;
		let y_ib = m_b * x_i + them.p0.y;
		let diff = (y_ia - y_ib).abs();
		if diff > (y_ia.min(y_ib) * 0.001).max(0.001) {
			tracing::warn!(%y_ia, %y_ib, %diff, "divergent intersection calculations");
		}
		let pt = Point2D::new(x_i, y_ia);
		let t_a = (x_i - self.p0.x) / self.v.dx;
		let t_b = (x_i - them.p0.x) / them.v.dx;
		Some(((t_a, t_b), pt))
	}

	pub fn project(self, time: f64) -> Point2D {
		let x = self.p0.x + self.v.dx * time;
		let y = self.p0.y + self.v.dy * time;
		Point2D::new(x, y)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point3D {
	x: f64,
	y: f64,
	z: f64,
}

impl Point3D {
	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Self { x, y, z }
	}
}

impl Sub<Self> for Point3D {
	type Output = Vector3D;

	fn sub(self, them: Self) -> Vector3D {
		Vector3D::new(self.x - them.x, self.y - them.y, self.z - them.z)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector3D {
	dx: f64,
	dy: f64,
	dz: f64,
}

impl Vector3D {
	pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
		Self { dx, dy, dz }
	}

	pub fn slope_xy(self) -> f64 {
		Vector2D::new(self.dx, self.dy).slope()
	}

	pub fn slope_xz(self) -> f64 {
		Vector2D::new(self.dx, self.dz).slope()
	}

	pub fn slope_yz(self) -> f64 {
		Vector2D::new(self.dy, self.dz).slope()
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Particle3D {
	p0: Point3D,
	v:  Vector3D,
}

impl Particle3D {
	pub fn new(p0: Point3D, v: Vector3D) -> Self {
		Self { p0, v }
	}

	pub fn project(self, time: f64) -> Point3D {
		let x = self.p0.x + self.v.dx * time;
		let y = self.p0.y + self.v.dy * time;
		let z = self.p0.z + self.v.dz * time;
		Point3D::new(x, y, z)
	}

	/// Finds an intersection between two rays.
	pub fn intersection(self, them: Self) -> Option<((f64, f64), Point3D)> {
		todo!()
	}
}
