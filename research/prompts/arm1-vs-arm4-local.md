You are reviewing two competing proposals for a copy-on-write filesystem being built from
scratch in Rust. Your assigned stance: find counterexamples. Attack proposal FOUR in
particular, because it claims to add zero new parameters and leave zero fake candidates.
Do not argue either proposal is good.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F14 and were verified against the repository by the requester.
ONE and FOUR are proposals, not facts.

--- BACKGROUND START ---
F1. A new root record is written into a ring on every publish. The ring has R regions and S
    slots per region; its depth N is R times S. R is settled at 3. S lives in the superblock,
    checked against a range at mount time; the repository says the value of S is explicitly
    not a format decision. Region positions are laid down at mkfs time.

F2. An invariant I-7.4 requires that blocks referenced by the most recent K valid roots have
    not been reallocated to another object nor wiped by the sweeper. Today K is a runtime
    policy with lower bound 2 and upper bound N.

F3. Note that I-7.4 names TWO distinct ways a block can be lost: reallocation by the
    allocator, and wiping by the sweeper. They are different mechanisms.

F4. A settled user decision lets an administrator roll back to any root in the ring that the
    instance table judges valid; rollback depth is at most the ring depth. Its candidate test
    filters abandoned timelines only and says nothing about whether blocks survive.

F5. Because K may be 2 while N is at least 3, older roots are what the repository calls fake
    rollback candidates. An arithmetic experiment found 21 of 22 scanned (N, K) pairs have
    fake candidates; only K equal N has none.

F6. A separate settled user decision fixes the sweeper admission test. Verbatim: the conjunct
    common to both branches is "not in the allocator in-flight overlay" and "header readable";
    entries additionally require "already freed" and "free generation at most the checkpoint
    generation of the OLDEST root in the ring". This test contains no K at all.

F7. The stated reason for F6 is that at that point no root in the ring still references the
    block.

F8. Freeing is defined as the moment the allocator puts the location into the deferred-free
    queue, which is when no root, including snapshots, references it.

F9. After a crash the in-memory deferred-free queue is lost, and the gate in F6 takes over.

F10. Allocation records form a copy-on-write tree under the root. On free, the entry is not
     deleted; it is rewritten as "freed plus free generation", and the entry stays until that
     location is reallocated and overwritten.

F11. A settled decision keeps the most recent K generations of accounting, choosing the
     conservative reading N because any surviving slot may be selected; its text says this
     item follows N and is no longer independent.

F12. There is an explicit warning in the repository that K must remain a runtime policy with
     lower bound 2 and must not become a format constant, because welding down a pure
     availability parameter buys nothing. The repository also states, a few lines below, that
     the conclusion stands but the evidence behind it has collapsed, and that the replacement
     reasoning has only been through one of three required review legs.

F13. Publishing happens on every fsync, measured at 2785 per second.

F14. Every invariant here is unimplemented; there is no filesystem code yet.

--- PROPOSALS ---
ONE. K is identically the ring depth N. The invariant text changes from "K is a runtime policy
     with lower bound 2" to "K equals the ring depth".

FOUR. Add no new parameter. Derive K from the sweeper predicate in F6, and add to the rollback
      candidate test of F4 one extra conjunct that is decidable from on-disk state. Claimed:
      zero new free parameters, zero fake candidates.

--- BACKGROUND END ---

Answer these five questions.

1. FOUR claims zero fake candidates. Attack that claim. Consider F3 carefully: the sweeper
   predicate in F6 governs only wiping. Who governs REALLOCATION, and can a block be
   reallocated while the sweeper predicate would still forbid wiping it? Construct the
   sequence if so.

2. FOUR needs an extra conjunct decidable from on-disk state. Using only F10, state what that
   conjunct would have to read, and construct a case where the on-disk state is insufficient
   to decide it. If you think it is sufficient, say so and show why.

3. FOUR claims zero new free parameters. Check that. Does deriving K from the F6 predicate
   leave anything tunable, and if not, is FOUR then subject to the same objection as ONE
   under F12?

4. Under ONE, K becomes N which is R times S. Give a concrete operational scenario where an
   administrator today could change K but under ONE could not, and say what it costs. If you
   cannot construct one, say so.

5. Which of ONE and FOUR is more likely to fail silently, meaning produce a wrong result that
   no check reports? Give the mechanism, not a preference.
