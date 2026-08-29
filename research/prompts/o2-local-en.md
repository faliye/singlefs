# Background: an internal contradiction in the singlefs design notes

singlefs is a copy-on-write filesystem being designed from scratch (Rust, format-design
phase, no code yet). It has **no reference implementation to compare against**, so
functional correctness must rest on oracles the project builds itself.

## The three oracles, as defined in decision D13 (verbatim translation)

| # | Oracle | What it judges | Where its independence comes from | What it cannot catch |
|---|---|---|---|---|
| O1 | In-memory model comparison (an ideal filesystem that is just a HashMap) | POSIX-visible semantics: contents, sizes, directory structure | It never touches the disk and never parses the on-disk format | Whether the on-disk structure is self-consistent; crash semantics; **both implementations sharing the same misreading of a POSIX rule** |
| O2 | Independent format parser + checker | Whether the bytes on disk satisfy invariants I-1..I-6 | **The checker may not call any deserialization or traversal code of the crate under test**; it must reimplement parsing and read the raw device | It only judges structural self-consistency: **a block overwritten with different but still legal content** keeps the Merkle chain consistent and the accounting correct |
| O3 | Independent specification executor | Whether **each disk state produced by truncating at a crash point falls inside the set of states the specification allows** | **Written in a different language** (not Rust), not consulting the implementation's code, deriving everything afresh from the prose of the decision and invariant documents | The specification itself may be wrong — **it is meaningless without a positive control** |

## The contradiction

A later decision (D23), while rejecting a "logical intent journal" design, argued:

> (b) Logical replay must allocate new blocks, and the allocation decisions are not in
> the record. So the implementation under test and O2 would put the same intent at
> **different physical blocks**. O2 therefore degrades from "**independently compute the
> same answer and then compare**" to "**independently run an invariant check**", and the
> latter is already what the checker does, so O2's marginal value shrinks a lot.

D23 elsewhere speaks of "**O2 doing a byte-for-byte comparison**".

**But D13 says O2 *is* "an independent format parser + checker", and that it explicitly
cannot catch a block overwritten with different-but-legal content.** That is precisely
the ability a byte-for-byte comparison would have. So the two documents disagree about
what O2 is. Note also that D23's own phrase "the latter is already what the checker
does" equates the *degraded* form with the checker — while D13 says O2 *is* the checker.

Two readings, neither comfortable:

1. **If D13 is authoritative**: O2 was never a byte comparator, so argument (b) has
   nothing to degrade — it is empty — and the rejection of logical intent journals rests
   only on argument (a), the cost of writing a second implementation.
2. **If D23 is authoritative**: O2 must independently compute the same physical image,
   which means **it needs its own allocator** — and that is exactly the cost argument (a)
   called "roughly a second filesystem". So (a) and (b) cannot both hold as written.

## Settled decisions you must respect

| # | Rule |
|---|---|
| A1 | The journal form is settled: every fsync writes the dirty leaves, **all ancestor nodes**, a root slot, and one record. Ancestors are never deferred. |
| A2 | **Allocation precedes the journal.** A record is a publish instruction for blocks that are already on disk. |
| A3 | A record names **which subtree roots moved to which blocks**, plus the new root. ⚠️ **It names subtree roots, not leaves.** The leaves were already on disk before the record was written, and **the leaves' physical locations were chosen by the allocator and are not in the record.** |
| A4 | Under A1, redo is empty: after a crash the root is already the newest state. |
| A5 | Whether the transaction layer promises serial commits is **still undecided**. |

---

# Your role: **find counterexamples**

Do not summarise the contradiction back to me. Do not argue for either reading. Attack.

1. **Construct a way for an independent checker to do byte-for-byte comparison WITHOUT
   reimplementing the allocator.** Take it seriously — think about what information is
   already on disk, what is in the records, and what a checker is allowed to read.
   Give the construction step by step, and state exactly what it can and cannot compare.
   If after real effort you conclude it is impossible, say so plainly and give the
   argument for impossibility.

2. **Attack the idea that renaming the argument from O2 to O3 rescues it.** O3 judges
   membership in "the set of states the specification allows". Find a reason why moving
   the argument to O3 does *not* work. Consider at least: does a larger allowed set
   actually mean a weaker check? Can the prose of a design document contain enough
   information to compute which physical states are allowed?

3. **Attack argument (a)** — "a second implementation is roughly a second filesystem".
   Find the cheapest honest way to build the thing (a) says is too expensive. What would
   it actually have to implement, and what could it legitimately skip?

4. **Construct the case where all three oracles miss the same bug.** D13 asks for a
   "three-oracle collusion test": feed a deliberately broken implementation through all
   three; if all three pass it, there is a shared blind spot. Give one concrete broken
   implementation that all three would pass, other than the example already given
   (a nonce that is always zero).

5. Finally: **which single sentence in this whole picture is most likely to be wrong,
   and what observation would show it?**

Write in English. Be structured and direct. No preamble.
