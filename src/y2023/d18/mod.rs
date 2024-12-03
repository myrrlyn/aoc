#![doc = include_str!("README.md")]

use std::fmt;

use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		take,
	},
	character::complete::{
		hex_digit1,
		i32 as get_i32,
		newline,
		space1,
	},
	combinator::{
		map,
		map_parser,
		map_res,
		value,
	},
	multi::separated_list1,
	sequence::{
		delimited,
		preceded,
		terminated,
		tuple,
	},
};
use rayon::prelude::*;
use tap::Tap;

use crate::{
	coords::{
		points::{
			Cartesian2D as Point2D,
			Direction2D,
		},
		spaces::dense::Axis,
	},
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 18, |t| t.parse_dyn_puzzle::<Lavagoon>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lavagoon {
	/// The list of pen motions in the area.
	sequence: Vec<Instruction>,
	/// The expansion of every Instruction into a single linear move.
	segments: Vec<Stroke>,
}

impl<'a> Parsed<&'a str> for Lavagoon {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(newline, Instruction::parse_wyz),
			|sequence| Self {
				sequence,
				segments: vec![],
			},
		)(text)
	}
}

impl Puzzle for Lavagoon {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.compute_strokes();
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.stroked_area()
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.sequence
			.par_iter_mut()
			.for_each(|insn| *insn = insn.swap_components());
		self.compute_strokes();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.stroked_area()
	}
}

impl Lavagoon {
	/// Transforms each relative instruction into an absolute movement.
	pub fn compute_strokes(&mut self) {
		self.segments.clear();
		let mut cursor = Point2D::ZERO;
		for &insn in self.sequence.as_slice() {
			// tracing::debug!(%insn, "swapped");
			let bgn = cursor;
			let end = match insn.direction {
				Direction2D::North => bgn - Point2D::new(0, insn.distance),
				Direction2D::South => bgn + Point2D::new(0, insn.distance),
				Direction2D::West => bgn - Point2D::new(insn.distance, 0),
				Direction2D::East => bgn + Point2D::new(insn.distance, 0),
			};
			self.segments.push(Stroke {
				bgn,
				end,
				dir: insn.direction,
			});
			cursor = end;
		}
	}

	/// Computes the area enclosed by the strokes.
	pub fn stroked_area(&self) -> eyre::Result<i64> {
		if self
			.segments
			.last()
			.ok_or_else(|| eyre::eyre!("cannot compute on an empty move-set"))?
			.end != Point2D::ZERO
		{
			eyre::bail!(
				"cannot compute the area enclosed by a curve that is not a \
				 closed circuit"
			);
		}

		// Get the bounding box of the entire drawn area.
		let (min, max) = self.segments.iter().fold(
			(Point2D::<i32>::ZERO, Point2D::<i32>::ZERO),
			|(mut min, mut max), stroke| {
				min.y = min.y.min(stroke.bgn.y).min(stroke.end.y);
				min.x = min.x.min(stroke.bgn.x).min(stroke.end.x);
				max.y = max.y.max(stroke.bgn.y).max(stroke.end.y);
				max.x = max.x.max(stroke.bgn.x).max(stroke.end.x);
				(min, max)
			},
		);

		let mut accum = 0;
		let mut progress = 0.0;
		tracing::debug!("beginning computation");
		for row in Axis::new_inclusive(min.y, max.y) {
			let pct = ((row as f64) / (max.y as f64) * 100.0).floor();
			let span = tracing::error_span!("row", %pct);
			let _span = span.enter();
			// Get all strokes which contact the current scan row.
			let strokes_in_row = self
				.segments
				.iter()
				.copied()
				.filter(|s| s.touches_row(row))
				.collect::<Vec<_>>();

			let (horiz, vert) =
				strokes_in_row.into_iter().partition::<Vec<_>, _>(|s| {
					s.dir == Direction2D::West || s.dir == Direction2D::East
				});
			// Only take the cells which are *not* in a vertical stroke.
			accum += horiz.iter().map(|h| h.len_exclusive()).sum::<i64>();
			// We know that this is a perfect non-convex curve in rectangular
			// space. This means that for any pair of adjacent vertical strokes
			// in a row, if the strokes point in the same direction, then they
			// have to be joined by a horizontal stroke, so the intervening
			// distance *is* counted, but the inside flag does not flip
			let mut inside = true;
			// The left-most stroke needs to be counted first.
			accum += 1;
			// Then, traverse each neighbored pairing and maybe-add the
			// distance between them, excluding the left stroke and including
			// the right.
			for pair in vert.tap_mut(|v| v.sort_by_key(|s| s.bgn.x)).windows(2) {
				let &[left, right] = pair
				else {
					unreachable!("windows always yields perfect subslices");
				};
				let (lx, rx) = (left.bgn.x, right.bgn.x);
				// Same-direction strokes are joined by a horizontal stroke. The
				// interior has already been counted, and the winding flag does
				// not flip.
				if left.dir == right.dir {
					continue;
				}
				// Opposing-direction strokes add their distance only while the
				// winding flag is high.
				if !horiz.iter().any(|h| h.touches_col(lx) && h.touches_col(rx))
				{
					if inside {
						accum += (rx - lx) as i64 - 1;
					}
					accum += 1;
				}
				inside = !inside;
			}
			if pct > progress {
				progress = pct;
				tracing::debug!(%accum);
			}
		}
		Ok(accum)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Instruction {
	direction: Direction2D,
	distance:  i32,
	hex_color: HexColor,
}

impl Instruction {
	pub fn swap_components(self) -> Self {
		let distance = ((self.hex_color.red as i32) << 12)
			| ((self.hex_color.grn as i32) << 4)
			| ((self.hex_color.blu as i32) >> 4);
		let direction = match self.hex_color.blu & 3 {
			0 => Direction2D::East,
			1 => Direction2D::South,
			2 => Direction2D::West,
			3 => Direction2D::North,
			_ => unreachable!("masking cannot produce any other values"),
		};
		let color_bytes = (distance << 4)
			| match self.direction {
				Direction2D::North => 3,
				Direction2D::South => 1,
				Direction2D::West => 2,
				Direction2D::East => 0,
			};
		let hex_color = HexColor {
			red: (color_bytes >> 16) as u8,
			grn: (color_bytes >> 8) as u8,
			blu: color_bytes as u8,
		};
		Self {
			direction,
			distance,
			hex_color,
		}
	}
}

impl<'a> Parsed<&'a str> for Instruction {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			tuple((
				terminated(
					alt((
						value(Direction2D::North, tag("U")),
						value(Direction2D::South, tag("D")),
						value(Direction2D::West, tag("L")),
						value(Direction2D::East, tag("R")),
					)),
					space1,
				),
				terminated(get_i32, space1),
				delimited(tag("("), HexColor::parse_wyz, tag(")")),
			)),
			|(direction, distance, hex_color)| Self {
				direction,
				distance,
				hex_color,
			},
		)(text)
	}
}

impl fmt::Display for Instruction {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(
			fmt,
			"{} {} ({})",
			self.direction, self.distance, self.hex_color
		)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexColor {
	pub red: u8,
	pub grn: u8,
	pub blu: u8,
}

impl<'a> Parsed<&'a str> for HexColor {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let hexpair = move |t| {
			map_res(map_parser(take(2usize), hex_digit1), |n| {
				u8::from_str_radix(n, 16)
			})(t)
		};
		map(
			preceded(tag("#"), tuple((hexpair, hexpair, hexpair))),
			|(red, grn, blu)| Self { red, grn, blu },
		)(text)
	}
}

impl fmt::Display for HexColor {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "#{:0>2x}{:0>2x}{:0>2x}", self.red, self.grn, self.blu)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tile {
	#[default]
	Void,
	Trench,
}

impl From<Tile> for char {
	fn from(tile: Tile) -> char {
		match tile {
			Tile::Void => ' ',
			Tile::Trench => '#',
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stroke {
	bgn: Point2D<i32>,
	end: Point2D<i32>,
	dir: Direction2D,
}

impl Stroke {
	pub fn touches_row(&self, row: i32) -> bool {
		let from = self.bgn.y.min(self.end.y);
		let to = self.bgn.y.max(self.end.y);
		(from ..= to).contains(&row)
	}

	pub fn touches_col(&self, col: i32) -> bool {
		let from = self.bgn.x.min(self.end.x);
		let to = self.bgn.x.max(self.end.x);
		(from ..= to).contains(&col)
	}

	pub fn len_inclusive(&self) -> i64 {
		1 + self.len_exclusive()
	}

	pub fn len_exclusive(&self) -> i64 {
		let out = match self.dir {
			Direction2D::North => self.bgn.y - self.end.y,
			Direction2D::South => self.end.y - self.bgn.y,
			Direction2D::West => self.bgn.x - self.end.x,
			Direction2D::East => self.end.x - self.bgn.x,
		};
		out as i64
	}
}

impl fmt::Display for Stroke {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let dist = self.end - self.bgn;
		let dist = dist.x.abs().max(dist.y.abs());
		write!(fmt, "{{{} | {}/{}}}", self.dir, self.bgn, dist)
	}
}
