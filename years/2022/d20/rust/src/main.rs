use std::{fmt, ops::Index};

use tap::{Pipe, Tap};

static INPUT: &str = wyz_aoc::input!();

const MARKERS: [usize; 3] = [1000, 2000, 3000];

const DECRYPTOR: i64 = 811589153;

fn main() {
    let numbers = INPUT
        .lines()
        .flat_map(|l| l.parse::<i64>())
        .collect::<Vec<_>>()
        .pipe(ArenaList::from);
    assert_eq!(
        numbers.len(),
        INPUT.lines().count(),
        "failed to parse some lines"
    );

    let mut numbers_pt1 = numbers.clone();
    numbers_pt1.mix();

    let pt1 = MARKERS.into_iter().map(|idx| numbers_pt1[idx]).sum::<i64>();
    println!("part 1: {pt1}");

    let mut numbers_pt2 = numbers;
    numbers_pt2
        .arena
        .iter_mut()
        .for_each(|Node { value, .. }| *value *= DECRYPTOR);
    for _ in 0..10 {
        numbers_pt2.mix();
    }

    let pt2 = MARKERS.into_iter().map(|idx| numbers_pt2[idx]).sum::<i64>();
    println!("part 2: {pt2}");
}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ArenaList<T> {
    arena: Vec<Node<T>>,
}

impl<T> ArenaList<T> {
    fn len(&self) -> usize {
        self.arena.len()
    }
}

impl ArenaList<i64> {
    fn mix(&mut self) {
        let len = self.arena.len();

        for idx in 0..len {
            let Node { value, prev, next } = self.arena[idx];
            let distance = value.rem_euclid(len as i64 - 1) as usize;

            if distance > 0 {
                let mut pos = idx;
                if distance > len / 2 {
                    for _ in 0..len - distance {
                        pos = self.arena[pos].prev;
                    }
                } else {
                    for _ in 0..distance {
                        pos = self.arena[pos].next;
                    }
                }
                self.arena[next].prev = prev;
                self.arena[prev].next = next;
                let prev = pos;
                let next = self.arena[prev].next;
                self.arena[prev].next = idx;
                self.arena[next].prev = idx;
                self.arena[idx].prev = prev;
                self.arena[idx].next = next;
            }
        }
    }

    fn zero_index(&self) -> Option<usize> {
        self.arena.iter().position(|&Node { value, .. }| value == 0)
    }
}

impl<T> From<Vec<T>> for ArenaList<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            arena: vec
                .into_iter()
                .enumerate()
                .map(|(idx, value)| Node {
                    value,
                    prev: idx.saturating_sub(1),
                    next: idx + 1,
                })
                .collect(),
        }
        .tap_mut(|this| {
            let last = this.len() - 1;
            this.arena[0].prev = last;
            this.arena[last].next = 0;
        })
    }
}

impl Index<usize> for ArenaList<i64> {
    type Output = i64;

    fn index(&self, idx: usize) -> &Self::Output {
        let mut cursor = self.zero_index().expect("indexing requires a zero entry");
        for _ in 0..idx % self.len() {
            cursor = self.arena[cursor].next;
        }
        &self.arena[cursor].value
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Node<T> {
    value: T,
    prev: usize,
    next: usize,
}

impl<T: fmt::Display> fmt::Debug for Node<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.value)
    }
}
