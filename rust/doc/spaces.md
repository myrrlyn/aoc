# Dimensional Spaces

This module contains data structures which describe sections of a space, as
frequently presented in Advent of Code problems. As with all parts of this
project, it is tailored for the specific patterns that come up in the AoC
puzzles, and makes no attempt to generalize towards reality or other use cases.

The AoC puzzles *that I have seen* use spatial concepts have all been 2- or 3-
dimensional Cartesian spaces with finite, quantized, divisions. Sparsity is also
a frequent-enough phenomenon that I have elected to build the module entirely
around the ability to represent sparsely-populated spaces, and handle dense
population as an unfortunately slow instance of the general case. I hope to
build a faster, `Vec`-based dense counterpart in the future for problems which
completely fill a region, but for now, `BTreeMap` suffices.

## Coördinate System

Since AoC is driven by text files, 2-D puzzles tend to exist *mostly* in
Quadrant IV of the classical Cartesian grid: that is, origin is in the top left,
X increases rightward, and Y increases downwards. The structures support
signed-integer ordinates, and so you *can* place data anywhere you desire.
However, in keeping with the computing tradition, renders to the screen will
(currently) always translate the minimum point to the origin, and display the
region in Quadrant IV.
