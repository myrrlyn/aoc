# *Advent of Code* Solution Management

While I structured this project to be able to do AoC puzzles in any language, my
primary is Rust and I don't have any of my pre-Rust solutions (I did 2015 in
Ruby, if I recall correctly) still around. I'm hopeful that I'll use this as an
opportunity to learn other languages by re-solving the puzzles and checking
their work against existing solvers.

AoC *loves* a handful of common themes and strongly encourages pulling
repetitive work out into a support library rather than re-doing that work every
time. For instance, *every* day involves writing a text parser. 2-D Cartesian
graphs happen *a lot*, and it would be beyond foolish to keep rebuilding a
canvas plotter.

## Daily Puzzles

In addition to the support library, we also need to implement the the puzzles
and figure out some way to execute them and get the answers back for submission.
This occurs in the `y*` family of modules.

The module tree `crate::y{year}::d{day}` contains all the individualized solving
logic. When solving a day's puzzles, the leaf module can register itself with
the execution harness by placing its entry point in a global collection.

Rust doesn't have reflection, so I can't have the library root iterate the year
modules, have *those* iterate the day modules, and collect all the entry points
into a single data structure. So rather than manually build up that structure, I
use a "distributed slice" provided by `linkme`. Every module puts its solver
entry point in that segment, the linker collates them all together into a slice,
and the library root knows how to read the slice. The execution harness then
turns the slice into a dispatch table.

Since each puzzle has its own state and behavior, but they all are required to
expose a function for each part of the puzzle which just produces an integer, I
use `dyn Puzzle` trait objects as a common interface.

Because Rust distinguishes between function *items* and function *pointers*, and
function names are typed as items, registration requires writing a do-nothing
closure, like this:

```rust,ignore
#[linkme::distributed_slice(SOLVERS)]
static THIS: Solver = Solver::new(year, day, |t| t.parse_dyn_puzzle::<Today>());
```

The execution harness interacts with solvers exclusively as `Box<dyn Puzzle>`
virtual objects. The `Puzzle` trait has two pairs of methods: preparation and
execution for both part 1 and part 2 of the day's challenge. The trait supplies
default implementations of all four methods, so that once a type is registered,
the harness can begin running immediately. By default, preparation does nothing,
and execution fails saying that the puzzle has not been implemented.

The harness guarantees that whenever a part's main execution runs, its
preparation has already succeeded; however, Part 2 cannot assume that Part 1
has *or has not* run its preparation or solver!
