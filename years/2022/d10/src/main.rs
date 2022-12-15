use bitvec::prelude::*;
use std::{collections::BTreeMap, error::Error, str::FromStr};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let program = INPUT
        .lines()
        .map(Opcode::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    let ticks = [20, 60, 100, 140, 180, 220];
    let mut moments = BTreeMap::new();
    let mut part1 = Processor::new();
    part1.run_program(&program, |tick, regx| {
        if ticks.contains(&tick) {
            moments.insert(tick, tick as i64 * regx);
        }
    });
    let sum: i64 = moments.values().sum();
    println!("part1: {sum}");

    let mut part2 = Screen::new();
    part2.run_program(&program);
    let screen = part2.render()?;
    print!("{screen}");

    Ok(())
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Screen {
    pixels: [BitArr!(for 40); 6],
    processor: Processor,
}

impl Screen {
    fn new() -> Self {
        Self {
            pixels: [bitarr![0; 40]; 6],
            processor: Processor::new(),
        }
    }

    fn run_program(&mut self, program: &[Opcode]) {
        let pixels = &mut self.pixels;
        let processor = &mut self.processor;
        processor.run_program(program, move |tick, regx| {
            let tick = tick - 1;
            let row = tick / 40;
            let col = tick as i64 % 40;
            if (regx - 1..=regx + 1).contains(&col) {
                pixels[row].set(col as usize, true);
            }
        });
    }

    fn render(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut out = String::new();
        for row in self.pixels {
            for col in row {
                out.write_str(if col { "#" } else { " " })?;
            }
            writeln!(&mut out)?;
        }
        Ok(out)
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Processor {
    clock: usize,
    regx: i64,
}

impl Processor {
    fn new() -> Self {
        Self { clock: 0, regx: 1 }
    }

    fn run_program(&mut self, program: &[Opcode], mut on_tick: impl FnMut(usize, i64)) {
        for opcode in program {
            for _tick in 0..opcode.ticks() {
                self.clock += 1;
                on_tick(self.clock, self.regx);
            }
            match *opcode {
                Opcode::Noop => {}
                Opcode::Addx(addend) => self.regx += addend as i64,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Opcode {
    #[default]
    Noop,
    Addx(i32),
}

impl Opcode {
    fn ticks(&self) -> usize {
        match self {
            Self::Noop => 1,
            Self::Addx(_) => 2,
        }
    }
}

impl FromStr for Opcode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            return Ok(Self::Noop);
        };
        if let Some(num) = s.strip_prefix("addx ") {
            return num
                .parse()
                .map_err(|_| "addx instruction carried invalid number")
                .map(Self::Addx);
        }
        Err("unknown instruction")
    }
}
