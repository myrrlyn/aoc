use std::{
	fmt::{
		self,
		Write as _,
	},
	fs,
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
	fn execute_program(&self) -> eyre::Result<()> {
		let mut cwd = std::env::current_dir()?;
		cwd.extend(&["assets", match self.data {
			Data::Sample => "samples",
			Data::Input => "inputs",
		}]);
		cwd.push(self.year.to_string());
		cwd.push(format!("d{:0>2}.txt", self.day));
		let src_file = cwd.display();
		tracing::debug!(?src_file);

		// Look up the requested solver in the registry
		let (year, day) = (self.year, self.day);
		let make_solver = wyz_aoc::solutions()
			.tap(|registry| tracing::debug!(?registry))
			.get(&year)
			.ok_or_else(|| eyre::eyre!("{year} has no registered solutions"))?
			.get(&day)
			.ok_or_else(|| eyre::eyre!("{}", render_known_puzzles()))
			.wrap_err_with(|| {
				eyre::eyre!("{year}-{day:0>2} has no registered solution")
			})?;
		let source_text = fs::read_to_string(cwd)?;

		// This error map is necessary because nom's default error holds views
		// into the source data, but the error is returned out of this function
		// after the source text is destroyed.
		let (rest, mut solver) = (make_solver)(source_text.as_str())
			.map_err(|err| eyre::eyre!("{err}"))?;
		if !rest.trim().is_empty() {
			tracing::warn!(?rest, "unparsed input remaining");
		}

		if self.step != Step::Two {
			tracing::info!(step = 1, "preparing");
			solver.prepare_1().with_context(|| {
				format!("error preparing {year}-{day:0>2}#1")
			})?;
			tracing::info!(step = 1, "running");
			solver
				.part_1()
				.with_context(|| format!("failure running {year}-{day:0>2}#1"))?
				.tap(|answer| {
					tracing::info!(?year, ?day, part = 1, ?answer, "solved!")
				});
		}
		if self.step != Step::One {
			tracing::info!(step = 2, "preparing");
			solver.prepare_2().with_context(|| {
				format!("error preparing {year}-{day:0>2}#2")
			})?;
			tracing::info!(step = 2, "running");
			solver
				.part_2()
				.with_context(|| format!("failure running {year}-{day:0>2}#2"))?
				.tap(|answer| {
					tracing::info!(?year, ?day, part = 2, ?answer, "solved!")
				});
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
		write!(fmt, "Source::{}", match self {
			Self::Sample => "Sample",
			Self::Input => "Input",
		})
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
					.context("did not provide a year and day")
					.with_context(render_known_puzzles);
			},
			_ => {
				eprintln!("{err:?}");
				return Err(err).context("failed to parse CLI args");
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
		.context("RUST_LOG envvar cannot be parsed as a tracing directive")?;
	tracing_subscriber::registry()
		.with(trace_fmt)
		.with(trace_filt)
		.try_init()
		.context("failed to install a trace sink")?;

	// Dispatch to the solvers!
	args.execute_program()
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
