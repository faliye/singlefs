You are reviewing a proposed clause for a copy-on-write filesystem being built from scratch in Rust.
Your assigned stance: verification by counterexample. Try to break the clause with concrete
scenarios. Do not argue that it is good.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F13 and were verified against the repository by the requester.
The clause under review is labeled E. It is a proposal, not a fact.

--- BACKGROUND START ---
F1. The filesystem writes a new root record into a ring on every publish. The ring has R
    regions and S slots per region, so its depth N is R times S. R is settled at 3. S lives in
    the superblock and is judged against a range at mount time; the repository explicitly says
    the value of S is not a format decision.

F2. An invariant called I-7.4 says the blocks referenced by the most recent K valid roots must
    not have been reallocated nor wiped. Today K is a runtime policy with a lower bound of 2
    and an upper bound of the ring depth N.

F3. A settled user decision says an administrator may roll back to any root in the ring that
    the instance table still judges valid, and that the rollback depth is at most the ring
    depth. Its candidate test filters out abandoned timelines only. It says nothing about
    whether the blocks are still there.

F4. Because K may be as small as 2 while N is at least 3, roots older than K generations are
    what the repository itself calls fake rollback candidates: self-consistent, judged valid,
    but pointing at blocks that may already be reused. An arithmetic experiment found that of
    22 scanned (N, K) pairs, 21 have fake candidates; only the pair where K equals N has none.

F5. A separate settled user decision about accounting keeps the most recent K generations of
    accounting, and chose the conservative reading: the number of ring slots N, because any
    surviving slot may be selected. Its text says this item is no longer independent, it
    follows N.

F6. The repository contains an explicit warning: K must be a runtime policy with lower bound 2
    and must not be a format constant, because turning a pure availability parameter into a
    format constant welds it down and buys nothing.

F7. A project design rule lists five hard requirements for branching. The fifth is: a branch
    variable must not change in the middle of an operation.

F8. An arithmetic experiment priced the alternatives. All four cost metrics are linear in the
    number of protected generations. Relative to protecting 2 generations, protecting D
    generations costs an extra (D-2) times 13 pinned blocks and an extra (D-2) times 64
    accounting keys. At N equal 48 that is 598 extra blocks and 2944 extra keys.

F9. The same experiment notes the pinned-block figure never reaches the previously registered
    threshold of 1000 blocks at which pinning starts to matter; at N equal 48 it is 624 blocks.

F10. Publishing happens on every fsync, measured at 2785 per second, so one generation lasts
     about 0.36 milliseconds.

F11. Every invariant in this repository is unimplemented; there is no filesystem code yet.

F12. Region positions are laid down on disk at mkfs time and are a fait accompli.

F13. A rule in this project says a clause that no observation could refute is not a clause but
     an article of faith.

--- CLAUSE UNDER REVIEW ---
E. K is identically equal to D, where D is the externally promised rollback depth, D is a
   runtime policy, and 2 <= D <= N. The settled promise changes from "rollback depth at most
   the ring depth" to "rollback depth at most D". Accounting retains D + 1 generations. The
   warning in F6 is claimed not to need changing, because D remains a runtime policy.

--- BACKGROUND END ---

Answer these five questions.

1. Construct a concrete scenario in which adopting E leads to a wrong outcome: data lost, an
   administrator rolling back to something unusable, or a corruption reported as healthy. Give
   concrete values of N, D, K and a concrete sequence of events. If you cannot construct one,
   say so plainly.

2. D can be changed at runtime. Construct the worst thing that happens if D is lowered while a
   rollback is in progress, and separately if D is raised. Consider F7.

3. Consider the case where D is left at its minimum of 2 forever. In that case what on disk
   distinguishes E from simply capping the promise at 2 generations? If nothing does, say so.

4. E claims to resolve the contradiction in F3 versus F2. Does it actually remove the
   contradiction, or does it move the contradiction into the choice of D? Explain the
   mechanism.

5. Is E falsifiable? State one concrete observation that would show E is wrong. If you cannot
   state one, say so, because per F13 that would itself be a finding.
