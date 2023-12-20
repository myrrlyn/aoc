#![doc = include_str!("lib.md")]

use std::{
	collections::BTreeMap,
	env,
	fmt,
	fs,
	ops::RangeInclusive,
	path::PathBuf,
	sync::OnceLock,
};

use eyre::Context;
pub use funty::{
	Integral,
	Signed,
};
use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::digit1,
	combinator::{
		map_res,
		value,
	},
	IResult,
};
use tap::Tap;

pub mod coords;
pub mod dictionary;
pub mod y2015;
pub mod y2016;
pub mod y2017;
pub mod y2018;
pub mod y2019;
pub mod y2020;
pub mod y2021;
pub mod y2022;
pub mod y2023;

pub mod prelude {
	pub use crate::{
		ParseResult,
		Parseable,
		Parsed,
		Puzzle,
		Solver,
		SOLVERS,
	};
}

pub use crate::coords::{
	Cartesian2DPoint as Coord2D,
	Cartesian2DSpace as Grid2D,
	Cartesian3DPoint as Coord3D,
	Cartesian3DSpace as Grid3D,
};

/// The output of the main data parsers.
pub type ParseResult<I, T> = IResult<I, T>;

/// A function which parses puzzle source data and produces a virtualized
/// solver. This allows the execution harness to dispatch into any given day's
/// solver without having concrete knowledge of the puzzle logic.
pub type DynParser =
	for<'a> fn(&'a str) -> ParseResult<&'a str, Box<dyn Puzzle>>;

/// Represents an entry in the puzzle set.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solver {
	/// The year half of the puzzle's identifier.
	pub year: u16,
	/// The day half of the puzzle's identifier.
	pub day:  u8,
	/// The puzzle entry point. This function parses input text and produces an
	/// engine capable of solving the puzzle.
	pub func: DynParser,
}

impl Solver {
	/// Creates a new solver object from its components.
	pub const fn new(year: u16, day: u8, func: DynParser) -> Self {
		Self { year, day, func }
	}

	/// Executes the solver
	#[tracing::instrument(name = "solve", skip(self, group), fields(year=%self.year, day=%self.day))]
	pub fn solve(
		&self,
		group: &str,
		part_1: bool,
		part_2: bool,
	) -> eyre::Result<(Option<i64>, Option<i64>)> {
		let text = self.load_input(group)?;
		tracing::trace!("loaded input");
		for line in text.lines().take(3) {
			tracing::trace!(%line, "input data");
		}

		let (rest, mut solver) = (self.func)(&text)
			.map_err(|err| eyre::eyre!("{err}"))
			.wrap_err("failed to parse input")?;
		if !rest.trim().is_empty() {
			let rest = rest.lines().next().ok_or_else(|| {
				eyre::eyre!("failed to render unparsed input for error message")
			})?;
			let rest = format!("{rest}...");
			tracing::warn!(%rest, "unparsed input remaining");
		}
		let mut one = None;
		if part_1 {
			solver
				.prepare_1()
				.wrap_err("could not prepare for part 1")?;
			one = Some(solver.part_1().wrap_err("could not solve part 1")?);
		}

		let mut two = None;
		if part_2 {
			solver
				.prepare_2()
				.wrap_err("could not prepare for part 2")?;
			two = Some(solver.part_2().wrap_err("could not solve part 2")?);
		}

		Ok((one, two))
	}

	#[tracing::instrument(name = "gather", skip(self))]
	pub fn load_input(&self, group: &str) -> eyre::Result<String> {
		let mut path: PathBuf = ["assets", group].into_iter().collect();
		path.push(self.year.to_string());
		path.push(format!("d{:0>2}.txt", self.day));
		tracing::trace!(file=%path.display(), "generated input path");
		let mut file = env::current_dir()?;
		file.extend(path.components());
		fs::read_to_string(&file)
			.wrap_err_with(|| eyre::eyre!("could not read {}", file.display()))
	}

	/// Parses the input into a solver engine.
	#[tracing::instrument(name = "parse", skip(self))]
	pub fn parse<'a>(
		self,
		text: &'a str,
	) -> ParseResult<&'a str, Box<dyn Puzzle>> {
		(self.func)(text)
	}
}

/// A collection of virtualized puzzle constructors, indexed by year and then
/// day.
pub type Registry = BTreeMap<u16, BTreeMap<u8, DynParser>>;

/// An unsorted collection of daily puzzle solvers.
#[linkme::distributed_slice]
pub static SOLVERS: [Solver];

/// Gets a structured view of all registered days.
pub fn solutions() -> &'static Registry {
	static REGISTRY: OnceLock<Registry> = OnceLock::new();
	REGISTRY.get_or_init(|| {
		SOLVERS.iter().fold(
			Registry::new(),
			|mut accum, &Solver { year, day, func }| {
				accum.entry(year).or_default().insert(day, func);
				accum
			},
		)
	})
}

/// A solver for the day's pair of puzzles.
///
/// Each day's module implements this trait and registers some
/// `(&str) -> Result<Box<dyn Puzzle>>` with the above collection in order for
/// the execution harness to find and run it.
pub trait Puzzle {
	/// Prepares a solver to execute part 1.
	fn prepare_1(&mut self) -> eyre::Result<()> {
		Ok(())
	}

	/// Runs the Part 1 solver.
	///
	/// This is permitted to modify `self`, but generally should not. Part 2
	/// solvers may wish to skip Part 1 if the computation is expensive and not
	/// relevant to Part 2's work.
	fn part_1(&mut self) -> eyre::Result<i64> {
		eyre::bail!("have not yet solved part 1");
	}

	/// Prepares a solver to execute part 2.
	///
	/// By default, this calls `self.prepare_1()`. Overriders should generally
	/// assume that `self.part_1()` has *not* been called
	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.prepare_1()?;
		Ok(())
	}

	/// Runs the Part 2 solver.
	///
	/// The execution harness is not required to call the Part 1 solver before
	/// invoking Part 2! Implementors can *only* assume that `.prepare_2()` has
	/// been called, and must tolerate the part-1 methods being either run *or*
	/// not run.
	fn part_2(&mut self) -> eyre::Result<i64> {
		eyre::bail!("have not yet solved part 2");
	}
}

pub trait Parsed<Input>: Sized {
	/// Parses the input into a fresh instance of `Self`.
	fn parse_wyz(src: Input) -> ParseResult<Input, Self>;

	/// Parses the input structure into a boxed instance of `Self`.
	fn parse_wyz_boxed(src: Input) -> ParseResult<Input, Box<Self>> {
		Self::parse_wyz(src).map(|(rest, this)| (rest, Box::new(this)))
	}

	/// Parses the input into a virtualized instance of `Self`.
	fn parse_dyn_puzzle(src: Input) -> ParseResult<Input, Box<dyn Puzzle>>
	where Self: 'static + Puzzle {
		Self::parse_wyz_boxed(src)
			.map(|(rest, this)| (rest, this as Box<dyn Puzzle>))
	}
}

/// Marks a data source as being parseable into some puzzle input.
pub trait Parseable: Sized {
	/// Parses a data stream into a puzzle input.
	fn parse_wyz<P: Parsed<Self>>(self) -> ParseResult<Self, P> {
		P::parse_wyz(self)
	}

	/// Parses a data stream into a virtualized puzzle object.
	fn parse_dyn_puzzle<P: 'static + Parsed<Self> + Puzzle>(
		self,
	) -> ParseResult<Self, Box<dyn Puzzle>> {
		P::parse_dyn_puzzle(self)
	}
}

/// Allow parsing text streams.
impl Parseable for &str {
}

/// In theory, we could parse binary streams.
impl Parseable for &[u8] {
}

/// Unifies a series of inclusive ranges by joining any that overlap.
pub fn unify_ranges_inclusive<I: Integral>(
	ranges: impl Iterator<Item = RangeInclusive<I>>,
) -> Vec<RangeInclusive<I>> {
	ranges
		.collect::<Vec<_>>()
		.tap_mut(|v| v.sort_by_key(|r| *r.start()))
		.into_iter()
		.fold(Vec::<RangeInclusive<I>>::new(), |mut acc, next| {
			if let Some(prev) = acc.last_mut() {
				let (a1, a2) = (*prev.start(), *prev.end());
				let (b1, b2) = (*next.start(), *next.end());
				if a2 >= b1 {
					*prev = a1.min(b1) ..= a2.max(b2);
					return acc;
				}
			}
			acc.push(next);
			acc
		})
}

/// Parses a sequence of decimal digits into a given numeric primitive.
pub fn parse_number<T: Integral>(text: &str) -> IResult<&str, T>
where <T as TryFrom<i8>>::Error: fmt::Debug {
	map_res(digit1, T::from_str)(text)
}

pub fn written_number<T: Integral>(text: &str) -> IResult<&str, T>
where <T as TryFrom<i8>>::Error: fmt::Debug {
	alt((
		value(T::try_from(0i8).expect("infallible"), tag("zero")),
		value(T::try_from(1i8).expect("infallible"), tag("one")),
		value(T::try_from(2i8).expect("infallible"), tag("two")),
		value(T::try_from(3i8).expect("infallible"), tag("three")),
		value(T::try_from(4i8).expect("infallible"), tag("four")),
		value(T::try_from(5i8).expect("infallible"), tag("five")),
		value(T::try_from(6i8).expect("infallible"), tag("six")),
		value(T::try_from(7i8).expect("infallible"), tag("seven")),
		value(T::try_from(8i8).expect("infallible"), tag("eight")),
		value(T::try_from(9i8).expect("infallible"), tag("nine")),
	))(text)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn written_numbers() -> eyre::Result<()> {
		let text = "onethreefive";
		let (rest, one) = written_number::<i8>(text)?;
		let (rest, three) = written_number::<i8>(rest)?;
		let (rest, five) = written_number::<i8>(rest)?;
		assert!(rest.is_empty());
		assert_eq!(one, 1);
		assert_eq!(three, 3);
		assert_eq!(five, 5);
		Ok(())
	}
}
