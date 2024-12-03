use std::cmp;

use bitvec::prelude::*;
use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::newline,
	combinator::{
		map,
		value,
	},
	multi::many1,
	sequence::terminated,
};
use tap::Pipe;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 7, |t| t.parse_dyn_puzzle::<Game>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
	hands: Vec<Hand>,
}

impl<'a> Parsed<&'a str> for Game {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		map(many1(terminated(Hand::parse_wyz, newline)), |hands| Self {
			hands,
		})(text)
	}
}

impl Puzzle for Game {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.hands.sort();
		// tracing::debug!(hands = ?self.hands, "sorted");
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.hands
			.iter()
			.map(|h| h.bid)
			.zip(1 ..)
			.map(|(bid, rank)| bid * rank)
			.sum::<i64>()
			.pipe(Ok)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		for hand in &mut self.hands {
			for card in &mut hand.cards {
				if *card == CardRank::Jack {
					*card = CardRank::Joker;
				}
			}
		}
		self.hands.sort();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.part_1()
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hand {
	cards: [CardRank; 5],
	bid:   i64,
}

impl Hand {
	pub fn kind(&self) -> Kind {
		let mut seen: [u8; 14] = [0; 14];
		let mut flags = bitarr![u16, Lsb0; 0; 14];
		let mut joker_mode = false;
		for card in self.cards {
			if card == CardRank::Joker {
				joker_mode = true;
			}
			let idx = card as u8 as usize - 1;
			seen[idx] += 1;
			flags.set(idx, true);
		}
		if !joker_mode {
			let biggest_group = seen.iter().copied().max().unwrap_or_default();
			match (flags.count_ones(), biggest_group) {
				// a a a a a
				(1, _) => Kind::FiveOf,
				// a a a a b
				(2, 4) => Kind::FourOf,
				// a a a b b
				(2, 3) => Kind::FullHouse,
				// a a a b c
				(3, 3) => Kind::ThreeOf,
				// a a b b c
				(3, 2) => Kind::TwoPair,
				// a a b c d
				(4, 2) => Kind::OnePair,
				// a b c d e
				(5, _) => Kind::HighCard,
				(u, b) => unreachable!(
					"found {u} discrete groups, the largest being {b}, with no \
					 jokers"
				),
			}
		}
		else {
			// Jokers do not count as unique ranks
			flags.set(0, false);
			let num_unique = flags.count_ones();
			// Skip the jokers when looking for clusters.
			let biggest_group =
				seen[1 ..].iter().copied().max().unwrap_or_default();
			let num_jokers = seen[0];
			match (num_unique, biggest_group, num_jokers) {
				(0, 0, 5) | (1, ..) => Kind::FiveOf, // J J J J J or x J J J J
				(2, 1, _) => Kind::FourOf,           // a b J J J
				(2, 2, 1) => Kind::FullHouse,        // a a b b J
				(2, 2, 2) => Kind::FourOf,           // a a b J J
				(2, 3, 0) => Kind::FullHouse,        // a a a b b
				(2, 3, 1) | (2, 4, 0) => Kind::FourOf, // a a a b J or a a a a b
				(3, 1, _) => Kind::ThreeOf,          // a b c J J
				(3, 2, 0) => Kind::TwoPair,          // a a b b c
				(3, 2, 1) => Kind::ThreeOf,          // a a b c J
				(4, 1, 0) => Kind::HighCard,         // a b c d e
				(4, 1, 1) => Kind::OnePair,          // a b c d J
				(u, b, j) => unreachable!(
					"found {u} discrete groups, the largest being {b}, with \
					 only {j} joker(s)"
				),
			}
		}
	}
}

impl PartialOrd<Self> for Hand {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		// tracing::debug!(this = ?self.kind(), that = ?other.kind(), "compare");
		match self.kind().cmp(&other.kind()) {
			cmp::Ordering::Equal => {},
			neq => return Some(neq),
		}
		for idx in 0 .. 5 {
			match self.cards[idx].cmp(&other.cards[idx]) {
				cmp::Ordering::Equal => {},
				neq => return Some(neq),
			}
		}
		Some(cmp::Ordering::Equal)
	}
}

impl Ord for Hand {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.partial_cmp(other).expect("total ordering")
	}
}

impl<'a> Parsed<&'a str> for Hand {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (text, one) = text.parse_wyz()?;
		let (text, two) = text.parse_wyz()?;
		let (text, three) = text.parse_wyz()?;
		let (text, four) = text.parse_wyz()?;
		let (text, five) = text.parse_wyz()?;
		let (text, bid) = parse_number(text.trim_start())?;
		Ok((text, Self {
			cards: [one, two, three, four, five],
			bid,
		}))
	}
}

/// Card ranks
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardRank {
	Joker = 1,
	Two   = 2,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	Jack,
	Queen,
	King,
	Ace,
}

impl<'a> Parsed<&'a str> for CardRank {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Ace, tag("A")),
			value(Self::King, tag("K")),
			value(Self::Queen, tag("Q")),
			value(Self::Jack, tag("J")),
			value(Self::Ten, tag("T")),
			value(Self::Nine, tag("9")),
			value(Self::Eight, tag("8")),
			value(Self::Seven, tag("7")),
			value(Self::Six, tag("6")),
			value(Self::Five, tag("5")),
			value(Self::Four, tag("4")),
			value(Self::Three, tag("3")),
			value(Self::Two, tag("2")),
		))(text)
	}
}

/// Possible categorizations of a hand of five cards.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Kind {
	/// All five cards differ. 1 1 1 1 1
	HighCard,
	/// Two cards match, three differ. 2 1 1 1 _
	OnePair,
	/// Two cards match, two others match, one differs. 2 2 1 _ _
	TwoPair,
	/// Three cards match, two differ. 3 1 1 _ _
	ThreeOf,
	/// A triple and a double. 3 2 _ _ _
	FullHouse,
	//// Four match, one differs. 4 1 _ _ _
	FourOf,
	/// All five cards match. 5 _ _ _ _ _
	FiveOf,
}
