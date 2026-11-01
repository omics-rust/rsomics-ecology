# Third-party notices

The alpha, phylogenetic-diversity and UniFrac implementation adapts behavior and
formulas from scikit-bio 0.7.3 (source commit
`a3beaa632077a532330343d5db0d8e30e58b6705`), copyright (c) 2013--,
scikit-bio development team. The modified BSD license is reproduced in
`licenses/scikit-bio-BSD-3-Clause.txt`; it applies to those adapted portions.
No upstream endorsement is implied.

Five nonphylogenetic beta modes adapt definitions/behavior from SciPy1.18.1
`scipy/spatial/distance.py`, read inside the original retained wheel.
Copyright (c) 2001-2002 Enthought, Inc. 2003, SciPy Developers. The BSD terms
from that wheel's top-level license are retained in
`licenses/scipy-BSD-3-Clause.txt`. No SciPy/OpenBLAS binaries or unrelated
bundled wheel components are copied into this product. The public named
source dispatch is compiled `_distance_pybind`, not the scalar Python bodies.

Rarefaction adapts the scikit-bio wrapper and BIOM 2.1.17 sparse sampling
behavior (BIOM source commit `2b81a013aae29004d9b82d13847450b27fd6b161`).
BIOM copyright (c) 2011-2017, The BIOM Format Development Team; its modified
BSD terms are retained in `licenses/biom-BSD-3-Clause.txt`.

The private random-number implementation adapts the recorded NumPy 2.5.4
SeedSequence, PCG64, bounded integer and `choice(shuffle=False)` bodies.
Applicable notices and terms are retained without changing upstream terms:

- `licenses/numpy-BSD-3-Clause.txt`: NumPy Developers, 2005-2025.
- `licenses/numpy-random-NCSA-BSD.md`: Kevin Sheppard, 2019; dual NCSA/BSD.
- `licenses/seed-sequence-MIT.txt`: Melissa E. O'Neill, 2015, and NumPy
  Developers, 2019; the separate notice in `bit_generator.pyx`.
- `licenses/pcg64-MIT.md`: Melissa O'Neill, 2014.
- `licenses/numpy-distributions.md`: original distribution-component notices.
  Only bounded uniform integers are adapted; no Julia-derived ziggurat
  implementation is included.

The original team-owned Rust assets and this product's original code use
MIT OR Apache-2.0 terms, reproduced in LICENSE-MIT and LICENSE-APACHE-2.0.
The notices above apply to their respective adapted portions. Registry
dependencies retain their own licenses. No upstream endorsement is implied.
Fixture provenance is recorded in `tests/data/rarefaction/PROVENANCE.md` and
`tests/data/beta/PROVENANCE.md`.
