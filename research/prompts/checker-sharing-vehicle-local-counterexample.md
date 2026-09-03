You are reviewing two engineering decisions for a copy-on-write filesystem built from scratch in
Rust. Your assigned stance: find counterexamples. Do not argue for or against; only try to break
the proposals with concrete scenarios.

Reply in English. Do not use any markdown emphasis such as asterisks or bold anywhere in your
reply. Plain sentences and simple numbered lists only.

Read the background first. Facts are labeled F1 to F12 and were verified by the requester.
P1 and P2 are the proposals under review, not facts.

--- BACKGROUND START ---
F1. D13 (settled): the checker O2 is a unary predicate over one image; it "must not call any
    deserialization or traversal code of the crate under test, must reimplement parsing, and reads
    the raw device directly". D13 conflict 1 (settled compromise): inside the checker there are two
    judging directions, traversal from the root and linear scan by unit header, which "share only
    format parsing". File: decisions/13-验证路线.md.

F2. D17 debt item 5 (settled): the checker must not link any pipeline's runtime accounting or
    checksum code, because "auditor and audited must not use the same code". C12 (checks-owed):
    the set of functions the checker calls to rebuild accounting and the set the runtime commit path
    calls must have empty intersection, with the explicit exception "format parsing and constants
    excepted"; plus a fault-injection self-test: negate one runtime accounting branch, I-3.1 must go
    red. Files: decisions/17-实现分层与第三方管道.md, checks-owed.md.

F3. Gate stage 27 (format constants): the kb carries machine-readable markers of the form
    <!-- format-const: NAME = value stale=old1|old2 --> next to the sentence that settles the value;
    the gate checks that experiment sources use the current value and that stale literals do not
    reappear in kb prose or sources. Exactly three constants are registered in the whole repository:
    DATA_UNIT_BYTES = 32768, NODE_BYTES = 16384, JOURNAL_HDR = 78. Every other settled width (91-byte
    unit header, 31-byte pointer head, 11-byte location entry, 121-byte root record, 56-byte named
    item, 4096-byte record, 42-byte common prefix) is unregistered. The gate's own header says it
    only covers registered constants; unregistered ones can drift silently.

F4. The kb itself is inconsistent on one registered constant: D23 says the journal header is 78
    bytes (registered), says elsewhere that two settled-but-unlanded increments make it 91, and
    elsewhere that three make it 95. The five-tuple in the unit header is settled as 33 bytes total
    with no per-field widths anywhere in the kb. Both are open kb defects reported to the user today.

F5. Shared rule machine-first (settled project rule): duplicated code is allowed only if it is
    generated, never hand-copied; if generated duplicates and their generator ever disagree, the DRY
    relaxation is withdrawn. Shared rule fs-design: different address spaces must be different Rust
    newtypes (logical address, physical address, device offset, generation, inode number) so misuse
    fails to compile.

F6. C48 (checks-owed): "three code-independent paths agreeing is not evidence when they share a
    wrong premise"; experiments must list what their paths share (constants, rules, spec) and ask for
    each shared item whether an error there makes all paths wrong together.

F7. Today's adversarial review of the build plan (research/prompts/verification-build-reverse-opus-
    output.md) constructed: with a shared constants file, JOURNAL_HDR = 78 wrong in the kb makes
    core and checker agree and every planned check stays green; forbidding the shared file would not
    catch that (both would read the same kb sentence), but it would catch an underspecified split
    such as the five-tuple, because two independent inventions of the split disagree loudly. It also
    showed that the runtime-probed physical_block_size (I-7.5, I-8.2 depend on it) is not in the
    image; if the checker takes it from the harness or the implementation, it is auditing a value
    supplied by the audited party. D22 item 2 (settled): mkfs divides by the pool's max
    physical_block_size and mount recomputes per device and refuses on mismatch.

F8. D7 (settled): the project does not go to Linux mainline for the first years. D17 (settled
    waistline): media pipelines below, transaction state machine and crash semantics above, owned
    by this project. Project CLAUDE.md: the first runnable target is "correctly commit one
    transaction", explicitly without directories; fs-design rule: "start from the transaction, not
    from features", "when a criterion cannot be written, build only one path and build the slow
    correct one first".

F9. D13 item 4 (settled today): crash-point replay records writes in userspace on the block-device
    abstraction (write, flush barrier, FUA write), enumerates whole-write subsets per barrier segment,
    materializes images and runs recovery, checker and record verifier per state. D13 item 3
    (settled): O1 becomes a full RefFS handling persistence and crash semantics; schedule undefined.
    Differential testing drives the same operation sequence into RefFS and the implementation and
    compares results; nothing in the kb requires a mounted filesystem for that.

F10. Environment (verified): host kernel 6.17.0-1028-oem; the VM kernel 6.17.0-lockdep has
    CONFIG_FUSE_FS=y, ships ublk_drv.ko and nbd.ko, and the VM harness (research/scripts/vm-bench.sh)
    boots a busybox initramfs, passes device paths such as /dev/vda to a statically linked musl
    binary and captures E7RESULT lines with a count-integrity gate; no kernel modules are copied into
    the initramfs today. Host has libfuse3 runtime and fusermount3 but no libfuse3 development
    headers. Nine existing experiment binaries open files or devices with O_DIRECT from userspace and
    one proves fdatasync reaches the device (E44). VM timings are not comparable to the host
    (lockdep kernel); counts are.

F11. Prior art in the kb (facts only, not arguments): btrfsutils (Rust) has a read-only FUSE
    mount; TableFS (FUSE, self-described inefficient) still beat ext4/XFS/btrfs on metadata-heavy
    loads by 50 to 1000 percent; Bento (FAST 2021) runs safe-Rust filesystems in the kernel with
    hot upgrade and is 7x to 90x faster than its FUSE version. None of this was measured in this
    project.

F12. The gate's Show-me-test stage recognizes implementation code only under crates/*/src/; the
    repository has no crates/ and no root Cargo.toml today. The research workspace lives under
    research/ and is built and unit-tested by a separate gate stage.

--- PROPOSALS UNDER REVIEW ---

P1 (answer to D13 open item 5). The checker crate and the core crate share exactly one artifact:
    a constants module generated from the kb's field tables by a generator script (kb to Rust),
    consumed by both and never hand-edited; gate stage 27 is extended so every width in
    first-txn-layout.md is a registered format constant, and the generated module is regenerated
    and diffed by the gate. Nothing else is shared: no types (each crate has its own newtypes), no
    parsing, no traversal, no checksum implementation (the checker uses an independent CRC32C
    implementation or a different crate), no accounting code. Runtime-probed values, in particular
    physical_block_size, are never passed from harness or core to the checker: the checker probes
    the device itself and compares with the value the superblock recorded; on a file-backed image
    it reports the geometry as "declared, not probed". Any field whose sub-widths are not in the kb
    (the five-tuple split) must be pinned in the kb before generation; the generator refuses to emit
    a constant that has no kb marker.

P2 (answer to D17 open item 5). The first version is a pure userspace Rust library over the
    block-device trait, with two backends: a regular file and a raw block device opened with
    O_DIRECT. It exposes a small operation API (mkfs, open, write extent, publish/fsync, recover)
    that the crash-point replay harness, the RefFS differential tester and the QEMU-tier workload
    binary all drive directly, the way today's experiment binaries drive their models. No mount
    exists in the first version; a FUSE adapter is a later thin layer added only after directories
    and the POSIX surface exist, and a kernel module is out of scope while D7 holds. The vehicle
    question is therefore answered as "library first, FUSE later, kernel never for now", and it does
    not change any on-disk byte.
--- BACKGROUND END ---

Your task, in order:

1. Construct at most three concrete implementation bugs that survive P1's sharing rule: the core
   writes an image, the checker built under P1 reports every invariant green, yet the image is
   wrong. For each give the bug in one sentence, the bytes involved with small concrete numbers,
   and why the checker stays green. Say whether forbidding all sharing would have caught it.

2. Construct at most two concrete scenarios where P2's library-only vehicle makes a bug in the
   first transaction (F8) unobservable by all three tools of F9 and F10, and say what vehicle would
   make it observable.

3. Check P1's rule that the checker probes physical_block_size itself: give a scenario where the
   checker's probe and mkfs's probe legitimately differ (same image, different host or different
   device path) and say what the checker should report so that it is neither a false red nor a
   silent pass.

4. End with one short paragraph: which counterexample is reachable rather than a cost, and what
   single repository check or experiment would most change your answer.
