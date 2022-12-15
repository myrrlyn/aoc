use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alpha1,
    character::complete::i32 as get_i32,
    combinator::{map, value},
    sequence::tuple,
    IResult,
};
use std::collections::BTreeMap;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let relations = INPUT
        .lines()
        .flat_map(Relation::parse)
        .map(|(_, r)| r)
        .fold(
            BTreeMap::new(),
            |mut graph, Relation { left, right, delta }| {
                graph
                    .entry(left)
                    .or_insert_with(BTreeMap::new)
                    .insert(right, delta);
                graph
            },
        );
    let mut relations = Relations { store: relations };
    let (_, score) = relations
        .permute()
        // .inspect(|(names, score)| println!("{names:?} => {score}"))
        .max_by_key(|(_, v)| *v)
        .expect("a permutation exists");
    println!("part 1: {score}");

    for slot in relations.store.values_mut() {
        slot.insert("self", 0);
    }
    let names = relations
        .store
        .keys()
        .map(|k| &**k)
        .map(|n| (n, 0))
        .collect();
    relations.store.insert("self", names);
    let (_, score) = relations
        .permute()
        .max_by_key(|(_, v)| *v)
        .expect("a permutation exists");
    println!("part 2: {score}");
}

struct Relations<'a> {
    store: BTreeMap<&'a str, BTreeMap<&'a str, i32>>,
}

impl<'a> Relations<'a> {
    fn permute(&'a self) -> impl 'a + Iterator<Item = (Vec<&'a str>, i32)> {
        self.store
            .keys()
            .map(|k| &**k)
            .permutations(self.store.len())
            .map(|names| {
                let score = self.score_for(names.as_slice());
                (names, score)
            })
    }

    fn score_for(&self, names: &[&'a str]) -> i32 {
        let len = names.len();
        names
            .windows(2)
            .chain(Some([names[len - 1], names[0]].as_slice()))
            .map(|pair| {
                let [left, right] = [pair[0], pair[1]];
                let left_score = self
                    .store
                    .get(left)
                    .and_then(|l| l.get(right))
                    .copied()
                    .unwrap_or(0);
                let right_score = self
                    .store
                    .get(right)
                    .and_then(|r| r.get(left))
                    .copied()
                    .unwrap_or(0);
                left_score + right_score
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Relation<'a> {
    left: &'a str,
    right: &'a str,
    delta: i32,
}

impl<'a> Relation<'a> {
    fn parse(line: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                alpha1,
                tag(" would "),
                alt((value(-1, tag("lose ")), value(1, tag("gain ")))),
                get_i32,
                tag(" happiness units by sitting next to "),
                alpha1,
                tag("."),
            )),
            |(left, _, sign, delta, _, right, _)| Self {
                left,
                right,
                delta: sign * delta,
            },
        )(line)
    }
}
