You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples and to
name candidates the material may have missed. Do not argue which option is better on
taste. Do not summarize the setting back to me.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

SETTING

Every on-disk object is a unit with a self-describing header. Pointers to data units and
to index nodes carry a key; a central map (itself a copy-on-write B-tree of class 2 nodes)
is the only authority for turning a key into a disk location. Location entries inside a
pointer are hints that may be stale. Releasing a block always goes through the map.
The map can be rebuilt from unit headers by a full scan. It must exist from the first
transaction.

Three structures are exempt from the map (bootstrap exemption): the map tree's own nodes,
the tree table unit, and the instance table unit. For these three, the location entries
inside their parent pointer are authoritative, not hints. All three are pool-wide, one
copy, with exactly one referrer. Each such parent pointer is 83 bytes wide: a 47-byte
header (which includes birth tree and birth txg), two 14-byte location entries, a 4-byte
instance number and a 4-byte birth sequence number.

The root record is a 242-byte fixed record written into a rotating root slot on every
publication (every checkpoint). It holds: magic, fsid, flags, instance number,
checkpoint_txg, a pointer to the tree table unit (83 bytes), the tree id watermark, a
self checksum, and a pointer to the instance table unit (83 bytes). The slot is 512 bytes.
The root record field table is a user decision.

The tree table unit is a 16 KiB unit rewritten copy-on-write on every publication. Each
entry is 145 bytes: length 2, tree kind 2, flags 2, tree id 8, root pointer 83,
previous_snapshot_txg 8, birth txg 8, reserved 32. Rules that hold for the tree table as
a whole: it is an open list from day one (an old reader skips an entry whose tree kind it
does not know, and simply does not see that tree); every snapshot is one entry; every
writable head has its own trees and its own previous_snapshot_txg in its entry; creating a
clone must write a watermark row for the new tree in that same publication; destroying a
head deletes its entries by prefix; unknown flag bits reject that entry with EIO. The tree
table entry field table is a user decision. Pool-wide trees that do not fork per head
(the allocation record tree and the accounting tree) already live as ordinary tree table
entries. Tree ids are planned as: extent 1, inode 2, allocation records 3, accounting 4,
central map 5, and the tree id watermark in the root record is bumped past the highest id
issued in a publication.

The superblock is a per-disk plaintext record in its own slot pair, rewritten by slot
rotation once per checkpoint, and it does not roll back with the root. A settled decision
says the superblock does not carry its own pointer to the tree table, for two independent
reasons: the publication order (units -> barrier -> journal records -> barrier -> root slot
with FUA) is a closed sequence with no superblock step, and the superblock does not roll
back with the root. Everything except the superblock is encrypted when encryption is on.

Administrator rollback picks an older root from the root ring (at most ring depth back);
the allocator cursor and all accounting values are reloaded from that root's state. An
invariant says blocks referenced by the newest K valid roots are never reallocated.

Class 2 node headers carry a tree id field and a level field. The tree table unit's own
header is planned to carry tree id 0 and level 0; the instance table unit carries tree id
0 meaning no owner.

THE QUESTION

Where does the 83-byte bootstrap-exempt pointer to the central map's root node live?
The candidates written before the review:

A. A tree table entry: tree kind = map, tree id 5, previous_snapshot_txg 0, birth txg 1.
   For this one entry the root pointer's location entries are authoritative; for every
   other kind the root pointer resolves through the map. This is the material author's
   leaning and the current planned layout of the first transaction.
B. A third pointer field in the root record (242 -> 325 bytes), next to the tree table
   pointer and the instance table pointer; the map takes no tree table entry.
C. A pointer field in the superblock (the map_sep arm of an earlier byte-budget
   experiment).

QUESTIONS

1. Rollback. Construct a sequence: publish N generations, roll back to generation N minus
   k, then dereference a version referenced only by that older generation. For each of A,
   B, C say exactly which map root the reader follows after rollback and whether the entry
   is there. Give the sequence step by step.

2. Publication order. For each of A, B, C, at which step of units -> barrier -> journal
   records -> barrier -> root slot does the new map root pointer become durable, and does
   any candidate need a fourth ordering point.

3. Mount chain. Write the chain root record -> ... -> map root node for A and for B, one
   step per line: which structure, which field, whether that field's location entries are
   authoritative, a hint, or exempt, and what rule says so. Point at any step where the
   same field (a tree table entry's root pointer) is authoritative for one tree kind and
   a hint for another, and say whether the 2-byte tree kind is enough to tell them apart.

4. Tree table rules applied to the map entry under A. Go through the rules listed in the
   setting one by one: old reader skips unknown kinds; every snapshot is one entry; clone
   writes a watermark row; destroy deletes by prefix; previous_snapshot_txg per head;
   enumerating live snapshots. For each rule, construct the operation sequence in which
   treating the map entry like an ordinary tree entry does something wrong, or state that
   the tree kind field is sufficient to exclude it and why.

5. Tree id. Under B and C the map takes no tree table entry. What tree id do its class 2
   node headers carry, does the watermark issue it, and can it collide with the meaning
   of tree id 0 (no owner) used by the instance table unit?

6. Missing candidates. Name any placement other than A, B, C that satisfies rollback,
   publication order and the mount chain, and say what it costs in bytes and in new rules.

For every counterexample give the concrete step sequence and the rule it breaks. When you
cannot construct one, say so plainly in one sentence; do not pad.
