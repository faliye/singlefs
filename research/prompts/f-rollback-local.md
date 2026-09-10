You are reviewing a proposed narrowing of a settled clause in a copy-on-write filesystem being
built from scratch in Rust. Your assigned stance: find counterexamples. Try to show the
narrowing loses something real. Do not argue it is good.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F13 and were verified against the repository by the requester.
Proposal P is a proposal, not a fact.

--- BACKGROUND START ---
F1. A new root record is written into a ring on every fsync. The ring has R regions and S
    slots per region; depth N is R times S. R is settled at 3, S is between 1 and 16.

F2. An invariant requires that blocks referenced by the most recent K valid roots are neither
    reallocated nor wiped. K is a runtime policy with lower bound 2.

F3. A settled user decision from 2026-09-05 says an administrator may, out of band, pick an
    older root from a candidate set and roll back to it. The candidate set is every root in
    the ring that the instance table still judges valid. Rollback depth is at most the ring
    depth. The candidate test filters abandoned timelines only and says nothing about whether
    the referenced blocks still exist.

F4. Because K can be 2 while N is at least 3, roots older than K generations point at blocks
    that may already be reused. The repository calls these fake rollback candidates. Of 22
    scanned (N, K) pairs, 21 have fake candidates.

F5. A separate settled passage computes the ring's wall-clock span: span equals ring depth
    divided by the fsync rate. At the locally measured device ceiling of 2785 fsync per
    second, 48 slots span 17.2 milliseconds. That same passage concludes that the ring only
    carries "if this generation's root is damaged you can still read the previous one", and
    that checkpoint-scale block-reuse constraints are carried by a different rule.

F6. Important correction: the 2785 figure is the DEVICE CEILING measured by issuing one
    fdatasync per block, not a workload rate. The repository has never pinned any target fsync
    rate. So the ring's wall-clock span has no computable value today; it is ring depth
    divided by an unknown.

F7. An enumeration of nine consumers of old roots found that only one requires more than 2
    generations: the administrator rollback above. Ordinary crash recovery needs exactly 2
    (one generation to fall back to, plus one because verifying the newest takes time).
    Generational incremental scrub, a frozen-image checker and a record verifier all need 0.

F8. There is an explicit warning in the repository that K must remain a runtime policy with
    lower bound 2 and must not become a format constant.

F9. Nothing anywhere in the repository says which mechanism carries human-timescale rollback,
    meaning rolling the filesystem back minutes or hours. Snapshots are not documented as
    doing this.

F10. Losing a whole disk was measured on real hardware: the worst case is that you can still
     mount and fall back exactly one generation.

F11. Rolling back is defined to take effect in the same publish as the first new root, with no
     persistent effect before that; if it crashes it is simply redone.

F12. Every invariant here is unimplemented; there is no filesystem code yet.

F13. A rule in this project says a clause that no observation could refute is not a clause but
     an article of faith.

--- PROPOSAL ---
P. Narrow the rollback candidate set to the most recent K generations, with K equal to 2, and
   record the reason in the settled clause: the root ring is a millisecond-to-second scale
   carrier and does not carry human-timescale rollback.

--- BACKGROUND END ---

Answer these five questions.

1. Construct a concrete scenario where narrowing the candidate set to 2 generations loses
   something real: some actor needs to roll back 3 or more generations and can no longer do
   it. Say who that actor is and how it learned it needed to roll back. If you cannot
   construct one, say so plainly.

2. Consider F6 carefully. The ring's wall-clock span is ring depth divided by an unknown rate.
   Does an unknown denominator argue FOR narrowing the promise, or AGAINST narrowing it until
   the rate is pinned? Give the stronger case for each side, then say which is stronger.

3. Consider F7: only the administrator rollback wants more than 2 generations. Is that an
   argument that narrowing is safe, or is it circular, given that the administrator rollback
   is exactly the clause being narrowed?

4. Under P, what happens to a root sitting in slot 5 of a 48 slot ring? It is still on disk,
   still self-consistent, still judged valid by the instance table, but no longer a legal
   candidate. Construct the worst confusion this can cause for an operator or a recovery tool.

5. Is P falsifiable? State one concrete observation that would show P is wrong. If you cannot
   state one, say so, because per F13 that would itself be a finding.
