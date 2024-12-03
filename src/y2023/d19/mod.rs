#![doc = include_str!("README.md")]

use std::{
	cmp,
	collections::{
		BTreeMap,
		VecDeque,
	},
	fmt,
	ops::{
		Index,
		IndexMut,
		Range,
	},
	sync::RwLock,
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		alpha1,
		newline,
		u16 as get_u16,
	},
	combinator::{
		map,
		value,
	},
	multi::{
		many1,
		separated_list1,
	},
	sequence::{
		delimited,
		pair,
		preceded,
		separated_pair,
		terminated,
		tuple,
	},
};
use tap::Pipe;

use crate::{
	dict::{
		Dictionary,
		Identifier,
	},
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2023, 19, |t| t.parse_dyn_puzzle::<QualityControl>());

#[derive(Clone, Debug)]
pub struct QualityControl {
	rules: BTreeMap<Identifier, RuleSet>,
	items: Vec<Item>,
	names: Dictionary<str>,
	start: Identifier,
}

impl<'a> Parsed<&'a str> for QualityControl {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let names = RwLock::new(Dictionary::new());
		let (_, start) = cached_label("in", &names)?;
		let (rest, (rules, items)) = separated_pair(
			many1(terminated(
				|t| RuleSet::parse_with_cache(t, &names),
				newline,
			)),
			newline,
			many1(terminated(Item::parse_wyz, newline)),
		)(text)?;
		let names = names.into_inner().expect("poisoned name-cache");
		Ok((rest, Self {
			rules: rules.into_iter().collect(),
			items,
			names,
			start,
		}))
	}
}

impl Puzzle for QualityControl {
	fn part_1(&mut self) -> eyre::Result<i64> {
		let mut accum = 0;
		for item in self.items.iter() {
			accum += self.execute(item)?.unwrap_or(0);
		}
		Ok(accum)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		self.compute_acceptance()
	}
}

impl QualityControl {
	pub fn execute(&self, item: &Item) -> eyre::Result<Option<i64>> {
		let mut rule_id = self.start;
		loop {
			let rule = self.get_ruleset(rule_id)?;
			if let Some(name) = self.names.lookup(rule_id) {
				tracing::trace!(%name, "applying ruleset");
			}
			else {
				tracing::warn!(%rule_id, "applying unnamed ruleset");
			}
			match rule.apply(&item) {
				Route::Reject => {
					tracing::trace!("reject");
					return Ok(None);
				},
				Route::Accept => {
					tracing::trace!("accept");
					return Ok(Some(item.score()));
				},
				Route::Forward(next) => rule_id = next,
			}
		}
	}

	pub fn compute_acceptance(&self) -> eyre::Result<i64> {
		let mut queue = VecDeque::new();
		queue.push_back((self.start, ItemSet::ALL));
		let mut accepted = vec![];
		'outer: while let Some((id, mut items)) = queue.pop_front() {
			let ruleset = self.get_ruleset(id)?;
			for rule in &ruleset.rules {
				let (yes, no) = rule.apply_range(items);
				if let Some((yes, next)) = yes {
					match next {
						Route::Reject => {},
						Route::Accept => accepted.push(yes),
						Route::Forward(next) => queue.push_back((next, yes)),
					}
				}
				match no {
					Some(remaining) => items = remaining,
					None => {
						continue 'outer;
					},
				}
			}
			match ruleset.default {
				Route::Reject => {},
				Route::Accept => accepted.push(items),
				Route::Forward(next) => queue.push_back((next, items)),
			}
		}
		accepted
			.iter()
			.map(ItemSet::acceptance)
			.sum::<i64>()
			.pipe(Ok)
	}

	pub fn get_ruleset(&self, id: Identifier) -> eyre::Result<&RuleSet> {
		self.rules.get(&id).ok_or_else(|| {
			let name = self.names.lookup(id);
			let name = name.as_deref().unwrap_or("<unnamed>");
			tracing::error!(%id, %name, "no such rule");
			eyre::eyre!("cannot find rule with ID {id} (name {name})")
		})
	}
}

/// A collection of rules, as well as a default case.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuleSet {
	/// A collection of rules to try in succession against an object.
	rules:   Vec<Rule>,
	/// If none of `.rules` succeed, then this takes effect.
	default: Route,
}

impl RuleSet {
	pub fn apply(&self, item: &Item) -> Route {
		self.rules
			.iter()
			.flat_map(|r| r.apply(item))
			.next()
			.unwrap_or(self.default)
	}

	pub fn parse_with_cache<'a>(
		text: &'a str,
		names: &RwLock<Dictionary<str>>,
	) -> ParseResult<&'a str, (Identifier, Self)> {
		let (rest, (name, (rules, default))) = pair(
			|t| cached_label(t, names),
			delimited(
				tag("{"),
				pair(
					many1(terminated(
						|t| Rule::parse_with_cache(t, names),
						tag(","),
					)),
					|t| Route::parse_with_cache(t, names),
				),
				tag("}"),
			),
		)(text)?;
		Ok((rest, (name, Self { rules, default })))
	}
}

/// A single decider.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Rule {
	/// The attribute being considered
	attr:   Attr,
	/// The attribute's relationship to the threshold value in order to pass the
	/// rule.
	filter: cmp::Ordering,
	/// The threshold value. The rule succeeds for an object if the object's
	/// attribute selected by `.kind` satisfies `.filter` against this.
	value:  Number,
	/// The destination if this rule successfully applies.
	target: Route,
}

impl Rule {
	pub fn apply(&self, item: &Item) -> Option<Route> {
		item[self.attr]
			.pipe(|v| v.cmp(&self.value))
			.pipe(|ord| ord == self.filter)
			.then_some(self.target)
	}

	pub fn apply_range(
		&self,
		items: ItemSet,
	) -> (Option<(ItemSet, Route)>, Option<ItemSet>) {
		let (yes, no) = items.split(self);
		(yes.map(|i| (i, self.target)), no)
	}

	pub fn parse_with_cache<'a>(
		text: &'a str,
		names: &RwLock<Dictionary<str>>,
	) -> ParseResult<&'a str, Self> {
		map(
			tuple((
				Attr::parse_wyz,
				alt((
					value(cmp::Ordering::Greater, tag(">")),
					value(cmp::Ordering::Equal, tag("=")),
					value(cmp::Ordering::Less, tag("<")),
				)),
				get_u16,
				preceded(tag(":"), |t| Route::parse_with_cache(t, names)),
			)),
			|(kind, filter, value, target)| Self {
				attr: kind,
				filter,
				value,
				target,
			},
		)(text)
	}
}

impl fmt::Display for Rule {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(
			fmt,
			"{}{}{}: {}",
			self.attr,
			match self.filter {
				cmp::Ordering::Greater => ">",
				cmp::Ordering::Equal => "==",
				cmp::Ordering::Less => "<",
			},
			self.value,
			self.target
		)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Route {
	/// Terminates rule processing as a success.
	Accept,
	/// Terminates rule processing as a failure.
	Reject,
	/// Continues processing with a new rule-set.
	Forward(Identifier),
}

impl Route {
	pub fn parse_with_cache<'a>(
		text: &'a str,
		names: &RwLock<Dictionary<str>>,
	) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::Accept, tag("A")),
			value(Self::Reject, tag("R")),
			map(|t| cached_label(t, names), Self::Forward),
		))(text)
	}
}

impl fmt::Display for Route {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Accept => fmt.write_str("A"),
			Self::Reject => fmt.write_str("R"),
			Self::Forward(ident) => write!(fmt, "-> {ident}"),
		}
	}
}

/// Various attributes that an object can have.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Attr {
	/// eXtremely cool-looking when seen
	X,
	/// Musical when struck
	M,
	/// Aerodynamic when thrown
	A,
	/// Shiny when lit
	S,
}

impl<'a> Parsed<&'a str> for Attr {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::X, tag("x")),
			value(Self::M, tag("m")),
			value(Self::A, tag("a")),
			value(Self::S, tag("s")),
		))(text)
	}
}

impl fmt::Display for Attr {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str(match self {
			Self::X => "X",
			Self::M => "M",
			Self::A => "A",
			Self::S => "S",
		})
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Item {
	x: Number,
	m: Number,
	a: Number,
	s: Number,
}

impl Item {
	pub fn score(&self) -> i64 {
		[self.x, self.m, self.a, self.s]
			.map(|i| i as i64)
			.into_iter()
			.sum()
	}
}

impl Index<Attr> for Item {
	type Output = Number;

	fn index(&self, index: Attr) -> &Self::Output {
		match index {
			Attr::X => &self.x,
			Attr::M => &self.m,
			Attr::A => &self.a,
			Attr::S => &self.s,
		}
	}
}

impl IndexMut<Attr> for Item {
	fn index_mut(&mut self, index: Attr) -> &mut Self::Output {
		match index {
			Attr::X => &mut self.x,
			Attr::M => &mut self.m,
			Attr::A => &mut self.a,
			Attr::S => &mut self.s,
		}
	}
}

impl<'a> Parsed<&'a str> for Item {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, attrs) = delimited(
			tag("{"),
			separated_list1(
				tag(","),
				separated_pair(Attr::parse_wyz, tag("="), get_u16),
			),
			tag("}"),
		)(text)?;
		let mut this = Self::default();
		for (kind, val) in attrs {
			*match kind {
				Attr::X => &mut this.x,
				Attr::M => &mut this.m,
				Attr::A => &mut this.a,
				Attr::S => &mut this.s,
			} = val;
		}
		Ok((rest, this))
	}
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ItemSet {
	x: Range<Number>,
	m: Range<Number>,
	a: Range<Number>,
	s: Range<Number>,
}

impl ItemSet {
	pub const ALL: Self = Self {
		x: 1 .. 4001,
		m: 1 .. 4001,
		a: 1 .. 4001,
		s: 1 .. 4001,
	};

	/// Splits an item-set according to a rule. The left return value is the set
	/// that matches the rule, and the right return is the set that does not.
	pub fn split(mut self, rule: &Rule) -> (Option<Self>, Option<Self>) {
		let range = self[rule.attr].clone();
		let (yes, no) = match rule.filter {
			cmp::Ordering::Greater => (
				(rule.value + 1) .. range.end,
				range.start .. (rule.value + 1),
			),
			cmp::Ordering::Equal => unreachable!("no equality comparisons"),
			cmp::Ordering::Less => {
				(range.start .. rule.value, rule.value .. range.end)
			},
		};
		let mut other = self.clone();
		other[rule.attr] = no.clone();
		self[rule.attr] = yes.clone();
		(
			(!yes.is_empty()).then_some(self),
			(!no.is_empty()).then_some(other),
		)
	}

	pub fn acceptance(&self) -> i64 {
		[&self.x, &self.m, &self.a, &self.s]
			.map(|i| i.len() as i64)
			.into_iter()
			.product()
	}
}

impl Index<Attr> for ItemSet {
	type Output = Range<Number>;

	fn index(&self, index: Attr) -> &Self::Output {
		match index {
			Attr::X => &self.x,
			Attr::M => &self.m,
			Attr::A => &self.a,
			Attr::S => &self.s,
		}
	}
}

impl IndexMut<Attr> for ItemSet {
	fn index_mut(&mut self, index: Attr) -> &mut Self::Output {
		match index {
			Attr::X => &mut self.x,
			Attr::M => &mut self.m,
			Attr::A => &mut self.a,
			Attr::S => &mut self.s,
		}
	}
}

pub fn cached_label<'a>(
	text: &'a str,
	names: &RwLock<Dictionary<str>>,
) -> ParseResult<&'a str, Identifier> {
	let (rest, name) = alpha1(text)?;
	if let Some(ident) = names
		.read()
		.expect("poisoned name-cache")
		.lookup_value(name)
	{
		return Ok((rest, ident));
	}
	let ident = names.write().expect("poisoned name-cache").insert(name);
	Ok((rest, ident))
}

/// The numeric type that fits the attribute values.
type Number = u16;
