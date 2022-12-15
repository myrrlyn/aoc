use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};

use nom::combinator::value;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i64 as get_i64, newline, u64 as get_u64},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded},
    IResult,
};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let (_, mut monkeys_1) =
        separated_list1(newline, Monkey::parse)(INPUT).map_err(|e| format!("{e}"))?;
    let mut monkeys_2 = monkeys_1.clone();
    let lcm = monkeys_1.iter().map(|m| m.modulus).product::<i64>();

    let (mut senders, mut receivers) = (
        Vec::with_capacity(monkeys_1.len()),
        Vec::with_capacity(monkeys_1.len()),
    );
    for _ in 0..monkeys_1.len() {
        let (s, r) = mpsc::channel();
        senders.push(s);
        receivers.push(r);
    }
    for _round in 0..20 {
        for monkey in monkeys_1.iter_mut() {
            monkey.throw_items(&receivers[monkey.id], &senders, 3, lcm);
        }
    }
    let mut inspections = monkeys_1.iter().map(|m| m.inspections).collect::<Vec<_>>();
    inspections.sort();
    inspections.reverse();
    println!(
        "part 1: {}",
        inspections.iter().copied().take(2).product::<usize>()
    );

    senders.clear();
    receivers.clear();
    for _ in 0..monkeys_2.len() {
        let (s, r) = mpsc::channel();
        senders.push(s);
        receivers.push(r);
    }
    for round in 1..=10_000 {
        if round % 100 == 0 {
            println!("round {round}");
        }
        for monkey in monkeys_2.iter_mut() {
            monkey.throw_items(&receivers[monkey.id], &senders, 1, lcm);
        }
    }
    inspections.clear();
    inspections = monkeys_2.iter().map(|m| m.inspections).collect();
    inspections.sort();
    inspections.reverse();
    println!(
        "part 2: {}",
        inspections.iter().copied().take(2).product::<usize>()
    );
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Monkey {
    id: usize,
    held_items: Vec<i64>,
    operation: Operation,
    modulus: i64,
    on_true: usize,
    on_false: usize,
    inspections: usize,
}

impl Monkey {
    fn parse(text: &str) -> IResult<&str, Self> {
        let (rest, id) = delimited(tag("Monkey "), get_u64, tag(":\n"))(text)?;
        let (rest, held_items) = delimited(
            tag("  Starting items: "),
            separated_list1(tag(", "), map(get_i64, i64::from)),
            newline,
        )(rest)?;
        let (rest, operation) = delimited(tag("  "), Operation::parse, newline)(rest)?;
        let (rest, divisor) = delimited(tag("  Test: divisible by "), get_i64, newline)(rest)?;
        let (rest, on_true) =
            delimited(tag("    If true: throw to monkey "), get_u64, newline)(rest)?;
        let (rest, on_false) =
            delimited(tag("    If false: throw to monkey "), get_u64, newline)(rest)?;
        Ok((
            rest,
            Self {
                id: id as usize,
                held_items,
                operation,
                modulus: divisor,
                on_true: on_true as usize,
                on_false: on_false as usize,
                inspections: 0,
            },
        ))
    }

    fn throw_items(
        &mut self,
        inbox: &Receiver<i64>,
        senders: &[Sender<i64>],
        worry_divisor: i64,
        lcm: i64,
    ) {
        self.held_items.extend(inbox.try_iter());
        for item in self.held_items.drain(..) {
            self.inspections += 1;
            let new_worry = (self.operation.compute(item) / worry_divisor) % lcm;
            let modulus = new_worry % self.modulus;
            senders[if modulus == i64::from(0) {
                self.on_true
            } else {
                self.on_false
            }]
            .send(new_worry)
            .expect("monkey has not hung up");
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Operation {
    Add(i64),
    Mul(i64),
    Square,
}

impl Operation {
    fn parse(text: &str) -> IResult<&str, Self> {
        preceded(
            tag("Operation: new = old "),
            alt((
                map(preceded(tag("+ "), get_i64), Operation::Add),
                map(preceded(tag("* "), get_i64), Operation::Mul),
                value(Self::Square, tag("* old")),
            )),
        )(text)
    }

    fn compute(&self, value: i64) -> i64 {
        match *self {
            Self::Add(a) => value + a,
            Self::Mul(m) => value
                .checked_mul(m)
                .unwrap_or_else(|| panic!("cannot compute {value} * {m}")),
            Self::Square => value
                .checked_mul(value)
                .unwrap_or_else(|| panic!("cannot compute {value}^2")),
        }
    }
}
