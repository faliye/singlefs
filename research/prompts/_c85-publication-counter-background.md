# Background: publication counter vs checkpoint_txg (C85 + C77), 2026-09-02

All facts below were verified against the singlefs kb on 2026-09-02 by the main agent
(file and item named per fact). Facts about other systems are labeled as such.

F1. D16 (publication semantics, settled): a checkpoint publishes a new root when
    T_time = 5 s or T_dirty = 2 GiB is reached, whichever first. Quote: "fsync 的效果是
    「提前触发一次发布」" (an fsync triggers a publication early). File: decisions/16-发布语义.md.

F2. D23 item 1 (settled): every fsync writes dirty leaves + all ancestors + a root slot
    + one journal record. File: decisions/23-journal的角色与格式.md.

F3. D22 item 2 (settled): root ring has R = 3 regions; slot order rotates across regions
    with "region = txg mod R". Invariant I-7.3 (settled, checker unimplemented): besides
    the newest self-valid root, at least one older-generation root must remain in the ring;
    a rotation that overwrites the previous generation is itself a red flag. Files:
    decisions/22-单元原子性怎么合成.md, invariants.md.

F4. D5 (settled): a block's birth is the checkpoint number in which it was published;
    snapshot S.txg is the last published txg captured by S; deadlist comparison granularity
    is the checkpoint granularity ("一个 txg 一个号", one number per txg, via the ZFS
    analogy quoted in D16). Accounting keys are (statistic, dimension tuple, generation)
    with generation = checkpoint number; only the last K generations are kept, and
    K = total root slots + 1 (experiment E54). Files: decisions/05-快照-空间记账机制.md,
    decisions/16-发布语义.md.

F5. D23 item 9 (settled): the journal record header carries jsn = 32-bit instance epoch
    + 48-bit counter (one increment per record), plus a separate 8-byte checkpoint_txg
    field. The stated reason jsn exists: multiple journal records within one checkpoint
    window share the same txg, so they need jsn to be ordered. File:
    decisions/23-journal的角色与格式.md.

F6. Machine measurement (E44): this machine sustains 2785 fsync/s (consumer NVMe,
    O_DIRECT, ext4 as an upper-bound proxy). Root ring total slots are on the order of
    tens (R = 3 regions, S = 1..16 slots per region, S is a superblock field).

F7. New model result E78 (2026-09-02): a recovery that replays from a stale tail and
    validates each named unit before applying aborts on perfectly healthy images, because
    the block-pinning window (I-7.4, at most a few dozen root generations) is far shorter
    than the journal ring (thousands of records), so old records legally point at reused
    blocks. Two fixes both passed all pre-registered criteria: (B) the root record carries
    a jsn watermark (10 bytes) and replay starts strictly after it; (C) blind-apply the
    whole continuous prefix, validate the final state, drop the trailing transaction and
    retry on failure. If and only if checkpoint_txg increments with every publication,
    the existing checkpoint_txg field in record headers can serve as the watermark
    (records with txg <= root.txg are known covered) and no new root field is needed.

F8. Nothing in the repo states whether an fsync-triggered publication increments
    checkpoint_txg. Four statements exist separately and have never been joined:
    fsync = early publication (F1); region = txg mod R (F3); one number per txg /
    deadlist granularity = checkpoint granularity (F4); accounting generation =
    checkpoint number (F4).

F9. Known tension, derived by the main agent (treat as a claim to check, not a fact):
    if txg does NOT increment per publication, two fsync publications inside one window
    compute the same root-ring region and slot (region = txg mod R), so the second
    overwrites the first while it is still the previous generation - violating I-7.3.
    If txg DOES increment per publication, generation-scoped structures tick at fsync
    rate (up to 2785/s on this machine) instead of once per 5 s window; whether any
    settled item is harmed by that has not been analyzed.

QUESTION. Should the design pin this single sentence: "every root publication increments
checkpoint_txg by 1 (fsync-triggered publications included); the same counter is the
root-ring rotation key and the accounting generation" - making the existing
checkpoint_txg field usable as the replay watermark (E78 fix B at zero new format bytes)?
Or should txg stay coarser than publications (one per timed window), with a separate
10-byte jsn watermark added to the root record instead?
