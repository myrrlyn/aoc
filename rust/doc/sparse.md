# Sparse Cartesian Grids

The `wyz_aoc::coords::spaces::sparse` module defines two data structures,
`Cartesian2D` and `Cartesian3D`, which implement *sparse* storage of objects in
a regularly-measured, finite, quantized space. As the names indicate, the space
can have either two or three dimensions.

The sparse storage means that these structures only consume memory for each
object embedded in the space, and not for each cell that the space logically
describes. This is in contrast with the `wyz_aoc::coords::spaces::::dense`
module, whose structures require the allocation of memory for every cell. Sparse
storage is roughly linear with the number of stored objects, while dense storage
is *cubic* according to the size of the dimensions.

The trade-off is that the sparse structures entail some memory fragmentation and
computational cost when traversing the space, whereas the dense structures can
traverse essentially for free. However, a sparse structure needs to be *fairly*
heavily populated before the cost approaches exceeding the memory weight of the
dense structure.

## Implementation

Internally, the sparse structure is a stack of B-Trees. Each tree layer
represents one dimension
