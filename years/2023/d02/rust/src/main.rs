use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, value},
    multi::separated_list1,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

static INPUT: &str = wyz_aoc::input!();

fn main() -> anyhow::Result<()> {
    let mut pt1_out = 0;
    let mut pt2_out = 0;
    for line in INPUT.lines() {
        let res = tuple((
            delimited(tag("Game "), get_num, tag(": ")),
            separated_list1(tag("; "), get_group),
        ))(&line);
        let (_, (num, data)) = match res {
            Ok(val) => val,
            Err(err) => anyhow::bail!("could not parse line: {err}"),
        };
        let max = data.into_iter().fold(Group::default(), |old, new| Group {
            red: old.red.max(new.red),
            grn: old.grn.max(new.grn),
            blu: old.blu.max(new.blu),
        });
        // println!(
        //     "{{num: {num}, red: {red}, grn: {grn}, blu: {blu}, unparsed: {rest:?}}}",
        //     red = max.red,
        //     grn = max.grn,
        //     blu = max.blu,
        // );
        if max.red <= PART1_KEY.red && max.grn <= PART1_KEY.grn && max.blu <= PART1_KEY.blu {
            pt1_out += num;
        }
        pt2_out += max.red * max.grn * max.blu;
    }
    println!(r#"{{ pt1: {pt1_out}, pt2: {pt2_out} }}"#);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Color {
    Red,
    Green,
    Blue,
}

fn get_color(text: &str) -> IResult<&str, Color> {
    alt((
        value(Color::Red, tag("red")),
        value(Color::Blue, tag("blue")),
        value(Color::Green, tag("green")),
    ))(text)
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Group {
    red: i32,
    grn: i32,
    blu: i32,
}

impl FromIterator<(i32, Color)> for Group {
    fn from_iter<I: IntoIterator<Item = (i32, Color)>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Group::default(), |grp, (ct, col)| match col {
                Color::Red => Group { red: ct, ..grp },
                Color::Blue => Group { blu: ct, ..grp },
                Color::Green => Group { grn: ct, ..grp },
            })
    }
}

fn get_group(text: &str) -> IResult<&str, Group> {
    let (rest, data) =
        separated_list1(tag(", "), separated_pair(get_num, tag(" "), get_color))(text)?;
    let grp = data.into_iter().collect::<Group>();
    Ok((rest, grp))
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Record {
    num: i32,
    max: Group,
}

const PART1_KEY: Group = Group {
    red: 12,
    grn: 13,
    blu: 14,
};

fn get_num<'a>(text: &'a str) -> IResult<&'a str, i32> {
    map_res(digit1, str::parse::<i32>)(text)
}
