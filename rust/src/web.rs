use std::{
	collections::{
		BTreeMap,
		BTreeSet,
		VecDeque,
	},
	iter::FusedIterator,
	mem,
	sync::{
		atomic::Ordering,
		mpsc,
		Arc,
	},
};

use radium::{
	Atom,
	Radium,
};
use rayon::Scope;

use crate::dict::{
	Dictionary,
	Identifier,
};

#[derive(Clone, Debug)]
pub struct Web<T> {
	names: Dictionary<str>,
	nodes: BTreeMap<Identifier, T>,
	edges: BTreeSet<Link>,
}

impl<T> Web<T> {
	pub fn new() -> Self {
		Self::default()
	}

	/// Emplaces a new item in the web.
	pub fn insert(
		&mut self,
		name: impl AsRef<str> + Into<Arc<str>>,
		value: T,
	) -> Identifier {
		let ident = self.names.insert(name);
		self.nodes.insert(ident, value);
		ident
	}

	pub fn node_count(&self) -> usize {
		self.nodes.len()
	}

	pub fn lookup(&self, ident: Identifier) -> Option<Arc<str>> {
		self.names.lookup(ident)
	}

	pub fn lookup_name(&self, name: impl AsRef<str>) -> Option<Identifier> {
		self.names.lookup_value(name)
	}

	pub fn has_node(&self, ident: Identifier) -> bool {
		self.nodes.contains_key(&ident)
	}

	pub fn has_link(&self, from: Identifier, to: Identifier) -> bool {
		self.edges.contains(&Link::new(from, to))
	}

	/// Creates a one-direction link between two nodes.
	///
	/// Has no effect if either of the Identifiers is missing from the Web.
	pub fn link(&mut self, from: Identifier, to: Identifier) {
		if self.names.contains_key(from) && self.names.contains_key(to) {
			self.edges.insert(Link { src: from, dst: to });
		}
	}

	/// Creates a bi-directional link between two nodes.
	pub fn link_bidi(&mut self, one: Identifier, two: Identifier) {
		self.link(one, two);
		self.link(two, one);
	}

	pub fn cut_link(&mut self, src: Identifier, dst: Identifier) {
		self.edges.remove(&Link::new(src, dst));
	}

	pub fn cut_link_bidi(&mut self, one: Identifier, two: Identifier) {
		self.cut_link(one, two);
		self.cut_link(two, one);
	}

	pub fn remove(&mut self, ident: Identifier) {
		self.edges.retain(|l| !l.contains(ident));
		self.nodes.remove(&ident);
	}

	/// Finds the shortest path between two nodes.
	///
	/// Returns `None` if no such path exists; otherwise, returns a list of all
	/// the identifiers in the path, including both the start and end nodes.
	pub fn shortest_path_between(
		&self,
		start: Identifier,
		end: Identifier,
	) -> Option<Vec<Identifier>>
	where
		T: Sync,
	{
		let (send, recv) = mpsc::channel();
		let cancel = Atom::new(false);
		let cancel_ref = &cancel;
		let start = Spider::new(self, start, end).ok()?;
		let mut out: Option<im::Vector<Identifier>> = None;
		let out_ref = &mut out;
		rayon::scope(move |s| {
			s.spawn(move |s| start.crawl_par(s, send, cancel_ref));
			while let Ok(path) = recv.recv() {
				*out_ref = Some(match out_ref.take() {
					Some(other) => {
						if other.len() < path.len() {
							other
						}
						else {
							path
						}
					},
					None => path,
				});
			}
		});
		out.map(|v| v.into_iter().collect())
	}

	/// Lists all identifiers of nodes which can be reached from a given
	/// identifier.
	pub fn neighbors_of<'a>(
		&'a self,
		name: Identifier,
	) -> impl 'a + Iterator<Item = Identifier> + DoubleEndedIterator + FusedIterator
	{
		self.edges
			.iter()
			.filter(move |Link { src, .. }| *src == name)
			.map(|Link { dst, .. }| *dst)
	}

	pub fn count_reachable(&self, from: Identifier) -> usize {
		let mut seen = BTreeSet::new();
		let mut queue = VecDeque::new();
		queue.push_back(from);
		while let Some(this) = queue.pop_front() {
			if !seen.insert(this) {
				continue;
			}
			for next in self.neighbors_of(this) {
				queue.push_back(next);
			}
		}
		seen.len()
	}

	pub fn print_dot_lang(&self) -> Result<String, std::fmt::Error> {
		use std::fmt::Write;
		let mut out = "digraph {\n".to_owned();
		for name in self.nodes.keys().copied().flat_map(|i| self.lookup(i)) {
			writeln!(&mut out, "  \"{name}\";")?;
		}
		for Link { src, dst } in self.edges.iter().copied() {
			let Some((src, dst)) = self
				.lookup(src)
				.and_then(|s| self.lookup(dst).map(|d| (s, d)))
			else {
				continue;
			};
			writeln!(&mut out, "  \"{src}\" -> \"{dst}\";")?;
		}
		out.push_str("}\n");
		Ok(out)
	}

	pub fn print_dot_lang_undirected(&self) -> Result<String, std::fmt::Error> {
		use std::fmt::Write;
		let mut out = "graph {\n".to_owned();
		for name in self.nodes.keys().copied().flat_map(|i| self.lookup(i)) {
			writeln!(&mut out, "  \"{name}\";")?;
		}
		let mut seen = BTreeSet::new();
		for l @ Link { src, dst } in self.edges.iter().copied() {
			if seen.contains(&Link::new(dst, src)) {
				continue;
			}
			seen.insert(l);
			let Some((src, dst)) = self
				.lookup(src)
				.and_then(|s| self.lookup(dst).map(|d| (s, d)))
			else {
				continue;
			};
			writeln!(&mut out, "  \"{src}\" -- \"{dst}\";")?;
		}
		out.push_str("}\n");
		Ok(out)
	}
}

impl<T> Default for Web<T> {
	fn default() -> Self {
		Self {
			names: Dictionary::new(),
			nodes: BTreeMap::new(),
			edges: BTreeSet::new(),
		}
	}
}

/// A one-directional link.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Link {
	src: Identifier,
	dst: Identifier,
}

impl Link {
	pub fn new(src: Identifier, dst: Identifier) -> Self {
		Self { src, dst }
	}

	pub fn contains(&self, ident: Identifier) -> bool {
		self.src == ident || self.dst == ident
	}
}

pub struct Spider<'w, T: Sync> {
	web:  &'w Web<T>,
	node: Identifier,
	goal: Identifier,
	path: im::Vector<Identifier>,
}

impl<'w, T: Sync> Spider<'w, T> {
	pub fn new(
		web: &'w Web<T>,
		node: Identifier,
		goal: Identifier,
	) -> eyre::Result<Self> {
		if !web.names.contains_key(node) || !web.names.contains_key(goal) {
			eyre::bail!("cannot create a spider with invalid node names");
		}
		Ok(Self {
			web,
			node,
			goal,
			path: im::vector![],
		})
	}

	/// Attempts to move from `node` to `goal`.
	pub fn crawl_par<'s, 'a>(
		mut self,
		scope: &Scope<'s>,
		output: mpsc::Sender<im::Vector<Identifier>>,
		cancel: &'a Atom<bool>,
	) where
		'a: 's,
		'w: 's,
	{
		'crawl: loop {
			if cancel.load(Ordering::Relaxed) {
				tracing::trace!("spider received shutdown signal");
				break 'crawl;
			}
			self.path.push_back(self.node);
			if self.node == self.goal {
				if let Err(_) = output.send(mem::take(&mut self.path)) {
					tracing::warn!("a spider could not report its result");
				}
				break 'crawl;
			}
			let mut steps = self
				.web
				.neighbors_of(self.node)
				.filter(|n| !self.path.contains(n));
			let next = steps.next();
			for step in steps {
				let mut twin = self.clone();
				twin.node = step;
				let chan = output.clone();
				scope.spawn(move |s| twin.crawl_par(s, chan, cancel));
			}
			let Some(step) = next
			else {
				break 'crawl;
			};
			self.node = step;
		}
	}
}

impl<T: Sync> Clone for Spider<'_, T> {
	fn clone(&self) -> Self {
		Self {
			path: self.path.clone(),
			..*self
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_web() -> Web<()> {
		let y2023d25 = include_str!("../../assets/samples/2023/d25.txt");
		let mut web = Web::new();
		for line in y2023d25.trim().lines() {
			let (left, right) =
				line.split_once(": ").expect("input has a colon");
			let left = web.insert(left, ());
			for name in right.split(" ") {
				let right = web.insert(name, ());
				web.link_bidi(left, right);
			}
		}
		web
	}

	#[test]
	fn crawl_web() {
		let web = make_web();
		let path = ["jqt", "nvd", "lhk", "lsr"].map(|n| {
			web.lookup_name(n)
				.expect("web has all of the provided names")
		});
		let found_path = web
			.shortest_path_between(path[0], path[3])
			.expect("a path exists between jqt and lsr");
		assert_eq!(
			&found_path,
			&path,
			"the shortest discovered path is: {}",
			found_path
				.iter()
				.copied()
				.flat_map(|i| web.lookup(i))
				.collect::<Vec<_>>()
				.join("->")
		);
	}
}
