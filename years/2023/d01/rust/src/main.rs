use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::digit1,
    combinator::{map_parser, map_res, value},
    IResult,
};

static INPUT: &str = wyz_aoc::input!();

fn main() {
    println!("part 1: {}", problem_one(INPUT).sum::<i32>());
    println!("part 2: {}", problem_two(INPUT).sum::<i32>());
}

fn problem_one<'a>(text: &'a str) -> impl 'a + Iterator<Item = i32> {
    text.lines()
        .map(|line| {
            line.chars()
                .filter(|c| c.is_ascii_digit())
                .map(|d| d as i32 - '0' as i32)
        })
        .flat_map(|mut digits| -> Option<i32> {
            let first = digits.next()?;
            let last = digits.last().unwrap_or(first);
            Some(first * 10 + last)
        })
}

fn munch_number<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, i32> {
    alt((
        value(1, tag("one")),
        value(2, tag("two")),
        value(3, tag("three")),
        value(4, tag("four")),
        value(5, tag("five")),
        value(6, tag("six")),
        value(7, tag("seven")),
        value(8, tag("eight")),
        value(9, tag("nine")),
        value(0, tag("zero")),
        map_res(map_parser(take(1u8), digit1), str::parse),
    ))
}

fn problem_two<'a>(text: &'a str) -> impl 'a + Iterator<Item = i32> {
    text.lines().flat_map(|line| {
        let (mut one, mut two) = (None, None);
        for (idx, _) in line.char_indices() {
            let span = &line[idx..];
            if let Ok((_, num)) = munch_number()(span) {
                if one.is_none() {
                    one = Some(num);
                } else {
                    two = Some(num);
                }
            }
        }
        print!("{one:?} {two:?}");
        let out = one.map(|n| n * 10 + two.unwrap_or(n));
        println!("{out:?}");
        out
    })
}
