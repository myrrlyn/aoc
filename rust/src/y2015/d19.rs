use nom::{
	bytes::complete::tag,
	character::complete::{
		alpha1,
		alphanumeric1,
		newline,
	},
	multi::{
		many1,
		separated_list1,
	},
	sequence::separated_pair,
};
use tap::Tap;

use crate::{
	dict::*,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2015, 19, |t| t.parse_dyn_puzzle::<Synth>());

pub struct Synth {
	rules:    Vec<Rule>,
	products: Dictionary<str>,
	seed:     String,
}

impl Puzzle for Synth {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		for rule in &self.rules {
			for (idx, _) in self.seed.match_indices(&rule.from) {
				self.products.insert(self.seed.clone().tap_mut(|s| {
					s.replace_range(idx .. (idx + rule.from.len()), &rule.into);
				}));
			}
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(self.products.len() as i64)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		todo!()
	}
}

impl<'a> Parsed<&'a str> for Synth {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, rules) = separated_list1(newline, Rule::parse_wyz)(src)?;
		let (rest, seed) = alphanumeric1(rest.trim_start())?;
		Ok((rest, Self {
			rules,
			products: Dictionary::new(),
			seed: seed.to_owned(),
		}))
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Rule {
	from: String,
	into: String,
}

impl<'a> Parsed<&'a str> for Rule {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, (from, into)) =
			separated_pair(alpha1, tag(" => "), alpha1)(src)?;
		Ok((rest, Self {
			from: from.to_owned(),
			into: into.to_owned(),
		}))
	}
}
