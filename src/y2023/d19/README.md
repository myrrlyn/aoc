# Filter Application

This problem is another classic AoC staple. The instructions form a decision
tree and objects are routed through various chains of deciders until they
ultimately match or reject. I don't have a library built up for this yet, but as
I see more and more common patterns I am getting the sense of what I should
start sketching.

For instance, you can tell from my first pass at this (*maybe* I'll come back
later and rewrite?) that I *really* like string-caching. Since the instructions
use text names as the ways to transition between nodes in the tree, a naive
approach would require storing a fresh copy of each string in both the container
and any nodes that point to it. A slightly more clever program holds a
`BTreeSet<Arc<str>>` and nodes hold ref-count handles, but this still requires
a `strcmp` operation for graph transition. So my trick is to store *two* cache
trees, one from text to unique integer and one going the other way; those two
caches hold `Arc<str>` and share ownership of the text, and the graph just has
integers.

I'm skipping the short-string optimization, which is silly since the names tend
to be single digits of characters, and the *easiest* cache is actually just a
`u64` which secretly holds an ASCII string in its lsbytes. Maybe I'll do that in
the hypothetical cache library.

Filtering values through the graph is pretty straightforward, and I'm not going
to talk about it here. The part 2 section was a power-set reduction, which went
similarly to day 5 this year and to other problems I remember from last year.

The gist is that we start out with a collection of ranges, and these ranges get
split into sub-ranges by the decider nodes, and each sub-range gets routed along
different paths in the graph. This had the added complexity that the power set
has four dimensions, not one, but it still obeys the same logic. Each reduction
has only one dimension, so the transitions are kept very simple.

My main source of frustration was off-by-one errors in the range splits.
