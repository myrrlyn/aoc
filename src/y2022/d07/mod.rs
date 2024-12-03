use std::{
	collections::BTreeMap,
	fmt,
	path::{
		Component,
		Path,
		PathBuf,
	},
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		i64 as get_i64,
		newline,
		not_line_ending,
	},
	combinator::{
		map,
		value,
	},
	multi::separated_list1,
	sequence::{
		preceded,
		separated_pair,
		terminated,
	},
};

use crate::prelude::*;

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver =
	Solver::new(2022, 7, |t| t.parse_dyn_puzzle::<Navigator>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Navigator {
	fs:     FsNode,
	script: Vec<Command>,
}

impl<'a> Parsed<&'a str> for Navigator {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, _) = terminated(Command::root, newline)(text)?;
		let (rest, cmds) = separated_list1(newline, Command::parse_wyz)(rest)?;
		Ok((rest, Self {
			script: cmds,
			fs:     FsNode::default(),
		}))
	}
}

impl Puzzle for Navigator {
	fn after_parse(&mut self) -> eyre::Result<()> {
		let mut cwd = PathBuf::from("/");
		for cmd in &self.script {
			match cmd {
				Command::List => continue,
				Command::Root => cwd = PathBuf::from("/"),
				Command::GoUp => drop(cwd.pop()),
				Command::GoDown { name } => cwd.push(name),
				Command::Dir { name } => {
					self.fs.dig(&cwd, name.clone(), FsNode::mkdir())?
				},
				Command::File { name, size } => {
					self.fs.dig(&cwd, name.clone(), FsNode::touch(*size))?
				},
			}
		}
		tracing::debug!("structured:\n{self:#}");
		Ok(())
	}
}

impl fmt::Display for Navigator {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.fs, fmt)
	}
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FsNode {
	Directory { contents: BTreeMap<PathBuf, FsNode> },
	File { size: i64 },
}

impl FsNode {
	pub fn mkdir() -> Self {
		Self::default()
	}

	pub fn touch(size: i64) -> Self {
		Self::File { size }
	}

	pub fn dir(&mut self) -> Option<&mut BTreeMap<PathBuf, FsNode>> {
		match self {
			Self::Directory { contents } => Some(contents),
			Self::File { .. } => None,
		}
	}

	pub fn dig(
		&mut self,
		path: &Path,
		name: PathBuf,
		node: Self,
	) -> eyre::Result<()> {
		let mut cursor = self
			.dir()
			.ok_or_else(|| eyre::eyre!("cannot dig through a file"))?;
		for step in path.components() {
			match step {
				Component::RootDir => {
					cursor = self.dir().expect(
						"already checked that we are digging a directory tree",
					)
				},
				Component::Normal(name) => {
					cursor = match cursor
						.entry(name.into())
						.or_insert_with(FsNode::mkdir)
					{
						FsNode::Directory { contents } => contents,
						FsNode::File { .. } => eyre::bail!(
							"cannot make a directory tree through a file"
						),
					}
				},
				Component::Prefix(_)
				| Component::ParentDir
				| Component::CurDir => continue,
			}
		}
		cursor.insert(name, node);
		Ok(())
	}

	pub fn render(
		&self,
		fmt: &mut fmt::Formatter,
		name: &Path,
		level: usize,
	) -> fmt::Result {
		for _ in 0 .. level {
			write!(fmt, "  ",)?;
		}
		write!(fmt, "{}", name.display())?;
		match self {
			Self::File { size } => writeln!(fmt, ": {size}")?,
			Self::Directory { contents } => {
				writeln!(fmt, "/")?;
				for (name, node) in contents {
					node.render(fmt, name, level + 1)?;
				}
			},
		}
		Ok(())
	}
}

impl Default for FsNode {
	fn default() -> Self {
		Self::Directory {
			contents: BTreeMap::new(),
		}
	}
}

impl fmt::Display for FsNode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.render(fmt, Path::new("<root>"), 0)
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Command {
	#[default]
	List,
	Root,
	GoUp,
	GoDown {
		name: PathBuf,
	},
	Dir {
		name: PathBuf,
	},
	File {
		name: PathBuf,
		size: i64,
	},
}

impl Command {
	pub fn root(text: &str) -> ParseResult<&str, Self> {
		value(Self::Root, tag("$ cd /"))(text)
	}

	pub fn cd(to: &str) -> Self {
		Self::GoDown { name: to.into() }
	}

	pub fn mkdir(dir: &str) -> Self {
		Self::Dir { name: dir.into() }
	}

	pub fn touch(name: &str, size: i64) -> Self {
		Self::File {
			name: name.into(),
			size,
		}
	}
}

impl<'a> Parsed<&'a str> for Command {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		alt((
			value(Self::List, tag("$ ls")),
			value(Self::Root, Self::root),
			value(Self::GoUp, tag("$ cd ..")),
			map(preceded(tag("$ cd "), not_line_ending), Self::cd),
			map(preceded(tag("dir "), not_line_ending), Self::mkdir),
			map(
				separated_pair(get_i64, tag(" "), not_line_ending),
				|(size, name)| Self::touch(name, size),
			),
		))(text)
	}
}

impl fmt::Display for Command {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::List => fmt.write_str("$ ls"),
			Self::Root => fmt.write_str("$ cd /"),
			Self::GoUp => fmt.write_str("$ cd .."),
			Self::GoDown { name } => write!(fmt, "$ cd {}", name.display()),
			Self::Dir { name } => write!(fmt, "- [dir] {}", name.display()),
			Self::File { name, size } => {
				write!(fmt, "- [txt] {}: {}", name.display(), size)
			},
		}
	}
}
