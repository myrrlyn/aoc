# Circuit Netlists

This is a combination of DAG routing and digital logic that usually crops up at
least once a year. I am pretty sure I remember 2022 having one, but I do not at
all remember details, and I certainly don't have a library for it.

## Part 1

This is a fairly straightforward signal-propagation problem. We run the netlist
for a thousand cycles, count all the signals, and are done.

## Part 2

This ... is not.

The goal is to count how many cycles it takes to get a desired exit junction to
emit a HIGH signal. I am sitting at over 40 million cycles simulated as I write
this and have decided that I need to figure out a more clever solution.

Junction nodes are NAND gates. When *all* of their inputs are HIGH, they are
LOW; otherwise, they are HIGH. Assuming that the signal graph is acyclic (which
I am going to choose to assume for today, but I worry might not actually be
true), then the period of a junction gate is the LCM of the period of its
inputs.

Since our output case is a junction fed by junctions, we can ignore the period
of a flip-flop and just seek the periods of the penultimate junctions, then find
their least-common multiple.

My answer was in the quadrillions (American; billiards, European), so I'm glad I
didn't try to spin the simulator until it latched. Somehow, even with a nearly
17-hour time, I still took #9655, (#12284 for part 1), so the rest of the crowd
clearly does not handle nets very well, or detect when a puzzle is an LCM
problem.
