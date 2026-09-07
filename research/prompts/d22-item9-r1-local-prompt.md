You are auditing a filesystem design claim. Answer in English. Do not use any markdown emphasis (no asterisks, no bold, no italics). Write plain prose.

Your assigned stance: find concrete counterexamples. Do not summarize, do not agree politely. Either produce a specific counterexample or state plainly that you could not construct one.

## Setting

A from-scratch copy-on-write filesystem, still in format design. No code exists. All facts below are settled clauses already written down in the project knowledge base. Treat them as given.

Settled facts:

F1. The superblock is stored once per device. Updates rotate among at least 2 slots per device.
F2. One slot is exactly one sector. Sector size is probed at mount time, either 512 or 4096 bytes.
F3. The superblock slot has four sections. Section one is a bootstrap header of 162 bytes containing: magic 4, format version 2, feature bits 96, filesystem uuid 16, this device id 4, slot generation 8, whole slot checksum 32. Section two is geometry, 127 bytes, containing node bytes 4, unit bytes 4, allocation grain 4, position entry widths 4, journal geometry 24, ring parameters 4, tree table unit pointer 59, map provenance 24. Section three is tunables, 36 bytes. Section four is runtime watermarks, 12 bytes.
F4. The device descriptor table does not fit in the slot. With 8 devices it overflows by 41 bytes. So it was moved out of the slot, reached through a pointer, giving a constant 420 bytes total.
F5. Heterogeneous pools are supported. One volume can hold several device layouts at the same time. A volume declares the incompat bits of all layouts it contains, and an implementation that knows all of them mounts normally.
F6. The superblock skeleton, the feature bit encoding, and the device to layout table are declared format identical across all layouts. The stated reason is that the superblock has no layer above it, so its parsing rules must be in hand before anything is read.
F7. A pointer to a unit is a position entry of 14 bytes: device id 4, physical offset 6 expressed as a 16 KiB slot number, ciphertext checksum 4. There is one position entry per replica.
F8. A unit never spans several stripe columns. One unit lands entirely in one column. This is a settled user ruling.
F9. Units have exactly two sizes: metadata units are 16384 bytes, data units are 32768 bytes.
F10. A stripe is written with w columns, where w equals the batched write volume plus one, clamped between 2 and 4, then clamped by the number of currently writable devices.
F11. All cells of one stripe must have equal width, because parity is a bitwise exclusive or across them.
F12. Allocation grain is 16384 bytes. Allocation records are keyed by device id 4 plus 16 KiB slot number 6.
F13. Every geometry quantity that enters address arithmetic must be compared per device at mount time. Mismatch refuses the mount.
F14. There is no logical address to chunk mapping table. Physical positions are listed one by one and each carries a device identity.

## Claim T1

The bootstrap circularity objection is unfounded. The objection said: a pointer carries a device id, resolving that device id requires the device table, and under heterogeneous pools the layout of device N also lives in that table, so reading the device table requires the device table.

T1 says this does not happen, for three reasons. First, parsing any device superblock requires no layout knowledge, by F6. Second, the mapping from device id to a real device handle is built by reading each device superblock in turn and taking its own this device id field, so the device table is not consulted, by F1 and F3. Third, translating a position entry into a byte offset on a device needs only the device handle and the slot number times the grain, because a unit lands entirely in one column, so no stripe geometry is involved, by F7, F8, F12, F14.

## Claim T2

Unit size and allocation grain must be uniform across the whole pool, and this follows from already settled clauses rather than needing a new decision. The derivation: by F11 all cells of a stripe have equal width, by F8 a cell equals a unit, and by F10 the stripe width is clamped by the number of currently writable devices, so any writable device may be drawn into any stripe. Therefore cell width, and hence unit size, must be equal on every device in the pool.

T2 also says this does not contradict the settled clause that puts each layout's own node layout and stripe geometry in the may differ bucket, because node layout means the internal arrangement inside a node, not how many bytes a node occupies.

## Your task

Attack T1 and T2 separately.

For T1, try to construct a concrete mount sequence in which some quantity needed to read the device table unit is itself only available from the device table unit. Name the quantity. If you cannot construct one, say so plainly.

For T2, try to construct a concrete configuration allowed by the settled facts in which two devices in one pool legitimately have different unit sizes or different allocation grains, without violating any of F1 to F14. If you cannot construct one, say so plainly.

Also answer this separately: is there any quantity needed to turn a 16 KiB slot number into a byte offset on a device, beyond the device handle and the grain? If yes, name it and say where it would have to be stored.
