#![doc = include_str!("d16.md")]

use std::{
	fmt,
	sync::atomic::{
		AtomicU8,
		AtomicUsize,
		Ordering,
	},
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::newline,
	combinator::{
		map,
		value,
	},
	multi::{
		many1,
		separated_list1,
	},
};
use rayon::{
	prelude::*,
	Scope,
};

use crate::{
	coords::{
		points::Direction2D,
		spaces::DisplayGrid,
		Cartesian2DPoint as Point2D,
		Dense2DSpace,
	},
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 16, |t| t.parse_dyn_puzzle::<LightGrid>());

/// A dense grid of tiles which may contain light-beam manipulators and/or light
/// beams.
#[derive(Clone, Debug, Default)]
pub struct LightGrid {
	grid: Dense2DSpace<i8, Tile>,
}

impl LightGrid {
	/// Counts how many tiles in the grid have *any number* of beams passing
	/// through them.
	pub fn count_illuminated(&self) -> i64 {
		self.grid
			.iter()
			.filter(|(_, tile)| tile.beams.count() > 0)
			.count() as i64
	}

	/// Erases all beams from the grid.
	pub fn clear(&mut self) {
		self.grid
			.iter()
			.par_bridge()
			.for_each(|(_, tile)| tile.beams.clear());
	}

	/// Walks a beam through the grid.
	pub fn walk(&mut self, beam: Beam) {
		let id = AtomicUsize::new(1);
		let grid = &self.grid;
		// Keep this call frame alive until all possible child beams are done
		// with their walks.
		rayon::scope(|scope| {
			// This closure is running on a worker in Rayon's thread-pool, so it
			// can just run the search directly rather than spawn a search task
			// and quit (which would cause the current worker to place the search
			// in the thread-pool queue and then immediately try to grab it back
			// off the queue, which is wasted effort).
			beam.walk(&id, scope, grid);
		});
		tracing::debug!(ct=%id.load(Ordering::Relaxed), "grid filled");
	}
}

impl<'a> Parsed<&'a str> for LightGrid {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(
				newline,
				many1(map(TileKind::parse_wyz, Tile::from)),
			),
			|table| Self {
				grid: Dense2DSpace::from_raw(Point2D::ZERO, table),
			},
		)(text)
	}
}

impl Puzzle for LightGrid {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.walk(Beam::new(Point2D::ZERO, Direction2D::East));
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		tracing::info!("\n{:}", self.display());
		Ok(self.count_illuminated())
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		let mut blank = self.clone();
		blank.clear();
		let (min, max) = self
			.grid
			.dimensions()
			.ok_or_else(|| eyre::eyre!("cannot process an empty hall"))?;

		// We need to simulate a light-beam that begins at each possible entry
		// slot. This means constructing beams on every edge tile, facing
		// inwards from that edge.
		let top_row = (min.x ..= max.x)
			.map(|x| Beam::new(Point2D::new(x, 0), Direction2D::South));
		// But we can skip (0, 0)/East, since part 1 already did that.
		let left_col = ((min.y + 1) ..= max.y)
			.map(|y| Beam::new(Point2D::new(0, y), Direction2D::East));
		let right_col = (min.y ..= max.y)
			.map(|y| Beam::new(Point2D::new(max.x, y), Direction2D::West));
		let bottom_row = (min.x ..= max.x)
			.map(|x| Beam::new(Point2D::new(x, max.y), Direction2D::North));

		// Now we can spawn a worker for every beam, propagate it through a
		// blank grid, and keep the brightest grid. A sequential search *could*
		// get away with re-uisng the same allocation and wiping it after each
		// search, but it's easier to just clone a blank grid...
		//
		// ... and since the `.map()` body doesn't have any captured external
		// state to modify, we can just drop in a `.into_par_iter()` and spray
		// the workers across every core in the machine.
		//
		// Since the allocator is global, we pre-allocate maps for each starting
		// seed before beginning the search, reducing system contention while
		// the search is underway.
		let best = top_row
			.chain(left_col)
			.chain(right_col)
			.chain(bottom_row)
			.map(|beam| (beam, blank.clone()))
			.collect::<Vec<_>>()
			.into_par_iter()
			.map(|(beam, mut grid)| {
				let span = tracing::debug_span!("search", %beam);
				let _span = span.enter();
				grid.walk(beam);
				tracing::debug!(score=%grid.count_illuminated(), "done");
				grid
			})
			.max_by_key(LightGrid::count_illuminated)
			.ok_or_else(|| {
				eyre::eyre!("could not compute any other sequences")
			})?;
		if best.count_illuminated() > self.count_illuminated() {
			*self = best;
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		tracing::info!("\n{:}", self.display());
		Ok(self.count_illuminated())
	}
}

impl DisplayGrid<i8, Tile> for LightGrid {
	fn bounds_inclusive(&self) -> Option<(Point2D<i8>, Point2D<i8>)> {
		self.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i8,
		col: i8,
		_row_abs: usize,
		_col_abs: usize,
	) -> char {
		match self.grid.get(Point2D::new(col, row)) {
			None => symbols.empty,
			Some(t) => match t.kind {
				TileKind::Void => [
					symbols.empty,
					symbols.quarter_1,
					symbols.quarter_2,
					symbols.quarter_3,
					symbols.full,
				][t.beams.count()],
				_ => t.kind.symbol(),
			},
		}
	}
}

/// A search cursor which can propagate through a `LightGrid`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Beam {
	pos: Point2D<i8>,
	dir: Direction2D,
}

impl Beam {
	pub fn new(point: Point2D<i8>, direction: Direction2D) -> Self {
		Self {
			pos: point,
			dir: direction,
		}
	}

	/// Propagates through a `LightGrid`.
	///
	/// Splitters cause the current beam to turn one direction, and a new beam
	/// to turn the other direction. The beams can propagate independently of
	/// each other, as long as they are able to share awareness of the overall
	/// grid, so the `scope` argument allows this to spawn new beam-search
	/// workers when encountering a splitter.
	///
	/// The `id` argument is only for being able to distinguish workers in the
	/// debug traces.
	fn walk<'a, 'scope: 'a>(
		mut self,
		id: &'scope AtomicUsize,
		scope: &'a Scope<'scope>,
		grid: &'scope Dense2DSpace<i8, Tile>,
	) {
		let my_id = id.load(Ordering::Relaxed);
		let span = tracing::debug_span!("walk", %my_id, bgn=%self);
		let _span = span.enter();
		tracing::trace!("begin");
		loop {
			// Exiting the grid stops the walk.
			if !grid.in_bounds(self.pos) {
				tracing::trace!(pos=%self.pos, dir=%self.dir, "exit");
				break;
			}
			let tile = &grid[self.pos];
			// If the grid already contains a beam at this location going this
			// direction, then we have produced a cycle, and can stop the walk.
			if tile.beams.insert(self.dir) {
				tracing::trace!(pos=%self.pos, dir=%self.dir, "cycle");
				break;
			}
			match tile.kind {
				// Void tiles do not turn the beam
				TileKind::Void => {},
				// Mirrors turn the beam according to their name: for each pair
				// in the name, a beam traveling on one direction in the pair
				// switches to the other at the mirror.
				TileKind::MirrorNwSe => {
					self.dir = match self.dir {
						Direction2D::North => Direction2D::West,
						Direction2D::South => Direction2D::East,
						Direction2D::West => Direction2D::North,
						Direction2D::East => Direction2D::South,
					}
					// Unlike splitters (see below), mirrors do not immediately
					// restart the loop, but allow the cursor to step away. This
					// is important, because restarting the loop would cause the
					// mirror to be marked as having a turned beam in it, and
					// that is not correct. Beams can only be marked as present
					// on *ingress*, otherwise a beam *exiting* on one side of
					// the tile would be indistinguishable from a beam
					// *entering* on the other side.
				},
				TileKind::MirrorSwNe => {
					self.dir = match self.dir {
						Direction2D::North => Direction2D::East,
						Direction2D::South => Direction2D::West,
						Direction2D::West => Direction2D::South,
						Direction2D::East => Direction2D::North,
					}
				},
				// Splitters turn the beam 90-deg in one direction, and also
				// spawn a new beam which is turned 90-deg in the *other*
				// direction! However, they only do this when the beam strikes
				// their side, rather than their end.
				TileKind::SplitterNS => match self.dir {
					Direction2D::West | Direction2D::East => {
						let other = Self {
							pos: self.pos,
							dir: Direction2D::North,
						};
						id.fetch_add(1, Ordering::Relaxed);
						scope.spawn(move |s| other.walk(id, s, grid));
						self.dir = Direction2D::South;
						// Each beam turns *in place*, and does not step away
						// from the splitter here. They step away from the
						// splitter in the *next* loop cycle, when the empty
						// branch matches instead of this one.
						continue;
					},
					Direction2D::North | Direction2D::South => {},
				},
				TileKind::SplitterWE => match self.dir {
					Direction2D::North | Direction2D::South => {
						let other = Self {
							pos: self.pos,
							dir: Direction2D::West,
						};
						id.fetch_add(1, Ordering::Relaxed);
						scope.spawn(move |s| other.walk(id, s, grid));
						self.dir = Direction2D::East;
						continue;
					},
					Direction2D::West | Direction2D::East => {},
				},
			}
			self.pos += self.dir.unit();
		}
	}
}

impl fmt::Display for Beam {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.pos, fmt)?;
		fmt.write_str("/")?;
		fmt::Display::fmt(&self.dir, fmt)
	}
}

/// A tile in the light grid.
#[derive(Clone, Debug, Default)]
pub struct Tile {
	/// What structure is on the tile.
	kind:  TileKind,
	/// Any beams which pass through the tile.
	beams: BeamSet,
}

impl From<TileKind> for Tile {
	fn from(kind: TileKind) -> Self {
		Self {
			kind,
			beams: BeamSet::default(),
		}
	}
}

/// A possible structure placed somewhere in the light grid.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TileKind {
	/// An empty tile. Beams propagate normally.
	#[default]
	Void,
	/// A `╲` mirror. Beams make one 90-deg turn.
	MirrorNwSe,
	/// A `╱` mirror. Beams make one 90-deg turn.
	MirrorSwNe,
	/// A tee junction. Beams are either unaffected (travelling parallel to
	/// splitter) or fork and make *both* 90-deg turns.
	SplitterNS,
	/// A tee junction. Beams are either unaffected (travelling parallel to
	/// splitter) or fork and make *both* 90-deg turns.
	SplitterWE,
}

impl TileKind {
	fn symbol(self) -> char {
		match self {
			Self::Void => ' ',
			Self::MirrorNwSe => '╲',
			Self::MirrorSwNe => '╱',
			Self::SplitterNS => '│',
			Self::SplitterWE => '─',
		}
	}
}

impl<'a> Parsed<&'a str> for TileKind {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Void, tag(".")),
			value(Self::MirrorNwSe, tag("\\")),
			value(Self::MirrorSwNe, tag("/")),
			value(Self::SplitterNS, tag("|")),
			value(Self::SplitterWE, tag("-")),
		))(text)
	}
}

/// Tracks each direction that a beam can travel.
///
/// This wraps an `AtomicU8` so that multiple beams can update the set in
/// parallel. Each direction is a bit-flag in the storage.
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct BeamSet {
	inner: AtomicU8,
}

impl BeamSet {
	/// Marks a direction of travel as having a beam passing through it.
	///
	/// Returns whether a beam traveling in that direction was already present,
	/// so that the worker can detect when it has encountered a path cycle.
	fn insert(&self, dir: Direction2D) -> bool {
		match dir {
			Direction2D::North => {
				self.inner.fetch_or(1, Ordering::Relaxed) & 1 != 0
			},
			Direction2D::South => {
				self.inner.fetch_or(2, Ordering::Relaxed) & 2 != 0
			},
			Direction2D::West => {
				self.inner.fetch_or(4, Ordering::Relaxed) & 4 != 0
			},
			Direction2D::East => {
				self.inner.fetch_or(8, Ordering::Relaxed) & 8 != 0
			},
		}
	}

	/// Erases the stored beams.
	fn clear(&self) {
		self.inner.store(0, Ordering::Relaxed);
	}

	/// Counts how many beams are passing through the set.
	///
	/// The renderer makes a tile more saturated the more beams it has.
	fn count(&self) -> usize {
		self.inner.load(Ordering::Relaxed).count_ones() as usize
	}
}

impl Clone for BeamSet {
	fn clone(&self) -> Self {
		Self {
			inner: AtomicU8::new(self.inner.load(Ordering::Relaxed)),
		}
	}
}
