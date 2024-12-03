#![doc = include_str!("README.md")]

use std::{
	collections::VecDeque,
	fmt,
	mem,
};

use tap::Pipe;

use crate::{
	coords::spaces::Sparse2D,
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 10, |t| t.parse_dyn_puzzle::<Plumbing>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plumbing {
	/// A 2D map where each cell contains a Tile marker and a step-count from
	/// an origin.
	///
	/// The map is in quadrant IV: North is lesser than South; West is lesser
	/// than East.
	map: Sparse2D<i16, Tile>,
}

impl Plumbing {
	/// Gets tiles whose pipes are connected to the provided point.
	fn find_connections(
		&self,
		coord: Coord2D<i16>,
	) -> [Option<Coord2D<i16>>; 4] {
		// It is not enough for the neighbor to point at the current; the
		// current must also point at the neighbor.
		//
		// `Start` points in all four directions until proven otherwise.
		let this = self.map.get(coord);
		let [north, south, west, east] = self.find_neighbors(coord);
		[
			this.zip(north.and_then(|pt| self.map.get(pt))).and_then(
				|(this, that)| match (this.sym, that.sym) {
					(
						Symbol::Start
						| Symbol::NorthSouth
						| Symbol::NorthEast
						| Symbol::NorthWest,
						Symbol::NorthSouth
						| Symbol::SouthEast
						| Symbol::SouthWest,
					) => north,
					_ => None,
				},
			),
			this.zip(south.and_then(|pt| self.map.get(pt))).and_then(
				|(this, that)| match (this.sym, that.sym) {
					(
						Symbol::Start
						| Symbol::NorthSouth
						| Symbol::SouthEast
						| Symbol::SouthWest,
						Symbol::NorthSouth
						| Symbol::NorthEast
						| Symbol::NorthWest,
					) => south,
					_ => None,
				},
			),
			this.zip(west.and_then(|pt| self.map.get(pt))).and_then(
				|(this, that)| match (this.sym, that.sym) {
					(
						Symbol::Start
						| Symbol::EastWest
						| Symbol::NorthWest
						| Symbol::SouthWest,
						Symbol::EastWest | Symbol::NorthEast | Symbol::SouthEast,
					) => west,
					_ => None,
				},
			),
			this.zip(east.and_then(|pt| self.map.get(pt))).and_then(
				|(this, that)| match (this.sym, that.sym) {
					(
						Symbol::Start
						| Symbol::EastWest
						| Symbol::NorthEast
						| Symbol::SouthEast,
						Symbol::EastWest | Symbol::NorthWest | Symbol::SouthWest,
					) => east,
					_ => None,
				},
			),
		]
	}

	/// Gets a list of direct (non-diagonal) neighbor points that are in the
	/// map. The list is ordered as `[N, S, W, E]`.
	fn find_neighbors(&self, coord: Coord2D<i16>) -> [Option<Coord2D<i16>>; 4] {
		[
			coord - Coord2D::new(0, 1),
			coord + Coord2D::new(0, 1),
			coord - Coord2D::new(1, 0),
			coord + Coord2D::new(1, 0),
		]
		.map(|pt| {
			if self.map.contains(pt) {
				Some(pt)
			}
			else {
				None
			}
		})
	}
}

impl<'a> Parsed<&'a str> for Plumbing {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		text.lines()
			.enumerate()
			.flat_map(|(row, line)| {
				line.char_indices().map(move |(col, sym)| {
					(Coord2D::new(col as i16, row as i16), Tile::new(sym.into()))
				})
			})
			.collect::<Sparse2D<i16, Tile>>()
			.pipe(|map| Ok(("", Self { map })))
	}
}

impl Puzzle for Plumbing {
	/// Find the starting tile, then walk the map counting traversed distance
	/// from the start.
	fn prepare_1(&mut self) -> eyre::Result<()> {
		let mut pending = VecDeque::new();
		let origin = self
			.map
			.iter()
			.find(|(_, &Tile { sym, .. })| sym == Symbol::Start)
			.map(|(coord, _)| coord)
			.ok_or_else(|| eyre::eyre!("no starting point found"))?;
		pending.push_back(origin);
		while let Some(point) = pending.pop_front() {
			let &Tile {
				distance: my_distance,
				..
			} = self.map.get(point).ok_or_else(|| {
				eyre::eyre!("unexpectedly absent tile at {point}")
			})?;
			for neighbor in self.find_connections(point).into_iter().flatten() {
				let Tile { distance: dist, .. } =
					self.map.get_mut(neighbor).ok_or_else(|| {
						eyre::eyre!("unexpectedly absent tile at {neighbor}")
					})?;
				// Each point can only be visited once.
				if *dist != 0 {
					continue;
				}
				*dist = my_distance + 1;
				pending.push_back(neighbor);
			}
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.map
			.iter()
			.map(|(_, &Tile { distance: ct, .. })| ct)
			.max()
			.map(|val| val as i64)
			.ok_or_else(|| eyre::eyre!("could not traverse the map"))
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		// Ensure that the main loop has been marked.
		self.prepare_1()?;

		// Replace the Start point by a real pipe marker.
		let origin = self
			.map
			.iter()
			.find(|(_, &Tile { sym, .. })| sym == Symbol::Start)
			.map(|(pt, _)| pt)
			.ok_or_else(|| eyre::eyre!("missing origin tile"))?;
		let connections = self.find_connections(origin);
		let origin = self
			.map
			.get_mut(origin)
			.expect("cannot fail to look up the just-discovered origin");
		// We don't care about distance anymore; this just needs to not be zero.
		origin.distance = -1;
		origin.sym = match connections {
			[Some(_), Some(_), None, None] => Symbol::NorthSouth,
			[Some(_), None, Some(_), None] => Symbol::NorthWest,
			[Some(_), None, None, Some(_)] => Symbol::NorthEast,
			[None, Some(_), Some(_), None] => Symbol::SouthWest,
			[None, Some(_), None, Some(_)] => Symbol::NorthEast,
			[None, None, Some(_), Some(_)] => Symbol::EastWest,
			_ => eyre::bail!("connection matrix must have exactly two links"),
		};

		// Inflate the map: double its dimensions, and in-fill the new points
		// by propagating the main loop.
		let mut new_map = Sparse2D::new();
		for (Coord2D { x, y }, tile @ Tile { sym, distance, .. }) in
			mem::take(&mut self.map).into_iter()
		{
			let new_pt = Coord2D::new(x * 2, y * 2);
			new_map.insert(new_pt, tile);
			// The tile to the south-east is always empty.
			new_map.insert(new_pt + Coord2D::new(1, 1), Tile::default());
			// If the current tile is off the main loop, in-fill emptiness.
			if distance == 0 {
				new_map.insert(new_pt + Coord2D::new(1, 0), Tile::default());
				new_map.insert(new_pt + Coord2D::new(0, 1), Tile::default());
				continue;
			}
			// If it is on the main loop, then the pipe component needs to be
			// stretched. If it points east, then the next point east is an
			// EastWest pipe; if it points south, then the next point south is a
			// NorthSouth pipe. Otherwise, in-fill emptiness.
			new_map.insert(new_pt + Coord2D::new(1, 0), match sym {
				Symbol::EastWest | Symbol::NorthEast | Symbol::SouthEast => {
					Tile {
						sym: Symbol::EastWest,
						..tile
					}
				},
				_ => Tile::default(),
			});
			new_map.insert(new_pt + Coord2D::new(0, 1), match sym {
				Symbol::NorthSouth | Symbol::SouthEast | Symbol::SouthWest => {
					Tile {
						sym: Symbol::NorthSouth,
						..tile
					}
				},
				_ => Tile::default(),
			});
		}

		self.map = new_map;

		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		// Starting from the rim *beyond* the map, seek inwards until finding a
		// section of the main loop that is fully perpendicular to the direction
		// of travel.
		let (min, max) = self
			.map
			.dimensions()
			.ok_or_else(|| eyre::eyre!("cannot analyze an empty map"))?;
		let mut pending = VecDeque::new();
		for col in (min.x - 1) ..= (max.x + 1) {
			pending.push_back(Coord2D::new(col, min.y - 1));
			pending.push_back(Coord2D::new(col, max.y + 1));
		}
		for row in (min.y - 1) ..= (max.y + 1) {
			pending.push_back(Coord2D::new(min.x - 1, row));
			pending.push_back(Coord2D::new(max.x + 1, row));
		}

		while let Some(current_pt) = pending.pop_front() {
			// Skip processing points on the main loop. This stops the search
			// propagation *after* they have been marked as reachable.
			if self
				.map
				.get(current_pt)
				.map(|t| t.distance != 0)
				.unwrap_or(false)
			{
				continue;
			}

			// Search all four neighbors. This allows the search to propagate
			// sideways after moving through a narrow channel into a wider
			// chamber.
			for neighbor in self.find_neighbors(current_pt).into_iter().flatten()
			{
				// If the neighbor doesn't exist in the map (happens immediately
				// when moving inwards from the edges), skip that point.
				let Some(tile) = self.map.get_mut(neighbor)
				else {
					continue;
				};
				// If the search has not yet visited the neighbor, add it to the
				// queue, and mark it as visited to prevent multiple inclusion.
				if !tile.reachable {
					pending.push_back(neighbor);
					tile.reachable = true;
				}
			}
		}

		// Deflate the map.
		self.map = mem::take(&mut self.map)
			.into_iter()
			.filter(|(Coord2D { x, y }, _)| x % 2 == 0 && y % 2 == 0)
			.map(|(Coord2D { x, y }, tile)| (Coord2D::new(x / 2, y / 2), tile))
			.collect();

		eprint!("{self:#}");

		// Reachable points on the original map are:
		// - reachable on the current map
		// - have X and Y coordinates in
		self.map
			.iter()
			.filter(
				|&(
					_,
					&Tile {
						reachable,
						distance,
						..
					},
				)| { !reachable && distance == 0 },
			)
			.count()
			.pipe(|val| Ok(val as i64))
	}
}

impl fmt::Display for Plumbing {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let (min, max) = match self.map.dimensions() {
			Some(val) => val,
			None => return Ok(()),
		};
		for (_, cols) in self.map.raw_data() {
			for col in (min.x) ..= (max.x) {
				match cols.get(&col) {
					Some(&Tile {
						sym,
						reachable,
						distance,
					}) => {
						// Under {:#}, clobber non-main-loop tiles
						if fmt.alternate()
							&& distance == 0 && sym != Symbol::Start
						{
							fmt.write_str(if reachable { " " } else { "█" })?;
						}
						else {
							fmt::Display::fmt(&sym, fmt)?;
						}
					},
					None => fmt.write_str(" ")?,
				}
			}
			writeln!(fmt)?;
		}
		Ok(())
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tile {
	sym:       Symbol,
	distance:  i16,
	reachable: bool,
}

impl Tile {
	pub fn new(sym: Symbol) -> Self {
		Self {
			sym,
			..Self::default()
		}
	}
}

/// A section of the map that may or may not contain some pipe routing.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Symbol {
	/// This section has no pipe on it.
	#[default]
	Empty,
	/// This section is the starting point, and has an unknown shape.
	Start,
	/// This tile is connected to its northern and southern neighbors.
	NorthSouth,
	/// This tile is connected to its eastern and western neighbors.
	EastWest,
	/// This tile is connected to the north and east.
	NorthEast,
	/// This tile is connected to the north and west.
	NorthWest,
	/// This tile is connected to the south and east.
	SouthEast,
	/// This tile is connected to the south and west.
	SouthWest,
}

impl From<char> for Symbol {
	fn from(c: char) -> Self {
		match c {
			'S' => Self::Start,
			'|' => Self::NorthSouth,
			'-' => Self::EastWest,
			'L' => Self::NorthEast,
			'J' => Self::NorthWest,
			'F' => Self::SouthEast,
			'7' => Self::SouthWest,
			'.' => Self::Empty,
			c => {
				tracing::warn!(?c, "unknown character, defaulting to empty");
				Self::Empty
			},
		}
	}
}

impl fmt::Display for Symbol {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str(match self {
			Self::Empty => " ",
			Self::Start => "S",
			Self::NorthSouth => "┃",
			Self::EastWest => "━",
			Self::NorthEast => "┗",
			Self::NorthWest => "┛",
			Self::SouthEast => "┏",
			Self::SouthWest => "┓",
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn snaky() -> eyre::Result<()> {
		let text = r#"
FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
		"#
		.trim();
		let (_, mut solver): (_, Plumbing) = text.trim().parse_wyz()?;
		solver.prepare_1()?;
		solver.part_1()?;
		solver.prepare_2()?;
		solver.part_2()?;
		Ok(())
	}
}
