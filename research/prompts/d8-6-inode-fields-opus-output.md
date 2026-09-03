# D8 未定项 6（inode 字段表）·反推攻击腿（opus）原样输出（2026-09-03）

## VERDICT

**F13 does not survive as written.** Its shape (a) and its no-extent-pointers call (c) are defensible and I concede both. But it contains one break that is fatal at format-freeze time, and two more that make its own stated justifications false. I read no repository; everything below is from F1–F13 as given, with inferences marked.

---

## STRONGEST BREAK — `locality_id` becomes the only key field on disk with no second witness, and F13 makes that permanent

**Setup, from settled text only:**
- F1: the data key is `(locality_id, inode, offset)` — `locality_id` is the **leading** component.
- F1: it is inherited from the parent at creation and **not updated on rename**; the dirent value is `inode`.
- F5: every data unit's plaintext identity five-tuple is (unit type tag, tree id, object id, birth generation, anchor offset). **`locality_id` is not in it.**
- F13(f): the inode record does not hold it either.

⇒ `locality_id` exists in exactly **one** place on disk: the key bytes inside index nodes. Every other identity component of a data unit is deliberately doubly witnessed (key + F5 header) so a rebuild can cross-check. The single field that orders the entire data keyspace is the one field with no witness.

**Three reachable consequences:**

1. **Lookup after rename.** Open `b/f` after `rename(a/f, b/f)`. The dirent gives you inode `N` (F1). The data lives at `(L_a, N, *)` where `L_a` was `a`'s locality at creation — and F1 says it was deliberately not updated. Nothing the reader now holds yields `L_a`. There is no point lookup; you would have to scan. F13(f) removes the only natural home for it.
2. **Rebuild.** From data-unit headers you can reconstruct `(object id, anchor offset)` — i.e. `(inode, offset)` — but you must **invent** the leading component to re-key anything. Lose an index node and the affected records are unplaceable, with no cross-check available anywhere on disk.
3. **The first transaction, today.** v1 has no directories. F1 sources `locality_id` from the parent at creation. With no parent, the first transaction **cannot construct its data key at all** without a settled default, and F13 neither holds the field nor names the default.

**F13(f)'s justification misreads F6.** F6 says the value is a hint that "may be wrong, may be stale" — that licenses *distrusting its meaning*, not *discarding its bits*. As an address component the value is exact and definitional: after creation the data simply lives under that number, whatever the number now means about locality. A stale hint that is also a key prefix must be persisted exactly. F13 converts "don't trust it" into "don't store it," which does not follow, and F11's warning is precisely about a conceptual conflation propagating.

---

## SECOND BREAK — F13(e) claims a full POSIX field set on day 1; it is not one, and the two omissions consume the entire reserve

- **`st_rdev` is absent.** A device node cannot be represented. There is no side channel, because F13(f) defers xattrs. This is not "later work" — under F12 it is a field-set hole arriving at the freeze.
- **`st_blocks` is absent, and this is a recoverability break, not a convenience.** With no extent pointers (c) and no block count, the record carries **no stored expectation** of how much data should exist. `size = 4 MiB` with one data unit found is *legitimately sparse* and *catastrophically lost* — the checker cannot tell them apart, because "sum of units found" has no independent counterpart to compare against. That is the checker's whole method, removed for this one relation.
- **Arithmetic:** 8 + 8 = 16 = the whole of F13(d)'s reserve, spent by two fields the proposal forgot while claiming completeness. F13's only other stated extension route (xattrs) is explicitly deferred. ⇒ **F13 reaches the freeze with no growth mechanism at all.** This is an evolvability argument, admissible; "make it smaller" is not the claim.
- (Flagged as outside the given facts, so a pointer only, not evidence: fscrypt-class designs need a per-file key/nonce per inode. If that lands here, it is a third draw on a reserve already at zero, and F7/F12's day-1-reservation discipline is exactly what F13(e) invokes for POSIX while skipping for crypto.)

---

## THIRD BREAK — F13(b) answers the one identity question that was never ambiguous, and neither of the two that are

- **It cannot order two versions.** F5's generation is *birth* generation — constant over the object's life. Two records for inode 42 found by scan carry the same birth gen. F13(d)'s timestamps are wall clock and non-monotone (NTP step, `settimeofday`, VM clock jump), so they cannot break the tie either. F10(a)'s "which generation am I" is answered by a **homonym**. Note the asymmetry: pointers are self-dating (F8, birth txg inline) and the inode record is not.
- **It cannot name its head.** Under F9, the discriminator lives in the key's low bits. F13(b) copies the inode number — which was never in doubt — and not the discriminator. So in the exact scan-rebuild scenario used to justify it, it fails.
- **Fork on F10(a)'s granularity.** F10(a) says "metadata *block*". Either it is satisfied at node granularity (then the containing node's header already supplies id/generation context and F13(b) is not the thing F10(a) demands), or at record granularity (then F13 is short a tree id — which F5's settled five-tuple *does* carry — plus the head discriminator). F13 does not say which, and metadata-node header contents are not in evidence. That gap is itself a freeze blocker under F12.
- **It cites an assumption as settled.** F5 states the widths are unsettled and E85's 8 bytes is explicitly an assumption. F13(b) matches them "matching F5's five-tuple widths," freezing the inode side of a cross-check whose other side is open. Freeze-order violation under F12.
- **Missing companion clause.** Any duplicated field needs a stated authority and a stated behaviour on mismatch (key says 42, record says 43 → repair? EIO? trust which?). Without it, redundancy converts key corruption into a silently ignored discrepancy, and the kernel path and the checker will diverge on it.

---

## (a) through (f)

**(a) Repeating id + generation.** Keep it — F10(b) runs in favour, and F4 is *not* a precedent against redundancy: F4 spends bytes for fixed width and reserved bits, so it argues the same direction. Omitting it costs the checker its only key-vs-payload cross-check for the inode tree, which is worth keeping. But as specified it is insufficient (BREAK 3): add tree id and the head discriminator, define whether "generation" is birth generation (F5) or a reuse counter — different mutation rules, different cross-checks — and write the mismatch-authority rule.

**(b) No extent pointers.** **Concede, conditional on BREAK 1.** No settled clause forbids a "first N extents" shortcut — F3 governs *data* inlining, not pointers — but F4's stated cost (one more parser + checker + crash-point replay, forever) applies squarely to a second representation of the same relation, and that argument is admissible where "it's faster" is not. F13 also fails to make the best argument for its own position: with no forward references in the record, a torn or missing data unit is a data-plane loss, never a dangling metadata pointer.

**(c) Fixed + reserved.** Shape right, reserve wrong. The "inodes are fewer than pointers" counter is inadmissible (space). The admissible counter is that pointers earned fixed width because their attribute set is **closed**, while an inode's is **open by construction** (xattrs, ACLs, project/quota id, crypto policy, future statx fields). What F13 needs before freeze is not a bigger reserve but a **settled overflow route** — the cheapest one consistent with settled text is to make the deferred attribute keyspace the sole extension mechanism, with a presence bit in `flags` whose semantics are defined now. Absent that bit, a reader that ignores an ACL applies mode bits only: silent permission escalation, a wrong-answer failure mode rather than a loud one.

**(d) Timestamps.** Widths survive on correctness: 8-byte seconds clears 2038, and 30 bits suffice for nanoseconds. Two findings: no monotone change counter (BREAK 3), and 12-byte timestamps place two of the four seconds fields at record offsets 52 and 76 (INFERENCE, from F13(d)'s stated order), i.e. unaligned — a parser hazard that pushes you toward `packed`-struct patterns. Cost only, fixable by reorder or pad.

**(e) F9.** **Must be decided now.** Two couplings: the record's inode field has to be defined as bare or composite (if snapshot bits are taken from the low bits, the effective inode-number space shrinks and the split becomes format-level), and the head discriminator must appear in the record or F13(b)'s justification collapses. F9 also says that low-bit space is contested with the encryption nonce carrier — so settling F13 first allocates contested key space **by side effect**, which is exactly the silent conceptual-model propagation F11 exists to prevent.

**(f) Omitted fields the design actually needs.** `st_rdev`; `st_blocks`; a `locality_id` source and v1 default; defined `flags` semantics including an attributes-present bit; a definition of "generation"; and — first-transaction reachable — **a defined `nlink` for the nameless v1 file**. With no dirent tree in v1, `nlink = 1` violates any "nlink equals dirent count" invariant on the very first image, and `nlink = 0` invites an orphan-reclaim pass to delete the only file in it. Both horns bite in the first transaction.

---

**reachable breaks:**
1. `locality_id` singly witnessed; unreachable after rename; undefined for the first transaction (F1/F5/F6/F13(f))
2. `st_blocks` absent ⇒ sparse vs. lost is undecidable by the checker
3. `st_rdev` absent ⇒ device nodes unrepresentable, no side channel while xattrs are deferred
4. reserve fully consumed by 2 + 3 ⇒ no extension path at freeze (F12)
5. record cannot order two versions of one inode: birth generation constant, wall clock non-monotone
6. under F9 the record cannot name its head — F13(b) fails inside its own justifying scenario
7. no authority or mismatch rule for the duplicated identity
8. identity widths cite E85's explicit assumption (via F5) as settled — freeze-order violation
9. v1 `nlink` undefined for a nameless file; both values break something
10. "generation" undefined: birth (F5) vs. reuse counter

**costs only:**
- 12-byte timestamps misalign two of four seconds fields — reorder or pad
- no quota/project id — routes to the same overflow mechanism as xattrs
- symlink targets become data units under F3 — consistent, just a lookup path worth noting
- the 112-byte total itself: inadmissible as a criticism per F10

**Cheapest check that would falsify me:** read the full settled text of **D8 item 3** for two things — the dirent tree's **value** (is it `inode`, or `(locality_id, inode)`?) and the stated procedure by which a reader obtains `locality_id` at open time. If the value carries it, or if locality is encoded in the inode number's high bits, BREAK 1's lookup horn dies immediately (and in the high-bits case F13(f)'s "not in the inode" is false by construction, leaving the bit split as the unsettled format-level item). Second-cheapest, for the surviving rebuild horn: **D18 item 3's** exact five-tuple field list, to confirm `locality_id` really is absent from the data-unit header.
