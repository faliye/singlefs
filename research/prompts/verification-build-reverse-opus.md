You are reviewing an engineering plan for a copy-on-write filesystem built from scratch in Rust.
Your assigned stance: adversarial. Assume the proposal is wrong and find where. Your best output
is a concrete failure the plan's own self-tests would not catch, or a settled fact the plan
violates. A cost or an inconvenience is worth listing but must be labeled as such, not as a break.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Read the background first. Facts are labeled F1 to F16 and were verified against the repository
by the requester. P1 to P6 are the proposal under review, not facts.

--- BACKGROUND START ---
F1. Current state: the repository has zero filesystem implementation code. There is no
    crates/ directory and no root Cargo.toml. research/ holds about 90 experiment binaries
    that are arithmetic or simulation models only. The gate prints three unimplemented
    stages every run: model-based differential testing, crash-point replay, QEMU real
    workload. Files: CLAUDE.md, .claude/singlefs-ai-sop/scripts/gate.sh.

F2. D13 (verification route, settled part): three independent oracles. O1 is an in-memory
    reference model (D13 item 3 later upgraded it to a full RefFS that handles persistence
    and crash semantics itself, schedule undefined). O2 is an independent parser plus
    checker: a unary predicate P(image) over ONE image; it must not call any
    deserialization or traversal code of the crate under test. O3 is an independent
    specification executor written in another language. A separate two-input "record
    verifier" takes (pre-crash image, record stream, post-crash image); it deliberately has
    no number. File: decisions/13-验证路线.md.

F3. D13 (settled): crash-point replay has three tiers. Tier 0: fixed-seed minimal workload
    with tens of write requests, NO sampling, exhaustive, run on any commit touching the
    write path. Tier 1: thousands of writes, stratified sampling by commit sub-phase.
    Tier 2: nightly, no sampling. The tier-1 wall-clock budget T1 is declared an input,
    but D13 also says the numbers C, K_max, T1 must not be written into the kb yet because
    the prerequisite chain (transaction layer, block-level write recording, harness, mkfs
    layout switches, N checkers) does not exist. File: decisions/13-验证路线.md.

F4. D13 (settled): the commit protocol is a single shared transaction state machine plus a
    closed enum CommitStep with an exhaustive match and no wildcard arm. D17 (settled
    waistline): media pipelines live below the waistline; the state machine, the step set
    and the crash semantics stay above it and belong to this project. C19 (checks-owed):
    no media or layout predicate symbol may appear in the transaction layer or admission
    path. Files: decisions/13-验证路线.md, decisions/17-实现分层与第三方管道.md, checks-owed.md.

F5. D13 item 2 (settled, user decision 2026-08-31): commits are serial in the first version.
    D12 item 5 (settled): the first layout line is pure SSD. D2 item 9 (settled): the first
    version runs on 2 disks. D9 item 10 (settled): encryption is not in the first runnable
    version, only its format fields are reserved.

F6. first-txn-layout.md (2026-09-03): "the first transaction" means mkfs, then write one
    small file, then publish once. No directories, snapshots, reflink, compression,
    encryption. Every byte segment points to a decision item. Four open items are judged
    "changes the bytes of the first transaction: yes": D22 open item 9 (superblock field
    table), D8 open item 6 (inode on-disk field table), D3 open item 7 (allocation record
    entry width and key encoding), D5 open item 5 (accounting key segment widths). Five
    more segments have a settled shape but no settled width (superblock slot rotation
    fields, birth txg inlined in pointers, accounting node header base bytes, journal
    transaction boundary field, tree-table unit header).

F7. Settled format facts the first version must follow: data unit is always 32 KiB
    including header (D4 items 5 and 7); unit header is a 91-byte field table for data
    units, with a 42-byte common plaintext prefix (D18 item 7); padding after the declared
    length is all zeros and covered by the checksum (invariant I-2.3); pointer = 31-byte
    head + two 11-byte location entries (D19); child checksum lives in the parent pointer,
    Merkle style (D4 item 1); journal record is 4 KiB fixed, header 78 bytes, 56 bytes per
    named item, one transaction may span several records and the header carries a
    transaction boundary field, back-chain CRC32C, payload checksum (D23 items 4, 7, 8,
    11, 12, 13); root record is 121 bytes = magic 4 + fsid 16 + flags 4 + instance epoch 4
    + checkpoint_txg 8 + tree-table unit pointer 53 + self checksum 32, always indirect
    through a tree-table unit (D22 item 7); root ring has R = 3 regions, prime-stride
    placement, slot width = runtime-probed physical_block_size, mkfs seeds generation 0
    into every region (D22 items 2 and 8); superblock one copy per disk with >= 2 slot
    rotation; journal ring mirrored on two disks (D22 item 8).

F8. D16 item 7 (settled 2026-09-02): the persistence order of one publication is fixed:
    COW units/nodes, then FLUSH barrier, then journal records, then FLUSH barrier, then
    root slot written with FUA; fsync returns only after the root slot is durable; recovery
    must validate the checksum of every named unit before applying any record. D16 item 6
    (settled): every publication increments checkpoint_txg by 1, fsync-triggered ones
    included; the same counter is the root-ring rotation key, the accounting generation
    and the replay watermark.

F9. D23 item 14 (settled 2026-09-02): recovery applies only records whose
    (instance epoch, checkpoint_txg) is strictly greater than the chosen root's; the four
    prefix conditions are: jsn strictly consecutive (stop at the first gap), watermark
    above the root, commit marker present for the transaction, per-item validation of
    named units before applying. D23 item 3 (settled): recovery must scan the whole ring
    and validate every record; it must not trust the persisted tail. D22 item 7: all root
    slot candidates are validated first, then the newest is chosen; ties on checkpoint_txg
    are broken by the higher instance epoch.

F10. E77 (model experiment, 2026-09-02): crash-state model = barriers cut the write stream
    into segments; a crash state is "all earlier segments durable + any subset of the
    current segment durable + nothing later". Each write is atomic in that model (no torn
    writes). With one publication of 6 units + 3 records + 1 root slot the state counts
    are: both barriers 72, only unit barrier 79, only root barrier 513, no barrier 1024.
    Without the barrier before the root slot, 504 of 1024 states violate; a replay that
    does not validate named units silently grafts garbage in 63 states of the b_rs arm.
    The experiment code contains a state enumerator, a recovery model and an independent
    audit function that shares no code with recovery. File:
    research/e7-index-bench/src/bin/e77_publish_order.rs.

F11. D20 (settled): units that have a parent pointer do not depend on any atomic write
    width at all, because the parent pointer holds their checksum and the parent is
    published after the child is durable. Only self-certifying units (root slot, journal
    record header, superblock slot) depend on a width, and that width is the runtime
    probed physical_block_size, never a hard-coded constant. Device-side atomic width was
    measured as 512 and is not to be relied upon. dm-flakey cannot produce torn writes
    (its abilities are drop_writes, error_writes, corrupt_bio_byte). D17 open item 1
    ("how the failure model is written as a contract") was deferred on the grounds that
    there is no third-party pipeline yet and no code.

F12. Environment (verified 2026-09-03): cargo 1.98.0, rustc 1.98.0, musl target
    installed; QEMU 8.2.2 with KVM readable and writable; herd7 7.58 with a kernel tree;
    host kernel 6.17.0-1028-oem has dm-log-writes and dm-flakey as loadable modules; the
    VM kernel 6.17.0-lockdep has CONFIG_BLK_DEV_DM=y and ships dm-log-writes.ko,
    dm-flakey.ko, nbd.ko, ublk_drv.ko, with FUSE built in; the VM harness initramfs
    currently contains only busybox and the test binary, no kernel modules. VM timings
    are not comparable to the host (lockdep kernel); counts are.

F13. Checks-owed entries that name the crash-replay harness as their prerequisite and
    prescribe a must-go-red self-test: C76 (remove the barrier before the root slot, must
    go red; disable per-item validation in replay, must go red), C42 (gap, recover, write
    a little, crash again: replaying a record of the discarded timeline must go red),
    C22 (set the delayed-reuse window to 0: invariant I-4.8 must go red), C80 (remove the
    accounting flush from the publish path: I-3.1 after each publish point must go red),
    C77 (stale tail plus reused blocks: recovery must complete and final state must equal
    the truth), C29 (a recovery that trusts the tail first must lose records), C6 (in QEMU
    with real virtio-blk, dropping one FUA must make the crash test go red). C13: a
    checker that runs green but no longer decides is indistinguishable from a working
    one, so each layout's checker must be run against a known-bad image in the same
    round. C12: the checker's accounting rebuild must not share functions with the
    runtime's accounting update. File: checks-owed.md.

F14. invariants.md has 43 live invariants, all with checker status "unimplemented". The
    encryption class I-6 must be reported as "not applicable" when encryption is off, never
    silently passed. I-3.1 (allocated statistics equal the traversal sum) is the only
    accounting check allowed to traverse the whole disk, and the runtime must use a
    different algorithm. The checker must judge in two directions, traversal from the
    root and linear scan by unit header, sharing only format parsing (D13 conflict 1).

F15. The kb nowhere decides the runtime vehicle of the first implementation (pure
    userspace library over a block-device abstraction, FUSE, or kernel module). D7 only
    says the project does not go to Linux mainline for the first years.

F16. D3 open item 7 (added 2026-09-03) is titled "allocation record carrier form" and
    lists "which tree, key encoding, entry width" as open, while D3 settled item 3
    (2026-09-01) already settled "a btree in its own keyspace, key = (device id, offset),
    value = allocation generation". Only entry width and key segment encoding widths are
    actually still open. Another session is editing D3 today.

--- PROPOSAL UNDER REVIEW (P1 to P6) ---

P1. Code layout: a root Cargo workspace with separate crates: singlefs-core (transaction
    layer, mkfs, recovery, block-device abstraction), singlefs-check (independent parser
    and checker, two directions, no dependency on singlefs-core), singlefs-crash (recorder
    over the block-device abstraction, crash-state enumerator lifted from E77, driver that
    runs recovery, checker and record verifier per crash state), singlefs-verify (the
    two-input record verifier). The checker may share with the core only a generated
    constants file (magic values, widths, offsets) that the existing gate keeps in sync
    with the kb; no executable code and no types.

P2. Crash-point replay records in userspace: every I/O of the implementation goes through
    one block-device trait exposing only write, flush barrier and FUA write; a recording
    wrapper captures the ordered stream. dm-log-writes is reserved for the C6 question
    (are real FLUSH and FUA semantics what we assume) in the QEMU tier, which requires
    adding module files to the harness initramfs.

P3. Crash-state model for tier 0: E77's segment model, plus torn-write enumeration only
    for writes larger than the probed physical_block_size and only for self-certifying
    units (root slot, journal record header, superblock slot); for units with a parent
    pointer, torn sub-states are collapsed to "not durable" because any torn content
    fails the parent checksum. Tier 0 enumerates exhaustively.

P4. Each crash state runs three independent judges: the implementation's own recovery,
    then the checker (unary), then the record verifier (binary). The five harness
    self-tests of F13 (C76 x2, C42, C22, C80) are required to go red before the harness
    result counts.

P5. First-version checker scope: judges I-1.1 to I-1.5, I-2.1 to I-2.3, I-3.1, I-5.1,
    I-5.2, I-7.1 to I-7.6, I-8.1 to I-8.5, and after replay I-4.1, I-4.2, I-4.3, I-4.8;
    reports "not applicable" for I-3.4 to I-3.7 (no snapshots or intents), I-4.4, I-4.6,
    I-4.7 (no intents) and the whole I-6 class (encryption off). Each implemented invariant
    ships with at least one known-bad image that triggers it, kept as a mutation table that
    the gate replays.

P6. Build order: step 0 settle the four byte gaps of F6 plus two new open items (the
    crash-state model of P3 as D13 open item 4; the allowed sharing set of P1 as D13 open
    item 5) and register the vehicle question of F15 as D17 open item 5; step 1 the
    format-independent skeletons (workspace, block-device trait and recorder, enumerator,
    checker two-direction skeleton with three-state reporting and the mutation-table
    mechanism, gate stages that report per-invariant "unimplemented"); step 2 two
    independent parsers for the self-certifying units (root record 121 bytes, journal
    header 78 bytes); step 3 mkfs plus an empty publication (no data) with tier-0 replay
    proving "barrier removed must go red"; step 4 the first real transaction (data unit,
    allocation tree, accounting tree, inode, tree-table unit) with all five self-tests red
    then green; step 5 record verifier, minimal RefFS, differential-testing gate stage.
--- BACKGROUND END ---

Your task, in order:

1. For each of P1 to P6, give the strongest concrete way it fails. Prefer this shape: a specific
   implementation bug, the crash state or image that exposes it, and why each of the planned
   judges (recovery, checker, record verifier) and each of the planned self-tests (F13) still
   passes. Small concrete numbers, step by step.

2. Attack P1's sharing rule in particular: a generated constants file shared by core and checker.
   Construct one wrong constant that makes both sides agree and every planned check stay green.
   Then say whether forbidding the shared file would have caught it, or whether the blind spot
   is elsewhere.

3. Attack P2's claim that a userspace recorder over the block-device trait is enough for tier 0.
   What does a userspace recorder structurally not observe that a real block layer would, and
   does any of that matter for the first transaction as defined in F6?

4. Attack P3's crash-state model. Using F10 and F11, find a crash state that a real device can
   produce and the model cannot enumerate, or show that the model over-approximates in a way
   that makes tier 0 report violations that cannot happen. Concrete example with a 32 KiB unit,
   a 4 KiB journal record and a probed physical block size of 512 or 4096.

5. Attack P6's build order: find a step whose success would be meaningless because a later step
   changes the bytes it tested, or a step that cannot actually be started when the plan says it
   can.

6. End with one short paragraph: which single attack is a reachable break rather than a cost,
   and what one repository check or experiment would most change your answer.
