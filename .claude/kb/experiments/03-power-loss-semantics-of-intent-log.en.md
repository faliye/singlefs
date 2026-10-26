## E3 Power-loss Semantics of Intent Log —— Unrun (Transaction layer and checker already exist, experiment unrun)

[中文](03-意图日志的断电语义.md) · **English** · [日本語](03-インテントログの停電セマンティクス.ja.md)

**Background**: Unbounded operations follow "write intent → batching → resumable",
whose correctness hinges entirely on "resumption is idempotent" and "intermediate states are all valid".
See the three-state power-loss table in [decisions.md](../decisions.md) D8 (Core Index Structure) settled item 5, corresponding to invariants I-4.4 (intent only describes target state) / I-4.6 (remaining invariants hold in intermediate state) / I-4.7 (in-deletion inode is accounted).
**"Resumption is idempotent" is not in the invariant checklist**—it requires comparing two images and cannot be determined on a single image;
criterion ③ of E3 (Power-loss Semantics of Intent Log) is the sole authoritative expression of this property.

**Method**: Issue unbounded operations (truncate large files, delete large directories) against an image containing large files/large directories,
use block-layer write records to truncate, replay, and run checker at **every** possible crash point,
then resume to completion and run checker once more.

**Criteria**: At all crash points—
① checker reports no errors; ② target state is reached after resumption; ③ resuming twice on the same intent yields identical results.

**What Counts as Failure**: If all crash points "happen" to fall between batches,
it indicates that truncation granularity is too coarse and **in-batch power loss was not genuinely tested**—the entire round is invalidated, and granularity must be reduced to individual write requests.
This is where E3 (Power-loss Semantics of Intent Log) is most prone to deceiving itself.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 5 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: this round only synchronized bracketed notes with index rows; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: only phrasing updated, conclusions, numbers, and artifacts unchanged |

## Historical Versions

E3 (Power-loss Semantics of Intent Log) history entries are consolidated in [experiments-history.md](../experiments-history.md).
