use std::str::FromStr;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    println!(
        "guide score 1: {}",
        read_games_1().map(|g| g.score()).sum::<usize>()
    );
    println!(
        "guide score 2: {}",
        read_games_2().map(|g| g.score()).sum::<usize>()
    );
}

fn read_games_1() -> impl Iterator<Item = Game> {
    INPUT
        .lines()
        .filter_map(|line| line.parse().map_err(|e| println!("{e}")).ok())
}

fn read_games_2() -> impl Iterator<Item = Game2> {
    INPUT
        .lines()
        .filter_map(|line| line.parse().map_err(|e| println!("{e}")).ok())
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Move {
    #[default]
    Rock = 1,
    Paper,
    Scissors,
}

impl Move {
    fn score(&self) -> usize {
        *self as usize
    }
}

impl FromStr for Move {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            _ => Err("unknown symbol"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Game {
    them: Move,
    mine: Move,
}

impl Game {
    fn score(&self) -> usize {
        self.mine.score()
            + match (&self.them, &self.mine) {
                // they win
                (Move::Rock, Move::Scissors)
                | (Move::Paper, Move::Rock)
                | (Move::Scissors, Move::Paper) => 0,
                // we win
                (Move::Rock, Move::Paper)
                | (Move::Paper, Move::Scissors)
                | (Move::Scissors, Move::Rock) => 6,
                // draw
                _ => 3,
            }
    }
}

impl FromStr for Game {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut letters = s.split_whitespace();
        let them = letters.next().ok_or("too few moves")?.parse()?;
        let mine = letters.next().ok_or("too few moves")?.parse()?;
        if letters.next().is_some() {
            return Err("too many moves");
        }
        Ok(Self { them, mine })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Outcome {
    Loss = 0,
    #[default]
    Draw = 3,
    Win = 6,
}

impl Outcome {
    fn score(&self) -> usize {
        *self as usize
    }
}

impl FromStr for Outcome {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "X" => Ok(Self::Loss),
            "Y" => Ok(Self::Draw),
            "Z" => Ok(Self::Win),
            _ => Err("unknown symbol"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Game2 {
    them: Move,
    outcome: Outcome,
}

impl Game2 {
    fn my_move(&self) -> Move {
        match (self.them, self.outcome) {
            (Move::Rock, Outcome::Loss) | (Move::Paper, Outcome::Win) => Move::Scissors,
            (Move::Paper, Outcome::Loss) | (Move::Scissors, Outcome::Win) => Move::Rock,
            (Move::Scissors, Outcome::Loss) | (Move::Rock, Outcome::Win) => Move::Paper,
            (m, Outcome::Draw) => m,
        }
    }

    fn score(&self) -> usize {
        self.my_move().score() + self.outcome.score()
    }
}

impl FromStr for Game2 {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut line = s.split_whitespace();
        let them = line.next().ok_or("no move")?.parse()?;
        let outcome = line.next().ok_or("no outcome")?.parse()?;
        if line.next().is_some() {
            return Err("too many symbols");
        }
        Ok(Self { them, outcome })
    }
}
