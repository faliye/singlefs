You are one of three independent reviewers of a design inference for a copy-on-write filesystem that is still at the on-disk format design stage. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 6.

Settled facts (do not dispute them; use them):

F1. A pool has two devices, each with two superblock slots. Slot rule: the slot generation starts at 1 and increases by 1 with each write on that device; the slot written next is generation mod 2; a reader picks, per device, the slot whose checksum verifies and whose generation is highest.

F2. A writable mount takes a new instance code = max(instance codes in the superblocks of the set of devices this mount opened exclusively, instance codes in all root records) + 1 and writes it into every superblock of that set, one slot write per device, all or nothing: an I/O error is retried until a retry budget runs out; then the copies already written are rewritten back to the old code; if that rewrite fails, the mount becomes read-only. Only after that are units written. Read-only mounts never take a code. Every later write of the mount carries the new code in its write order.

F3. Every publish persists in this order: units, flush barrier, journal record, flush barrier, root slot written with force-unit-access, superblock rotation. Crash states are: all earlier segments durable, any subset of the current segment durable. A flush barrier makes every earlier write on every device durable.

F4. Published predicate for a unit with write order (i, n) and birth checkpoint b, where i_now is the instance of the mounted root: i < i_now and no instance-table row for i means published; i == i_now means published if b <= the mounted root's checkpoint; i > i_now means corrupt. Recovery writes an instance-table row for every instance in [chosen root instance, new instance), in the same publish as the first new root.

F5. Today's implementation writes generation 2 at every acquisition and txg + 2 after each publish, the same value on both devices. After the first transaction, slot 0 holds generation 4 and slot 1 generation 5 on each device.

F6. Candidate package P under test:
P1. After both acquisition superblock writes, at least one completed flush barrier before the first non-superblock write of the new instance (on the first mount the warm-up publish's opening barrier counts). No writer issues units, records or roots before that barrier completes; the all-or-nothing rewrite back to the old code happens before any unit.
P2. Every superblock slot write (acquisition, publish rotation, rewrite back) writes that device's highest verified generation + 1.
P3. The acquisition max reads every verified slot (checksum passes, same filesystem id) on every device of the opened set; the invariant checker uses the same reading.
P4. The invariant is replaced by two sentences: (1) every root record, journal record and unit write order with this pool's filesystem id carries an instance code <= the largest superblock instance code; if anything is unreadable this half reports not-applicable; (2) within the opened set, if the devices' superblock instance codes differ, the larger one appears in no root record, journal record or unit write order. The old explicit exception for the rewrite-back window is deleted.
P5. The rollback rule's phrase "no durable effect before the rollback publish" is rewritten as: the rollback row, the rollback publish's units and the rollback root take no effect before that publish; after a crash before it, the next mount's chosen root, applied records and predicate verdicts are the same as if no rollback had been started; the new code already written into the superblocks is the only byte left behind; it makes the next acquisition skip one code, and the skipped code gets a row by the recovery row rule.

F7. Alternatives excluded last round: G1 (every superblock write uses the pool-wide highest generation + 1; reason given: on a lagging device it overwrites that device's newest slot); reading only each device's chosen slot (reason: it misses a rewrite-back write, and is safe only because rewrite-back comes before units); B2 (device 0 superblock, barrier, device 1 superblock, barrier; reason: cost only); an older invariant rewrite that only checks when the codes differ (reason: it misses the collision image).

Answer these six items:

1. I/O error path under P: a retry budget runs out on device 1, device 0 already holds the new code and is rewritten back. Consider the rewrite landing, tearing, a crash before or after it, and the read-only fallback. Is there any reachable state where a code carried by a unit, record or root is issued again by a later acquisition? Does P4 report a legitimate state on this path as an error?

2. Under P2 and P3, over several mounts (recovery falling back to an older root and publishing again, a torn rotation write, a rewrite back, an instance switch), can an acquisition that has later writes depending on it become invisible to the next acquisition? Give a history listing each superblock write as device, slot, generation, instance code, or say why none exists.

3. Does P5 open a new hole? Consider redoing a rollback after a crash that happened after the acquisition, the candidate set and the chosen old root, the rule that the new instance must cover both devices with its own roots before the rollback is confirmed to the administrator, and a shadow ledger that keeps units referenced by abandoned roots from being reused. Describe any state whose next mount differs from "no rollback was started".

4. Can a checker implement P4 (it must read the write order in every unit header, filter by filesystem id, and report not-applicable when something is unreadable)? Give any legitimate image it would flag, or any collision image it would miss.

5. Defend each of the four excluded alternatives in F7 as strongly as you honestly can: is there a hole in P2 that G1 closes, an error P3 makes that the chosen-slot reading avoids, something B2 buys that P1 does not, or a collision image the older rewrite catches and P4 misses? Say plainly where you cannot.

6. Does P change the bytes of the first transaction on the first mount (are the generations written still 2, 3, 4, 5)? How many of today's crash states in which exactly one device received the acquisition write would the P4 checker flag? What must change in code and in written rules?
