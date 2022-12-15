use std::{cmp, fmt, str::FromStr};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::u8 as get_u8, combinator::map,
    multi::separated_list0, sequence::delimited, IResult,
};
use tap::{Tap, TapFallible};
use wyz::FmtForward;

fn main() {
    let pairs = wyz_aoc::input!()
        .split("\n\n")
        .zip(1..)
        .map(|(lines, idx)| (lines.parse::<Pair>(), idx))
        .flat_map(|(res, idx)| res.tap_err(|err| eprintln!("{err}")).ok().map(|p| (p, idx)))
        .collect::<Vec<_>>();

    let pt1 = pairs
        .iter()
        .filter(|(p, _)| p.is_sorted())
        .map(|(_, i)| i)
        .sum::<i32>();
    println!("part 1: {pt1}");

    let markers = [Item::Scalar(2).into_vector(), Item::Scalar(6).into_vector()];
    let pt2 = wyz_aoc::input!()
        .lines()
        .filter(|s| !s.is_empty())
        .flat_map(|line| line.parse::<Item>().tap_err(|err| eprintln!("{err}")).ok())
        .chain(markers.clone())
        .collect::<Vec<Item>>()
        .tap_mut(|v| v.sort())
        .into_iter()
        .zip(1..)
        .filter(|(item, _)| markers.contains(item))
        .map(|(_, idx)| idx)
        .product::<usize>();
    println!("part 2: {pt2}");
}

struct Pair {
    left: Item,
    right: Item,
}

impl Pair {
    fn is_sorted(&self) -> bool {
        self.left <= self.right
    }
}

impl FromStr for Pair {
    type Err = String;

    fn from_str(lines: &str) -> Result<Self, Self::Err> {
        let mut iter = lines.lines().take(2).map(Item::parse);
        let (rest, left) = iter
            .next()
            .transpose()
            .map_err(|err| format!("parse error: {err}"))?
            .ok_or_else(|| "not enough lines".to_owned())?;
        if !rest.trim().is_empty() {
            return Err(format!("incomplete parse: {rest}"));
        }
        let (rest, right) = iter
            .next()
            .transpose()
            .map_err(|err| format!("parse error: {err}"))?
            .ok_or_else(|| "not enough lines".to_owned())?;
        if !rest.trim().is_empty() {
            return Err(format!("incomplete parse: {rest}"));
        }

        Ok(Self { left, right })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Item {
    Scalar(u8),
    Vector(Vec<Self>),
}

impl Item {
    fn parse(text: &str) -> IResult<&str, Self> {
        alt((
            delimited(
                tag("["),
                map(separated_list0(tag(","), Self::parse), Self::Vector),
                tag("]"),
            ),
            map(get_u8, Self::Scalar),
        ))(text)
    }

    fn into_vector(self) -> Self {
        match self {
            v @ Self::Vector(_) => v,
            s @ Self::Scalar(_) => Self::Vector(vec![s]),
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("Item has a total ordering")
    }
}

impl PartialOrd<Self> for Item {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Scalar(a), Self::Scalar(b)) => a.partial_cmp(b),
            (a @ Self::Scalar(_), b @ Self::Vector(_)) => a.clone().into_vector().partial_cmp(b),
            (a @ Self::Vector(_), b @ Self::Scalar(_)) => a.partial_cmp(&b.clone().into_vector()),

            (Self::Vector(a), Self::Vector(b)) => {
                let mut one = a.iter();
                let mut two = b.iter();

                loop {
                    match (one.next(), two.next()) {
                        (None, None) => return Some(cmp::Ordering::Equal),
                        (None, Some(_)) => return Some(cmp::Ordering::Less),
                        (Some(_), None) => return Some(cmp::Ordering::Greater),
                        (Some(a), Some(b)) => match a.partial_cmp(b) {
                            Some(cmp::Ordering::Equal) => continue,
                            other => return other,
                        },
                    }
                }
            }
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Scalar(s) => fmt::Display::fmt(s, fmt),
            Self::Vector(v) => fmt
                .debug_list()
                .entries(v.iter().map(|i| i.fmt_display()))
                .finish(),
        }
    }
}

impl FromStr for Item {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s) {
            Ok(("", this)) => Ok(this),
            Ok((rest, this)) => {
                eprintln!("[warn ]: unparsed input: {rest:?}");
                Ok(this)
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
