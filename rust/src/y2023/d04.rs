use std::collections::{
	BTreeMap,
	BTreeSet,
};

use nom::{
	bytes::complete::tag,
	character::complete::{
		newline,
		space0,
		space1,
	},
	multi::many1,
	sequence::{
		delimited,
		preceded,
		terminated,
	},
};
use tap::TapFallible;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 4, |t| t.parse_dyn_puzzle::<Lottery>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lottery {
	cards: Vec<Card>,
}

impl<'a> Parsed<&'a str> for Lottery {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (text, cards) = many1(terminated(Card::parse_wyz, newline))(text)?;
		Ok((text, Self { cards }))
	}
}

impl Puzzle for Lottery {
	fn part_1(&mut self) -> eyre::Result<i64> {
		eyre::ensure!(!self.cards.is_empty(), "empty card set");
		Ok(self.cards.iter().map(Card::score).sum())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let mut pile = BTreeMap::new();
		for card in self.cards.as_slice() {
			let this = pile.entry(card.ident).or_insert(0);
			*this += 1;
			let this = *this;
			for id in (card.ident + 1) ..= (card.ident + card.matches()) {
				*pile.entry(id).or_insert(0) += this;
			}
		}
		Ok(pile.into_values().sum())
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Card {
	ident: u8,
	wins:  BTreeSet<i8>,
	picks: BTreeSet<i8>,
}

impl Card {
	pub fn matches(&self) -> u8 {
		self.wins.intersection(&self.picks).count() as u8
	}

	pub fn score(&self) -> i64 {
		match self.matches() {
			0 => 0,
			n => 1 << (n - 1),
		}
	}
}

impl<'a> Parsed<&'a str> for Card {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (text, ident) = delimited(
			terminated(tag("Card"), space1),
			parse_number::<u8>,
			tag(":"),
		)(text)?;
		let (text, wins) = terminated(
			many1(delimited(space0, parse_number::<i8>, space0)),
			tag("|"),
		)(text)
		.tap_err(|err| {
			tracing::error!(?err, "failed to parse winning numbers")
		})?;
		let (text, picks) = many1(preceded(space1, parse_number::<i8>))(text)
			.tap_err(|err| {
				tracing::error!(?err, "failed to parse chosen numbers")
			})?;

		Ok((text, Self {
			ident,
			wins: wins.into_iter().collect(),
			picks: picks.into_iter().collect(),
		}))
	}
}
