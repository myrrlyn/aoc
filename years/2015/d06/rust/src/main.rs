use bitvec::domain::Domain;
use bitvec::prelude::*;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u16 as get_u16,
    combinator::value,
    sequence::{separated_pair, terminated},
};
use std::ops::RangeInclusive;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let program = INPUT
        .lines()
        .flat_map(|line| Step::parse(line).ok().map(|(_, s)| s))
        .collect::<Vec<_>>();

    let mut grid = Box::new(BinaryGrid::ZERO);
    grid.run_program(&program);
    let pt1 = grid.count_on();
    println!("part 1: {pt1}");

    let mut grid = RheostatGrid::zero();
    grid.run_program(&program);
    let pt2 = grid.total_brightness();
    println!("part 2: {pt2}");
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct BinaryGrid {
    matrix: [BitArr!(for 1000); 1000],
}

impl BinaryGrid {
    const ZERO: Self = Self {
        matrix: [<BitArr!(for 1000)>::ZERO; 1000],
    };

    fn run_program(&mut self, program: &[Step]) {
        for Step { action, cols, rows } in program.iter().cloned() {
            match action {
                Opcode::On => self.turn_on(cols, rows),
                Opcode::Off => self.turn_off(cols, rows),
                Opcode::Toggle => self.toggle(cols, rows),
            }
        }
    }

    fn turn_on(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            row[cols.clone()].fill(true);
        }
    }

    fn turn_off(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            row[cols.clone()].fill(false);
        }
    }

    fn toggle(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            match row[cols.clone()].domain_mut() {
                Domain::Enclave(mut elem) => {
                    elem.invert();
                }
                Domain::Region { head, body, tail } => {
                    if let Some(mut h) = head {
                        h.invert();
                    }
                    for elem in body {
                        *elem ^= !0;
                    }
                    if let Some(mut t) = tail {
                        t.invert();
                    }
                }
            }
        }
    }

    fn count_on(&self) -> usize {
        self.matrix.iter().map(|ba| ba.count_ones()).sum()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct RheostatGrid {
    matrix: [[u32; 1000]; 1000],
}

impl RheostatGrid {
    fn zero() -> Box<Self> {
        Box::new(Self {
            matrix: [[0; 1000]; 1000],
        })
    }

    fn run_program(&mut self, program: &[Step]) {
        for Step { action, cols, rows } in program.iter().cloned() {
            match action {
                Opcode::On => self.turn_up(cols, rows),
                Opcode::Off => self.turn_down(cols, rows),
                Opcode::Toggle => self.turn_up2(cols, rows),
            }
        }
    }

    fn turn_up(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            row[cols.clone()].iter_mut().for_each(|e| *e += 1);
        }
    }

    fn turn_down(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            row[cols.clone()]
                .iter_mut()
                .for_each(|e| *e = e.saturating_sub(1));
        }
    }

    fn turn_up2(&mut self, cols: RangeInclusive<usize>, rows: RangeInclusive<usize>) {
        for row in &mut self.matrix[rows] {
            row[cols.clone()].iter_mut().for_each(|e| *e += 2);
        }
    }

    fn total_brightness(&self) -> u32 {
        self.matrix
            .iter()
            .map(|row| row.iter().copied().sum::<u32>())
            .sum::<u32>()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Step {
    cols: RangeInclusive<usize>,
    rows: RangeInclusive<usize>,
    action: Opcode,
}

impl Step {
    fn parse(line: &str) -> IResult<&str, Self> {
        let pair = || separated_pair(get_u16, tag(","), get_u16);
        map(
            tuple((
                terminated(Opcode::parse, tag(" ")),
                separated_pair(pair(), tag(" through "), pair()),
            )),
            |(action, ((x1, y1), (x2, y2)))| Self {
                cols: x1 as usize..=x2 as usize,
                rows: y1 as usize..=y2 as usize,
                action,
            },
        )(line)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Opcode {
    On,
    Off,
    Toggle,
}

impl Opcode {
    fn parse(text: &str) -> IResult<&str, Self> {
        alt((
            value(Self::On, tag("turn on")),
            value(Self::Off, tag("turn off")),
            value(Self::Toggle, tag("toggle")),
        ))(text)
    }
}
