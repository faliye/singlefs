This is an arithmetic task about the cost of a design choice in a copy-on-write file system. Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings marked with asterisks. Give numbered answers. Do not write source-code line numbers or document file line numbers in your answer; when you cite a fact, name its row id only (for example P3, S2, C1, E4). For every numbered answer, add one line stating what fact or observation would overturn that answer.

Background. The system publishes new state in transactions called publishes. Each publish writes some units and nodes, one journal record, one root slot, and one system-configuration slot, in a fixed order (table P below). The system-configuration slot also holds the journal tail value and other fields (table C below). A design question is how often the system-configuration slot must be rewritten, and what that costs compared with an older alternative (a fixed-length journal ring versus a chained journal, table E below, from an experiment called E23). All facts below are given as fixed data; treat every table row as literally true and do not question it. Where a computation is not settled by the given rows, say so explicitly rather than inventing a missing fact.

Table P. Publish order and sequence points. Source: decisions/16-发布语义.md (file name given for your own bookkeeping only, do not put a line number in your answer).

P1. The persist order of one publish is always: COW units / nodes, then a barrier, then the journal record, then a barrier, then the root slot (FUA), then the system-configuration slot, then a barrier. fsync does not return until the system-configuration rotation is durable.
P2. The system-configuration slot is updated after the root slot is durable, once per checkpoint, and is the last step of the publish sequence.
P3. Barrier accounting: two FLUSHes, plus the root-slot FUA, plus one barrier after the system-configuration rotation, equal four sequence points per publish.
P4. Every publish increments a counter called checkpoint_txg by one. A publish triggered by fsync is also a publish; there is no exception for a small publish that goes unrecorded.
P5. Because of P4, "once per checkpoint" and "once per publish" are the same frequency: an fsync-triggered publish also writes the system-configuration slot.

Table S. Segment sequence of one ordinary publish (the publish that creates the first file in a new pool). Source: layout/01-first-txn.md.

S1. The segments of this publish are: twelve unit writes, each written to two disks, so 24 writes; then a barrier; then one journal record written to two disks, so 2 writes; then a barrier; then one root-slot FUA write, so 1 write; then one system-configuration-slot write to two disks, so 2 writes; then a barrier. The segment sizes in order are 24, 2, 1, 2.
S2. Totals for this publish: 35 operations in all (29 writes, plus 3 barriers, each barrier counted as two steps, one per device). Written as a kind string: unit_write times 24, barrier times 2, then journal_record times 2, barrier times 2, then root_record_fua times 1, then system_configuration_slot times 2, barrier times 2.
S3. Every write in this system is replicated on at least two disks: all units and records get one copy on each of two disks.
S4. The system-configuration-slot rotation write's content is 489 bytes, and it lives in a 4096-byte slot. In this publish it targets slot 1, generation number 5, tail value 3 (for background only, not part of this publish: instance acquisition earlier wrote slot 0 generation 2, and the two warm-up publishes each wrote once, first slot 1 generation 3, then slot 0 generation 4).

Table C. System-configuration slot content and write-tearing rule. Source: decisions/22-单元原子性怎么合成.md.

C1. The system-configuration field table has 46 rows and totals 489 bytes. It lives in a 4096-byte system-configuration slot; 3607 bytes of the slot are unused padding.
C2. Of the 489 bytes, 60 bytes are called the system runtime quantity, made of: slot generation number 8 bytes, whole-slot checksum 32 bytes, journal tail 8 bytes, instance number 4 bytes, rollback floor F 8 bytes.
C3. The 489 bytes of content are allowed to cross a 512-byte boundary (512 is the detected physical_block_size in this scenario). After crossing, one write may land only half torn. This is detected by a checksum that covers the entire 4096-byte slot including its padding. The slot-selection rule (checksum passes and generation number is largest) then automatically falls back to the other slot. The tear is called detectable and recoverable.

Table E. E23 journal-geometry experiment, the tail-location arm. Source: experiments/23-journal几何.md.

E1. Arm tail_system_configuration (lives in the journal superblock, overwritten in place, FUA write): steady-state extra write is 21 blocks, equal to the number of checkpoints in the workload below. Blocks to replay after an idle-period crash: 0.
E2. Arm tail_inline (inlined in every journal record header as a field called tail_lsn): steady-state extra write is 0 blocks. Blocks to replay after an idle-period crash: 500 blocks when the physical block size pbs is 4096, or 63 blocks when pbs is 512.
E3. The workload used for E1 and E2 is 20000 fsyncs, with one checkpoint every 1000 fsyncs. Under this workload, tail_system_configuration's steady-state overhead is stated as 0.105 percent, computed as 21 divided by 20000.
E4. Under a barrier-based accounting the conclusion is stated to be unchanged: 21 extra FUA writes against 40000 durability points (two durability points per fsync, over 20000 fsyncs) is stated as 0.05 percent.
E5. This is a pure counting model that does not touch the device: it counts blocks only, not bytes and not barriers. Two open cost-accounting debts (one about the block-count basis, one about barrier counts never having been measured on a real device) apply to this model.

Questions. Fill in the blank answer table below; do not answer any question with only yes or no.

Q1. How many of the 29 writes in one ordinary publish (S2) are system-configuration-slot writes, and what fraction (percent) of 29 is that? Show the arithmetic.

Q2. Of the four sequence points per publish (P3), how many are specifically for the system-configuration rotation (count both the rotation write itself and the final barrier that confirms it durable), and what fraction of 4 is that? Show the arithmetic.

Q3. How many bytes does the system-configuration rotation add to one ordinary publish, separately for pbs=512 and for pbs=4096? Consider two possible readings and say which one the given rows settle, if either: reading A, a slot write always writes all 4096 bytes of the slot regardless of pbs; reading B, a slot write only needs to cover the physical blocks that hold the 489 bytes of content, so it writes ceil(489 / pbs) times pbs bytes. Give both readings' numbers for pbs=512 and pbs=4096, and state which fact row, if any, decides between them.

Q4. Recompute E23's ratio (E1 vs E2) under E23's own stated workload (E3), separately for pbs=512 and pbs=4096. State the new ratio as two numbers (extra blocks for tail_system_configuration : extra blocks for tail_inline) for each pbs value, and state whether it differs from the 21:500 ratio that E1 and E2 already give for pbs=4096.

Q5. Now recompute the same ratio under the premise "every publish is a checkpoint" (P4, P5): the checkpoint frequency becomes one checkpoint per fsync instead of one per 1000 fsyncs, so over the same 20000 fsyncs there are 20000 checkpoints instead of 21. Using E1's rule (steady-state extra write equals the number of checkpoints), give the new steady-state extra-write count for tail_system_configuration. Then say whether tail_inline's blocks-to-replay-after-an-idle-crash number (500 for pbs=4096, 63 for pbs=512) is also expected to change under this premise, and why; if no given fact row settles this, say so explicitly rather than guessing a number. Give the resulting ratio, separately for pbs=512 and pbs=4096, or say that it cannot be computed from the given rows and name the missing fact.

Answer table to fill, one row per question:

Question | pbs=512 answer | pbs=4096 answer | formula (row ids used) | what would refute this answer
Q1 | | | |
Q2 | | | |
Q3 | | | |
Q4 | | | |
Q5 | | | |

For Q1 and Q2 the pbs=512 and pbs=4096 columns should be the same value if pbs does not enter the computation; say so in the formula column rather than leaving a cell blank.
