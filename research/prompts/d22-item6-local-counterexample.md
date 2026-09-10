Role: you are the counterexample leg of a three-way design review. Your job is to
construct a counterexample, not to agree. Answer in English. Do not use any markdown
emphasis in your answer.

Context. This is a from-scratch copy-on-write filesystem, still in format design, no code.
It targets multiple storage media, one on-disk layout per medium, each layout occupying
its own incompat feature bit, so that a reader that does not recognise a layout refuses to
mount rather than misreading bytes. The first release implements exactly one layout, plain
SSD, with no requirement on FDP or ZNS or any special device interface.

The load-bearing surface of this design is the unit: its atomicity and its self
description. Quoted: upper structures, meaning index shape, snapshot model, tiering
policy, media layout, may all be replaced; the load-bearing surface may not.

Atomicity is synthesised by the core layer, never assumed from the device. The measured
conclusion is quoted: this project may not assume atomicity wider than 512 bytes, and even
512 may only be treated as probably true; unit atomicity must be synthesised by ourselves.

The synthesis uses two shapes, split by unit class. Units that have a parent pointer need
no width assumption at all, because the checksum lives in the parent, so any torn subset
of bytes mismatches. Self-proving units, meaning root slots and journal record headers,
need a width, and that width is the physical block size probed at mount time, never a
hardcoded constant. Self-proving units are protected by whole-unit checksum plus a
generation number that is actually checked plus slot rotation.

Fixed structures today. The superblock is one copy per disk, updated by rotating among at
least two slots per disk. The journal ring is mirrored on two disks. Mkfs seeds generation
zero of the root into every region of the root ring. The root ring has three regions,
placed at prime stride, with slot order rotating across regions, and slot width equal to
the probed physical block size.

The open question. On a zoned device, meaning ZNS or SMR, each sequential zone carries a
write pointer maintained by the device. This project can neither compute it nor verify it.
After a crash the write pointer may be ahead of the commit point this project believes in.
The load-bearing surface statement, as written today, does not cover it. No candidate
shapes have ever been written down. That is what this round is for.

External fact, checked line by line against a pinned Linux 6.17 source tree today, offered
only as a hypothesis source and not as evidence. In btrfs zoned mode, superblock placement
works like this. If the zone is a conventional zone, the superblock sits at a fixed offset,
exactly as on a non-zoned device. If the zones are sequential, btrfs uses a pair of zones
per superblock mirror, reads the device-maintained write pointer to decide where to write
next and where to read from, and resets a zone before reusing it. But when both zones of
the pair are full, so the write pointer cannot distinguish them, btrfs does not use the
write pointer: it reads both superblocks and compares its own generation field. Two known
differences from this project: btrfs compares generation inside the superblock object
itself, while this project keeps its selection key, an instance number plus a checkpoint
transaction number, inside the root record, a different structure from the superblock; and
btrfs has no root ring, its three superblock mirrors are at fixed offsets.

Candidate conclusions under test. There are two separable questions.

Question one, the status of the write pointer. Candidate A1: the write pointer is an
addressing hint only and never an input to any decision. The load-bearing surface statement
is narrowed to cover only bytes this project wrote itself. All device-maintained state,
meaning zone write pointers, flash translation layer maps, and shingled-recording implicit
remapping, may be used only to decide where to write next or where to start scanning, and
may never be an input to selecting the newest version, to deciding whether something was
published, or to any integrity judgement. Rival arms: A2, the write pointer is the
authoritative upper bound during recovery. A3, the write pointer is not authoritative but
feeds a check that must go red when it disagrees with the commit point this project
believes in. A4, leave it undecided until a zoned layout line is opened.

Question two, in-place rotation. Candidate B3: branch on zone type. If conventional zones
exist, fixed structures live in them and in-place slot rotation is unchanged. If only
sequential zones exist, replace in-place rotation with zone-pair alternation, append
writes, and explicit zone reset. Rival arms: B1, always zone-pair alternation. B2, always
conventional zones, and refuse mkfs on a device that has none. B4, leave undecided.

Your task. Find counterexamples. Concretely:

1. Construct a concrete failure for A1. Find a situation on a zoned device where refusing
   to read the write pointer as a decision input loses a detection this project would
   otherwise have, and where nothing else in the design recovers that detection. Be
   specific about which unit class, which crash point, and which of the seven detection
   mechanisms is the one that fails.
2. Construct a concrete failure for A3 that A1 does not have. In particular: the checker
   in this project must run on an image, and a check whose input is device-maintained state
   cannot be reproduced from an image alone. Say whether that alone kills A3, or whether
   there is a shape of A3 that survives it.
3. Attack B3 using this project's own rule for branching, quoted: a branch variable may not
   change during an operation. Is zone type a branch variable that can change mid
   operation? Consider device replacement, adding a disk to a pool, and a pool that mixes a
   conventional-zone device with a sequential-only device.
4. Attack B3 using this project's own test for whether a format branch is worth its cost,
   quoted: a format branch is worth it only if it turns using the wrong one into failing to
   mount. Under B3, do the bytes on disk actually differ between the conventional-zone
   variant and the sequential-only variant? If they do, name the bytes. If they do not,
   then by this project's own test B3 buys nothing and must not be opened.
5. Give one concrete observation, doable today on this design with no code, that would
   falsify A1.

If you cannot construct a counterexample for a given item, say so explicitly for that item
and list what you searched. Do not invent facts not in this prompt.
