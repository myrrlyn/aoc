use std::{
	collections::{
		BTreeSet,
		VecDeque,
	},
	iter::FusedIterator,
	sync::atomic::Ordering,
};

use eyre::Context;
use nom::{
	bytes::complete::tag,
	character::complete::{
		alpha1,
		newline,
		space1,
	},
	multi::separated_list1,
	sequence::separated_pair,
};
use radium::{
	Atom,
	Radium,
};

use crate::{
	prelude::*,
	web::Web,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 25, |t| t.parse_dyn_puzzle::<Wiring>());

#[derive(Debug, Default)]
pub struct Wiring {
	web: Web<Atom<i32>>,
}

impl<'a> Parsed<&'a str> for Wiring {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut web = Web::new();
		let (rest, links) = separated_list1(
			newline,
			separated_pair(alpha1, tag(": "), separated_list1(space1, alpha1)),
		)(text)?;
		for (src, dsts) in links {
			let src = web.insert(src, Atom::new(0));
			for dst in dsts {
				let dst = web.insert(dst, Atom::new(0));
				web.link_bidi(src, dst);
			}
		}
		Ok((rest, Self { web }))
	}
}

impl Puzzle for Wiring {
	fn after_parse(&mut self) -> eyre::Result<()> {
		tracing::debug!(ct=%self.web.node_count(), "finished parsing");
		// I used this to manually cull nodes until I found the answer.
		if false {
			let known_good =
				std::fs::read_to_string("assets/outputs/2023/d25/found.txt")
					.wrap_err("could not find save file")?
					.split_whitespace()
					.map(|n| {
						self.web
							.lookup_name(n)
							.ok_or_else(|| eyre::eyre!("no such name: {n}"))
					})
					.collect::<eyre::Result<Vec<_>>>()?;
			let remove =
				std::fs::read_to_string("assets/outputs/2023/d25/trim.txt")
					.wrap_err("could not find trim file")?;
			let nodes = self.web.node_count();
			for name in remove.lines() {
				let Some(ident) = self.web.lookup_name(name)
				else {
					continue;
				};
				if known_good.iter().any(|&i| {
					if self.web.has_link(ident, i) {
						tracing::info!(%name, "preserving");
						true
					}
					else {
						false
					}
				}) {
					continue;
				}
				self.web.remove(ident);
				assert!(!self.web.has_node(ident));
			}
			let removed = nodes - self.web.node_count();
			tracing::trace!(%removed, "took out nodes");
			std::fs::create_dir_all("assets/outputs/2023")?;
			std::fs::write(
				"assets/outputs/2023/d25/input.gv",
				self.web.print_dot_lang_undirected()?,
			)?;
		}
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		let pairs =
			std::fs::read_to_string("assets/outputs/2023/d25/found.txt")?
				.lines()
				.map(|l| l.split_once(" ").expect("badly formatted line"))
				.map(|(a, b)| {
					(
						self.web.lookup_name(a).unwrap(),
						self.web.lookup_name(b).unwrap(),
					)
				})
				.collect::<Vec<_>>();
		for &(one, two) in &pairs {
			self.web.cut_link_bidi(one, two);
		}
		let (mut left, mut right) = (0, 0);
		rayon::scope(|s| {
			s.spawn(|_| left = self.web.count_reachable(pairs[0].0));
			s.spawn(|_| right = self.web.count_reachable(pairs[0].1));
		});
		tracing::info!(%left, %right, "subgroup sizes");
		Ok((left * right) as i64)
	}
}

impl Wiring {
}
