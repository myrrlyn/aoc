# Web Networking

This module describes a non-Cartesian graph of interlinked nodes, and provides
routing between them.

I originally made it for Y2023 D25, which does not have per-link cost or
per-node metadata.

## Overall Structure

The web uses a `Dictionary` to store all of its node names. The web itself is
a mapping of dictionary `Identifier`s to other `Identifier`s. This mapping
defines edges between nodes.

Additionally, the connection from one node to another is capable of storing all
nodes *beyond* the destination which can be best reached by traveling through
that edge.

For instance, in the graph

```text
A - B   F --+
|   |\ /    |
|   | C     |
|   |/ \    |
D - E - H - I
```

the best path from `A` to `I` is `ABCFI`, while the best path from `D` to `I` is
`DEHI`. Once discovered, the web has the following cached routes:

- Node `A`'s link to `B` contains `[C, F, I]`
- `B`'s link to `C` contains `[F, I]`
- `C`'s link to `F` contains `[I]`
- `F` has a link to `I`, and does not need any additional cached routes.
- `D`'s link to `E` contains `[H, I]`
- `E`'s link to `H` contains `[I]`
- `H` has a link to `I`, and does not need any additional cached routes.

Additionally, since paths are symmetric, the search algorithm records the
reverse paths:

- `I`'s link to `F` contains `[C, B, A]`
- `I`'s link to `H` contains `[E, D]`
- `F`'s link to `C` contains `[B, A]`
- `H`'s link to `E` contains `[D]`
- `C`'s link to `B` contains `[A]`

### Route Adjustments

If we then cut the `E <--> H` link, `D` still knows that `I` is reachable
through `H` and so a path-finding attempt would follow that known path rather
than spawn across all outbound links, such as `D <--> A`. However, `E` no longer
knows how to reach `I`, since `E`'s `C` link was never used in a best-path. As
such, `E` has to spawn a spider on both `B` and `C`. Both `B` and `C` know how
to reach `I`, and so the spiders can advance immediately along the existing
paths: `DECHI` wins and gets recorded in `E`s routing table, while the `DEBCHI`
spider observes that when it reaches `C`, its sibling has already been there,
and quits.
