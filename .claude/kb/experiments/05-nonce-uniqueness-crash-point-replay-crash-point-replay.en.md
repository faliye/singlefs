## E5 Nonce Uniqueness × Crash-Point Replay —— Unrun (Transaction layer and checker already exist, encryption implementation still owed)

[中文](05-nonce唯一性×崩溃点重放.md) · **English** · [日本語](05-nonce一意性-クラッシュポイントリプレイ-クラッシュポイント再生.ja.md)

**Background**: [invariants/08-加密不变量.md](../../../crates/singlefs-checker/invariants/08-加密不变量.md) I-6.1 (nonce not reused) and I-6.5 (nonce watermark monotonic).
Nonce reuse under stream ciphers like ChaCha20 **directly leaks plaintext**, while generating no read/write errors
and triggering no checksum failures—if not actively inspected, it will never be discovered.
The danger window is after a crash: data is written to disk with a certain nonce before its pointer is inserted into the tree;
therefore, after an unclean shutdown, there exist nonces that were "used but not findable in the tree"
(similar issues in bcachefs are documented in [prior-art.md](../prior-art.md) 6.3).

**Method**: Run random write + overwrite + truncation workloads against an encrypted image, with the block layer recording all write requests;
truncate, replay, and recover at **every** crash point, and continue writing a segment of new data after recovery;
collect the (key identifier, nonce) pairs for each block throughout the entire run.

**Criteria** (Must meet all three to pass, all passed across N ≥ 5 rounds):

1. Across all crash points, no duplicate pairs.
2. Newly allocated nonces after recovery are strictly greater than the recorded watermark.
3. Those nonces written to disk before the crash but not yet inserted into the tree **genuinely appear in the collection results**—
   otherwise "no duplicates" in Item 1 proves nothing.

**What Counts as Failure**: If this check **still passes all green** after turning off the watermark mechanism,
it indicates that the scan never saw the nonces written out before the crash, **the check lacks discriminative power, and the entire round is invalidated**.
This step must be performed first: make it red first, then make it green.

**Prerequisites**: Transaction layer, encryption implementation, block layer write recording, checker's I-6.1 (nonce not reused) / I-6.5 (nonce watermark monotonic) checks.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D9 (Encryption) settled item 4 | Preparation | 2026-10-17 unaffected: this round only synchronized bracketed notes with index rows; conclusions, numbers, and artifacts unchanged; previously 2026-10-13 unaffected: renaming only touched wording, conclusions, numbers, and artifacts unchanged |

## Historical Versions

E5 (Nonce Uniqueness × Crash-Point Replay) history entries are consolidated in [experiments-history.md](../experiments-history.md).
