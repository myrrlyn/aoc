# Web Cutting

This one is HARD. I took 38 hours and I still made it in the top 10k (#9731). I
still have no idea what the analytic solution is, but I think I have an idea
which is chewing along as I write this.

This problem presents a tangle of connected nodes, divided into two large
subgroups. The subgroups are connected to each other by three cross-links. Our
job is to find those links, cut them, and count how many nodes are in each
now-separated group.

1. Parse the input: the input is a series of `node: connection...` lines.
   Connections are bidirectional. This is easy: split the line, add each named
   node to the graph, and establish bidi links between the left node and each of
   the right nodes on the line.
2. Find the cross-links: ?????????? WHAT THE HELL ??????????
3. Cut the cross-links: remove these links from the graph structure.
4. Count the two halves: Take one of the now-cut links. Starting from each side
   of it, flood the graph and count how many nodes can be reached. That is the
   size of the two halves.
5. Done!

That pesky little step two is **aggravating**.

I wound up dumping my parsed structure as GraphViz text and just looking at it
with my eyes. Sure enough, it printed two enormous tangles, and three
plainly-visible strands between them.

The endpoints of those strands were *impossible* to find. So, I adapted my
program to read a list of nodes from another text file, and rip those nodes out
of the graph before printing it, and set about manually plucking out nodes that
I could see didn't have cross-links. After about three hundred, the inner edges
of the two clouds had been pruned enough that the cross-linked nodes were now
visible.

I wrote down the links in *another* file, had the program load *that*, cut those
links, and count the groups. Once past the step 2 hurdle, I was instantly right.

## Analytic Solution

This is a networking problem. BGP solves this in the real world.

I don't know how to write routing tables.

Since each cluster is *fairly* thoroughly inter-linked, I think I know how to
at least *detect* the cross-connections. Within a cluster, the best-path between
any two nodes will be likely very close to uniformly distributed across the
intervening nodes, so if we map *every* best-path between two node-pairs, we
should see that within a cluster, a bell-curve distribution arises for the
number of paths crossing any given link, with the middle links being highest
loaded and the outer links being least.

And the three cross-links should be OBSCENELY saturated. After all, while a
best-path within a cluster has many possible candidate routes, (even in the
middle), *every* best-path in the entire graph that crosses from one cluster to
the other has to go through one of three links, and those links are probably
roughly uniformly balanced on each side of the bridge.

So it's simple. We "just" do an N<sup>2</sup>/2 computation of every best-path
between every pair of nodes in a graph of ~1100, and since I don't know how to
do routing that means each pathfinding needs to flood the graph and report back
the best path sizes until only one candidate remains, so I can't search pairs in
parallel since the path-finder is already parallelized and attempting both would
grind my machine to a halt, then I have each link count how many best-paths go
through it, print them out in descending order, and see if the algorithm
discovered the cross-connections I already know about.

## What If We Derived IP Routing From First Principles

There's a lot of duplicated work in the algorithm I described above. The
path-finders are starting tabula rasa every time. What I *should* be doing is,
rather than just counting how many best-paths go across any given link,
informing each *node* about the destination nodes that are best reached through
each of its links.

In this graph:

```text
a \     / d - f
|  + c +      |
b /     \ e - g
```

we have two cyclic clusters with a single bridge node. Let us walk through the
A-G routing:

1. Spider 1 spawns at A, and moves to B, length 2.
1. Spider 2 spawns at A, and moves to C, length 2.
1. Spider 1 moves to C, length 3. We have an opportunity here for Spider 1 to
   observe that Spider 2 is already at C, with length 2 (better than its own),
   and so Spider 1 can abort.
1. Spider 2 moves to D, length 3.
1. Spider 3 spawns at C and moves to E, length 3.
1. Spider 1, if it is still alive, moves to D, length 4.
1. Spider 4 spawns at C and moves to E, length 4. You can see why we would want
   spiders to eagerly abort.
1. Spider 2 moves to F, length 4.
1. Spider 3 moves to G, length 4. It sends its route to its parent, which then
   marks each *link* in the path that it is the best route for all *nodes* in
   the path after itself. That is, when a spider is at node A, link AB is the
   best candidate for reaching B, and AC is the best candidate for reaching C,
   E, and G. At C, link CE is best for E and G, but CD is best for D and F.

Because the node names are not distributed in any useful pattern, I think every
link needs to store information about the *entire* graph; there does not seem to
be a way to compress the space of node names or divide the net into any other
clustering patterns.

----

I GOT IT!!

I was right: plotting every single route through the web caused the three
cross-links to have the highest number of paths in the whole web, with a
separation of about 200 from their runners-up. Cutting those links and counting
the size of the reachable sub-graph from each endpoint of one of the links
(doesn't matter which) gave the correct answer.

It's not the most efficient solution, but it solves on my machine in under 6
minutes, so I'LL TAKE IT.
