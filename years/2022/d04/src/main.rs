use std::{ops::RangeInclusive, str::FromStr};

static INPUT: &str = include_str!("../input.txt");

fn main() {
    let supers = part1();
    println!("Number of superset pairs: {supers}");
    let overs = part2();
    println!("Number of superset pairs: {overs}");
}

fn get_ranges() -> impl Iterator<Item = Pair> {
    INPUT.lines().filter_map(|l| l.parse().ok())
}

struct Assign {
    range: RangeInclusive<usize>,
}

impl FromStr for Assign {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let (a, b) = (
            parts.next().ok_or("no start")?,
            parts.next().ok_or("no end")?,
        );
        if parts.next().is_some() {
            return Err("unexpected third component");
        }
        let (a, b) = (
            a.parse().map_err(|_| "not a number")?,
            b.parse().map_err(|_| "not a number")?,
        );
        Ok(Self { range: a..=b })
    }
}

struct Pair {
    left: RangeInclusive<usize>,
    right: RangeInclusive<usize>,
}

impl Pair {
    fn is_superset(&self) -> bool {
        let a = *self.left.start();
        let b = *self.left.end();
        let c = *self.right.start();
        let d = *self.right.end();

        let e = a <= c && b >= d;
        let f = a >= c && b <= d;
        e || f
    }

    fn is_overlapping(&self) -> bool {
        let a = *self.left.start();
        let b = *self.left.end();
        let c = *self.right.start();
        let d = *self.right.end();

        let e = self.left.contains(&c) || self.left.contains(&d);
        let f = self.right.contains(&a) || self.right.contains(&b);

        e || f
    }
}

impl FromStr for Pair {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');
        let (a, b) = (
            parts.next().ok_or("no first range")?,
            parts.next().ok_or("no second range")?,
        );
        if parts.next().is_some() {
            return Err("unexpected third range");
        }
        Ok(Self {
            left: a.parse::<Assign>()?.range,
            right: b.parse::<Assign>()?.range,
        })
    }
}

fn part1() -> usize {
    get_ranges().filter(|p| p.is_superset()).count()
}

fn part2() -> usize {
    get_ranges().filter(|p| p.is_overlapping()).count()
}
