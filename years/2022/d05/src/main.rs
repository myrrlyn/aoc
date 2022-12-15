use std::{collections::BTreeMap, error::Error};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, newline, u32 as get_u32},
    combinator::{map, opt, value},
    multi::{many1, separated_list1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let (_, mut program) = Dockyard::parse(INPUT)?;
    let mut program2 = program.clone();
    program.run_program_9000()?;
    for col in program.state.values() {
        if let Some(Crate { letter }) = col.last() {
            print!("{letter}");
        }
    }
    println!();
    program2.run_program_9001()?;
    for col in program2.state.values() {
        if let Some(Crate { letter }) = col.last() {
            print!("{letter}");
        }
    }
    println!();
    // println!("{:#?}", program.state);
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Crate {
    letter: char,
}

impl Crate {
    fn parse_row(line: &str) -> IResult<&str, Vec<Option<Self>>> {
        separated_list1(
            tag(" "),
            alt((
                value(None, tag("   ")),
                map(delimited(tag("["), anychar, tag("]")), |c| {
                    Some(Crate { letter: c })
                }),
            )),
        )(line)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Move {
    count: usize,
    from: usize,
    into: usize,
}

impl Move {
    fn parse(line: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("move "),
                get_u32,
                tag(" from "),
                get_u32,
                tag(" to "),
                get_u32,
            )),
            |(_, count, _, from, _, into)| Move {
                count: count as usize,
                from: from as usize,
                into: into as usize,
            },
        )(line)
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Dockyard {
    state: BTreeMap<usize, Vec<Crate>>,
    moves: Vec<Move>,
}

impl Dockyard {
    fn parse(text: &str) -> Result<(&str, Self), String> {
        let (rest, rows) =
            many1(terminated(Crate::parse_row, newline))(text).map_err(|e| format!("pile: {e}"))?;
        let (rest, cols) = terminated(parse_markers, tuple((newline, newline)))(rest)
            .map_err(|e| format!("marker: {e}"))?;
        let mut moves = Vec::new();
        for line in rest.lines() {
            let (rest, insn) = Move::parse(line).map_err(|e| format!("move: {e}"))?;
            if !rest.is_empty() {
                return Err("extra text in move line".to_owned())?;
            }
            moves.push(insn);
        }

        let mut state = cols
            .into_iter()
            .map(|c| (c, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        for row in rows.into_iter().rev() {
            for (pkg, col) in row.into_iter().zip(1..) {
                if let Some(pkg) = pkg {
                    state.entry(col).or_default().push(pkg);
                }
            }
        }
        Ok(("", Self { state, moves }))
    }

    fn run_program_9000(&mut self) -> Result<(), String> {
        for Move { count, from, into } in self.moves.drain(..) {
            for _ in 0..count {
                let tmp = self
                    .state
                    .get_mut(&from)
                    .ok_or_else(|| format!("no such source: {from}"))?
                    .pop()
                    .ok_or("cannot pull from empty column")?;
                self.state
                    .get_mut(&into)
                    .ok_or_else(|| format!("no such destination: {into}"))?
                    .push(tmp);
            }
        }
        Ok(())
    }

    fn run_program_9001(&mut self) -> Result<(), String> {
        for Move { count, from, into } in self.moves.drain(..) {
            let src = self
                .state
                .get_mut(&from)
                .ok_or_else(|| format!("no such source: {from}"))?;
            let mid = src.len().checked_sub(count).ok_or_else(|| {
                format!(
                    "cannot move {count} items from stack {from} (size {len})",
                    len = src.len()
                )
            })?;
            let tmp = src.split_off(mid);
            self.state
                .get_mut(&into)
                .ok_or_else(|| format!("no such destination: {into}"))?
                .extend(tmp);
        }
        Ok(())
    }
}

fn parse_markers(s: &str) -> IResult<&str, Vec<usize>> {
    separated_list1(
        tag(" "),
        map(delimited(tag(" "), get_u32, opt(tag(" "))), |n| n as usize),
    )(s)
}
