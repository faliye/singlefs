You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 5.

Settled facts (do not dispute them; use them):

F1. Instances run one after another. Each unit carries a write order (instance i, sequence n) and a birth checkpoint b. Recovery picks the newest readable root; replay applies whole publishes only and never crosses an instance boundary.

F2. Published predicate, with i_now = instance of the mounted root: i < i_now and no row for i means published; with a row (i, T_pub, W), a code-1 unit is published when b <= T_pub or n <= W, a code-2 or code-3 unit when b <= T_pub; i == i_now means published when b <= the mounted root's checkpoint; i > i_now means corrupt.

F3. Rows are unique per instance and a later write overwrites an earlier one, except that a rollback row must never be overwritten by a later recovery. A row may be deleted only after a full sweep finds no unpublished unit of that instance, no read failed, all devices are online, and the root ring holds no root published by that instance.

F4. An instance switch happens inside a mount when a publish fails but a probe write succeeds: it takes a new instance code, keeps the transactions of the re-sent checkpoint up to W, and redoes the rest.

F5. The package under test:
P329: the rows publish is the new instance's first publish and every root of the instance references rows for [max(chosen root's instance, 1), new instance); every writable mount writes rows; the rows publish's metadata is paid from a switch reserve sized for one more switch; the admission rule forbids an F-raising empty publish before the rows publish; row deletion also requires every root ring slot to be readable (an unreadable slot counts as possibly holding that instance's root).
P330: every instance strictly between the chosen root's instance and the new instance that does not own kept transactions gets (i, 0, 0); a rollback's publish starts its instance table from the version the rollback target references; on a switch, W goes to the row of the instance that wrote the kept transactions, and if that instance never published a root its row is (k, 0, W_k).
R: the chosen root's own instance row takes T_pub = checkpoint of the root after replay.

Answer these five items:

1. Rows are overwritten by later writes. Can a later recovery overwrite the owner row (k, 0, W_k) with (k, 0, 0) or (k, T_sel, 0) so that the kept units turn unpublished? Should the owner row get the same protection as a rollback row? Give a history if you find one.

2. Put the switch inside other publishes: inside a rollback publish, inside a warm-up empty publish, and inside the first publish after R took effect. Does the owner rule still pick the right row and values?

3. With the extra deletion condition, a root slot that stays unreadable keeps rows forever. How large can the instance table grow, and does that affect the switch reserve, which is computed from the number of rows read at mount?

4. Write a failing check for the owner row rule and for the deletion condition, with how to prove each check can fail.

5. Does the package change the first transaction's bytes, or contradict the rule that rows are unique and below the mounted root, or the root tie-break? Which parts would you keep?
