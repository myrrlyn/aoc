# Take a Hike

There've been a lot of path-finding lately, and I'm falling into ruts about the
search patterns. Rayon has made breadth-first flooding through a maze fairly
easy, which worked out nicely for d16's beam-spreading through a sparse grid. It
*nearly* worked for d17, but has been choking on the full input.

It also worked on part 1 today, which had constraints keeping the flood from
spreading too widely. Part 2 removes those constraints, and the flood has been
running for about fifteen minutes without getting a first candidate to the end.

The grid library already has a BFS algorithm in it that does not use a
threadpool, so I'll need to try to move my ad-hoc patterns into that to speed it
up. In the meantime, this problem looks like it wants a DFS instead. Time to try
that.

----

I've switched to a DFS search and while that has been able to start producing
candidates almost immediately, it looks to be taking a *long* time to climb
upwards. The fact that search paths can't be *culled* easily, since we're trying
to find the *longest* path rather than the shortest, is especially difficult on
memory pressure. It's already blown past 5GiB resident while I write this.

If I remember correctly, it should at *least* be able to discard useless
branches? I'm not actually sure though.

----

It's still ticking upwards, and memory pressure periodically collapses down to
under a GiB. I'll wait it out.

What really aggravates me is that the BFS and DFS searchers don't have different
*algorithm*s, just different *queue*s. The Rayon threadpool will de-schedule
running workers to try to maintain fairness, so a deep walker will cause itself
to become paused just by spawning side branches for future processing.

----

I switched back to a flooding BFS and went to bed. Turns out it took just under
an hour to solve.

I think I have an algorithmic solution: sprint across the map as fast as
possible, but save forking points into a collection. Then, spawn searchers from
each forking point and have them try to connect back to the existing route. If
the new path from a fork to the main sequence is longer than the original
choice, delete the original, splice in the new, and try again. This should
cause the path history to steadily become larger, so the search space steadily
shrinks even though it can re-tread the same area as the path moves around
(which is normal in path-optimizing searches).
