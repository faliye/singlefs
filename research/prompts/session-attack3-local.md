You are auditing a design session for a from-scratch copy-on-write filesystem called singlefs.
It is in the format-design stage: no code, no on-disk format frozen yet.
Do not use any markdown emphasis in your answer. Write plain English. Keep it under 700 words.

Your angle is: which claims cannot be re-derived by a reader who has only the repository. Two other
auditors cover whether settled items hold up and whether the open-item list is complete, so do not
spend words on those. Four earlier attack rounds already covered over-extrapolation, internal
contradictions, fragility, the repairs themselves, methodology, and omissions. Do not repeat those.

The session produced these numbers. Each is claimed to be derivable from something in the repo.

From an experiment on how many bytes an extension point inside each on-disk unit may occupy:
per-unit metadata 108 bytes, split as a 55 byte in-unit header plus 53 bytes of pointer that live
in the parent node; structural limits 4040 bytes in a 4 KiB unit, 1944 in a 2 KiB index node;
self-witnessing limits 428 bytes in a 512 byte sector after an 84 byte journal record header, and
4012 in a 4096 byte one; a lower bound range of 7, 9, 11 or 13 bytes; tree-height thresholds of
865, 1473, 6081 and 55233 for node sizes 2 KiB, 4 KiB, 16 KiB and 64 KiB with 16 million leaves,
40 byte pointers and a 64 byte node header.

From an experiment on a rotating ring of root slots: a candidate of 2 regions times 4 slots times
256 bytes, so 8 slots and 2048 bytes; one full lap is 8 operations when a root is published per
fsync, 8000 when published per checkpoint; a 256 byte slot endangers 2 slots on a 512 byte atomic
width and 16 on a 4096 byte one; raising the slot width to the atomic width makes the ring 4096 or
32768 bytes; the whole 2048 byte ring fits inside a 4096 or 65536 byte minimum io size.

From an experiment on checksum width: 1.273e12 block reads from 10 TB over 4 KiB blocks scrubbed
weekly for ten years; 1273 corrupt reads at a rate of 1e-9 per block read; minimum width 31 bits;
margin at 32 bits of 3.37; widths of 34, 37, 41 and 44 bits as the corruption rate rises by factors
of ten; theoretical miss rates of 3906, 244 and 15.3 per million at 8, 12 and 16 bits.

Some inputs come from elsewhere in the project rather than from the experiments: the 55 and 53 byte
split, the 84 byte journal record header, the 40 byte pointer and 64 byte node header, the 10 TB
pool and weekly scrub and ten year service life, the 1e-9 corruption rate, and this machine's
reported 512 byte physical block size.

Your task:
1. For each borrowed input listed in the last paragraph, say whether a reader could tell, from the
number alone as it appears in a conclusion, that it is borrowed rather than measured. Name the ones
where they could not.
2. Name every number above that would silently change if one of the borrowed inputs changed, and
say which input.
3. A reader three months from now finds one of these numbers quoted in a decision document. What
single piece of metadata, if attached to every number, would let them tell in one step whether it
is still valid? Be specific about the form it should take.
4. Name the one number here that is most likely to be quoted out of context, and say what damage
that would do.
