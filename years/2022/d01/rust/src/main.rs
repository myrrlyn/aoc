use std::collections::BTreeSet;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let elves = count_elves(INPUT);
    println!("part 1: {}", top_n(&elves, 1));
    println!("part 2: {}", top_n(&elves, 3));
}

fn count_elves(text: &str) -> BTreeSet<i32> {
    text.split("\n\n")
        .map(|block| {
            block
                .lines()
                .flat_map(|line| line.parse::<i32>())
                .sum::<i32>()
        })
        .collect::<BTreeSet<i32>>()
}

fn top_n(elves: &BTreeSet<i32>, n: usize) -> i32 {
    elves.iter().copied().rev().take(n).sum()
}
