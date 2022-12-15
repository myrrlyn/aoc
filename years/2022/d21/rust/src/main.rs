use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, i64 as get_i64},
    combinator::{map, value},
    sequence::{separated_pair, tuple},
    IResult,
};

use std::collections::BTreeMap;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let rules = INPUT
        .lines()
        .flat_map(|line| Production::parse_relation(line).map(|(_, p)| p));
    let mut monkeys = Monkeys::new(rules);
    let mut monkeys2 = monkeys.clone();

    monkeys.reduce();
    let pt1 = monkeys
        .values
        .get("root")
        .expect("the root monkey should have been solved");
    println!("part 1: {pt1}");

    monkeys2.values.remove("humn");
    match monkeys2
        .relations
        .get_mut("root")
        .expect("root monkey exists")
    {
        Production::Value(_) => panic!("root monkey already solved"),
        Production::Arith(boxed) => boxed.1 = Operation::Eq,
    }

    let stack = monkeys2.try_solve("root").unwrap_err();
    let res = Hole::solve(stack).expect("solvable");

    println!("part 2: {res:?}");
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Monkeys<'a> {
    values: BTreeMap<&'a str, i64>,
    relations: BTreeMap<&'a str, Production<'a>>,
}

impl<'a> Monkeys<'a> {
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
                Production::Arith(boxed) => {
                    let (left, op, right) = &mut **boxed;
                    if let (Some(&lval), Some(&rval)) =
                        (self.values.get(left), self.values.get(right))
                    {
                        *rule = Production::Value(op.evaluate(lval, rval));
                        changes += 1;
                    }
                    true
                }
            });
            if changes == 0 {
                break;
            }
        }
    }

    /// Solves a given monkey, returning either its value, or the name of a
    /// monkey who must be solved first.
    fn try_solve(&mut self, name: &'a str) -> Result<i64, Vec<(&'a str, Hole<'a>)>> {
        if let Some(&val) = self.values.get(name) {
            return Ok(val);
        }
        if let Some(prod) = self.relations.remove(name) {
            match prod {
                Production::Value(val) => {
                    self.values.insert(name, val);
                    Ok(val)
                }
                Production::Arith(boxed) => {
                    let (left, op, right) = *boxed;
                    match (self.try_solve(left), self.try_solve(right)) {
                        (Ok(lval), Ok(rval)) => {
                            let val = op.evaluate(lval, rval);
                            self.values.insert(name, val);
                            Ok(val)
                        }
                        (Ok(lval), Err(mut holes)) => {
                            holes.push((name, Hole::Right(lval, op, right)));
                            Err(holes)
                        }
                        (Err(mut holes), Ok(rval)) => {
                            holes.push((name, Hole::Left(left, op, rval)));
                            Err(holes)
                        }
                        (Err(_), Err(_)) => panic!("both {left} and {right} are unknown"),
                    }
                }
            }
        } else {
            Err(vec![(name, Hole::Base(name))])
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Hole<'a> {
    Base(&'a str),
    Left(&'a str, Operation, i64),
    Right(i64, Operation, &'a str),
}

impl<'a> Hole<'a> {
    fn solve(mut holes: Vec<(&'a str, Self)>) -> Result<i64, Vec<(&'a str, Self)>> {
        let mut value = 0;
        while let Some((_, hole)) = holes.pop() {
            match hole {
                Self::Base(_) => break,
                Self::Left(_, op, right) => match op {
                    // l + right = value
                    Operation::Add => value -= right,
                    // l - right = value
                    Operation::Sub => value += right,
                    // l * right = value
                    Operation::Mul => value /= right,
                    // l / right = value
                    Operation::Div => value *= right,
                    Operation::Eq => value = right,
                },
                Self::Right(left, op, _) => match op {
                    // left + r = value
                    Operation::Add => value -= left,
                    // left - r = value
                    Operation::Sub => value = left - value,
                    // left * r = value
                    Operation::Mul => value /= left,
                    // left / r = value
                    Operation::Div => value = left / value,
                    Operation::Eq => value = left,
                },
            }
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Production<'a> {
    Value(i64),
    Arith(Box<(&'a str, Operation, &'a str)>),
}

impl<'a> Production<'a> {
    fn arith_or_lit(text: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(get_i64, Production::Value),
            map(tuple((alpha1, Operation::parse, alpha1)), |group| {
                Self::Arith(Box::new(group))
            }),
        ))(text)
    }

    fn parse_relation(text: &'a str) -> IResult<&'a str, (&'a str, Self)> {
        separated_pair(alpha1, tag(": "), Self::arith_or_lit)(text)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

impl Operation {
    fn parse(text: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Add, tag(" + ")),
            value(Self::Sub, tag(" - ")),
            value(Self::Mul, tag(" * ")),
            value(Self::Div, tag(" / ")),
        ))(text)
    }

    fn evaluate(self, lhs: i64, rhs: i64) -> i64 {
        match self {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
            Self::Div => lhs / rhs,
            Self::Eq => (lhs == rhs) as i64,
        }
    }
}
