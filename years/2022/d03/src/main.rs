use bitvec::prelude::*;
use std::error::Error;

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let p1 = part1()?;
    println!("Sum of doubled priorities: {p1}");
    let p2 = part2()?;
    println!("Sum of throuples: {p2}");
    Ok(())
}

fn part1() -> Result<usize, Box<dyn Error>> {
    let mut sum = 0;
    for line in INPUT.lines() {
        let pair = Pair::parse(line.trim())?;
        let prio = pair.doubled_priority()?;
        sum += prio;
    }
    Ok(sum)
}

struct Pair<'a> {
    left: &'a str,
    right: &'a str,
}

impl<'a> Pair<'a> {
    fn parse(line: &'a str) -> Result<Self, &'static str> {
        let line = line.trim();
        let len = line.len();
        if len % 2 != 0 {
            eprintln!("[ERROR]: {line} has len {len}");
            return Err("line must have even length");
        }
        let (left, right) = line.split_at(len / 2);
        Ok(Self { left, right })
    }

    fn doubled_priority(&self) -> Result<usize, &'static str> {
        let left = priority_map(self.left);
        let right = priority_map(self.right);

        let intersection = left & right;
        intersection.first_one().ok_or_else(|| {
            eprintln!("[ERROR]: {}/{}", self.left, self.right);
            "no intersection"
        })
    }
}

fn char_to_priority(c: char) -> usize {
    match c {
        'a'..='z' => 1 + (c as usize - 'a' as usize),
        'A'..='Z' => 27 + (c as usize - 'A' as usize),
        _ => 0,
    }
}

fn priority_map(s: &str) -> BitArr!(for 53) {
    let mut out = BitArray::ZERO;
    for idx in s.chars().map(char_to_priority) {
        out.set(idx, true);
    }
    out
}

fn part2() -> Result<usize, Box<dyn Error>> {
    let mut sum = 0;
    for [a, b, c] in Triple::new(INPUT).map_each(priority_map) {
        let all = a & b & c;
        let mut ones = all.iter_ones();
        sum += ones.next().ok_or("no common item")?;
        if ones.next().is_some() {
            return Err("too many common items".into());
        }
    }
    Ok(sum)
}

struct Triple {
    iter: std::str::Lines<'static>,
}

impl Triple {
    fn new(s: &'static str) -> Self {
        Self { iter: s.lines() }
    }

    fn map_each(
        self,
        f: fn(&'static str) -> BitArr!(for 53),
    ) -> impl Iterator<Item = [BitArr!(for 53); 3]> {
        self.map(move |[a, b, c]| {
            let (a, b, c) = (f(a), f(b), f(c));
            [a, b, c]
        })
    }
}

impl Iterator for Triple {
    type Item = [&'static str; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let (a, b, c) = (self.iter.next()?, self.iter.next()?, self.iter.next()?);
        Some([a, b, c])
    }
}
