You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 5.

Settled facts (do not dispute them; use them):

F1. Instances run one after another (a writable mount opens a majority of devices exclusively and takes a code larger than any seen). Each unit carries a write order (instance i, sequence n) and a birth checkpoint b. Each publish advances the checkpoint number by one from the mount's current root. Recovery picks the newest readable root by (checkpoint, instance); a root slot can be temporarily unreadable.

F2. Published predicate, with i_now = instance of the mounted root: i < i_now and no row for i means published; with a row (i, T_pub, W), a code-1 unit is published when b <= T_pub or n <= W, a code-2 or code-3 unit when b <= T_pub; i == i_now means published when b <= the mounted root's checkpoint; i > i_now means corrupt.

F3. Replay applies journal records after the chosen root by swapping four root fields for the ones in the record (no tree semantics), so units written by a replayed publish are referenced as they are by the rebuilt root. A replay never crosses an instance boundary. One publish is applied whole or not at all.

F4. A clean unmount leaves nothing on disk that tells the next mount it was clean.

F5. Four packages are under test.
P329: (1) the rows publish is the new instance's first publish, and every root of the instance references a table holding rows for [max(chosen root's instance, 1), new instance); (2) every writable mount writes rows (not only after unclean ends), except the first mount after format; (3) the rows publish's metadata is paid from the switch reserve, which is sized for one more switch than before; user data redone in the same publish goes through normal admission and returns ENOSPC if short; (4) the admission rule states that no F-raising empty publish may precede the rows publish.
P330: (1) every instance strictly between the chosen root's instance and the new instance gets row (i, 0, 0); (2) a rollback's publish starts its instance table from the version the rollback target root references, then adds the rollback row and (i, 0, 0) for instances in between; (3) with consecutive switches, W goes to the row of the instance that owns the kept transactions (the chosen root's instance), in-between instances get (i, 0, 0).
Q330: a new instance's first checkpoint number = max(superblocks, readable roots, every verified journal record's checkpoint) + 1.
R: the chosen root's own instance row takes T_pub = the checkpoint of the root after replay (instead of recovery rewriting the replayed code-2 and code-3 units in its first root).

Answer these five items:

1. For each package, find a new failure that still lets a unit no reachable root references count as published, or loses a write that was already confirmed to the caller. Consider an I/O error inside the rows publish, a rollback followed by recovery, rows written at every mount being reclaimed, the Q330 maximum being raised by records from an abandoned timeline or from older instances, and journal ring wraparound.

2. Can R or P330 (2) mark a unit that the mounted root references as unpublished, or mark a unit of an abandoned timeline as published? Given that one publish is applied whole or not at all, does W still do anything?

3. What does each package cost: an extra row and chain rewrite at every mount, one more switch reserve share, the scan for Q330? Do the bytes of the first transaction on the first mount change?

4. Write a failing check for each package that would catch its removal, and say where it runs (a multi-mount crash-point replay, or an image checker).

5. Does any package contradict the admission rule, the warm-up rule (a new instance keeps publishing empty roots until its roots cover both devices), the rollback rule, the root tie-break (equal checkpoint numbers go to the higher instance), or the rule that rows are unique and below the mounted root? Which combination do you recommend?
