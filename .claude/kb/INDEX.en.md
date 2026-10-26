# kb index

[中文](INDEX.md) · **English** · [日本語](INDEX.ja.md)

**Rules do not belong in kb; they belong in [CLAUDE.md](../../CLAUDE.md).** kb holds only facts, decisions, surveys, and measured data.

| File | Contents |
|---|---|
| [decisions.md](decisions.md) | Decision index: number, short name, status, progress, links to full texts |
| [decisions/](decisions/) | One file per decision (`NN-short-name.md`), with full text and rationale |
| [decisions-history.md](decisions-history.md) | Changelog of all decisions |
| [../../crates/singlefs-checker/invariants/README.md](../../crates/singlefs-checker/invariants/README.md) | Invariant list split into 9 files in the same directory by category; README holds notes, status, and count registry. checker is its executable form |
| [prior-art.md](prior-art.md) | Survey of external designs, including sources and measurement basis |
| [pitfalls.md](pitfalls.md) | Pitfalls checklist. Review every time a design decision is made |
| [experiments.md](experiments.md) | Experiment index: number, short name, status, links to full texts |
| [experiments/](experiments/) | One file per experiment (`NN-short-name.md`): what is measured, criteria, and measurement basis |
| [experiments-history.md](experiments-history.md) | Changelog of all experiments |
| [checks-owed.md](checks-owed.md) | Owed checks: checks we know we need to block but cannot enforce yet, including preconditions |
| [pending-rules.md](pending-rules.md) | Pending rules: clauses withdrawn from shared SOP because they cannot be enforced as failing checks, preserved verbatim with explanations of what is needed to make them rules |
| [tooling.md](tooling.md) | Tooling and environment facts: current status and gaps of local LLMs, Rust toolchain, and QEMU harness |
| [vm-harness.md](vm-harness.md) | How to run an experiment inside a VM on a **real block device**: three preconditions, hygiene checks, which verification path is added in VM, known-answer gates for masked CPU features, independent device-side recording (blklogwrites), auxiliary root and disk image preallocation, and measurement basis for timing and block-layer counts |
| [verification-build.md](verification-build.md) | How the three verification methods (checker, transaction layer, crash-point replay) land: which settled clauses each consumes, which unsettled items block each, what can be lifted from research/, first-version minimal scope, pending user settlements at the time (only T1 (regular tier wall-clock budget) remained unsettled), crash amplification process and the two tiers set by the user |
| [layout/](layout/) | Which bytes are newly written in each milestone, one file per milestone: `01-first-txn.md` new-pool file creation (each segment and field points to a decision item, with width and values; the segment sequence registry is also here), `02-second-txn.md` overwrite, release, rollback and reuse newly written patterns, `03-third-txn.md` deletion newly written patterns |
| [milestone/](milestone/) | Milestone planning, one file per milestone (`NN-short-name.md`): what each step intends to implement, anticipated details, acceptance criteria, written bytes, and encountered decision points; written based on judgment at the time and updated before starting each step |
| [feature-bits.md](feature-bits.md) | Feature bit ledger: incompat / compat_ro / compat three bitmaps of 256 bits each, one line per allocated bit; the sole authoritative registry for bit numbers is settled item 4 of D15 (Format Freeze Policy) |
| [freeze-layer-membership.md](freeze-layer-membership.md) | Freeze layer membership registry: one line per structure, which layer of settled item 7 of D15 (Format Freeze Policy) it belongs to, whether it is authoritative or derived state, and whether it withdraws from freeze |
| [term-renames.md](term-renames.md) | Term rename mapping table: the sole authoritative mapping between old and new names; use new names everywhere else |
| [translation-glossary.md](translation-glossary.md) | Translation glossary: Chinese, English, and Japanese side-by-side; trilingual versions of the root README, `briefs/` and `.claude/kb/` follow its terms |

## Tokens that are not identifiers

The tokens in the `doc-lint:not-numbers` line share the shape of identifiers (uppercase letter + numbers),
but they are domain terms, not identifiers of this project, and therefore do not require short names. Add new similar terms to that line; do not modify the gate.
`F41` is similarly the last four hex digits truncated from the Castagnoli polynomial `0x1EDC6F41` of CRC-32C (settled item 17 of D18 (Information Carried in a Block) hardcodes this parameter).

<!-- doc-lint:not-numbers L1 L2 L3 AES-256 AES256 SHA256 SHA-256 SHA512 CRC32 CRC32C CRC64 RAID5 RAID6 OCFS2 Z3 T10 M1 M2 M3 M4 M5 M6 M7 M8 M9 M12 M13 M25 M26 F41 -->

`M` followed by numbers (`M1`–`M9`, `M12`, `M13`, `M25`, `M26` in the `doc-lint:not-numbers` line) are **row names in the mutation table**, not identifiers of this project. Their registry location is
`scripts/mutation/tables/<bin>.tsv`—the first column of each table, with a one-to-one correspondence to that binary.
When referencing them, include the associated binary (for example, "`M8_退回第一版哈希` in `e39_back_chain`"),
because **the same `M8` refers to different things in different tables**.
