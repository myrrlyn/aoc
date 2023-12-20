#![doc = include_str!("d20.md")]

use std::{
	collections::{
		BTreeMap,
		VecDeque,
	},
	fmt,
	ops::Not,
	sync::Arc,
};

use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{
		alpha1,
		newline,
	},
	combinator::map,
	multi::separated_list1,
	sequence::{
		preceded,
		separated_pair,
	},
};

use crate::{
	dictionary::{
		Dictionary,
		Identifier,
	},
	prelude::*,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 20, |t| t.parse_dyn_puzzle::<Machine>());

#[derive(Clone, Debug)]
pub struct Machine {
	nodes: Netlist,
	/// The ID of the entry-point node.
	entry: Identifier,
	/// The queue of pulses propagating through the machine.
	queue: Bus,
}

impl<'a> Parsed<&'a str> for Machine {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let (rest, nodes) = Netlist::parse_wyz(text)?;
		let entry =
			nodes.names.lookup_value("broadcaster").ok_or_else(|| {
				tracing::error!("a netlist must have a `broadcaster` node");
				nom::Err::Failure(nom::error::Error::new(
					text,
					nom::error::ErrorKind::Tag,
				))
			})?;
		Ok((rest, Self {
			nodes,
			entry,
			queue: Bus::default(),
		}))
	}
}

impl Puzzle for Machine {
	fn prepare_1(&mut self) -> eyre::Result<()> {
		self.nodes.insert_test_point("rx");
		for n in 0 .. 1000 {
			tracing::trace!(%n, "loop");
			self.pulse(None)?;
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		tracing::info!(low=%self.queue.ct_lo, high=%self.queue.ct_hi, "pulses");
		Ok(self.queue.ct_lo * self.queue.ct_hi)
	}

	fn prepare_2(&mut self) -> eyre::Result<()> {
		self.reset();
		Ok(())
	}

	fn part_2(&mut self) -> eyre::Result<i64> {
		let rx = self.nodes.names.insert("rx");
		// The problem statement says only one node is an input to `rx`.
		let (&end_name, end) = self
			.nodes
			.nodes
			.iter()
			.find(|(_, n)| n.outputs.contains(&rx))
			.ok_or_else(|| {
				eyre::eyre!("never found a node which outputs to rx")
			})?;
		// The problem statement does not assert that the ending node is a
		// Conjunction. However, it is in my input, as are all of *its* inputs,
		// so I didn't have to do the math for computing FlipFlop periodicity.
		// If your end conditions have a FlipFlop, send me your input and I'll
		// deal with it.
		let mut penultimates = match &end.kind {
			Kind::Conjunction { inputs } => inputs
				.keys()
				.copied()
				.map(|i| (i, None))
				.collect::<BTreeMap<_, _>>(),
			_ => eyre::bail!("ending node is not a Conjunction"),
		};
		for n in 1 ..= i64::MAX {
			let span = tracing::error_span!("sim", %n);
			let _span = span.enter();
			// If the pulse yields a matching message source,
			if let Some(ident) = self.pulse(Some(end_name))? {
				// Stash the current simulator cycle in that source's entry.
				penultimates.insert(ident, Some(n));
				// If all of the expected messages have arrived,
				if penultimates.values().all(Option::is_some) {
					tracing::debug!("found all periods");
					// Compute the LCM of their periods, and quit.
					return penultimates
						.values()
						.flatten()
						.copied()
						.reduce(num::integer::lcm)
						.ok_or_else(|| {
							unreachable!(
								"this only happens when at least one period is \
								 present"
							)
						});
				}
			}
		}
		eyre::bail!("never found an exit signal before wrapping i64");
	}
}

impl Machine {
	pub fn pulse(
		&mut self,
		test_point: Option<Identifier>,
	) -> eyre::Result<Option<Identifier>> {
		self.queue.send(self.entry, self.entry, Pulse::Low);
		let mut out = None;
		while let Some(Packet {
			send: from,
			recv: this,
			value,
		}) = self.queue.recv()
		{
			let send_name = self.nodes.node_name(from);
			let recv_name = self.nodes.node_name(this);
			let span = tracing::error_span!("processing", from=%send_name, to=%recv_name, %value);
			let _span = span.enter();

			if value == Pulse::High && Some(this) == test_point {
				tracing::debug!("bkpt");
				if out.is_some() {
					tracing::error!(
						"found multiple LOW test-points during the same pulse"
					);
				}
				out = Some(from);
			}

			let to_send = if let Ok(node) = self.nodes.get_mut(this) {
				node.apply_signal(from, value)
			}
			else {
				continue;
			};
			if let Some(to_send) = to_send {
				let Ok(node) = self.nodes.get(this)
				else {
					continue;
				};
				for &dest in &node.outputs {
					tracing::error_span!("sending", next=%self.nodes.node_name(dest)).in_scope(||
					self.queue.send(this, dest, to_send));
				}
			}
		}
		Ok(out)
	}

	pub fn reset(&mut self) {
		self.nodes.reset();
		self.queue.reset();
	}
}

#[derive(Clone, Debug)]
pub struct Netlist {
	/// The collection of all textual names for the machine's nodes.
	names: Dictionary<str>,
	/// The collection of all nodes in the machine's net-list.
	nodes: BTreeMap<Identifier, Node>,
	/// A default name.
	blank: Arc<str>,
}

impl Netlist {
	pub fn node_name(&self, ident: Identifier) -> Arc<str> {
		let name = self.names.lookup(ident);
		name.unwrap_or_else(|| self.blank.clone())
	}

	pub fn insert_test_point(
		&mut self,
		name: impl AsRef<str> + Into<Arc<str>>,
	) -> Identifier {
		let name = self.names.insert(name);
		self.nodes.entry(name).or_insert(Node {
			name,
			kind: Kind::Broadcaster,
			outputs: vec![],
		});
		name
	}

	pub fn get(&self, ident: Identifier) -> eyre::Result<&Node> {
		self.nodes.get(&ident).ok_or_else(|| {
			let name = self.node_name(ident);
			tracing::warn!(%name, "no such node");
			eyre::eyre!("no such node: {name}")
		})
	}

	pub fn get_mut(&mut self, ident: Identifier) -> eyre::Result<&mut Node> {
		let name = self.node_name(ident);
		self.nodes.get_mut(&ident).ok_or_else(|| {
			tracing::warn!(%name, "no such node");
			eyre::eyre!("no such node: {name}")
		})
	}

	pub fn reset(&mut self) {
		for node in self.nodes.values_mut() {
			match &mut node.kind {
				Kind::Broadcaster => {},
				Kind::FlipFlop { memory } => *memory = Pulse::Low,
				Kind::Conjunction { inputs } => {
					for value in inputs.values_mut() {
						*value = Pulse::Low;
					}
				},
			}
		}
	}
}

impl<'a> Parsed<&'a str> for Netlist {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut names = Dictionary::new();
		let (rest, nodes) = separated_list1(newline, |t| {
			Node::parse_with_context(t, &mut names)
		})(text)?;
		let nodes: BTreeMap<Identifier, Node> =
			nodes.into_iter().map(|n| (n.name, n)).collect();
		let mut this = Self {
			names,
			nodes,
			blank: Arc::from("<unnamed>"),
		};

		// Walk all the nodes, and inspect each output connection on each node.
		// If the destination is a Conjunction, then the destination needs to be
		// told about the node that sends to it. Since we can't write into the
		// collection while traversing it, we need to build up a collection of
		// all discovered conjunctions and their input nodes, and we can then
		// splice that collection into the net-list once the scan is complete.
		tracing::trace!("connecting Conjunction input ports");
		let mut backlinks: BTreeMap<Identifier, BTreeMap<Identifier, Pulse>> =
			BTreeMap::new();
		for (&ident, node) in this.nodes.iter() {
			let from_name = this.node_name(ident);
			for &dest in &node.outputs {
				let to_name = this.node_name(dest);
				let Some(dest_node) = this.nodes.get(&dest)
				else {
					tracing::error!(from=%from_name, dest=%to_name, "could not create link");
					continue;
				};
				if dest_node.is_conjunction() {
					backlinks.entry(dest).or_default().insert(ident, Pulse::Low);
				}
			}
		}
		tracing::trace!("applying Conjunction inputs");
		for (ident, filled_inputs) in backlinks.into_iter() {
			let name = this.node_name(ident);
			let Some(node) = this.nodes.get_mut(&ident)
			else {
				tracing::error!(%name, "cannot find expected Conjunction");
				continue;
			};
			if let Kind::Conjunction { inputs } = &mut node.kind {
				tracing::trace!(%name, ct=%filled_inputs.len(), "applying Conjunction inputs");
				*inputs = filled_inputs;
			}
		}
		Ok((rest, this))
	}
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bus {
	queue: VecDeque<Packet>,
	ct_lo: i64,
	ct_hi: i64,
}

impl Bus {
	pub fn send(&mut self, from: Identifier, to: Identifier, value: Pulse) {
		*match value {
			Pulse::Low => &mut self.ct_lo,
			Pulse::High => &mut self.ct_hi,
		} += 1;
		self.queue.push_back(Packet::new(from, to, value));
	}

	pub fn recv(&mut self) -> Option<Packet> {
		self.queue.pop_front()
	}

	pub fn reset(&mut self) {
		*self = Default::default();
	}
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Node {
	/// The node's own name.
	name:    Identifier,
	/// Mark what kind of node this is.
	kind:    Kind,
	/// The list of nodes to which this sends pulses. Pulses are emitted in
	/// storage order.
	outputs: Vec<Identifier>,
}

impl Node {
	pub fn is_conjunction(&self) -> bool {
		if let Kind::Conjunction { .. } = &self.kind {
			true
		}
		else {
			false
		}
	}

	pub fn apply_signal(
		&mut self,
		sender: Identifier,
		pulse: Pulse,
	) -> Option<Pulse> {
		match &mut self.kind {
			Kind::Broadcaster => {
				tracing::trace!("broadcasting");
				Some(pulse)
			},
			Kind::FlipFlop { memory } => match pulse {
				Pulse::High => {
					tracing::trace!("flip-flop ignore");
					None
				},
				Pulse::Low => {
					*memory = !*memory;
					tracing::trace!("flip-flop toggle");
					Some(*memory)
				},
			},
			Kind::Conjunction { inputs } => {
				*inputs.entry(sender).or_default() = pulse;
				Some(match inputs.values().all(|&p| p == Pulse::High) {
					true => {
						tracing::trace!("latch all-high");
						Pulse::Low
					},
					false => {
						tracing::trace!("latch some-low");
						Pulse::High
					},
				})
			},
		}
	}

	pub fn parse_with_context<'a>(
		text: &'a str,
		ctx: &mut Dictionary<str>,
	) -> ParseResult<&'a str, Self> {
		let (rest, ((kind, name), outputs)) = separated_pair(
			alt((
				map(tag("broadcaster"), |t| (Kind::Broadcaster, t)),
				map(preceded(tag("%"), alpha1), |t| (Kind::new_flipflop(), t)),
				map(preceded(tag("&"), alpha1), |t| {
					(Kind::new_conjunction(), t)
				}),
			)),
			tag(" -> "),
			separated_list1(tag(", "), map(alpha1, |t| ctx.insert(t))),
		)(text)?;
		let name = ctx.insert(name);
		Ok((rest, Self {
			kind,
			name,
			outputs,
		}))
	}
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
	/// Entry point to the net-list. Must only exist once.
	Broadcaster,
	/// A memory cell. Ignores HIGH pulses; on LOW pulses, it inverts its memory
	/// and then emits the updated value.
	FlipFlop {
		/// The remembered state.
		memory: Pulse,
	},
	/// A NAND gate. Upon receipt of a pulse from one of its inputs, it
	/// remembers that pulse, then emits LOW if all input memories are HIGH, or
	/// HIGH if any of its inputs are LOW.
	Conjunction { inputs: BTreeMap<Identifier, Pulse> },
}

impl Kind {
	pub const fn new_flipflop() -> Self {
		Self::FlipFlop { memory: Pulse::Low }
	}

	pub const fn new_conjunction() -> Self {
		Self::Conjunction {
			inputs: BTreeMap::new(),
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Pulse {
	#[default]
	Low,
	High,
}

impl fmt::Display for Pulse {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str(match self {
			Self::Low => "LOW",
			Self::High => "HIGH",
		})
	}
}

impl Not for Pulse {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::Low => Self::High,
			Self::High => Self::Low,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Packet {
	send:  Identifier,
	recv:  Identifier,
	value: Pulse,
}

impl Packet {
	pub fn new(from: Identifier, to: Identifier, value: Pulse) -> Self {
		Self {
			send: from,
			recv: to,
			value,
		}
	}
}
