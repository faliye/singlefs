Two questions about the intent record of background compaction in a copy-on-write file system: whether a resume point must be stored, and when the stop predicate may delete a live intent.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. An intent is an ordinary key in the logged_ops tree, riding on the same transaction layer. After power loss there are three states: no intent; intent present with work half done, so recovery resumes and idempotently redoes it; work done but intent not yet deleted, so resuming finds nothing to do and then deletes the intent. Deleting an intent must commit in the same transaction as the last batch. An intent declares a target state, never an increment. Each action in a batch first checks that what it removes is still there.
F2. The compaction intent's target state: inside region R there is no placement whose allocation generation is <= c0 and that is not freed. For these questions take R to be one 64-slot cluster segment; its shape is not decided yet. While the intent is live, R may not be an allocation destination.
F3. The stop predicate: empty_cluster_segments(now) - segs0 >= 1, where segs0 is the value written into the intent when it was signed.
F4. A decided rule lists six persisted observables. One row says: compaction progress lives in the intent record, and it is the resume point of the intent mechanism. Another decided rule says c0 lives in the intent record next to that resume point. The term resume point is not defined anywhere else.
F5. Two definitions conflict: one rule counts a segment as empty when none of its 64 slots has an unfreed allocation record; another says segments in the defer window do not count as empty. Freed slots in the defer window already have records marked freed.
F6. No rule says what happens to a live intent whose target state is not reached when the stop predicate becomes true.
F7. First-round finding under test: a stored resume point is safe only with this advance rule: advance only past placements verified as moved or gone, and before deleting the intent check the whole of R against the target state. Not storing it would change the wording of the two decided rules in F4.

Questions:
1. Can the row in F4, compaction progress is the resume point, be read as a value recomputed from the intent and the allocation records instead of a stored field, without changing its words? Give the reason.
2. Does the advance rule in F7 conflict with the third power-loss state in F1?
3. Under each definition in F5, give a sequence where the stop predicate becomes true while the target state in F2 is not reached. Under each of the three possible answers to F6 (delete the intent, keep it and stop, only evaluate the predicate when the intent is deleted by its target state), what goes wrong?
4. Find a sequence where the advance rule in F7 skips a placement that was not moved, or where the full check before deletion passes while R still holds a placement with generation <= c0.
5. Which answer to F6 would you pick, and what single observation would change your pick?
