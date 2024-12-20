# Space coloring

This one took me just under an hour for Part 1, and _over four more_ hours to
get Part 2. Unlike the path-walking puzzles I've done previously (keeping in
mind that as of this writing, I have done some of 2015, 2017, and 2021, and most
of 2022, but none of the other years), this one isn't a maze; it's a _drawing_.
In part 1, I need to detect a continuous curve starting from some origin point
-- not terribly difficult.

In part 2, I need to test whether a given point is inside or outside the
non-convex curve.

Difficult.

I think in SVG this is referred to as the "winding number"? All image editors
have support for this behavior -- it's how the paint bucket works -- but I've
never had to do it myself before and I was pretty much inventing the process
from first principles.

I elected to go with an infilling flood: mark the entire outer perimiter of the
canvas as candidates for the search, and then begin flooding the canvas from
there. The flood ignores all tiles that are not part of the primary curve, and
_touches_, but refuses to _cross_, tiles that are part of it.

At first, I tried to come up with a way to handle the fact that the flood is
permitted to pass between two parallel edges of the curve by detecting curve
edges that ran in the direction of the current flood (that is, a flood cell
attempting to move east and discovering an EW, NE, or SE segment) and
fast-forwarding along them until the curve turned away from the flood travel,
but I wasn't able to express my thoughts here in a way I was confident was
correct.

Since the puzzle guarantees that it is an _even_ curve (one continuous loop,
only one inside and one outside, regardless of how space-filling it is), I am
_fairly_ sure that, mathematically, this is sound. Unfortunately, I couldn't
(and still can't) figure out how to handle the Hilbert case, where a traversal
from West to East on row 2 below could "jump tracks" from column 2 to 3 and
continue forward to detect the void at row 2 columns 7-8.

```text
1:  F7  F--7
2: -JL--J  |
3: --------J
  123456789
```

So I just doubled the canvas area, which expands each tile to be four tiles. The
north-west tile of each quad is the original value, the south-east tile is
always empty, and the north-east and south-west are horizontal and vertical
edges, respectively, if and only if the north-west tile pointed in those
directions. This now meant that parallel edges in the curve suddenly had actual
empty channels between them, that the flood algorithm could traverse without
having to know anything about the shape of the curve. Once the canvas was
completely flooded, still refusing to cross the main curve, I halved the canvas
back to its original size, and that wrapped up the problem.

I am really glad that I had the foresight to make the Cartesian space a reüsable
module last year. I should probably figure out how to lift the flood algorithm
out of this file and into the general library; I strongly suspect it will happen
again!

> Update: it did.
