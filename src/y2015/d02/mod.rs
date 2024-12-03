use nom::{
	bytes::complete::tag,
	character::complete::newline,
	combinator::map,
	multi::separated_list1,
	sequence::tuple,
};

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2015, 2, |t| t.parse_dyn_puzzle::<Dimensions>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dimensions {
	presents: Vec<Dim>,
}

impl<'a> Parsed<&'a str> for Dimensions {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(newline, Dim::parse_wyz), |presents| Self {
			presents,
		})(text)
	}
}

impl Puzzle for Dimensions {
	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(self.presents.iter().map(Dim::paper_needed).sum::<i32>() as i64)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		Ok(self.presents.iter().map(Dim::ribbon_needed).sum::<i32>() as i64)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dim {
	x: i32,
	y: i32,
	z: i32,
}

impl Dim {
	pub fn paper_needed(&self) -> i32 {
		let xy = self.x * self.y;
		let yz = self.y * self.z;
		let zx = self.z * self.x;
		let extra = xy.min(yz).min(zx);
		2 * (xy + yz + zx) + extra
	}

	pub fn ribbon_needed(&self) -> i32 {
		let xy = 2 * (self.x + self.y);
		let yz = 2 * (self.y + self.z);
		let zx = 2 * (self.z + self.x);
		let bow = self.x * self.y * self.z;
		let min = xy.min(yz).min(zx);
		bow + min
	}
}

impl<'a> Parsed<&'a str> for Dim {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			tuple((
				parse_number,
				tag("x"),
				parse_number,
				tag("x"),
				parse_number,
			)),
			|(x, _, y, _, z)| Self { x, y, z },
		)(text)
	}
}
