# D8 未定项 6（inode 字段表）·正推腿（sonnet）原样输出（2026-09-03）

## Verdict: survives-with-amendment

F13 is largely a direct, consistent application of the settled clauses, with one already-existing precedent (F5) that makes it stronger than the background's own framing suggests — but it also imports an unsettled number as if it were fixed, understates a real dependency on F9, and its field list doesn't match its own "full POSIX" claim.

### (a) Repeating inode number + generation

Survives, and the justification is stronger than F10(a) alone. F5 already establishes this exact pattern for data units: the identity section (unit type tag, tree id, OBJECT ID, OBJECT BIRTH GENERATION, anchor offset) repeats OBJECT ID/generation inside the unit even though F1's data keyspace already keys data blocks by `(locality_id, inode, offset)`. That is the *same* redundancy F13(b) proposes, already crossed and settled at the data layer. So F13(b) isn't introducing a new kind of redundancy — it's extending an existing one to the metadata layer, which is the more defensible position.

Two real gaps, though: (1) F5 explicitly flags the 8-byte widths as an *unsettled* experiment assumption (E85), so F13(d) locking in "8 bytes each, matching F5" launders an assumption into a settled fact — not permitted as written. (2) the argument implicitly assumes the inode record itself is located by a key containing the inode number (analogous to F1's pattern) — no given F-fact actually states what key locates an inode record. That's a silent prerequisite this proposal should surface, not absorb.

Concrete failure if omitted: a detached/orphaned index node found during scan-rebuild (no parent pointing to it) can't say whose record it holds; and if "generation" here means inode-number-reuse detection (distinct from F5's birth-generation and from any node-level COW generation), omitting it means a reused inode number can alias stale external references with no way to detect it.

### (b) No extent pointers in the inode

Consistent with F1 + F3, no violation. F1 already commits to one addressing path (the data keyspace) for reaching file data; F3's inline threshold of 0 forecloses embedding data-adjacent shortcuts in the record. The cost — every read, even of a 1-block file, needs a second tree descent into the data keyspace — is real but not a correctness problem, and per F4's "faster is not a valid branch justification" it can't be argued away without measurement. The reserved bytes in F13(d) already leave room to add a *non-authoritative* first-extent hint later, following F6's precedent (locality_id: a hint, never load-bearing) — that's the right escape hatch if this ever needs revisiting, not a redesign now.

### (c) Fixed-length + reserved shape

Transfers, but conditionally. F4's reasoning ("one more parser + checker + crash-point replay forever") is an argument against a *second format branch*, not against fixed width per se — and it only holds here because F13(f) pushes the genuinely unbounded/variable-growth data (xattrs, ACLs) into a separate, not-yet-designed keyspace. If that externalization holds, the inode record itself is a bounded, slow-growing POSIX-ish field set, and fixed+reserved is the right shape (matches the incompat-bit escape hatch: reserved bytes get a meaning later without changing record length).

But F13(f) itself says that keyspace is "not yet designed" — so the premise this whole argument rests on is itself open. And 16 reserved bytes is a bare number with no stated basis; F10(b)'s "prefer bigger, default is do-not-drop" argues for more headroom here, not less, especially with two open dependents (F9, and the eventual xattr/ACL pointer-out mechanism) both plausibly wanting bits from the same pool.

### (d) Timestamp widths

Survive as written, no amendment needed. 8-byte seconds is not indulgence — it's a correctness argument (avoids a Y2038/Y2106-style wraparound decades out), which is exactly the kind of self-containment F10(b) licenses. 4-byte nanoseconds tightly covers 0..999,999,999 with no waste either direction. Nothing in the given facts gives grounds to shrink or grow these on a correctness basis.

### (e) F9 interaction

Not blocking for the record's *value* layout, but blocking for freeze. F13(d)'s field list is about payload contents; F9 (snapshot id candidate in low bits of the *key*) is about addressing. They can be decided on separate timelines — as long as "inode_number" in F13(d) is defined as the logical inode number, independent of however the key chooses to route/shard it. But F12's freeze precondition requires every format-level item settled first, and key layout is format-level, so F12 cannot close while F9 sits open — this must be tracked as an explicit dependency, not silently deferred. Given the uncertainty, keep the reserved-byte budget generous rather than fixing it now, since resolving F9 could still create a need for a value-side snapshot tag.

### (f) Missing fields for the first transaction

F13(d) omits an allocated-block-count field (POSIX `st_blocks` equivalent) — distinct from `size`, needed once anything beyond "one small fully-allocated file" exists, and arguably needed now for consistency with the project's general space-accounting discipline (incrementally maintained at commit time, not swept later). More importantly, F13(e) claims "full POSIX field set on day 1" but the actual list in F13(d) is missing `st_blocks`, `st_rdev`, `st_dev`, `st_blksize` — that claim and that list contradict each other and must be reconciled one way (add the fields) or the other (name the claim down to "core POSIX subset" and enumerate exclusions with reasons).

## Required amendments before this is settled

1. Add `blocks: u64` (allocated block count) to the field list, or strike "full POSIX field set" from F13(e) and replace with an explicit, justified exclusion list.
2. Do not inherit "8 bytes each" for `inode_number`/`generation` from F5/E85 silently — either justify independently or mark explicitly "provisional pending D18, numerically pinned to F5's OBJECT ID / OBJECT BIRTH GENERATION widths."
3. Rename the repeated generation field to something that states its purpose (e.g. `inode_generation`, reuse-detection counter) and state explicitly that it is *not* the same quantity as F5's OBJECT BIRTH GENERATION or any node-level COW generation.
4. Add, as a companion decision (not left implicit), what key locates the inode record itself — F13(b)'s redundancy argument depends on it and no given fact states it.
5. Increase `reserved` from 16 to at least 32 bytes, citing F10(b)'s generosity default plus the two named open dependents (F9, xattr/ACL pointer-out) — not a performance justification.
6. Record explicitly that F13's freeze is gated on F9 (D6) being at minimum resolved to "out of v1, N bits reserved in key," per F12.
7. Pin timestamp encoding: signed 64-bit seconds since Unix epoch, unsigned 32-bit nanoseconds (0..999,999,999), `ctime` = POSIX metadata-change semantics.
