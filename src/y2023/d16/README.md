# Light Beam Propagation

This puzzle requires us to assemble a hall of mirrors/prisms and propagate a
light beam through it. Mirrors only turn the existing beam in a new direction,
while prisms split the beam into two new beams, each traveling in a *different*
direction from the old beam and each other.

As a cursor propagates through the grid, it has to mark its travel into the grid
history so that it can detect cycles. Each tile needs to be able to remember
each *direction* in which a beam travels through it, and any given tile can have
up to four beams (one in each direction) which do not interfere with each other.
A cycle only occurs when a beam travels through a tile in a direction that has
already been traveled.

Cursors stop their travel either when they encounter a cycle (continuing forward
would subject the cursor to the exact same rules that have already been
followed, so no more new information can be produced) or attempt to move out of
the constructed area.

This puzzle is a prime candidate for parallelization: beams propagating through
a grid need to be able to see each other’s passage, but are unlikely to contend
for the same tile at the same time. I suspect that having many workers spread
through and contaminated the same memory region causes a lot of cache
invalidation and makes beam-walking *essentially* sequential when beams are in
the same row, but *probably* re-parallelize as they spread apart vertically. But
I don’t know how to instrument this without the instrumentation being heavier
than the actual work, and I don’t care enough to try to figure it out.

In part two, we need to search the same grid, from the same unlit starting
condition, with many different starting seeds. *This* means allocating fresh,
blank, copies of the grid for each new search, and *those* searches *do* run
entirely independently of each other.

On my machine, part 1 takes 1ms, and part 2 (which has 439 beams in parallel)
takes ... 9ms.
