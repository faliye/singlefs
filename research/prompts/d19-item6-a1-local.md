You are the counterexample leg of a three-way adversarial review for a from-scratch
copy-on-write filesystem. Your only job is to construct concrete counterexamples
against one verdict. Do not argue which option is better. Do not summarize.

Do not use any markdown emphasis in your answer. No bold, no italics, no asterisks.
Write plain sentences and plain numbered lists. Answer in English.

BACKGROUND

A central map is the only entry point for dereference and for the free decision.
Pointer position entries are hints only. The map is derived state: it must be
rebuildable by scanning unit headers. Relocation copies a unit byte for byte;
only the map entry changes.

Unit classes: code 1 data units (32 KiB), code 2 index nodes (16 KiB B-tree nodes),
code 3 packed record units (for example the inode tree leaves). Small objects may
also be packed into slots of a shared container; a slot carries its object's own
logical identity, a 33 byte five-tuple: unit type tag 1, tree id 8, object id 8,
object birth generation 8, anchor offset 8.

Facts you can rely on:
G1. A unit header never changes after birth. Relocation copies it unchanged.
G2. Every unit header carries its birth tree id (8 bytes) and its birth checkpoint
    number (8 bytes). A code 1 header also carries a write order field: a 4 byte
    instance id plus a 6 byte transaction number. A code 3 header carries the same
    10 byte field. A code 2 header carries only the 4 byte instance id there.
G3. Every writable mount takes a fresh instance id = max(ids seen) + 1 before it
    touches any unit; read-only mounts take none. An instance switch inside a mount
    also takes a fresh id. So an instance id is never reused.
G4. Writes are split into transactions per unit: one transaction writes at most one
    data unit's user data (this sentence is still pending user review). Transaction
    numbers count from 1 within an instance.
G5. After a failed unit write the mount switches instance: the in-flight checkpoint
    is re-issued; transactions numbered at most W are kept (adopted), those above W
    are redone with new write orders. A published-predicate judges each unit: adopted
    units are published; units of the abandoned attempt whose birth checkpoint is
    above the last published checkpoint are unpublished.
G6. Index nodes and packed containers are written only at the checkpoint's fixed
    point. The fixed point may iterate, and today's invariant text says the same
    container may be written more than once in one checkpoint; every round uses the
    same transaction number.
G7. The first copy-on-write of a node by a clone head gives the new version the
    writing tree's id as its birth tree.
G8. Authoritative state is exactly: units, accounting, roots. Indexes (trees) are
    derived. The map is derived. The reverse mapping's subject lives in each unit:
    a unit's own header answers "what object is at this location".
G9. An encryption rule says the format must provide on day 1: "reverse index entry
    identity == AAD identity == authoritative tree key, one definition". The AAD
    binds the five-tuple. Encryption is off in version one.
G10. Shared blocks (reflink, clone) do not use the O(1) snapshot free path. That
    path reads the birth checkpoint number from the pointer.

THE VERDICT UNDER ATTACK

After round 3, only one of seven candidate map-key families is not dominated: the
class-reuse write-order key, tightened with a class tag.
- code 1 key = class tag 1 + birth tree 8 + birth checkpoint 8 + write order 10 = 27 bytes
- code 2 and code 3 key = class tag 1 + birth tree 8 + birth checkpoint 8 + instance id 4
  + birth sequence 4 = 25 bytes
- the birth sequence counter scope is (tree, checkpoint number, instance id); it is
  issued in memory from 0, one counter shared by code 2 and code 3
- a pointer to a code 1 unit carries 26 bytes of this key; a pointer to a code 2 or
  code 3 unit carries 24 bytes; the class tag comes from the lookup path and the unit
  header, not from the pointer
- code 1 header: no new bytes; code 2 and code 3 headers: 4 more bytes (birth sequence)
- a packed container slot carries its object's key with its own class tag

Claims made by the verdict:
C1. Scan rebuild: for code 1 units, code 3 units, container slots and code 2 nodes of
    authoritative trees, the key can be rebuilt from the unit alone and equals what
    the referrers hold.
C2. Sharing and reflink: every referrer of one shared physical version holds the same key.
C3. Relocation: the key does not change and no referrer has to change.
C4. The class tag is available on every path that enters the map (dereference, the
    snapshot free decision, deadlist and livelist processing, relocation, checker,
    scan rebuild) without an extra read or an extra tree lookup.
C5. Five other candidates are dominated because the set of settled clauses this key
    must reopen (the code 2 header field table, the code 3 identity segment table, the
    code 2 and code 3 cells of the header width cost line) is a strict subset of what
    each of them must reopen; each of them also adds bytes to the code 1 header.
C6. The day-1 encryption rule in G9 does not constrain the map key, because the map
    is none of the three things it names.

TASK

For each of C1 to C6, try to construct a concrete counterexample. Give the unit class,
the exact sequence of writes, snapshots, reflinks, relocations, instance switches or
mounts, and the key bytes each party holds. For C5, name a settled clause this key must
reopen that a dominated candidate would not have to reopen. For C6, say what text would
have to exist for C6 to fail, and whether the facts above contain it.

If you cannot break a claim, say exactly what you tried and why it failed. Do not stop
after the first attempt on any claim.
