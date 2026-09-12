You are the counterexample leg of a three-way design review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples.
Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

SETTING

Index trees are copy-on-write B-trees of class 2 nodes. A node X that was lost must be
rebuilt from node versions still on disk, as X was at its own birth checkpoint b_X. X's
birth and checksum are read from the intact pointer that points to X. Old versions stay
on disk while any snapshot references them. Snapshots share nodes with the live tree.

Each pointer to a child holds, besides identity fields, two location entries (device,
slot, a 4-byte checksum of the ciphertext at that slot). Location entries are hints; a
central map is the authority on where a node lives. Moving a node only updates the map;
the parents keep their old, now stale, location entries and are never rewritten.

Extent tree internal nodes carry a message buffer (pending updates not yet pushed to the
children). Inode tree internal nodes carry no buffer. A settled rule says: before any
index rebuild (changing buffer size, changing node size, or damage recovery), all buffers
must be drained first, enforced by the rebuild entry point. Another settled rule says
buffered messages can be recomputed from authoritative state (data units with their write
order, the instance table, the ancestor table, tombstone units).

Rebuild rule (common to all options): pick X's children as the versions one level below
X, inside X's key range, born at or before b_X, and not covered by a later version (still
born at or before b_X) whose range overlaps; key ranges tile the parent exactly. The root's
level is stored in the tree table. Serialization of a node is canonical.

OPTIONS

C. The rebuilt node, serialized, must match the checksum stored in the intact parent
   pointer (the 4-byte ciphertext checksum when unencrypted, the MAC when encrypted).
   Otherwise stop read-only.
C-L. Every pointer to a class 2 node also stores a logical checksum (4 or 8 bytes) of the
   child's plaintext with all of the child's own location entries set to zero. The gate
   compares this instead. Location entries in the rebuilt node are filled from the map.
C-L-e0. C-L, and extent tree internal nodes keep no buffer.

WHAT TO PRODUCE

1. For C-L: a lost node where a wrong child set, or a stale or never-referenced version,
   still passes the logical-checksum gate. Say whether 4 bytes versus 8 bytes matters.
2. With location entries left out of the gate: a corrupted or tampered location entry in
   an intact node that makes a reader get wrong data without any check failing. Answer
   once for an unencrypted volume and once for an encrypted one.
3. For an extent internal node under C-L: two different buffer states of X at b_X that
   are both consistent with the same authoritative state and the same children.
4. The drain rule versus reproducing the original node: can a rebuild that must drain
   buffers first ever hand back a node equal to the original X?
5. For C-L-e0: any remaining case where the gate passes but the content is wrong, or where
   the gate fails in normal operation.
6. Without any gate: the parent P and the current version of child X are both lost, and
   an older version of X is still on disk. Does anything in the setting stop the rule from
   picking the older X?
If you find nothing for an item, say so plainly and list what you tried.
