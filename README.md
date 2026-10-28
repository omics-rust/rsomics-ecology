# rsomics-ecology

Private development of one sparse count-table diversity product. Only the
complete alpha workflow is implemented here. This is not a stable release:
the intended first product also needs beta diversity, rarefaction, native
four-platform acceptance and representative performance evidence.

```sh
rsomics-ecology alpha --counts community.tsv --output alpha.tsv --threads 4
rsomics-ecology alpha --counts community.tsv --tree phylogeny.nwk \
  --metrics observed_features,shannon,faith_pd,phydiv \
  --pd-rooted auto --pd-weight 0.25 --format json --output alpha.json --json
```

The executable is the supported interface. The library exposes only
`run_cli`; count models, tree preparation and compute functions are private.
`publish = false` is intentional. No remote repository is configured.

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

## Output and failures

`--output` is a required named destination. The shared rsomics atomic-output
contract keeps an existing destination intact on parsing, validation,
calculation or write failure. Input/output aliases, including hard links,
are rejected. `--threads` uses the shared nonzero worker control;
`--json` emits the common command-result envelope to stdout, independently
of the named data file's `--format tsv|json`.

TSV contains a `sample` header and selected metric columns. Source-unavailable
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
The guard is copied unchanged from the owning rsomics-deseq tool and checks
external Cargo home/resolved target/TMPDIR and boot APFS occupancy before
execution. It is a local macOS development tool, not a portable installer.
For formatting, use `--exec` with the pinned toolchain's `cargo-fmt fmt`
executable directly; `cargo fmt` on this host otherwise selects an
unconfigured rustup shim. No global default or new installation is required.

Two integrated release tests cover the full multi-alpha file workflow at
one/four workers and compact invalid/atomic cases. The private, standard-
library-only `scripts/compare_alpha_source.py` compares both original and
source-rarefied tables across all 15 retained source alpha/PD profiles at
one/four workers, plus valid zero/root/extra-tip cases. It imports no
scientific packages and does not rerun or refit the source. Absolute and
relative bands remain `1e-10` each; identities/order and nonfinite classes
are exact. Pass `--binary`, `--witness` and a new external `--output` directory.

No beta/rarefy command, adapter, fallback or empty future module is present.
Their pending public boundary decisions are not chosen by this alpha slice.
BIOM import, continuous abundances, ordination, association, inference and
the historical alpha metrics outside this selection are excluded.
See [MERGE.md](MERGE.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
