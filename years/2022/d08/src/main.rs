static INPUT: &str = include_str!("../input.txt");

fn main() {
    let fst = Forest::parse(INPUT);
    println!("Found {} trees", fst.len());
    let visible = fst.count_visible();
    println!("Visible trees: {visible}");
    let scenic = fst.view_score();
    println!("Best scenery: {scenic}");
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Forest {
    /// The input file as written: a list of horizontal rows, top to bottom
    ew: Vec<Vec<u8>>,
    /// The transposed input file: a list of vertical columns, left to right
    ns: Vec<Vec<u8>>,
}

impl Forest {
    fn parse(text: &str) -> Self {
        let mut ew = Vec::new();
        let mut ns = Vec::new();
        for (row, line) in text.lines().enumerate() {
            let mut rank = Vec::with_capacity(line.len());
            for (col, ht) in line.bytes().map(|b| b - b'0').enumerate() {
                rank.push(ht);
                if row == 0 {
                    ns.push(vec![ht]);
                } else {
                    ns[col].push(ht);
                }
            }
            ew.push(rank);
        }
        Self { ew, ns }
    }

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
                if col == 0 || row == 0 || col == rank.len() - 1 || row == file.len() - 1 {
                    ct += 1;
                    continue;
                }

                let top = &file[..row];
                let left = &rank[..col];
                let right = &rank[col + 1..];
                let bottom = &file[row + 1..];

                if hidden_behind(top, ht)
                    && hidden_behind(left, ht)
                    && hidden_behind(right, ht)
                    && hidden_behind(bottom, ht)
                {
                } else {
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
                if col == 0 || row == 0 || col == rank.len() - 1 || row == file.len() - 1 {
                    continue;
                }

                let top = &file[..row];
                let left = &rank[..col];
                let right = &rank[col + 1..];
                let bottom = &file[row + 1..];

                let top_s = top
                    .iter()
                    .rev()
                    .zip(1..)
                    .find(|(&h, _)| h >= ht)
                    .map(|(_, d)| d)
                    .unwrap_or(top.len());
                let left_s = left
                    .iter()
                    .rev()
                    .zip(1..)
                    .find(|(&h, _)| h >= ht)
                    .map(|(_, d)| d)
                    .unwrap_or(left.len());
                let right_s = right
                    .iter()
                    .zip(1..)
                    .find(|(&h, _)| h >= ht)
                    .map(|(_, d)| d)
                    .unwrap_or(right.len());
                let bottom_s = bottom
                    .iter()
                    .zip(1..)
                    .find(|(&h, _)| h >= ht)
                    .map(|(_, d)| d)
                    .unwrap_or(bottom.len());

                best = (top_s * left_s * right_s * bottom_s).max(best);
            }
        }
        best
    }
}
