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
		Ok(())
	}

	fn part_1(&mut self) -> eyre::Result<i64> {
		self.web.find_all_routes();

		let mut links = self.web.bidi_links().collect::<Vec<_>>();
		links.sort_by_key(|l| cmp::Reverse(l.count()));
		let (left, right) = links[0].ends();
		for (one, two) in
			links[.. 3].iter().map(|l| l.ends()).collect::<Vec<_>>()
		{
			self.web.remove_link(one, two);
		}

		self.web.clear_routes();
		let (mut ct_left, mut ct_right) = (0, 0);
		rayon::scope(|s| {
			s.spawn(|_| ct_left = self.web.count_reachable(left));
			s.spawn(|_| ct_right = self.web.count_reachable(right));
		});
		tracing::info!("DONE");
		self.web.find_all_routes();
		std::fs::write(
			"assets/outputs/2023/d25/webbing.gv",
			self.web.print_graphviz()?,
		)?;

		Ok((ct_left * ct_right) as i64)
	}
}

impl Wiring {
}
