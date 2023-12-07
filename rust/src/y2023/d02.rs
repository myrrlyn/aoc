use std::collections::BTreeMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{newline, space1},
    combinator::value,
    multi::separated_list1,
    sequence::{delimited, separated_pair, tuple},
};
use tap::Pipe;

use crate::{parse_number, ParseResult, Parseable as _, Parsed, Puzzle, Solver};

#[linkme::distributed_slice(crate::SOLVERS)]
static ITEM: Solver = Solver::new(2023, 2, |t| t.parse_dyn_puzzle::<GameSet>());

static FILTER: Record = Record {
    red: 12,
    blue: 14,
    green: 13,
};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameSet {
    games: BTreeMap<u8, Record>,
}

impl<'a> Parsed<&'a str> for GameSet {
    fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
        let (rest, games) = separated_list1(
            newline,
            tuple((
                delimited(tag("Game "), parse_number::<u8>, tag(": ")),
                separated_list1(tag("; "), Record::parse_wyz),
            )),
        )(text)?;

        Ok((
            rest,
            Self {
                games: games
                    .into_iter()
                    .map(|(ident, records)| {
                        (
                            ident,
                            records.into_iter().fold(
                                Record::default(),
                                |max, Record { red, blue, green }| Record {
                                    red: max.red.max(red),
                                    blue: max.blue.max(blue),
                                    green: max.green.max(green),
                                },
                            ),
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            },
        ))
    }
}

impl Puzzle for GameSet {
    fn part_1(&mut self) -> anyhow::Result<i64> {
        // tracing::debug!(?self);
        self.games
            .iter()
            .filter(|(_, &Record { red, blue, green })| {
                red <= FILTER.red && blue <= FILTER.blue && green <= FILTER.green
            })
            .map(|(&ident, _)| ident as i64)
            .sum::<i64>()
            .pipe(Ok)
    }

    fn part_2(&mut self) -> anyhow::Result<i64> {
        self.games
            .values()
            .map(|Record { red, blue, green }| (red * blue * green) as i64)
            .sum::<i64>()
            .pipe(Ok)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    red: i32,
    blue: i32,
    green: i32,
}

impl<'a> Parsed<&'a str> for Record {
    fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
        let (rest, colors) = separated_list1(
            tag(", "),
            separated_pair(parse_number::<i32>, space1, Color::parse_wyz),
        )(text.trim_start())?;
        let mut this = Self::default();
        for (count, color) in colors {
            match color {
                Color::Red => this.red = count,
                Color::Blue => this.blue = count,
                Color::Green => this.green = count,
            }
        }
        Ok((rest, this))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    Red,
    Blue,
    Green,
}

impl<'a> Parsed<&'a str> for Color {
    fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
        alt((
            value(Self::Red, tag("red")),
            value(Self::Blue, tag("blue")),
            value(Self::Green, tag("green")),
        ))(text)
    }
}
