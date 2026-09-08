You are attacking a filesystem format decision. Your job is to find concrete counterexamples
and attacks. Do not summarize. Do not agree. Look for the case that breaks it.

Do not use any markdown emphasis in your answer. No bold, no italics, no headings with asterisks.
Write in plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem stores every object in a unit: 32768 bytes for data,
16384 bytes for metadata. Each unit begins with a header. There is no implementation yet;
the on-disk format is still being designed, so any part of it can still change.

The header today, for a data unit (unit class code 1), is 105 bytes:

Common plaintext prefix, 42 bytes: magic 4, format version 2, flags 2 (byte 6 is the unit class
code, byte 7 is eight flag bits), declared length 2, header checksum 32.

Class identity section, 63 bytes: a 5-tuple of 33 bytes which is (unit class code 1, tree id 8,
object id 8, object birth generation 8, anchor offset 8); birth txg 8; fsid 8; write order 10
(instance id 4 plus transaction number low 48 bits); payload CRC 4 (CRC32C over the bytes from
the end of the header to the end of the unit).

For metadata units (class code 2 and 3) the rule is already different and already settled:
an existing invariant says that when encryption is on, metadata units may keep only the common
prefix in plaintext, and all identity fields go into the ciphertext. The header checksum then
covers only the plaintext part, and the integrity of the identity section is carried by the
AEAD tag instead. Data units are the only class exempted from this.

THE PROBLEM

When encryption is on and an attacker reads the raw disk without the key, the plaintext 5-tuple
tells them, for every data unit: which tree, which object, that object's birth generation, and
the anchor offset of this unit inside the object. The anchor offset therefore directly reveals
file sizes and the location of holes in files. The owner has ruled this leak unacceptable.

THE OBSTACLE

The AEAD associated data is settled as exactly (unit class code, tree id, object id,
object birth generation, anchor offset), a fixed-length fixed-order canonical encoding.
This composition is a day-one permanent contract.

There is a rule about where the expected values of those AAD fields may come from. Quoting it:
the expected value of every AAD field may come only from the reader's lookup path, or from a
ciphertext-side field that has already been authenticated by the tag of the level above.
It may never come from the plaintext side, and never be read back out of the pointer itself.
The design document states that if this rule fails, three of its propositions all collapse.

So the 5-tuple is simultaneously the thing that leaks and the AAD itself. On the normal read
path the AAD expected values come from the lookup path, so the plaintext header is not used
at all. But on the scan-rebuild path, which is the last-resort recovery when the index trees
are gone, there is no lookup path and no level above, so the AAD can only come from the unit
itself. That is the circularity.

Two more settled facts. First, when encryption is on, the keyless side is read-only and
detect-only; it does not move, evacuate or reclaim anything, and what it is promised to do is
exactly "detect damage and report bad blocks". There is no plaintext mapping layer and no
plaintext reverse index, so the only plaintext left on disk is the superblock and the unit
headers. Second, a measurement showed that if data units carry no self-description, scan
rebuild recovers every metadata unit but misses exactly all data units (1024 metadata plus
4096 data units gave 1024 recovered and 4096 missing), and adding self-description to data
units brought the gap to zero.

One externally verified fact: in the Linux implementation of ChaCha20-Poly1305, the decrypt
direction decrypts first and verifies the tag afterwards. Decryption itself does not consume
the AAD; the tag covers AAD and ciphertext. This says nothing about whether using unverified
plaintext to build the AAD is safe. That is exactly what you are asked to attack.

THE FOUR CANDIDATE ARMS

Arm A, leave it as it is. Keep the 105-byte plaintext header. This is the baseline; the owner
has already rejected the leak.

Arm B, move the identity into the ciphertext and make rebuild decrypt before it verifies.
The plaintext whitelist for data units shrinks to the common prefix plus fsid plus payload CRC.
The 5-tuple, birth txg and write order stay at the same offsets and the same widths but their
contents become ciphertext, so the total header width is unchanged at 105. The rebuild path
becomes: decrypt the unit, read the candidate identity out of the decrypted bytes, use that
candidate as the AAD, recompute the tag, and only believe the identity if the tag verifies.
The rule about where AAD expected values may come from gains a third permitted source for
this path.

Arm C, put the 5-tuple in the ciphertext but leave an 8-byte grouping tag in plaintext,
computed as a truncated key derivation over the 5-tuple. Without the key nobody can compute it,
so the keyless side can only compare tags for equality.

Arm D, when encryption is on, data units carry no identity section at all, and scan rebuild
accepts the degraded outcome measured above.

YOUR TASK

Attack arm B specifically, and then say whether any other arm survives your attacks better.
Answer these, each with a concrete construction rather than a general worry:

1. Construct an attack in which the decrypt-then-verify rebuild path accepts an identity that
   the writer never wrote. You control the whole disk, you do not have the key, and you may
   copy, cut, splice, replay or reorder any bytes, including whole units, and you may replay
   units from an older point in time.

2. Under arm B, name something an attacker without the key can still learn about file sizes,
   hole structure, object count or object grouping, from what remains in plaintext:
   magic, format version, flags with the unit class code, declared length, header checksum,
   fsid, payload CRC, and the physical position of the unit on the disk. Be specific about
   the inference chain.

3. Does arm B break the promise that the keyless side can detect damage and report bad blocks?
   Say precisely which bytes the keyless side can still check and which it can no longer check.

4. Arm C claims to be a middle ground. Show whether the 8-byte grouping tag reintroduces the
   very leak it was meant to remove, and quantify what an attacker learns from equality
   comparison alone.

5. Name the one thing in arm B that you think is most likely to be wrong, and say what
   observation would reveal it.

Answer in plain English. Give constructions, not opinions.
