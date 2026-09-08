# C163: the tie-breaker that cannot break ties

From-scratch COW filesystem, format design phase, no code yet. Answer in English.
Do not use any markdown emphasis in your answer.

## The contradiction, verbatim from settled clauses

Every line below was checked against the repository today by the main agent.

1. Invariant I-1.8 gives the total-order key for class code 3 (packed record container)
   as (birth txg, instance id, transaction number), and states verbatim why the
   transaction number is there:

   "the transaction number stays in the last position as the tie-breaker within one
   checkpoint (the fixed point iterates, a container is not guaranteed to be written
   only once per checkpoint)"

2. D18 settled item 7 gives the value rule for that same field, verbatim:

   "a container is written out only at the checkpoint fixed point; the transaction number
   takes the largest transaction number already allocated in that checkpoint"

3. D23 settled item 7, verbatim: the transaction number "is counted per instance, starts
   at 1, all records of one transaction share it, and it is a different counter from jsn".

4. D3 settled item 5 and experiment E81 define the fixed point: every structure that
   records allocation is itself a COW tree, so writing a node allocates blocks, and
   allocating blocks writes the allocation-record tree, which allocates more blocks.
   E81 measured it: with scattered allocation one 12-unit fsync blows up to 5909 metadata
   blocks over 65 rounds; with clustered metadata the chain breaks after 2 rounds and
   28 blocks. D3 settled item 5 already settled that commit-internal blocks are allocated
   from a clustered segment, so the settled configuration is the 2-round one.
   E81's model assumes "one COW pass at the end of publication"; it says an
   allocate-as-you-go implementation is more expensive.

5. Owed check C83, still unpaid: the convergence and round bound of that fixed point have
   zero coverage anywhere in the repository.

## Why this is a contradiction

The fixed point runs during the publication phase of a checkpoint. By then all user
transactions of that checkpoint have already been accumulated (D16: batch, then publish
the root). So "the largest transaction number already allocated in that checkpoint" is
already fixed before the fixed point starts iterating, and every round of the fixed point
reads the same value.

Therefore two versions of the same container written in two different rounds of one
checkpoint's fixed point carry byte-identical write sequences: same birth txg (same
checkpoint), same instance id (same instance), same transaction number. The total-order
key of I-1.8 cannot separate them, which is exactly what clause 1 says it is for.

The same field exists on class code 2 (index node) with the same value rule, where it has
no consumer at all. That question is a separate open item (D18 open item 12) and is
blocked on this one.

## What is NOT yet established, and is the reason you are being asked

Whether the fixed point iterating in N rounds actually means a given container is written
to disk N times. If the fixed point converges in memory and all nodes are written once at
the end (E81's own modelling assumption), then no container is ever written twice within
one checkpoint, the premise quoted in clause 1 is false, and the tie-breaker is not needed
by anyone. Nobody in the repository has written down which of these the implementation
must do. C83 records that the fixed point has zero coverage.

## The four arms

A. Change the value rule so the transaction number on code 2 and code 3 becomes the fixed
   point round ordinal within the checkpoint. Zero new bytes; changes the on-disk meaning
   of an already settled field, so it changes C113 ruling P1 and P3.

B. Keep the value rule and find a different tie-breaker for the same-checkpoint case.
   Any new discriminator has to live somewhere, so this probably costs bytes in a field
   table that is frozen in layer 1 of the format freeze.

C. Make it a write-path discipline: the fixed point converges in memory and each container
   is written out at most once per checkpoint. The premise in clause 1 then becomes false
   by construction and no tie-breaker is needed. Zero format cost, but it constrains the
   implementation, and it needs a check that can actually fail, otherwise it is just a
   sentence. Note the project rule: a pitfall written as a reminder sentence rather than
   as a failing check is explicitly forbidden.

D. Keep everything and accept that within one checkpoint multiple readable versions of one
   container cannot be told apart. Say what actually goes wrong on disk if a rebuild picks
   the wrong one, and whether anything else already prevents that.

## What the published predicate does with these intermediate versions

Invariant I-1.2: for code 3 the published predicate is b <= T_pub. An intermediate version
from a fixed point round has b equal to that checkpoint's txg, so if the checkpoint
published successfully, every intermediate version is judged published and enters the
candidate set. They are not filtered out before the total-order key is applied.

## Your assignment: hunt for counterexamples

Do not pick an arm. Find concrete scenarios, built from the settled clauses, where an arm
produces a wrong outcome on disk. For each surface below say whether you found one.

1. Is the contradiction real at all? Construct a concrete sequence in which two versions
   of the same code 3 container, both readable, both judged published, both from the same
   checkpoint, exist on disk. Name every field of both headers. If you cannot construct
   such a sequence, the tie-breaker was never needed and arm D wins by default.

2. Arm C. If the fixed point must converge in memory before anything is written, what
   breaks? Consider that a checkpoint can carry up to 2 GiB of dirty data, that the
   allocation itself must be decided before writing, and that a crash mid-publication
   must still leave a recoverable state.

3. Arm A. If the transaction number becomes a fixed point round ordinal on code 2 and
   code 3, but keeps its old meaning on code 1, one field carries two meanings depending
   on class. Find a path where a reader that does not know the class misreads it.

4. The published predicate. Intermediate versions from earlier fixed point rounds are
   garbage that was never referenced by the final root. Is there any settled clause that
   already prevents them from being reachable by a scan rebuild, making the whole
   tie-breaker question moot?

For every counterexample give the exact event sequence and name the violated clause.
If you cannot construct one, say so plainly rather than inventing one.
