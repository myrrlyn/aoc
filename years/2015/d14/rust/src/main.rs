use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, i32 as get_i32},
    combinator::map,
    sequence::tuple,
    IResult,
};

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let racers = INPUT
        .lines()
        .flat_map(Reindeer::parse)
        .map(|(_, r)| (r, (0, 0)))
        .collect();
    let mut race = Race { racers, clock: 0 };

    for _ in 0..2503 {
        race.tick();
    }
    let pt2 = race.racers.values().map(|(_, s)| *s).max().unwrap_or(0);
    println!("part 2: {pt2}");
}

#[derive(Clone, Debug, Default)]
struct Race {
    racers: HashMap<Reindeer, (i32, i32)>,
    clock: i32,
}

impl Race {
    fn tick(&mut self) {
        let mut best = 0;
        for (racer, (distance, _)) in &mut self.racers {
            let interval = racer.run_time + racer.rest_time;
            if self.clock % interval < racer.run_time {
                *distance += racer.speed;
            }
            best = best.max(*distance);
        }
        for (distance, score) in self.racers.values_mut() {
            if *distance == best {
                *score += 1;
            }
        }
        self.clock += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Reindeer {
    name: &'static str,
    speed: i32,
    run_time: i32,
    rest_time: i32,
}

impl Reindeer {
    fn parse(line: &'static str) -> IResult<&'static str, Self> {
        map(
            tuple((
                alpha1,
                tag(" can fly "),
                get_i32,
                tag(" km/s for "),
                get_i32,
                tag(" seconds, but then must rest for "),
                get_i32,
                tag(" seconds."),
            )),
            |(name, _, speed, _, run_time, _, rest_time, _)| Self {
                name,
                speed,
                run_time,
                rest_time,
            },
        )(line)
    }
}
