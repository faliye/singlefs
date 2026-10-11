**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1-15`（整段抄，未转述）**

```markdown
## E171 三oracle合谋测试 —— 已跑（s4 整表 117 行、F4 扩表 164 行与编不过 6 行的补位都已跑，读数到齐；结论三轮判决已写回）

<!-- doc-lint:not-numbers O1 O2 O3 Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 N1 N2 N3 N4 N5 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 X12 X13 X14 X15 X52 X53 X54 X55 X56 X57 X58 X59 X60 X61 X62 X63 X64 X65 X66 X67 X68 X69 X70 X71 X72 X73 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 V1 V2 W1 K4 -->

跑前登记 `research/prompts/e171-preregistration.md`（含第十二节修订 1–5；Q4 的另一种读法由主 agent 在装置写之前删去，只留整表 ≤ 60 秒那一种）；问题单 `research/prompts/three-oracle-collusion-questions.md` 的 Q1（一张坏实现表在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行）、Q2（手造坏镜像喂 O2、O3，有没有一张两条全放行）、Q3（拿已知被抓的坏实现逐条关掉一条 oracle，剩下的还红不红）、Q4（这道测试一次跑多久、怎么进门禁）。被测条款 D13（验证路线） 已定项 6：三个 oracle 的独立性要靠合谋测试验证，不许自称。

**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3 在 37 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2 放行而 O3 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2 放行而 O3 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2 一列（底态 F 红、Mk 不红），O3 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2 放行而 O3 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2 放行而 O3 判不在」，不写「O3 独有」，Q2 只比 O2 与 O3 一对，O1 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1 的 32、45、108、115、172、233、237、249、250、281；只靠 O2 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:16-31`（整段抄，未转述）**

```markdown
### 装置、研究一侧与变异表

| 件 | 位置 | 说明 |
|---|---|---|
| 入库装置 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | 模式 `baseline`、`row`、`known-bad-images`，开关 `--positive-control`、`--disable-oracle`；只读、只驱动、只观测，不改 core、checker、harness、format、replay-prefix；内联单测 31 条（`grep -c '#\[test\]'`） |
| 研究一侧 | `scripts/experiments/e171_three_oracle_collusion/`（Python，目录里没有 `main.rs`，cargo 不认它当 bin） | 挑法脚本（登记 5.2，sha256 `4032a0e0…`，原样抄）、117 行的表 `bad-implementations-117.tsv`（登记 5.9 照名字抄出）、F4 扩表 `bad-implementations-expansion-164.tsv` 与造它的 `expansion_table.py`、`row_driver.py`、`comparator.py` 与它的几个模块、冻结执行器的接线 `frozen_o3.py`、判坏镜像的 `o3_bad_image_judge.py`、四个单测文件、变异驱动 |
| 变异 | 研究一侧 `scripts/mutations/e171_three_oracle_collusion.tsv` 67 行；装置一侧 `crates/mutations.tsv` 末尾 15 行（名字以「E171 装置」起头，`grep -c '^E171 装置' crates/mutations.tsv`） | 研究一侧经本目录的 `mutation_driver.py` 跑（登记修订 1 第 1 行）；装置一侧是 checker 档，三个数归提交时的门禁 checker-tier-crates-mutation-replay |
| 复跑 | `scripts/replay.sh` 里 E171 十一行登记（`grep -c '^E171.@driver' scripts/replay.sh`） | 见「复跑」一节 |
| 产物 | `research/results/` 下 `e171-three-oracle-collusion-2026-10-03-` 起头的 46 份（`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`） | 逐份在「复跑」一节的产物表里 |

- 流的驱动：两条流（新池新建文件、固定脚本到 E 再正常卸载）照抄 E170（O3规约执行器第一版） 导出装置的两条流驱动、取样三批与中性格式的写出（不 `#[path]` 引它），改三处：驱动里的 API 调用不 `.expect`，结局（成功、错误成员、panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`；跟着每次调用推理想模型，记 O1-对拍（模型的答案与实现的结局比，`judge_and_advance`）；每个取样态算 O1-崩溃判官（`crash_recovery_disagreement`）与 O2（`check_pool_image`），写 `oracle-verdicts.tsv`。`baseline` 回比本地常量与 crates 的现算值（S1，42 条 `name=s1` 行都是 `holds=true`；`grep -c 'name=s1 ' …-baseline-run1.out` 与 `grep -c 'name=s1 .*holds=false'` 现数）；`row` 不回比，只取批 1 与批 2。
- S8 的三处调用在装置里的行号（`grep -n` 现取）：`newest_persisted_root` 第 1883 行、`observed_read_back_after_a_crash` 第 1885 行、`crash_recovery_disagreement` 第 1907 行，都在 `crash_judge_of_the_state`（第 1875 行）与它转调的 `crash_judge_verdict` 里；O1-对拍的 `judge_and_advance` 在第 544 行。
- 已知坏镜像族：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908…`）的导入、常量、辅助函数与 `known_bad_images` 照抄进装置里的一个模块（第 7–1294 行，加上 `known_bad_images` 用到而散在第 1295 行之后的 12 个常量与辅助函数）；`known-bad-images` 模式核条数 41、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的（S4）。
- 逐行驱动：一次跑先拍一张仓快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，237 个文件，总摘要写在驱动的标准输出第一行），每条线道从快照同步出自己的树与编译目录，改一处、`cargo build --release`、装置 `row` 经 `run-with-memory-cap.sh`（限时 1800 秒）、还原；表里的行照名字取、核锚点，不行按补位规则换同组下一行（至多三次）。s4 与 F4 两趟时驱动只在编之前的失败上补位；第八段起编不过之后也补位（换上的行编不过再换，编之前的失败与编不过合起来至多三次），组里不含 E171 装置自己的变异行，`row-result.tsv` 记 `substituted_from` 与每一个试过而没用上的 `failed_attempt_<序>`；开关 `--keep-compile-failures-as-they-are` 只给 `replay.sh` 复跑 s4、F4 两份产物用，补位那一份产物不带它。


```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:32-79`（整段抄，未转述）**

````markdown
### s4 整表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out`（2119 行，末行 `E7RESULT name=done emitted=2119`）。冻结核对、阳性对照四条 oracle 各两趟（基线与空变异）、Q1 的分母与三种读法、7 行全放行（甲）、Q3 两种读法、Q3-核 9 趟与判决行：

```
E7RESULT name=config segment=s4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=empty_mutation red_units=1 red=true
E7RESULT name=q1_denominator rows_run=117 counted_D=19 untouched=93 only_in_the_ignored_pass=0 compile_failed=2 anchor_not_unique=0 not_found=0 self_crash=3 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=5 rows_run=117 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=19 O1=5 O1-对拍=5 O1-崩溃判官=1 O2=8 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
E7RESULT name=q1_alpha counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_quiet_row reading=q1_alpha row=15 row_name=实审_A2c_第_15_条：用户数据落点不从这张图的单元区起点起扫 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=22/22 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=23 row_name=并行线三_C116：分裂出来的右半容器号不是触发分裂那条新记录的_inode_号 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=26/26 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=27 row_name=实二十：记录头读者把末条标志位_0_读反（带标志的读成不是末条） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=34 row_name=步_5：抬_F_生效之后不放开扣住的槽 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/498 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=42 row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：卸载把_F_抬到现行那一版带的_F（原地不动）而不是它的_txg o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/537 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=92 row_name=步_1_验收第_4_条：inode_记录的写入时间留成_A_的（覆盖写照抄上一版的写入时间） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/61 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=112 row_name=步_3：前缀跨实例边界（把别的实例的记录也接上） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_beta counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=19 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_beta caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_core_row closing=O1 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=no_executor row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=7 q1_beta_all_quiet_rows=7 q1_gamma_all_quiet_rows=7 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=11 q3_beta_rows_held_by_a_single_oracle=11 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
```

分族、分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `2 compile_failed`、`19 counted`、`3 self_crash`、`93 untouched`（四行）。不进分母的 5 行：第 6、7、85 行实现自崩，第 33、59 行编不过。登记 5.2 写「一行编不过……换同一文件组内下标 j + 1」，s3、s4 那一版逐行驱动只在编之前的失败（找不到、名字不唯一、锚点不唯一）上补位，第 33、59 行没有补位；驱动在第八段改成编不过之后也补位，这两行连同扩表编不过的 4 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:80-95`（整段抄，未转述）**

```markdown
### 三条全放行的 7 行

读数取 s4 的导出目录（`scripts/experiments/target/e171-dumps/rows-s4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比；草稿脚本 `quiet_rows_s4_fua.py`、`returned_check.py` 只读导出目录，不入库）。三分照 `.claude/rules/mutation-sampling.md` 的口径落到这一批上：**对这一批等价**＝两条流、批 1 批 2 的每个态连同实现在态上的恢复都在条款允许的集合里（写依据；会分开的历史写在旁边，没跑）；**取样点不敏感**＝两条流里有条款不许的态，取样规则没取到；**真盲区**＝取到的态里有条款不许的，三条都放行。「镜像不同」数的是 `states.tsv` 里镜像摘要与基线那一态不同的态；「择到新版」数的是其中实现看 journal 那一遍恢复择到的根 txg 不小于「第一处不同之后写出的第一条根」的态。三条各自看什么：O1-对拍比每次调用之后实现交回的 `ObservedRoot`（`crates/singlefs-harness/src/model.rs` 第 589 行：根的 key、journal 计数器、根带的 F、文件内容、实例表、每块盘的分配记录），O1-崩溃判官比实现恢复择到的根与读回内容；O2 判单个镜像的结构；O3 的允许集合 A(c) 由这一行自己写出的根槽写构成（`scripts/experiments/e170_second_run_o3_spec_executor/version_table.py` 第 3 行），一版只投影根的六样（`root_ring.py` 第 22 行：实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F），判的是它自己从镜像恢复出的那一版在不在 A(c) 里（`executor_core.py` 第 127 行）。下表末列的三分是三轮结论判决之后的现值（旧三分列的改动写在「历史版本」一节），各行归属的全貌在「三轮判决之后的三分」一节。

| 行（变异） | 和基线不同在哪 | 落在哪种状态里 | O1 / O2 / O3 看不看得到 | 三分 |
|---|---|---|---|---|
| 15（用户数据落点不从这张图的单元区起点起扫） | 两条流第 22 条 op 起：用户数据单元两块盘各 32 KiB 落点 822149120 → 823132160，内容逐字节相同；指向它的 16 KiB 节点里指针与校验和随之变；同一次发布的十一个提交内生块（t2–t12）也整体后移 64 槽，开放段 [50240, 50304) 整段后移到 [50304, 50368)：设备 0 上 12 条写换了落点（第 22 条 op 50180 → 50240，第 24 条 50240 → 50304，第 26 条 50242 → 50306，……，第 44 条 50252 → 50316；两条流相同，`research/prompts/e171-conclusion-r3-opus-model/out/row15-offsets.out` 第 1、2 行 `name=row15_first_publish`）；流二第 4 次调用起返回点后移 4 条 op（`mount_writable` 92–179 → 92–183），共 555 条 op 不同；调用结局同；看 journal 那一遍恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到新版：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1：`ObservedRoot` 不带落点，读回同，看不到；O2：落点在单元区里、记账自洽，判绿；O3：投影不带落点，「在」 | **真盲区候选**：依据是 D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md` 第 208 行）、射程「改新池新建文件的字节：是」「十一个提交内生块（默认几何）的落点整体重排」（第 226 行）、字节表「t2 到 t12 依次 50240、50242–50243、50244、…」（`.claude/kb/layout/01-first-txn.md` 第 307 行）；变异下用户数据落到 50240、十一个提交内生块整体挪 64 槽，不在 ⑤ 钉的落点上，三条都放行。另一条依据（用户 2026-10-03 定案之后才有）：D3（空间分配） 已定项 8 射程「例外是**从 mkfs 水位第一次建树的那一版**」「新池新建文件是这一类」、t1@50180「登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里，算格式级，偏离它写出的这一类镜像不合法」（`.claude/kb/decisions/03-空间分配.md` 第 171 行），变异把 t1 从 50180 挪到 50240，本身就不合法；三轮判决定去留时还没有这一条，只靠 ⑤。会分开的历史（推的）：单元区刚好够用的池，跳过开头一段之后发布在模型说该成的地方报空间不够，O1-对拍会红 |
| 23（分裂出来的右半容器号不是触发分裂那条新记录的 inode 号） | 两条流第 26 条 op 起：第一个文件那次发布写的 inode 叶（码 3，两块盘各 32 KiB）偏移 53 一字节 0x01 → 0x00（容器号 1 → 0），头校验和随之变；同一次发布第 28 条 op（16 KiB 节点）偏移 52、60、131、149 等处 0x01 → 0x00；之后重写 inode 树的每次发布都带着（流二 155 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到第一个文件那一版（txg 3）及之后：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1：不读盘，`ObservedRoot` 里没有容器号，读回同；O2：I-9.4（容器号不超最小 key）只要「inode 号 ≥ 容器号、沿叶序严格递增」，一片叶时 1 ≥ 0 成立，判绿；O3：投影不进 inode 树，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 6 定案写「**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md` 第 203 行），射程写「每个容器号都是某条记录首次插入时的 inode 号」（第 205 行）；容器号 0 不是任何一条记录的 inode 号。择到新版的那些态不在条款写得出的镜像里，三条都放行。会让 O2 红的取样点（推的，没跑）：一片叶装满之后的分裂，右半容器号 0 小于左半，I-9.4 的「沿叶序严格递增」判红 |
| 27（记录头读者把末条标志位 0 读反） | 写流与调用结局全同；只在看 journal 那一遍恢复上不同：流一批 1 6 态、流二批 1 51 态（批 2 0 态）里，基线施加 1 条记录、择到新的一版，变异把带末条标志的记录读成「后面还有」、施加 0 条、停在前一版；读回内容不同的 3 / 30 态（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 发布写到一半的态：被丢掉的那一版的调用没返回、它的根槽没落（`returned_check.py` 57 态全是 `call_returned=False its_root_slot_landed=False`） | O1-崩溃判官：前一版是模型提交过的、不比盘上已持久的最新根槽旧、读回与那一版相同，判绿是对的；O1-对拍：调用不经恢复；O2、O3：镜像与基线逐字节相同，各自不看实现的恢复 | **另一类：三条不判实现恢复里靠记录重建的那一段**。态本身合法（调用没返回，新旧两版都在允许集合里），错在实现的恢复少施加一条记录，违反 D13（验证路线） 已定项 1 的 M4「`Recover` 返回的根恰是 j\*」与 D23（journal 的角色与格式） 已定项 15「由记录重建那次发布的根」、已定项 14「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」；O1-崩溃判官的判据（模型提交过的某一版、不比盘上已持久的最新根旧）比 M4 弱，O2 判恢复之前的镜像，O3 不跑实现的恢复；M4 的落点是 E165 与 Verus，不是这三条。会分开的取样点（推的，没跑）：一次已经返回、提交点只在 journal 记录里（根槽还没写）的发布之后的态，变异会丢掉一次已返回的发布；两条流的发布都在返回之前写根槽，这样的态不存在。那时 O1-崩溃判官的「不许比盘上已持久的最新根槽旧」也挡不住（根槽没写），会不会三条都放行没量过 |
| 34（抬 F 生效之后不放开扣住的槽） | 流一无不同；流二第 498 条 op 起（抬 F 之后那次 `publish`）：用户数据单元落点 822083584 → 822411264，内容相同；指向它的节点与分配记录随之变（51 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 68 态、批 2 21 态；择到 txg 17 及之后的根：批 1 60 态、批 2 14 态 | 同第 15 行：O1 不比落点，O2 判绿（记账自洽），O3「在」 | **另一类：分配政策，镜像合法**：抬 F 之后那次发布换了一个空闲槽、没复用回收的槽，每个态的分配记录与记账自洽；D3（空间分配） 已定项 8 射程「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法，拦它只能靠运行时读数」（`.claude/kb/decisions/03-空间分配.md` 第 171 行；新池新建文件那一格例外，这一行的改动在流二抬 F 之后那次发布上，不在其中），三条按设计都不判分配政策。会分开的历史（推的）：抬 F 之后空间只够复用扣住那几槽的池，发布报空间不够而模型说该成，O1-对拍会红；这一行在 `crates/mutations.tsv` 点名的测试 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 直接断言复用 |
| 42（卸载把 F 抬到现行那一版带的 F 而不是它的 txg） | 流一无不同；流二第 537 条 op 起（卸载那次调用的第一条写：两块盘的系统配置槽，偏移 481 一字节 0x11 → 0x0b，即 F 17 → 11，连同校验和），之后卸载推的空发布写的根与节点里同样是 11（24 条 op 不同）；卸载照常 `ok`；恢复逐态同 | 镜像不同：流二批 1 48 态、批 2 14 态，全持久态在内；择到卸载写出的根（txg ≥ 18）：批 1 32 态、批 2 7 态 | O1：卸载没有模型的答（登记读法 6），这次调用不对拍、写落在卸载里的态崩溃判官记不适用，看不到（若有卸载的答，`ObservedRoot` 带根的 F，会比出 11 对 17，推的）；O2：系统配置的 F 与根带的 F 都是 11，I-7.12（系统配置 F 不低于同盘根上的 F）成立，单个镜像不知道该是 17；O3：投影带 F，但 A(c) 就是这一行自己写的 F = 11 的根，「在」 | **真盲区候选**：D16（发布语义） 已定项 1 定案的字段表「正常卸载」一行写「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md` 第 42 行），射程「正常卸载的代价」那一条写「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（第 57 行）。变异下卸载返回之后的全持久态 F = 11，回退候选集里留着 txg 11–16 的根，不在条款允许的镜像里，三条都放行。O1 那一半是登记读法 6 写明的已知缺口（模型没有卸载的答） |
| 92（inode 记录的写入时间留成 A 的） | 流一无不同；流二第 61 条 op 起（第二次 `publish`，覆盖写那次的 inode 叶，两块盘各 32 KiB）：叶内偏移 200、208、216 三个字节 0x3c → 0x00（第 0 条记录起点 136，+64 / +72 / +80 是 atime、mtime、ctime 秒：基线 60，变异留 A 的 0），头里偏移 10–13（头校验和）与 89–92 另有不同；之后重写这片叶的发布都带着（139 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 344 态、批 2 112 态；择到覆盖写那一版（txg 4）及之后：批 1 336 态、批 2 105 态 | O1：`ObservedFile` 只有内容，`model.rs` 与 `model_comparison.rs` 里 `write_time` 零命中，D13（验证路线） 已定项 6 表 O1 行判「内容、大小、目录结构」，看不到；O2：单个镜像不知道这次发布的写入时间参数，判绿；O3：投影不进 inode 树，「在」 | **另一类：条款在字节表与里程碑验收，决策正文欠一句**：D8（核心索引结构） 已定项 6 的字段表只定了偏移 64「atime / mtime / ctime 秒」的位置；「写入时间是发布参数」写在 `.claude/kb/milestone/01-first-txn.md` 第 103 行与 `02-second-txn.md` 第 103 行，字节表写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md` 第 267 行），验收写「inode 记录的写入时间留成 A 的 ⇒ 判红」（`02-second-txn.md` 第 113 行），决策正文里没找到取值的一句。择到新版的态（写入时间 0、这次发布的参数是 60）不合验收，三条按设计看不见（没有 API 交回写入时间），不当独立性失败 |
| 112（前缀跨实例边界） | 写流与调用结局全同；看 journal 那一遍恢复在两条流各 3 态（批 1 第 9–11 态，段 1：暖机的第一条写——新实例 1 的 journal 记录——落了、它的根槽没落）不同：基线择 `0,0`、施加 0 条，变异把实例 1 的记录接到实例 0 的根上、施加 1 条、得到 `1,1`；读回都没有文件 | 暖机写到一半的态（`returned_check.py` 6 态全是 `call_returned=False`） | O1-崩溃判官：`1,1` 是模型提交过的一版、不比已持久的最新根槽旧、读回同，判绿是对的；O2、O3：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段（多施加）**：暖机没返回，`0,0` 与 `1,1` 都在允许集合里，错在实现的恢复把别的实例的记录也接上，违反 D23（journal 的角色与格式） 已定项 14 注 1「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md` 第 369 行），依据只有这一条注；这一行的变异已在门禁 checker-tier-formal-verification 的红例钉表里（`.claude/gate.d/e165-pinned-red-examples.tsv` 第 26 行），由 E165 的有界穷举判。会分开的取样点（推的，没跑）：环里留着别的实例、不在所选根那条时间线上的记录（崩溃恢复抛弃了根之后的新实例），接进来之后恢复落到模型没提交过的一版，O1-崩溃判官会红；两条流里没有这种态 |

第 102 行（抬 F 先写系统配置那一步不带新 F）是批 2 多出来的那一行全放行（`name=q1_alpha … rows_that_differ_from_batch_1=102`）：流二第 440 条 op（抬 F 先写的系统配置槽，偏移 481 一字节 0x0b → 0x00）与第 537 条 op（卸载先写的系统配置槽，0x11 → 0x0b）不同，4 条 op；O2 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 60、73 段各 6 态（`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2 total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`）。批 2 在流二只取 17 段（7、11、16、20、24、28、32、36、40、44、48、52、57、61、65、70、74），第 60、73 段一个态都没取（`awk -F'\t' '$3==60\|\|$3==73' states.tsv` 只出批 1 的 8 + 8 态）：**取样点不敏感**，会分开的取样点就是第 60、73 段的任一态。这一格两批翻面，记「不稳定」；Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 8 行仍 ≥ 1；只靠一条的 10 行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:96-115`（整段抄，未转述）**

```markdown
### Q3 只靠一条的 11 行

只写够不到的对象与没判的字段，不判结论。读数取 s4 的导出目录（草稿脚本 `caught_rows_s4.py`，不入库）与产物的 `name=trajectory` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 32（可写挂载不写实例表的行，实例表照抄） | O1-对拍：流二第 4 次调用 `mount_writable` 判 `disagrees:这一版的实例表`（模型要有上一个实例那一行，实现交回空表），模型停在这次调用，之后流二 324 态（批 1）崩溃判官记不适用 | O2：这一行 0 态红（批 1 流二 397 态都绿），实现了的不变量里没有一条拿根的实例代号去对实例表的行（由读数推的，没逐条查 invariants.md）；O3：投影里没有实例表，A(c) 是这一行自己写的根，「在」 |
| 45（挂载那一刻就读不出的槽也重读、仍坏就拒） | O1-对拍：流二第 12 次调用 `raise_floor` 交回 `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，模型说该成 | O2、O3：被拒之前的写流与基线逐条相同（流二停在第 440 条 op，0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，单镜像的判官没有对象可判 |
| 108（码 2 读者补齐字节从最后一个载荷字节算起） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（胶水解不开实现交回的单元：「补齐字节非零」）；流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（流一 0 条 op 不同，流二停在第 92 条 op 之前也 0 条不同）；O2 用自己的解析器读裸字节、不调 core 的读者，O3 用 Python 自己的读者，两边看到的镜像与基线逐字节相同。层 0 那份手写版本表的 oracle 与记录核对器红（`layer0_or_record_checker_red=true`，不是三条之一） |
| 115（取前缀核锚点筛选不看 txg） | O1-崩溃判官：批 1 流一 32 态、流二 52 态「走读失败：没择到根」（实现恢复报 `RootPublishCarriesMoreThanOneLastRecordFlag…`），段为正 8/12、12/16；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery`，模型说该成 | 写出的字节与基线相同（流一 0 条 op 不同）；O2 判镜像结构、O3 用它自己的前缀规则从镜像恢复，都不看实现的恢复，看到的镜像与基线逐字节相同。层 0 那份 oracle 与记录核对器红 |
| 8（空闲不随分配减） | O2：I-5.2（空闲统计对得上）批 1 流一 9 态、流二 353 态，段为正 2/12、69/79；批 2 流二 112 态 | O1：`ObservedRoot` 里没有记账行，调用都成、内容都对；O3：投影不进记账树，「在」 |
| 65（blocks 向下取整） | O2：I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）批 1 流一 9 态、流二 353 态 | O1：`model.rs` 与 `model_comparison.rs` 里 `blocks` 零命中，读回内容同；O3：投影不进 inode 树 |
| 86（写系统配置槽时 F 那 8 字节写成 0） | O2：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 97 态，段为正 19/79；批 2 28 态 | O1：比的是根带的 F（`ObservedRoot` 的 `rollback_floor`），不比系统配置槽里的 F；O3：投影的 F 取自根，系统配置槽只拿来择几何，A(c) 是这一行自己写的根，「在」 |
| 87（core 那份在飞上限把 F 多算 1） | O2：I-8.1（环几何够大）批 1 流一 51 态、流二 385 态，I-7.13（系统配置池级字段在读者收的范围里）流一 2 态、流二 12 态；段为正 12/12、79/79 | O1：不比系统配置槽的字段；O3：同第 86 行，投影里没有在飞上限 |
| 90（新池新建文件的改动计数写回 1） | O2：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流一 9 态、流二 240 态 | O1：不比 inode 记录的改动计数；O3：投影不进 inode 树 |
| 99（defer 行写 0） | O2：I-3.11（已分配减 defer 等于最新根走读）批 1 流一 9 态、流二 353 态 | O1：不比记账行；O3：投影不进记账树 |
| 102（抬 F 先写系统配置那一步不带新 F） | O2：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 12 态，第 60、73 段各 6 态；批 2 不红（取样点不敏感，见「三条全放行的 7 行」一节末） | O1：`raise_floor` 的答比的是写出的根（根带的 F 对），不比系统配置槽；O3：同第 86 行 |

O3 在 s4 计入分母的 19 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`），而它在每行两条流的 450 个批 1 态上都给出了严格判定（`o3_judged_states_strict=450`，`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。由执行器的写法读出来的一条（读码，没另跑）：A(c) 只由这一行自己写出的根槽写构成，投影只有根的六样，所以 O3 只在「实现自己的恢复规则与条款不同、或一次发布在崩溃之后半施加」这一类上红；实现把某个字段一致地写错，O3 照样判「在」。O3 在坏镜像那一族（Q2）上红得多：P 行 26 张红 22、盲测 40 张全红、已知坏镜像 41 张严格红 28（`name=q2_family` 四行）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:116-119`（整段抄，未转述）**

```markdown
### s3 整表作废

s3 那一次整表（输出根 `scripts/experiments/target/e171-dumps/rows-s3/`）两条线道各自跨行复用编译目录；还原改过的文件用 `copy2`、同步用 `rsync -a`，都把修改时间带回快照那一刻，cargo 按修改时间判新旧，线道 1 跑过第 112 行（replay-prefix 的变异）之后、跑第 114 行之前的 45 行，编出来的装置都带着第 112 行的变异。登记修订 5 照实记了这 45 行、cargo 1.98.0 上的机理实测与驱动的改法；s3 整表、s3 的 Q3-核各趟与 s3 比对器产物不再当 Q1、Q3、Q4 的读数。留档不删：`research/results/` 下的 `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out`（第五段对第 2、27、42、65 行的干净复核），与 `scripts/experiments/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`。s3 读到的「Q1 全放行 40 行、分母 52」里 38 行是被带着跑的；s4 在修过的驱动上重跑全部 117 行，判决行 `rows_compiled_without_the_rebuild_check=0`。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:120-128`（整段抄，未转述）**

```markdown
### F4 扩表

登记第十节 F4：「触达行太少，Q1 的『没有』不够判：进第三段扩表（5.2），扩完仍不够记『不够判』，不记『没有』」，触发观测「Q1-分母 D < 30」；s4 的 D = 19。扩表照登记 5.2「扩表」：步长 4 的挑法比步长 8 多出来的 164 行（与步长 8 重叠的 56 行不重跑），按挑法脚本的输出顺序接在 117 后面编号 118–281、刀号 3。

- 行文件 `scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`（sha256 `f1bd082e…`），由同一目录的 `expansion_table.py` 起挑法脚本两次（步长 8、步长 4）造出；表取 `crates/mutations.tsv` 去掉 E171 装置自己的变异行（与 A6、A7 同一个去法）。在 s4 冻住的树的那一份与今天仓里的那一份上造出来逐字节相同；按文件：admission 5、allocation_record_tree 2、allocator 12、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 6、mount 44、mounted_read 3、mounted_session 2、records 2、recovery 28、root_record 1、system_configuration 3、transaction 34、unit 2、replay-prefix 8。登记 5.9 只钉了 117 行的行文 sha256，扩表各行的行文摘要取造表那一刻的表。
- 逐行驱动加 `--other-selected-rows`：补位时跳过 117 行与扩表两张表里的每一个名字（登记 5.2「扩表与补位都不重挑已跑过的行」）。
- 比对器 `--rows-root` 可以给两份（117 行那份与扩表那份，同一个行号在两份里都有就拒），`--expansion-rows` 给扩表时多两行结果：`name=f4_expansion`（扩表与挑法脚本在 `--mutations-table` 那张表上现算的逐行相同、164 行、序号连续、刀号 3、与 117 行不重名）与 `name=q1_denominator_by_table`（分母按表分开再报）；Q1 的分母与三种读法、Q3 都按并起来的 281 行算。拿 s4 的输入重跑改过的比对器（只给 rows-s4），E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）。
- 扩表那一趟在 s4 那一份冻住的树上跑（快照摘要 `d1586465…`，与 s4 五趟驱动的相同），两半才能并成同一批；树不重新冻。主 agent 照第七段交回的脚本跑完七步（空变异一行对 `baseline-run1` 未触达、扩表与挑法脚本在冻住的树的表上现算的逐字节相同、这一次两趟驱动与 s4 的快照摘要只有一种），扩表那一趟两条线道 7 607.9 秒。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:129-161`（整段抄，未转述）**

````markdown
### F4 扩表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`（4698 行，末行 `E7RESULT name=done emitted=4698`；耗时行在 `…-s4-f4-rows-timing.out`，291 行）。并表、分母、三种读法、Q3、判决行，与扩表新出的全放行行（甲 5 行、丙多出的第 233 行）：

```
E7RESULT name=config segment=s4-f4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4+rows-s4-f4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=f4_expansion rows=164 expected=164 first_number=118 last_number=281 numbers_are_consecutive=true slice_3_rows=164 names_shared_with_the_117_rows=0 found_once_in_today_table=164 equal_to_the_selection_script_on_today_table=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=12 rows_run=281 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_beta counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=117 counted_D=19 counted_row_numbers=8,11,15,23,27,32,34,42,45,65,86,87,90,92,99,102,108,112,115
E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=164 counted_D=18 counted_row_numbers=135,172,233,234,237,238,242,244,245,249,250,258,265,269,270,271,277,281
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=2 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=12 q1_beta_all_quiet_rows=12 q1_gamma_all_quiet_rows=13 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=23 q3_beta_rows_held_by_a_single_oracle=23 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
E7RESULT name=q1_quiet_row reading=q1_alpha row=242 row_name=增补_3_第_4_件（增补_2_收口表第_40_行_/_C381_的前提）：发布的持久顺序把系统配置槽轮换挪到根槽_FUA_写之前，最后一步失败时根就不会已经落盘 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=516 o3_judged_states_provisional=516 first_operation_that_differs_from_the_baseline=8/8 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=269 row_name=C554_乙-配置续：见证值取成槽世代号（mkfs_之后第一次取号写的_tail_不再是_0，新池新建文件的字节变了） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=0/0 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=270 row_name=C577：发布在系统配置轮换之后不过屏障就返回（树表_0_条的一版上返回之后见证值可以落后） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=390 o3_judged_states_provisional=390 first_operation_that_differs_from_the_baseline=20/20 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=271 row_name=D8_已定项_8_头_ID：deleted_inodes_的树表条目头_ID_退回写_0（不写它服务的_inode_树的树_ID） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=44/44 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=277 row_name=取前缀核单测：施加一次发布之后新根段不换（停在所选根的） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_gamma row=233 row_name=由记录重建的根不照抄所选根的文件系统标识（代码轮_m3-formal-code-r1_K3，R3-1「其余字段等于所选根」）：改写全零 o1_comparison_red_calls=0 o1_crash_judge_red_states=45 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=true
```

分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `6 compile_failed`、`37 counted`、`6 self_crash`、`232 untouched`（四行）。不进分母的 12 行：第 6、7、85、125、235、272 行实现自崩，第 33、59、127、163、206、243 行编不过（这一份是补位之前的读数，补位那一趟在「编不过的 6 行补位（已跑）」一节）。第 1–117 行的结果行（`name=row`、`row_touch`、`row_matrix`、`trajectory`、`row_o3`、`row_s5`）在 F4 产物与 s4 产物里逐字节相同（1819 行，`cmp` 无输出），7 行全放行的 `name=q1_quiet_row` 三种读法逐行相同：s4 那 7 行的三分不重做。Q2、F3 与对照的结果行两份也逐字节相同。拿第八段改过的比对器重跑 F4 与 s4 的输入，E7RESULT 行与两份产物各自逐字节相同。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:162-176`（整段抄，未转述）**

```markdown
### F4 新出的全放行行

读数取 F4 的导出目录（`scripts/experiments/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比）；草稿脚本 `quiet_rows_f4.py`（第七段 `quiet_rows_s4_fua.py` 改输出根）、`extra_checks_f4.py`、`order_by_segment_f4.py`（都只读导出目录，不入库）。三分的口径与「三条全放行的 7 行」一节相同。顺序类的两行（第 242、270 行）写流的条数或次序变了，逐态与基线比没有意义，改用一条判法在这一行自己的态上数「条款不许的态」，同一判法在基线上数一遍都是 0（判法分得出）：切段用比对器自己的 `baseline_checks.segments_of_the_write_stream`（FUA 写放行它那块盘、屏障放行它那块盘）。

| 行（变异） | 和基线不同在哪 | 条款不许的态（批 1 / 批 2） | O1 / O2 / O3 看不看得到 | 三分 |
|---|---|---|---|---|
| 242（发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前） | 两条流第 8 条 op 起：每次取号、暖机、发布末尾的「根槽 FUA → 两块盘的系统配置槽 → 屏障」换成「两块盘的系统配置槽 → 根槽 FUA → 屏障」，写的内容逐条相同、次序变了（流一 9 条、流二 57 条 op 按下标比不同）；基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段（流一 8 段里 3 段、流二 59 段里 19 段，基线 0 段）；调用结局同 | 同一段里系统配置写已落、根槽 FUA 没落全：流一 21 / 3 态，流二 133 / 19 态；基线 0 | O1-崩溃判官：根槽没落，恢复择到前一版，前一版是模型提交过的、不比已持久的最新根槽旧，判绿；O1-对拍：调用结局同；O2：这一行 0 态红（单个镜像里系统配置槽比根早一次发布，没逐条查 invariants.md 有没有管这一样的）；O3：A(c) 由这一行自己写的根构成，恢复落到的版本在里面，「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」「系统配置槽在根槽持久之后再更新」（`.claude/kb/decisions/16-发布语义.md` 第 181 行）；系统配置先于它见证的根落盘的态，照条款的写序造不出来，三条都放行 |
| 269（见证值取成槽世代号） | 流一第 0、1 条 op（mkfs 之后第一次取号写的两块盘系统配置槽）偏移 469 一字节 0x00 → 0x01，连同校验和（155–158）；流二另有第 92、93 条 op（可写挂载取号那一写）0x04 → 0x06。偏移 469 是 journal tail（`crates/singlefs-checker/src/lib.rs` 第 200 行 `SYSTEM_CONFIGURATION_TAIL_OFFSET`）；调用结局同；看 journal 那一遍恢复逐态同 | 镜像与基线不同的态：流一 31 / 0，流二 78 / 14（全持久态相同：之后的发布把那一槽写成正确的 tail） | O1：`ObservedRoot` 不带系统配置，恢复不先信 tail（D23（journal 的角色与格式） 已定项 3），读回同；O2：读了 tail（`crates/singlefs-checker/src/lib.rs` 第 285 行），没有一条不变量判它的值；O3：执行器只在锚点断言里读 tail（`scripts/experiments/e170_second_run_o3_spec_executor/executor_main.py` 第 180 行，`anchor_assertions` 里），只核全持久态，判定不用它 | **真盲区候选**：D23（journal 的角色与格式） 已定项 14 写「取号那一写……改写取号那一刻按同一取法读到的见证值 c_见证」（全部自证过的槽 journal_tail 取最大）、「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行）；tail 写成 1 的系统配置槽不在条款写得出的镜像里，三条都放行 |
| 270（发布在系统配置轮换之后不过屏障就返回） | 每次发布末尾那道屏障没了：流一 57 → 53 条 op、流二 595 → 565 条，之后的返回点整体前移；写的内容同；系统配置轮换与下一次调用的第一批写切进同一段 | 一次调用的系统配置写没落全、之后的调用的写已落：流一 7 / 6 态，流二 70 / 59 态；基线 0 | O1-崩溃判官：那次发布的根槽 FUA 已落，恢复择到它，判绿；O1-对拍：调用结局同；O2：系统配置落后根一次发布在单个镜像里合法（I-7.12 只比 F）；O3：「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」（第 181 行），射程「屏障口径」写明系统配置轮换之后那一道屏障是「发布返回前加屏障」（第 186 行）；返回之后系统配置还没落、后面的写先落的态照条款造不出来，三条都放行 |
| 271（deleted_inodes 的树表条目头 ID 写 0） | 两条流第 44、45 条 op（树表单元，两块盘各 16 KiB）偏移 1647、1649 两字节 0x01 → 0x00（头 ID 65537 → 0），连同单元校验和；指向它的节点（第 48、49 条 op）与根槽（第 52 条 op）的指针与校验和随之变；流二之后重写树表的发布都带着（85 条 op 不同）；调用结局同；恢复逐态同 | 择到新版（根 txg 不小于第一处不同之后写出的第一条根）：流一 12 / 0 态，流二 356 / 112 态 | O1：不读树表；O2：树表条目只解树 ID、种类、诞生 txg（`crates/singlefs-checker/src/walk.rs` 第 2953、2955 行），头 ID 不判；O3：投影只带树表单元指针，A(c) 是这一行自己写的根，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 8 定案「头 ID 8 字节的取值：……`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md` 第 250 行）；择到新版的态里头 ID 是 0，三条都放行 |
| 277（取前缀施加一次发布之后新根段不换） | 写流与调用结局全同；看 journal 那一遍恢复在流一批 1 6 态、流二批 1 51 态（批 2 0 态）不同：基线施加 1 条记录、落到新的一版，变异同样施加 1 条、却停在所选根那一版（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 0：57 态里被丢掉的那一版的调用都没返回、它的根槽都没落（`extra_checks_f4.py`：全是 `call_returned=False its_root_slot_landed=False`） | O1-崩溃判官：前一版是模型提交过的、不比已持久的最新根槽旧、读回与那一版相同，判绿是对的；O2、O3：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段**：调用没返回，新旧两版都在「根 + journal 任意前缀」（D16（发布语义） 已定项 12）里，态本身合法；错在实现的恢复施加记录之后新根段不换，违反 D13（验证路线） 已定项 1 的 M4 与 D23（journal 的角色与格式） 已定项 15（同第 27 行），三条不判；M4 的落点是 E165 与 Verus。会分开的历史（推的，没跑）：一次已返回的发布之后根槽读不出、恢复要靠 journal 重放追上那一版（D16（发布语义） 已定项 7 射程「同一实例里退一代，靠 journal 重放追得上」），变异会停在前一版；两条流里没有读坏根槽的态 |

- **第 233 行（由记录重建的根不照抄所选根的文件系统标识，改写全零），只在丙里全放行**：写流与调用结局全同；看 journal 那一遍恢复在施加了记录的态上走读失败（`InvariantViolated { invariant: "I-1.4", detail: "树根 fsid 不符" }`），流一批 1 3 态、流二批 1 48 态（其中 6 态写落在卸载里、崩溃判官记不适用）。甲、乙里 O1-崩溃判官判红（流一 3 态、流二 42 态，段为正 2/12、28/79），只靠 O1；丙只取 O1-对拍，调用不经恢复，三条全放行。在丙里是**真盲区候选**：实现在合法的崩溃镜像上恢复不出任何一版，D13（验证路线） 已定项 1 的 M2 写重放结果的根「其余字段等于所选根」（`.claude/kb/decisions/13-验证路线.md` 第 32 行）；O2 判恢复之前的镜像（与基线逐字节相同），O3 跑的是它自己的恢复。层 0 那份 oracle 与记录核对器在这一行上红（`layer0_or_record_checker_red=true`，不是三条之一）。批 2 不红：两条流批 2 一个「基线施加了记录」的态都没有（`extra_checks_f4.py` 输出 `b2_base_records_applied_gt0: 0`），**取样点不敏感**，会分开的取样点是任一个施加了记录的态；这一格两批翻面，记「不稳定」。
- **第 265 行（抬 F 那一串不先写系统配置），批 2 多出来的全放行**：只有流二第 440 条 op 起不同（抬 F 先写系统配置那一步整步没了，149 条 op 按下标比不同）；O2 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 59、71 段各 6 态（`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2 total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`）。批 2 在流二取的段是 7、11、16、20、24、28、32、36、40、44、48、52、56、60、64、68、72，没有第 59、71 段（`extra_checks_f4.py` 输出 `batch2_states_in_segments_59_71=0`）：**取样点不敏感**，同第 102 行；记「不稳定」。条款是 D16（发布语义） 已定项 7 定案「抬 F 那一串……先把新 F 写进每块盘的系统配置、过一道屏障」。Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 15 行，只靠一条的行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:177-193`（整段抄，未转述）**

```markdown
### 三轮判决之后的三分

三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`；第三轮是最终轮，以它第三节「三轮之后的结论」与第五节为准）把这批读数重新三分。口径：**真盲区候选**＝这一批取到的态在已定条款下不合法、三条都放行（E171 s4 树）；**另几类**＝三条按设计不判、或错不在态上。执行员原判「对这一批等价」的五行（第 15、27、34、112、277 行）一行都不再是等价。

| 类 | 行 | 一句话依据（各行条款原文在上面两节的行里） | 三轮里的翻动 |
|---|---|---|---|
| 真盲区候选 | 第 15 行 | D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：变异下十一个提交内生块整体挪 64 槽（`research/prompts/e171-conclusion-r3-opus-model/out/row15-offsets.out`），⑤ 与 `layout/01-first-txn.md` 第 307 行钉的是这些落点 | 第一轮从「对这一批等价」挪进来（D3 已定项 10 ③）；第二轮维持，依据写 ⑤ 落点表与 `layout/01-first-txn.md` 第 309 行的 t1@50180；第三轮留名单、依据换成 ⑤ 的 t2–t12 落点（主 agent 现查 ⑤ 表 t2@50240 与第 220 行「改新池新建文件的字节：是」，复跑 `row15-offsets.out` 同） |
| 真盲区候选 | 第 23、42、271 行 | 第 23 行 D8（核心索引结构） 已定项 6 的容器号；第 42 行 D16（发布语义） 已定项 1 卸载抬 F；第 271 行 D8（核心索引结构） 已定项 8 头 ID | 三轮都在名单 |
| 真盲区候选 | 第 242、269、270 行 | 第 242、270 行 D16（发布语义） 已定项 7 的写序；第 269 行 D23（journal 的角色与格式） 已定项 14 的见证值 | 第一轮被改出名单（按已定项 12 合法、写序义务无人判），第二轮翻回，第三轮没被打中 |
| 另一类 | 第 27、112、277 行 | 三条不判实现恢复里靠记录重建的那一段（D13（验证路线） 已定项 1 的 M4、D23（journal 的角色与格式） 已定项 14 注 1 与已定项 15）；第 112 行已在 `.claude/gate.d/e165-pinned-red-examples.tsv` 里 | 第一轮采正推改归这一类，第二轮站住、措辞收窄，第三轮没变 |
| 另一类 | 第 34 行 | 分配政策，镜像合法：D3（空间分配） 已定项 8 射程 | 第一轮与第 15 行一起挪进真盲区候选，第二轮翻回，第三轮维持 |
| 另一类 | 第 92 行 | D8（核心索引结构） 已定项 6 已补写入时间取值（2026-10-04）；三条 oracle 按设计判不到，由钉写的用例判 | 第一轮归「条款没定且不可见」不进名单，第二轮改理由，第三轮维持 |
| 另一类（零轮） | 第 110、162、174 行 | 运行时读数（写账、分支名），三条按设计都不判，与第 34 行同类 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |
| 另一类（零轮） | 第 49、252 行 | 交回值或内存状态错、写流不变，要更长的历史才分得开 | 同上，第三轮新冒出 |

第 155、161 行是已知的那两行（O1-对拍判红），不在这几类里；第 233 行（丙里全放行、甲乙里 O1-崩溃判官抓到）、第 265 行（批 2 取样点不敏感）的读法在「F4 新出的全放行行」一节，第 102 行（批 2 取样点不敏感）在「三条全放行的 7 行」一节末，三轮没有改。「另一类（零轮）」两行的意思是这一类是第三轮才冒出来的，没有再开轮，进欠账，不改登记的触达定义。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:194-214`（整段抄，未转述）**

```markdown
### F4 新出的只靠一条的 12 行

只写够不到的对象与没判的字段，不判结论。读数取 F4 的导出目录（草稿脚本 `caught_rows_f4.py`，第七段 `caught_rows_s4.py` 改输出根，不入库）与产物的 `name=trajectory`、`name=row_matrix` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 172（交进来的盘数等于 w 的下限也拒） | O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:WritableDeviceCountBelowTheStripeWidthLowerBound`，模型说该成；流二停在第 92 条 op | 被拒之前的写流与基线逐条相同（0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，O2、O3 没有对象可判（同第 45 行） |
| 233（由记录重建的根不照抄所选根的 fsid） | O1-崩溃判官：批 1 流一 3 态、流二 42 态走读失败（I-1.4 树根 fsid 不符），段为正 2/12、28/79；批 2 0 态 | 错在实现的恢复，写流与基线逐字节相同；O2 判恢复之前的镜像，O3 跑它自己的恢复。批 1 与批 2 红的列不同：批 2 没有施加了记录的态，取样点不敏感（「F4 新出的全放行行」一节） |
| 237（解系统配置槽时根环起点读错了位置） | O1-崩溃判官：批 1 流一 53 态、流二 73 态（段为正 12/12、16/16），批 2 7、14 态，「没择到根」（`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`）；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（0 条 op 不同）；O2、O3 各用自己的解析器读系统配置，看到的镜像与基线逐字节相同 |
| 249（一个数据单元的发布也切成两条记录） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:jsn`（模型说那一版的 jsn 3，实现 4），模型停下 | 两条记录合起来仍是一次完整的发布：O2 在这一行 0 态红（没逐条查 invariants.md 有没有限一次发布几条记录的）；O3 自己重放记录，落到的版本在 A(c) 里，投影不带 jsn |
| 250（数据单元的净荷字节倒序写） | O1-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（读回 3000 字节、内容倒序） | 单元的校验和按写下去的字节算，结构自洽：D13（验证路线） 已定项 6 表 O2 行「验不到」一栏写的「一个块被覆盖成另一份合法内容」；O3 的投影不带文件内容，不拿写入的内容去比 |
| 281（择根取第一个自证过的槽就不再换） | O1-崩溃判官：批 1 流一 41 态、流二 61 态（段为正 9/12、13/16），批 2 7、14 态，「走到实例 0 第 0 代根」（比盘上已持久的最新根槽旧）；O1-对拍：流二第 4 次调用 `mount_writable` 交回 `err:NewerStateStillUnreadableAfterOneReread` | 错在实现的择根，写流与基线逐字节相同（0 条 op 不同）；O2 判镜像结构，O3 用自己的择根规则恢复，看到的镜像与基线相同 |
| 135（复活 defer 里那一条时 defer 槽数不减） | O2：I-3.11（已分配减 defer 等于最新根走读）批 1 流二 225 态，段为正 44/79；批 2 70 态 | O1：`ObservedRoot`（`crates/singlefs-harness/src/model.rs` 第 594 行）没有记账行；O3：投影不进记账树 |
| 234（根记录写者不照卸载记号写 flags） | O2：I-7.9（回退下界 F 不高于抬 F 的上限，按卸载记号分两支）批 1 流二 29 态，第 73 段起段为正 6/79；批 2 7 态 | O1：写落在卸载那次调用里，模型没有卸载的答（登记读法 6），对拍不比、崩溃判官记不适用，`ObservedRoot` 也不带根的 flags；O3：投影不带卸载记号（`root_ring.py` 第 22 行起的说明：「卸载记号与根记录其余字段不进投影」） |
| 238（改动计数留 1） | O2：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1：不比 inode 记录的改动计数；O3：投影不进 inode 树 |
| 245（覆盖写的反向链取上一版记录自己的反向链） | O2：I-8.6（反向链不等的记录不进重放）批 1 流二 336 态，段为正 67/79；批 2 105 态 | O1：`ObservedRoot` 只带 journal 计数器，不带反向链；O3：它的前缀规则核反向链（`scripts/experiments/e170_second_run_o3_spec_executor/replay_prefix.py` 第 188 行，不等就停），这一行严格、暂代都判「在」；停下之后落到的那一版仍在 A(c) 里是推的（根槽已落、不靠重放），没逐态查它停在哪 |
| 258（不释放旧落点） | O2：I-3.1、I-3.11 批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1：`ObservedUnitAllocationRecords` 只带这一版每个角色的单元的分配记录（`model.rs` 第 638 行），旧落点释没释放不在里面；O3：投影不进分配记录树与记账树 |
| 265（抬 F 那一串不先写系统配置） | O2：I-7.12 批 1 流二 12 态（第 59、71 段各 6 态）；批 2 0 态 | O1：比的是根带的 F，不比系统配置槽；O3：投影的 F 取自根，A(c) 是这一行自己写的根。批 1 与批 2 红的列不同：批 2 没取第 59、71 段，取样点不敏感（「F4 新出的全放行行」一节） |

O3 在 F4 计入分母的 37 行上一行都没红（`name=q1_column_red_rows … O3甲=0 O3乙=0`），每行两条流的批 1 态都给了严格判定（`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:215-229`（整段抄，未转述）**

````markdown
### 没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行

`name=untouched_but_red rows=2 row_numbers=155,161`：两行的变异是同一处（`crates/singlefs-core/src/mount.rs` 里抬 F 上限 `newest_on_every_device.min(fourth_newest)` 改成 `.max(…)`，两行原文只差行首空白），触达三样都不同（`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`），O1-对拍在流二第 12 次调用 `raise_floor` 判 `disagrees:抬 F 的上限`（`call-outcomes.tsv`：模型「上限 Some(11)（D16（发布语义） 已定项 1）」，实现「实现报上限 13」），批 1、批 2 都红；O2、O3 不红。

- 不是装置、比对器或取样的缺陷：比对器的触达照登记第一节读法 4 字面算（「任一次调用的结局（成功 / 哪一种错误）」，`collusion_matrix.py` 的 `outcome_words` 只比调用名与结局列），这一次调用两边都是 `ok`；O1-对拍比的是成功的调用交回的值（这里是抬 F 的上限），写流不变是因为这一次请求的 F 不超过 11，上限放宽没起作用。O1-对拍按调用算，与取样点无关，两批都红。
- 缺口在登记读法 4 的触达定义：它把「成功的调用交回的值」排在三样之外，却写「三样都相同的行……在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红」；这两行交回值不同，那句前提不成立。装置的 `call-outcomes.tsv` 只在对拍判出不同时记两边的值，成功的调用交回什么没有单独记，入库装置没有这一列。触达定义没改（产物跑过之后不改登记），分母仍 37。
- 别的未触达行有没有同类差异：攻方（第三轮）在 s4 树的副本上量了（副本探针，s4 树，入库装置没有这一列）。探针照流二的驱动在内存稀疏盘上跑一遍，记每次成功调用交回值的 `{:?}` 文本摘要、末了的分配器与现行那一版；基线与 E171 导出的 595 条 op 逐条相同；再把未触达的行逐行打进副本、重编重跑、还原。跑了 232 行（238 行去掉 `make_filesystem.rs` 的 6 行，mkfs 不是一次调用，探针不记它的交回值，第 149–154 行没跑），7 行不同：第 49、110、155、161、162、174、252 行。第 155、161 行就是上面那两行（这把尺子在同一次第 12 次调用 `raise_floor` 上抓到它们，上限 11 → 13，是现成的阳性对照）；另 5 行 O1-对拍不比那几个字段所以不红：第 49 行可写挂载交回的见证值 4 → 3，第 110 行写账的结构种类，第 162 行影子账分支名，第 174 行抬 F 交回的写账，第 252 行内存里的分配器少了根槽登记（第 49、252 行交回值或分配器不同，写流不变）。汇总行整行抄自攻方 `research/prompts/e171-conclusion-r3-opus-model/out/return-rows.out` 第 239 行：

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

  主 agent 复跑（`tools/rerun.sh … all`）这 7 行逐字同；`return-rows.out` 只差变异表第 28–31 行那四条 `name=return_row`（复跑是 `status=no_output`、汇总 `hidden_difference=7 identical=221 no_output=10`，攻方是 `identical`）：攻方在决定去掉 `make_filesystem.rs` 的行之前已跑过这四行，`rerun.sh` 的 `awk` 把它们滤掉，复跑就不再跑（`research/prompts/e171-conclusion-r3-main-verification.md` 第四节）。害处没量：第 49、252 行推的是要更长的历史才分得开。
- 对其余读数的影响：两行不进分母，也不进任何 Q1、Q3 名单。照「触达」把它们算进去，D = 39，两行都只有 O1 红，Q3 只靠 O1 的行多两行（第 155、161 行），Q1 三种读法的全放行行不变。把 7 行交回值不同的行与第 125 行（「它答不了的」）都算进来，D 由 37 变 45（推的，算术，没在入库装置上跑），那 5 行 O1 不红，会成为新的全放行行。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:230-257`（整段抄，未转述）**

````markdown
### 编不过的 6 行补位（已跑）

登记第一节读法 5「替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）」；s4 与 F4 那一版驱动只在编之前的失败上补位，第 33、59、127、163、206、243 行编不过之后照原样记编不过、没有补位。第八段把驱动改成编不过之后也补位（换同一文件组里下标 j + 1、两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点，编不过再换，编之前的失败与编不过合起来一行至多三次；组里不含 E171 装置自己的变异行），加 `--rerun-compile-failed-from`（只跑几个输出根里结局是编不过的行）；比对器加 `--substitution-rows-root`（补位那一趟的行换掉原来编不过的同号行，别的情形拒），多出 `name=substitution` 与 `name=substitution_summary` 两种结果行。

- 补位那一趟在 s4 那一份冻住的树上跑（快照摘要与 s4 五趟的相同，空变异一行对 `baseline-run1` 未触达），只跑两份输出根里结局是编不过的行（`rerun_compile_failed rows=33,59,127,163,206,243`），两条线道、每条 2 线程，六步都退 0，补位 6 行挂钟 1 277.9 秒；比对器把 117 + 164 + 补位并成 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（4795 行，末行 `E7RESULT name=done emitted=4795`），耗时行在 `…-s4-f4-substitutes-rows-timing.out`（292 行）。
- 补位的六行（整行抄自 `…-s4-f4-substitutes-comparator.out` 第 4625–4630 行）、补位汇总（第 4631 行）与分母（第 4521 行）：

```
E7RESULT name=substitution row=33 original_name=步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 original_status=compile_failed substituted_by=步_4：影子账把被抛弃根账里已释放的落点也隔离 failed_attempts=1 attempts=compile_failed:步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 category=untouched
E7RESULT name=substitution row=59 original_name=C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） original_status=compile_failed substituted_by=C483_②：挂载态打开时_inode_叶容器只按位置提示读（不经映射回退） failed_attempts=1 attempts=compile_failed:C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） category=untouched
E7RESULT name=substitution row=127 original_name=增补_2：回落只看已分配位（发出影子账隔离的槽） original_status=compile_failed substituted_by=增补_2_第_20c_行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放） failed_attempts=2 attempts=compile_failed:增补_2：回落只看已分配位（发出影子账隔离的槽）|compile_failed:增补_2：回落只看已分配位（抬_F_的空发布发出扣住的槽） category=untouched
E7RESULT name=substitution row=163 original_name=C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） original_status=compile_failed substituted_by=D8_已定项_8_②：可写挂载写行那次的树_ID_水位只沿所选那一版带，不取根环与环里记录的_max（原为_C511_第_3_步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护） failed_attempts=1 attempts=compile_failed:C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） category=untouched
E7RESULT name=substitution row=206 original_name=步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） original_status=compile_failed substituted_by=增补_3_第_4_件：一块盘两个系统配置槽都无效就整体_return_Err（每盘一份的冗余当场作废，D22_已定项_8_第_1_条） failed_attempts=1 attempts=compile_failed:步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） category=untouched
E7RESULT name=substitution row=243 original_name=并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 original_status=compile_failed substituted_by=普查_R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员） failed_attempts=1 attempts=compile_failed:并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 category=untouched
E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

- 六行换上的都是 `category=untouched`（第 127 行前一个换上的行也编不过，换了第二次）；分母 D = 37 没变：`compile_failed` 6 → 0、`untouched` 232 → 238，`name=f6_table_failures` 的 `rows_not_counted_for_the_table` 12 → 6（只剩六行实现自崩）。补位产物比补位之前的 F4 产物多出来的行只有这六行的 `name=row`、`row_touch`、`row_matrix`、`row_s5`、`row_o3` 与 `trajectory`，加 `name=substitution` 六行与 `name=substitution_summary`，改了的行是 `name=config`、`name=q1_denominator`、`name=f6_table_failures` 与 `name=done`。
- Q1、Q3 名单与补位之前逐字同（自己 diff 两份产物核）：

```
cd research/results; A=e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out; B=e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out; P='name=(q1_alpha|q1_beta|q1_gamma|q1_quiet_row|q1_column_red_rows|q3_alpha|q3_beta|untouched_but_red|q1_denominator_by_table|q1_delta|q1_o3_coverage) '; diff <(grep -E "$P" $A) <(grep -E "$P" $B); echo "diff_rc=$?"; grep -cE "$P" $A; grep -cE "$P" $B
```

  输出 `diff_rc=0`、`48`、`48`；`name=q2_`、`name=f3_`、`name=verdict`、`name=f4_expansion`、`name=q1_denominator_by_table`、`name=positive_control`、`name=freeze_check`、`name=q3_core_row` 开头的行两份也逐行同（同一条命令把 `P` 换成 `'name=(q2_|f3_|verdict|f4_expansion|q1_denominator_by_table|positive_control|freeze_check|q3_core_row)'`，输出 `diff_rc=0`、`171`、`171`）。
- 拿改过的比对器（不给补位根）重跑 s4 与 F4 的输入，E7RESULT 行与两份产物各自逐字节相同；拿一个手造的补位根（一行换成第 34 行的导出、一行仍编不过）空跑过一遍，分母与名单照换过的行算、两种新结果行照写（草稿，不入库）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:258-286`（整段抄，未转述）**

````markdown
### Q2 里 O2 放行而 O3 判不在的坏镜像（整行抄自产物）

`…-s4-f4-substitutes-comparator.out` 第 4759、4760 行（补位之前的 `…-s4-f4-comparator.out` 第 4662、4663 行同两行，`sed -n '4662,4663p'` 与 `sed -n '4759,4760p'` 各自喂 `md5sum` 都是 `07b56b24014dac5d9eb543ef15384aa9`）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 张是 O2 放行而 O3 严格判「不在」。从 `name=q2_row` 逐行数同一批、按补丁种类分（命令与原样输出；`B` 是上面那份产物）：

```
grep 'name=q2_row family=[PB] ' B | grep ' o2=green ' | grep -c 'o3_strict=not_in'
29
for f in P B; do echo "== $f"; grep "name=q2_row family=$f " B | grep ' o2=green ' | grep 'o3_strict=not_in' | sed -E 's/.*bad_name=([^ ]*) .*/\1/' | sed -E 's/^B[0-9]+:(P[0-9]+):.*/\1/; s/^(P[0-9]+)-.*/\1/' | sort | uniq -c; done
== P
      1 P2
      4 P3
      4 P4
      1 P5
      2 P9
== B
      4 P2
     12 P3
      1 P5
```

P 族 12 张是 5 种补丁（P2、P3、P4、P5、P9）的实例，盲测 B 族 17 张是 3 型（P2、P3、P5）的随机实例；全是结构自洽的伪造根、伪造续接镜像。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:287-389`（整段抄，未转述）**

````markdown
### 第一段的基线与对照（整行抄自 s1 产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out`（基线第一趟；两趟的 E7RESULT 行逐字节相同，`diff` 无输出）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=completed exported=true not_exported_why=- operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82 model_stopped_at_call=none
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 o1_crash_judge_red=0 o1_crash_judge_green=53 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=7 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=completed exported=true not_exported_why=- operations=595 writes=477 segments=78 tearable_writes=46 calls=15 full_states=1662648564 quick_tier_states=278 sampled_states=890 model_stopped_at_call=14
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 o1_crash_judge_red=0 o1_crash_judge_green=349 o1_crash_judge_not_applicable=48 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=105 o1_crash_judge_not_applicable=14 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out`，基线与对照（Z1 两趟 26 个文件逐字节相同、S2 与 E170（O3规约执行器第一版） 导出装置 18 项逐字节相同、S5 两个数比对器自己现算、A3 基线 0 红）：

```
E7RESULT name=a4 table_rows=117 expected=117 per_file_matches_the_registration=true found_once_in_today_table=117 line_unchanged_since_the_registration=117
E7RESULT name=a5 first_slice_rows=24 expected=24 distinct_files=24
E7RESULT name=a6 step_8_rows=117 step_4_rows=220 expected_step_4=220 overlap=56 expected_overlap=56 step_4_only=164 expected_step_4_only=164 step_8_names_equal_the_registered_table=true
E7RESULT name=a7 implementation_rows=844 expected=844 files=24 expected_files=24
E7RESULT name=a3_baseline_reds o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
E7RESULT name=s5_batch_1_states stream=new_pool_file_creation recomputed=53 device=53 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=1 recomputed=53 device=53 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=2 recomputed=7 device=7 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=3 recomputed=22 device=22 equal=true
E7RESULT name=s5_batch_1_states stream=fixed_script_through_e_then_unmount recomputed=397 device=397 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=1 recomputed=349 device=349 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=2 recomputed=105 device=105 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=3 recomputed=330 device=330 equal=true
E7RESULT name=z1_summary files_compared=26 files_differing=0
E7RESULT name=s2_summary files_compared=18 files_differing=0
```

阳性对照（登记 5.6，每条 oracle 在基线与空变异各一趟）与关一条的开关自检：

```
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=q3_core closed=O1 run=disable-O1 differences_outside_the_closed_column=0 where=none
E7RESULT name=q3_core closed=O2 run=disable-O2 differences_outside_the_closed_column=0 where=none
```

三行试跑的分类、触达三样与 Q1 的分母：

```
E7RESULT name=row row=0 row_name=空变异（C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式）） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=0 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=1 row_name=C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=1 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=6 row_name=K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） category=self_crash substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=6 write_stream_differs=true call_outcomes_differ=true recovery_report_differs=false ignored_pass_differs=false touched=true
E7RESULT name=q1_denominator rows_run=3 counted_D=0 untouched=2 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=1 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=1 rows_run=3 more_than_half=false
E7RESULT name=q1_o1_and_o2_both_quiet counted_rows=0 rows=0 row_numbers=none o3=not_run
E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows=0 rows_that_differ_from_batch_1=none
E7RESULT name=q1_column_red_rows counted_rows=0 O1=0 O1-对拍=0 O1-崩溃判官=0 O2=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
```

Q2 的 O2 一列（O3 两列、盲测族没跑）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=N rows=10 o2_red=2 o2_quiet=8 o2_quiet_rows=new:14,new:15,new:16,new:17,fix:14,fix:15,fix:16,fix:17
E7RESULT name=q2_family family=K rows=41 o2_red=41 o2_quiet=0 o2_quiet_rows=none
E7RESULT name=q2_blind_family rows=0 expected_when_run=40 status=not_run why=seed_needs_the_frozen_executor_s7
E7RESULT name=q2_negative_bases bases=8 expected=8 o2_red_bases=0 red_base_list=none
E7RESULT name=a9_a10_a8 p_rows=26 p_rows_expected=26 n_rows=10 n_rows_expected=10 known_bad_rows=41 known_bad_rows_expected=41 blind_rows=0 q2_table_rows_when_the_blind_family_exists=107
E7RESULT name=q2_all_quiet status=not_run why=o3_frozen_version_not_available_s7 o2_quiet_rows_total=16
```

判决行（比对器自己的那一行；O3 一律 `not_run`）：

```
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=6 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 known_bad_rows_with_o2_quiet=0 o3_columns=not_run q1_alpha=not_run q1_beta=not_run q2_all_quiet=not_run q3_alpha=not_run
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out`（第一刀第 6 行：分配记录树按槽号找叶时叶宽算成两倍，实现在第一次 `publish` 里 panic，装置记「实现自崩」、这条流不导出）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=implementation_crashed:call_2_publish:分配记录（盘DeviceIdentity(0)槽SlotNumber(50176)跨2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 exported=false not_exported_why=implementation_crashed_in_a_call:call_2_publish:分配记录（盘_DeviceIdentity(0)_槽_SlotNumber(50176)_跨_2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 operations=22 writes=12 segments=7 tearable_writes=0 calls=3 full_states=0 quick_tier_states=0 sampled_states=0 model_stopped_at_call=2
E7RESULT name=call stream=new_pool_file_creation call=2 label=publish first_operation=22 return_point=22 status=panic o1_comparison=not_modelled
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=implementation_crashed:call_2_publish:分配记录（盘DeviceIdentity(0)槽SlotNumber(50176)跨2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 exported=false not_exported_why=implementation_crashed_in_a_call:call_2_publish:分配记录（盘_DeviceIdentity(0)_槽_SlotNumber(50176)_跨_2）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过 operations=22 writes=12 segments=7 tearable_writes=0 calls=3 full_states=0 quick_tier_states=0 sampled_states=0 model_stopped_at_call=2
E7RESULT name=call stream=fixed_script_through_e_then_unmount call=2 label=publish first_operation=22 return_point=22 status=panic o1_comparison=not_modelled
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out`（41 条已知坏镜像在流一全持久态上：补丁叠回与直接改坏判红的不变量集合相等、目标不变量都红、条目名与原文件逐条相同）：

```
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_build_pool local="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" crates="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" holds=true
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_the_full_state_of_the_dump local=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") crates=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") holds=true
E7RESULT name=s4 check=known_bad_test_file_sha256_is_the_copied_one local="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" crates="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" holds=true
E7RESULT name=s4 check=known_bad_entry_count_here local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_count_in_the_original_file local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_names_equal_the_original_file_in_order local=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] crates=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] holds=true
E7RESULT name=verdict known_bad_entries_failing_s4=0 e170_bad_rows_failing_s3_or_s4=0
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:390-396`（整段抄，未转述）**

```markdown
### 对照与自检

- 阴性（登记 5.6）：基线两趟、三批的每一态与每一次调用上，O1-对拍、O1-崩溃判官、O2 判红都是 0 处（A3，`name=a3_baseline_reds`）；空变异（拿表里第一行、把替换文换成原文本身，走完整条逐行驱动）判未触达、四列不红（`name=row row=0`、`name=row_touch row=0`、`name=row_matrix row=0`）；8 张不打补丁的底态（两条流的 F、Mk、Mk⁺、Mk⁻）O2 不红（`name=q2_negative_bases`）。O3（第二段起）：冻结的执行器在基线两条流批 1、批 2 的 576 态上判红 0 处，s4 产物原行 `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`；8 张底态 O3 甲乙都不红（`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`）。s4 整表开跑前，同一棵冻住的树上的空变异一行对 `baseline-run1` 未触达（主 agent 跑的整表脚本第 3 步：`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`）。
- 阳性（装置一侧三条，登记 5.6）：O1-对拍在流二第一次 `publish` 的观测里把那一版文件内容的末字节翻一位，对拍判出一处；O1-崩溃判官在每条流批 1 第一个「实现读回了文件」的态把读回内容末字节翻一位，每条流各一处、合计 2 处（登记字面是「第一个」，装置按每条流各翻一次，对照要的「红」没变）；O2 在流一全持久态上叠已知坏镜像族第一条，1 处。基线一趟与空变异一趟各跑一遍，红都一路带到 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`（比对器的 `name=positive_control` 六行）。O3 的阳性对照（第二段起）：流一全持久态叠 P1 补丁交冻结的执行器，基线与空变异两趟各判红 1 处（s4 产物 `name=positive_control oracle=O3` 两行）。翻过的那次对拍让模型停在那次调用，此后的调用不比、写落在它之后的态 O1-崩溃判官记「不适用」，所以 O1-对拍那一趟的 `o1_crash_judge_red_states` 是 0。
- 停机条款：S1 本地常量与 crates 回比 42 条全过；S2 两条流的八个文件与 `impl-verdicts.tsv`（去掉耗时列）逐字节等于 E170（O3规约执行器第一版） 导出装置在同一棵树上 `dump` 写出的；S3 O2 直接调 `check_pool_image` 与 `judge_crash_image` 内那一遍判红的不变量集合在基线每个取样态上相同（`o2_matches_judgement_false=0`），在 76 行坏状态上与 E170（O3规约执行器第一版） `bad-impl-verdicts.tsv` 池级 checker 那一列逐行相同；S4 已知坏镜像族 41 条、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的、补丁叠回与直接改坏判红集合相等、目标不变量都红；S5 比对器自己现算的两个数（批 1 的态数与序号集合、O1-崩溃判官适用的态数）与装置的计数相等；S6 没触发：E170（O3规约执行器第一版） 导出装置在今天的树上 `dump` 退 0、36 条 `holds=true`、0 条 `holds=false`；S7 在第一段触发（O3 列不跑），第二段起 O3 用登记修订 2 抄下冻结行的那一版，s4 比对器开头与收尾两次冻结核对都是 0 处不同；S8 的三处调用在「装置、研究一侧与变异表」一节给了行号。逐行那一趟的 S5（登记修订 3、4）在 s4 117 行上 0 行不过：判决行 `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`，`name=q1_denominator … device_stop=0`。
- 被测的是跑那一刻的树：s4 的空变异、117 行与 Q3-核三趟从同一份冻住的树拍快照（五趟驱动的 `snapshot_digest` 只有一种，`d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`）；开跑前树核对拿它与第六段读码时的清单逐文件比，只差一个别的 bin（`e168_delete_ring_window_borrow.rs`，不挡）。第一段的基线、对照与试跑用的是 `crates/` 在跑那一刻的状态；E170（O3规约执行器第一版） 导出装置一侧用的是 `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 取出的暂存版本（sha256 `8c526293…`，登记第三节钉的那一份）在草稿目录里编出来的，它的 `dump` 与 `bad-states` 输出与工作区里 E170（O3规约执行器第一版） 第 1 次重跑改过的版本逐字节相同（`cmp` 无输出），所以复跑驱动直接用工作区里的版本。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:397-402`（整段抄，未转述）**

```markdown
### 单测与变异

- Python 一侧单测 110 条（研究一侧目录里 `grep -c '    def test_' test_*.py` 四个文件 32 + 29 + 24 + 25），`python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` 连跑 5 轮每轮 `Ran 110 tests` / `OK`（手造数据、确定性，5 轮只说明没有隐藏状态）：矩阵（触达三样、O3 两列的读法、全放行谓词、关一条、门槛、轨迹）、切段与取样公式现算、导出目录读回、坏实现表（锚点、补位、空变异）、挑法脚本的行数、冻结核对、坏镜像的 O3 判法、逐行驱动的编译记录与重编核对（真 `rsync -a` 与 `copy2`、假 cargo）、F4 扩表（挑法脚本两次的差集与编号、比对器的逐行核对、两个输出根的合并与重号拒绝、分母按表分开、补位跳过两张表的名字）、编不过之后的补位（手造组 r0–r5、表里有 r0 与 r2、装置行插在中间：编不过依次换到 r1、r3、r4，第四次补不上记编不过；换上的行锚点不过与编不过合算三次；空变异不补位；组里没有装置行；驱动在同一个行目录里重编换上的行、`compile-failed-1.log` 与 `failed_attempt_1` 同号、复跑开关不补位；比对器只拿补位行换掉编不过的同号行、别的情形拒、补位行的结果行写换上的名字）。每个会被变异弄坏的谓词都配了钉绝对值的断言（例：扩表在手造表上必须是 `(118, a4)`、`(119, a12)`、`(120, b2)` 三行；补位的整行 `name=substitution row=3 … substituted_by=c_换 failed_attempts=1 …` 写死）。
- Python 一侧变异 67 条，表 `scripts/mutations/e171_three_oracle_collusion.tsv`，跑法 `mutation_driver.py`（登记第九节写「经 `scripts/mutate.sh` 跑」，那条脚本只起 cargo test，起不了 Python 的单测，登记修订 1 第 1 行记了这一处）：`e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 第 137 行 `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`，抓到 / 无效 / 没红 = 67 / 0 / 0（结局符号逐个数：✅ 67、⏭ 0、❌ 0、⏱ 0、💥 0、⚠️ 0、🧱 0）。X60–X74 是编不过补位那一批（编不过不补位、不计次数、组里算进装置行、换上的行锚点不过时结局不记编不过、补位那一趟领了不是编不过的行、不记试过的名字、编不过之后不换行直接跑装置、比对器不核原行是编不过、不核补位自的名字、结果行写原名、合计把补不上的算换上、行分析不套补位、日志与试过的名字不同号、复跑开关不起作用、补位那一趟同一行在两个根里不拒）。同名的 `…-r6.out` 是同一批的前 13 条加进来时的全表跑（65 条，65 / 0 / 0），之后补了 X73、X74，留档。按「没红的三类」想过而没加的一条：编不过之后的补位从表里原名往下找、不从编不过的那一行往下找——补位跳过表里与已用过的名字，原名与编不过那一行之间的每个名字都在这两样里，两种找法在所有输入上同值，是等价变异，没加进表，等价性写成一条留档测试（`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`，手造组上逐个起点核）；加了这条测试之后变异表重跑一遍，输出与 `-r7.out` 逐字节相同。X52–X59 是 F4 扩表那一批（扩表不去掉重叠行、从 117 编号、把装置自己的变异行算进挑法、两个输出根同号不拒、补位不跳过别的表、扩表核对只比行数、分母按表对调、只读第一个输出根），每条都由 `F4ExpansionTests` 里对应的那条单测抓到。同名的 `…-r4.out` 是同一张表的一次中途被停的跑（外面套的 `timeout 230` 在第 8 条变异时停了它，61 行、没有判决行），不算数、留档。更早的 `…-s1-comparator-mutations.out`、`…-s2-comparator-mutations.out`、`…-s3-comparator-mutations.out` 与 `…-s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` 是表在 9、26、37、40、44、44 条时的全表跑，各自全抓。
- 装置内联单测 31 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）：编过（`scripts/compile-then-swap.py` 的 `cargo build` 与 `cargo test --no-run`），没跑（checker 档，归提交）。装置一侧变异 15 行追加在 `crates/mutations.tsv` 末尾（名字以「E171 装置」起头），三个数待提交时门禁 checker-tier-crates-mutation-replay。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:403-415`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 批 1 的态数（装置） | 取样三批的公式作用在层 0 枚举表给的每段序号区间上，再加撕裂扫描；枚举点，不是全量 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1346` `sample_ordinals` | `layer0_enumeration_tables` 的段区间与哪几次写取三态（来自 `crates/singlefs-checker-tier/src/crash.rs`） |
| 2 | 批 1 的态数（比对器现算） | 从 `ops.tsv` 按切段规则切段、`torn.tsv` 的写序号当三态写，每段状态数 = 各写取几态的乘积减 1，再按同一套取样公式现算序号集合 | `scripts/experiments/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | 取样公式（E170（O3规约执行器第一版） 登记 5.4 的文字）；`torn.tsv` 与 `ops.tsv`（装置从 crates 的枚举表与写表写出）：两条路径共用「哪几次写取三态」 |
| 3 | O1-崩溃判官适用的态数（装置） | 一态里持久或撕裂的写属于的最后一次调用不早于模型停下的那次调用，这一态不适用 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1920` `crash_judge_is_applicable` | 调用表的返回点与写表的流序下标 |
| 4 | O1-崩溃判官适用的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 的落地向量数写落在 `unmount` 那次调用里的态 | `scripts/experiments/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | `calls.tsv` 的返回点（装置写出）；模型只停在 `unmount` 这一条读法（登记第一节读法 6） |
| 5 | O2 在坏状态行上的判定（装置） | 从中性格式按落地向量重建 `MemoryPool`、叠 E170（O3规约执行器第一版） 写的补丁、调 `check_pool_image` | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4646` `e170_bad_rows_on_both_streams` | 池级 checker 本身、E170（O3规约执行器第一版） 写的 `bad-patches.bin` 与落地向量 |
| 6 | O2 在坏状态行上的判定（E170（O3规约执行器第一版） 导出装置） | `CrashImage` 上把补丁接在写表末尾当持久的系统配置槽写，调 `judge_crash_image` 取池级 checker 一项 | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:2994` `observe_bad_row` | 池级 checker 本身；同一批补丁字节与落地向量 |

路径 1 与 2、3 与 4 在产物里逐格比（`name=s5_batch_1_states`、`name=s5_o1_applicable` 八行 `equal=true`），路径 5 与 6 逐行比（`name=e170_bad_row` 76 行 `o2_equals_e170=true`）；这几处比对写在产物里，没有 `assert` 把两条路径直接连起来，还欠这一条。共用项里最要紧的是池级 checker 本身（路径 5、6 同一份 `check_pool_image`）：它错了两条一起错，所以这一格不能当 O2 判得对的证据；路径 1、2 共用「哪几次写取三态」，它错了两份态数一起错。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:416-535`（整段抄，未转述）**

````markdown
### 复跑

装置与 E170（O3规约执行器第一版） 导出装置都是 checker 档的入库装置，经内存包装起；研究一侧是 Python。`scripts/replay.sh` 里 E171 的十一行登记，各带自己的临时目录，产物只留 E7RESULT 行、exact 比：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | 装置 `baseline` 一趟 |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | 基线、E170 导出装置 `bad-states`（占位种子）、装置 `known-bad-images` |
| `driver_e171_s2_bad_states_export`、`driver_e171_s2_known_bad_images`、`driver_e171_s2_images_o3`、`driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`、`…-s2-known-bad-images.out`、`…-s2-images-o3.out`、`…-s2-o3-control-baseline.out` | 坏镜像三族在冻结种子下的造法、O2 与冻结执行器的判定、O3 阳性对照 |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | 第二段比对器那一串（九行试跑）；比对器在第三至六段改过判决行与 config 行，这份今天复跑对不上，没改指 |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | 研究一侧 67 条变异，`mutation_driver.py`，约一分钟 |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | 第二段那一串前置、修过的逐行驱动跑全部 117 行（两条线道、每条 2 线程，一份冻住的树）、Q3-核三趟、比对器；推的约两小时，没真跑过这个驱动函数（重型，归提交时的全部实验复跑）。117 行的读数读今天的 `crates/`，实现、O2、O1、装置或那 117 行的行文变了产物就变。整表那一步带 `--keep-compile-failures-as-they-are`：这份产物是编不过不补位的那一版驱动跑的 |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | 先整条跑 `driver_e171_s4_comparator`，再在它那一份冻住的树与工作根上跑扩表 164 行（同样带 `--keep-compile-failures-as-they-are`），比对器并两半、带扩表、段名 s4-f4；推的约四小时，没真跑过（`bash -n` 过；比对器那一半用同一组参数在已有导出上重跑过、与产物逐字节相同） |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | 先整条跑 `driver_e171_s4_f4_comparator`（它的草稿目录留着 s4、F4 的导出与冻住的树），再在同一份冻住的树与工作根上只跑两份输出根里结局是编不过的行（`--rerun-compile-failed-from`，补位跳过两张表的全部名字，两条线道、每条 2 线程，不带 `--keep-compile-failures-as-they-are`），比对器并 117 + 164 + 补位、带 `--substitution-rows-root`、段名 s4-f4-substitutes；推的约四个半小时（s4 与 F4 那一串约四小时，加补位 1 277.9 秒），没真跑过（`bash -n` 过；各步参数照补位那一趟实际跑的命令逐条对过，没有重跑驱动或比对器）。读今天的 `crates/`，实现、O2、O1、装置或那六行的行文变了产物就变 |

拿 s4 的输入（已有的导出目录与驱动输出）重跑今天的比对器，E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）：比对器这一半复得出；装置与驱动那一半要整表重跑才核得到。第一段几样的单条命令：

```
bash scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash scripts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd scripts/experiments/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

第一段比对器那份（`…-s1-comparator.out`，第一段的 `driver_e171_s1_comparator` 已由 s2 那一份取代、不再登记）要先跑齐基线两趟、E170（O3规约执行器第一版） 导出装置的 `dump`、关一条两趟、阳性对照六趟（基线三趟，空变异三趟经 `row_driver.py --empty-mutation --positive-control <名>`）、逐行驱动三行（`row_driver.py --empty-mutation --rows 1,6`），再 `python3 comparator.py …`，大约十几趟装置（每趟约一分钟），驱动函数把这一串写死；比对器的 config 行只写目录的名字，所以复跑目录的名字要与驱动里的相同。比对器的 A4–A7 读今天的 `crates/mutations.tsv`：表里别处加了核心实现的行或改了那 117 行，这几行会变，不是装置坏了。占位种子 `0000000000000000` 造出的盲测行 E171（三oracle合谋测试） 不用；E170（O3规约执行器第一版） 在第 1 次重跑里加了第二套坏状态表（`bad-states-second-run`），`bad-states` 仍是第一套，两个版本的导出装置在第一套上的输出逐字节相同（「对照与自检」一节最后一条）。

复跑驱动的自检：基线、已知坏镜像、变异驱动三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同（`cmp` 无输出）；比对器那一个驱动函数没有整条真跑（十几趟装置，前台跑不完），它的胶水（各步命令行、目录的名字、传给比对器的参数）用「装置、E170（O3规约执行器第一版） 导出装置与逐行驱动换成从已有导出目录拷」的桩跑过，比对器照真的跑，输出与入库产物逐字节相同；各步命令本身在做产物时逐条真跑过。整条真跑由提交时的复跑做。

产物（都在 `research/results/` 下，名字前缀 `e171-three-oracle-collusion-2026-10-03-s1-` 之后接）：

| 文件名（前缀之后） | 内容 |
|---|---|
| `baseline-run1.out`、`baseline-run2.out` | 基线两趟的 E7RESULT 行（两份逐字节相同） |
| `baseline-timing.out` | 两趟的 E7TIMING 行（每批每段耗时、每条流驱动、判、写各花多久） |
| `known-bad-images.out` | 装置 `known-bad-images` 模式：41 条已知坏镜像与 E170（O3规约执行器第一版） 的 76 行坏状态上的 O2 |
| `comparator.out` | 比对器：登记锚点、基线与对照、关一条、三行试跑、Q1 分母、Q2 的 O2 一列 |
| `rows-timing.out` | 逐行驱动与比对器的 E7TIMING 行（Q4 单行数） |
| `comparator-mutations.out` | Python 一侧 9 条变异 |
| `e170-pinned-dump.out` | E170 暂存版本导出装置的 `dump`（S2 的对照） |
| `e170-bad-states-placeholder-seed.out` | 同一版本的 `bad-states`（P、N 行的出处，盲测行不用） |
| `positive-control-o1-comparison.out`、`positive-control-o1-crash-judge.out`、`positive-control-o2.out` | 三条阳性对照在基线那一趟的装置输出 |
| `disable-o1.out`、`disable-o2.out` | 关一条的开关自检（基线上各一趟）的装置输出 |
| `row-0-device.out`、`row-1-device.out`、`row-6-device.out` | 三行试跑的装置输出（row-0 是空变异） |

全名（`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`，17 份）：

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

第二段起的产物（`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`，29 份，名字前缀 `e171-three-oracle-collusion-2026-10-03-` 之后接）：

| 文件名（前缀之后） | 内容 | 现在算不算数 |
|---|---|---|
| `s2-bad-states-export.out`、`s2-known-bad-images.out`、`s2-images-o3.out`、`s2-o3-control-baseline.out`、`s2-o3-baseline.out` | 坏镜像三族的造法与 O2、O3 判定、O3 阳性对照、冻结执行器在基线上（A3 的 O3 一份） | 算：s4 比对器读的就是这几份 |
| `s2-comparator.out`、`s2-rows-timing.out` | 第二段比对器（Q2 全表、九行试跑）与它的耗时 | Q2 的数同 s4；九行试跑由 s4 整表取代 |
| `s2-comparator-mutations.out` | 变异表 26 条时的全表跑 | 历史 |
| `s3-baseline-after-the-fix.out`、`s3-row-0-device.out` | 修订 3 修过取号、暖机记红之后的装置基线与空变异一行（与修之前逐字节相同） | 算：修订 3 的实核 |
| `s3-k4c-attack-row-device.out`、`s3-k4c-check-driver.out` | 修订 3：取号恒拒那一行经逐行驱动跑一行（O1 红） | 算：修订 3 的实核 |
| `s3-comparator.out`、`s3-rows-timing.out` | s3 整表 117 行的比对器与耗时 | 作废留档（「s3 整表作废」一节） |
| `s3-clean-check-comparator.out` | 第五段在干净工作根上复核第 2、27、42、65 行 | 作废的佐证，留档 |
| `s3-comparator-mutations.out`、`s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` | 变异表 37、40、44、44 条时的全表跑 | 历史 |
| `s3-comparator-mutations-2026-10-03-r4.out` | 52 条那一次中途被停（61 行、没有判决行） | 不算数，留档 |
| `s3-comparator-mutations-2026-10-03-r5.out` | 变异表 52 条的全表跑，52 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r6.out` | 变异表 65 条的全表跑（X60–X72 加进来之后），65 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r7.out` | 变异表 67 条的全表跑，67 / 0 / 0 | 算：复跑登记指它 |
| `s4-comparator.out`、`s4-rows-timing.out` | s4 整表 117 行、Q3-核三趟、比对器与耗时 | 算：117 行那一半的读数（F4 产物里第 1–117 行的结果行与它逐字节相同） |
| `s4-f4-comparator.out`、`s4-f4-rows-timing.out` | F4 扩表 164 行与 s4 的 117 行并表的比对器与耗时 | 算：补位之前的读数（复跑登记里带 `--keep-compile-failures-as-they-are` 的那一份） |
| `s4-f4-substitutes-comparator.out`、`s4-f4-substitutes-rows-timing.out` | 117 + 164 行并上编不过 6 行的补位之后的比对器与耗时（补位那一趟 1 277.9 秒） | 算：补位之后的 Q1、Q3、Q4 读数（Q1、Q3 名单与补位之前逐字同），三轮判决引的产物行号取自这一份 |

全名：

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:536-557`（整段抄，未转述）**

```markdown
### 它答不了的

- Q1 的「有」是照字面判的（≥ 1 行）；12 行（丙 13 行）里哪几行算真坏，三轮结论判决给了三分（「三轮判决之后的三分」一节）：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），状态不合法所依的条款句写在「三条全放行的 7 行」「F4 新出的全放行行」两节；另几类 5 行里第 92 行那一句在字节表与里程碑验收、不在决策正文。三分只是「这一批取到的态」上的读法，各行会分开的历史多数是推的、没跑。分母 D = 37 补位之前与之后相同（补位的六行都未触达），Q1 的「有」没翻。
- 281 行里 238 行未触达（补位之后；补位之前 232 行，另 6 行编不过）：这两条流的历史只走到实现的一小部分（取号、暖机、发布、可写挂载、管理员回退、抬 F、正常卸载各一两次，没有分裂、没有删除、没有空间紧的池），未触达只说那一处改动的写流在这两条流上没变，不说它被哪条 oracle 罩着；其中 7 行改动走到了、只是没写到盘上（第 49、110、155、161、162、174、252 行，「没触达却红的两行与未触达行里交回值不同的七行」一节）。三轮之后没有一行全放行行还记「对这一批等价」。
- 触达的定义（登记读法 4）不比成功的调用交回的值，这一条没改（登记不改），分母仍 37：第 155、161 行抬 F 的上限交回 13（模型 11）、写流与结局都同，记未触达，而 O1-对拍判红；攻方在 s4 树的副本上量了 232 行未触达行的成功调用交回值，7 行不同（第 49、110、155、161、162、174、252 行），汇总行 `name=return_summary hidden_difference=7 identical=225 no_output=6`（副本探针，s4 树，入库装置没有这一列；整行与来历在「没触达却红的两行与未触达行里交回值不同的七行」一节）；mkfs 的 6 行（第 149–154 行）没量。
- 编不过的 6 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）；s4、F4 两份产物是补位之前的驱动跑的，复跑它们的登记行带 `--keep-compile-failures-as-they-are`、照旧不补位。
- 顺序类的条款（第 242、270 行）是在层 0 的切段模型上数的：「条款不许的态」靠切段规则（FUA 写只放行它自己那块盘、屏障放行它那块盘）分得出，真机上一次 FUA 写与随后的写怎么排，不在这一批里。
- O3 在 37 行上一行都没红：它的 A(c) 由实现自己写出的根构成，抓的是「恢复落到的那一版不在 [ack, begun] 里」这一类，实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判「在」。这是读执行器代码读出来的，没有另造一行只让 O3 红的坏实现去核；坏镜像那一族里 O3 红得多（Q2）。
- 三行试跑（第一段）不是表的抽样，s4 整表取代了它们当 Q1 的读数；第 1 行未触达只说那一处改动在这两条流的历史上没走到。
- 基线上 O1-崩溃判官有一部分态不适用：流二最后一次调用 `unmount` 模型没有答，写落在这次调用里的态不判；基线批 1 流二 397 态里 48 个、批 2 的 119 态里 14 个不适用（`o1_crash_judge_not_applicable`），这些态上 O1 一列是空的。流一没有不适用的态。
- 阳性对照 O1-崩溃判官每条流各翻一次（合计 2 处），登记字面是「批 1 里第一个」；O2 的对照只在流一的全持久态上叠第一条已知坏镜像，没有别的取样点。对照证明管子把红带到矩阵，不证明每条 oracle 对每种改坏都分得出。
- Q2 的 O2 一列取自两条路径（路径 5、6）；P 行的补丁字节与落地向量是 E170（O3规约执行器第一版） 导出装置写的，期望是 E170（O3规约执行器第一版） 设计员推的（登记 A2、F3）。Q2 全放行的 4 张全是 P8，那一判归 F3。41 条已知坏镜像是 O2 自己的回归语料，O2 41 条全红是构造的结果，不是 O2 独立性的证据。
- 逐行驱动与装置读跑那一刻的 `crates/`：s4 五趟驱动是同一份冻住的树，但 `crates/` 里别处在变，整表跑出来的数只说那一份树。
- 坏镜像上实现的恢复读回什么（登记第一节读法 11，附带）、旧种子盲测族的回归都没跑。Q1-丁（三条全放行而记录核对器或层 0 手写版本表的 oracle 红的行）跑了：F4 的 12 行全放行（甲、乙）里一行都没有（`name=q1_delta … =none`）；比对器只按甲乙算这一格，丙多出的第 233 行上层 0 那份 oracle 与记录核对器红（`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`）；层 0 那份 oracle 与记录核对器另在只靠 O1 的第 108、115 行上红。
- 装置内联单测没跑（checker 档），装置一侧变异 15 行的三个数没有；Python 一侧的变异只证明比对器与驱动的谓词会红，不证明它们读的装置输出是对的（装置输出由两趟基线、S2、S5 与对照另行钉）。
- E171 的 O1 一列是模型对拍加模型的崩溃判官，不接 RefFS（登记把崩溃判官算进 O1，另写本实验不用 RefFS），与 D13（验证路线） 已定项 6 表 O1 行把崩溃之后允许看到的集合归 RefFS 不同；两者的下界也不同（盘上已持久的最新根，对最后一次返回成功的发布）。「已确认而丢了」那一类在 E171 的四列里只有 O3 判，没有在 37 行上跑过 RefFS（读码读出来的）。RefFS 的数只在三轮的第二轮判决里（`research/prompts/e171-conclusion-r2-main-verification.md` 第二节 R6 一格，读数在 `research/prompts/e171-conclusion-r2-opus-model/out/judge-summary.out`，攻方副本探针，s4 树 core，RefFS 直调 crate、崩溃落点手给、没经翻译层）：暖机那一形 O3 判「不在」27 态，RefFS 判绿的也是这 27 态（27 / 27）；带内容的同形写流 O3 判「不在」的 27 态 RefFS 全红，RefFS 红的共 36 态，另 9 态 RefFS 红而 O3 判在（两边归属崩溃时刻的规矩不同，推的）。这几个数不是 E171 装置读数。
- Q2 里 O2 放行而 O3 判不在的 29 张（P 族 12、盲测 B 族 17，「Q2 里 O2 放行而 O3 判不在的坏镜像」一节）全是结构自洽的伪造根、伪造续接镜像，不是坏实现；补丁与期望同出 E170（O3规约执行器第一版） 的改法代码（E170 页「它答不了的」），源头独立在 Q2 只量过 O2、O3 一对，不当独立读数，O1 不在 Q2 里；流一 P2、P5 两种补丁的 O2 一列随底态翻面。这 29 张的复跑读今天的 `crates/`，不还原 s4 的读数：基线流与 s4 树不同，数不保证仍是 12 与 17。D13 依据段只指到本页的这个数。
- 这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树：它与今天的 `crates/` 不是同一份，页里的名单不能直接当今天实现的名单；`replay.sh` 的 E171 复跑行读今天的 `crates/`，复跑一次出的是今天那棵树的读数。入库装置上除 Q2 的 29 张之外，下面几个数都不能原样重做（装置没有挂载模式、没有 RefFS 列、panic 的流不导出）。
- 第 125 行是自崩的 6 行之一：登记读法 5 把 panic 记自崩、不进分母，D = 37。它在 panic 之前有 13 次调用可判：副本探针在 s4 树上驱动、打第 125 行变异，第 4–13 次调用的全持久态 O2 判 I-3.11 红，第 14 次（`unmount`）panic（`research/prompts/e171-conclusion-r2-opus-model/out/drive-s4-m125.tsv`），装置对 panic 整条流不导出。把 panic 与报错同样截断照判，计入分母 D = 38（改口径的预演，不是装置读数，37 与 38 都写）；若触达也把成功调用的交回值算进去，含第 155、161 行，D = 40（37 加第 125、155、161 行）；7 行交回值不同的行都算进来、含第 125 行，D = 45（推的，算术）。
- 谓词 W（正确实现的可写挂载会把这一态读成「见证过的更新一版读不出」）在第 242、269 行取到的崩溃态上为真的态各 15、63 个，基线 0 / 576；s4 树 core 上两次可写挂载的见证类拒 61 个（第 242 行 9、第 269 行 52）。这些数是攻方副本脚本（W）与副本探针（挂载）在 s4 树上量的，入库装置没有 W、也没有挂载模式。今天的 core 读 s4 写出的镜像大面积 `refused:Undecidable`（基线流一 44 / 60、流二 500 / 516）：格式变了（journal 头 311 → 487、新根段 188 → 364、点名项 67 → 64），旧记录在新解析器下读不出，不是回归；所以 W 的 15 / 63 与 61 个见证拒只对 s4 core 成立，在今天的 core 上没有对照意义。
- 用户数据落点的取值算不算 layout 登记的字节，这一页不判；三轮判决交用户（`research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行，零轮），用户 2026-10-03 定「算」：D3（空间分配） 已定项 8 射程改成新池新建文件上的落点取值（t1@50180）算格式级，检查欠在 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥。它不改这一页的三分：第 15 行的依据是 ⑤ 的 t2–t12 落点；第 34 行的落点改动在流二抬 F 之后那次发布上，不是新池新建文件，仍归分配政策。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:558-576`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-03 改了：E171 触发了定案里「若三者全部放行，说明存在共同盲区，回头补 oracle」那一支（Q1 在坏实现表上读到三者全部放行的行），也支撑「独立性要被验证，不许自称」一句；表第 4 列「独立性来自哪里」E171 只在 Q2 量过 O2、O3 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1 不在 Q2 里，所以不支撑也不推翻「三个互相独立」；该分项依据段与本行同一次改动里引回本页（`research/prompts/e171-conclusion-r3-main-verification.md` 第二节 K1 第 1 条） |
| D13（验证路线） 已定项 1 | 不影响 | 2026-10-03 不受影响：只拿 M2 重放结果的根「其余字段等于所选根」当第 233 行（丙）实现恢复结果合不合法的判据，M4「`Recover` 返回的根恰是 j\*」当第 27、112、277 行（三条不判实现恢复里靠记录重建的那一段）的判据；实验不量这一项 |
| D13（验证路线） 已定项 7 | 备料 | 2026-10-03 不受影响：记录核对器与层 0 手写版本表的 oracle 不是三条之一，比对器给它们留了附带列（Q1-丁）；F4 的 12 行全放行（甲乙）里它们一行都没红，在丙多出的第 233 行与只靠 O1 的第 108、115 行上红；已定项 7 的定案句没被动 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-03 不受影响：117 行整表两条线道 5 569.6 秒、扩表 164 行另一趟 7 607.9 秒，Q4 照登记判「只能进全量」；已定项 15 取的「debug 下单条跑到 60 秒及以上算重」那一档与「checker 档默认只在提交时跑」的定案句没被动，合谋测试进哪一档由主 agent 照 Q4 定 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-03 不受影响：只拿它定案里「新池新建文件写」那一句的容器号 1 与射程里「每个容器号都是某条记录首次插入时的 inode 号」当第 23 行状态合不合法的判据；字段表只定了写入时间的位置（偏移 64）、不定取值，决策正文欠一句，第 92 行归另一类（条款在字节表与里程碑验收）；实验不量这一项 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：只拿它字段表「正常卸载」一行与射程「卸载之后 F = 现行那一版的 txg」当第 42 行状态合不合法的判据；实验不量这一项 |
| D16（发布语义） 已定项 7 | 不影响 | 2026-10-03 不受影响：只拿定案的持久顺序（根槽 FUA → 系统配置槽 → 屏障、系统配置轮换持久之后才返回、抬 F 那一串先写系统配置）当第 242、270、265 行状态合不合法的判据，射程「同一实例里退一代，靠 journal 重放追得上」当第 277 行会分开的历史；实验不量这一项 |
| D16（发布语义） 已定项 12 | 不影响 | 2026-10-03 不受影响：只拿「根 + journal 任意前缀」当第 277 行新旧两版都合法的依据；实验不量这一项 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-03 不受影响：只拿定案「头 ID 8 字节的取值」里 `deleted_inodes` 写 inode 树的树 ID 那一句当第 271 行的判据；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 3 | 不影响 | 2026-10-03 不受影响：只拿「恢复不许先信 tail」说明第 269 行 tail 写错而恢复逐态相同；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-03 不受影响：只拿取号那一写写见证值 c_见证、mkfs 之后第一次取号写 0 那一句当第 269 行的判据，同一条见证判据当第 242 行「态不合法」的第二块依据，注 1「前缀规则不跨实例边界」当第 112 行的判据；实验不量这一项 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-03 不受影响：只拿「由记录重建那次发布的根」当第 27、277 行的判据；实验不量这一项 |
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-04 不受影响：只拿射程「其余的写（覆盖写、之后的发布、重挂之后不属于这一类的版本）上违反它的实现写出的镜像合法」当第 34 行归分配政策的判据（第 34 行的落点改动在流二抬 F 之后那次发布上，不在新池新建文件里）；射程与 `layout/01-first-txn.md` 第 67、309 行的不一致由三轮判决交用户，用户同日定新池新建文件的落点取值算格式级，那一格不碰这一页的读数；实验不量这一项 |
| D3（空间分配） 已定项 10 | 不影响 | 2026-10-04 不受影响：只拿 ⑤ 的 t2–t12 落点当第 15 行的判据；实验不量这一项 |

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-4`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants.md](../invariants.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、今天几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:25-73`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——层 0 崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「层 0 不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 checker-tier-formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-mutations.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，层 0 与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants.md](../invariants.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-mutations.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等），取在崩溃注入的二次崩溃上判）未实现。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同 |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树的根指针 | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的两根指针；树表单元自己的落点不比 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 分配记录 | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿 |

**射程**：refinement 的边际价值只在层 0 枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维层 0 已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置今天报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 不在表里，各归哪一类没定（C655（挂载透明逐字段表有字段没归类））。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对今天的 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `research/prompts/m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，J1 与 J2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-harness/src/mount_transparency.rs`，checker 档的崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-harness/src/mount_transparency.rs`。

**欠**：C37（事务与记录的对应无定义）：M2 的可判定形式；C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：M2 语义层的反向链与点名单元只能写成后置条件；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：挂载透明逐字段表没碰的字段。

```

**出处 `.claude/kb/decisions/13-验证路线.md:74-87`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是现在就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）；C37（事务与记录的对应无定义）：记录与事务对应的不变量与崩溃点重放。

```

**出处 `.claude/kb/decisions/13-验证路线.md:88-120`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：harness 档经翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `research/prompts/m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:121-136`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。层 0 的扇区取 512 字节；两条层 0 流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条层 0 流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；层 0 的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，层 0 小流里记录核对器报出来）在 `research/prompts/m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `research/prompts/e142-r19-runner3-report.md` 结论段，登记 `research/prompts/e142-r19-prereg.md` 第 85 行 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:137-153`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）；C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:154-182`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 今天的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份在 `scripts/experiments/e170_second_run_o3_spec_executor/`（E170（O3规约执行器第一版） 第 1 次重跑照今天的 kb 改过一次后冻结；第一次那一份 `scripts/experiments/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与今天的 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1），没有判官问「这个崩溃态没注入读故障，判据凭什么为真」（欠 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官））。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照今天的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）；C672（交回值或内存状态错而写流不变的一类）；C673（三 oracle 合谋测试的补 oracle 改法没做）；C679（卸载记号没进模型对拍）

```

**出处 `.claude/kb/decisions/13-验证路线.md:183-201`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（层 0 的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 checker-tier-layer0-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 层 0 不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `research/prompts/m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:202-225`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，今天都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；代际增量 scrub 零代码没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:226-259`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 层 0 冒烟 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 层 1 常规 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 层 2 全量 | 层 1 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快速档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-layer0-replay `--full`（门禁 checker-tier-layer0-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按暂存改动碰到的路径判：`crates/singlefs-checker/`、`crates/singlefs-checker-tier/src/` 下 `crash*`、`layer0_progress`、`verdict_store`、`crash_identity`、`crash_judge_*`、`crates/singlefs-replay-prefix/`。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：今天只有层 0（门禁 checker-tier-layer0-replay）；层 0 分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-layer0-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；层 1、层 2 在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，层 1、层 2 的负载与挂钟都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；层 0 规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 T1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 T1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：层 1、层 2 在门禁里没有阶段，没有 C 编号；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:260-289`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：层 1 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立层 2 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、今天几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是后者，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、今天几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：层 1 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:290-324`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 层 1 挂钟预算 | 层 1 本地挂钟预算，是输入不是输出 |

2. **`T1`（层 1 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「层 1 崩溃点重放」这个阶段，量不出 `T1`（层 1 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（层 1 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；层 0 全量枚举不需要 `T1`（层 1 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：层 1 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:325-340`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants.md](../invariants.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 今天两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段），没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:341-361`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 有代码、能对同一（历史、崩溃点）交出判定行之前，O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 今天的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀那一截，已定项 1 的 M1、M2 句法层）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:362-380`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而后者恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:381-393`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-layer0-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是 `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（装置二进制的内联单测不在快档里）（门禁 checker-tier-layer0-replay 在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-layer0-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:6-24`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；里程碑收尾与动了崩溃点重放或判器的那批提交前弹窗请用户跑全量档，红或不跑照样提交、记未结；先保失败可复现 **状态：已定。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（层 1 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量由用户要求时跑 **状态：已定。** |

```
