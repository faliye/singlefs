You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italics. Use plain words; do not invent compound words. Number your answers 1 to 6. Your stance is adversarial: assume the proposed clause is wrong and look for where it fails; if you cannot find a failure, say so plainly.

Settled facts (do not dispute them; use them):

F1. The baseline placement for user data is settled: in the first version user data blocks do not enter the clustered segments; each block goes to the lowest free slot on the device already chosen for it, skipping slots inside a region that background compaction is currently emptying. Clustered segments are reserved for blocks that the commit itself produces (tree nodes), and a settled clause says those blocks must be allocated from clustered segments.

F2. The allocator hint layer is described by three sentences that any hint rule must satisfy: the rule does not live on disk; the rule can be re-evaluated at every allocation; the rule adds zero format bits. Three hard requirements also apply: the rule is explicit and computable and written down; a test can force the branch; the branch variable does not change in the middle of an operation.

F3. An earlier candidate, a locality identifier inherited from the parent directory at creation and never changed, was rejected because it lives in the key encoding on disk and because inheriting once at creation empties the sentence that the rule can be re-evaluated at every allocation. A note says that if a hint is ever made, the only mechanism allowed is inherit once at creation and never change.

F4. A counting model of aging placed 8192 objects in 10240 slots with 64-slot segments and treated each run of 32 consecutive keys as one directory, initially contiguous. Two hint arms were compared against the baseline over 2000 checkpoints. Arm A places a rewritten object into the lowest free slot of any segment that already holds a member of the same directory, else falls back to the baseline. Arm B gives each directory a fixed home: its initial segment plus one overflow segment taken from the initially empty tail, shared by eight directories; a rewrite goes to the lowest free slot in the home, else falls back, and the home never changes.

F5. Results after 2000 checkpoints. Arm A: the number of contiguous stretches when reading a directory is 97 to 99 percent of the baseline; a directory is spread over up to 18 to 24 segments versus 26 to 27 for the baseline. Arm B: 54 to 55 percent of the baseline; every directory stays within 2 segments; on a workload that rewrites 8 consecutive keys per request the key-order fragmentation is 18 percent lower than the baseline; on a uniform workload it is equal. The pre-registered success threshold was 50 percent and the refutation threshold was 80 percent; arm B landed between them.

F6. Cost of arm B: its overflow segments are the 32 initially empty tail segments, which are consumed within the first 2 to 15 checkpoints; with 9 commit-produced blocks per checkpoint, 17936 of 18000 such blocks could not find a clustered segment and fell back. Under the baseline that count is 0. The same quantity decided an earlier question against a policy that let user data share the clustered allocator.

F7. The target workload ranking names sequential large-file writes as primary; small-file and multi-stream workloads are secondary; the workload table has no column for how directories are interleaved.

F8. A one-shot model without aging had shown that grouping by directory buys fewer stretches on a directory walk and frees whole segments when a directory is deleted, but only when directories are interleaved at creation.

The proposed clause: the first version makes no allocator hint at all; user data placement follows the settled baseline; the criterion field is recorded as none; if a hint is ever added later, it must inherit once at creation and must not consume the empty segments that commit-produced blocks need.

Answer these six items:

1. Name one settled fact above that today consumes segment-level directory locality or whole segments freed by deleting a directory. If none of the facts consumes it, say none, and say what kind of clause would have to exist for the benefit to count.

2. Arm B is the only arm that keeps a directory within 2 segments, and it does so by taking the tail segments as overflow. Describe an overflow source that would not consume the empty segments needed by commit-produced blocks, and say whether a directory could still stay within 3 segments with that source. If you cannot describe one, say so.

3. Does arm B's fixed home violate the sentence that the rule can be re-evaluated at every allocation, given that the home is fixed at creation? Compare with why the locality identifier was rejected in F3. Answer yes or no and give the reason.

4. Arm B changes the baseline sentence "lowest free slot on the chosen device" into "lowest free slot inside the home, else lowest free slot anywhere". Is that a narrowing of the baseline (it demands more and permits nothing new) or a widening? Explain in two sentences.

5. The success threshold was 50 percent and arm B landed at 54 to 55 percent. Should the reviewer read that as a near miss that a better arm would pass, or as evidence that the threshold measured the wrong quantity, given that copy-on-write rewrites scatter slots for every policy? Give one reason.

6. Name any settled fact above that the proposed clause contradicts, quoting which one. If none, say none.
