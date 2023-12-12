use std::ops::RangeInclusive;

use bitvec::{
	access::BitSafeUsize,
	prelude::*,
};
use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		newline,
		space1,
		u16 as get_u16,
	},
	combinator::{
		map,
		value,
	},
	multi::separated_list1,
	sequence::{
		separated_pair,
		terminated,
		tuple,
	},
};
use tap::Pipe;

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2015, 6, |t| t.parse_dyn_puzzle::<LightGrid>());

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LightGrid {
	steps: Vec<Instruction>,
	state: Grid,
}

impl<'a> Parsed<&'a str> for LightGrid {
	fn parse_wyz(_: &'a str) -> ParseResult<&'a str, Self> {
		unimplemented!(
			"this structure is too large to carry in the stack. use \
			 `.parse_wyz_boxed`"
		);
	}

	fn parse_wyz_boxed(text: &'a str) -> ParseResult<&'a str, Box<Self>> {
		let (out, steps) =
			separated_list1(newline, Instruction::parse_wyz)(text)?;
		Ok((
			out,
			Box::new(Self {
				steps,
				state: Grid::Digital(BitArray::ZERO),
			}),
		))
	}
}

impl Puzzle for LightGrid {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		if let Grid::Analog(_) = self.state {
			self.state = Grid::Digital(BitArray::ZERO);
		}
		let Grid::Digital(grid) = &mut self.state
		else {
			unreachable!("enforced digital");
		};
		grid.fill(false);
		for step in &self.steps {
			step.digital(&mut *grid);
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		let Grid::Digital(grid) = &mut self.state
		else {
			eyre::bail!("part 1 is a digital grid");
		};
		Ok(grid.count_ones() as i64)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		if let Grid::Digital(_) = self.state {
			self.state = Grid::Analog([[0; DIMENSION]; DIMENSION]);
		}
		let Grid::Analog(grid) = &mut self.state
		else {
			unreachable!("enforced analog");
		};
		grid.iter_mut().for_each(|row| row.fill(0));
		for step in &self.steps {
			step.analog(&mut *grid);
		}
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let Grid::Analog(grid) = &mut self.state
		else {
			eyre::bail!("part 2 is an analog grid");
		};
		grid.iter()
			.map(|row| row.iter().copied())
			.flatten()
			.map(|a| a as i64)
			.sum::<i64>()
			.pipe(Ok)
	}
}

pub const DIMENSION: usize = 1000;
pub type Digital = BitArr![for DIMENSION * DIMENSION];
pub type Analog = [[u32; DIMENSION]; DIMENSION];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Grid {
	Digital(Digital),
	Analog(Analog),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Instruction {
	cols: RangeInclusive<usize>,
	rows: RangeInclusive<usize>,
	insn: Opcode,
}

impl Instruction {
	pub fn digital(&self, grid: &mut Digital) {
		match self.insn {
			Opcode::On => self.turn_on(grid),
			Opcode::Off => self.turn_off(grid),
			Opcode::Toggle => self.toggle(grid),
		}
	}

	pub fn analog(&self, grid: &mut Analog) {
		match self.insn {
			Opcode::On => self.turn_up(grid),
			Opcode::Off => self.turn_down(grid),
			Opcode::Toggle => self.turn_up_2(grid),
		}
	}

	pub fn turn_on(&self, grid: &mut Digital) {
		for row in self.digital_rows(grid) {
			row[self.cols.clone()].fill(true);
		}
	}

	pub fn turn_off(&self, grid: &mut Digital) {
		for row in self.digital_rows(grid) {
			row[self.cols.clone()].fill(false);
		}
	}

	pub fn toggle(&self, grid: &mut Digital) {
		for row in self.digital_rows(grid) {
			row[self.cols.clone()] ^= bits![1; DIMENSION];
		}
	}

	pub fn turn_up(&self, grid: &mut Analog) {
		let cols = self.cols.clone();
		self.analog_rows(grid)
			.map(move |row| {
				let cols = cols.clone();
				&mut row[cols]
			})
			.flatten()
			.for_each(|cell| *cell += 1);
	}

	pub fn turn_down(&self, grid: &mut Analog) {
		let cols = self.cols.clone();
		self.analog_rows(grid)
			.map(move |row| {
				let cols = cols.clone();
				&mut row[cols]
			})
			.flatten()
			.for_each(|cell| *cell = cell.saturating_sub(1));
	}

	pub fn turn_up_2(&self, grid: &mut Analog) {
		let cols = self.cols.clone();
		self.analog_rows(grid)
			.map(move |row| {
				let cols = cols.clone();
				&mut row[cols]
			})
			.flatten()
			.for_each(|cell| *cell += 2);
	}

	fn digital_rows<'a, 'b>(
		&'a self,
		grid: &'b mut Digital,
	) -> impl 'b + Iterator<Item = &'b mut BitSlice<BitSafeUsize>> {
		let rows = self.rows.clone();
		grid.chunks_exact_mut(DIMENSION)
			.enumerate()
			.filter(move |(row, _)| rows.contains(row))
			.map(|(_, row)| row)
	}

	pub fn analog_rows<'a, 'b>(
		&'a self,
		grid: &'b mut Analog,
	) -> impl 'b + Iterator<Item = &'b mut [u32; DIMENSION]> {
		let rows = self.rows.clone();
		grid.iter_mut()
			.enumerate()
			.filter(move |(row, _)| rows.contains(row))
			.map(|(_, row)| row)
	}
}

impl<'a> Parsed<&'a str> for Instruction {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let coords = |t: &'a str| -> ParseResult<&'a str, (usize, usize)> {
			map(separated_pair(get_u16, tag(","), get_u16), |(a, b)| {
				(a as usize, b as usize)
			})(t)
		};
		map(
			tuple((
				terminated(Opcode::parse_wyz, space1),
				separated_pair(coords, tag(" through "), coords),
			)),
			|(insn, ((x1, y1), (x2, y2)))| Self {
				cols: x1 ..= x2,
				rows: y1 ..= y2,
				insn,
			},
		)(text)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Opcode {
	On,
	Off,
	Toggle,
}

impl<'a> Parsed<&'a str> for Opcode {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::On, tag("turn on")),
			value(Self::Off, tag("turn off")),
			value(Self::Toggle, tag("toggle")),
		))(text)
	}
}
