# Linear Arithmetic

This one is less of a CS problem and more of a middle school algebra problem:
given a collection of vectored points, compute possible intersections.

The part 1 arithmetic is fairly simple, though it did require me to
remember/derive linear equations from scratch. It has been a while since I've
had to use them.

## Linear Motion in Free Space

For a particle with an initial position $p_0$ and a steady motion vector $V$,
its position can be expressed as a function of time: $p(t) = p_0 + (V * t)$.

This works no matter how many dimensions the coördinate system has.

The problem space has three dimensions, though part 1 only requires two of them.
We can project down to the X/Y plane simply by not including the Z dimension in
our calculations.

## Intersection in Space Only

We have two particles with their individual `p0` and `V` components. We need to
find where their paths cross, even if the particles do not intersect in the time
dimension.

We also have the constraint that the particles in this simulation only move
forward in time.

The function of position over time is $p(t) = p_0 + \vec{V} \times t$. This can
be decomposed into motion along each axis by projecting the $\vec{V}$ vector
onto each axis.

Part 1 needs us to compute $X/Y$ intersections without caring if the paths
intersect in $t$ as well.

We can do this by solving the following system of equations:

$$
\forall {\vec{V}} \rarr m(\vec{V}) = \frac{\vec{V}[y]}{\vec{V}[x]}
\\
\forall {P, \vec{V}} \rarr P_0[y] = P[y] - m(\vec{V}) \times P[x]
\\
\forall {a, b} \rarr y_i(x) = (P_a)_0[y] + m(\vec{V_a}) \times x = (P_b)_0[y] +
m(\vec{V_b}) \times x
\\
x_i = \frac{(P_a)_0[y] - (P_b)_0[y]}{m(\vec{V_b}) - m(\vec{V_a})}
\\
y_i = m(\vec{V_a}) \times x_i + (P_a)_0[y] = m(\vec{V_b}) \times x_i +
(P_b)_0[y]
\\
t_a, t_b = \frac{x_i - P_a[x]}{\vec{V}_a[x]}, \frac{x_i - P_b[x]}
{\vec{V}_b[x]}
$$

We run this system on all unique unordered pairs in the collection, discard
intersections with a negative time answer, and have part 1.

## Intersection in Space and Forecasted Time

Part 2 is a more daunting explanation: we need to find a vector such that a
particle travelling along it shall intersect every other particle in the system,
which is also moving in the same time-frame.

The answer is, fortunately, symbolically simple (though analytically expensive):
we already have all of the particles' initial positions and vectors, so we can:

1. walk through every unique ordered pair of particles in the collection,
1. and for each of them, project one forward to $t=1$ and the other to $t=2$,
1. then compute the ray that connects these projections, and project it
   infinitely forward.
1. Discard this pairing if the new ray fails to intersect with *every* other
   ray in space and at $t > 2$
1. Discard this pairing if the set of all intersections in space has duplicate
   moments in time.
1. For each pair which survives these filters, project the new ray back to
   $t=0$, and return that point.

Unlike part 1, this plot takes place in a volume, not in a plane. However, we
can analyze 3-d motion in the XYZ volume by projecting it into the XZ and YZ
planes, performing linear intersection analysis in those planes, and discarding
candidates where the $z(x)$ and $z(y)$ intersections differ or $z(x, y)$ is not
on the ray.

## Footnotes

In an unordered pairing, $\{p_0, p_1\} = \{p_1, p_0\}$. In an ordered pairing,
these are distinct.
