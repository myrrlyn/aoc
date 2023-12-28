#![doc = include_str!("../doc/web.md")]

use std::{
	collections::{
		BTreeMap,
		BTreeSet,
		VecDeque,
	},
	fmt::{
		self,
		Write,
	},
	iter::FusedIterator,
	ops::{
		Deref,
		DerefMut,
		Index,
		IndexMut,
	},
	sync::Arc,
};

use rayon::prelude::*;

use crate::dict::{
	Dictionary,
	Identifier,
};

/// A set of nodes which can be reached through a port.
pub type Reachable = BTreeSet<Identifier>;

/// A collection of interlinked nodes.
#[derive(Clone)]
pub struct Web {
	/// A collection of text names for nodes.
	names:     Dictionary<str>,
	/// A collection of nodes, each of which may contain outbound links to other
	/// nodes.
	pub nodes: BTreeMap<Identifier, Ports>,
	/// The fallback name text.
	blank:     Arc<str>,
}

impl Web {
	/// Creates an empty web.
	pub fn new() -> Self {
		let mut dict = Dictionary::new();
		let blank: Arc<str> = "<unnamed>".into();
		dict.insert(blank.clone());
		Self {
			names: dict,
			nodes: Default::default(),
			blank,
		}
	}

	/// Places a new node in the web. It must be explicitly linked in order to
	/// be reachable.
	pub fn insert(
		&mut self,
		name: impl AsRef<str> + Into<Arc<str>>,
	) -> Identifier {
		let ident = self.names.insert(name);
		self.nodes.entry(ident).or_default();
		ident
	}

	/// Removes a node from the web.
	///
	/// This deletes all of its own outgoing links, as well as any links coming
	/// in towards it.
	pub fn remove(&mut self, ident: Identifier) {
		self.nodes.remove(&ident);
		for ports in self.nodes.values_mut() {
			ports.remove(&ident);
		}
	}

	/// Creates a bi-directional link between two nodes.
	pub fn create_link(&mut self, one: Identifier, two: Identifier) {
		self.nodes.entry(one).or_default().entry(two).or_default();
		self.nodes.entry(two).or_default().entry(one).or_default();
	}

	/// Removes the bi-directional link between two nodes.
	///
	/// This not only deletes the link; it also removes the now-broken routing
	/// data from all recursively-incoming links.
	pub fn remove_link(&mut self, one: Identifier, two: Identifier) {
		self.remove_link_one_way(one, two);
		self.remove_link_one_way(two, one);
	}

	/// Gets the text name of a particular node.
	pub fn get_name(&self, ident: Identifier) -> Arc<str> {
		self.names
			.lookup(ident)
			.unwrap_or_else(|| self.blank.clone())
	}

	/// Fetches the node object that an `Identifier` names.
	pub fn get_node(&self, ident: Identifier) -> Option<Node<'_>> {
		self.nodes.get(&ident).map(|ports| Node {
			web: self,
			id: ident,
			ports,
		})
	}

	/// Finds a best-path route for all node-pairs in the web, and records them
	/// in the web's internal routing tables.
	///
	/// NOTE: This is quadratic in the number of nodes (`ceil(N^2 / 4)`) and can
	/// take a *very* long time. It is _somewhat_ parallelized, but requires
	/// frequent synchronization points so that the searches can take advantage
	/// of the increasingly-populated web topology caches.
	///
	/// On my machine, routing a 1500-node graph with an average connectivity
	/// factor of ~3-4 takes about 5 minutes.
	pub fn find_all_routes(&mut self) {
		self.clear_routes();
		let parallelism = std::thread::available_parallelism()
			.ok()
			.map(|n| n.get())
			.unwrap_or(2);
		let idents = self.nodes.keys().copied().collect::<Vec<_>>();
		let ct = idents.len();

		for (idx, src) in idents.iter().copied().enumerate() {
			let src_name = self.get_name(src);
			let span = tracing::error_span!("", %idx, %ct, src=%src_name);
			let _span = span.enter();
			tracing::trace!("exploring");
			for group in idents[idx + 1 ..].chunks(parallelism) {
				// This is a compromise between being able to use all of the
				// cores and being able to use the cache. We spawn a handful of
				// spiders, wait for them all to end, then apply all of their
				// outputs to the map before spawning a new batch.
				for mut path in group
					.iter()
					.par_bridge()
					.flat_map(|&dst| self.find_route(src, dst))
					.collect::<Vec<_>>()
				{
					self.mark_path(&path);
					path.reverse();
					self.mark_path(&path);
				}
			}
		}
	}

	/// Finds a best-path route between two nodes.
	///
	/// The search front expands in a ring from the starting node, advancing
	/// outward across all outbound links until either the web is completely
	/// flooded or one of the searchers arrives at the destination node. Since
	/// the search expands in lockstep, the first searcher to arrive knows that
	/// it has the shortest possible length.
	///
	/// I could make the search *concurrent*, but this would require
	/// restructuring the visibility tracking system away from a single-writer
	/// structure, and I don't want to do that right now.
	///
	/// There may be multiple routes which are equally the shortest possible
	/// length. Because `Identifier`s are deterministically ordered, the
	/// algorithm is biased to prefer lower-numbered `Identifiers` more than
	/// balancing "traffic" across different links. If I wanted to simulate load
	/// balancing, I could have each spider record the heaviest-trafficked link
	/// that it crossed, and then only commit the best-path with the lightest
	/// traffic score, but this is an AoC library, not an IP routing library. So
	/// I am not doing that.
	pub fn find_route(
		&self,
		src: Identifier,
		dst: Identifier,
	) -> Option<Vec<Identifier>> {
		let src_name = self.get_name(src);
		let dst_name = self.get_name(dst);
		let span = tracing::error_span!("route", src=%src_name, dst=%dst_name);
		let _span = span.enter();

		let Some(spider) = Spider::new(&*self, src, dst)
		else {
			return None;
		};
		let mut queue = VecDeque::new();
		let mut seen = BTreeSet::new();
		queue.push_back(spider);
		let mut path = None;
		'crawler: while let Some(spider) = queue.pop_front() {
			if let Some(p) = spider.crawl_seq(&mut queue, &mut seen) {
				path = Some(p);
				break 'crawler;
			}
		}
		let Some(path) = path
		else {
			return None;
		};
		Some(path.into_iter().collect())
	}

	/// Removes all cached routing information, without clearing
	pub fn clear_routes(&mut self) {
		// Get the link map from every node,
		for ports in self.nodes.values_mut() {
			// And get the routing table from every link in the map,
			for route in ports.values_mut() {
				// And clear *that*. Clearing the link map would disconnect all
				// the nodes from each other.
				route.clear();
			}
		}
	}

	/// Counts how many nodes are reachable from the given node, *including*
	/// *itself*.
	pub fn count_reachable(&self, ident: Identifier) -> usize {
		let mut queue = VecDeque::new();
		let mut seen = BTreeSet::new();
		let Some(spider) = Spider::new(self, ident, ident)
		else {
			return 0;
		};
		queue.push_back(spider);
		while let Some(spider) = queue.pop_front() {
			spider.find_reachable_seq(&mut queue, &mut seen);
		}
		seen.len()
	}

	/// Loops over all known nodes in the web.
	pub fn nodes<'a>(
		&'a self,
	) -> impl 'a
	+ Iterator<Item = Node<'a>>
	+ DoubleEndedIterator
	+ ExactSizeIterator
	+ FusedIterator {
		self.nodes.iter().map(|(&name, ports)| Node {
			web: self,
			id: name,
			ports,
		})
	}

	/// Loops over all known linked node-pairs in the web.
	pub fn bidi_links<'a>(
		&'a self,
	) -> impl 'a + Iterator<Item = Pair<'a>> + DoubleEndedIterator + FusedIterator
	{
		let mut seen = BTreeSet::new();
		self.nodes()
			.flat_map(move |Node { id, ports, .. }| {
				ports.iter().flat_map(move |(&other, one_two)| {
					let two_one = self.get_node(other)?.ports.get(&id)?;
					Some(Pair {
						web: self,
						one: id,
						two: other,
						one_two,
						two_one,
					})
				})
			})
			.filter(move |&Pair { one, two, .. }| {
				let a = seen.insert((one, two));
				let b = seen.insert((two, one));
				a && b
			})
	}

	fn mark_path(&mut self, mut path: &[Identifier]) {
		// The front of the list is the node being updated.
		while let Some((&current, rest)) = path.split_first() {
			// The next item in the list is the outbound port in that node, and
			// the rest of the list after those are the remote nodes accessible
			// through that port.
			let Some((&next, remote)) = rest.split_first()
			else {
				break;
			};
			self.nodes
				.entry(current)
				.or_default()
				.entry(next)
				.or_default()
				.extend(remote.iter().copied());
			path = rest;
		}
	}

	/// Deletes a one-directional link between two nodes, and informs all nodes
	/// which routed through `src -> dst` that `dst` (and any further nodes on
	/// that path) are no longer reachable through `src`.
	fn remove_link_one_way(&mut self, src: Identifier, dst: Identifier) {
		let src_name = self.get_name(src);
		let dst_name = self.get_name(dst);
		let span = tracing::error_span!("prune", %src_name, %dst_name);
		let _span = span.enter();

		// Contains identifiers of nodes to which incoming links must have their
		// service tables pruned.
		let mut queue = VecDeque::new();
		let mut to_prune = BTreeSet::new();
		let Some(ports) = self.nodes.get_mut(&src)
		else {
			tracing::trace!("no such link");
			return;
		};
		to_prune.insert(dst);
		// If `src -> dst` serviced any other nodes, add them to the prune list.
		if let Some(through_dst) = ports.remove(&dst) {
			to_prune.extend(through_dst);
		}
		queue.push_back(src);
		// while let Some(target) = queue.pop_front() {
		// 	// Find nodes which have outbound links pointing to the current
		// 	// target.
		// 	for (ident, reachable) in
		// 		self.nodes.iter_mut().flat_map(|(&ident, ports)| {
		// 			ports.get_mut(&target).map(|ps| (ident, ps))
		// 		}) {
		// 		// Remove the nodes serviced by src->dst from this link table.
		// 		// If the link table has entries, then all incoming links to
		// 		// *this* node have those same entries. If the table does not,
		// 		// then incoming nodes also do not.
		// 		if to_prune.iter().any(|i| reachable.remove(i)) {
		// 			queue.push_back(ident);
		// 		}
		// 	}
		// }
	}
}

impl Web {
	/// Prints the web as a Graphviz document.
	///
	/// While webs are bidirectional, for clarity with source material, it is
	/// rendered with uni-directional edges pointing from lower-numbered IDs to
	/// higher. The edges are labeled such that `up` counts the number of
	/// best-paths moving from the lower ID to higher, while `dn` counts the
	/// number of best-paths moving from higher to lower.
	pub fn print_graphviz(&self) -> Result<String, fmt::Error> {
		let mut out = String::new();
		writeln!(&mut out, "digraph {{")?;
		for node in self.nodes() {
			writeln!(
				&mut out,
				"    \"{id}\" [label = \"{name}/{id}\"];",
				id = node.id,
				name = node.web.get_name(node.id),
			)?;
		}
		for link in self.bidi_links() {
			writeln!(
				&mut out,
				"    \"{one}\" -> \"{two}\" [label = \"up {up} dn {dn}\"];",
				one = link.one,
				two = link.two,
				up = 1 + link.one_two.len(),
				dn = 1 + link.two_one.len(),
			)?;
		}
		writeln!(&mut out, "}}")?;
		Ok(out)
	}
}

impl Default for Web {
	fn default() -> Self {
		Self::new()
	}
}

impl fmt::Debug for Web {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_map()
			.entries(self.nodes().map(|node| (self.get_name(node.id), node)))
			.finish()
	}
}

impl<S: AsRef<str> + Into<Arc<str>>> FromIterator<(S, S)> for Web {
	fn from_iter<II: IntoIterator<Item = (S, S)>>(iter: II) -> Self {
		let mut this = Self::new();
		for (one, two) in iter {
			let one = this.insert(one);
			let two = this.insert(two);
			this.create_link(one, two);
		}
		this
	}
}

impl Index<Identifier> for Web {
	type Output = Ports;

	fn index(&self, ident: Identifier) -> &Self::Output {
		self.nodes.get(&ident).unwrap_or_else(|| {
			panic!("no such node: {} (#{ident})", self.get_name(ident))
		})
	}
}

impl IndexMut<Identifier> for Web {
	fn index_mut(&mut self, ident: Identifier) -> &mut Self::Output {
		let name = self.get_name(ident);
		self.nodes
			.get_mut(&ident)
			.unwrap_or_else(|| panic!("no such node: {name} (#{ident})"))
	}
}

/// A collection of outbound links and all remote nodes reachable by each link.
#[derive(Clone, Default)]
pub struct Ports {
	inner: BTreeMap<Identifier, Reachable>,
}

impl Ports {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Deref for Ports {
	type Target = BTreeMap<Identifier, Reachable>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Ports {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

/// Represents a single node and its outbound connections.
#[derive(Clone, Copy)]
pub struct Node<'a> {
	id:    Identifier,
	web:   &'a Web,
	ports: &'a Ports,
}
impl<'a> Node<'a> {
	pub fn id(&self) -> Identifier {
		self.id
	}

	pub fn neighbors<'n>(
		&'n self,
	) -> impl 'n
	+ Iterator<Item = Identifier>
	+ DoubleEndedIterator
	+ ExactSizeIterator
	+ FusedIterator {
		self.ports.keys().copied()
	}

	/// If this node knows how to route to the requested target `Identifier`, it
	/// returns which of its neighbors is next in the path to it.
	///
	/// Returning `None` does not necessarily mean that this node has no pathway
	/// to the target, only that no path has yet been discovered.
	pub fn route_to(&self, ident: Identifier) -> Option<Identifier> {
		self.ports
			.iter()
			.find(|&(&i, p)| i == ident || p.contains(&ident))
			.map(|(&i, _)| i)
	}
}

impl fmt::Debug for Node<'_> {
	#[tracing::instrument(skip(self, fmt))]
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		// let this = self.web.get_name(self.name);
		fmt.debug_list()
			.entries(
				self.ports
					.keys()
					.map(|&i| self.web.get_name(i))
					// .inspect(|that| tracing::trace!("found {this} -> {that}"))
			)
			.finish()
	}
}

/// Represents a bi-directional pair of uni-directional connections in the web.
///
/// Contains the names of the two ends of the link, as well as the lists of all
/// node-names which can be reached from one end by going through the other.
pub struct Pair<'a> {
	web:     &'a Web,
	one:     Identifier,
	two:     Identifier,
	one_two: &'a BTreeSet<Identifier>,
	two_one: &'a BTreeSet<Identifier>,
}

impl Pair<'_> {
	pub fn name(&self) -> String {
		format!(
			"{}/{}",
			self.web.get_name(self.one),
			self.web.get_name(self.two),
		)
	}

	pub fn count(&self) -> usize {
		2 + self.one_two.len() + self.two_one.len()
	}

	pub fn ends(&self) -> (Identifier, Identifier) {
		(self.one, self.two)
	}
}

/// A search context for traversing the web.
#[derive(Clone, Debug)]
pub struct Spider<'a> {
	/// The node at which the spider is currently sitting.
	node: Node<'a>,
	/// If the spider is finding a best-path, the node towards which it is
	/// moving.
	goal: Identifier,
	/// If th espider is finding a best-path, the list of all nodes it has
	/// visited so far.
	path: im::Vector<Identifier>,
}

impl<'a> Spider<'a> {
	/// Constructs a new spider at a given location in a web.
	///
	/// Fails if the starting position is not in the web.
	pub fn new(
		web: &'a Web,
		start: Identifier,
		end: Identifier,
	) -> Option<Self> {
		let node = web.get_node(start)?;
		Some(Self {
			node,
			goal: end,
			path: im::Vector::new(),
		})
	}

	/// Performs a single generation of a crawl across the web.
	///
	/// The caller is responsible for providing a a queue which can hold spiders
	/// awaiting processing and a context that all spiders share to understand
	/// what portions of the web have already been visited.
	///
	/// In each generation, the spider performs the following work:
	///
	/// 1. If it is on a node that has already been visited by any spider in the
	///    current search context, it aborts.
	/// 2. If it is at the goal, it returns a list of all nodes it has visited,
	///    including both the start and end nodes.
	/// 3. It checks if its current node already knows a path that goes to its
	///    desired end node. If so, it spawns a child only on the next node in
	///    the known path.
	/// 4. If there is no known path, it spawns a child on *all* neighboring
	///    nodes from its current position.
	/// 5. It dies.
	///
	/// This generational process, combined with a FIFO queue acting as a
	/// scheduler, ensures that the first spider to reach the goal has the best
	/// possible path, at the cost of some additional memory pressure.
	#[tracing::instrument(skip(self, spawner, crawled), fields(node=%self.node.web.get_name(self.node.id), goal=%self.node.web.get_name(self.goal)))]
	pub fn crawl_seq(
		mut self,
		spawner: &mut VecDeque<Self>,
		crawled: &mut BTreeSet<Identifier>,
	) -> Option<im::Vector<Identifier>> {
		// If we are not the first spider to get here, quit.
		if !crawled.insert(self.node.id) {
			return None;
		}
		// Remember that we have reached the current node.
		self.path.push_back(self.node.id);
		// If we are at the destination, succeed.
		if self.node.id == self.goal {
			return Some(self.path);
		}

		// Attempt to discover an existing path.
		if let Some(next) = self.node.route_to(self.goal) {
			// God wouldn't TCO be nice right about now. Oh well.
			if let Some(next) = self.node.web.get_node(next) {
				// Once a path is discovered, kill all the other spiders.
				if !spawner.is_empty() {
					// tracing::warn!("discovered known path; aborting search");
					// spawner.clear();
				}
				self.node = next;
				spawner.push_back(self.clone());
				return None;
			}
		}

		// If we did not fast-forward down a known path, then we need to explore
		// the graph. Spawn for each of the neighbors. This does cause
		// backtracking, but that is caught in the function entry on next call.
		// We could save a little memory pressure on the spawner by filtering
		// *here*, but we still need the escape guards up above, and it's faster
		// to jitter a single buffer than it is to make repeated indirect
		// accesses.
		for neighbor in self.node.neighbors() {
			let Some(next) = self.node.web.get_node(neighbor)
			else {
				continue;
			};
			let mut child = self.clone();
			child.node = next;
			spawner.push_back(child);
		}
		// Die.
		None
	}

	/// Floods the web visiting every node possible.
	pub fn find_reachable_seq(
		self,
		spawner: &mut VecDeque<Self>,
		crawled: &mut BTreeSet<Identifier>,
	) {
		if !crawled.insert(self.node.id) {
			return;
		}
		for next in self
			.node
			.neighbors()
			.flat_map(|i| self.node.web.get_node(i))
		{
			let mut child = self.clone();
			child.node = next;
			spawner.push_back(child);
		}
	}
}
