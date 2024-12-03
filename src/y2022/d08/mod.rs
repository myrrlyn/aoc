use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2022, 8, |t| t.parse_dyn_puzzle::<Forest>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Forest {
	/// The input file as written: a list of horizontal rows, top to bottom
	ew: Vec<Vec<u8>>,
	/// The transposed input file: a list of vertical columns, left to right
	ns: Vec<Vec<u8>>,
}

impl<'a> Parsed<&'a str> for Forest {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut ew = Vec::new();
		let mut ns = Vec::new();
		for (row, line) in text.lines().enumerate() {
			let mut rank = Vec::with_capacity(line.len());
			for (col, ht) in line.bytes().map(|b| b - b'0').enumerate() {
				rank.push(ht);
				if row == 0 {
					ns.push(vec![ht]);
				}
				else {
					ns[col].push(ht);
				}
			}
			ew.push(rank);
		}
		Ok(("", Self { ew, ns }))
	}
}

impl Puzzle for Forest {
	fn after_parse(&mut self) -> eyre::Result<()> {
		tracing::info!(ct=%self.len(), "found trees");
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(self.count_visible() as i64)
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		Ok(self.view_score() as i64)
	}
}

impl Forest {
	fn len(&self) -> usize {
		let ew = self.ew.iter().map(|v| v.len()).sum();
		let ns = self.ns.iter().map(|v| v.len()).sum();
		assert_eq!(ew, ns, "E/W {ew} count does not match N/S count {ns}");
		ew
	}

	fn count_visible(&self) -> usize {
		fn hidden_behind(trees: &[u8], ht: u8) -> bool {
			trees.iter().any(|&h| h >= ht)
		}
		let mut ct = 0;
		for (row, rank) in self.ew.iter().enumerate() {
			for (col, &ht) in rank.iter().enumerate() {
				let file = &self.ns[col];
				if col == 0
					|| row == 0 || col == rank.len() - 1
					|| row == file.len() - 1
				{
					ct += 1;
					continue;
				}

				let top = &file[.. row];
				let left = &rank[.. col];
				let right = &rank[col + 1 ..];
				let bottom = &file[row + 1 ..];

				if hidden_behind(top, ht)
					&& hidden_behind(left, ht)
					&& hidden_behind(right, ht)
					&& hidden_behind(bottom, ht)
				{
				}
				else {
					ct += 1;
				}
			}
		}
		ct
	}

	fn view_score(&self) -> usize {
		let mut best = 0;
		for (row, rank) in self.ew.iter().enumerate() {
			for (col, &ht) in rank.iter().enumerate() {
				let file = &self.ns[col];
				if col == 0
					|| row == 0 || col == rank.len() - 1
					|| row == file.len() - 1
				{
					continue;
				}

				let top = &file[.. row];
				let left = &rank[.. col];
				let right = &rank[col + 1 ..];
				let bottom = &file[row + 1 ..];

				let top_s = top
					.iter()
					.rev()
					.zip(1 ..)
					.find(|(&h, _)| h >= ht)
					.map(|(_, d)| d)
					.unwrap_or(top.len());
				let left_s = left
					.iter()
					.rev()
					.zip(1 ..)
					.find(|(&h, _)| h >= ht)
					.map(|(_, d)| d)
					.unwrap_or(left.len());
				let right_s = right
					.iter()
					.zip(1 ..)
					.find(|(&h, _)| h >= ht)
					.map(|(_, d)| d)
					.unwrap_or(right.len());
				let bottom_s = bottom
					.iter()
					.zip(1 ..)
					.find(|(&h, _)| h >= ht)
					.map(|(_, d)| d)
					.unwrap_or(bottom.len());

				best = (top_s * left_s * right_s * bottom_s).max(best);
			}
		}
		best
	}
}
