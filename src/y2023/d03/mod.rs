use std::collections::BTreeMap;

use nom::{
	bytes::complete::tag,
	character::complete::{
		anychar,
		digit1,
	},
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 3, |t| t.parse_dyn_puzzle::<Blueprint>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Blueprint {
	/// All discovered numbers in the schema
	all_nums:  BTreeMap<usize, Vec<Token<i32>>>,
	/// All non-. symbols in the schema
	symbols:   BTreeMap<usize, Vec<Token<char>>>,
	/// All numbers which are adjacent to non-. symbols.
	part_nums: Vec<i32>,
	/// All * symbols.
	gears:     BTreeMap<(usize, usize), Vec<i32>>,
}

impl<'a> Parsed<&'a str> for Blueprint {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut this = Self::default();
		for (mut line, row) in text.lines().zip(1 ..) {
			let mut col = 1;
			let mut tokens = vec![];
			while !line.is_empty() {
				if let Ok((rest, _)) = tag::<&'a str, &'a str, ()>(".")(line) {
					col += 1;
					line = rest;
					continue;
				}
				if let Ok((rest, num)) = digit1::<&'a str, ()>(line) {
					let end = col + num.len();
					let val = Legend::PartNumber(num.parse().unwrap_or(0));
					tokens.push(Token { bgn: col, end, val });
					col = end;
					line = rest;
					continue;
				}
				if let Ok((rest, sym)) = anychar::<&'a str, ()>(line) {
					tokens.push(Token {
						bgn: col,
						end: col + 1,
						val: Legend::Symbol(sym),
					});
					col += 1;
					line = rest;
					continue;
				}
			}
			for Token { bgn, end, val } in tokens {
				match val {
					Legend::PartNumber(val) => {
						this.all_nums.entry(row).or_default().push(Token {
							bgn,
							end,
							val,
						});
					},
					Legend::Symbol(val) => {
						this.symbols.entry(row).or_default().push(Token {
							bgn,
							end,
							val,
						});
					},
				}
			}
		}
		Ok(("", this))
	}
}

impl Puzzle for Blueprint {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		// Loop through all lines which have numbers in them
		for (&part_line, part_nums) in self.all_nums.iter() {
			// And select each number token in that line
			for &Token { bgn, end, val } in part_nums {
				// Select the current line and its two neighbors
				for sym_line in
					part_line.saturating_sub(1).max(0) ..= (part_line + 1)
				{
					// And get all the non-number, non-. symbols in those lines
					for symbol in self
						.symbols
						.get(&sym_line)
						.map(Vec::as_slice)
						.unwrap_or_default()
					{
						// If the symbol is in the region `[.NUMBER.]`, save the
						// number
						if symbol.bgn >= bgn.saturating_sub(1).max(0)
							&& symbol.bgn <= end
						{
							self.part_nums.push(val);
						}
					}
				}
			}
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.part_nums
			.iter()
			.copied()
			.map(|n| n as i64)
			.sum::<i64>()
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		// Just copy over the prep-1 loop and add gear-saving.
		// TODO(myrrlyn): Swap number and symbol scanning, only look for gears,
		// and only save numbers touching gears.
		// Loop through all lines which have numbers in them
		for (&part_line, part_nums) in self.all_nums.iter() {
			// And select each number token in that line
			for &Token { bgn, end, val } in part_nums {
				// Select the current line and its two neighbors
				for sym_line in
					part_line.saturating_sub(1).max(0) ..= (part_line + 1)
				{
					// And get all the non-number, non-. symbols in those lines
					for symbol in self
						.symbols
						.get(&sym_line)
						.map(Vec::as_slice)
						.unwrap_or_default()
					{
						// If the symbol is in the region `[.NUMBER.]`, it's a
						// part
						if symbol.bgn >= bgn.saturating_sub(1).max(0)
							&& symbol.bgn <= end
						{
							// And furthermore, if the symbol is a gear, save its
							// number.
							if symbol.val == '*' {
								self.gears
									.entry((sym_line, symbol.bgn))
									.or_default()
									.push(val)
							}
						}
					}
				}
			}
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.gears
			.values()
			.filter(|v| v.len() == 2)
			.map(|v| v.iter().copied().map(|n| n as i64).product::<i64>())
			.sum::<i64>()
			.pipe(Ok)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Token<T> {
	pub bgn: usize,
	pub end: usize,
	pub val: T,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Legend {
	PartNumber(i32),
	Symbol(char),
}
