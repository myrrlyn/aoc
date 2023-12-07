use crate::{written_number, ParseResult, Parseable as _, Parsed, Puzzle, Solver};
use tap::Pipe;

#[linkme::distributed_slice(crate::SOLVERS)]
static ITEM: Solver = Solver::new(2023, 1, |t| t.parse_dyn_puzzle::<Calibration>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Calibration {
    digits_only: Vec<i64>,
    digits_and_words: Vec<i64>,
}

impl<'a> Parsed<&'a str> for Calibration {
    fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
        let mut digits_only = vec![];
        let mut digits_and_words = vec![];
        for line in text.lines() {
            let mut digits = line
                .chars()
                .filter(char::is_ascii_digit)
                .map(|d| d as i64 - '0' as i64);
            let ten = match digits.next() {
                Some(val) => val,
                None => {
                    tracing::error!(?line, "did not discover any numbers");
                    0
                }
            };
            let one = digits.last().unwrap_or(ten);
            digits_only.push(ten * 10 + one);

            let (mut ten, mut one) = (None, None);
            for (idx, sym) in line.char_indices() {
                let rest = &line[idx..];
                let num = if let Ok((_, num)) = written_number::<i64>(rest) {
                    num
                } else if sym.is_ascii_digit() {
                    sym as i64 - '0' as i64
                } else {
                    continue;
                };
                if ten.is_none() {
                    ten = Some(num);
                } else {
                    one = Some(num);
                }
            }
            match (ten, one) {
                (None, _) => {
                    tracing::error!(?line, "did not discover any numbers, including spelled");
                    continue;
                }
                (Some(ten), one) => digits_and_words.push(ten * 10 + one.unwrap_or(ten)),
            }
        }
        Ok((
            "",
            Self {
                digits_only,
                digits_and_words,
            },
        ))
    }
}

impl Puzzle for Calibration {
    fn part_1(&mut self) -> anyhow::Result<i64> {
        self.digits_only.iter().copied().sum::<i64>().pipe(Ok)
    }

    fn part_2(&mut self) -> anyhow::Result<i64> {
        self.digits_and_words.iter().copied().sum::<i64>().pipe(Ok)
    }
}
