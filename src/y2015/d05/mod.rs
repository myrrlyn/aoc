use std::collections::BTreeMap;

use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2015, 5, |t| t.parse_dyn_puzzle::<NaughtyList>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NaughtyList {
	source: String,
}

impl<'a> Parsed<&'a str> for NaughtyList {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		Ok(("", Self {
			source: text.trim().to_owned(),
		}))
	}
}

impl Puzzle for NaughtyList {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.source
			.lines()
			.filter(|line| {
				let mut vowels = 0;
				let mut paired = false;
				let mut not_forbidden = true;
				for pair in line.as_bytes().windows(2) {
					if is_ascii_vowel(&pair[0]) {
						vowels += 1;
					}
					if pair[0] == pair[1] {
						paired = true;
					}
					if [&b"ab"[..], &b"cd"[..], &b"pq"[..], &b"xy"[..]]
						.contains(&pair)
					{
						not_forbidden = false;
					}
				}
				if is_ascii_vowel(&line.as_bytes()[line.len() - 1]) {
					vowels += 1;
				}
				vowels >= 3 && paired && not_forbidden
			})
			.count()
			.pipe(|val| Ok(val as i64))
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut pairs: BTreeMap<&[u8], Vec<usize>> = BTreeMap::new();
		self.source
			.lines()
			.filter(move |line| {
				line.as_bytes()
					.windows(3)
					.any(|triple| triple[0] == triple[2])
			})
			.filter(move |line| {
				pairs.clear();
				for (pos, pair) in line.as_bytes().windows(2).enumerate() {
					let slot = pairs.entry(pair).or_default();
					if let Some(&prev) = slot.last() {
						if prev == pos - 1 {
							continue;
						}
					}
					slot.push(pos);
				}
				pairs.values().any(|v| v.len() > 1)
			})
			.count()
			.pipe(|val| Ok(val as i64))
	}
}

pub fn is_ascii_vowel(byte: &u8) -> bool {
	b"aeiou".contains(byte)
}
