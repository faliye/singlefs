Cost arithmetic attack on a proposed crash-state recording and checking pipeline for a filesystem checker. Answer every question by filling the numbered table cells with computed numbers. Do not write yes or no alone for any cell. Do not write source-code line numbers or file line numbers anywhere in your answer; refer to steps by their function name or by table row number only. For every numbered answer, add one sentence starting with "This would be falsified if" describing what observation would overturn that answer.

SECTION 0. BACKGROUND

A verification pipeline enumerates crash states of a filesystem. Three full enumeration streams exist today:

Stream P1 (file crash_enumeration_new_pool_file_creation_stream.rs): the segment length array is [2,2,1,2,2,1,2,24,2,1,2] (11 segments). Total full-enumeration state count is 16,777,260.

Stream P2 (file crash_enumeration_fixed_script_stream.rs): the segment length array has 78 segments and 477 writes total. Total full-enumeration state count through the final unmount is 1,662,648,564.

Stream P3 (file crash_enumeration_multi_record_publish_stream.rs): the segment length array is [2,2,1,2,2,1,2,24,2,1,2,28,4,1,2,30,6,1,2] (19 segments). Total full-enumeration state count is 1,358,954,634.

The sum of the three streams' totals is 16,777,260 + 1,662,648,564 + 1,358,954,634 = 3,038,380,458.

A design proposal ("A1") models every crash-recorded outcome as a node identified by: <code file name>::<verification target name>::<process step number>::<hash string>. The process step number is composed of: which path on the operation tree, which operation step, which segment, and which write within the segment, plus a subset index, ending at one state. Step numbers are only ever appended, never reordered or reclaimed.

The following are attributes, not part of the identity: the content hash of the bytes written by that step, the code digest at recording time, the verdict digest (a digest of which checker code judged the result), and the version table (the oracle's expected values). Re-recording and re-checking are decided only by whether an attribute changed:

If only the checker changes: only re-check happens (no re-record) -- because the checker-code digest attribute changed but the written-content-hash attribute did not.
If only the core changes: only the step whose content-hash attribute changed, plus every step below it in the same identity chain, gets re-recorded (and, since its output is new, also re-checked).
If neither changed (same code on a different machine, or a different operator): the identity is the same and the previously stored result is reused directly, with zero re-recording and zero re-checking.

Two points about A1 are undecided; this task explores both candidates for each:

Point 1, what the hash string hashes: candidate H1 is "the flow definition only" (this step's operation sequence and parameters, not the written bytes and not the code version); candidate H2 is "the written content" (the bytes actually written by this step); candidate H3 is "both of the above, concatenated".

Point 2, what happens to identity when a code file is renamed or moved: candidate R1 is "freeze at registration" (the identity keeps the file name it had at first registration; a separate registry records which file it lives in now); candidate R2 is "follow the current name" (renaming the file changes the identity of every step recorded under that file name).

SECTION 1. FACT TABLE: node states, stream P1/P2/P3 shared prefix (source file research/prompts/m3-prune-gpu-r1-facts-case-design.md, table "2.1 节点表")

Columns: node id, parent node id, operation, full-enumeration count expression (copied verbatim), which streams pass through this node.

| node | parent | operation | full-enumeration count (verbatim) | streams |
|---|---|---|---|---|
| T1 (line 72) | T0 | acquire slot 1, same process | 3 × 8 | P1, P2, P3 |
| T4 (line 75) | T3 | publish A, first file, product capacity | 3 × 16777227 (plus, at the end of P1 only, one fully-persisted state) | P1, P2, P3 |
| T5 (line 76) | T4 | publish B, overwrite txg 4 | 1 × 16777227 | P2 only |
| T6 (line 77) | T5 | process reopen: acquire slot 2 | 1 × 8 | P2 only |
| T7 (line 78) | T6 | write row txg 5 (18 unit writes) | 1 × 262155 | P2 only |
| T8 (line 79) | T7 | warm-up txg 6 (16 writes) | 1 × 65547 | P2 only |

SECTION 2. FACT TABLE: node states, stream P2 continued (same source and table)

| node | parent | operation | full-enumeration count (verbatim) | streams |
|---|---|---|---|---|
| T9 (line 80) | T8 | warm-up txg 7 (16 writes) | 1 × 65547 | P2 only |
| T10 (line 81) | T9 | publish C, overwrite txg 8 (24 writes) | 1 × 16777227 | P2 only |
| T11 (line 82) | T10 | publish D, roll forward to A, txg 9 (20 writes) | 1 × 1048587 | P2 only |
| T12 (line 83) | T11 | overwrite txg 10 (28 writes) | 1 × 268435467 | P2 only |
| T13-T16 (line 84) | chained after T12 | four more overwrites, txg 11 through 14 (28 writes each) | each 1 × 268435467 (four nodes, same count each) | P2 only |
| T17 (line 85) | T16 | raise floor to 11 (write system configuration, empty publishes txg 15, 16) | 1 × 131102 (8 + 65547 + 65547) | P2 only |

SECTION 3. FACT TABLE: node states, end of P2 and branch of P3 (same source and table)

| node | parent | operation | full-enumeration count (verbatim) | streams |
|---|---|---|---|---|
| T18 (line 86) | T17 | publish E, overwrite txg 17 (28 writes) | 1 × 268435467 | P2 only |
| T19 (line 87) | T18 | normal unmount (write floor=17, empty publishes txg 18, 19) | 1 × 131102 (plus one fully-persisted state) | P2 only |
| T32 (line 92) | T4 | publish B: sequential write of two data units, two records, txg 4 | 1 × 268435479 | P3 only |
| T33 (line 93) | T32 | publish C: sequential write of three data units, three records, txg 5 | 1 × 1073741895 (plus one fully-persisted state) | P3 only |

SECTION 4. FACT TABLE: GPU cards, VRAM total in MiB (source research/prompts/e161-preregistration.md line 172, entry 4.6; the two home-machine readings are measured with nvidia-smi on 2026-09-27, the second machine's three readings are relayed by the lead agent and not independently measured)

| card | host | vram_total_mib | measured or relayed |
|---|---|---|---|
| RTX 5090 | home machine | 32607 | measured |
| RTX 5060 Ti | home machine | 16311 | measured |
| RTX 5080 | second machine | 16303 | relayed, not independently measured |
| RTX 5060 Ti (first of two) | second machine | 16311 | relayed, not independently measured |
| RTX 5060 Ti (second of two) | second machine | 16311 | relayed, not independently measured |

SECTION 5. FACT TABLE: raw per-item verdict count for one crash state (three sources)

| item group | count | source |
|---|---|---|
| pool-level checker invariants | 49 | crates/singlefs-checker/src/image.rs, array IMPLEMENTED_INVARIANTS, line 68 |
| oracle passes (consult journal / ignore journal) | 2 | crates/singlefs-checker-tier/src/crash.rs, function classified_oracle_violation_for_versions called at lines 1168 and 1190 |
| record checker flags | 2 | crates/singlefs-checker-tier/src/crash.rs, constants RECORD_CHECKER_ROOT_WITHOUT_RECORD (line 485) and RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT (line 487), booleans set at lines 143 and 159 |

SECTION 6. FACT TABLE: E161 measured per-state elapsed time in nanoseconds, two cells (source research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out, lines quoted verbatim; threads=10, one machine, other load present, so treat as order-of-magnitude reference not a clean benchmark)

Line 48, cell first_segment_head, 4096 states: state_elapsed_ns=5361069162 recovery_consult_elapsed_ns=493604274 recovery_ignore_elapsed_ns=441067394 oracle_elapsed_ns=2454094 pool_checker_elapsed_ns=3967962616 record_checker_elapsed_ns=455165874

Line 147, cell second_segment_head, 4096 states: state_elapsed_ns=19150712953 recovery_consult_elapsed_ns=1114083742 recovery_ignore_elapsed_ns=988126998 oracle_elapsed_ns=39248714 pool_checker_elapsed_ns=16482335779 record_checker_elapsed_ns=525887270

SECTION 7. FACT TABLE: assumed KV block size (source research/prompts/m3-prune-gpu-r1-facts-kv.md line 200, marked as an assumption, not measured)

One KV block holds 2^16 = 65536 states.

SECTION 8. QUESTIONS

Q1. Under candidate H1 (hash the flow definition only), if the core code changes in a way that changes the written bytes produced by node T5 but does not change T5's operation sequence or parameters, does the identity hash string of T5 change? Answer yes or no, then explain in one sentence citing the attribute rule in section 0 whether re-recording still happens even if the identity string is unchanged.

Q2. Fill this table for "one core change hits node T1's step, table sections 1-3 supply every downstream node's own count within P1/P2/P3": for each of H1, H2, H3, compute the total number of states across P1+P2+P3 that must be re-recorded, and the total that must be re-checked. State the two numbers for each of the three rows.

| hash candidate | states re-recorded | states re-checked |
|---|---|---|
| H1 | ? | ? |
| H2 | ? | ? |
| H3 | ? | ? |

Q3. Fill this table for "one core change hits node T5's step" (P2 only, using sections 1-3's own-counts for T5 through T19, remembering T13-T16 is four nodes with the same count each, and T20/T21 which are not listed here contribute their own count of zero from the source table): for each of H1, H2, H3, compute the total P2 states re-recorded and re-checked (sum of T5's own count plus every downstream node's own count through T19).

| hash candidate | states re-recorded | states re-checked |
|---|---|---|
| H1 | ? | ? |
| H2 | ? | ? |
| H3 | ? | ? |

Q4. Fill this table for "one core change hits node T18's step only" (P2 only, sum of T18 and T19's own counts): for each of H1, H2, H3, compute the total P2 states re-recorded and re-checked.

| hash candidate | states re-recorded | states re-checked |
|---|---|---|
| H1 | ? | ? |
| H2 | ? | ? |
| H3 | ? | ? |

Q5. One checker-only change (no core change) touches code used by every state's checking step. Using the combined total 3,038,380,458 from section 0, how many states need re-recording and how many need re-checking? Does the answer depend on which of H1/H2/H3 is chosen, or on which node the change conceptually belongs to? Answer with two numbers and one sentence.

Q6. One file rename touches the file of stream P2 only (crash_enumeration_fixed_script_stream.rs is renamed, no byte of any operation changes). Fill this table for candidates R1 and R2, giving the P2 states re-recorded and re-checked:

| rename candidate | states re-recorded | states re-checked |
|---|---|---|
| R1 (freeze at registration) | ? | ? |
| R2 (follow current name) | ? | ? |

Q7. Using section 4's five VRAM readings and section 5's raw per-item count (49+2+2=53 items), compute two encodings of per-state raw verdict bytes: encoding B1 is one byte per item (53 bytes per state); encoding B2 is one bit per item packed to the nearest byte (ceil(53/8) bytes per state). For each of the five cards, and each encoding, compute how many states fit in one batch (floor(vram_total_mib × 1048576 / bytes_per_state)), and how many 65536-state blocks that is (batch_states / 65536, give both the whole number of blocks and the remainder in states). Fill this table (10 rows, one per card per encoding):

| card | encoding | bytes_per_state | states_per_batch | whole_blocks_of_65536 | remainder_states |
|---|---|---|---|---|---|
| RTX 5090 | B1 | ? | ? | ? | ? |
| RTX 5090 | B2 | ? | ? | ? | ? |
| RTX 5060 Ti (home) | B1 | ? | ? | ? | ? |
| RTX 5060 Ti (home) | B2 | ? | ? | ? | ? |
| RTX 5080 | B1 | ? | ? | ? | ? |
| RTX 5080 | B2 | ? | ? | ? | ? |
| RTX 5060 Ti (second, first card) | B1 | ? | ? | ? | ? |
| RTX 5060 Ti (second, first card) | B2 | ? | ? | ? | ? |
| RTX 5060 Ti (second, second card) | B1 | ? | ? | ? | ? |
| RTX 5060 Ti (second, second card) | B2 | ? | ? | ? | ? |

Q8. Using section 6's two measured cells, define "recording time per state" as recovery_consult_elapsed_ns plus recovery_ignore_elapsed_ns divided by 4096 states, and "checking time per state" as oracle_elapsed_ns plus pool_checker_elapsed_ns plus record_checker_elapsed_ns divided by 4096 states. State explicitly that this split is one candidate definition of "recording" versus "checking", not a measurement of two separate stages -- today's pipeline runs both fused in one pass. Compute, for each of the two cells (first_segment_head, second_segment_head):

| cell | recording_ns_per_state | checking_ns_per_state | checking/recording ratio |
|---|---|---|---|
| first_segment_head | ? | ? | ? |
| second_segment_head | ? | ? | ? |

Q9. Using the RTX 5090's B1-encoding batch size from Q7 as "one GPU batch", and the second_segment_head recording rate from Q8 (states per second = 1e9 / recording_ns_per_state), compute how many seconds of recording it takes to fill one full GPU batch. Then propose a numeric backlog cap expressed as a multiple of one batch (for example "2.5 batches") such that the GPU is not left idle waiting for a batch to fill, and the unchecked backlog never exceeds a small bounded multiple of one batch; state your chosen multiple and the two competing failure modes it is trading off between (GPU idling versus unbounded backlog growth), and state explicitly that no measured GPU checking throughput exists yet -- your checking-side numbers rest on the CPU-only split defined in Q8, not on a GPU measurement.

Q10. List every node in sections 1-3 whose own full-enumeration count is exactly zero or is not given a definite number in section 0-3 (there should be none among T1, T4-T19, T32, T33 listed here; confirm this by re-scanning the tables, or name any you find).
