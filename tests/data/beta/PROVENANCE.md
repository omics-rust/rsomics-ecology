# Frozen beta qualification data

These JSON files are unmodified original source requests and observations,
not native-generated expectations. They come from hosted run40634594900,
attempt1, artifact witness at:

`/Volumes/Zane's HDD/rsomics-fixtures/evidence/ecology-diversity-oracle-2026-10-28/run-40634594900/artifact/witness`.

The source was Python3.12.15, scikit-bio0.7.3 commit
`a3beaa632077a532330343d5db0d8e30e58b6705`, NumPy2.5.4, SciPy1.18.1 and
BIOM2.1.17. Witness SHA256 is
`b6eb9045c2dceba4754821f90c8c553a3da221f04405364bbd0639fd4fd146a1`;
manifest SHA256 is
`34b96bfccadb55f6a2ff57e443c8089a1b3636171bea7be6b89c4d86ef727439`.
`source-files.json` records every copied byte identity; the stdlib-only beta
comparator verifies each against both this fixture and the original witness.

The original12-feature/six-sample request and depth8/seed42 source-rarefied
request retain all eight beta matrices plus11 alpha/PD profiles. Boundary
requests retain exactly the original available beta observations: all eight
for zero_samples/no_features, and three UniFrac modes for root_length,
zero_lengths, extra_tips, unary and polytomy. Missing five-metric observations
in the latter groups are not synthesized. Alpha/rarefaction assets retain
their separate provenance in `../rarefaction/PROVENANCE.md`.

The integrated test first computes all eight matrices, runs real native
rarefaction at depth8/seed42, reloads its actual TSV, and recomputes all eight
plus alpha/PD. Native1/4 JSON and TSV matrix bytes must match. Comparison uses
unchanged absolute1e-10 plus relative1e-10 and exact IDs/order/shape/symmetry.

Original source NaNs and warnings remain untouched. Bray-Curtis zero_samples
has two off-diagonal NaNs for empty-a/empty-b. Normalized zero_lengths has30
off-diagonal NaNs. Native finite-matrix policy reports these as metric/pair
errors and leaves destination bytes unchanged; they are not zero/null
numeric goldens. Other source double-empty cases are defined0. Zero-feature
matrices are0 for all eight. Diagonals are constructed0, not self-divisions.

The native domain additionally requires integer counts/totals<=2^53-1, unique
nonempty identities, named tips and finite nonnegative branch lengths, even
for no-feature UniFrac. Source empty-dimension dispatch may bypass malformed
tree validation; native remains strict. All non-root lengths are required,
root degree<=2 (unary accepted), stored root length included, and extra tips
allowed. Equal positive zero-root-path samples still fail normalized UniFrac.
Historical huge2e16/5e16 counts are outside this domain, never successful data.

This capture is compact compatibility evidence, not whole-study beta
qualification, native four-platform acceptance, performance or a release.
No scientific Python package/source was executed on this Mac for this wave.
