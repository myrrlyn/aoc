# Advent of Rust

This section is an exercise not only in my ability to express algorithms, but
also how to structure and outfit an executable program.

The project is split into three main parts: the core library, the execution
harness, and the puzzle solvers.

## Core Library

This holds the common glue used to allow the harness to reach individual
solvers, as well as logic that can be hoisted out of any particular day and made
fit for re-use.

### Parsing

Each solver must implement `Parsed<&str>` with the grammar that transforms the
source data into the initial state, and `Puzzle` with the logic that solves
the state for submission.

Once a solver has these implementations, it can add `Self::parse_dyn_puzzle`
to the global registry and become available for dispatch from the harness.
Registration is done by the `#[linkme::distributed_slice(SOLVERS)]` attribute
on a `(year, day, fn(&str) -> Result<(&str, Self), _>)` static record.

### Solving

The harness calls `.prepare_N()` and `.part_N()`, for `N=1` and/or `N=2`, on the
solver selected by CLI args. Solvers *must* implement the `.part_N` trait
methods, and *may* implement `.prepare_N`, for the harness to successfully run
them.

## Execution Harness

The executable crate sets up a tracing environment and parses command-line
arguments to determine which day’s solver to run and on which data, then invokes
the solver. It expects its working directory to be the repository root, so that
it can discover the source data files in `../assets/`.

## Solvers

Solvers are individual modules in the `src/y{year}/d{day}.rs` tree. Each
implements the interfaces defined in the core library, and can be selected by
the harness.

When I notice that multiple puzzles use the same general principles, I’ll try
to lift that logic out of the individual days’ solvers and into the core
library.

## Project-Root Workspace

This is just a convenience to allow Cargo to discover the project while
operating out of the repository root.
