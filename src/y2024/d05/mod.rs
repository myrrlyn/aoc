use std::collections::{
	BTreeMap,
	BTreeSet,
};

use bitvec::prelude::*;
use nom::{
	bytes::complete::tag,
	character::complete::newline,
	combinator::map,
	multi::{
		many1,
		separated_list1,
	},
	sequence::separated_pair,
};
use tap::Pipe;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 5, |t| t.parse_dyn_puzzle::<Printer>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Printer {
	rules:        Vec<Rule>,
	dependencies: BTreeMap<u8, BitArr![for 128, in usize]>,
	groups:       Vec<Group>,
}

impl Printer {
	pub fn find_dependencies(&mut self) {
		for &Rule { before, after } in &self.rules {
			self.dependencies
				.entry(after)
				.or_default()
				.set(before as usize, true);
		}
	}
}

impl Puzzle for Printer {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.find_dependencies();
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.groups
			.iter()
			.filter(|g| g.correctly_ordered(&self.dependencies))
			.map(|g| g.pages[g.pages.len() / 2] as i64)
			.sum::<i64>()
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.prepare_1()
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut sum = 0;
		for grp in &mut self.groups {
			if !grp.correctly_ordered(&self.dependencies) {
				grp.make_ordered(&self.dependencies)?;
				sum += grp.pages[grp.pages.len() / 2] as i64;
			}
		}
		Ok(sum)
	}
}

impl<'a> Parsed<&'a str> for Printer {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_pair(
				separated_list1(newline, Rule::parse_wyz),
				many1(newline),
				separated_list1(newline, Group::parse_wyz),
			),
			|(rules, groups)| Self {
				rules,
				groups,
				dependencies: BTreeMap::new(),
			},
		)(src)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Rule {
	before: u8,
	after:  u8,
}

impl<'a> Parsed<&'a str> for Rule {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_pair(parse_number::<u8>, tag("|"), parse_number::<u8>),
			|(before, after)| Self { before, after },
		)(src)
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Group {
	pages: Vec<u8>,
}

impl Group {
	pub fn correctly_ordered(
		&self,
		deps: &BTreeMap<u8, BitArr![for 128, in usize]>,
	) -> bool {
		let mut mask = bitarr![0; 128];
		for &page in &self.pages {
			mask.set(page as usize, true);
		}
		let mut printed = BTreeSet::new();
		for page in &self.pages {
			for dep in
				(deps.get(page).copied().unwrap_or_default() & mask).iter_ones()
			{
				let dep = dep as u8;
				if !printed.contains(&dep) {
					return false;
				}
			}
			printed.insert(page);
		}
		true
	}

	pub fn make_ordered(
		&mut self,
		deps: &BTreeMap<u8, BitArr![for 128, in usize]>,
	) -> eyre::Result<()> {
		let begin = std::time::Instant::now();
		while !self.correctly_ordered(deps) {
			if std::time::Instant::now() - begin
				> std::time::Duration::from_secs(5)
			{
				eyre::bail!("timed out re-ordering");
			}
			let mut mask = bitarr![0; 128];
			for &page in &self.pages {
				mask.set(page as usize, true);
			}
			for one in 0 .. self.pages.len() {
				for two in (one + 1) .. self.pages.len() {
					let before = self.pages[one];
					let after = self.pages[two];
					if deps.get(&before).copied().unwrap_or_default()
						[after as usize]
					{
						self.pages.swap(one, two);
						tracing::debug!(?self.pages, "swapped {} before {}", after, before);
					}
				}
			}
		}
		Ok(())
	}
}

impl<'a> Parsed<&'a str> for Group {
	fn parse_wyz(src: &'a str) -> ParseResult<&'a str, Self> {
		map(separated_list1(tag(","), parse_number::<u8>), |pages| {
			Self { pages }
		})(src)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_ordering() -> eyre::Result<()> {
		let sample = include_str!("sample.txt");
		let (_, mut printer) = sample.parse_wyz::<Printer>()?;
		printer.find_dependencies();

		assert!(printer.groups[0].correctly_ordered(&printer.dependencies));
		assert!(printer.groups[1].correctly_ordered(&printer.dependencies));
		assert!(printer.groups[2].correctly_ordered(&printer.dependencies));
		assert!(!printer.groups[3].correctly_ordered(&printer.dependencies));
		assert!(!printer.groups[4].correctly_ordered(&printer.dependencies));
		assert!(!printer.groups[5].correctly_ordered(&printer.dependencies));
		Ok(())
	}
}
