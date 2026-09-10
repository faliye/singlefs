You are reviewing a proposed scoping clause for a copy-on-write filesystem built from scratch in Rust.
Your assigned stance: find counterexamples. Do not argue the clause is good. Try to break it with
concrete scenarios.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F12 and were verified against the repository by the requester.
The clause under review is labeled A. It is a proposal, not a fact.

--- BACKGROUND START ---
F1. The filesystem keeps a ring of root records on disk. Each publish writes one new root into
    the next slot. An invariant called I-7.4 says: the blocks referenced by the most recent K
    valid roots must not have been reallocated to another object nor wiped by the sweeper.
    K is a runtime policy with a lower bound of 2, and K is at most the ring depth, which is
    the number of regions times the number of slots per region.

F2. A publish happens on every fsync. The measured publish rate on the local machine is 2785
    per second, so one generation lasts about 0.36 milliseconds.

F3. An experiment enumerated every consumer that depends on old roots staying valid, and
    priced each one. The pricing had to be one of three kinds, fixed before the run:
    generations, wall clock, or external event. Results:
    c1 recovery root fallback, priced in generations, requirement exactly 2 generations
       (one generation to fall back to, plus one more because verifying the newest root takes
       time during which the previous one must stay alive);
    c2 replay watermark, priced in generations, requirement value is the in-flight record
       limit, and that number does not exist anywhere in the repository;
    c3 generational incremental scrub or checker walking an old root, priced in wall clock,
       requirement is however long the scan takes;
    c4 timeline discrimination after a device disappears and comes back, pricing BLANK because
       the consumer is not implemented at all;
    c5 the phase count of two block-pinning rules, priced in generations, requirement defined
       in terms of K itself so it adds no independent requirement.

F4. Same experiment, measured: the K-generation window in wall clock is 1.077 milliseconds at
    K=3 and 17.2 milliseconds even at K=48 (the largest ring depth considered). The smallest
    scrub case scanned was 10 000 blocks at 100 000 blocks per second, which takes 0.1
    seconds. Every one of the 15 cells scanned (3 tree sizes by 5 values of K) exceeded the
    window. To cover a 10 second scan you would need K of about 27 850.

F5. The positive control of that experiment independently recomputed the lower bound of 2 for
    K, which matches what I-7.4 already had written down. The two derivations share no code.

F6. There is a second, different block-pinning rule: everything freed inside the journal
    replay window stays pinned until a checkpoint is persisted. A prior experiment measured
    that these two rules pin different sets of blocks, and that the root ring rule only helps
    on a small number of phases per checkpoint cycle.

F7. Every invariant in this repository, including I-7.4, is currently unimplemented. There is
    no filesystem code yet, only design documents and arithmetic experiments.

F8. The independent checker is forbidden from sharing traversal, parsing, checksum or
    accounting code with the implementation under test. It may only share one constants module
    generated from the design documents. So the checker cannot take any in-memory lock held by
    the implementation.

F9. btrfs solves the analogous problem by having scrub read the commit root, taking a rwsem
    when the commit root switches, and explicitly pausing scrub at each transaction commit.
    Its default commit interval is 30 seconds, which is about 83 000 times longer than this
    project's publish interval.

F10. ZFS solves it differently: its scrub does not pin an old root at all. It records a
     generation watermark when the scan starts and skips any block whose birth generation is
     newer than that watermark, walking the live tree instead.

F11. Background activities such as scrub, compaction and sweeping are explicitly exempt from
     this project's rule that accounting must be maintained incrementally; they are allowed to
     traverse and to be resumable. Runtime admission decisions are not.

F12. The design rule that governs experiments requires that a clause which cannot be falsified
     by any observation is not a clause but an article of faith.

--- CLAUSE UNDER REVIEW ---
A. The K of I-7.4 only serves consumers priced in generations. Consumers priced in wall clock
   may not rely on K for protection and must bring their own mechanism. The value of K is not
   influenced by wall-clock consumers.

--- BACKGROUND END ---

Answer these five questions.

1. Construct a concrete scenario in which adopting A leads to a wrong outcome: data lost, a
   corruption that goes unreported, or a consumer that silently reads reused blocks and
   reports it as damage. Give concrete numbers and a concrete sequence. If you cannot
   construct one, say so plainly.

2. A names two of the three pricing kinds. Construct a consumer priced by EXTERNAL EVENT and
   say what A does with it. Is the omission harmless or is it a hole?

3. A says the value of K is not influenced by wall-clock consumers. Attack that half
   specifically. Consider whether a wall-clock need can come back in through the OTHER
   pinning rule in F6 rather than through K.

4. Consumer c4 has BLANK pricing because it is not implemented. Which side of A does it fall
   on, and what goes wrong if that guess is later found to be the wrong side?

5. Is A falsifiable? State one concrete observation that would show A is wrong. If you cannot
   state one, say so, because per F12 that would itself be a finding.
