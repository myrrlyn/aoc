use std::fmt::Write as _;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2015, 4, |t| t.parse_dyn_puzzle::<Miner>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Miner {
	seed: String,
}

impl<'a> Parsed<&'a str> for Miner {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		Ok(("", Self {
			seed: text.trim().to_owned(),
		}))
	}
}

impl Puzzle for Miner {
	fn part_1(&mut self) -> eyre::Result<i64> {
		let mut to_hash = String::new();
		for ct in 1 .. {
			if ct % 1000 == 0 {
				tracing::debug!(%ct, "round");
			}
			to_hash.clear();
			write!(&mut to_hash, "{seed}{ct}", seed = self.seed)?;
			let hashed = md5::compute(to_hash.as_str());
			if &hashed[.. 2] == &[0, 0] && hashed[2] & 0xF0 == 0 {
				return Ok(ct);
			}
		}
		eyre::bail!("never found a 00000- prefix");
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut to_hash = String::new();
		for ct in 1 .. {
			if ct % 1000 == 0 {
				tracing::debug!(%ct, "round");
			}
			to_hash.clear();
			write!(&mut to_hash, "{seed}{ct}", seed = self.seed)?;
			let hashed = md5::compute(to_hash.as_str());
			if &hashed[.. 3] == &[0, 0, 0] {
				return Ok(ct);
			}
		}
		eyre::bail!("never found a 000000- prefix");
	}
}
