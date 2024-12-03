use std::cmp;

use nom::{
	character::complete::{
		i64 as get_i64,
		newline,
	},
	multi::{
		many1,
		separated_list1,
	},
	sequence::terminated,
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2022, 1, |t| t.parse_dyn_puzzle::<Commissary>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Commissary {
	packs: Vec<i64>,
}

impl<'a> Parsed<&'a str> for Commissary {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, packs) =
			separated_list1(newline, many1(terminated(get_i64, newline)))(text)?;
		let packs = packs.into_iter().map(|v| v.into_iter().sum()).collect();
		Ok((rest, Self { packs }))
	}
}

impl Puzzle for Commissary {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.packs.sort_by_key(|&i| cmp::Reverse(i));
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.packs
			.first()
			.copied()
			.ok_or_else(|| eyre::eyre!("cannot handle an empty group"))
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.packs.sort_by_key(|&i| cmp::Reverse(i));
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.packs[.. 3]
			.iter()
			.copied()
			.sum::<i64>()
			.pipe(Some)
			.ok_or_else(|| eyre::eyre!("cannot handle an empty group"))
	}
}
