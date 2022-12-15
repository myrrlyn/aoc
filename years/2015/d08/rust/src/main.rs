static INPUT: &str = wyz_aoc::input!();

fn main() {
    let pt1 = INPUT
        .lines()
        .map(count_unescaped)
        .map(|(lit, sto)| lit - sto)
        .sum::<usize>();
    println!("part 1: {pt1}");

    let pt2 = INPUT
        .lines()
        .map(count_reescaped)
        .map(|(enc, lit)| enc - lit)
        .sum::<usize>();
    println!("part 2: {pt2}");
}

fn count_unescaped(line: &str) -> (usize, usize) {
    let text_len = line.len();
    let inner = &line[1..text_len - 1];
    let mut count = 0;
    let mut walker = inner.bytes();
    while let Some(cursor) = walker.next() {
        match cursor {
            b'\\' => {
                match walker.next().expect("incomplete escape") {
                    b'\\' => {}
                    b'x' => {
                        walker.next().expect("incomplete hex escape");
                        walker.next().expect("incomplete hex escape");
                    }
                    b'"' => {}
                    _ => panic!("unknown escape"),
                }
                count += 1;
            }
            _ => count += 1,
        }
    }
    (text_len, count)
}

fn count_reescaped(line: &str) -> (usize, usize) {
    let orig = line.len();
    let mut count = 2;
    for cursor in line.bytes() {
        count += match cursor {
            b'\\' => 2,
            b'"' => 2,
            _ => 1,
        };
    }
    (count, orig)
}
