use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
    str::FromStr,
};

use wyz_aoc::{Coord2D, Grid2D};

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let coords = INPUT
        .lines()
        .flat_map(|l| l.parse::<Coord3D>())
        .collect::<Vec<_>>();

    let adjacencies = compute_adjacencies_in(&coords);
    let pt1 = (coords.len() * 6) - (adjacencies * 2);
    println!("part 1: {pt1}");

    let [x_min, x_max, y_min, y_max, z_min, z_max] =
        coords.iter().copied().fold([0; 6], |mut accum, next| {
            accum[0] = accum[0].min(next.x);
            accum[1] = accum[1].max(next.x);

            accum[2] = accum[2].min(next.y);
            accum[3] = accum[3].max(next.y);

            accum[4] = accum[4].min(next.z);
            accum[5] = accum[5].max(next.z);

            accum
        });

    let mut pts = 0;
    let mut grid = (x_min - 1..=x_max + 1)
        .into_iter()
        .flat_map(|x| {
            (y_min - 1..=y_max + 1).into_iter().flat_map(move |y| {
                (z_min - 1..=z_max + 1)
                    .into_iter()
                    .map(move |z| Coord3D { x, y, z })
            })
        })
        .fold(BTreeMap::new(), |mut planes, Coord3D { x, y, z }| {
            planes
                .entry(x)
                .or_insert_with(Grid2D::default)
                .rows
                .entry(y)
                .or_default()
                .insert(z, Fill::Air);
            pts += 1;
            planes
        });
    for Coord3D { x, y, z } in coords.iter().copied() {
        grid.entry(x)
            .or_default()
            .rows
            .entry(y)
            .or_default()
            .insert(z, Fill::Lava);
    }
    let grid = Grid3D { planes: grid };
    let pt2 = grid.reachable_faces();
    println!("part 2: {pt2}");
}

/// Finds the number of faces shared by two cubes.
fn compute_adjacencies_in(points: &[Coord3D]) -> usize {
    // Collect lines in which to look for adjacencies
    let mut xy = BTreeMap::new();
    let mut yz = BTreeMap::new();
    let mut xz = BTreeMap::new();
    for &Coord3D { x, y, z } in points {
        xy.entry((x, y)).or_insert_with(Vec::new).push(z);
        yz.entry((y, z)).or_insert_with(Vec::new).push(x);
        xz.entry((x, z)).or_insert_with(Vec::new).push(y);
    }
    let mut all = [&mut xy, &mut yz, &mut xz];
    all.iter_mut()
        .map(|planes| {
            planes
                .values_mut()
                .map(|plane| {
                    plane.sort();
                    plane
                        .windows(2)
                        .map(|w| unsafe { &*w.as_ptr().cast::<[i32; 2]>() })
                        .filter(|&&[left, right]| left + 1 == right)
                        .count()
                })
                .sum::<usize>()
        })
        .sum::<usize>()
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Grid3D {
    /// Contains a collection of yz planes stacked along the x axis.
    planes: BTreeMap<i32, Grid2D<i32, Fill>>,
}

impl Grid3D {
    /// Counts how many *faces* reachable from the outside of the volume belong
    /// to a lava cube.
    ///
    /// This is a BFS path search through the grid volume. It requires that the
    /// grid be *fully* populated to enclose the search volume, so that each
    /// traversable point has a fill marker. It moves from cube-center to
    /// cube-center, and tests the fill of each axial neighbor at each point it
    /// visits.
    ///
    /// Axial neighbors filled with lava increment the sum. A given lava point
    /// is axial neighbor to six other points, and they all must be allowed to
    /// try to reach it in order to count all of its faces.
    ///
    /// The search pattern expands into each axial neighbor that is filled with
    /// air and has *not* already been visited.
    fn reachable_faces(&self) -> usize {
        let corner = self
            .planes
            .iter()
            .next()
            .and_then(|(&x, yz)| yz.rows.iter().next().map(|(&y, zs)| (x, y, zs)))
            .and_then(|(x, y, zs)| zs.keys().next().map(|&z| (x, y, z)))
            .map(|(x, y, z)| Coord3D { x, y, z })
            .expect("volume has been filled");
        let mut searched = BTreeSet::new();
        let mut search_queue = VecDeque::new();
        search_queue.push_back(corner);
        let mut lava_faces = 0;
        let mut longest_queue = 0;
        while let Some(pt) = search_queue.pop_front() {
            searched.insert(pt);
            for neighbor in pt.neighbors(1, 1) {
                match self.lookup(&neighbor) {
                    Some(Fill::Lava) => lava_faces += 1,
                    Some(Fill::Air) => {
                        if !searched.contains(&neighbor) {
                            search_queue.push_back(neighbor)
                        }
                    }
                    None => continue,
                }
            }
            longest_queue = longest_queue.max(search_queue.len());
        }
        println!("longest search queue: {longest_queue}");
        lava_faces
    }

    fn lookup(&self, coord: &Coord3D) -> Option<Fill> {
        let Coord3D { x, y, z } = coord;
        self.planes
            .get(x)
            .and_then(|yz| yz.rows.get(y))
            .and_then(|zs| zs.get(z))
            .copied()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Coord3D {
    x: i32,
    y: i32,
    z: i32,
}

impl Coord3D {
    /// Produces all nearby coordinates which are not itself. The "radius" is
    /// the search distance along each axis, and the max distance is measured in
    /// Manhattan steps.
    fn neighbors(&self, radius: i32, max_distance: i32) -> Vec<Self> {
        let mut out = Vec::new();
        for x in -radius..=radius {
            for y in -radius..=radius {
                for z in -radius..=radius {
                    let mh_dist = x.abs() + y.abs() + z.abs();
                    if mh_dist == 0 || mh_dist > max_distance {
                        continue;
                    }
                    out.push(Self {
                        x: self.x + x,
                        y: self.y + y,
                        z: self.z + z,
                    });
                }
            }
        }
        out
    }
}

impl FromStr for Coord3D {
    type Err = String;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut nums = line.split(',').take(3).map(|w| w.parse::<i32>());
        let x = nums
            .next()
            .ok_or("missing coords")?
            .map_err(|_| "invalid integer")?;
        let y = nums
            .next()
            .ok_or("missing coords")?
            .map_err(|_| "invalid integer")?;
        let z = nums
            .next()
            .ok_or("missing coords")?
            .map_err(|_| "invalid integer")?;
        Ok(Self { x, y, z })
    }
}

impl fmt::Display for Coord3D {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Fill {
    #[default]
    Air,
    Lava,
}
