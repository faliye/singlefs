You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 6.

Settled facts (do not dispute them; use them):

F1. Instances: every writable mount opens a majority of the devices exclusively and takes a new instance code, larger than every code seen before; instances therefore run one after another, never at the same time. Every unit carries a write order (instance i, sequence n) and a birth checkpoint b. Every publish advances the checkpoint number by one from the mount's current root.

F2. Published predicate for a unit (i, n, b), with i_now = the instance of the mounted root: if i < i_now and the instance table has no row for i, the unit counts as published; if there is a row (i, T_pub, W), a code-1 unit is published when b <= T_pub or n <= W, a code-2 or code-3 unit when b <= T_pub; if i == i_now, published when b <= the mounted root's checkpoint; if i > i_now, corrupt. Units judged published can come back during a scan rebuild, so a unit that no reachable root references must never count as published.

F3. Row rule: an instance that ended uncleanly gets rows at the next recovery: for every instance i in [instance of the chosen root, new instance) write (i, checkpoint of the chosen root, the largest transaction of instance i that this replay applied). Rows are written only at recovery and rollback, in the same publish as the first new root. A rollback writes (old root instance, old root checkpoint, 0) and (i, 0, 0) for every instance in between. A clean end writes no row.

F4. Recovery always picks the newest root that can be read and verifies; that may not be the newest root ever published (a root slot can be temporarily unreadable).

F5. Admission: when space is short, the mount first pushes empty publishes that raise a rollback lower bound F, up to a bounded number. A writable mount also requires a reserved amount for instance switches.

F6. Hole C329 (constructed by an earlier attack): after an unclean end, the next writable mount first pushes an empty F-raising publish; its root lands, then the crash happens before the rows publish lands. The mount after that picks the new instance's root, and the rows it writes start from that instance, so the old instance never gets a row and its leftover units count as published by the no-row branch.

F7. Hole C330 (constructed by an earlier attack): mount 2 picks root (1, 3), takes instance 2, lands root (2, 4), crashes. Mount 3 cannot read root (2, 4) for the moment, picks (1, 3), takes instance 3, writes rows (1, 3, 0) and (2, 3, 0), publishes checkpoint 4 whose units (instance 3, birth 4) land, and crashes before its root lands. Mount 4 can read (2, 4) again, picks it, takes instance 4 and writes rows (2, 4, 0) and (3, 4, 0). Instance 3's orphans (birth 4) now count as published because 4 <= 4.

F8. The owner says the roughly 103 bytes of immutable data in each unit header can be adjusted and may grow, if a fix needs a new field.

Candidates:
C329-A: the rows publish is the new instance's first publish; it is paid from the switch reserve, never from admission, so no F-raising publish may come before it.
C329-B: every root the new instance writes, including any empty publish before the rows publish, references an instance table that already holds the rows.
C329-C: clean ends also write a row, and the no-row branch of F2 becomes "unpublished".
C330-A: every instance strictly between the chosen root's instance and the new instance gets the row (i, 0, 0), as rollback already does.
C330-B: superblock or root carries the highest checkpoint number ever issued; a new instance's first publish uses that plus one, so a temporarily unreadable root's number is never reused (format change).
C330-C: each unit header gets a field "checkpoint of the root this instance started from", and the predicate judges instances in between by it (format change).

Answer these six items:

1. Does C329-A close F6 and does C330-A close F7? Try other failures: an instance switch instead of recovery, a rollback instead of a normal mount, falling back more than one root, several F-raising publishes. Give any reachable state where an unreferenced unit still counts as published.

2. Can C330-A mark a unit unpublished that the chosen root actually references? Can any instance strictly between the chosen root's instance and the new instance have units that the chosen root references? Can C329-C misjudge clean ends, instance switches or skipped instance codes?

3. Can C329-A always be done without a format change? What if the switch reserve is short? How many bytes do C330-B and C330-C need and where?

4. What failing check would prove each candidate is enforced (for example a crash-point replay over the non-first-mount paths, a multi-mount history, an image checker)? Name one candidate for which no such check can be written.

5. Does any candidate contradict the admission rule, the warm-up rule (a new instance publishes empty roots until its roots cover both devices), the rollback rule, or the rule that rows are unique and below the mounted root?

6. Which candidate do you recommend for each hole, and what single observation would change your mind?
