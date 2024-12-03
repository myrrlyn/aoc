# myrrlyn Advent of Code

This is where I store all my Advent of Code work.

## New Structure

(Migration in progress)

I am working on re-architecting to favor a single common library and a single
execution harness, with a module per day that plugs into the executor and can be
dynamically selected. I'd also like to be able to consider using more than one
language as possible solvers, and to keep input files next to the
implementations, so the filesystem is laid out as:

- `src/`: most languages either implicitly, or can be told to, read this as the
  compilation root

  - `y{year}/`: parent module for all the puzzles in a given year
    - `d{day}/`: child module for any implementations of that day
      - `input.txt`: real puzzle data, not committed
      - `sample.txt`: sample puzzle data, committed for test usage
      - solution file(s)

## Rust

This is my primary language, plus I didn’t have very much of a theoretical CS
education, so brute-forcing my way through solutions is somewhat less painful
with a fast runtime!

From the project workspace, `cargo run --` launches the Rust execution harness.
It prints out the CLI it expects as well as all the puzzles currently known to
it. `cargo run -- someyear someday` runs the corresponding solver. The other
switches (`--step one|two|all`, `--data sample|input`,
`--format compact|plain|pretty|json`) control which solvers are run on which
data, and how it is rendered to the console.

Don’t forget to use `cargo run --release` on some days! Some of my choices are
grindingly slow without that.

## Old Structure

- `years/`: Each year gets a folder in here
  - `<a year>/`
    - `d{01..25}/`: Every day gets its own folder
      - `<a language>/`: Each language used to solve the day gets _its_ own
        folder.
      - `input.txt`: Puzzle input, mandatory
      - `sample.txt`: Sample input, optional
- `<a language>/`: Each language used to solve _any_ puzzle gets its own
  top-level folder used as a common library across the entire project. This is
  supposed to enable code re-use between puzzles.

Packages will be named in the form `wyz_aoc_<year>_d<day>`.
