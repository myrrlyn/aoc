# Area Inside a Curve

We’ve already had at least one other puzzle that involved finding the area
enclosed by a curve, so for this one, I felt okay trying to re-use that
solution. And for part one, which had a bounding box of a mere 201x112, sure,
drawing the curve, flooding the interior, and counting how many total tiles had
been marked worked fine.

And then. Came. Part. Two.

The part two bounding box was 14,312,378x9,623,928. 137,741,295,380,784 (that’s
137 quadrillion (American) or billiard (European) pixels, which is, shall we
say, Not Resident In Memory. Or in swap. Packing it with `bitvec` would take
16 TiB).

This forced me to implement an actual working algorithmic solution.

The problem here has to do with the winding number of a curve. The curve in the
puzzle input is not convex, and has many different phalanges and offshoots which
make analysis of the curve as a sequence of linear segments, complicated.
However, it essentially boils down to a few axioms.

One: the curve is a closed cycle. The input data causes the last stroke to end
at the origin, so we do not need to attempt to close the gap ourselves.

Two: the curve is a single, continunous, non-self-intersecting, non-backtracking
sequence of strokes. This means that every pixel in the stroke is guaranteed to
have exactly two neighboring pixels that are in the stroke, and two that are
not. The shortest possible stroke segment is 2 pixels long, and every change in
direction from horizontal to vertical has exactly one choice.

## Algorithmic Computation

Terminology: because corner points are in two strokes (one vertical, one
horizontal), all stroke lengths are *exclusive*, and are the length described in
the instruction rather than the number of pixels touched by the stroke. For
instance, the first instruction moving N pixels from the origin has a distance of
N, but colors N + 1 pixels.

We begin by translating the draw instructions into a sequence of
absolutely-positioned strokes in a grid. We can then compute the bounding box
which encloses all strokes, and scan down it in rows.

## Stroke Finding

We have a small collection of strokes. During our scan, we need to filter that
collection by whether a given stroke is present in a particular pixel. This is
just a matter of knowing whether the sampling pixel is in between the start and
end pixels of the stroke.

We also need to know the direction of the stroke: whether the pen is moving up,
left, right, or down. This directionality allows us to compute the winding
number of the curve and detect whether a pair of strokes describes a space which
is inside or outside the whole curve.

### Row Marching

In each row, we find the length of all the horizontal strokes and add them to
the accumulator.

We also know that there is *at least one* pair of vertical strokes in every row
in the bounding box. There can be no empty rows, because the curve is continuous
and the box minimized. So we need to march across each *pair* of adjacent
vertical strokes, and start by adding one to the accumulator to represent the
left-most stroke, which will not otherwise be included.

#### Column Marching

In each row, we march from one side to the other. The first time we encounter a
vertical stroke, we transition from being outside the curve to inside.

We then walk across each pair of adjacent vertical strokes which touch the row.
There are three possible cases:

1. The vertical strokes are going in the same direction. This means that one of
   them is ending in the row, the other is beginning, and they are connected by
   a horizontal stroke, whose length has already been added to the accumulator.

   This pair has no effect on the algorithm, and is skipped.

2. The strokes are going in opposite directions, and are connected by a
   horizontal stroke. The horizontal stroke is not accumulated a second time.

3. The strokes are going in opposite directions, and are not connected by a
   horizontal stroke in this row. If we are currently inside the curve, then the
   distance between the strokes (excluding the left stroke pixel, including the
   right stroke pixel) needs to be accumulated.

   If we are outside the curve, then the right stroke pixel needs to be
   accumulated, but the gap between the strokes does not.

In both of the latter cases, we invert the inside-or-outside marker before
continuing.

The area enclosed by the curve, including the curve’s own pixels, is the sum of
all such row accumulations. The operating complexity is a factor of stroke
length and count.

On my machine, 660 strokes describing a curve whose bounding box is 14 million
by 10 million pixels, could be computed in 16 seconds.
