You are attacking a set of design conclusions for a from-scratch copy-on-write filesystem called
singlefs. It is in the format-design stage: no code, no on-disk format frozen yet.
Do not use any markdown emphasis in your answer. Write plain English. Keep it under 700 words.

Your attack angle is: which single conclusion is the most fragile, meaning one new fact would
overturn it. Two other attackers cover over-extrapolation and internal contradictions, so do not
spend words on those angles.

Two rulings by the project owner, which you may not attack directly, only what was derived from them:
R1. Being cheaper in space than other filesystems is not a criterion. Saving space was never a goal.
R2. Unit geometry is computed dynamically but once, at mount time. Never per operation. Each check
returns mount or refuse-to-mount.

Experiment conclusions from this session, each five identical runs with mutation testing:

X1. Extension point byte limit. No binding upper limit was found. A previous upper limit of 19
bytes, derived from staying cheaper than ZFS, was deleted along with its reasoning because of R1.
What remains, all unrelated to space: a structural limit where the payload is squeezed to zero
(4040 bytes in a 4 KiB unit, 1944 in a 2 KiB index node); a self witnessing unit limit, because
root slots and journal record headers have no parent pointer with a checksum, so their atomic width
is the probed physical block size, giving 255 bytes for a 256 byte root slot and 428 bytes left in
a 512 byte sector after an 84 byte journal header; and tear isolation. The lower bound is a range
of 7 to 13 bytes.

X2. Root ring geometry. A candidate of 2 regions times 4 slots times 256 bytes was proposed. The
256 byte slot width is rejected: on a 512 byte atomic width one write endangers 2 slots, on 4096 it
endangers 16. The rule is that slot width must be at least the atomic width probed at mount, and
equality is enough. An invariant is missing: root slots must be aligned to the atomic width,
because a 512 byte slot offset by half a slot again straddles 2 units. The 2 regions times 4 slots
dimension has zero coverage, because the model has no region spacing and no failure domain inputs.
The whole 2048 byte ring fits inside a 4096 or 65536 byte minimum io size, so device internal
read-modify-write can cover the entire ring in one write; the fix is spacing the regions apart, not
widening slots.

X3. Ciphertext checksum width. The original derivation is reproduced: 1.273e12 block reads, 1273
corrupt reads, minimum width 31 bits. The margin at 32 bits is only 3.37 times. Five inputs all
enter linearly, so each one being wrong by that factor alone pushes the answer to 33 bits. A
corruption rate ten times higher needs 34 bits, a thousand times higher needs 41. On the collision
side, truncating to 8, 12 and 16 bits gives measured miss rates matching two to the minus n, so 32
and 64 bit rows are extrapolation, not measurement. No candidate algorithm was eliminated. A
separate finding: a truncated CRC is not a shorter CRC. Full CRC32C misses zero single bit flips by
construction, but truncated to 8 bits it misses 3212 per million.

Decisions recorded this session:
D1. A comparison table against other filesystems may only be used to refute the claim that this
project is extravagant. It may never be used as an upper limit.
D2. Unit geometry is evaluated once at mount, four checks each returning mount or refuse-to-mount.
The recorded cost: the same declaration may mount on a 512 byte device and be refused on a 4096
byte one.
D3. Disks need not be equal in size. The credit goes to two older decisions, not to R2.
D4. The granularity of variable stripe width remains open; R2 does not settle it.
D5. A new open item: how geometry is re-evaluated after the device set changes. Neither a forced
remount nor an explicit re-evaluation transaction exists anywhere in the project.
D6. An invariant was corrected: ring depth is an upper bound on the block reuse delay, not the same
number as it.

Your task:
1. Name the single most fragile conclusion. State the one new fact that would overturn it, and say
whether that fact is cheap or expensive to obtain.
2. Rank the next four by fragility, one sentence each on what would overturn them.
3. For the most fragile one, say what the project should do now: keep it with a warning, narrow it,
or withdraw it.
4. Name any conclusion that looks fragile but is actually robust, and say why.
