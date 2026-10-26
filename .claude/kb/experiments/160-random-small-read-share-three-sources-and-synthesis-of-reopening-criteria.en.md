## E160 Random Small Read Share: Three Sources and Synthesis of Reopening Criteria —— Partially Run (First Segment + Sample Points for Arm A at Real Weights, 2026-09-24, Deterministic, 25 Unit Tests / 19 Mutations All Caught)

[中文](160-随机小读在负载里占多少三个来源与两条重开判据的合成.md) · **English** · [日本語](160-ランダム小読み取り比率-3つの起源と再開判定基準の合成.ja.md)

The question is about the reopening criterion of settled item 1 / settled item 8 of D4 (Checksum Location) (device-side multiplier for random small reads ≥ 1.136) and the crossover point of settled item 5
(random page reads taking 1.0%–1.5% of user bytes). Both sit on the same axis, namely how much of the workload is random small reads, yet each side says "do not reopen on this basis"
and there is no synthesis rule (C357 (two reopening criteria on same axis never synthesized)). Three sources each give this share: **A** (the weights in the settled item 1 table of D25 (Target Workload Priority)),
**B** (publicly available reference workloads with citations), **C** (the job mix of E152 (Per-Milestone File Performance Against Six File Systems));
four synthesis rules: first-to-trigger / additive / criterion 1 only / criterion 5 only. Fork sheet `m2-d25-weights-forks.md`, rows 1 and 2.

**Only the first segment was run (pure arithmetic, no disk access).** The second segment (freshly measured actual device-side multiplier and with-header cost for random small reads on this machine) was not run;
whether to resume after the verdict is up to the main agent. **Continuation**: the user supplied the real weights of the D25 (Target Workload Priority) settled item 1 table
(seq 60, metaheavy 10, rand 10, multistream 10, smallfile 10), and the sample points of arm A under these weights were added,
substituted into the two sets of illustrative b already in the registration (M1, M2); the real share of A still cannot be answered (all five tiers of b are missing: neither settled item 1 of D25 (Target Workload Priority) nor the workload generators of E16 (Role of Journal: WAL vs Intent Log) and E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) yet have a way to read this dimension).

### Setup

`scripts/experiment-apparatus/e160_random_small_read_share.rs`: an independent hand-written model that shares no code with `crates/`
(item 4 of `.claude/rules/implementation-first.md`). Rational numbers (`RationalNumber`, i128 numerator and denominator) carry all arithmetic;
comparisons near the criterion thresholds do not use floating point. Registration before the run, `e160-preregistration.md` (criteria, arms, failure and void clauses
written before the setup).

### Results (artifact `e160-random-small-read-share-segment1-2026-09-24.out`, 70 lines, `emitted=70`;
the continuation adds `research/results/e160-random-small-read-share-segment1-2026-09-24-realweight.out`,
75 lines, `emitted=75`: the first 69 data lines are byte-identical to the old file, checked with `diff`; the 5 new lines come after the original data and before the `done` line.
The E160 (Random Small Read Share: Three Sources and Synthesis of Reopening Criteria) registration row in `scripts/experiment-apparatus/replay.sh` now points to the new file; the old file is kept as is and not overwritten)

Configuration line:

```
E7RESULT name=configuration unit_bytes_32=32768 unit_bytes_16=16384 header_registered_bytes=105 nonce_mac_reserved_bytes=29 net_payload_bytes=32634 padded_payload_bytes=28672 anchor_multiplier=1.086000 judgement_one_threshold=1.136000 band_lower=1.000000% band_upper=1.500000% additive_threshold_primary=0.136000
```

**Fork sheet row 1: the share given by each of the three sources:**

- **A**: gives only a function, not a single number (D25 (Target Workload Priority) settled item 6: whom to optimize for can only be decided by a human; there were no real weights during the first segment).
  The ω direct sampling table (draws the shape of the function, 12 points):

  ```
  E7RESULT name=weight_sample kind=omega mapping=direct parameter=1.0% byte_share_percent=1.000000% request_share_percent=72.112676% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=带内 g1_side_lower_bound_as_crossed=true rule_first_to_trigger=不重开 rule_only_one=不重开 rule_only_five=不重开 additive_upper_bound_at_0.136=不重开 additive_upper_bound_at_0.05=不可判 additive_upper_bound_at_0=不可判
  E7RESULT name=weight_sample kind=omega mapping=direct parameter=1.5% byte_share_percent=1.500000% request_share_percent=79.585492% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=带内 g1_side_lower_bound_as_crossed=true rule_first_to_trigger=不重开 rule_only_one=不重开 rule_only_five=不重开 additive_upper_bound_at_0.136=不重开 additive_upper_bound_at_0.05=不可判 additive_upper_bound_at_0=不可判
  E7RESULT name=weight_sample kind=omega mapping=direct parameter=2% byte_share_percent=2.000000% request_share_percent=83.934426% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 g1_side_lower_bound_as_crossed=true rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开 additive_upper_bound_at_0.136=不重开 additive_upper_bound_at_0.05=不可判 additive_upper_bound_at_0=不可判
  ```

  The full 12 points (0 to 100%) and the other 14 rows of the two mappings × 7 ε values are all in the artifact (`name=weight_sample`, 26 lines).

  **Continuation: real-weight sample points (the real share of A still cannot be answered)**: the weights w = (seq 60%, metaheavy 10%, rand 10%, multistream 10%, smallfile 10%) from settled item 1 of D25 (Target Workload Priority) given by the user were substituted into the two sets of
  illustrative b already in the registration (not real b, just assumed mappings):

  ```
  E7RESULT name=weight_sample kind=real_weight_2026_09_24_illustrative_random_share mapping=only_random_tier parameter=seq60_metaheavy10_rand10_multistream10_smallfile10 byte_share_percent=10.000000% request_share_percent=96.603774% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 g1_side_lower_bound_as_crossed=true rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开 additive_upper_bound_at_0.136=不重开 additive_upper_bound_at_0.05=不可判 additive_upper_bound_at_0=不可判
  E7RESULT name=weight_sample_additive_conclusion kind=real_weight_2026_09_24_illustrative_random_share mapping=only_random_tier upper_bound_shortcut_applicable=false additive_conclusion=要第二段 reason=byte_share不小于1.0%_此时C5符号未定_登记第四节丙类的上界捷径只对byte_share小于下沿成立
  E7RESULT name=weight_sample kind=real_weight_2026_09_24_illustrative_random_share mapping=small_operation_tiers parameter=seq60_metaheavy10_rand10_multistream10_smallfile10 byte_share_percent=30.000000% request_share_percent=99.096774% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 g1_side_lower_bound_as_crossed=true rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开 additive_upper_bound_at_0.136=不重开 additive_upper_bound_at_0.05=不可判 additive_upper_bound_at_0=不可判
  E7RESULT name=weight_sample_additive_conclusion kind=real_weight_2026_09_24_illustrative_random_share mapping=small_operation_tiers upper_bound_shortcut_applicable=false additive_conclusion=要第二段 reason=byte_share不小于1.0%_此时C5符号未定_登记第四节丙类的上界捷径只对byte_share小于下沿成立
  E7RESULT name=weight_arm_true_occupancy_unavailable missing_input=b tiers_missing_b=seq,metaheavy,rand,multistream,smallfile written_method_in_registration=无 reason=E16负载生成器只产写不产读_E140读不按D25档分_smallfile在D25已定项1表三列未测 note=以上两行name=weight_sample_kind=real_weight_2026_09_24_illustrative_random_share是把真实w代进登记已有的illustrative_b_M1_M2不是甲的真实占比
  ```

  Both illustrative points (10%, 30%) fall on the "crossed" side of criterion 5 (> 1.5%), and criterion 1 (reading A) still equals
  the anchor 1.086 at both points, not triggered (consistent with Q13, "criterion 1 is unrelated to the share"); so "first-to-trigger" and "criterion 5 only" both give
  **reopen** at these two points, "criterion 1 only" gives **do not reopen** in both; "additive" cannot be answered at either point (`additive_conclusion=要第二段`):
  the byte_share of both points is ≥ 1.0%, and the upper-bound shortcut in section 4 class C of the registration (T_pad < 0) holds only when byte_share < 1.0%,
  so it does not apply to these two points; the second segment must measure T_dev and T_pad before a verdict. **This is not the real share of A**: b (each tier's own share of random small reads)
  cannot be read today for any of the five tiers, not just smallfile being "unmeasured" in the settled item 1 table of D25 (Target Workload Priority): the `generate_operations` of E16 (Role of Journal: WAL vs Intent Log)
  produces only writes and has no read operations at all (`grep -c -i read scripts/experiment-apparatus/e16_journal.rs` hits 0),
  the workload of E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) has only the three tiers `rand`/`randq`/`seq` and is not split by the five tiers of D25 (Target Workload Priority); the registration did not fix a way to compute b from E16 (Role of Journal: WAL vs Intent Log) or E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header)
  (this sentence was already stated in the file header registered before the run, and was re-checked now to confirm it is still so today).

- **B**: the `research/prompts/e160-prior-art.md` (82 lines) delivered by the researcher gives 6 facts, copied one by one into the setup's input table;
  only facts 1/2 (the `bssplit` defined values of fio upstream `iometer-file-access-server.fio`) yield numbers, and the other four
  (TPC-C / TPC-E / build server / DTRS of Kavalanekar 2008, AliCloud / TencentCloud / MSRC of the three-way comparison paper)
  have no joint statistic for "random and < 32 KiB" (only marginal distributions), so p_B and p_N cannot be read (not 0):

  ```
  E7RESULT name=prior_art_source label=乙1_fio_iometer_文件服务器定义值_R32切点 citation=research/prompts/e160-prior-art.md:26-27,41-53 byte_share_percent=30.685921% request_share_percent=86.000000% judgement_one_value=1.064765 judgement_one_triggered=false judgement_five_side=越过 rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开
  E7RESULT name=prior_art_source label=乙1a_fio_iometer_文件服务器定义值_R4切点_G2敏感性 citation=research/prompts/e160-prior-art.md:26-27,41-53 byte_share_percent=23.465704% request_share_percent=80.000000% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开
  ```

  Both cut points, R32 (< 32768 bytes) and R4 (≤ 4096 bytes, G2 sensitivity), fall on the "crossed" side (same side, F7 not triggered).
  B is not "the whole column unreadable", so F4 is not triggered.

- **C-spec** (function):

  ```
  E7RESULT name=declared_c_arm point=X*/2_at_0.5025% reads_per_second=264.791919 byte_share_percent=0.502513% request_share_percent=56.387665% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=未越过 rule_first_to_trigger=不重开 rule_only_one=不重开 rule_only_five=不重开
  E7RESULT name=declared_c_arm point=positive_control_1000 reads_per_second=1000.000000 byte_share_percent=1.871650% request_share_percent=83.001328% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开
  ```

- **C-measured** (the raw output of the E152 (Per-Milestone File Performance Against Six File Systems) 2026-09-15 single-disk / two-disk mirror run, taken from `git show 3cff909^:research/results/e152-file-system-benchmark*-2026-09-15.out` and copied verbatim into
  `research/results/e160-e152-input-single-2026-09-15.out` and `...-mirror-2026-09-15.out`, with 890 / 941 lines,
  and the last line `emitted=174` consistent with the registration, V5 not triggered): all 14 configurations (7 single-disk + 7 mirror) **fall on the "crossed" side**,
  with no exception (F5 not triggered):

  ```
  E7RESULT name=measured_c_arm source=single configuration=zfs random_median_read_kibibytes=257068 sequential_median_read_kibibytes=4194304 sequential_matches_declared_assumption=true byte_share_percent=5.775028% request_share_percent=94.008455% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开
  E7RESULT name=measured_c_arm source=single configuration=raw random_median_read_kibibytes=5684524 sequential_median_read_kibibytes=4194304 sequential_matches_declared_assumption=true byte_share_percent=57.542494% request_share_percent=99.712607% judgement_one_value=1.086000 judgement_one_triggered=false judgement_five_side=越过 rule_first_to_trigger=重开 rule_only_one=不重开 rule_only_five=重开
  E7RESULT name=measured_c_arm_summary count=14 minimum_byte_share_percent=5.620506% median_byte_share_percent=50.439841% maximum_byte_share_percent=57.542494% count_below_lower=0 count_within_band=0 count_above_upper=14
  ```

**Fork sheet row 2: how the two criteria combine, the part the first segment can judge:**

- **Q13 (whether criterion 1 is related to the random small read share): unrelated under reading A.** For A (6 ε sample points, p_B from 0 to 20%) and C-spec
  (4 sample points, reads_per_second from 1 to 1 000 000), the criterion 1 values computed point by point are all identical, all equal to the anchor 1.086:

  ```
  E7RESULT name=q13 weight_arm_all_equal=true weight_arm_reference=1.086000 declared_arm_all_equal=true declared_arm_reference=1.086000 reading=无关
  ```

  The reason is that the R class in both the A and C models is always a single 4 KiB request, and the value of reading A is determined only by the size mix inside the R class and does not enter p_B or p_N.
  This directly limits the reach of the statement in C357 (two reopening criteria on same axis never synthesized) that "the two criteria sit on the same axis": **it holds for criterion 5 and does not hold for criterion 1 (reading A)**;
  whether criterion 1 flips between sources can only go through the request size distribution or queue depth, not through the share. The sizes of B 1 (fio defined values) are mixed
  (512 B to 64 KiB), and the criterion 1 value drops to 1.064765 (still < 1.136, not triggered), but this is "the same source with a different size mix",
  not "the same source with a different share", and does not contradict the Q13 reading.
- **Instances with p_B < 1.0%: "additive" gives its conclusion by the upper bound:** upper bound of T_dev = anchor − 1 = 0.086 (T_pad < 0, the definition of the crossover point).
  0.086 < 0.136 (the additive primary threshold), so as long as a source's p_B < 1.0%, "additive" at T_sum = 0.136 certainly does not reopen;
  at T_sum = 0.05 or 0 the upper bound cannot decide (0.086 ≥ both), and the second segment must measure the real values of T_dev and T_pad:

  ```
  E7RESULT name=additive_rule_demo load_level_cost=0.08 header_cost=0.07 threshold=0.136 reopens=true
  E7RESULT name=additive_rule_demo load_level_cost=0.08 header_cost=0.07 threshold=0.05 reopens=true
  E7RESULT name=additive_rule_demo load_level_cost=0.08 header_cost=0.07 threshold=0 reopens=true
  ```

  (These three lines are a demonstration of the additive rule's arithmetic itself: `load_level_cost=0.08, header_cost=0.07` is row f of the truth table in section 5.4 of the registration,
  not the measured T_dev or T_pad of any source. Today there is no real instance with p_B < 1.0%: both cut points of B 1, C-spec from X*/2 onward, and
  all 14 configurations of C-measured have p_B ≥ 0.5%, and the only ones falling into [0, 1.0%) are some ε/ω sample points of A,
  but A has no real weights today, so for the branch "p_B < 1.0%" no source currently gives a real share, only a conclusion derived from the theoretical upper bound).
- **On instances where the four synthesis rules can compute a concrete share today:** the p_B of B 1 (both cut points), C-spec (from X ≥ X*), and C-measured (all 14
  configurations) all fall on "crossed", and criterion 1 (reading A) is always 1.086 and never triggers on these instances (B5 refers to the `judgement_five_side` column, whose values are consistent with "crossed").
  So "first-to-trigger" and "criterion 5 only" both reopen on these instances, "criterion 1 only" never reopens, and "additive" on these instances
  has p_B ≥ 1.0% ⇒ the upper bound cannot give a conclusion, and the second segment must measure.

Other artifact details (page-to-unit geometry, read amplification reference, the B1–B9 series, conversion between the two models):

```
E7RESULT name=unit_geometry period_pages=16317 spanning_pages=2047 spans_page_at_period_boundary=false device_amplification_l16=4 device_amplification_l32=8 device_amplification_l32_padded=8 device_amplification_l32h=9.003616
```

### Stop Clauses

- **S1** (page-to-unit arithmetic differential): `spans_two_data_units_at_page` in `scripts/experiment-apparatus/e160_random_small_read_share.rs` gives, at payload 32634, a period of 16317 pages,
  2047 pages spanning units, and page 16317 not spanning; `cargo test -p singlefs-core --lib
  about_one_in_eight_four_kibibyte_pages_spans_two_data_units` was run fresh on 2026-09-24 and passed, with the same conclusion
  (16317 pages, 2047 pages spanning units). The two sides are independent implementations that share no code; not stopped.
- **S3** (constant differential): the setup's hand-written `HEADER_REGISTERED_BYTES` (105), `NONCE_MAC_RESERVED_BYTES` (29),
  `UNIT_BYTES_32` (32768) were checked fresh on 2026-09-24 against the current values of `DATA_UNIT_HEADER_BYTES`, `NONCE_MAC_ALGORITHM_RESERVED_BYTES`, `DATA_UNIT_BYTES` in `crates/singlefs-format/src/lib.rs`
  and agree; not stopped.
- **S2** (L32h timing-arm differential): belongs to the second segment, not applicable to this segment on 2026-09-24.

### What It Cannot Answer

- **The real share of A**: on 2026-09-24 the user supplied the real weights w of settled item 1 of D25 (Target Workload Priority), but b (each tier's own
  share of random small reads) cannot be read for any of the five tiers: the workload generator of E16 (Role of Journal: WAL vs Intent Log) produces only writes and has no read operations, and the reads of E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) are not split by the tiers of D25 (Target Workload Priority),
  and the registration did not fix a way to compute b from them (stated in the file header registered before the run, and re-checked on 2026-09-24 to be still so). With w in hand
  it can only be substituted into the two sets of "assumed" illustrative b already in the registration (M1, M2) to draw two points (10%, 30%), which are not real shares.
- **Whether "additive" reopens when p_B ≥ 1.0%**: needs the freshly measured T_dev (workload-level device time ratio) and T_pad (cost of with-header relative to padding)
  from the second segment; in this segment on 2026-09-24 no real source has p_B < 1.0%, and there is no second-segment measured cost either, so "additive" cannot be answered for any instance with a computable concrete
  share (only the upper-bound conclusion for p_B < 1.0% can be given, and this branch has no real instance); the two
  real-weight illustrative points (10%, 30%) of the 2026-09-24 continuation also fall on this branch, marked `additive_conclusion=要第二段`.
- **The `additive_upper_bound_at_*` fields of `emit_weight_instance` likewise do not apply to rows with p_B ≥ 1.0% in the existing ε/ω sample tables**
  (found during the 2026-09-24 continuation): those three fields look only at the threshold, not byte_share, and unconditionally print
  the same set of conclusions for all weight sample points, while section 4 class C of the registration and Q5/Q7 in section 6 both state that this upper-bound shortcut holds only when byte_share < 1.0%. In the existing
  ε/ω tables, the `additive_upper_bound_at_0.136` field of rows with p_B ≥ 1.0% (for example the ω=100% row) literally reads
  "do not reopen", but this conclusion likewise does not hold for those rows; however, before the 2026-09-24 continuation the first-segment body text and the experiment page never cited this field to judge the
  "additive" conclusion of those rows (they went through `additive_rule_demo` or the upper-bound derivation), so no published conclusion went wrong. This time only the two newly added
  real-weight points got a correction line (`name=weight_sample_additive_conclusion`); `emit_weight_instance`
  itself was not changed and the output of existing ε/ω rows was not touched (`diff` confirmed the first 69 lines unchanged); whether to add a correction to the existing rows too is left for the main agent to decide.
- **Whether the criterion 1 anchor holds on this machine**: this segment on 2026-09-24 used throughout the anchor 1.086 of E58 (End-to-End Cost of Checksum Granularity), with no new measurement;
  whether F1 hits requires the second segment.
- **The real crossover point of criterion 5 under the clause-basis form (header 133, AES-256-GCM)**: this segment on 2026-09-24 used only the band given by the clause,
  [1.0%, 1.5%], with no newly measured crossover point; whether F2 hits requires the second segment.
- **The shares of the six sources, Kavalanekar 2008 and the three-way comparison paper, for B**: the public data itself has no joint statistic of random + small,
  and a search found no second independent route to fill it; this is not an omission of this segment on 2026-09-24.

### Replay Command

```bash
bash scripts/experiment-apparatus/replay.sh E160
# 直接跑装置：
cd scripts/experiment-apparatus && cargo run --release --bin e160-random-small-read-share
```

Code `scripts/experiment-apparatus/e160_random_small_read_share.rs` (25 unit tests), registration before the run
`e160-preregistration.md` (section 12 "Revisions" records the sample points added by the 2026-09-24 continuation and the correction of
one implementation defect); mutation table `scripts/mutation/tables/e160_random_small_read_share.tsv`
(19 all caught, including M22 and M23 added on 2026-09-24):

```bash
cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e160-random-small-read-share e160_random_small_read_share.rs ../mutations/e160_random_small_read_share.tsv
```

The inputs of C-measured are copied verbatim into `research/results/e160-e152-input-single-2026-09-15.out` and
`research/results/e160-e152-input-mirror-2026-09-15.out` (taken from `git show 3cff909^:research/results/e152-file-system-benchmark{,-mirror}-2026-09-15.out`).

### Affected Decisions

| Decision item | Relation | Review |
|---|---|---|
| D4 (Checksum Location) settled item 1 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference); conclusions, numbers and artifacts unchanged |
| D4 (Checksum Location) settled item 5 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference); conclusions, numbers and artifacts unchanged |
| D4 (Checksum Location) settled item 8 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference); conclusions, numbers and artifacts unchanged |
| D25 (Target Workload Priority) settled item 1 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference); conclusions, numbers and artifacts unchanged |
| D25 (Target Workload Priority) settled item 6 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference); conclusions, numbers and artifacts unchanged |

## Historical Versions

### 2026-09-24 (After the First Page Write, Continuation the Same Day)
- When the page was first written: 22 unit tests, 17 mutations all caught, only the function and sample points of A (no real weights), and the only artifact was
  `e160-random-small-read-share-segment1-2026-09-24.out` (70 lines).
- After the continuation: the user supplied the real weights of settled item 1 of D25 (Target Workload Priority), and the sample points of arm A at these weights, substituted into the two sets of
  illustrative b, were added (the continuation of fork sheet row 1); along the way, it was found and corrected that the `additive_upper_bound_at_*` fields of `emit_weight_instance`
  do not apply to rows with byte_share ≥ 1.0% (only the two new points got correction lines; the output of
  existing ε/ω rows was not changed). Unit tests increased to 25, mutations to 19 all caught (M22 and M23 added), and the new artifact
  `e160-random-small-read-share-segment1-2026-09-24-realweight.out` (75 lines, the first 69 lines byte-identical to the old file).
  Section 12 "Revisions" of the pre-run registration records the sample points added this time and the basis.
