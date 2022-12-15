use std::{
    collections::BTreeMap,
    error::Error,
    path::{Component, Path, PathBuf},
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{not_line_ending, u64 as get_u64},
    combinator::{map, value},
    sequence::{preceded, separated_pair},
    IResult,
};

static INPUT: &str = include_str!("../input.txt");

fn main() -> Result<(), Box<dyn Error>> {
    let (_, fs) =
        parse_filesystem(INPUT).map_err(|e| format!("failed to read shell history: {e}"))?;
    let sz = fs_size(&fs);
    println!("{sz:#?}");
    let sz = part1(&fs);
    println!("{sz:#?}");
    let sz = part2(&fs);
    println!("{sz:#?}");
    Ok(())
}

type Fs = BTreeMap<PathBuf, DirEnt>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum DirEnt {
    Dir { children: Fs },
    File { size: u64 },
}

impl DirEnt {
    fn new_dir() -> Self {
        Self::Dir {
            children: Fs::new(),
        }
    }

    fn new_file(size: u64) -> Self {
        Self::File { size }
    }

    fn size(&self) -> u64 {
        match self {
            Self::Dir { children } => fs_size(children),
            Self::File { size } => *size,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Line {
    #[default]
    List,
    Root,
    Up,
    Down {
        name: PathBuf,
    },
    Dir {
        name: PathBuf,
    },
    File {
        name: PathBuf,
        size: u64,
    },
}

impl Line {
    fn parse(s: &str) -> IResult<&str, Self> {
        alt((
            value(Self::List, tag("$ ls")),
            value(Self::Root, tag("$ cd /")),
            value(Self::Up, tag("$ cd ..")),
            map(preceded(tag("$ cd "), not_line_ending), |name: &str| {
                Self::Down { name: name.into() }
            }),
            map(preceded(tag("dir "), not_line_ending), |name: &str| {
                Self::Dir { name: name.into() }
            }),
            map(
                separated_pair(get_u64, tag(" "), not_line_ending),
                |(size, name): (_, &str)| Self::File {
                    name: name.into(),
                    size,
                },
            ),
        ))(s)
    }
}

fn dig(fs: &mut Fs, path: &Path, name: PathBuf, dirent: DirEnt) {
    let mut cursor = &mut *fs;
    for step in path.components() {
        match step {
            Component::RootDir => cursor = &mut *fs,
            Component::Normal(name) => {
                cursor = match cursor.entry(name.into()).or_insert_with(DirEnt::new_dir) {
                    DirEnt::Dir { children } => children,
                    DirEnt::File { .. } => panic!("cannot make a directory tree through a file"),
                }
            }
            Component::Prefix(_) => continue,
            Component::ParentDir => continue,
            Component::CurDir => continue,
        }
    }
    cursor.insert(name, dirent);
}

fn parse_filesystem(text: &str) -> IResult<&str, Fs> {
    let mut fs = Fs::new();
    let mut cwd = PathBuf::new();
    let mut rest = text;
    for line in text.lines() {
        let (_, parsed) = Line::parse(line)?;
        rest = rest[line.len()..].trim_start();
        match parsed {
            Line::List => continue,
            Line::Root => cwd = PathBuf::from("/"),
            Line::Up => drop(cwd.pop()),
            Line::Down { name } => cwd.push(name),
            Line::Dir { name } => dig(&mut fs, &cwd, name, DirEnt::new_dir()),
            Line::File { name, size } => dig(&mut fs, &cwd, name, DirEnt::new_file(size)),
        }
    }

    Ok((rest, fs))
}

fn fs_size(fs: &Fs) -> u64 {
    fs.values().map(DirEnt::size).sum()
}

fn fs_descend(fs: &Fs, func: &mut impl FnMut(&Path, &DirEnt)) {
    for (path, dirent) in fs {
        match dirent {
            de @ DirEnt::File { .. } => func(path, de),
            de @ DirEnt::Dir { children } => {
                func(path, de);
                fs_descend(children, func);
            }
        }
    }
}

fn part1(fs: &Fs) -> u64 {
    let mut sum = 0;
    fs_descend(fs, &mut |_, dirent| {
        if let de @ DirEnt::Dir { .. } = dirent {
            let desz = de.size();
            if desz <= 100000 {
                sum += desz;
            }
        }
    });
    sum
}

fn part2(fs: &Fs) -> u64 {
    static TOTAL: u64 = 70000000;
    static MIN_FREE: u64 = 30000000;

    let current_free = TOTAL.checked_sub(fs_size(fs)).expect("fs larger than disk");
    let floor = MIN_FREE
        .checked_sub(current_free)
        .expect("more free space than needed");
    let mut smallest_sufficient = !0;
    fs_descend(fs, &mut |_, dirent| {
        if let de @ DirEnt::Dir { .. } = dirent {
            let desz = de.size();
            if desz >= floor {
                smallest_sufficient = smallest_sufficient.min(desz);
            }
        }
    });
    smallest_sufficient
}
