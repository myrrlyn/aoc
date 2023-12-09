use eyre::Context;
use nom::{
	bytes::complete::tag,
	character::complete::{digit1, newline, space1},
	combinator::{map, map_res, opt, recognize},
	multi::{many1, separated_list1},
	sequence::{preceded, terminated},
};

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 9, |t| t.parse_dyn_puzzle::<Oasis>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Oasis {
	histories: Vec<HistoricalRecord>,
}

impl<'a> Parsed<&'a str> for Oasis {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(many1(HistoricalRecord::parse_wyz), |histories| Self {
			histories,
		})(text)
	}
}

impl Puzzle for Oasis {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		for (row, hist) in self.histories.iter_mut().enumerate() {
			hist.compute_derivatives().wrap_err_with(|| {
				eyre::eyre!(
					"cannot compute derivatives for record {rec}",
					rec = row + 1
				)
			})?;
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.histories
			.iter()
			.map(|rec| rec.make_prediction(Prediction::Next))
			.flat_map(Result::ok)
			.reduce(|a, b| a + b)
			.ok_or_else(|| {
				eyre::eyre!("cannot predict from a set of empty oases")
			})
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		if self
			.histories
			.iter()
			.any(|hist| hist.derivatives.is_empty())
		{
			self.prepare_1()?;
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.histories
			.iter()
			.map(|rec| rec.make_prediction(Prediction::Prev))
			.flat_map(Result::ok)
			.reduce(|a, b| a + b)
			.ok_or_else(|| {
				eyre::eyre!("cannot back-predict from a set of empty oases")
			})
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoricalRecord {
	readings: Vec<i64>,
	derivatives: Vec<Vec<i64>>,
}

impl HistoricalRecord {
	pub fn make_prediction(&self, direction: Prediction) -> eyre::Result<i64> {
		let (getter, forecaster): (
			fn(&[i64]) -> Option<&i64>,
			fn(i64, i64) -> i64,
		) = match direction {
			Prediction::Next => (|s: &[i64]| s.last(), |a, b| a + b),
			Prediction::Prev => (|s: &[i64]| s.first(), |a, b| b - a),
		};
		// This is the value being bubbled up the derivative stack.
		let mut bubble = 0;
		for derivative in self.derivatives.iter().rev() {
			let &point = getter(derivative.as_slice()).ok_or_else(|| {
				eyre::eyre!("derivative sequence did not contain any points")
			})?;
			bubble = forecaster(bubble, point);
		}
		getter(self.readings.as_slice())
			.map(|&pt| forecaster(bubble, pt))
			.ok_or_else(|| {
				eyre::eyre!("cannot predict from an empty reading set")
			})
	}

	pub fn compute_derivatives(&mut self) -> eyre::Result<()> {
		self.derivatives.clear();
		// Begin by taking the derivative of the readings.
		let mut sequence: &[i64] = self.readings.as_slice();
		while !sequence.iter().all(|&pt| pt == 0) {
			let derivative = sequence
				.windows(2)
				.flat_map(|pair| match pair {
					&[a, b] => Some(b - a),
					_ => None,
				})
				.collect::<Vec<_>>();
			self.derivatives.push(derivative);
			sequence = self
				.derivatives
				.last()
				.ok_or_else(|| eyre::eyre!("did not compute any derivatives"))?;
		}
		// Remove the all-zero line.
		self.derivatives.pop();
		Ok(())
	}
}

impl<'a> Parsed<&'a str> for HistoricalRecord {
	fn parse_wyz(line: &'a str) -> ParseResult<&'a str, Self> {
		map(
			terminated(
				separated_list1(
					space1,
					map_res(
						recognize(preceded(opt(tag("-")), digit1)),
						|s: &str| s.parse::<i64>(),
					),
				),
				newline,
			),
			|readings| Self {
				readings,
				derivatives: vec![],
			},
		)(line)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Prediction {
	Next,
	Prev,
}
