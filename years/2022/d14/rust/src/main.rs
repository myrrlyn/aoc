use std::{collections::BTreeMap, fmt};

use nom::{
    bytes::complete::tag, character::complete::u16 as get_u16, combinator::map,
    multi::separated_list1, sequence::separated_pair, IResult,
};

use wyz_aoc::Puzzle;

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let rocks = INPUT
        .lines()
        .flat_map(|line| Rocks::parse(line).ok().map(|(_, rocks)| rocks));
    let mut cave = Cave::new();

    cave.fill_with_rock(rocks);
    cave.fill_with_sand(false);
    cave.clear_source();
    let pt1 = cave.count_sand();
    println!("part 1: {pt1}");

    cave.reset();
    cave.fill_with_sand(true);
    let pt2 = cave.count_sand();
    println!("part 2: {pt2}");
}

struct Y2022D14;

impl Puzzle for Y2022D14 {
    type Input = Vec<Rocks>;
    type State = Cave;
    type ParseError<'a> = String;
    type ComputeError = String;

    fn parse(input: &str) -> Result<Self::Input, Self::ParseError<'_>> {
        input
            .lines()
            .map(|line| {
                Rocks::parse(line)
                    .map(|(_, rocks)| rocks)
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    fn prepare_state(input: Self::Input) -> Result<Self::State, Self::ComputeError> {
        let mut cave = Cave::new();
        cave.fill_with_rock(input.into_iter());
        Ok(cave)
    }

    fn part_1(state: &mut Self::State) -> Result<i64, Self::ComputeError> {
        state.fill_with_sand(false);
        state.clear_source();
        Ok(state.count_sand() as i64)
    }

    fn part_2(state: &mut Self::State) -> Result<i64, Self::ComputeError> {
        state.reset();
        state.fill_with_sand(true);
        Ok(state.count_sand() as i64)
    }
}

struct Cave {
    space: BTreeMap<Coords, Fill>,
    min_x: usize,
    max_x: usize,
    max_y: usize,
}

impl Cave {
    const SOURCE: Coords = Coords { x: 500, y: 0 };
    fn new() -> Self {
        Self {
            space: BTreeMap::new(),
            min_x: !0,
            max_x: 0,
            max_y: 0,
        }
    }

    fn fill_with_rock(&mut self, rocks: impl Iterator<Item = Rocks>) {
        for rock in rocks {
            self.space.extend(rock.points().map(|c| (c, Fill::Rock)));
        }
        for &Coords { x, y } in self.space.keys() {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
            self.max_y = self.max_y.max(y);
        }
    }

    fn fill_with_sand(&mut self, infinite_floor: bool) {
        self.space.insert(Self::SOURCE, Fill::Sand);
        let (min_x, max_x, max_y) = if infinite_floor {
            (0, !0, self.max_y + 1)
        } else {
            (self.min_x, self.max_x, self.max_y)
        };
        let mut counter = 1;
        'outer: loop {
            println!("placing grain {counter}");
            let mut pt = Self::SOURCE;
            'inner: loop {
                // Get the point south of current. If there is no such point,
                // the sand is leaving the area; quit.
                let south = if let Some(pt) = pt.south(max_y) {
                    pt
                } else if infinite_floor {
                    let south = Coords {
                        x: pt.x,
                        y: pt.y + 1,
                    };
                    self.space.insert(south, Fill::Rock);
                    south
                } else {
                    println!("Abyss at {pt}");
                    break 'outer;
                };
                // If the point is filled with air, move into that point and
                // continue the search.
                if let Fill::Air = self.space.entry(south).or_default() {
                    pt = south;
                    continue;
                }

                // The above block spun the loop until `south` is not air. Try
                // the southwest point.
                let southwest = if let Some(pt) = pt.southwest(min_x, max_y) {
                    pt
                } else if infinite_floor {
                    let southwest = Coords {
                        x: pt.x.saturating_sub(1),
                        y: pt.x + 1,
                    };
                    self.space.insert(southwest, Fill::Rock);
                    southwest
                } else {
                    println!("Abyss at {pt}");
                    break 'outer;
                };
                // If it is air, move into it.
                if let Fill::Air = self.space.entry(southwest).or_default() {
                    pt = southwest;
                    continue;
                }

                // The above block spun the loop until both `south` and
                // `southwest` are not air. Try the southeast point.
                let southeast = if let Some(pt) = pt.southeast(max_x, max_y) {
                    pt
                } else if infinite_floor {
                    let southeast = Coords {
                        x: pt.x + 1,
                        y: pt.x + 1,
                    };
                    self.space.insert(southeast, Fill::Rock);
                    southeast
                } else {
                    println!("Abyss at {pt}");
                    break 'outer;
                };
                // If it is air, move into it.
                if let Fill::Air = self.space.entry(southeast).or_default() {
                    pt = southeast;
                    continue;
                }

                println!("Sand: {pt}");
                // If no motion took place, become sand and start another route.
                // If this point is *already* sand, quit entirely.
                if let Some(Fill::Sand) = self.space.insert(pt, Fill::Sand) {
                    println!("Source blocked!");
                    break 'outer;
                } else {
                    break 'inner;
                }
            }
            counter += 1;
        }
    }

    fn clear_source(&mut self) {
        self.space.insert(Self::SOURCE, Fill::Air);
    }

    fn reset(&mut self) {
        self.space.retain(Fill::is_rock);
    }

    fn count_sand(&self) -> usize {
        self.space.values().filter(Fill::is_sand).count()
    }
}

struct Rocks {
    vertices: Vec<Coords>,
}

impl Rocks {
    fn parse(line: &str) -> IResult<&str, Self> {
        map(separated_list1(tag(" -> "), Coords::parse), |vertices| {
            Self { vertices }
        })(line)
    }

    fn points(&self) -> impl '_ + Iterator<Item = Coords> {
        self.vertices
            .windows(2)
            .flat_map(|coords| coords[0].line_between(coords[1]))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Coords {
    x: usize,
    y: usize,
}

impl Coords {
    fn parse(text: &str) -> IResult<&str, Self> {
        map(separated_pair(get_u16, tag(","), get_u16), |(x, y)| Self {
            x: x as usize,
            y: y as usize,
        })(text)
    }

    fn line_between(self, other: Self) -> Vec<Self> {
        // Vertical
        if self.x == other.x {
            if self.y < other.y {
                self.y..=other.y
            } else {
                other.y..=self.y
            }
            .into_iter()
            .map(|y| Self { x: self.x, y })
            .collect()
        }
        // Horizontal
        else if self.y == other.y {
            if self.x < other.x {
                self.x..=other.x
            } else {
                other.x..=self.x
            }
            .into_iter()
            .map(|x| Self { x, y: self.y })
            .collect()
        } else {
            unreachable!("the input cannot contain diagonals")
        }
    }

    fn south(self, max_y: usize) -> Option<Self> {
        if self.y >= max_y {
            return None;
        }
        Some(Self {
            x: self.x,
            y: self.y + 1,
        })
    }

    fn southeast(self, max_x: usize, max_y: usize) -> Option<Self> {
        if self.x >= max_x || self.y >= max_y {
            return None;
        }
        Some(Self {
            x: self.x + 1,
            y: self.y + 1,
        })
    }

    fn southwest(self, min_x: usize, max_y: usize) -> Option<Self> {
        if self.x <= min_x || self.y >= max_y {
            return None;
        }
        Some(Self {
            x: self.x.saturating_sub(1),
            y: self.y + 1,
        })
    }
}

impl fmt::Display for Coords {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Fill {
    #[default]
    Air,
    Rock,
    Sand,
}

impl Fill {
    fn is_sand(this: &&Self) -> bool {
        matches!(this, Self::Sand)
    }

    fn is_rock(_: &Coords, this: &mut Self) -> bool {
        matches!(this, Self::Rock)
    }
}
