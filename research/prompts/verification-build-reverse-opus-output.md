# Adversarial review of P1 to P6 (reverse leg)

Stance: assume the proposal is wrong and find where. Everything below was checked against the
repository on 2026-09-03. One finding is not an argument but a measurement: I ran the repository's
own E77 binary and its output contradicts F13 and P4 directly. Reproduction command and raw output
are in section 7.

Terminology used throughout for E77's four arms, because the arm names are misleading and the
confusion is load bearing. Writes are U (six COW units), R (three journal records), S (root slot).

1. b_all = [U][R][S], both barriers present. This is the settled order of F8 / D16 item 7.
2. b_ur = [U][R,S], only the unit-to-record barrier kept. This is "the barrier before the root
   slot has been removed".
3. b_rs = [U,R][S], only the barrier before the root slot kept. This is "the unit-to-record
   barrier has been removed".
4. b_none = [U,R,S], both barriers removed.

The arms are named after the barrier that is kept, not the one that is removed. F13 and P4 read
them as if they were named after the one removed.

---

## Task 1. Strongest concrete failure of each of P1 to P6

### P1. Break: the checker cannot be the unary predicate F2 settled, for two invariants P5 puts in scope

The bug. P1 makes singlefs-check independent of singlefs-core and F2 fixes O2 as a unary predicate
P(image) over ONE image. Two invariants P5 places in the first-version checker scope cannot be
evaluated from one image. I-7.5 requires that each root slot start offset be a multiple of the
physical_block_size probed at mount. I-8.2 requires that each journal record header fall entirely
inside one probed physical_block_size unit, and says in its own text that the width is probed at
runtime and must not be hard coded. F7 and D22 item 2 make the root slot width equal to that probed
value. The probed value is a property of the device, not of the image. The superblock is the only
place it could be recorded, and D22 open item 9 leaves the superblock field table undefined, so
today it is nowhere in the bytes.

The image that exposes it. Take the step-3 mkfs image built on a file-backed device in the P2
userspace harness. A regular file has no physical_block_size, so mkfs takes the default 512 and
lays root slots at stride 512. The intended deployment target is a 4Kn device with
physical_block_size 4096.

Why each judge passes. Recovery reads root slot candidates at stride 512, the same constant it
wrote with, and finds all three regions. The checker receives 512 from the harness (it has no other
source), so I-7.5 is green because every slot offset is a multiple of 512, and I-8.2 is green
because 78 is less than 512. The record verifier does not look at geometry at all.

Why each self-test passes. C76's two self-tests concern the barrier before the root slot and per-item
validation in replay. C42 concerns jsn continuity. C22 concerns the delayed-reuse window. C80
concerns the accounting flush. None of the five reads a slot stride, so all five behave identically
whether the stride is 512 or 4096.

What actually breaks. D22 item 2 says mkfs divides by the pool's max physical_block_size and mount
recomputes per device and refuses to mount on mismatch. So the image that passed the entire
verification pipeline is unmountable on its intended target, and I-7.5's stated purpose (E41
measured 1 risky slot when aligned versus 2 when offset by 256) has been checked against a number
the checker got from the thing it is auditing.

Label. Break of P1's premise that singlefs-check is an independent judge. It is independent of
singlefs-core's code and dependent on singlefs-core's runtime beliefs, which is the part that
matters here.

Secondary, labeled a cost: P1's own crate graph puts core and check in one process. singlefs-crash
must call the implementation's recovery, so it depends on singlefs-core; P1 also has it run the
checker per crash state, so it depends on singlefs-check. Nothing in D13's rule ("must not call any
deserialization or traversal code of the crate under test") is violated textually, but any lazily
initialized value in that process (the probe result, a CRC32C table, an endianness helper) is shared
by construction, and this is the delivery mechanism for the break above.

### P2. Break: tier 0's base image can never contain a journal residual, so an entire settled failure class is unreachable by construction

The bug. Recovery derives expected from the chosen root's watermark (F9) and stops at the first gap.
D23 never settled how jsn maps to a ring slot; E32 measured the slot = jsn mod n branch, and on that
branch a record left in the ring by a discarded timeline has a good self checksum and a jsn exactly
equal to expected. D23's own text says so in one line: jsn == expected is a pass, not a gate.

The crash state that exposes it. It requires the ring to already contain a record that this run did
not write. P2's recorder captures the writes the implementation issues; the harness applies a subset
of them to a base image. For tier 0 the base image is the freshly mkfs'd image, whose ring is empty.
F3 sizes tier 0 at tens of write requests, and P6 step 3 and step 4 both use a single publication.
Under those conditions no residual can exist no matter how exhaustive the enumeration is, because
exhaustiveness is over subsets of this run's writes, not over the contents of the disk.

Why each judge passes. There is nothing to catch, because the state is never generated. When it is
eventually generated in production, the checker is unary on the post-recovery image and I-8.3
(replay prefix strictly consecutive) is satisfied by the residual. Only the record verifier, which
takes the pre-crash image and the record stream, can tell the two timelines apart, and P6 defers it
to step 5.

Why each self-test passes. C42 is exactly the self-test for this class, and it is the one P4
requires to go red. P6 assigns it to step 4, whose workload is F6's first transaction: one
publication. C42's sequence needs gap, recover, write a little, crash again, which is at minimum
three publications with a recovery in the middle. So the self-test cannot go red in the step it is
scheduled in, and by P4's own rule the harness result never counts.

Label. Break of P2's claim that a userspace recorder over the block-device trait is enough for tier
0. The recorder is not the deficient part; the base image is. P2 says nothing about seeding it.

### P3. Break: the torn carve-out is empty for the root slot on every device, and empty entirely on a 4Kn device

The bug. P3 enumerates torn sub-states only for writes larger than the probed physical_block_size
and only for self-certifying units. D22 item 2 sets the root slot width equal to the probed
physical_block_size. A root slot write is therefore never larger than the probe, on any device, so
the carve-out that P3 introduces specifically to protect self-certifying units never applies to the
root slot. On the local machine D20 measured physical_block_size = 512, so the torn set is the
journal record (4096) and the superblock slot. On a 4Kn device the probe is 4096, the journal record
is exactly 4096 and not larger, and the torn set is empty. P3 then degenerates to F10's model, in
which every write is atomic, while F11 and D20 say the project may not assume atomicity above 512
and may treat even 512 only as probable.

The crash state that exposes it, composed with the P6 finding below. Take a 4Kn device, a 4096-byte
journal record, and the 78-byte header that P6 step 2 builds parsers for. Arithmetic: 78 + 7 x 56 =
470, which fits in one 512-byte sector; 78 + 8 x 56 = 526, which does not. So from the eighth named
item on, the named-item array crosses a sector boundary. Crash with sector 0 durable and sector 1
not: sector 1 still holds the previous wrap's bytes at that offset. The header (78 bytes, all in
sector 0) is new; its 32-byte header checksum covers the header only, so it passes. jsn equals
expected, so the prefix condition passes. D23 item 13's payload checksum over the named-item array
is one of the three settled fields that D23 item 4 says are not in the 78 bytes, so there is no
payload checksum to fire. Named item 8 is therefore the previous record's item 8: it names a unit
that is still physically on disk and still CRC32C-valid. F9's per-item validation reads that unit,
computes its CRC32C, and it matches. The stale unit is grafted.

Why each judge passes. Recovery reports success. The checker sees a grafted unit with a valid
91-byte header, a matching 32-bit CRC32C in its parent pointer, and birth txg not exceeding the
root's checkpoint_txg; if the residual item named the previous version of the same tree node, which
is the likely case, the five-tuple matches its position too, so I-1.1, I-1.2, I-2.1, I-4.2 and I-4.8
are all green. The record verifier is the only judge that compares the record stream against the
result, and P6 defers it to step 5. RefFS is also step 5.

Why each self-test passes. None of the five produces a torn record. C76's two are about barriers and
about validation being switched off, C42 is about a jsn gap, C22 about the reuse window, C80 about
accounting. P3 does not enumerate the state, so no self-test can be built on it.

Label. Break. This is E77's silent-graft outcome (63 states of the b_rs arm) reached through a
channel P3 explicitly excludes from enumeration.

Cost, labeled as such: P3's collapse rule for parent-pointered units rests on a 32-bit CRC32C, not a
digest. D19 settled item 2 fixes the ciphertext checksum at 32 bits, and D9 item 10 keeps the 16-byte
MAC reserved and unwritten in the first version, so in the first version the parent pointer's only
integrity value for its child is 4 bytes. The collapse is sound at 2^-32 per torn state, which is
acceptable, but the plan should say it is 2^-32 and not "any torn content fails the parent
checksum".

### P4. Break: four of the five required self-tests cannot go red in the configuration and workload the plan assigns them to

This is the strongest finding in the review and it is a measurement, not an argument. Running the
repository's E77 binary unmodified gives, per arm, in validating and in naive mode:

1. b_all validating: 72 states, 0 violations. b_all naive: 72 states, 0 violations.
2. b_ur validating: 79 states, 0 violations. b_ur naive: 79 states, 0 violations.
3. b_rs validating: 513 states, 0 violations. b_rs naive: 513 states, 63 violations.
4. b_none validating: 1024 states, 504 violations. b_none naive: 1024 states, 567 violations.

Self-test C76-a, "remove the barrier before the root slot, must go red". Removing the barrier before
the root slot from the settled order b_all yields b_ur. b_ur has 0 violations out of 79 states, in
both modes. The self-test goes green. To reach 504 out of 1024 you must remove both barriers, which
is b_none, and that is what F10's sentence and C76's parenthetical ("built after E77's b_none arm")
actually describe. The mechanism is elementary: with the unit-to-record barrier kept, any durable
root implies all records durable implies all units durable, so a published root can never point at a
missing unit.

Self-test C76-b, "disable per-item validation in replay, must go red". Under the settled two-barrier
order b_all, naive equals validating: 0 violations out of 72. Per-item validation is provably dead
code in E77's model once both barriers are in place, for the same reason: replay only runs when the
root is absent, and any durable record already implies every unit durable. Naive only diverges in
b_rs (63) and b_none (567 versus 504), that is, only when the unit-to-record barrier is also removed.
The self-test goes green.

Self-test C22, "set the delayed-reuse window to 0, I-4.8 must go red". I-4.8 needs at least K = 2
generations of roots and needs a block freed by one publication and rewritten by a later one. F6's
first transaction is mkfs then one publication. There is no later publication to do the reuse, and
on a freshly mkfs'd disk the allocator has no reason to pick a just-freed location anyway
(D3 item 5 allocates commit-internal blocks from the clustered segment). Cannot go red.

Self-test C42, as analysed under P2: needs at least three publications and one recovery. Cannot go
red in step 4.

Self-test C80, "remove the accounting flush from the publish path, I-3.1 must go red". This one is
reachable with a single publication: the persisted allocated statistic stays at its mkfs value while
the traversal sum grows by the new data unit and nodes.

Consequence under P4's own rule. P4 says the five self-tests are required to go red before the
harness result counts. Four of them cannot, so P6 steps 3 and 4 can never be declared complete. The
realistic outcome is worse than a stall: someone reworded the self-tests until they go red, and the
rewording will be invisible because F13's own text already conflates b_ur with b_none.

Label. Break, and it is reachable today with no code at all, by rerunning E77.

Second break in P4: P4 requires three judges per crash state starting immediately, while P6 delivers
the record verifier only in step 5. Steps 3 and 4 therefore run with two judges. The class this
loses is named in E77's own tally column: record_holes, states where the root is durable and records
are missing. b_ur has 7 of 79 such states; b_none has 448 of 1024. E77 counts them separately
precisely because a unary checker cannot see them: the tree is intact and the record stream has a
hole, which is only visible with the record stream as a second input.

### P5. Break: I-7.3 is red on the mkfs image and on nearly every tier-0 crash state of step 3

The bug. I-7.3 (ring health) requires that, besides the record with the highest generation, the
valid root slot set contain at least one record with a strictly earlier generation. F7 and D22 item
8 say mkfs seeds generation 0 into every one of the R = 3 regions. So on the mkfs image the valid
set is three records all at generation 0: the maximum is 0 and there is no strictly earlier record.
I-7.3 is red on the first legal image the project will ever produce.

The crash states that expose it. Step 3's minimal write stream on the settled 2-disk first version
(D2 items 6, 9 and 10; D22 item 8) is: two copies of the tree-table unit, then a barrier, then two
mirrored copies of the journal record, then a barrier, then one root slot with FUA. Segments
[2][2][1] give 4 + 4 + 2 = 10 raw states minus 2 duplicates = 8 crash states. The root slot persists
in exactly 1 of them. In the other 7, recovery legitimately rolls back (fsync did not return, so
E77's own is_violation rule says rollback is not a violation), and the post-recovery image again has
generation 0 in all three regions. So the checker reds on 7 of 8 tier-0 states in the very step P6
uses to prove the harness works. Adding the accounting node makes it [4][2][1], that is 19 of 20.

Why each judge passes and only the checker reds. Recovery is correct. The record verifier agrees
that no committed record was lost. Only the unary checker fires, and it fires on a legal state.

Why this is worse than noise. The predictable fix is to weaken I-7.3, either by relaxing "strictly
earlier" to "distinct slot" or by special-casing the post-mkfs image. That is exactly C13's failure
mode: a checker that still runs and still goes green but no longer decides. The rationale text of
I-7.3 says its value is that it catches broken rotation logic on the next ordinary commit without
needing a crash; the weakening would take precisely that away.

Label. Break of P5's scope claim.

Additional, labeled a cost: if the 78-byte header forces one-record transactions (see P6 below),
I-4.3 (commit is atomic, no partially applied transaction) becomes a tautology on a single image, and
P5 lists it as judged. E42 measured that 87.4 percent of crash points land in the middle of a
transaction, so the invariant P5 counts on to cover that is the one that would be vacuous.

### P6. Break: step 2 and step 3 are built on a journal header width that D23's own text contradicts, and step 4 changes it

The bug. F7 and P6 step 2 use "journal header 78 bytes". D23 item 4's own paragraph says, verbatim,
that three settled but unlanded increments are not in the 78: item 7's transaction number plus
commit marker (9 bytes), item 8's back-chain (4 bytes), item 13's payload checksum (4 bytes),
totalling 95. A second passage in the same file, in the item 9 discussion, says the settled unlanded
increments make it 91. So the kb states two different values for the same quantity, and the
authoritative marker registered for the gate is 78.

Why the plan cannot detect this. Gate stage 27 pins the marker
`format-const: JOURNAL_HDR = 78 stale=... 84 ... 86 ...` and enforces that no other value appears in
kb prose or experiment sources. So the one automated mechanism P1 relies on actively enforces the
value that D23's own prose contradicts, and lists 84 and 86 as the stale ones while 91 and 95 are
not mentioned. Both the core author and the checker author read the same settled sentence and both
write 78.

What it costs at step 4. F9's third prefix condition is "commit marker present for the transaction".
The commit marker is in the missing 9 bytes. An implementer working from the constants file has two
options. Either add the field, in which case step 2's two parsers, step 3's replay corpus and every
mutation-table known-bad image built from them are invalidated, which is exactly "a step whose
success is meaningless because a later step changes the bytes it tested". Or make every transaction
exactly one record, in which case D23 item 7 ("one transaction may span several records", settled)
is silently dropped, F9's commit-marker condition becomes vacuous, I-4.3 becomes a tautology, and
nothing in P4's five self-tests notices, because none of them tests multi-record transactions.

Second break: step 3 cannot be started when the plan says it can. Step 3 is mkfs plus an empty
publication. D22 item 7 makes the root record always indirect through a tree-table unit, so the
publication must write one. The tree-table unit is an index node, and D18 item 7 gives the index-node
class identity segment a field set with no widths, verbatim "widths follow the implementation". D22
item 7 carries its own 2026-09-03 warning that its capacity figure of 267 must be recomputed and
cannot be computed today for exactly this reason. first-txn-layout lists the tree-table unit header
as one of five shape-settled, width-unsettled segments that deliberately carry no open-item number,
so gate stage 31 (every open item must have judged whether it changes the first transaction's bytes)
says nothing about it, and P6 step 0 does not settle it because step 0's list is the four F6 open
items plus two D13 items. Step 3 therefore starts on undefined bytes.

Third break: the P4 versus P6 contradiction on the record verifier, stated under P4 above.

Cost, labeled as such: P6 step 0 proposes registering the crash-state model as D13 open item 4, the
sharing set as D13 open item 5, and the vehicle question as D17 open item 5. All three already exist
as of 2026-09-03. D13 open item 4 also states that the candidate P3 adopts ("only enumerate torn for
self-certifying units, collapse torn parent-pointered units into not durable") is an inference that
must go through three-way argument before it is settled. P3 adopts it as settled without that
argument.

Cost, labeled as such: step 3's mkfs must write I-8.1's worst-case journal occupancy and F into the
superblock, computed at mkfs and recomputed at mount. The worst case is a function of how many named
items a transaction can produce, which is not known until step 4's transaction shape exists. So
step 3's superblock bytes are provisional and step 4 rewrites them.

---

## Task 2. Attacking P1's sharing rule: one wrong constant that keeps both sides green

The constant: JOURNAL_HDR = 78.

Why it is wrong. D23 item 4's own note says the three settled unlanded fields bring the header to
95; the item 9 discussion in the same file says 91. Both figures are settled consequences of settled
items (D23 items 7, 8 and 13), so 78 is not merely incomplete, it is a header that cannot carry
three fields that F8 and F9 make recovery depend on: the commit marker, the back-chain CRC32C and
the payload checksum.

Why both sides agree. P1's generated file is generated from the kb and gate stage 27 enforces the
registered marker, which is 78. singlefs-core writes 78-byte headers; singlefs-check parses at
78-byte headers; there is no third reading of the kb anywhere in the plan, because O3 (the
other-language spec executor of F2) is not in P6 at all.

Why every planned check stays green. The header checksum is 32 bytes over the header, so it is
self-consistent at any header length both sides agree on. I-8.2 (header inside one physical block)
is green: 78 is less than 512 and less than 4096, and so is 95. I-8.1 (ring geometry) is green:
mkfs computes the worst case from the same 78. I-8.3 (prefix strictly consecutive) is green: jsn is
present in 78. I-8.4 (replay idempotent) is green. I-4.3 is green because with one-record
transactions there is no partial transaction to find. The five self-tests of F13 are green-then-red
exactly as before, because none of them depends on the header length: C76-a and C76-b are barrier
and validation switches, C42 is jsn continuity, C22 is the reuse window, C80 is the accounting
flush. The record verifier, which is the one judge that would compare a transaction's record set
against the applied result and could notice a missing commit marker, is deferred by P6 to step 5.

A second constant, of a different class, worth stating because it changes the answer to the second
half of this question. D18 item 7 gives the data unit's class identity segment as "five-tuple, 33
bytes" as a group. The per-field split of those 33 bytes (unit type tag, tree ID, object ID, object
birth generation, anchor offset) appears nowhere in the kb. The generated constants file must
therefore invent the split. Whatever split it invents, core writes it and check parses it, so I-1.1
(header states its own logical address) and I-1.3 (header tree ID matches) become tautologies: the
checker extracts the fields at the same offsets the core wrote them.

Would forbidding the shared file have caught it?

For JOURNAL_HDR = 78, no. Two independently written parsers would both read the same settled
sentence "header 78 bytes" in D23 item 4 and both produce 78. Independence of code does not buy
independence of premise. This is C48's shape exactly, and it is the class that hurts, because the
error is in the kb.

For the 33-byte five-tuple split, yes, and it is the only class where forbidding the file helps: two
independent inventions of the split would disagree on every image, and the disagreement is loud and
immediate. Forbidding the shared file converts an underspecified kb entry from a silent tautology
into a noisy failure. That is a real argument for forbidding it, but it is a narrow one.

For the probed physical_block_size (section P1 above), forbidding the file changes nothing, because
that value is not in the kb and never passes through the constants file. It reaches both sides
through the harness, by design, since D22 item 2 deliberately makes it a runtime value rather than a
format constant.

So the blind spot is elsewhere, in two places. First, in the kb itself being self-inconsistent while
the gate enforces one of the two values; gate stage 27's own header says it only checks constants
that have been registered, and there are exactly three registered in the whole repository
(NODE_BYTES = 16384, DATA_UNIT_BYTES = 32768, JOURNAL_HDR = 78). Every other width in F7 (the
31-byte pointer head, the two 11-byte location entries, the 121-byte root record and its seven field
offsets, the 91-byte unit header and its 42-byte prefix, the 56-byte named item, the 4096-byte
record) is unregistered and can drift silently. P1's claim that "the existing gate keeps it in sync
with the kb" is therefore false as stated for all but three constants, and that is checkable in one
command. Second, in the values that are deliberately runtime rather than format, which the sharing
rule does not even mention.

---

## Task 3. Attacking P2's claim that a userspace recorder is enough for tier 0

What a userspace recorder over a write/flush/FUA trait structurally does not observe:

1. Sector-level completion inside one trait-level write. It records one 32768-byte write; the device
commits 64 sectors of 512 independently. F10's model assumes each write is atomic, and that
assumption is not a modelling simplification the recorder could ever correct, because the recorder
sits above the split point. Does it matter for F6's first transaction? Not for parent-pointered
units: the parent pointer's checksum covers the whole unit, so any partial content fails. Not for
the journal record either, at F6's size: F6's transaction names roughly five units (data unit,
allocation record, accounting node, inode, tree-table unit), and 78 + 5 x 56 = 358 bytes, entirely
inside the first 512-byte sector. It starts to matter at eight named items (526 bytes), which is an
ordinary file write touching eight extents, that is, the second workload the project will run.

2. Whether the FLUSH reached the device. If the harness image is a file, the trait's barrier is an
fsync and the ordering guarantee is the host filesystem's, not the device's. P2 concedes this to C6
and the QEMU tier, but F12 says the VM harness initramfs contains only busybox and the test binary
with no kernel modules, so C6's self-test ("dropping one FUA must make the crash test go red")
cannot be run until modules are added. Every tier-0 result is therefore conditional on an assumption
that currently has no self-test. Labeled a cost, since P2 acknowledges it, but the plan should say
that the acknowledgement is not yet a check.

3. Reads. The recorder records writes. Every recovery bug whose trigger is what recovery READS
rather than what the run WROTE is invisible: C42's residual record, C77's stale tail over reused
blocks, C29's tail-first recovery. Making those reachable requires seeding the base image, and P2
and P6 say nothing about seeding. This is the break stated under P2 in Task 1, and it matters for F6
in the negative sense: it means the F6 workload can never exercise them, so the self-tests that P4
requires cannot go red there.

4. The mirror pair. D2 item 9 fixes the first version at 2 disks, D2 item 6 fixes w at least 2, D2
item 10 puts a whole unit on one column with one location entry per copy, and D22 item 8 mirrors the
journal ring on two disks. So every unit and every journal record is two physical writes on two
different disks that can persist independently. P2 does not say whether the block-device trait is
per-device or pool-level. If per-device, the recorder sees both writes and the enumeration doubles,
which means E77's counts (72, 79, 513, 1024) cannot be used as the state-count assertion for the
harness, and the state-count assertion is E77's own criterion 2 for not voiding the round. If
pool-level, the divergent-mirror states are lost entirely. For F6 the correctness case is saved by
the barrier (both copies are in the units segment, so a durable root implies both copies durable),
so this is a cost rather than a break, but it invalidates any state count copied from F10.

5. A flush on a 2-disk pool is two device flushes, not one event. The trait as described in P2 has
no device argument on the barrier. Labeled a cost for F6, because a correct implementation issues
both and waits for both, and the model's single frontier is then sound.

---

## Task 4. Attacking P3's crash-state model

Direction one, a state a real device produces that the model cannot enumerate.

P3 enumerates torn sub-states only for writes larger than the probed physical_block_size and only
for self-certifying units. D22 item 2 defines the root slot width as exactly the probed
physical_block_size. So the root slot write is never larger than the probe, on any device, and P3's
carve-out is empty for the root slot by construction, which is the one write in the whole protocol
that has no parent checksum. On this machine, where D20 measured
atomic_write_unit_min = atomic_write_unit_max = logical_block_size = physical_block_size = 512, the
torn set is the journal record (4096 bytes, larger than 512) and the superblock slot. On a 4Kn
device the probe is 4096, the journal record is exactly 4096 and therefore not larger, and the torn
set is empty: P3 collapses to F10's no-torn model on the entire device class, while F11 and D20 both
say that even 512 may only be treated as probable and that nothing above 512 may be assumed.

The concrete state, with the numbers requested. Probed physical_block_size 4096, journal record 4096
bytes, header 78 bytes, named item 56 bytes. 78 + 7 x 56 = 470, fits in sector 0 of 512. 78 + 8 x 56
= 526, crosses into sector 1. Crash with sectors 0 to 3 durable and sectors 4 to 7 not (a 512-byte
tear inside a 4096-byte write, which D20 says the device may do because Linux's physical_block_size
is only the device's own claim). Sector 1 still holds the previous wrap's bytes. The header checksum
(32 bytes) covers only the header, so it passes. jsn equals expected, so F9's first condition passes.
With the 78-byte header there is no payload checksum, so nothing covers the named-item array. Named
item 8 is the previous record's item 8, naming a unit that is still on disk and still CRC32C-valid,
so F9's per-item validation passes and the stale unit is grafted. P3 enumerates none of this.

If the header is 95 bytes with D23 item 13's payload checksum present, the payload checksum catches
it and the record is dropped, which is the correct outcome. That is the point: P3's model gap is
only harmless if P6's header width is wrong, and P6's header width is what the gate currently
enforces. The two findings compose.

Direction two, over-approximation producing violations that cannot happen.

I-7.3 on the mkfs image, as set out under P5 in Task 1. Concretely for P6 step 3 on 2 disks: write
stream is 2 tree-table unit copies, barrier, 2 mirrored journal record copies, barrier, 1 root slot
with FUA. Segments [2][2][1] give 4 + 4 + 2 minus 2 duplicates = 8 tier-0 crash states. The root
slot persists in exactly 1. In the other 7, recovery legitimately rolls back to generation 0, and
the image then has three valid root records all at generation 0, no strictly earlier record, and
I-7.3 red. So 7 of 8 exhaustively enumerated tier-0 states report a violation that is not a
violation, in the step P6 designates to prove the harness works. This is not a defect of the
enumerator; it is a defect of putting I-7.3 into P5's scope without noticing that D22 item 8's
"mkfs seeds generation 0 into every region" makes it false at generation 0. Labeled a break of P5.

A second over-approximation, labeled a cost: the model's "not durable" is a boolean, but the actual
bytes at a not-durable location are whatever was there before. On a fresh image that is zeros and
harmless. Once the delayed-reuse window has elapsed and a location is reused, "not durable" means a
valid old unit with a valid header and a valid CRC32C, which the checker's scan direction will
accept as live (I-1.2 passes, because its birth txg is below the current checkpoint_txg) and which
I-5.1 may then see as a doubly-claimed physical range. Tier 0 as scoped by P6 never reaches this,
for the same reason C22 cannot go red there.

---

## Task 5. Attacking P6's build order

1. A step whose success is meaningless because a later step changes the bytes it tested: step 2.
It builds two independent parsers for the journal record header at 78 bytes. D23 item 4's own text
says the three settled unlanded fields make it 95 (9 + 4 + 4), and the item 9 discussion in the same
file says 91. Step 4 needs the commit marker, which is inside the missing 9 bytes, because F9 makes
"commit marker present for the transaction" one of the four prefix conditions. So step 2's parsers,
step 3's replay corpus, and every mutation-table known-bad image derived from them are invalidated
at step 4. Gate stage 27 will not warn, because it pins 78 and lists only 84 and 86 as stale values.
Break.

2. A step that cannot actually be started when the plan says it can: step 3. mkfs plus an empty
publication must write a tree-table unit, because D22 item 7 makes the root record always indirect
through one. The tree-table unit is an index node; D18 item 7 defines the index-node class identity
segment as a field set with no widths ("widths follow the implementation"). D22 item 7 itself
carries a 2026-09-03 warning that it can no longer compute its own capacity figure for exactly this
reason. first-txn-layout classifies the tree-table unit header as one of five shape-settled,
width-unsettled segments that deliberately carry no open-item number, so gate stage 31 cannot speak
about it and P6 step 0 does not settle it (step 0's list is the four F6 open items plus two D13
items). Break.

3. A second instance of the same shape, labeled a cost: step 3's mkfs writes I-8.1's worst-case
journal occupancy and F into the superblock, computed at mkfs and rechecked at mount. That worst
case depends on the maximum named-item count of a transaction, which is only fixed once step 4's
transaction shape exists. Step 4 changes the superblock bytes step 3 wrote.

4. An internal contradiction between P4 and P6: P4 requires three judges per crash state; P6 delivers
the record verifier only at step 5. Steps 3 and 4 run with two judges. The uncovered class is E77's
record_holes column, 7 of 79 states in b_ur and 448 of 1024 in b_none, which E77 counts separately
precisely because a unary checker cannot see a hole in the record stream. Break.

5. Step 4's stated exit criterion, "all five self-tests red then green", cannot be met, per Task 1
under P4: C76-a is green (b_ur, 0 of 79), C76-b is green (b_all naive, 0 of 72), C22 needs a second
publication, C42 needs three publications and a recovery. Only C80 is reachable in F6's workload.
Break.

6. Labeled a cost: step 0 proposes registering three open items that already exist as of 2026-09-03
(D13 open item 4, D13 open item 5, D17 open item 5), which suggests the plan was written against a
snapshot of the kb that is already stale. D13 open item 4 additionally records that P3's chosen
candidate is an inference requiring three-way argument; P3 adopts it as settled without one.

---

## Task 6. The single reachable break, and what would change my answer

The reachable break is P4's self-test set, and it is a break rather than a cost because it is not a
risk that something might go wrong later: it is a measurement available today. Running the
repository's own E77 binary shows that removing the barrier before the root slot from the settled
two-barrier order gives b_ur with 0 violations out of 79 states, and that disabling per-item
validation under the settled two-barrier order gives 0 violations out of 72, because with the
unit-to-record barrier in place a durable root implies every unit durable and the replay path never
runs. Both of C76's self-tests, which P4 makes a precondition for the harness result counting at
all, go green in the configuration the project actually shipped in D16 item 7; F13 and F10 read
E77's b_none numbers (504 of 1024) as if they belonged to the b_ur arm. Add C22 and C42, which need
a second and a third publication that F6's first transaction does not have, and four of the five
required self-tests cannot go red where P6 schedules them. The danger is not the stall; it is that
the wording will be adjusted until the tests go red, and the adjustment will look like a correction
because the kb's own sentence already conflates the arms.

The one thing that would most change my answer: extend E77 to two publications with block reuse and
a ring seeded with a residual record from a discarded timeline, then rerun the naive arm under the
settled two-barrier order. If per-item validation goes red there with a non-zero count, then C76-b
is a sound self-test that merely needs its workload restated (two publications instead of one) and
its arm named correctly, and my headline shrinks to a wording defect plus the C22 and C42
scheduling problem. If it is still 0, then per-item validation as settled in D16 item 7 has no
self-test that can ever fire under the shipped barrier configuration, and P4's precondition is
unsatisfiable rather than merely misscheduled.

---

## 7. Reproduction

Command, run from the repository root on 2026-09-03:

    cd research/e7-index-bench && cargo run --release --quiet --bin e77-publish-order

Raw output lines used above:

    E7RESULT name=tally arm=b_all mode=validating states=72 violations=0 state_new=2 state_old=70 record_holes=0
    E7RESULT name=tally arm=b_all mode=naive      states=72 violations=0 state_new=2 state_old=70 record_holes=0
    E7RESULT name=tally arm=b_ur  mode=validating states=79 violations=0 state_new=9 state_old=70 record_holes=7
    E7RESULT name=tally arm=b_ur  mode=naive      states=79 violations=0 state_new=9 state_old=70 record_holes=7
    E7RESULT name=tally arm=b_rs  mode=validating states=513 violations=0  state_new=2 state_old=511 record_holes=0
    E7RESULT name=tally arm=b_rs  mode=naive      states=513 violations=63 state_new=2 state_old=448 record_holes=0
    E7RESULT name=tally arm=b_none mode=validating states=1024 violations=504 state_new=9 state_old=511 record_holes=448
    E7RESULT name=tally arm=b_none mode=naive      states=1024 violations=567 state_new=9 state_old=448 record_holes=448
    E7RESULT name=verdict safe_arms_validating=b_all,b_ur,b_rs

Other facts were read directly from the repository and can be rechecked with:

    grep -rn "format-const:" .claude/kb/
    sed -n '899,903p' .claude/kb/decisions/23-journal的角色与格式.md
    sed -n '526,531p' .claude/kb/decisions/23-journal的角色与格式.md
    grep -n "I-7.3" .claude/kb/invariants.md
    grep -n "已定项 8" .claude/kb/decisions/22-单元原子性怎么合成.md
    grep -n "宽度随实现定" .claude/kb/decisions/18-块里携带什么信息.md
