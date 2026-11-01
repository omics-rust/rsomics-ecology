# Rust dependency licenses and notices

The recorded normal/build graphs contain 46 registry packages per native target
and 47 in their union: Linux and macOS, each on x86_64 and aarch64. These are
resolved dependencies, including procedural-macro tools, not a claim that every
package is linked into the executable or a legal-clearance certification.
Dev-only dependencies are excluded unless also enabled through normal/build
edges. In particular, `tempfile` is a normal dependency of `rsomics-common`.

Original license, copyright and notice bodies are retained in
[`licenses/registry/`](licenses/registry/).
[`licenses/provenance.json`](licenses/provenance.json) records the locked crate
archive checksums, per-target graphs, original archive member paths, and both
original and copied body sizes and SHA-256 hashes. Native delivery must match
each recorded graph against its actual locked Cargo normal/build tree.

All 99 bodies are retained byte-for-byte; no newline normalization, generic
license-template substitution or SPDX-only supplementation was required. The
Unicode terms in `unicode-ident` and the `COPYRIGHT` and LLVM-exception texts
in `rustix` and `linux-raw-sys` are included. No regex crate is in these graphs.

Adapted scientific implementation notices remain separately in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and the eight existing
files directly under `licenses/`; fixture provenance remains under `tests/data/`.
Those sources and fixtures are not registry dependency packages.

## Recorded packages and original texts

| Package | Declared license expression | Retained license and notice texts |
|---|---|---|
| anstream 1.0.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/anstream-1.0.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/anstream-1.0.0/LICENSE-MIT) |
| anstyle 1.0.14 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/anstyle-1.0.14/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/anstyle-1.0.14/LICENSE-MIT) |
| anstyle-parse 1.0.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/anstyle-parse-1.0.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/anstyle-parse-1.0.0/LICENSE-MIT) |
| anstyle-query 1.1.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/anstyle-query-1.1.5/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/anstyle-query-1.1.5/LICENSE-MIT) |
| bitflags 2.13.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/bitflags-2.13.2/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/bitflags-2.13.2/LICENSE-MIT) |
| cfg-if 1.0.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/cfg-if-1.0.5/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/cfg-if-1.0.5/LICENSE-MIT) |
| clap 4.6.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/clap-4.6.7/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/clap-4.6.7/LICENSE-MIT) |
| clap_builder 4.6.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/clap_builder-4.6.7/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/clap_builder-4.6.7/LICENSE-MIT) |
| clap_derive 4.6.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/clap_derive-4.6.7/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/clap_derive-4.6.7/LICENSE-MIT) |
| clap_lex 1.1.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/clap_lex-1.1.1/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/clap_lex-1.1.1/LICENSE-MIT) |
| colorchoice 1.0.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/colorchoice-1.0.5/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/colorchoice-1.0.5/LICENSE-MIT) |
| crossbeam-deque 0.8.8 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/crossbeam-deque-0.8.8/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/crossbeam-deque-0.8.8/LICENSE-MIT) |
| crossbeam-epoch 0.9.21 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/crossbeam-epoch-0.9.21/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/crossbeam-epoch-0.9.21/LICENSE-MIT) |
| crossbeam-utils 0.8.23 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/crossbeam-utils-0.8.23/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/crossbeam-utils-0.8.23/LICENSE-MIT) |
| either 1.19.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/either-1.19.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/either-1.19.0/LICENSE-MIT) |
| errno 0.3.14 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/errno-0.3.14/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/errno-0.3.14/LICENSE-MIT) |
| fastrand 2.5.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/registry/fastrand-2.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/fastrand-2.5.0/LICENSE-MIT) |
| getrandom 0.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/getrandom-0.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/getrandom-0.4.3/LICENSE-MIT) |
| heck 0.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/heck-0.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/heck-0.5.0/LICENSE-MIT) |
| is_terminal_polyfill 1.70.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/is_terminal_polyfill-1.70.2/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/is_terminal_polyfill-1.70.2/LICENSE-MIT) |
| itoa 1.0.18 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/itoa-1.0.18/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/itoa-1.0.18/LICENSE-MIT) |
| libc 0.2.191 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/libc-0.2.191/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/libc-0.2.191/LICENSE-MIT) |
| linux-raw-sys 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [COPYRIGHT](licenses/registry/linux-raw-sys-0.12.1/COPYRIGHT), [LICENSE-APACHE](licenses/registry/linux-raw-sys-0.12.1/LICENSE-APACHE), [LICENSE-Apache-2.0_WITH_LLVM-exception](licenses/registry/linux-raw-sys-0.12.1/LICENSE-Apache-2.0_WITH_LLVM-exception), [LICENSE-MIT](licenses/registry/linux-raw-sys-0.12.1/LICENSE-MIT) |
| memchr 2.8.3 | Unlicense OR MIT | [COPYING](licenses/registry/memchr-2.8.3/COPYING), [LICENSE-MIT](licenses/registry/memchr-2.8.3/LICENSE-MIT), [UNLICENSE](licenses/registry/memchr-2.8.3/UNLICENSE) |
| once_cell 1.21.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/once_cell-1.21.4/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/once_cell-1.21.4/LICENSE-MIT) |
| proc-macro2 1.0.107 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/proc-macro2-1.0.107/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/proc-macro2-1.0.107/LICENSE-MIT) |
| quote 1.0.47 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/quote-1.0.47/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/quote-1.0.47/LICENSE-MIT) |
| rayon 1.12.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/rayon-1.12.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/rayon-1.12.0/LICENSE-MIT) |
| rayon-core 1.13.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/rayon-core-1.13.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/rayon-core-1.13.0/LICENSE-MIT) |
| rsomics-common 0.12.3 | MIT OR Apache-2.0 | [LICENSE-APACHE-2.0](licenses/registry/rsomics-common-0.12.3/LICENSE-APACHE-2.0), [LICENSE-MIT](licenses/registry/rsomics-common-0.12.3/LICENSE-MIT) |
| rsomics-help 0.4.0 | MIT OR Apache-2.0 | [LICENSE-APACHE-2.0](licenses/registry/rsomics-help-0.4.0/LICENSE-APACHE-2.0), [LICENSE-MIT](licenses/registry/rsomics-help-0.4.0/LICENSE-MIT) |
| rsomics-phylo-tree 0.2.0 | MIT OR Apache-2.0 | [LICENSE-APACHE-2.0](licenses/registry/rsomics-phylo-tree-0.2.0/LICENSE-APACHE-2.0), [LICENSE-MIT](licenses/registry/rsomics-phylo-tree-0.2.0/LICENSE-MIT) |
| rustix 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [COPYRIGHT](licenses/registry/rustix-1.1.6/COPYRIGHT), [LICENSE-APACHE](licenses/registry/rustix-1.1.6/LICENSE-APACHE), [LICENSE-Apache-2.0_WITH_LLVM-exception](licenses/registry/rustix-1.1.6/LICENSE-Apache-2.0_WITH_LLVM-exception), [LICENSE-MIT](licenses/registry/rustix-1.1.6/LICENSE-MIT) |
| same-file 1.0.6 | Unlicense/MIT | [COPYING](licenses/registry/same-file-1.0.6/COPYING), [LICENSE-MIT](licenses/registry/same-file-1.0.6/LICENSE-MIT), [UNLICENSE](licenses/registry/same-file-1.0.6/UNLICENSE) |
| serde 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/serde-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/serde-1.0.229/LICENSE-MIT) |
| serde_core 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/serde_core-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/serde_core-1.0.229/LICENSE-MIT) |
| serde_derive 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/serde_derive-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/serde_derive-1.0.229/LICENSE-MIT) |
| serde_json 1.0.154 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/serde_json-1.0.154/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/serde_json-1.0.154/LICENSE-MIT) |
| strsim 0.11.1 | MIT | [LICENSE](licenses/registry/strsim-0.11.1/LICENSE) |
| syn 3.0.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/syn-3.0.9/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/syn-3.0.9/LICENSE-MIT) |
| tempfile 3.27.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/tempfile-3.27.0/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/tempfile-3.27.0/LICENSE-MIT) |
| terminal_size 0.4.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/terminal_size-0.4.4/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/terminal_size-0.4.4/LICENSE-MIT) |
| thiserror 2.0.21 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/thiserror-2.0.21/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/thiserror-2.0.21/LICENSE-MIT) |
| thiserror-impl 2.0.21 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/registry/thiserror-impl-2.0.21/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/thiserror-impl-2.0.21/LICENSE-MIT) |
| unicode-ident 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [LICENSE-APACHE](licenses/registry/unicode-ident-1.0.27/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/unicode-ident-1.0.27/LICENSE-MIT), [LICENSE-UNICODE](licenses/registry/unicode-ident-1.0.27/LICENSE-UNICODE) |
| utf8parse 0.2.2 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/registry/utf8parse-0.2.2/LICENSE-APACHE), [LICENSE-MIT](licenses/registry/utf8parse-0.2.2/LICENSE-MIT) |
| zmij 1.0.23 | MIT | [LICENSE-MIT](licenses/registry/zmij-1.0.23/LICENSE-MIT) |


The project license texts are [`LICENSE-MIT`](LICENSE-MIT) and
[`LICENSE-APACHE-2.0`](LICENSE-APACHE-2.0). Dependencies retain their own terms.
No upstream endorsement is implied.
