use std::fmt;

use funty::Signed;
use nom::{
	bytes::complete::tag,
	character::complete::{
		i64 as get_i64,
		newline,
		space1,
	},
	combinator::map,
	multi::separated_list1,
	sequence::{
		delimited,
		pair,
		separated_pair,
		tuple,
	},
};

use crate::{
	prelude::*,
	Coord2D,
	Coord3D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 24, |t| t.parse_dyn_puzzle::<Hailstorm>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hailstorm {
	list: Vec<(Coord3D<i64>, Vector3D<i64>)>,
}

impl<'a> Parsed<&'a str> for Hailstorm {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		fn triple(text: &str) -> ParseResult<&str, (i64, i64, i64)> {
			map(
				tuple((
					get_i64,
					pair(tag(","), space1),
					get_i64,
					pair(tag(","), space1),
					get_i64,
				)),
				|(x, _, y, _, z)| (x, y, z),
			)(text)
		}
		let (rest, list) = separated_list1(
			newline,
			separated_pair(
				map(triple, |(x, y, z)| Coord3D::new(x, y, z)),
				delimited(space1, tag("@"), space1),
				map(triple, |(x, y, z)| Vector3D::new(x, y, z)),
			),
		)(text)?;
		Ok((rest, Self { list }))
	}
}

impl Puzzle for Hailstorm {
	fn part_1(&mut self) -> eyre::Result<i64> {
		const MIN: i64 = 200_000_000_000_000;
		const MAX: i64 = 400_000_000_000_000;
		// const MIN: i64 = 7;
		// const MAX: i64 = 27;
		//               680_830_520_594_748
		let ct = self
			.intersections_2d((Coord2D::new(MIN, MIN), Coord2D::new(MAX, MAX)));
		Ok(ct)
	}
}

impl Hailstorm {
	pub fn intersections_2d(
		&self,
		(min, max): (Coord2D<i64>, Coord2D<i64>),
	) -> i64 {
		let mut count = 0;
		for (a, n) in self.list.iter().copied().zip(0 ..) {
			let span = tracing::error_span!("a", %n);
			let _span = span.enter();
			for (b, n2) in self.list[n + 1 ..].iter().copied().zip(n + 1 ..) {
				let span = tracing::error_span!("b", %n2);
				let _span = span.enter();
				// skip auto-intersection any parallel lines.
				if a.0 == b.0 || a.1.slope_xy() == b.1.slope_xy() {
					continue;
				}
				if let Some((a_t, b_t, (x, y))) = intersect_2d(a, b) {
					let span = tracing::error_span!("i", %a_t, %b_t, %x, %y);
					let _span = span.enter();
					let in_x = (min.x as f64 ..= max.x as f64).contains(&x);
					let in_y = (min.y as f64 ..= max.y as f64).contains(&y);
					let fwd_a = a_t >= 0.0;
					let fwd_b = b_t >= 0.0;
					if in_x && in_y && fwd_a && fwd_b {
						tracing::debug!("accepted");
						count += 1;
					}
				}
			}
		}
		count
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector3D<I: Signed> {
	dx: I,
	dy: I,
	dz: I,
}

impl<I: Signed> Vector3D<I> {
	pub fn new(dx: I, dy: I, dz: I) -> Self {
		Self { dx, dy, dz }
	}

	pub fn slope_xy(&self) -> f64 {
		let out = self.dy.as_f64() / self.dx.as_f64();
		out
	}
}

impl<I: Signed> fmt::Display for Vector3D<I> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "(d{}, d{}, d{})", self.dx, self.dy, self.dz)
	}
}

/// Computes the intersection of two lines in 2-D space.
///
/// The slope equation is `y = m*x + b`, where `m` is the dy/dx slope and `b` is
/// the y-value at `x=0`. Two different lines intersect if there exists a point
/// `x` such that `m1 * x + b1 == m2 * x + b2`.
///
/// Yields the time of intersection for each particle, as well as the location
/// of the intersection.
pub fn intersect_2d(
	(a_pos, a_vel): (Coord3D<i64>, Vector3D<i64>),
	(b_pos, b_vel): (Coord3D<i64>, Vector3D<i64>),
) -> Option<(f64, f64, (f64, f64))> {
	let (a_x, a_y) = (a_pos.x as f64, a_pos.y as f64);
	let (b_x, b_y) = (b_pos.x as f64, b_pos.y as f64);

	let (a_dx, b_dx) = (a_vel.dx as f64, b_vel.dx as f64);

	// Find the slope of each vector.
	let a_m = a_vel.slope_xy();
	let b_m = b_vel.slope_xy();

	// Parallel vectors do not intersect.
	if a_m == b_m {
		return None;
	}

	// Find the Y-intercept.
	let a_x0 = a_y - (a_m * a_x);
	let b_x0 = b_y - (b_m * b_x);

	// a_m * x + a_x0 == b_m * x + b_x0
	// (a_m - b_m) * x == (b_x0 - a_x0)
	// x = (b_x0 - a_x0) / (a_m - b_m)
	let x = (b_x0 - a_x0) / (a_m - b_m);
	let y_a = a_m * x + a_x0;
	let y_b = b_m * x + b_x0;
	if (y_a - y_b).abs() > (0.001 * y_a.min(y_b).abs()).max(0.001) {
		tracing::warn!(%y_a, %y_b, diff=%(y_a - y_b).abs(), "intercept calculation divergence");
	}

	// Compute the time spent traveling along the vector. This is the
	// displacement along an axis divided by the rate of travel along that axis.
	let a_t = (x - a_x) / a_dx;
	let b_t = (x - b_x) / b_dx;
	Some((a_t, b_t, (x, y_a)))
}
