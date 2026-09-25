This is an arithmetic filling task about a filesystem design question. You are not being asked to invent a design or to judge which design is best. You are given a fixed set of numeric facts and a fixed set of formulas. Your job is to substitute the numbers into the formulas and report the results, one cell at a time, showing the arithmetic step for every cell. Do not skip a cell and do not answer a cell with only yes or no; every yes/no cell must be paired with the number that produced it.

Formatting rules for your answer:
1. Do not use markdown emphasis (no bold, no italics, no headings marked with number signs).
2. Number your top-level answers 1, 2, 3 (one per candidate design, defined below).
3. Do not cite any source-code line number or file line number in your answer. If you need to refer to one of the facts below, refer to it by its fact number (F1, F2, ...). If you need to refer to a row in one of the tables below, refer to it by its row label.
4. At the end of each of your three numbered answers, add one sentence stating what new fact (a different established byte width, a different device count, a different upper bound on concurrently-alive rollbacks) would overturn the numbers you filled in for that candidate.
5. Plain sentences and plain pipe tables are fine. Show your arithmetic (for example: 481 + 65 = 546, then 512 - 546 = -34) rather than only the final number.

Setting. This filesystem persists metadata in fixed-size structures on disk. After an administrator rolls a mounted instance back to an older root, the filesystem wants to keep a durable witness recording that a rollback happened, so that if that rollback instance's own root records later become unreadable, recovery can still tell a rollback took place and must not silently revert it. This round only asks about the arithmetic cost of three different places to put that witness. It does not ask you to decide which place is best, and it does not ask you to find new failure histories.

Fixed facts. Every fact below was read directly from the named file at the named line; treat each as given, do not question the sourcing.

F1. The system-configuration field table that is already committed today totals 481 bytes. Source: crates/singlefs-format/src/lib.rs line 190, constant SYSTEM_CONFIGURATION_BYTES = 481.
F2. The system-configuration slot is 4096 bytes. Source: crates/singlefs-format/src/lib.rs line 199, constant SYSTEM_CONFIGURATION_SLOT_BYTES = 4096.
F3. Two candidate widths for a single witness entry have been measured: 16 bytes and 12 bytes. Source: crates/singlefs-harness/src/bin/e158_root_choice_repair.rs, function report_witness_width_and_slot_overlap_arithmetic, local variables single_16 and single_12; also .claude/kb/experiments/158-择根与修复四岔路.md line 15.
F4. A witness table holding n entries of width w bytes, with a 1-byte leading count/header field, totals 1 + w * n bytes. Source: same function as F3, local variables table_16 and table_12, computed as 1 + 16 * n and 1 + 12 * n; also kb line 15 of F3.
F5. Across four tested geometries, three of them (named GEOMETRY_PRIMARY, S16, and a small-ring geometry) each need at most 3 concurrently-alive rollback witnesses at their peak; the fourth geometry (named S4, whose root ring has a smaller capacity of 12 slots, tested with an extra search depth of 4) needs at most 4 concurrently-alive rollback witnesses at its peak. Source: .claude/kb/experiments/158-择根与修复四岔路.md line 14, the sentence computing N_w = max(3,3,3,4) = 4.
F6. A second, separate change under discussion (identified as C331 candidate B) proposes adding one extra checkpoint-txg-typed field to the system-configuration slot; the established width for a checkpoint-txg-typed field in this format is 8 bytes. Source for the proposal: .claude/kb/checks-owed.md line 285, which names the candidate "system configuration carries txg". Source for the 8-byte width convention: crates/singlefs-format/src/lib.rs line 152, the field list for the root record, entry "checkpoint_txg 8". The same 8-byte figure is also used directly in crates/singlefs-harness/src/bin/e158_root_choice_repair.rs, function report_witness_width_and_slot_overlap_arithmetic, in the line computing 512 - 481 - 8 - width.
F7. On the machine where this format was measured, the physical block size of the NVMe device is 512 bytes. The alternative value 4096 bytes is the well-known physical sector size of a 4Kn-formatted device; this alternative value is general industry knowledge and is not itself a number recorded in this project's knowledge base. Source for 512: .claude/kb/decisions/20-承重面单元的原子性与自包含.md line 133.
F8. For a self-certifying structure in this format (the source line names three examples: root-ring slots, journal record headers, and the system-configuration slot; the general property they share, given elsewhere in the same decision, is that they are not protected by a parent pointer's checksum and must authenticate themselves), the tear-detection resolution equals whichever physical block size is detected at runtime, and this width must never be hardcoded into the format. A write no larger than one physical block lands as a single atomic physical write. A write larger than one physical block can be partially applied, but the partial application is still fully detectable, because the structure carries its own whole-structure checksum together with a generation number that is actually checked, and either one alone is not enough. Source: .claude/kb/decisions/20-承重面单元的原子性与自包含.md line 68 (the equals-runtime-detected-size sentence); .claude/kb/decisions/22-单元原子性怎么合成.md line 197 (the allowed-to-cross-512-and-still-detectable sentence, about the system-configuration slot specifically) and line 195 (the system-configuration slot's own self-certifying fields, which is also the source for F9 below).
F9. The system-configuration slot's own self-certifying overhead, which is the established pattern this project already uses, is an 8-byte generation number (named "slot generation number") plus a 32-byte whole-structure checksum (named "whole-slot checksum"), 40 bytes together. Source: .claude/kb/decisions/22-单元原子性怎么合成.md line 195. The 32-byte checksum width is corroborated by a second, independent self-certifying structure in this format, the root record, whose own self-certifying checksum field is also 32 bytes. Source: crates/singlefs-format/src/lib.rs line 152, field list for the root record, entry "自证校验和 32" (self-certifying checksum, 32 bytes).
F10. The first runnable version of this filesystem targets exactly 2 devices. Source: .claude/kb/decisions/02-RAID条带策略.md lines 148-150, the definition of already-settled item 9, "the first runnable target runs 2 devices".
F11. The system-configuration slot is already rewritten at the end of every publish, including the publish that a management rollback itself performs; a witness placed inside the system-configuration slot therefore does not need any write beyond that rewrite. Source: crates/singlefs-core/src/transaction.rs, function persist_the_root_then_rotate_the_system_configuration, defined at line 575 (its doc comment sits directly above it, starting at line 572), which is called from three separate publish-path functions (call sites at lines 643, 998, and 3869 of the same file); crates/singlefs-core/src/mount.rs, function mount_rollback (starting at line 1778), which always finishes by calling function establish_instance (starting at line 1478), and establish_instance always calls exactly one of those three publish-path functions.
F12. The system configuration is already read by trying slot 0 and then slot 1 on every device, at every mount, regardless of any witness feature; a witness placed inside the system-configuration slot therefore does not need any read beyond that existing read. Source: crates/singlefs-core/src/recovery.rs, function choose_system_configuration (starting at line 516), with the two reads at lines 525 (slot 0) and 540 (slot 1).

Three placement candidates for the witness.

Candidate A. Placed into the system-configuration slot, allowed to grow past 512 bytes; relies on the system configuration's already-existing whole-slot checksum and two-slot rotation to detect a torn write (this is what F8 and F9 describe already happening for the system-configuration slot itself). Source: research/prompts/_m2-witness-r1-background.md line 14, clause (a): "放进系统配置槽、越过 512 字节，靠系统配置已有的校验和与两槽轮换认撕裂".

Candidate B. An independent self-certifying unit, separate from the system-configuration slot, carrying its own generation number and its own whole-structure checksum (use the F9 widths: 8 bytes generation, 32 bytes checksum), with two-slot rotation of its own. Source: same background line 14, clause (b): "独立的自证单元，自己带校验和、两槽轮换".

Candidate C. Split into several pieces, each piece no larger than 512 bytes, each piece self-certifying on its own (each piece carries its own generation number and its own whole-piece checksum, same F9 widths). Source: same background line 14, clause (c): "拆成几片、每片不超过 512 字节、各自自证".

Formulas. Use these exactly; do not invent different ones.

Let w be the entry width (16 or 12, from F3). Let n be N_w (3 or 4, from F5). Let devs = 2 (from F10).

table_total(w, n) = 1 + w * n   (this is F4)

Candidate A:
width_A(w, n) = table_total(w, n)
occupied_A(w, n) = 481 + width_A(w, n)              (481 is F1)
occupied_A_with_B(w, n) = 481 + 8 + width_A(w, n)    (8 is F6, the C331 candidate B field)
past_512_A = yes if occupied_A > 512 else no ; margin_512_A = 512 - occupied_A
past_4096_A = yes if occupied_A > 4096 else no ; margin_4096_A = 4096 - occupied_A   (4096 is F2)
margin_512_with_B_A = 512 - occupied_A_with_B
extra_writes_per_rollback_A = 0     (this is F11, show it as "0, by F11")
extra_reads_per_mount_A = 0         (this is F12, show it as "0, by F12")
extra_disk_bytes_A(w, n) = width_A(w, n) * 2 * devs   (2 is the system configuration's own two rotating slots per device, already allocated; this counts the growth in payload, not a new slot)

Candidate B:
width_B(w, n) = 40 + table_total(w, n)     (40 is F9, generation 8 plus checksum 32)
past_512_B = yes if width_B > 512 else no ; margin_512_B = 512 - width_B
past_4096_B = yes if width_B > 4096 else no ; margin_4096_B = 4096 - width_B
margin_512_with_B_B = not applicable; candidate B is an independent structure and does not share any bytes with the 481-byte system-configuration field table, so adding the C331 candidate B field to the system-configuration slot does not change candidate B's own margin. State this as the answer for this cell, do not leave it blank.
extra_writes_per_rollback_B = 2 * devs   (2 is candidate B's own two rotating slots)
extra_reads_per_mount_B = 2 * devs
For each physical block size PB in {512, 4096} (F7): footprint_B(w, n, PB) = ceil(width_B(w, n) / PB) * PB ; extra_disk_bytes_B(w, n, PB) = footprint_B(w, n, PB) * 2 * devs

Candidate C:
per_piece_overhead = 40 + 1 = 41   (40 is F9; the extra 1 is a leading count/header byte inside each piece, same convention as F4's leading 1 byte)
usable_payload_per_piece = 512 - per_piece_overhead = 471
entries_per_piece(w) = floor(471 / w)
pieces_needed(w, n) = ceil(n / entries_per_piece(w))
width_C(w, n) = per_piece_overhead * pieces_needed(w, n) + w * n
past_512_C: check whether any single piece exceeds 512 bytes; by construction each piece holds at most entries_per_piece(w) entries plus per_piece_overhead, so this should always compute to no across this table; if you ever compute yes, say so explicitly, because it would mean pieces_needed(w, n) was computed wrong.
margin_512_C = 512 - (per_piece_overhead + w * (n - entries_per_piece(w) * (pieces_needed(w, n) - 1)))   (this is the margin of the last, possibly partial, piece; when pieces_needed(w, n) = 1, this simplifies to 512 - width_C(w, n))
past_4096_C = yes if width_C > 4096 else no ; margin_4096_C = 4096 - width_C
margin_512_with_B_C = not applicable, same reason as candidate B: candidate C does not share any bytes with the system-configuration field table.
extra_writes_per_rollback_C(w, n) = pieces_needed(w, n) * 2 * devs
extra_reads_per_mount_C(w, n) = pieces_needed(w, n) * 2 * devs
For each physical block size PB in {512, 4096}: extra_disk_bytes_C(w, n, PB) = ceil(width_C(w, n) / PB) * PB * 2 * devs

Worked examples, one per candidate, using w = 16 and n = 4. Follow this exact method for every other row; do not change the method.

Candidate A worked example (w=16, n=4): width_A = 1 + 16*4 = 65. occupied_A = 481 + 65 = 546. past_512_A: 546 > 512, yes. margin_512_A = 512 - 546 = -34. past_4096_A: 546 > 4096, no. margin_4096_A = 4096 - 546 = 3550. occupied_A_with_B = 481 + 8 + 65 = 554. margin_512_with_B_A = 512 - 554 = -42. extra_writes_per_rollback_A = 0. extra_reads_per_mount_A = 0. extra_disk_bytes_A = 65 * 2 * 2 = 260. (The -34 and -42 numbers in this worked example match numbers already produced by a prior, independently run experiment recorded in .claude/kb/experiments/158-择根与修复四岔路.md line 133 and line 918; this is given to you only so you can check your own arithmetic method against a known-correct pair of numbers, not as a fact to cite in your own answer.)

Candidate B worked example (w=16, n=4): width_B = 40 + (1 + 16*4) = 40 + 65 = 105. past_512_B: 105 > 512, no. margin_512_B = 512 - 105 = 407. past_4096_B: 105 > 4096, no. margin_4096_B = 4096 - 105 = 3991. extra_writes_per_rollback_B = 2 * 2 = 4. extra_reads_per_mount_B = 2 * 2 = 4. footprint_B at PB=512: ceil(105/512)*512 = 512; extra_disk_bytes_B(512) = 512 * 2 * 2 = 2048. footprint_B at PB=4096: ceil(105/4096)*4096 = 4096; extra_disk_bytes_B(4096) = 4096 * 2 * 2 = 16384.

Candidate C worked example (w=16, n=4): entries_per_piece(16) = floor(471/16) = 29. pieces_needed(16, 4) = ceil(4/29) = 1. width_C = 41*1 + 16*4 = 41 + 64 = 105. margin_512_C = 512 - 105 = 407 (pieces_needed is 1, so the simplified formula applies). past_4096_C: 105 > 4096, no. margin_4096_C = 4096 - 105 = 3991. extra_writes_per_rollback_C = 1 * 2 * 2 = 4. extra_reads_per_mount_C = 4. extra_disk_bytes_C at PB=512: ceil(105/512)*512*2*2 = 512*2*2 = 2048. extra_disk_bytes_C at PB=4096: ceil(105/4096)*4096*2*2 = 4096*2*2 = 16384.

Now answer three numbered questions, one per candidate. Use the exact method shown in that candidate's worked example above for every row.

Question 1. For candidate A, fill in the same set of cells as the worked example (width_A, occupied_A, past_512_A, margin_512_A, past_4096_A, margin_4096_A, occupied_A_with_B, margin_512_with_B_A, extra_writes_per_rollback_A, extra_reads_per_mount_A, extra_disk_bytes_A) for these three remaining rows: (w=16, n=3), (w=12, n=3), (w=12, n=4). The worked example already covers (w=16, n=4); repeat it in your answer too so all four rows are together in one table. End with the one-sentence falsification statement required by rule 4 above.

Question 2. For candidate B, fill in the same set of cells as the worked example (width_B, past_512_B, margin_512_B, past_4096_B, margin_4096_B, margin_512_with_B_B, extra_writes_per_rollback_B, extra_reads_per_mount_B, extra_disk_bytes_B at PB=512, extra_disk_bytes_B at PB=4096) for these three remaining rows: (w=16, n=3), (w=12, n=3), (w=12, n=4). Repeat the (w=16, n=4) worked-example row too so all four rows are together. End with the one-sentence falsification statement required by rule 4 above.

Question 3. For candidate C, fill in the same set of cells as the worked example (entries_per_piece, pieces_needed, width_C, margin_512_C, past_4096_C, margin_4096_C, margin_512_with_B_C, extra_writes_per_rollback_C, extra_reads_per_mount_C, extra_disk_bytes_C at PB=512, extra_disk_bytes_C at PB=4096) for these three remaining rows: (w=16, n=3), (w=12, n=3), (w=12, n=4). Note that entries_per_piece(w) only depends on w, not on n, so you only need to compute it once for w=16 and once for w=12. Repeat the (w=16, n=4) worked-example row too so all four rows are together. End with the one-sentence falsification statement required by rule 4 above.

Answer in English. Write out every arithmetic step; do not just state a final number without showing 481 + 65 = 546 style substitutions. Do not add a fourth candidate or a fourth question.
