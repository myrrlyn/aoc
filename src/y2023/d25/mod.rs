use std::cmp;

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

use crate::{
	prelude::*,
	web::Web,
};

#[linkme::distributed_slice(SOLVERS)]
static ITEM: Solver = Solver::new(2023, 25, |t| t.parse_dyn_puzzle::<Wiring>());

#[derive(Debug, Default)]
pub struct Wiring {
	web: Web,
}

impl<'a> Parsed<&'a str> for Wiring {
	fn parse_wyz(text: &'a str) -> ParseResult<&'a str, Self> {
		let mut web = Web::new();
		let (rest, links) = separated_list1(
			newline,
			separated_pair(alpha1, tag(": "), separated_list1(space1, alpha1)),
		)(text)?;
		for (src, dsts) in links {
			let src = web.insert(src);
			for dst in dsts {
				let dst = web.insert(dst);
				web.create_link(src, dst);
			}
		}
		Ok((rest, Self { web }))
	}
}

impl Puzzle for Wiring {
	fn after_parse(&mut self) -> eyre::Result<()> {
		tracing::debug!(ct=%self.web.nodes().count(), "finished parsing");
		self.web.find_all_routes();
		std::fs::write(
			"assets/outputs/2023/d25/webbing.gv",
			self.web.print_graphviz()?,
		)?;
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		let mut ends = None;
		for _ in 0 .. 3 {
			self.web.find_all_routes();
			let mut links = self.web.bidi_links().collect::<Vec<_>>();
			links.sort_by_key(|l| cmp::Reverse(l.count()));
			// Because the routing algorithm biases towards lower-numbered
			// nodes, it is possible that links become artificially loaded
			// without actually being a critical path. This selects the
			// highest-numbered link of the heaviest loadings, under the
			// assumption that it is unlikely to be a false positive. It works
			// correctly on the sample data.
			let Some(heaviest) = links
				.iter()
				.filter(|l| l.count() == links[0].count())
				.last()
			else {
				eyre::bail!("could not find a link to cut")
			};
			let (one, two) = heaviest.ends();
			ends = Some((one, two));
			self.web.remove_link(one, two);
			self.web.clear_routes();
		}

		let Some((left, right)) = ends
		else {
			eyre::bail!("did not find any links to cut");
		};
		let (mut ct_left, mut ct_right) = (0, 0);
		rayon::scope(|s| {
			s.spawn(|_| ct_left = self.web.count_reachable(left));
			s.spawn(|_| ct_right = self.web.count_reachable(right));
		});
		tracing::info!("DONE");
		self.web.find_all_routes();
		std::fs::write(
			"assets/outputs/2023/d25/webbing-cut.gv",
			self.web.print_graphviz()?,
		)?;

		Ok((ct_left * ct_right) as i64)
	}
}

impl Wiring {
}
