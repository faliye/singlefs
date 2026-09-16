Whether a proposed stop rule for background compaction in a copy-on-write file system holds, and whether a proposed resume rule can loop forever.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given):

F1. A compaction intent targets region R: inside R there must be no unfreed placement whose allocation generation is <= c0. While the intent is live, R may not be an allocation destination. Intents live in a tree under the root and roll back with it.
F2. Proposed pair under test: a cluster segment counts as empty when none of its 64 slots has an unfreed allocation record (freed slots still in the defer window count as empty), and the stop predicate empty_segments(now) - segs0 >= 1, per device, is evaluated only at the moment the intent is deleted because its target state is reached; it decides whether to sign the next intent.
F3. Freed blocks become allocatable only after their free generation passes a rollback floor, so a segment can be empty by records while its slots cannot be allocated yet.
F4. Proposed resume rule under test: the stored resume point advances only past placements verified as moved or gone; when the region scan reaches the end and the full check of R against the target state fails, the resume point goes back to the start of R.
F5. The same logged_ops tree also holds intents for truncate, collapse and deleting large directories.

Questions:
1. Under F2 and F3, give a sequence where the predicate says there is output but no allocatable empty segment was gained, or says there is none while one was gained. Consider the open cluster segment, slots held back for an abandoned root after rollback, and snapshot deadlist entries.
2. Under F4, with foreground writes and frees outside R and occasional rollbacks, can the intent fail to be deleted within a bounded number of batches? Give a sequence or explain why not.
3. For a shared tree (F5), compare key layouts that start with the intent kind, with R, or with the instance id, for two needs: at most one live compaction intent per segment, and a bounded number of live intents.
4. Which choices would you make, and what single observation would change them?
