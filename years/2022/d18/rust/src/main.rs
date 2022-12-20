use wyz_aoc::{Coord3D, Grid3D};

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let coords = INPUT.lines().flat_map(parse_coord).collect::<Vec<_>>();

    let mut droplet = Grid3D::new();
    for coord in coords.iter().copied() {
        droplet.insert(coord, Fill::Lava);
    }

    let faces = coords.len() * 6;
    let mut adjacencies = 0;
    droplet.search_bfs(
        |_| coords.iter().copied(),
        |pt, this, _| {
            for pt2 in pt.nearby(1, 1) {
                if let Some(Fill::Lava) = this.get(pt2) {
                    adjacencies += 1;
                }
            }
        },
    );
    let pt1 = faces - adjacencies;
    println!("part 1: {pt1}");

    let (min, max) = droplet.bounds_inclusive().expect("bounds exist");
    let one = Coord3D::new(1, 1, 1);
    droplet.insert(min - one, Fill::Air);
    droplet.insert(max + one, Fill::Air);
    let mut pt2 = 0;
    droplet.search_bfs(
        |_| [min, max],
        |pt, this, queue| {
            for neighbor in pt.nearby(1, 1) {
                match this.get(neighbor) {
                    Some(Fill::Lava) => pt2 += 1,
                    _ => queue.push_back(neighbor),
                }
            }
        },
    );
    println!("part 2: {pt2}");
}

fn parse_coord(line: &str) -> Result<Coord3D<i32>, &'static str> {
    let mut parts = line.split(',');
    let x = parts
        .next()
        .ok_or("missing x coordinate")?
        .parse()
        .map_err(|_| "invalid integer")?;
    let y = parts
        .next()
        .ok_or("missing y coordinate")?
        .parse()
        .map_err(|_| "invalid integer")?;
    let z = parts
        .next()
        .ok_or("missing z coordinate")?
        .parse()
        .map_err(|_| "invalid integer")?;
    Ok(Coord3D::new(x, y, z))
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Fill {
    #[default]
    Air,
    Lava,
}
