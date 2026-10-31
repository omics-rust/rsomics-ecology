# Retained rarefaction fixtures

These files are byte-preserving copies from qualified source run
`40634594900`, attempt1, artifact `witness/`. No source program was rerun
to generate or adjust these expected values for the Rust implementation.

The original input tables/tree are team-owned historical test assets:
`rsomics-unifrac` at `a631051290342991f411b68d7c10735493162a25` and
`rsomics-subsample-counts` at `715ab363e0cabe91ddaa6457ef064930a9d278a9`.
Their original team terms are MIT OR Apache-2.0. Expected tables are separately
recorded outputs from those inputs, not copies of upstream program text;
running upstream software is not by itself a new data-license claim.
Relevant implementation notices are in THIRD_PARTY_NOTICES.md and licenses/.

The captured environment was scikit-bio0.7.3
(`a3beaa632077a532330343d5db0d8e30e58b6705`), BIOM2.1.17
(`2b81a013aae29004d9b82d13847450b27fd6b161`), actual NumPy2.5.4,
fresh same-seed PCG64 per sample and choice without replacement/shuffle.
Witness SHA256 `b6eb9045c2dceba4754821f90c8c553a3da221f04405364bbd0639fd4fd146a1`;
manifest SHA256 `34b96bfccadb55f6a2ff57e443c8089a1b3636171bea7be6b89c4d86ef727439`;
admitted verification SHA256
`2d99ce956ac4dbc0bbee0775f650e5fe7f1ce32efabc8509c39e4b394b0894ae`.

| Local file | Original witness path | SHA256 |
|---|---|---|
| primary-in.tsv | inputs/primary-counts.tsv | b79678b55391e8c43a8e47a84de4a71e92f2d738353526cc9af6547de53559a7 |
| primary-tree.nwk | inputs/primary-tree.nwk | 61ea355b828455e9cca5bc92ad99959ebf3afb0df6b846af7ad4f34206565cbd |
| primary-expected.tsv | cases/rarefied/counts.tsv | c67cb6a096031117a79443fe4210abbcc14df0126320fe29d7fdce2c15f5b8f6 |
| small-in.tsv | inputs/legacy-small.tsv | c8f56e0111400ce52398a253b182fede1af2e1016b200526987573024f5e06b4 |
| small-expected.tsv | sampling-regimes/small/counts.tsv | bd5d513386cadacf375779a6ec968c66a294bf30cc670ff9010228f6324826cb |
| big-in.tsv | inputs/legacy-big.tsv | 88ee972c87413a617591cc93bb679944ecb29811c19b70d72c00894e2fba46da |
| big-expected.tsv | sampling-regimes/big/counts.tsv | 6069980670c46feeed1bf326471e7dc973260173aba7d4b28891c49637462745 |

Primary is12features×6samples, depth8/seed42. Small is40×3, original totals
1036/1030/1188, depth200/seed42, Floyd branch. Big is300×1, total59297,
depth29648/seed123, partial tail-shuffle branch. Tests compare every captured
cell and identity exactly at one/four workers. Only the first header label
is normalized from source `#OTU ID` to native canonical `feature`.

The maximum exact-integer depth-one case in the integrated test is the
retained source boundary `[9007199254740991,0] → [1,0]`, seed42. It reaches
the64-bit bound path but does not independently expose selected raw indices
or the RNG word stream. Empty/all-zero depth0 success is an explicit native
bug fix, not the captured upstream IndexError status.
