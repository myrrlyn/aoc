use bitvec::BitArr;
use nom::{
	character::complete::{
		alpha1,
		newline,
	},
	combinator::map,
	multi::separated_list1,
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2022, 3, |t| t.parse_dyn_puzzle::<Commissary>());

type Priorities = BitArr!(for 53);

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Commissary {
	packs: Vec<Backpack>,
}

impl<'a> Parsed<&'a str> for Commissary {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(newline, Backpack::parse_wyz), |packs| {
			Self { packs }
		})(text)
	}
}

impl Puzzle for Commissary {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.packs
			.iter()
			.copied()
			.map(|b @ Backpack { left, right }| {
				(left & right).first_one().map(|i| i as i64).ok_or_else(|| {
					tracing::error!(?b, "no common item");
					eyre::eyre!("no common item")
				})
			})
			.collect::<eyre::Result<Vec<i64>>>()?
			.into_iter()
			.sum::<i64>()
			.pipe(Ok)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.packs
			.chunks_exact(3)
			.map(|c| {
				c.iter()
					.map(|b| b.left | b.right)
					.reduce(|accum, next| accum & next)
					.and_then(|prio| prio.first_one())
					.map(|i| i as i64)
					.ok_or_else(|| {
						tracing::error!("no common item in group");
						eyre::eyre!("no common item in group")
					})
			})
			.collect::<eyre::Result<Vec<i64>>>()?
			.into_iter()
			.sum::<i64>()
			.pipe(Ok)
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Backpack {
	left:  Priorities,
	right: Priorities,
}

impl<'a> Parsed<&'a str> for Backpack {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, line) = alpha1(text)?;
		let ct = line.len();
		if ct % 2 != 0 {
			tracing::error!(%line, "must have even number of symbols");
			return Err(nom::Err::Failure(nom::error::Error::new(
				line,
				nom::error::ErrorKind::Alpha,
			)));
		}
		let (left, right) = line.split_at(ct / 2);
		Ok((rest, Self {
			left:  Self::segment_to_priorities(left),
			right: Self::segment_to_priorities(right),
		}))
	}
}

impl Backpack {
	fn segment_to_priorities(text: &str) -> Priorities {
		text.chars()
			.map(Item::from)
			.map(|Item { priority }| priority)
			.fold(Priorities::ZERO, |mut accum, elem| {
				accum.set(elem, true);
				accum
			})
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Item {
	priority: usize,
}

impl From<char> for Item {
	fn from(c: char) -> Self {
		Self {
			priority: match c {
				'a' ..= 'z' => 1 + (c as usize - 'a' as usize),
				'A' ..= 'Z' => 27 + (c as usize - 'A' as usize),
				c => {
					tracing::error!(%c, "unknown priority symbol");
					0
				},
			},
		}
	}
}
