You are attacking a filesystem format proposal. Find concrete counterexamples. Do not summarize,
do not agree, do not restate the proposal back at me. Look for the case that breaks it.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem stores every object in a fixed-size unit: 32768 bytes for
data units, 16384 bytes for metadata units. There is no implementation yet, so the on-disk format
can still change. Every unit starts with a header.

Encryption is a whole-volume, all-or-nothing setting declared in the superblock. When it is on,
the side without the key is read-only and detect-only: it does not move, evacuate or reclaim
anything. What that keyless side is promised to be able to do is exactly this: detect damage and
report bad blocks.

A data unit header today is 105 bytes and looks like this.

Common plaintext prefix, 42 bytes: magic 4, format version 2, flags 2 where byte 6 is the unit
class code and byte 7 is eight flag bits, declared length 2, header checksum 32.

Class identity section, 63 bytes: a 5-tuple of 33 bytes holding unit class code, tree id, object
id, object birth generation and anchor offset; birth txg 8; fsid 8; write order 10; payload CRC 4.

An earlier round already decided the direction: when encryption is on, the 5-tuple, the birth txg
and the write order move into the ciphertext. They keep the same offsets and the same widths, only
their content becomes ciphertext. What stays readable without the key is the common prefix plus
fsid plus payload CRC. The payload CRC is computed over the bytes as they sit on the disk, that is,
over the ciphertext, so the keyless side can still detect damage.

THE PROPOSAL YOU ARE ATTACKING

The declared length field, 2 bytes, currently sits in the plaintext common prefix. It records how
many bytes of this unit are real user data; the rest of the unit up to 32768 is zero padding.
Because one logical extent occupies exactly one unit and never spans units, the declared length of
a unit that holds a whole small file is that file's exact size in bytes. So it leaks per-file
sizes even after the 5-tuple is hidden.

The proposal is to move the declared length into the ciphertext as well, at the same offset and
the same width. After that, the only things readable without the key are: magic, format version,
flags including the unit class code, header checksum, fsid, payload CRC, and the physical position
of the unit on the disk.

WHAT THE DECLARED LENGTH IS USED FOR TODAY

First, the scanner's claim rule. When a scan walks the raw device looking for units, it steps in
fixed increments and tests each position. Once it claims a unit at some position, the repository
text says it then skips the whole payload using the declared length, so that probes landing inside
that payload are not themselves claimed as units. A measurement showed this rule matters: with the
settled form of the rule the scan misclassifies zero units, and with a different form it
misclassifies twelve.

Second, an invariant that says the bytes after the declared length up to the end of the unit are
all zero and participate in the checksum. That check runs inside the checker, which has the key.

Third, packed record units, a different unit class, judge things in this order: header checksum,
then declared length, then payload checksum, then the individual records.

One more settled fact you will need: units are fixed size, and the unit class code, which tells
you whether this unit is 32768 or 16384 bytes long, stays in plaintext.

YOUR TASK

1. The proposal's central claim is that the scanner does not actually need the declared length to
   skip a claimed unit's payload, because units are fixed size and the class code that gives the
   size is still plaintext, so the scanner can simply jump to unit start plus unit size. Attack
   this. Construct a concrete case where jumping by unit size gives a different, worse outcome
   than skipping by declared length. If you cannot construct one, say so plainly and say what you
   checked.

2. After the declared length is hidden, what can an attacker without the key still learn about
   individual file sizes? Give the inference chain concretely. Consider at least: the physical
   position of units, the total count of data units, the unit class code, the payload CRC, and the
   fact that a file smaller than one unit still occupies a whole unit.

3. Does hiding the declared length break the promise that the keyless side can detect damage and
   report bad blocks? Say precisely which bytes the keyless side can still check afterwards and
   which it can no longer check.

4. The packed record unit class judges declared length before the payload checksum. If declared
   length is ciphertext, that ordering cannot run before decryption. Construct a case where this
   ordering change lets a corrupt or hostile unit get further than it does today.

5. Name the single assumption in this proposal that you think is most likely to be false, and give
   the observation that would reveal it.

Give constructions, not opinions.
