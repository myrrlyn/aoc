use std::{collections::BTreeSet, fmt};

static INPUT: &str = wyz_aoc::input!();

fn main() {
    let slice = INPUT.lines().next().expect("must have a password");
    let password = Password {
        letters: slice.as_bytes().try_into().expect("must be 8 chars"),
    };
    let pt1 = password.next();
    println!("part 1: {pt1}");
    let pt2 = pt1.next();
    println!("part 2: {pt2}");
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Password {
    letters: [u8; 8],
}

impl Password {
    fn is_valid(&self) -> bool {
        let is_ascii = self.letters.iter().all(|b| b.is_ascii());
        let has_run = self
            .letters
            .windows(3)
            .any(|cs| cs[1].wrapping_sub(cs[0]) == 1 && cs[2].wrapping_sub(cs[1]) == 1);
        let no_forbidden = self.letters.iter().all(|b| !b"ilo".contains(b));
        let has_pairs = self
            .letters
            .windows(2)
            .filter(|cs| cs[0] == cs[1])
            .collect::<BTreeSet<_>>()
            .len()
            >= 2;
        is_ascii && has_run && no_forbidden && has_pairs
    }

    fn next(mut self) -> Self {
        fn add_one(b: u8) -> (u8, bool) {
            match b {
                b'a'..=b'y' => (b + 1, false),
                b'z' => (b'a', true),
                _ => panic!("not ascii"),
            }
        }

        let mut is_valid = false;
        while !is_valid {
            for letter in self.letters.iter_mut().rev() {
                let (next, cout) = add_one(*letter);
                *letter = next;
                if !cout {
                    break;
                }
            }
            is_valid = self.is_valid();
        }
        self
    }
}

impl fmt::Display for Password {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(text) = std::str::from_utf8(&self.letters) {
            fmt.write_str(text)
        } else {
            fmt.write_str("<password is non-ascii>")
        }
    }
}
