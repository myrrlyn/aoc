use nom::{
	bytes::complete::tag,
	character::complete::{
		newline,
		space1,
	},
	multi::many1,
	sequence::{
		delimited,
		preceded,
	},
};
use tap::{
	Pipe,
	Tap,
	TapFallible,
};

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 6, |t| t.parse_dyn_puzzle::<Races>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Races {
	races: Vec<Race>,
}

impl Races {
	fn execute(&self) -> eyre::Result<i64> {
		self.races
			.iter()
			.map(move |&Race { time, dist }| {
				(1 .. time)
					.map(move |speed| speed * (time - speed))
					.filter(move |&d| d > dist)
					.count()
					.tap(|ct| tracing::debug!(?ct, ?dist, "ways to win"))
			})
			.product::<usize>()
			.pipe(|v| v as i64)
			.pipe(Ok)
	}
}

impl<'a> Parsed<&'a str> for Races {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (text, times) = delimited(
			tag("Time:"),
			many1(preceded(space1, parse_number)),
			newline,
		)(text)
		.tap_err(|err| tracing::error!(?err, "could not parse race times"))?;
		let (text, dists) = delimited(
			tag("Distance:"),
			many1(preceded(space1, parse_number)),
			newline,
		)(text)
		.tap_err(|err| {
			tracing::error!(?err, "could not parse race distances")
		})?;
		let races = times
			.into_iter()
			.zip(dists.into_iter())
			.map(|(time, dist)| Race { time, dist })
			.collect();
		Ok((text, Self { races }))
	}
}

impl Puzzle for Races {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		eyre::ensure!(!self.races.is_empty(), "did not find any race data");
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.execute()
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		let (time, dist) = self.races.iter().copied().fold(
			(String::new(), String::new()),
			|(mut times, mut dists), Race { time, dist }| {
				times.push_str(&format!("{time}"));
				dists.push_str(&format!("{dist}"));
				(times, dists)
			},
		);
		let time = time.parse::<i64>()?;
		let dist = dist.parse::<i64>()?;
		self.races = vec![Race { time, dist }];
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.execute()
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Race {
	time: i64,
	dist: i64,
}
