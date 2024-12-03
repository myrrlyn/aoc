use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		alpha1,
		digit0,
		one_of,
		u16 as get_u16,
	},
	combinator::{
		all_consuming,
		map,
		recognize,
		value,
	},
	multi::separated_list1,
	sequence::{
		pair,
		preceded,
		tuple,
	},
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 15, |t| t.parse_dyn_puzzle::<Lenses>());

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Lenses {
	sequence:  Vec<Instruction>,
	lightpath: [Vec<Lens>; 256],
}

impl<'a> Parsed<&'a str> for Lenses {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(
			separated_list1(tag(","), Instruction::parse_wyz),
			|sequence| Self {
				sequence,
				lightpath: std::array::from_fn(|_| Vec::new()),
			},
		)(text)
	}
}

impl Puzzle for Lenses {
	fn part_1(&mut self) -> eyre::Result<i64> {
		self.sequence
			.iter()
			.map(|i| i.hash)
			.fold(0, |accum, hash| accum + hash as i64)
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for insn in self.sequence.as_slice() {
			let grp = &mut self.lightpath[hash(&insn.name) as usize];
			match insn.opcode {
				Opcode::Insert(value) => {
					match grp.iter_mut().find(|lens| lens.name == insn.name) {
						Some(lens) => lens.value = value,
						None => grp.push(Lens {
							name: insn.name.clone(),
							value,
						}),
					}
				},
				Opcode::Remove => {
					grp.retain(|lens| lens.name != insn.name);
				},
			}
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.lightpath
			.iter()
			.zip(1i64 ..)
			.map(|(grp, gnum)| {
				// tracing::debug!(%gnum, ?grp, "box");
				grp.iter()
					.zip(1i64 ..)
					.map(move |(lens, lnum)| gnum * lnum * (lens.value as i64))
			})
			.flatten()
			.sum::<i64>()
			.pipe(Ok)
	}
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Instruction {
	name:   String,
	opcode: Opcode,
	hash:   u8,
}

impl<'a> Parsed<&'a str> for Instruction {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, snippet) =
			recognize(tuple((alpha1, one_of("=-"), digit0)))(text)?;
		let hash = hash(snippet);
		let (_, (name, opcode)) = all_consuming(pair(
			map(alpha1, ToOwned::to_owned),
			Opcode::parse_wyz,
		))(snippet)?;
		Ok((rest, Self { name, opcode, hash }))
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Opcode {
	Insert(u16),
	Remove,
}

impl<'a> Parsed<&'a str> for Opcode {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			map(preceded(tag("="), get_u16), Self::Insert),
			value(Self::Remove, tag("-")),
		))(text)
	}
}

fn hash(text: &str) -> u8 {
	text.as_bytes()
		.iter()
		.fold(0, |accum, &byte| ((accum + byte as u16) * 17) % 256) as u8
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lens {
	name:  String,
	value: u16,
}
