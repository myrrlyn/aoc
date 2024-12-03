use std::ops::Range;

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
	IResult,
};
use tap::TapFallible;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 5, |t| t.parse_dyn_puzzle::<Lookup>());

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lookup {
	seeds:       Vec<i64>,
	seed_ranges: Vec<Range<i64>>,
	almanac:     Almanac,
}

impl<'a> Parsed<&'a str> for Lookup {
	fn parse_wyz(text: &str) -> ParseResult<&str, Lookup> {
		let (text, seeds) = preceded(
			tag("seeds:"),
			many1(preceded(space1, parse_number::<i64>)),
		)(text)
		.tap_err(|err| tracing::error!(?err, "could not parse the seed list"))?;
		let (text, almanac) = text.trim_start().parse_wyz().tap_err(|err| {
			tracing::error!(?err, "could not parse an almanac")
		})?;
		Ok((text, Self {
			seeds,
			seed_ranges: vec![],
			almanac,
		}))
	}
}

impl Puzzle for Lookup {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.almanac
			.min_location(
				self.seeds
					.iter()
					.copied()
					.inspect(|seed| tracing::debug!(?seed)),
			)
			.ok_or_else(|| eyre::eyre!("had no input seeds"))
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.seed_ranges = self
			.seeds
			.chunks_exact(2)
			.filter_map(|c| match c {
				&[a, b] => Some(a .. a + b),
				_ => None,
			})
			.collect();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.almanac
			.min_location(
				// This is a brutally slow and stupid way to run the solver.
				// Checking every single entry in the range description, which
				// the author helpfully made to be around two billion, is the
				// kind of naïve idiocy that only works because this is a Rust
				// program with a very tight inner loop.
				self.seed_ranges
					.iter()
					.cloned()
					.inspect(
						|Range { start, end }| tracing::debug!(bgn = ?start, ?end, len = end - start),
					)
					.flat_map(|range| range.into_iter()),
			)
			.ok_or_else(|| eyre::eyre!("had no input seeds"))
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Almanac {
	seed_soil: Vec<Relation>,
	soil_fertilizer: Vec<Relation>,
	fertilizer_water: Vec<Relation>,
	water_light: Vec<Relation>,
	light_temperature: Vec<Relation>,
	temperature_humidity: Vec<Relation>,
	humidity_location: Vec<Relation>,
}

impl Almanac {
	pub fn seed_to_location(&self, seed: i64) -> i64 {
		tracing::trace!(from = "seed", into = "soil");
		let soil = Self::lookup(&self.seed_soil, seed);
		tracing::trace!(from = "soil", into = "fertilizer");
		let fertilizer = Self::lookup(&self.soil_fertilizer, soil);
		tracing::trace!(from = "fertilizer", into = "water");
		let water = Self::lookup(&self.fertilizer_water, fertilizer);
		tracing::trace!(from = "water", into = "light");
		let light = Self::lookup(&self.water_light, water);
		tracing::trace!(from = "light", into = "temperature");
		let temperature = Self::lookup(&self.light_temperature, light);
		tracing::trace!(from = "temperature", into = "humidity");
		let humidity = Self::lookup(&self.temperature_humidity, temperature);
		tracing::trace!(from = "humidity", into = "location");
		let location = Self::lookup(&self.humidity_location, humidity);
		tracing::trace!(%seed, %soil, %fertilizer, %water, %light, %temperature, %humidity, %location);
		location
	}

	pub fn min_location(
		&self,
		seeds: impl IntoIterator<Item = i64>,
	) -> Option<i64> {
		seeds
			.into_iter()
			.map(|seed| self.seed_to_location(seed))
			.min()
	}

	fn lookup(maps: &[Relation], key: i64) -> i64 {
		maps.iter()
			.filter_map(|rel| rel.lookup(key))
			.next()
			.unwrap_or(key)
	}
}

impl<'a> Parsed<&'a str> for Almanac {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		fn header<'a>(
			from: &'static str,
			into: &'static str,
		) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
			move |t| {
				delimited(
					delimited(tag(from), tag("-to-"), tag(into)),
					tag(" map:"),
					newline,
				)(t)
			}
		}
		let (text, seed_soil) = preceded(
			header("seed", "soil"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| tracing::error!(?err, "could not parse seed/soil map"))?;
		let (text, soil_fertilizer) = preceded(
			header("soil", "fertilizer"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse soil/fertilizer map")
		})?;
		let (text, fertilizer_water) = preceded(
			header("fertilizer", "water"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse fertilizer/water map")
		})?;
		let (text, water_light) = preceded(
			header("water", "light"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse water/light map")
		})?;
		let (text, light_temperature) = preceded(
			header("light", "temperature"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse light/temperature map")
		})?;
		let (text, temperature_humidity) = preceded(
			header("temperature", "humidity"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse temperature/humidity map")
		})?;
		let (text, humidity_location) = preceded(
			header("humidity", "location"),
			many1(Relation::parse_wyz),
		)(text.trim_start())
		.tap_err(|err| {
			tracing::error!(?err, "could not parse humidity/location map")
		})?;
		Ok((text, Self {
			seed_soil,
			soil_fertilizer,
			fertilizer_water,
			water_light,
			light_temperature,
			temperature_humidity,
			humidity_location,
		}))
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Relation {
	orig: i64,
	dest: i64,
	span: i64,
}

impl Relation {
	fn lookup(&self, key: i64) -> Option<i64> {
		let orig_end = self.orig + self.span;
		if (self.orig .. orig_end).contains(&key) {
			let val = key - self.orig + self.dest;
			return Some(val);
		}
		None
	}
}

impl<'a> Parsed<&'a str> for Relation {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (text, dest) = parse_number(text.trim_start())?;
		let (text, orig) = parse_number(text.trim_start())?;
		let (text, span) = parse_number(text.trim_start())?;
		Ok((text, Self { orig, dest, span }))
	}
}
