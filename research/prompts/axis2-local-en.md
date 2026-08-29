# Background: singlefs — should ancestor nodes be written at fsync, or deferred?

singlefs is a copy-on-write filesystem being designed from scratch (Rust, format-design
phase, no code yet). Everything below is either an already-settled decision of this
project or a measurement from experiment E16.

## Settled (do NOT treat these as open)

| # | Rule |
|---|---|
| C1 | A journal record carries **pointer-layer target state**: which **subtree roots** moved to which blocks, plus the new root. |
| C2 | **Allocation precedes the journal.** A record is a *publish instruction for blocks already on disk*. |
| C3 | A logical-intent journal was rejected for two reasons: (a) the independent spec executor "O2" is forbidden from calling any code of the implementation under test, and logical replay would require reimplementing the allocator, tree operations, checksums and accounting; (b) **allocation decisions are not in the record, so the implementation and O2 would place the same intent at different physical blocks**, degrading O2 from "independently compute the same answer and compare bytes" to "run an invariant check", which the checker already does. |
| C4 | Publication semantics are checkpoint-based. The root ring is a fixed set of K slots outside the journal; publishing a root overwrites one slot. |
| C7 | **Already decided: every fsync publishes a root.** Not publishing means the journal ring can never be truncated (measured peak 48 B vs 10^4–10^5 B), forces a mount-time replay path, and pulls the journal into the validation chain. Barrier count is 2 either way, so not publishing saves no barriers. |
| C8 | Checksums are inlined into the **parent pointer**, forming a Merkle tree. |
| C9 | This project has **no oracle**: designing from scratch means there is no reference implementation to compare against. Functional correctness rests on model-based comparison and on O2. |
| C10 | Project rule: *be stingy with format branches*. The test for a format branch is **"does it turn 'used it wrong' into 'cannot mount'"**, and the test explicitly **forbids "it is faster"** as a justification. Also: *when you cannot write down the criterion, implement only one path, and implement the slow correct one first.* |

## The three candidates for this axis

| Candidate | What fsync writes | Does replay need to allocate? | Can O2 compare bytes? |
|---|---|---|---|
| **A: no deferral** | dirty leaves + all ancestors + root slot + record | No | Yes |
| **B: defer, record names only the leaves** | dirty leaves + record | **Yes** | **No** |
| **C: defer, but the record names the physical locations and checksums of the whole ancestor chain** (unevaluated) | dirty leaves + a larger record | No (locations are in the record) | Possibly |

## Measurements (E16, 200,000 operations, checkpoint interval 1000; units are BLOCKS, not time)

| Workload | fsync | A write-amp | B write-amp | A blocks per fsync | B blocks per fsync |
|---|---|---|---|---|---|
| random small writes | every op | 7.000 | **4.101** | 7 | **2** |
| metadata-heavy | every op | 4.968 | **2.149** | 10 | **3** |
| random small writes | every 10 ops | 4.265 | **3.201** | 43 | **11** |
| sequential | every op | 1.750 | **1.134** | 14 | **9** |
| any | **no fsync** | **equal** | **equal** | 0 | 0 |

The last row matters most: **with no fsync, B's marginal benefit is exactly zero.**
What B buys is amortising the spine of ancestor writes up to the root — and group commit
buys the same thing. So they are substitutes, not complements. B's 1.3–2.3x therefore
exists only where group commit cannot batch: **single-stream synchronous writes**.

## Two costs that B pays

1. C3(b) reappears unchanged: B's replay output is *ancestors + root*, and **100% of that
   output has undetermined placement**. The part that could be compared (the leaves) is
   exactly the part that was already on disk and that replay never touched.
2. The format gains a second place where checksums live: what a B record names has no
   parent on disk yet, so **the record must temporarily act as the parent**, growing each
   entry from 24 to 56 bytes. C8's shape is "checksums live in exactly one place".

## The claim to attack

> **Claim S**: take candidate **A** (no deferral). Not because it is faster — it is slower —
> but for three reasons: (1) B sells off exactly what this project is shortest of (C9: no
> oracle, and O2 is one of the few we can build ourselves); (2) under C10's format-branch
> test, B does not turn "used it wrong" into "cannot mount", and "it is faster" is not an
> admissible justification; (3) B's benefit has a bad shape — zero without fsync, and a
> substitute for group commit.
> **Plus: reserve a type field in the record header**, so that if experiment E22 finds
> candidate C viable, this axis can be reopened without breaking the format.

---

# Your role: **find counterexamples**

Do not argue for Claim S. Do not review the experiment's modelling. Your only job is to
attack. Produce the following, in this order:

1. **Three concrete workloads on which choosing A instead of B is a serious mistake.**
   For each: what the workload does, why group commit cannot batch it, and roughly how
   much A costs versus B. Be specific about the mechanism. If you can only think of one
   or two, say so — **do not invent a third to fill the list.**

2. **Attack reason (1) directly.** Is "B destroys O2's byte comparison" actually true?
   Try to construct a way for O2 to still compare bytes under candidate B. If you cannot,
   say plainly that you cannot, and state what would have to change in the format to make
   it possible.

3. **Attack the "substitutes, not complements" claim.** Find a situation where group
   commit is fully available *and* B still wins. Give the mechanism, not an assertion.

4. **Attack the reserved type field.** Describe a concrete path where reserving one type
   field in the record header turns out to be insufficient to reopen this axis later —
   i.e. the format still has to break.

5. Finally: **which single sentence of Claim S is most likely to be wrong, and what
   observation would show it?**

Write in English. Be structured and direct. No preamble, no restating the background.
