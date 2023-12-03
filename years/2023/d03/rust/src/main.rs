use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag,
    character::complete::{anychar, digit1},
    IResult,
};
use serde::{Deserialize, Serialize};

static INPUT: &str = wyz_aoc::input!();

fn main() -> anyhow::Result<()> {
    let mut schema = Schema {
        part_nums: BTreeMap::new(),
        symbols: BTreeMap::new(),
    };
    for (line_idx, line_txt) in INPUT.lines().enumerate() {
        let line_num = line_idx + 1;
        // let line_txt = line_txt?;
        let (_, found) = parse(line_txt).map_err(|e| anyhow::anyhow!("{e}"))?;
        for Token { bgn, end, val } in found {
            match val {
                Legend::PartNumber(num) => {
                    schema.part_nums.entry(line_num).or_default().push(Token {
                        bgn,
                        end,
                        val: num,
                    });
                }
                Legend::Symbol(sym) => {
                    schema
                        .symbols
                        .entry(line_num)
                        .or_default()
                        .push(Token { bgn, end, val: sym });
                }
            }
        }
    }
    // serde_json::to_writer(io::stdout().lock(), &schema)?;

    let mut useful_nums = vec![];
    let mut line_nums = vec![];
    let mut gears: BTreeMap<(usize, usize), Vec<i32>> = BTreeMap::new();
    for (part_line, part_nums) in &schema.part_nums {
        // println!("{{kind: \"line\", row: {part_line}}}");
        for &Token { bgn, end, val: num } in part_nums {
            // println!("{{kind: \"pt\", row: {part_line}, bgn: {bgn}, end: {end}, val: {num}}}");
            for sym_line in part_line.saturating_sub(1).max(0)..=(part_line + 1) {
                for symbol in schema
                    .symbols
                    .get(&sym_line)
                    .map(Vec::as_slice)
                    .unwrap_or_default()
                {
                    // println!(
                    //     "{{kind: \"sym\", row: {sym_line}, bgn: {sbgn}, end: {send}, val: \"{sym}\"}}",
                    //     sym = symbol.val,
                    //     sbgn = symbol.bgn,
                    //     send = symbol.end,
                    // );
                    if symbol.bgn >= bgn.saturating_sub(1).max(0) && symbol.bgn <= end {
                        // println!("{{kind: \"match\"}}");
                        line_nums.push(num);
                        if symbol.val == '*' {
                            gears.entry((sym_line, symbol.bgn)).or_default().push(num);
                        }
                    }
                }
            }
        }
        // println!("{{line: {line_num}, parts: {line_nums:?}}}");
        useful_nums.append(&mut line_nums);
    }
    // serde_json::to_writer(
    //     io::stdout().lock(),
    //     &gears
    //         .into_iter()
    //         .map(|(k, v)| (format!("{k:?}"), v))
    //         .collect::<BTreeMap<String, Vec<i32>>>(),
    // )?;
    println!("part 1: {}", useful_nums.into_iter().sum::<i32>());
    println!(
        "part 2: {}",
        gears
            .into_values()
            .filter(|v| v.len() == 2)
            .map(|v| v.into_iter().product::<i32>())
            .sum::<i32>()
    );
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Schema {
    part_nums: BTreeMap<usize, Vec<Token<i32>>>,
    symbols: BTreeMap<usize, Vec<Token<char>>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Token<T> {
    pub bgn: usize,
    pub end: usize,
    pub val: T,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
enum Legend {
    PartNumber(i32),
    Symbol(char),
}

fn parse<'a>(mut text: &'a str) -> IResult<&'a str, Vec<Token<Legend>>> {
    let mut idx = 0;
    let mut out = vec![];
    while !text.is_empty() {
        if let Ok((rest, _)) = tag::<&str, &str, ()>(".")(text) {
            idx += 1;
            text = rest;
            continue;
        }
        if let Ok((rest, num)) = digit1::<&str, ()>(text) {
            let end = idx + num.len();
            let val = Legend::PartNumber(num.parse::<i32>().unwrap_or_default());
            out.push(Token { bgn: idx, end, val });
            idx = end;
            text = rest;
            continue;
        }
        if let Ok((rest, sym)) = anychar::<&str, ()>(text) {
            out.push(Token {
                bgn: idx,
                end: idx + 1,
                val: Legend::Symbol(sym),
            });
            idx += 1;
            text = rest;
            continue;
        }
    }
    Ok((text, out))
}
