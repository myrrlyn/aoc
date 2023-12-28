use std::fmt::{
	self,
	Write as _,
};

use clap::{
	error::ErrorKind,
	Parser,
	ValueEnum,
};
use eyre::WrapErr as _;
use tap::Tap;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use wyz_aoc::Solver;

/** Runs an Advent of Code solution.

This harness expects to load puzzle data from the well-known filesystem tree in
`assets/`, and expects to be run from the project root, **not** the Rust harness
root.

It is capable of selecting either, or both, of a day's puzzles.

Days become selectable when the module `y{year}::d{day}` registers a parser with
the harness' dispatch calendar. That parser is responsible for consuming puzzle
input and producing a `dyn Puzzle` solver, which is then invoked according to
the CLI input.

All messages are emitted through `tracing` events, including the solutions.
Solutions are delivered at the INFO level. ERROR and WARNING should only appear
when a puzzle is not correctly built for the input. Enable DEBUG or TRACE to
observe the solvers in action.
 */
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Parser)]
#[command(author, version, about)]
pub struct Args {
	/// The desired puzzle year.
	year:   u16,
	/// The desired puzzle day.
	day:    u8,
	/// Whether to use the sample or real input data.
	#[arg(short, long, value_enum, default_value_t)]
	data:   Data,
	/// Which step(s) to run.
	#[arg(short, long, value_enum, default_value_t)]
	step:   Step,
	/// How to render trace messages
	#[arg(short, long, value_enum, default_value_t)]
	format: TraceFormat,
}

impl Args {
	#[tracing::instrument(name = "run", skip(self), fields(year=%self.year, day=%self.day))]
	fn execute_program(&self) -> eyre::Result<()> {
		let span = tracing::error_span!("lookup");
		let span = span.enter();
		// Look up the requested solver in the registry
		let (year, day) = (self.year, self.day);
		let solution = wyz_aoc::solutions()
			.get(&year)
			.and_then(|y| y.get(&day))
			.ok_or_else(|| eyre::eyre!("{}", render_known_puzzles()))
			.wrap_err_with(|| {
				eyre::eyre!("{year}-{day:0>2} has no registered solution")
			})?;
		let solver = Solver::new(self.year, self.day, *solution);
		tracing::trace!("found solver");
		drop(span);

		let source_text = solver.load_input(match self.data {
			Data::Sample => "samples",
			Data::Input => "inputs",
		})?;

		let span = tracing::error_span!("solve");
		let _span = span.enter();
		tracing::trace!("parsing");
		// This error map is necessary because nom's default error holds views
		// into the source data, but the error is returned out of this function
		// after the source text is destroyed.
		let (rest, mut solver) = solver
			.parse(source_text.as_str())
			.map_err(|err| eyre::eyre!("{err}"))?;
		if !rest.trim().is_empty() {
			tracing::warn!(?rest, "unparsed input remaining");
		}
		solver.after_parse().wrap_err(
			"input was successfully parsed, but was not valid for the rules of \
			 the puzzle",
		)?;

		if self.step != Step::Two {
			let span = tracing::error_span!("", part = 1);
			let _span = span.enter();
			tracing::info!("preparing");
			solver.prepare_1().wrap_err_with(|| {
				format!("error preparing {year}-{day:0>2}#1")
			})?;
			tracing::info!("running");
			solver
				.part_1()
				.wrap_err_with(|| format!("failure running {year}-{day:0>2}#1"))?
				.tap(|answer| tracing::info!(?answer, "solved!"));
		}
		if self.step != Step::One {
			let span = tracing::error_span!("", part = 2);
			let _span = span.enter();
			tracing::info!("preparing");
			solver.prepare_2().wrap_err_with(|| {
				format!("error preparing {year}-{day:0>2}#2")
			})?;
			tracing::info!("running");
			solver
				.part_2()
				.wrap_err_with(|| format!("failure running {year}-{day:0>2}#2"))?
				.tap(|answer| tracing::info!(?answer, "solved!"));
		}

		Ok(())
	}
}

#[derive(
	Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, ValueEnum,
)]
pub enum Data {
	#[default]
	Sample,
	Input,
}

impl fmt::Display for Data {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, fmt)
	}
}

#[derive(
	Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, ValueEnum,
)]
pub enum Step {
	One,
	Two,
	#[default]
	All,
}

impl fmt::Display for Step {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, fmt)
	}
}

#[derive(
	Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, ValueEnum,
)]
pub enum TraceFormat {
	Compact,
	#[default]
	Plain,
	Pretty,
	Json,
}

impl fmt::Display for TraceFormat {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, fmt)
	}
}

fn main() -> eyre::Result<()> {
	color_eyre::install()?;

	// Get the CLI args
	let args = match Args::try_parse() {
		Ok(args) => args,
		Err(err) => match err.kind() {
			// These are not a failed run
			ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
				err.print()?;
				println!("\n{}", render_known_puzzles());
				return Ok(());
			},
			ErrorKind::MissingRequiredArgument => {
				return Err(err)
					.wrap_err("did not provide a year and day")
					.wrap_err_with(render_known_puzzles);
			},
			_ => {
				eprintln!("{err:?}");
				return Err(err).wrap_err("failed to parse CLI args");
			},
		},
	};

	// Install the tracing sinks
	let trace_fmt = tracing_subscriber::fmt::layer()
		.with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339());
	let trace_fmt = match args.format {
		TraceFormat::Compact => trace_fmt.compact().boxed(),
		TraceFormat::Plain => trace_fmt.boxed(),
		TraceFormat::Pretty => trace_fmt.pretty().boxed(),
		TraceFormat::Json => trace_fmt.json().boxed(),
	};
	let trace_filt = tracing_subscriber::EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env()
		.wrap_err("RUST_LOG envvar cannot be parsed as a tracing directive")?;
	tracing_subscriber::registry()
		.with(trace_fmt)
		.with(trace_filt)
		.try_init()
		.wrap_err("failed to install a trace sink")?;

	// Dispatch to the solvers! *Off* the main thread, just in case I ever
	// figure out how to do window drawings.
	let handle = std::thread::spawn(move || args.execute_program());
	handle
		.join()
		.map_err(|_| eyre::eyre!("solver thread panicked"))?
}

fn render_known_puzzles() -> String {
	let mut show = String::new();
	writeln!(&mut show, "Known solutions are:").ok();
	for (year, days) in wyz_aoc::solutions() {
		let mut days = days.keys();
		if let Some(day) = days.next() {
			write!(&mut show, "- y{year}: d{day:0>2}").ok();
		}
		for day in days {
			write!(&mut show, ", d{day:0>2}").ok();
		}
		writeln!(&mut show).ok();
	}
	write!(
		&mut show,
		"Do not use the `y` or `d` prefixes when providing arguments."
	)
	.ok();
	show
}
