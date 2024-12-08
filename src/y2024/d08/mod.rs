use std::collections::{
	BTreeMap,
	BTreeSet,
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		newline,
		none_of,
	},
	combinator::{
		map,
		value,
	},
	multi::{
		many1,
		separated_list1,
	},
};
use tap::{
	Pipe,
	TapOptional,
};

use crate::{
	coords::{
		spaces::DisplayGrid,
		Dense2DSpace,
	},
	prelude::*,
	Coord2D,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 8, |t| t.parse_dyn_puzzle::<Antennae>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Antennae {
	grid:  Dense2DSpace<i8, Square>,
	freqs: BTreeMap<char, BTreeSet<Coord2D<i8>>>,
}

impl Puzzle for Antennae {
	fn after_parse(&mut self) -> eyre::Result<()> {
		for (loc, freq) in
			self.grid.iter().filter_map(|(c, s)| s.freq.map(|f| (c, f)))
		{
			self.freqs.entry(freq).or_default().insert(loc);
		}
		Ok(())
	}

	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.mark_antinodes(false)
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.grid
			.iter()
			.filter(|(_, s)| s.has_node)
			.count()
			.pipe(|ct| Ok(ct as i64))
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.mark_antinodes(true)?;
		println!("{:}", self.display());
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.grid
			.iter()
			.filter(|(_, s)| s.has_node)
			.count()
			.pipe(|ct| Ok(ct as i64))
	}
}

impl Antennae {
	pub fn mark_antinodes(&mut self, repeat: bool) -> eyre::Result<()> {
		for (&freq, sites) in self.freqs.iter() {
			tracing::info!(%freq, "scanning");
			for &site in sites {
				for &other in sites {
					if site == other {
						continue;
					}
					let delta = other - site;
					let mut sites =
						(1 ..).map(|scale| site + (delta * scale)).pipe(|it| {
							if repeat {
								Box::new(it) as Box<dyn Iterator<Item = _>>
							}
							else {
								Box::new(it.skip(1).take(1))
							}
						});
					while let Some(sq) = sites.next().and_then(|sq| {
						self.grid.get_mut(sq).tap_some(
							|_| tracing::trace!(%freq, %sq, "found antinode"),
						)
					}) {
						sq.has_node = true;
					}
				}
			}
		}
		Ok(())
	}
}

impl DisplayGrid<i8, Square> for Antennae {
	fn bounds_inclusive(&self) -> Option<(Coord2D<i8>, Coord2D<i8>)> {
		self.grid.dimensions()
	}

	fn print_cell(
		&self,
		symbols: &crate::coords::spaces::Symbols,
		row: i8,
		col: i8,
		_: usize,
		_: usize,
	) -> char {
		let sq = self.grid[Coord2D::new(col, row)];
		if let Some(f) = sq.freq {
			f
		}
		else if sq.has_node {
			symbols.full
		}
		else {
			symbols.empty
		}
	}
}

impl<'a> Parsed<&'a str> for Antennae {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(newline, many1(Square::parse_wyz)),
			|squares| Self {
				grid:  Dense2DSpace::from_raw(Coord2D::ZERO, squares),
				freqs: BTreeMap::new(),
			},
		)(src)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Square {
	freq:     Option<char>,
	has_node: bool,
}

impl<'a> Parsed<&'a str> for Square {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			alt((value(None, tag(".")), map(none_of("\r\n"), Some))),
			|freq| Self {
				freq,
				has_node: false,
			},
		)(src)
	}
}
