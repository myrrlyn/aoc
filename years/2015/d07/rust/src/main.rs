use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, u16 as get_u16},
    combinator::{map, value},
    sequence::{separated_pair, tuple},
    IResult,
};

use std::collections::BTreeMap;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let rules = INPUT
        .lines()
        .flat_map(|line| Production::parse_relation(line).ok().map(|(_, p)| p));
    let mut circuit = Circuit::new(rules);
    let mut circuit2 = circuit.clone();

    circuit.reduce();
    let pt1 = *circuit.values.get("a").expect("failed to solve for `a`");
    println!("part 1: {pt1}");

    circuit2.values.insert("b", pt1);
    circuit2.reduce();
    let pt2 = *circuit2.values.get("a").expect("failed to solve for `a`");
    println!("part 2: {pt2}");
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Circuit<'a> {
    values: BTreeMap<&'a str, u16>,
    relations: BTreeMap<&'a str, Production<'a>>,
}

impl<'a> Circuit<'a> {
    fn new(rules: impl Iterator<Item = (&'a str, Production<'a>)>) -> Self {
        let mut values = BTreeMap::new();
        let mut relations = BTreeMap::new();
        for (name, rule) in rules {
            if let Production::Value(v) = rule {
                values.insert(name, v);
            } else {
                relations.insert(name, rule);
            }
        }
        Self { values, relations }
    }

    fn reduce(&mut self) {
        while !self.relations.is_empty() {
            let mut changes = 0;
            self.relations.retain(|name, rule| match rule {
                Production::Value(v) => {
                    self.values.insert(name, *v);
                    changes += 1;
                    false
                }
                Production::Name(n) => {
                    if let Some(&v) = self.values.get(*n) {
                        *rule = Production::Value(v);
                        changes += 1;
                    }
                    true
                }
                Production::Gate(boxed) => {
                    let (op, lhs, rhs) = &mut **boxed;
                    if let (Production::Value(lhs), Production::Value(rhs)) = (&mut *lhs, &mut *rhs)
                    {
                        let val = op.evaluate(*lhs, *rhs);
                        *rule = Production::Value(val);
                        changes += 1;
                    } else {
                        if let Production::Name(name) = lhs {
                            if let Some(val) = self.values.get(*name) {
                                *lhs = Production::Value(*val);
                                changes += 1;
                            }
                        }
                        if let Production::Name(name) = rhs {
                            if let Some(val) = self.values.get(*name) {
                                *rhs = Production::Value(*val);
                                changes += 1;
                            }
                        }
                    }
                    true
                }
            });
            if changes == 0 {
                break;
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Production<'a> {
    Value(u16),
    Name(&'a str),
    Gate(Box<(Operation, Production<'a>, Production<'a>)>),
}

impl<'a> Production<'a> {
    const ZERO: Self = Self::Value(0);

    fn name_or_lit(text: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(alpha1, Production::Name),
            map(get_u16, Production::Value),
        ))(text)
    }

    fn parse(text: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                separated_pair(Operation::parse, tag(" "), Self::name_or_lit),
                |(o, p)| Production::Gate(Box::new((o, Production::ZERO, p))),
            ),
            map(
                tuple((
                    Self::name_or_lit,
                    tag(" "),
                    Operation::parse,
                    tag(" "),
                    Self::name_or_lit,
                )),
                |(a, _, op, _, b)| Production::Gate(Box::new((op, a, b))),
            ),
            Self::name_or_lit,
        ))(text)
    }

    fn parse_relation(line: &'a str) -> IResult<&'a str, (&'a str, Self)> {
        map(
            separated_pair(Production::parse, tag(" -> "), alpha1),
            |(prod, name)| (name, prod),
        )(line)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Operation {
    And,
    Or,
    Not,
    LShift,
    RShift,
}

impl Operation {
    fn parse(text: &str) -> IResult<&str, Self> {
        alt((
            value(Self::And, tag("AND")),
            value(Self::Or, tag("OR")),
            value(Self::Not, tag("NOT")),
            value(Self::LShift, tag("LSHIFT")),
            value(Self::RShift, tag("RSHIFT")),
        ))(text)
    }

    fn evaluate(self, lhs: u16, rhs: u16) -> u16 {
        match self {
            Self::And => lhs & rhs,
            Self::Or => lhs | rhs,
            Self::Not => !rhs,
            Self::LShift => lhs << rhs,
            Self::RShift => lhs >> rhs,
        }
    }
}
