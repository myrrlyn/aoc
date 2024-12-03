use bitvec::bitarr;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2022, 6, |t| t.parse_dyn_puzzle::<Message>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
	text: String,
}

impl<'a> Parsed<&'a str> for Message {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		Ok(("", Self {
			text: text.to_owned(),
		}))
	}
}

impl Puzzle for Message {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.find_sync(4)
			.ok_or_else(|| eyre::eyre!("no sync sequence found"))
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.find_sync(14)
			.ok_or_else(|| eyre::eyre!("no sync sequence found"))
	}
}

impl Message {
	pub fn find_sync(&self, len: usize) -> Option<i64> {
		self.text
			.as_bytes()
			.windows(len)
			.enumerate()
			.find(|&(_, window)| {
				let mut seen = bitarr![0; 128];
				for &b in window {
					seen.set(b as usize, true);
				}
				seen.count_ones() == len
			})
			.map(|(start, _)| (start + len) as i64)
	}
}
