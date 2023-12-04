//! Run as `open ../input.txt | cargo run | lines | each { from json }` in nu

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead as _, Write as _},
    str::FromStr,
};

use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    multi::many1,
    sequence::{delimited, preceded, terminated},
};
use serde::{Deserialize, Serialize};
use wyz_aoc::parse_number;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    let cards = io::stdin()
        .lock()
        .lines()
        .map(|line| -> anyhow::Result<Card> { line?.parse::<Card>() })
        .filter_map(|res| match res {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!("failed to parse card: {err:?}");
                None
            }
        })
        .collect::<Vec<Card>>();
    let part1 = cards.iter().map(Card::score).sum::<i32>();
    writeln!(stdout, "{{part: 1, value: {part1}}}")?;
    let mut pile = BTreeMap::new();
    for card in cards {
        let this = pile.entry(card.ident).or_insert(0);
        *this += 1;
        let this = *this;
        for id in (card.ident + 1)..=(card.ident + card.matches()) {
            *pile.entry(id).or_insert(0) += this;
        }
    }
    let part2 = pile.into_values().sum::<i32>();
    writeln!(stdout, "{{part: 2, value: {part2}}}")?;
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Card {
    ident: usize,
    winners: BTreeSet<i32>,
    picks: BTreeSet<i32>,
}

impl Card {
    fn matches(&self) -> usize {
        self.winners.intersection(&self.picks).count()
    }

    fn score(&self) -> i32 {
        match self.matches() {
            0 => 0,
            n => 1 << (n - 1),
        }
    }
}

impl FromStr for Card {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (rest, ident) =
            delimited(terminated(tag("Card"), space1), parse_number, tag(":"))(line)
                .map_err(|err| anyhow::anyhow!("failed to parse header: {err:?}"))?;
        let (rest, winners) =
            terminated(many1(delimited(space0, parse_number, space0)), tag("|"))(rest)
                .map_err(|err| anyhow::anyhow!("failed to parse winning numbers: {err:?}"))?;
        let (rest, picks) = many1(preceded(space1, parse_number))(rest)
            .map_err(|err| anyhow::anyhow!("failed to parse guesses: {err:?}"))?;
        if !rest.is_empty() {
            anyhow::bail!("failed to consume all input: {rest:?} remaining");
        }

        Ok(Self {
            ident: ident as usize,
            winners: winners.into_iter().collect(),
            picks: picks.into_iter().collect(),
        })
    }
}
