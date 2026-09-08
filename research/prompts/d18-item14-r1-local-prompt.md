You are attacking a filesystem format proposal. Find concrete counterexamples and attacks.
Do not summarize, do not agree, do not restate the proposal back at me.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem stores every object in a fixed-size unit: 32768 bytes for
data units, 16384 bytes for metadata units. There is no implementation yet. Each unit starts with
a plaintext header. Encryption is a whole-volume setting; when it is on, everything except the
superblock and a minimal plaintext header is encrypted with an AEAD.

A decision was just taken: when encryption is on, the data unit's identity fields and its declared
length move into the ciphertext, at the same offsets and widths. What stays readable without the
key is: magic, format version, flags including the unit class code, header checksum, fsid,
payload CRC, and a field called the nonce identifier whose width and meaning have never been
defined anywhere in the project.

THE PROBLEM

There is a last-resort recovery path called scan rebuild. It runs when the index trees are gone.
It walks the raw device, finds units by their magic and header checksum, and reclaims them. Since
the identity is now ciphertext, this path must decrypt each unit to learn what it is.

Decryption needs the full 96-bit nonce. But a settled clause says the full 96-bit nonce is stored
in the pointer header, one per logical extent. And the pointers live inside the index trees, which
are exactly what the rebuild is reconstructing. So on the rebuild path there is no pointer and
therefore no nonce.

A second settled clause says a nonce watermark exists, that it must be monotonic, and that it must
not live in the bare plaintext superblock but in a field covered by a MAC derived from the master
key. So the nonce is a counter, not a random value.

A third settled clause says all replicas of one extent are the same ciphertext written N times and
must share one nonce.

A fourth settled clause explicitly rejects deriving the nonce on the fly from other fields, on the
grounds that derivation turns "which nonce goes with this ciphertext" into an inference whose
inputs can later change.

THE FOUR CANDIDATE ARMS

Arm A. The nonce identifier in the plaintext header is 12 bytes and is simply the full nonce. The
pointer keeps its copy as redundancy. Rebuild reads the nonce from the unit and decrypts.

Arm B. Arm A, plus delete the nonce from the pointer entirely, saving 12 bytes per pointer.

Arm C. The 12 bytes in the plaintext header are a whitened nonce: the writer keeps a counter, and
what goes on disk is the image of that counter under a bijection keyed by a separate subkey, so
that the on-disk values look unordered. Uniqueness still comes from the counter. The monotonic
watermark is checked on the counter, which lives in a MAC-covered field. Someone without the key
can compare on-disk values for equality but cannot recover the order.

Arm D. Do not move the nonce. Change the rebuild path so that it does not need to decrypt in order
to claim a unit. No concrete form has been proposed yet.

YOUR TASK

1. Attack arm C. The claim is that whitening hides the write order from someone without the key.
   Construct a concrete way for that attacker to recover the write order, or a useful part of it,
   anyway. You may use anything visible without the key: the physical position of each unit on the
   device, which units are adjacent, the payload CRC, the unit class code, fsid, the header
   checksum, and the fact that the filesystem is copy-on-write so overwriting a unit writes a new
   unit somewhere else and leaves the old one until it is reclaimed.

2. Attack arm A. With the full counter nonce sitting in plaintext, state precisely what an attacker
   learns. Then say whether arm C actually removes that, or only makes it harder.

3. All replicas of an extent share one nonce. Under arms A, B and C each replica unit carries that
   nonce in its own plaintext header. Construct an attack that exploits the fact that these copies
   must be equal. Consider what an attacker can do by editing one copy.

4. Arm B deletes the pointer's copy of the nonce, leaving the unit's own plaintext header as the
   only place it exists. Construct a case where that single copy being wrong, or being attacker
   controlled, causes a worse outcome than it does under arm A where the pointer also has a copy.

5. Name the single assumption across these arms that you think is most likely to be false, and give
   the observation that would reveal it.

Give constructions, not opinions.
