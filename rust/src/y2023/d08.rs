use std::collections::BTreeMap;

use nom::{
	bytes::complete::tag,
	character::complete::alphanumeric1,
	sequence::{delimited, separated_pair},
};
use tap::TapFallible;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 8, |t| t.parse_dyn_puzzle::<Maps>());

pub struct Maps {
	switches: String,
	strings: BTreeMap<String, CacheKey>,
	cache: BTreeMap<CacheKey, String>,
	graph: BTreeMap<CacheKey, (CacheKey, CacheKey)>,
}

impl<'a> Parsed<&'a str> for Maps {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut lines = text.lines();
		let switches =
			lines.next().expect("must have a sequence line").to_owned();
		let _ = lines.next();
		let mut strings = BTreeMap::new();
		let mut graph = BTreeMap::new();
		let mut key_source = (0..).map(|key| CacheKey { key });
		for line in lines {
			let (_, (from, (left, right))) = separated_pair(
				alphanumeric1,
				tag(" = "),
				delimited(
					tag("("),
					separated_pair(alphanumeric1, tag(", "), alphanumeric1),
					tag(")"),
				),
			)(line)?;
			let from_key = strings.get(from).copied().unwrap_or_else(|| {
				let key = key_source.next().unwrap();
				strings.insert(from.to_owned(), key);
				key
			});
			let left_key = strings.get(left).copied().unwrap_or_else(|| {
				let key = key_source.next().unwrap();
				strings.insert(left.to_owned(), key);
				key
			});
			let right_key = strings.get(right).copied().unwrap_or_else(|| {
				let key = key_source.next().unwrap();
				strings.insert(right.to_owned(), key);
				key
			});
			graph.insert(from_key, (left_key, right_key));
		}
		let cache = strings
			.iter()
			.map(|(key, &val)| (val, key.clone()))
			.collect();
		Ok((
			"",
			Self {
				switches,
				strings,
				cache,
				graph,
			},
		))
	}
}

impl Puzzle for Maps {
	fn part_1(&mut self) -> eyre::Result<i64> {
		let &bgn = self
			.strings
			.get("AAA")
			.ok_or_else(|| eyre::eyre!("missing the AAA starting node"))?;
		let &end = self
			.strings
			.get("ZZZ")
			.ok_or_else(|| eyre::eyre!("missing the ZZZ ending node"))?;
		let mut steps = 0;
		let mut current = bgn;
		let mut switches = self.switches.chars().cycle();
		while current != end {
			current = self.jump(current, switches.next().unwrap())?;
			steps += 1;
			if steps % 1000 == 0 {
				tracing::trace!(?steps, "loop ticking");
			}
		}
		Ok(steps)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.strings
			.iter()
			.filter(|(text, _)| text.ends_with("A"))
			.map(|(_, &key)| self.seek_any_endpoint(key))
			.filter_map(|res| res.tap_err(|err| tracing::error!("{err}")).ok())
			.reduce(num::integer::lcm)
			.ok_or_else(|| eyre::eyre!("did not find any starting nodes"))
	}
}

impl Maps {
	fn jump(&self, key: CacheKey, switch: char) -> eyre::Result<CacheKey> {
		let &(left, right) = self.graph.get(&key).ok_or_else(|| {
			let text =
				self.cache.get(&key).expect("no dangling string-cache keys");
			eyre::eyre!("node {text} is referenced but not defined")
		})?;
		match switch {
			'L' => Ok(left),
			'R' => Ok(right),
			c => eyre::bail!("unknown branch indicator `{c}`"),
		}
	}

	fn seek_any_endpoint(&self, mut key: CacheKey) -> eyre::Result<i64> {
		let mut ct = 0;
		let mut switches = self.switches.chars().cycle();
		while !self
			.cache
			.get(&key)
			.expect("no dangling cache keys")
			.ends_with("Z")
		{
			key = self.jump(key, switches.next().unwrap())?;
			ct += 1;
		}
		Ok(ct)
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CacheKey {
	key: u16,
}
