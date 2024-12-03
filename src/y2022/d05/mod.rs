use std::{
	collections::BTreeMap,
	fmt,
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		anychar,
		newline,
		u32 as get_u32,
	},
	combinator::{
		map,
		value,
	},
	multi::{
		many1,
		separated_list1,
	},
	sequence::{
		delimited,
		preceded,
		terminated,
		tuple,
	},
};

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2022, 5, |t| t.parse_dyn_puzzle::<Dockyard>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dockyard {
	pile:   BTreeMap<usize, Vec<Crate>>,
	moves:  Vec<Move>,
	answer: String,
}

impl Dockyard {
	pub fn run_singly(&mut self) -> eyre::Result<()> {
		let mut pile = self.pile.clone();
		for Move { cnt, src, dst } in &self.moves {
			for _ in 0 .. *cnt {
				let tmp = pile
					.get_mut(src)
					.ok_or_else(|| eyre::eyre!("no such source column {src}"))?
					.pop()
					.ok_or_else(|| {
						eyre::eyre!("cannot pull from an empty column")
					})?;
				pile.get_mut(dst)
					.ok_or_else(|| {
						eyre::eyre!("no such destination column {dst}")
					})?
					.push(tmp);
			}
		}
		self.answer = pile
			.values()
			.flat_map(|v| v.last())
			.map(|Crate { ident }| ident)
			.collect();
		Ok(())
	}

	pub fn run_multi(&mut self) -> eyre::Result<()> {
		let mut pile = self.pile.clone();
		for Move { cnt, src, dst } in &self.moves {
			let from = pile
				.get_mut(src)
				.ok_or_else(|| eyre::eyre!("no such source column {src}"))?;
			let mid = from.len().checked_sub(*cnt).ok_or_else(|| {
				eyre::eyre!(
					"cannot move {cnt} items from stack {src} (size {len})",
					len = from.len(),
				)
			})?;
			let tmp = from.split_off(mid);
			pile.get_mut(dst)
				.ok_or_else(|| eyre::eyre!("no such destination column {dst}"))?
				.extend(tmp);
		}
		self.answer = pile
			.values()
			.flat_map(|v| v.last())
			.map(|Crate { ident }| ident)
			.collect();
		Ok(())
	}
}

impl<'a> Parsed<&'a str> for Dockyard {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, rows) = many1(terminated(
			separated_list1(
				tag(" "),
				alt((value(None, tag("   ")), map(Crate::parse_wyz, Some))),
			),
			newline,
		))(text)?;
		let (rest, idents) = terminated(
			separated_list1(
				tag(" "),
				delimited(tag(" "), map(get_u32, |n| n as usize), tag(" ")),
			),
			newline,
		)(rest)?;
		let mut pile = BTreeMap::<usize, Vec<Crate>>::new();
		// Accumulate from the bottom of the diagram upwards
		for row in rows.into_iter().rev() {
			for (pkg, col) in row.into_iter().zip(idents.iter().copied()) {
				if let Some(pkg) = pkg {
					pile.entry(col).or_default().push(pkg);
				}
			}
		}
		let (rest, _) = newline(rest)?;
		let (rest, moves) = separated_list1(newline, Move::parse_wyz)(rest)?;
		Ok((rest, Self {
			pile,
			moves,
			answer: String::new(),
		}))
	}
}

impl Puzzle for Dockyard {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		tracing::debug!("parsed\n{self:#}");
		self.run_singly()?;
		tracing::info!(%self.answer, "top crates");
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(0)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.run_multi()?;
		tracing::info!(%self.answer, "top crates");
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		Ok(0)
	}
}

impl fmt::Display for Dockyard {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let highest = self
			.pile
			.values()
			.map(|v| v.len())
			.max()
			.unwrap_or_default();
		for row in (0 .. highest).rev() {
			writeln!(
				fmt,
				"{}",
				self.pile
					.values()
					.map(|v| v.get(row))
					.map(|pkg| match pkg {
						Some(pkg) => format!("{pkg}"),
						None => format!("   "),
					})
					.collect::<Vec<_>>()
					.join(" ")
					.trim_end()
			)?;
		}
		writeln!(
			fmt,
			"{}",
			self.pile
				.keys()
				.map(|n| format!(" {n} "))
				.collect::<Vec<_>>()
				.join(" ")
				.trim_end()
		)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Crate {
	ident: char,
}

impl<'a> Parsed<&'a str> for Crate {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(delimited(tag("["), anychar, tag("]")), |ident| Self {
			ident,
		})(text)
	}
}

impl fmt::Display for Crate {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "[{}]", self.ident)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Move {
	/// How many crates to move.
	cnt: usize,
	/// The column from which to move them.
	src: usize,
	// The column to which to move them.
	dst: usize,
}

impl<'a> Parsed<&'a str> for Move {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			tuple((
				preceded(tag("move "), get_u32),
				preceded(tag(" from "), get_u32),
				preceded(tag(" to "), get_u32),
			)),
			|(cnt, src, dst)| Self {
				cnt: cnt as usize,
				src: src as usize,
				dst: dst as usize,
			},
		)(text)
	}
}
