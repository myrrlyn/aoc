use std::{
    collections::BTreeSet,
    error::Error,
    iter::FusedIterator,
    ops::{Add, AddAssign},
};

use nom::{
    bytes::complete::tag, character::complete::i32 as get_i32, character::complete::one_of,
    combinator::map, sequence::separated_pair, IResult,
};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let moves = INPUT
        .lines()
        .map(|line| Move::parse(line).map(|(_, m)| m))
        .collect::<Result<Vec<Move>, _>>()?;

    let mut grid1 = Grid::new(2, &moves);
    grid1.run_program();
    let uniques = grid1.tail_visits.len();
    println!("Part 1: {uniques}");

    let mut grid2 = Grid::new(10, &moves);
    grid2.run_program();
    let uniques = grid2.tail_visits.len();
    println!("Part 2: {uniques}");

    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Move {
    delta: Coords,
}

impl Move {
    fn parse(line: &str) -> IResult<&str, Self> {
        map(
            separated_pair(one_of("URDL"), tag(" "), get_i32),
            |(dir, dist)| match dir {
                'U' => Self {
                    delta: Coords { x: 0, y: dist },
                },
                'R' => Self {
                    delta: Coords { x: dist, y: 0 },
                },
                'D' => Self {
                    delta: Coords { x: 0, y: -dist },
                },
                'L' => Self {
                    delta: Coords { x: -dist, y: 0 },
                },
                _ => unreachable!("the nom parser will not provide an un-asked character"),
            },
        )(line)
    }
}

impl Iterator for Move {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let (this, out) = match self.delta {
            Coords { x: 0, y: 0 } => return None,
            Coords { x, y: y @ 0 } if x > 0 => (Coords { x: x - 1, y }, Coords { x: 1, y }),
            Coords { x, y: y @ 0 } if x < 0 => (Coords { x: x + 1, y }, Coords { x: -1, y }),
            Coords { x: x @ 0, y } if y > 0 => (Coords { x, y: y - 1 }, Coords { x, y: 1 }),
            Coords { x: x @ 0, y } if y < 0 => (Coords { x, y: y + 1 }, Coords { x, y: -1 }),
            _ => unreachable!("this check is, in fact, exhaustive"),
        };
        self.delta = this;
        Some(out)
    }
}

impl ExactSizeIterator for Move {
    fn len(&self) -> usize {
        self.delta.manhattan_distance(&Coords::default()) as usize
    }
}

impl FusedIterator for Move {}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Coords {
    x: i32,
    y: i32,
}

impl Coords {
    fn manhattan_distance(&self, other: &Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn compute_tail_motion(&self, tail: &Self) -> Option<Self> {
        let (dx, dy) = (self.x - tail.x, self.y - tail.y);
        let unit = -1..=1;
        if unit.contains(&dx) && unit.contains(&dy) {
            return None;
        }
        Some(Self {
            x: dx.signum(),
            y: dy.signum(),
        })
    }
}

impl Add<Self> for Coords {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Self> for Coords {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Rope {
    knots: Vec<Coords>,
}

impl Rope {
    fn new(knots: usize) -> Self {
        Self {
            knots: vec![Coords::default(); knots],
        }
    }

    fn apply_move(&mut self, mv: Move) -> BTreeSet<Coords> {
        let mut tail_positions = BTreeSet::new();
        for delta in mv {
            let head = self.knots.first_mut().expect("must have at least one knot");
            *head += delta;
            for idx in 0..(self.knots.len() - 1) {
                let prev = self.knots[idx];
                let next = &mut self.knots[idx + 1];
                if let Some(motion) = prev.compute_tail_motion(&*next) {
                    *next += motion;
                } else {
                    break;
                }
            }
            let tail = self.knots.last().expect("must have at least one knot");
            tail_positions.insert(*tail);
        }
        tail_positions
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Grid<'a> {
    rope: Rope,
    moves: &'a [Move],
    tail_visits: BTreeSet<Coords>,
}

impl<'a> Grid<'a> {
    fn new(knots: usize, moves: &'a [Move]) -> Self {
        Self {
            rope: Rope::new(knots),
            moves,
            ..Self::default()
        }
    }

    fn run_program(&mut self) {
        for mv in self.moves.iter().copied() {
            self.tail_visits.extend(self.rope.apply_move(mv));
        }
    }
}
