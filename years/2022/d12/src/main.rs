use std::{
    collections::{BTreeSet, VecDeque},
    error::Error,
    ops::Index,
    str::FromStr,
};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let map: TopoMap = INPUT.parse()?;
    if let Some(route) = map.navigate(|coord| coord == map.origin) {
        println!("part 1: {}", route.len());
    }
    if let Some(route) = map.navigate(|coord| map[coord] == 0) {
        println!("part 2: {}", route.len())
    }
    println!("Hello, world!");
    Ok(())
}

struct TopoMap {
    points: Vec<Vec<u8>>,
    origin: Coords,
    dest: Coords,
}

impl TopoMap {
    fn navigate(&self, is_done: impl Fn(Coords) -> bool) -> Option<Route> {
        let mut visited = BTreeSet::new();

        let mut candidates = VecDeque::new();
        candidates.push_back(Route::new(self.dest));
        let mut finished_routes = Vec::new();
        while let Some(candidate) = candidates.pop_front() {
            let end = candidate.end();
            if !visited.insert(end) {
                continue;
            }
            for neighbor in self.neighbors_for(end).into_iter().flatten() {
                let path = candidate.push(neighbor);
                if is_done(neighbor) {
                    finished_routes.push(path);
                } else {
                    candidates.push_back(path);
                }
            }
        }
        finished_routes.into_iter().min_by_key(|r| r.steps.len())
    }

    fn neighbors_for(&self, point: Coords) -> [Option<Coords>; 4] {
        let Coords { x, y } = point;
        let n = y.checked_sub(1);
        let e = x + 1;
        let s = y + 1;
        let w = x.checked_sub(1);

        let mut n = n.map(|y| Coords { x, y });
        let mut e = if e as usize == self.points[0].len() {
            None
        } else {
            Some(Coords { x: e, y })
        };
        let mut s = if s as usize == self.points.len() {
            None
        } else {
            Some(Coords { x, y: s })
        };
        let mut w = w.map(|x| Coords { x, y });

        if let Some(pt) = n {
            if self[point] > self[pt] + 1 {
                n = None;
            }
        }
        if let Some(pt) = e {
            if self[point] > self[pt] + 1 {
                e = None;
            }
        }
        if let Some(pt) = s {
            if self[point] > self[pt] + 1 {
                s = None;
            }
        }
        if let Some(pt) = w {
            if self[point] > self[pt] + 1 {
                w = None;
            }
        }

        [n, e, s, w]
    }
}

impl Index<Coords> for TopoMap {
    type Output = u8;

    fn index(&self, Coords { x, y }: Coords) -> &u8 {
        &self.points[y as usize][x as usize]
    }
}

impl FromStr for TopoMap {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut origin = None;
        let mut dest = None;
        let points = s
            .lines()
            .enumerate()
            .map(|(row, line)| {
                line.bytes()
                    .enumerate()
                    .map(|(col, b)| match b {
                        b'S' => {
                            origin = Some(Coords {
                                x: col as u8,
                                y: row as u8,
                            });
                            0
                        }
                        b'E' => {
                            dest = Some(Coords {
                                x: col as u8,
                                y: row as u8,
                            });
                            25
                        }
                        b'a'..=b'z' => b - b'a',
                        _ => unreachable!("the input does not have other data"),
                    })
                    .collect()
            })
            .collect();
        let origin = origin.ok_or("did not find origin point")?;
        let dest = dest.ok_or("did not find destination point")?;
        Ok(Self {
            points,
            origin,
            dest,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Route {
    steps: Vec<Coords>,
}

impl Route {
    fn new(pt: Coords) -> Self {
        Self { steps: vec![pt] }
    }

    fn end(&self) -> Coords {
        *self.steps.last().expect("routes are never empty")
    }

    fn push(&self, pt: Coords) -> Self {
        let mut out = self.clone();
        out.steps.push(pt);
        out
    }

    fn len(&self) -> usize {
        self.steps
            .len()
            .checked_sub(1)
            .expect("routes always have an initial point")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Coords {
    x: u8,
    y: u8,
}
