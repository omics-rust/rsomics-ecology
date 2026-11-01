# Ecology implementation provenance

This is one private development slice of the planned ecology alpha/beta/
rarefaction product, not a revived micro-crate or a complete product release.
No Git history has been imported, rewritten or committed by this task. The
following clean team-owned source assets were inspected at their exact heads.

| Historical asset and commit | Classification and use |
|---|---|
| alpha-diversity `542030df8d84dca87d30e3b4a24cec6aaaf1ac0b` | Refactor then merge: nine selected formulas and per-sample summary concept; discard dense/raw count copies, old parser/help surface and `all` selection. Remaining metrics are deferred, not advertised. |
| faith-pd `45641adf130ae241733aee06bbd44d8ea48a9759` | Refactor then merge: occupied rootward branch concept; replace duplicate parsing and historical tree-field access with one prepared tree using phylo-tree 0.2. |
| phydiv `46cdf30219f394d353c417604247bd4ab3abba10` | Refactor then merge: source-rootedness/weighting formulas; replace duplicate topology and dense per-sample clearing with shared bounded touched-node accumulation. |
| unifrac `a631051290342991f411b68d7c10735493162a25` | Refactor then merge: `src/lib.rs` unweighted/weighted/normalized formulas and root-path concepts into `src/beta.rs` and existing prepared phylogeny. Retain qualified12-feature/six-sample table/tree and root/unary/extra-tip boundaries as assets. Discard mutable duplicate tree/parser, dense per-sample counts, pair-list/upper-vector, matrix/formatter duplication and missing-oracle skip harness. |
| subsample-counts `715ab363e0cabe91ddaa6457ef064930a9d278a9` | Refactor then merge: scalar SeedSequence/PCG64, source-width bounded integers and cutoff20 choice logic; replace dense parser/sums/output and old help surface with shared sparse Counts, atomic output, current common/help and fallible sampling workspaces. |
| beta-diversity `10f7564b08f333b984255256cf10244c5bf3d9a5` | Refactor then merge: `src/metric.rs` five selected formulas into direct sparse merges in `src/beta.rs`; valid integer/edge cases remain test concepts. Discard permissive parser, public old models/config/API, unselected aliases, pair+upper allocation, full JSON copy, live-oracle skip and old speed claims. Counts2e16/5e16 are rejection/formatter-only assets, outside the accepted integer domain. |

The shared count parser is newly organized around validated sparse
sample-major entries with complete IDs/order. The prepared-tree module uses
the current immutable `rsomics-phylo-tree` topology, traversal and tip index;
it does not expose a second tree API. Its only additional arrays are feature
mapping, branch lengths, postorder ranks and per-parallel-chunk reusable
accumulators/touched nodes. Beta retains sorted occupied branches once per
sample and normalized proportions. Normalized correction adds the two
sample proportions before multiplying by the shared tip-root length during
the sorted sparse pair merge, not cached per-sample mass. Shared all-tree
infinite-tip counts and internal-mask state retain source nonfinite-path
classification: omitted infinite tips and infinite internal paths contribute
`inf*0`/NaN, while occupied infinite tips may give a defined infinite
denominator. Double-empty remains0 before correction.
Lengths/postorder and sample accumulation are shared with alpha; no complete
edge-by-sample count array is created. Matrix storage is one fallibly allocated
square buffer, and serializers borrow rows. These are storage properties,
not a measured resource or speed claim.

Registry foundations are exactly common 0.12.3, help 0.4.0 and phylo-tree 0.2.0.
There are no path dependencies, Layer-B dependencies or new foundation APIs.
The existing common error, alias, atomic-output, result-envelope and thread
contracts and help's Clap presentation are used directly.

## Compatibility authority

Actual scikit-bio 0.7.3 source commit
`a3beaa632077a532330343d5db0d8e30e58b6705` is the alpha/PD behavior authority:
`skbio/diversity/alpha/_base.py`, `_chao1.py`, `_pd.py`, public
`alpha_diversity`, count validation and tree/taxon validation. Formulas were
read from these actual retained bodies and adapted, not guessed from names.
The BSD terms are retained in `licenses/scikit-bio-BSD-3-Clause.txt`.

Hosted source run `40634594900`, attempt 1, used the actual installed
NumPy 2.5.4 / SciPy 1.18.1 / scikit-bio 0.7.3 / BIOM 2.1.17 profile. Installed
Python bodies were byte-compared with pinned source before capture. BIOM
source commit `2b81a013aae29004d9b82d13847450b27fd6b161`, its sparse sampler,
and NumPy 2.5.4 SeedSequence/PCG64/bounded/choice bodies are the rarefaction
authority. Their relevant notices are retained in `licenses/`.

Retained source proof:

- Witness `b6eb9045c2dceba4754821f90c8c553a3da221f04405364bbd0639fd4fd146a1`.
- Manifest `34b96bfccadb55f6a2ff57e443c8089a1b3636171bea7be6b89c4d86ef727439`.
- Immutable verification `2d99ce956ac4dbc0bbee0775f650e5fe7f1ce32efabc8509c39e4b394b0894ae`.

The source comparator freezes absolute/relative `1e-10` bands separately
from native outputs and checks exact identities/order and nonfinite classes.
It records complete native artifacts; it does not scientifically replay the
source, derive reference outputs from native formulas or alter expected data.

The native integer/length domain is intentionally stricter than some source
observations: no fractional/nonfinite/negative counts, counts/sample totals
at most 2^53-1, unique nonempty identities/named tree tips, finite nonnegative
required branch lengths. Source root heuristics and stored root lengths are
preserved within that domain. No claim is made for inputs outside it.

Rarefaction uses a fresh same-seed scalar-u128 PCG64 per sample, default
SeedSequence pool4, no spawn key, and `choice(replace=False, shuffle=False)`.
Mixed32/64 draws preserve the high32 cache; population>10000 and
depth>population/20 selects tail shuffle, otherwise Floyd. The shared count
domain remains at most 2^53-1. Depth zero intentionally returns same-dimensional
zeros, fixing the captured upstream empty-data IndexError by explicit user
decision. Depth>total fails before output rather than inheriting the old
helper's zeroed undersized columns.

Beta uses the actual retained public `beta_diversity` driver, `_unifrac.py`
kernels/multiple-sample setup/branch correction, `_phylogenetic.pyx` tip-root
accumulation and tree validator. Nonphy formulas and named optimized dispatch
were read from `scipy/spatial/distance.py` inside the original1.18.1 wheel;
member SHA256 `e0cd0871d483a355eaf656ee0849e6579af45f579f33ec4ca326df2626b3f196`.
The five public named drivers use `_distance_pybind.pdist_*`, not scalar
Python callbacks. Its C++ kernel bodies were not available in the wheel and
are not claimed inspected. Retained original observations control numerical
acceptance, not algebraic resemblance or historical loose tolerances.

Beta preserves stored-root contribution to branch sums and every tip-root
path, source root-degree<=2 (including unary), extra tips and zero-feature
all-zero matrices. Strict tree validation still precedes native zero-feature
success, unlike the source driver's malformed-tree bypass. Distinct undefined
Bray double-empty or normalized zero-correction pairs fail atomically with
metric/sample IDs. Normalized double-empty, unweighted denominator0, Jaccard
union0 and Canberra zero coordinates remain defined0; the diagonal is0.
No epsilon, clamping, equal-positive shortcut or source-NaN substitution.
Two integrated boundary cases derive from retained source/IEEE arithmetic:
smallest-positive-subnormal path with equal0.5 tip proportions is defined0;
extra zero-abundance tips whose finite branch lengths overflow root paths
make normalized correction nonfinite. They are not new upstream captures.
`tests/data/beta/source-files.json` binds the unmodified requests/results;
`scripts/compare_beta_source.py` checks their original identities and keeps
source nonfinite observations separate from native expected errors.

Frozen primary12×6, small40×3 and big300×1 tables exercise actual source counts
at workers1/4. The maximum exact-integer fixture reaches the 64-bit bound
path with a depth-one Floyd workspace; its one-positive-feature result is
not an independent raw RNG-stream witness. The alpha comparator can consume
the native-rarefied primary table with unchanged expected values and bands.
No scientific source rerun, seed search or sampling-parameter tuning is used.

## Remaining product gates

The separately qualified real community study (41,328 features / 1,105 samples /
314,497 nonzeros) motivated sparse storage. Its raw files are not modified or
benchmarked here. Allocation bounds are not measured RSS or throughput proof.
The control plane adopted the baseline11-alpha/rarefaction/alpha real-study
receipt `.autopilot/oracles/ecology-whole-real-2026-11-01.json`, SHA256
`179835de584fe258fcf62859565203d2d0c4742c27fe32c0636015341c7bd3b7`.
That receipt does not accept beta, and its timing/scaling is not beta evidence.
The new beta wave still needs full-study correctness and its own per-metric/
suite performance gates, one whole-feature independent review and native
Linux/macOS x86_64/aarch64 candidate tests before stable/public release. `publish = false`
remains in force. No registry name, remote repository or publication is created.
