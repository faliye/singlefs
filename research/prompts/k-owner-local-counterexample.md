You are reviewing a proposed clause for a copy-on-write filesystem built from scratch in Rust.
Your assigned stance: find counterexamples. Do not argue the clause is good. Try to break it.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Facts are labeled F1 to F14 and were verified against the repository by the requester.
The clause under review is labeled C. It is a proposal, not a fact.

--- BACKGROUND START ---
F1. The filesystem writes a new root record into a ring on every publish. An invariant called
    I-7.4 says the blocks referenced by the most recent K valid roots must not have been
    reallocated to another object nor wiped by the sweeper. K is a runtime policy with lower
    bound 2, and K is at most the ring depth, which is the number of regions R times the
    number of slots per region S.

F2. R is settled at 3, derived as R = F + 1 where F = 2 is a product judgement about how many
    failure domains to tolerate. S lives in the superblock; its lower bound is 1 and its upper
    bound is computed at mount time from how many blocks I-7.4 pins. The specific value of S
    is explicitly declared not to be a format decision.

F3. A publish happens on every fsync; the measured publish rate is 2785 per second.

F4. An experiment enumerated nine consumers that depend on old roots staying valid, priced
    each one, and was re-run after an enumeration gate fired. Results:
    c1 recovery root fallback: priced in generations, requires exactly 2 generations;
    c2 replay watermark: priced in generations, its requirement value is the in-flight record
       limit, a number that does not exist in the repository;
    c3 generational incremental scrub: requirement 0, because the only clause defining it says
       it maintains a watermark and only walks blocks whose birth generation exceeds the
       watermark, so it never holds an old root;
    c4 timeline discrimination: pricing blank, consumer not implemented;
    c5 the phase count of two pinning rules: defined in terms of K itself, no independent
       requirement;
    c6 an invariant that walks from any of the last K roots on a crash-replayed image:
       requirement 0, frozen image, no allocator racing it;
    c7 a record verifier taking (pre-crash image, record stream, post-crash image):
       requirement 0, all three inputs frozen;
    c8 administrator rollback: priced by external event; how far back you can go is DEFINED by
       the ring depth, so it does not demand a larger K;
    c9 traversal hop, reading a parent pointer and then the child block: priced in wall clock,
       requirement is one hop of latency.

F5. There are two DIFFERENT block-pinning rules and a prior experiment measured that they pin
    different sets. Rule one is I-7.4, whose free variable is K. Rule two pins everything
    freed inside the whole journal replay window, and its free variable is the number of
    fsyncs since the last checkpoint.

F6. The settled text says the real constraint for the replay case is the entire journal replay
    window, not the checkpoint boundary, and that a block freed at journal position p and
    reallocated at position q greater than p makes any replay prefix between them produce a
    tree pointing at an address whose bytes are already q's content.

F7. The failure mode of losing that constraint is stated as: detectable, because the parent
    pointer checksum mismatches, but not recoverable.

F8. There is a separate settled clause about accounting that also says "keep the most recent K
    generations". The repository has no clause forcing that K and I-7.4's K to be the same
    variable.

F9. Administrator rollback is a settled user decision. Its text says the rollback depth is at
    most the ring depth, and the candidate set is the roots in the ring that the instance
    table still judges valid.

F10. Every invariant in this repository is unimplemented. There is no filesystem code yet.

F11. Background activities such as scrub and compaction are explicitly allowed to be slow,
     resumable and to traverse; runtime admission decisions are not.

F12. The sweeper admission gate is phrased as: a block may be swept only if its free
     generation is at most the oldest root generation in the ring.

F13. After a crash the in-memory deferred-free queue is lost, and the runtime K is taken over
     by the ring depth, so the protection width changes.

F14. A rule in this project says a clause that no observation could refute is not a clause but
     an article of faith.

--- CLAUSE UNDER REVIEW ---
C. The value of K, and the ring depth, are determined by only two things: the 2-generation
   lower bound required by c1 recovery root fallback, and the failure domain count
   (R = F + 1). No other consumer demands a larger K.
   Void clause: if a consumer appears whose requirement for a published root to remain valid
   exceeds 2 generations AND which cannot abort and re-anchor, then C is void at once and the
   value of K must be reopened.

--- BACKGROUND END ---

Answer these five questions.

1. Find a consumer, anywhere in F1 to F14, that demands K greater than 2. Name it and give the
   mechanism. If you cannot find one, say so plainly.

2. Attack the void clause. It is a conjunction: requirement over 2 generations AND cannot abort
   and re-anchor. Construct a defence in which a consumer that plainly needs more than 2
   generations is argued to be abortable, so the void clause never fires. Does the conjunction
   make the clause unfalsifiable in practice?

3. C says K and the ring depth are determined by c1 and the failure domains. Ring depth is
   R times S. Per F2 the value of S is explicitly not a format decision and its upper bound
   comes from how many blocks I-7.4 pins. Does C silently decide S? Show the contradiction if
   there is one.

4. F8 says a different settled clause keeps the most recent K generations for accounting, and
   nothing forces the two Ks to be the same variable. Construct the concrete failure that
   results from treating them as one variable, and the concrete failure from treating them as
   two.

5. Consider F13: after a crash the runtime K is taken over by the ring depth. Does that make
   the ring depth a consumer of itself, and does it break C's claim that only c1 and failure
   domains determine it?
