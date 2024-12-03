use nom::{
	character::complete::{
		newline,
		space1,
	},
	multi::separated_list1,
};
use tap::Tap;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 2, |t| t.parse_dyn_puzzle::<Reports>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Reports {
	reports: Vec<Report>,
}

impl Puzzle for Reports {
	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(self
			.reports
			.iter()
			.map(Report::strict_condition)
			.filter(Condition::is_safe)
			.count() as i64)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		Ok(self
			.reports
			.iter()
			.map(Report::weak_condition)
			.filter(Condition::is_safe)
			.count() as i64)
	}
}

impl<'a> Parsed<&'a str> for Reports {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, reports) = separated_list1(newline, Report::parse_wyz)(src)?;
		Ok((rest, Reports { reports }))
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Report {
	values: Vec<i32>,
}

impl Report {
	pub fn conditions<'a>(&'a self) -> impl 'a + Iterator<Item = Condition> {
		self.values.windows(2).map(|elts| match elts {
			&[left, right] => match right - left {
				n if n > 3 => Condition::Excessive,
				n if (1 ..= 3).contains(&n) => Condition::Rising,
				0 => Condition::Chaotic,
				n if (-3 ..= -1).contains(&n) => Condition::Falling,
				n if n < -3 => Condition::Collapse,
				_ => Condition::Chaotic,
			},
			_ => panic!("windows never yields non-pair slices"),
		})
	}

	pub fn strict_condition(&self) -> Condition {
		self.conditions()
			.reduce(|prev, next| {
				if prev == next {
					prev
				}
				else {
					Condition::Chaotic
				}
			})
			.unwrap_or_default()
	}

	pub fn weak_condition(&self) -> Condition {
		let strict = self.strict_condition();
		if strict.is_safe() {
			return strict;
		}
		for idx in 0 .. self.values.len() {
			let values = self.values.clone().tap_mut(|vs| {
				vs.remove(idx);
			});
			let weak = Self { values }.strict_condition();
			if weak.is_safe() {
				return weak;
			}
		}
		strict
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Condition {
	Collapse,
	Falling,
	#[default]
	Chaotic,
	Rising,
	Excessive,
}

impl Condition {
	pub fn is_safe(&self) -> bool {
		match self {
			Self::Falling | Self::Rising => true,
			_ => false,
		}
	}
}

impl<'a> Parsed<&'a str> for Report {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, values) = separated_list1(space1, parse_number::<i32>)(src)?;
		Ok((rest, Self { values }))
	}
}
