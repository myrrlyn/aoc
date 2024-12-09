use std::{
	fmt,
	iter::FusedIterator,
	mem,
};

use heapsize::HeapSizeOf;
use nom::{
	bytes::complete::take,
	combinator::{
		map_parser,
		opt,
	},
	sequence::pair,
};
use tap::Conv;

use crate::{
	parse_number,
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2024, 9, |t| t.parse_dyn_puzzle::<TrashFs>());

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TrashFs {
	// Implementation note: it's actually smaller to just use a straight Vec
	// than it is a "sparse" tree structure. If BTM is Vec<(K, V)>, then it
	// takes 16 bytes to record offset/span, plus 3 for the contents (2 for file
	// id, 1 for discriminant), so EVERY entry is 19 bytes, while Vec<Block>
	// ranges from 0 to 27 bytes per entry, but averaging 15.
	//
	// On my input, the map takes 455,208 bytes, while the vec takes 400,000
	// before shrinkwrapping and 378,380 after.
	// diskmap: BTreeMap<usize, (Block, usize)>,
	diskvec: Vec<Block>,
}

impl Puzzle for TrashFs {
	fn after_parse(&mut self) -> eyre::Result<()> {
		tracing::warn!("heap used: {}", self.heap_size_of_children());
		self.diskvec.shrink_to_fit();
		tracing::warn!("heap used: {}", self.heap_size_of_children());
		Ok(())
	}

	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.compact();
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		Ok(self.checksum() as i64)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		if self
			.diskvec
			.iter()
			.enumerate()
			.filter(|(_, b)| b.is_empty())
			.map(|(p, _)| p)
			.next()
			.unwrap_or_else(|| self.diskvec.len())
			> 20
		{
			eyre::bail!(
				"part 1 cannot be undone. Re-run the puzzle performing part 2 \
				 only"
			);
		}
		self.defrag()
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		Ok(self.checksum() as i64)
	}
}

impl TrashFs {
	pub fn compact(&mut self) {
		let span = self.diskvec.as_mut_slice();
		let mut back = span.len() - 1;
		for front in 0 .. span.len() {
			if !span[front].is_empty() {
				continue;
			}
			while span[back].is_empty() {
				back -= 1;
			}
			if front >= back {
				break;
			}
			span.swap(front, back);
		}
	}

	pub fn defrag(&mut self) -> eyre::Result<()> {
		let mut span = self.diskvec.as_mut_slice();
		let mut back_cursor = span.len();
		// Each file, in reverse order, attempts to move to the first empty span
		// that can take it.
		while let Some(file) = span[.. back_cursor]
			.conv::<FsWalker>()
			.rfind(|s| !s.kind.is_empty())
		{
			tracing::debug!(?file.kind, "defragging");
			back_cursor = file.from;
		}
		tracing::debug!(%self, "after defrag");
		Ok(())
	}

	pub fn checksum(&self) -> usize {
		self.diskvec
			.iter()
			.copied()
			.enumerate()
			.filter_map(|(pos, block)| {
				block.file_id().map(|id| id as usize * pos)
			})
			.sum::<usize>()
	}
}

impl HeapSizeOf for TrashFs {
	fn heap_size_of_children(&self) -> usize {
		0 // self.diskmap.heap_size_of_children()
			+ self.diskvec.capacity() * mem::size_of::<Block>()
	}
}

impl<'a> Parsed<&'a str> for TrashFs {
	fn parse_wyz(mut src: &'a str) -> ParseResult<&'a str, Self> {
		let mut id = 0;
		// let mut cursor = 0;
		// let mut diskmap = BTreeMap::new();
		let mut diskvec = Vec::with_capacity(src.len() * 5);
		while src.trim().len() > 1 {
			let (rest, (file, free)) = pair(
				map_parser(take(1usize), parse_number::<u8>),
				map_parser(take(1usize), parse_number::<u8>),
			)(src)?;
			tracing::trace!(%file, %free, "pair");
			let (file, free) = (file as usize, free as usize);
			// diskmap.insert(cursor, (Block::FilePart { id }, file));
			// cursor += file;
			for _ in 0 .. file {
				diskvec.push(Block::FilePart { id })
			}
			// diskmap.insert(cursor, (Block::Empty, free));
			// cursor += free;
			for _ in 0 .. free {
				diskvec.push(Block::Empty)
			}
			id += 1;
			src = rest;
		}
		let (rest, file) =
			opt(map_parser(take(1usize), parse_number::<u8>))(src)?;
		if let Some(file) = file {
			tracing::trace!(%file, "singlet");
			// diskmap.insert(cursor, (Block::FilePart { id }, file as usize));
			for _ in 0 .. file {
				diskvec.push(Block::FilePart { id });
			}
		}
		tracing::debug!("blocks: {}", diskvec.len());
		Ok((rest, Self {
			// diskmap,
			diskvec,
		}))
	}
}

impl fmt::Display for TrashFs {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let mut cursor = 0;
		let mut list = fmt.debug_list();
		while cursor < self.diskvec.len() {
			let bgn = self.diskvec[cursor];
			let len = self.diskvec[cursor ..]
				.iter()
				.copied()
				.position(|b| b != bgn)
				.unwrap_or_else(|| self.diskvec.len());
			list.entry(&(bgn, len));
			cursor += len;
		}
		list.finish()
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Block {
	#[default]
	Empty,
	FilePart {
		id: u16,
	},
}

impl Block {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Empty => true,
			_ => false,
		}
	}

	pub fn file_id(&self) -> Option<u16> {
		match self {
			Self::Empty => None,
			&Self::FilePart { id } => Some(id),
		}
	}
}

impl HeapSizeOf for Block {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}

struct FsWalker<'a> {
	data:  &'a [Block],
	/// The first block in a run.
	front: usize,
	/// One past the last block in a run.
	back:  usize,
}

impl<'a> From<&'a [Block]> for FsWalker<'a> {
	fn from(data: &'a [Block]) -> Self {
		let back = data.len();
		Self {
			data,
			front: 0,
			back,
		}
	}
}

impl Iterator for FsWalker<'_> {
	type Item = Sector;

	fn next(&mut self) -> Option<Self::Item> {
		if self.front >= self.back {
			return None;
		}
		let kind = self.data[self.front];
		let from = self.front;
		self.front = self
			.data
			.iter()
			.position(|b| *b != kind)
			.unwrap_or(self.back);
		let span = self.front - from;
		Some(Sector { kind, from, span })
	}
}

impl DoubleEndedIterator for FsWalker<'_> {
	fn next_back(&mut self) -> Option<Self::Item> {
		let span = tracing::debug_span!("TrashFs::next_back");
		let _span = span.enter();
		if self.front >= self.back {
			return None;
		}
		let kind = self.data[self.back - 1];
		let end = self.back;
		self.back = self
			.data
			.iter()
			.rposition(|b| *b != kind)
			.unwrap_or(self.front);
		let from = if self.front == self.back {
			self.back
		}
		else {
			self.back + 1
		};
		tracing::debug!("sub?");
		let span = end - self.back;
		Some(Sector { kind, from, span })
	}
}

impl FusedIterator for FsWalker<'_> {
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Sector {
	kind: Block,
	from: usize,
	span: usize,
}
