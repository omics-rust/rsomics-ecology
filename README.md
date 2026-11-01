# rsomics-ecology

One sparse count-table diversity product:11 alpha metrics, seeded rarefaction
and eight beta modes. The supported interface is one executable using the
shared rsomics help, worker, result-envelope and atomic-output contracts.

The implementation has accepted full-study alpha/rarefaction and all eight
beta comparisons plus four-native-platform tests. The [0.1.0 release](https://github.com/omics-rust/rsomics-ecology/releases/tag/v0.1.0)
provides native Linux/macOS x86_64/aarch64 archives, checksums and a complete
locked source package. Clean source installation and the full installed
workflow pass on all four native platforms; see [CI](https://github.com/omics-rust/rsomics-ecology/actions/runs/41341218191).
The crate is not published to crates.io; `publish = false` remains intentional.

## Installation

Select the release's `linux-x86_64`, `linux-aarch64`,
`macos-x86_64` or `macos-aarch64` archive, verify it against the release's
`SHA256SUMS`, and extract it. Run the enclosed `rsomics-ecology` executable;
the other files retain project and dependency notices. Candidates are unsigned
and not notarized. Linux archives use GNU libc, not musl.

To build from a complete GitHub source package or source checkout with Rust1.91
or newer, enter its extracted root and run:

```sh
cargo install --path . --locked
rsomics-ecology --help
```

## Quick start

```sh
rsomics-ecology alpha --counts community.tsv --output alpha.tsv --threads 4
rsomics-ecology rarefy --counts community.tsv --depth 200 --seed 42 \
  --output rarefied.tsv --threads 4 --json
rsomics-ecology alpha --counts rarefied.tsv --output rarefied-alpha.tsv
rsomics-ecology beta --counts rarefied.tsv --metric braycurtis \
  --output braycurtis.tsv --threads 4
rsomics-ecology beta --counts community.tsv --tree phylogeny.nwk \
  --metric weighted_normalized_unifrac --format json --output unifrac.json
rsomics-ecology alpha --counts community.tsv --tree phylogeny.nwk \
  --metrics observed_features,shannon,faith_pd,phydiv \
  --pd-rooted auto --pd-weight 0.25 --format json --output alpha.json --json
```

The library exposes only `run_cli`; count models, tree preparation and compute
functions remain private. This release slice does not promise a scientific
Rust library API.

## Input

Use named files. Counts are unquoted feature-by-sample TSV. The first header
must be exactly `feature` or `#OTU ID`, followed by at least one unique,
nonempty sample ID. Each following row names one unique feature and supplies
exactly one count per sample. IDs retain their original order and spelling;
surrounding ID whitespace and control characters are errors. Numeric fields
may have surrounding whitespace. Empty fields, blank records, comment lines,
signed/fractional/nonfinite values and ragged rows are errors. Counts and
sample totals must be integers in `0..=9007199254740991`. Zero features and
zero-total samples are retained; a header with no feature rows is valid.

The TSV reader streams rows into sample-major nonzero entries. It retains
all feature/sample identities without building a dense count matrix.

Phylogenetic metrics require a Newick tree. Table features must each identify
one non-root tip; additional tree tips are allowed. Tip IDs must be unique
and named. Every non-root length must exist and be finite/nonnegative. The
root's missing length is zero; a stored root length is preserved. Negative
lengths are rejected even though the qualified source accepts them. Parsing
uses `rsomics-phylo-tree` defaults, including conversion of unquoted
underscores to spaces; quote Newick labels to preserve literal underscores.

## Metrics

`--metrics` accepts an ordered, comma-separated selection, without duplicates:

| Label | Qualified scikit-bio 0.7.3 contract |
|---|---|
| `observed_features` | Number of positive features |
| `shannon` | Natural-log entropy |
| `simpson` | `1 - dominance`, no finite-sampling correction |
| `dominance` | Sum of squared proportions |
| `inv_simpson` | Reciprocal dominance, no finite-sampling correction |
| `simpson_e` | Reciprocal of observed richness times dominance |
| `pielou_e` | Shannon divided by log richness; one taxon returns one |
| `chao1` | Bias-corrected Chao1 |
| `goods_coverage` | One minus singleton count divided by total |
| `faith_pd` | Occupied rootward branch length; root degree must be at most two |
| `phydiv` | Generalized PD with explicit root/weight options |

The first nine metrics, in the order above, are the default. There is no
ambiguous `all` selector. `--tree` is required only for the last two metrics.
`--pd-rooted` and `--pd-weight` require `phydiv`.

Generalized PD defaults to `--pd-rooted auto --pd-weight 0`.
`auto` means rooted exactly when the root has two children. `rooted` and
`unrooted` explicitly select the source calculation; neither asserts
biological rootedness. Weight zero is unweighted, one is fully weighted,
and an intermediate exponent gives partial abundance weighting. Unrooted
mode excludes branches above the community LCA when unweighted and uses
`2 * min(p, 1-p)` balances when weighted. Stored root lengths affect rooted
metrics, not unrooted metrics.

The same per-sample summary serves all selected nonphylogenetic metrics.
One prepared immutable phylogeny serves Faith and generalized PD. Workers
reuse bounded node accumulators and touched-node lists, not an edge-by-sample
array; reductions within each sample have a fixed order. This layout is a
memory design, not a measured speed or RSS claim.

## Beta diversity

`beta` requires exactly one `--metric`: `braycurtis`, `jaccard`, `euclidean`,
`canberra`, `cityblock`, `unweighted_unifrac`, `weighted_unifrac` or
`weighted_normalized_unifrac`. `--tree` is required exactly for the three
UniFrac metrics, which share Faith's source root-degree<=2 rule. Unary roots,
additional tips and stored root lengths are preserved. The normalized name
maps to source `weighted_unifrac(normalized=True)`; ordinary weighted UniFrac
uses the source's unnormalized default. No `all`, aliases or root switch.

Nonphylogenetic modes use raw counts, except Jaccard uses presence/absence.
Sparse feature lists are merged directly. UniFrac prepares occupied branches
once per sample through the same accumulation used by alpha, with bounded
per-worker dense workspaces and shared postorder/lengths. Pair kernels merge
sparse branches. Normalized correction multiplies tip-root lengths by the
pair's summed proportions in that same merge. Shared all-tree path state
preserves nonfinite correction errors for unoccupied/extra infinite tips or
infinite internal masked paths, without rejecting defined infinite-denominator
results. No per-sample
correction-mass cache changes subnormal rounding or nonfinite classification.
The output has one checked/fallibly allocated square buffer; workers write
disjoint upper rows, select the first failing index pair
deterministically, then mirror only after success. JSON borrows matrix rows.
There is no full pair list or dense sample-by-full-tree array. Sparse branch
storage can still grow to the occupied sample-by-tree worst case; none of
these design choices establishes performance or RSS superiority.

Each invocation writes one complete finite labelled matrix. Zero-feature
input yields zeros in every mode after strict input/tree validation. Jaccard
empty union, Canberra double-zero coordinates, unweighted UniFrac zero branch
denominator and normalized double-empty samples are defined0. Bray-Curtis
with features and two empty samples is undefined. Normalized weighted
UniFrac with nonempty counts but zero tip-root correction is undefined,
including identical positive samples on zero-length paths. Only distinct
pairs are evaluated; diagonals are0. Undefined/nonfinite pairs fail with
metric and both sample IDs before replacing output, not NaN-to-zero.

TSV starts with an empty header cell and exact ordered sample labels; finite
numbers round-trip. Numeric source compatibility does not promise Python
float-repr byte identity. JSON schema1 records `metric`, `source_profile`,
`tree_profile`, `sample_ids` and row-major `values`, with no null/nonfinite
cells. The result envelope records metric/source/tree provenance separately.

## Rarefaction

`rarefy` samples each column without replacement and writes a canonical
`feature`-headed TSV accepted directly by `alpha` and `beta`. Both `--depth` and
`--seed` are required. The seed is a decimal nonnegative scalar `u128`;
every sample initializes its own same-seed PCG64 stream. Worker scheduling
does not change the table. The result envelope records depth, decimal seed
and profile `skbio-0.7.3/biom-2.1.17/numpy-2.5.4-pcg64-choice-shuffle-false`.
This is a pinned profile, not compatibility with arbitrary NumPy versions,
seed forms or an omitted seed.

Depth greater than any sample total is an error before writing output.
Depth zero preserves all feature/sample dimensions and produces zeros,
including no-feature and all-zero tables. This deliberately fixes the
captured upstream BIOM empty-data `IndexError`; the success status is not
claimed to be upstream-compatible. Exact-total sampling retains the input
counts. All identities and their order are preserved.

Sparse counts remain sample-major. Sorted sampled read positions are mapped
onto positive feature spans without expanding a repeated-read feature array.
Floyd selection uses depth-sized storage plus a lookup table. The source's
large-population/high-depth branch still needs a population-sized `u64`
tail-shuffle workspace; allocation failure is an error. TSV emission uses
one cursor per sample, not a second dense matrix. These are storage-design
properties, not measured throughput or RSS improvements.

## Output and failures

`--output` is a required named destination. The shared rsomics atomic-output
contract keeps an existing destination intact on parsing, validation,
calculation or write failure. Input/output aliases, including hard links,
are rejected. `--threads` uses the shared nonzero worker control;
`--json` emits the common command-result envelope to stdout, independently
of alpha/beta's named data file `--format tsv|json`. Rarefaction always writes TSV.

Alpha TSV contains a `sample` header and selected metric columns. Source-unavailable
values are spelled `nan` (`inf`/`-inf` for corresponding nonfinite results).
JSON schema version 1 retains `feature_ids`, `sample_ids`, `metrics`, effective
generalized-PD options and sample-major `values`. Each value is an object:
finite results have `value`; unavailable results have `value: null`, a
typed `reason` and exact `nonfinite` class. JSON never contains a bare NaN.
For empty communities observed richness, corrected Chao1, Faith and
generalized PD are zero. The other selected alpha metrics are unavailable
with reason `empty_community`, matching the qualified source NaN class.

## Verification

Run local commands through `bash scripts/guarded-cargo.sh` from this repository.
The guard verifies that Cargo home, resolved target, rustup/toolchain, TMPDIR
and caches physically remain external before execution. Unrelated boot-disk
occupancy does not block work. It is a local macOS development tool, not a
portable installer.
For formatting, use `--exec` with the pinned toolchain's `cargo-fmt fmt`
executable directly; `cargo fmt` on this host otherwise selects an
unconfigured rustup shim. No global default or new installation is required.

Integrated tests cover multi-alpha and rarefaction file workflows at one/four
workers and compact invalid/atomic cases. Frozen primary, Floyd and tail-
shuffle expected tables are independent retained source captures, not
native-generated expectations. The private, standard-
library-only `scripts/compare_alpha_source.py` compares both original and
source-rarefied tables across all 15 retained source alpha/PD profiles at
one/four workers, plus valid zero/root/extra-tip cases. It imports no
scientific packages and does not rerun or refit the source. Absolute and
relative bands remain `1e-10` each; identities/order and nonfinite classes
are exact. Pass `--binary`, `--witness` and a new external `--output` directory.
Use `--rarefied-counts` to feed an actual native-rarefied primary table into
the complete alpha/PD comparison after validating it against the capture.

The integrated beta workflow uses original40634594900 frozen source data,
then actual depth8/seed42 rarefaction and reload, for all eight modes plus
alpha/PD at native1/4. Compact boundaries retain source NaNs separately from
native expected errors. `scripts/compare_beta_source.py` accepts `--binary`,
`--witness` and a new external `--output` directory and verifies frozen-file
hashes, source identities/domain, round-trip TSV/JSON and native1/4 bytes at
the same fixed1e-10 bands. It uses only Python's standard library and does
not run an upstream oracle. See `tests/data/beta/PROVENANCE.md`.

Owning CI runs the six complete integrations in debug, release and packaged
release on actual Linux/macOS x86_64/aarch64 hosts, then installs the complete
Cargo source package and exercises the installed rarefy/11-alpha/eight-beta
lifecycle. Original CLI outputs and package/notice identities are retained in
delivery artifacts. Retained source observations are data fixtures; this is
not a new live upstream execution on each platform.

## Qualified performance and scope

The control-plane receipt `ecology-whole-real-2026-11-01.json` accepts the
baseline11-alpha/depth3-rarefaction/alpha profile on the complete41,328-feature,
1,105-sample study. It binds implementation `e1b03cb6` and measured executable
`58cd2ec1`, not a timing of newly packaged binaries. Its one/four-worker gates
pass, but native worker scaling is limited.

`ecology-beta-eight-real-2026-11-02.json` accepts separate full-study upstream
comparisons and serial throughput/RSS gates for all eight beta modes at
implementation `111adf5e`, measured Linux executable `d7dc1832`. Each uses the
same complete study and the three UniFrac modes retain the full162,331-tip
tree. The UniFrac source baseline is the declared contiguous-public layout;
the2,858,608,480-byte layout copy is charged. It is not unchanged-default source
memory. Metrics ran on different actual hosts, so their times are not summed.
Four-worker CPU, faults and RSS costs remain recorded; no universal scaling
claim is made. Delivery executables are not remeasured or assigned old timings.

Those records and `ecology-native-beta-2026-11-01.json` live in
[rsomics-world](https://github.com/omics-rust/rsomics-world/tree/main/.autopilot/oracles).
The older compact native receipt does not qualify a dedicated source-on-native
gate; the accepted full-study comparisons are separate evidence. The qualified
GitHub installation/source package is also separate from the held registry
publication; no new timing is assigned to its executables.

BIOM import, continuous abundances, ordination, association, inference and
the historical alpha metrics outside this selection are excluded.
See [MERGE.md](MERGE.md), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).
