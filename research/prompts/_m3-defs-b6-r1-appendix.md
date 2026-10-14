**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1-19`（整段抄，未转述）**

````markdown
## E171 三oracle合谋测试 —— 第 1 次重跑第一段与第二段整表（修订 6 树）已跑（Q1 甲、Q2 乙、Q3 甲且分母 D = 39、Q4 基线甲与第 242、269 行乙；Q5 告警自证已过，判甲；最终主线的修订 8 那一趟待拍树）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 V1 V2 W1 K4 Q5 S15 S17 Z7 U1 U9 U13 U16 V17 V31 V33 V36 R14 R19 R20 R33 F8 -->

```text
本页的本地臂名与记号写小写（a1、a2、a3、a4 ……）：跑前登记、装置、产物与命令里写作对应的大写，与全局登记的编号同形；kb 正文里改成小写只为与那些编号分开，读产物时按大写对。
```

跑前登记 `e171-preregistration.md`（含第十二节修订 1–5；Q4 的另一种读法由主 agent 在装置写之前删去，只留整表 ≤ 60 秒那一种）；问题单 `three-oracle-collusion-questions.md` 的 Q1（一张坏实现表在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行）、Q2（手造坏镜像喂 O2（独立解析器 + checker）、O3（独立规约执行器），有没有一张两条全放行）、Q3（拿已知被抓的坏实现逐条关掉一条 oracle，剩下的还红不红）、Q4（这道测试一次跑多久、怎么进门禁）。被测条款 D13（验证路线） 已定项 6：三个 oracle 的独立性要靠合谋测试验证，不许自称。

**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3（独立规约执行器） 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3（独立规约执行器） 在 37 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3（独立规约执行器） 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2（独立解析器 + checker） 一列（底态 F 红、Mk 不红），O3（独立规约执行器） 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在」，不写「O3（独立规约执行器） 独有」，Q2 只比 O2（独立解析器 + checker） 与 O3（独立规约执行器） 一对，O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的 32、45、108、115、172、233、237、249、250、281；只靠 O2（独立解析器 + checker） 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:20-35`（整段抄，未转述）**

```markdown
### 装置、研究一侧与变异表

| 件 | 位置 | 说明 |
|---|---|---|
| 入库装置 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | 模式 `baseline`、`row`、`known-bad-images`，开关 `--positive-control`、`--disable-oracle`；只读、只驱动、只观测，不改 core、checker、harness、format、replay-prefix；内联单测 31 条（`grep -c '#\[test\]'`） |
| 研究一侧 | `scripts/experiments/e171_three_oracle_collusion/`（Python，目录里没有 `main.rs`，cargo 不认它当 bin） | 挑法脚本（登记 5.2，sha256 `4032a0e0…`，原样抄）、117 行的表 `bad-implementations-117.tsv`（登记 5.9 照名字抄出）、F4 扩表 `bad-implementations-expansion-164.tsv` 与造它的 `expansion_table.py`、`row_driver.py`、`comparator.py` 与它的几个模块、冻结执行器的接线 `frozen_o3.py`、判坏镜像的 `o3_bad_image_judge.py`、四个单测文件、变异驱动 |
| 变异 | 研究一侧 `scripts/mutation/tables/e171_three_oracle_collusion.tsv` 67 行；装置一侧 `crates/mutations.tsv` 末尾 15 行（名字以「E171（三oracle合谋测试） 装置」起头，`grep -c '^[E]171 装置' crates/mutations.tsv`） | 研究一侧经本目录的 `mutation_driver.py` 跑（登记修订 1 第 1 行）；装置一侧是 checker 档，三个数归提交时的门禁 checker-tier-crates-mutation-replay |
| 复跑 | `scripts/experiments/replay.sh` 里 E171（三oracle合谋测试） 十六行登记（`grep -c '^[E]171.@driver' scripts/experiments/replay.sh`）：这一页原来的十一行，加第 1 次重跑第一段·修订 6 那一趟的三行与第二段的两行 | 原来十一行见「复跑」一节；第 1 次重跑第一段那三行见「比对器、复跑与入库的跑法（修订 6 那一趟）」一节，第二段那两行见「第 1 次重跑第二段·修订 6 那一趟」一节 |
| 产物 | `research/results/` 下 `e171-three-oracle-collusion-2026-10-03-` 起头的 46 份（`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`） | 逐份在「复跑」一节的产物表里 |

- 流的驱动：两条流（新池新建文件、固定脚本到 E 再正常卸载）照抄 E170（O3规约执行器第一版） 导出装置的两条流驱动、取样三批与中性格式的写出（不 `#[path]` 引它），改三处：驱动里的 API 调用不 `.expect`，结局（成功、错误成员、panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`；跟着每次调用推理想模型，记 O1（内存模型对拍（HashMap 理想文件系统））-对拍（模型的答案与实现的结局比，`judge_and_advance`）；每个取样态算 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官（`crash_recovery_disagreement`）与 O2（独立解析器 + checker） 的 `check_pool_image`，写 `oracle-verdicts.tsv`。`baseline` 回比本地常量与 crates 的现算值（s1，42 条 `name=s1` 行都是 `holds=true`；`grep -c 'name=s1 ' …-baseline-run1.out` 与 `grep -c 'name=s1 .*holds=false'` 现数）；`row` 不回比，只取批 1 与批 2。
- S8 的三处调用在装置里的行号（`grep -n` 现取）：`newest_persisted_root` 第 1883 行、`observed_read_back_after_a_crash` 第 1885 行、`crash_recovery_disagreement` 第 1907 行，都在 `crash_judge_of_the_state`（第 1875 行）与它转调的 `crash_judge_verdict` 里；O1（内存模型对拍（HashMap 理想文件系统））-对拍的 `judge_and_advance` 在第 544 行。
- 已知坏镜像族：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908…`）的导入、常量、辅助函数与 `known_bad_images` 照抄进装置里的一个模块（第 7–1294 行，加上 `known_bad_images` 用到而散在第 1295 行之后的 12 个常量与辅助函数）；`known-bad-images` 模式核条数 41、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的（S4）。
- 逐行驱动：一次跑先拍一张仓快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，237 个文件，总摘要写在驱动的标准输出第一行），每条线道从快照同步出自己的树与编译目录，改一处、`cargo build --release`、装置 `row` 经 `run-with-memory-cap.sh`（限时 1800 秒）、还原；表里的行照名字取、核锚点，不行按补位规则换同组下一行（至多三次）。s4 与 F4 两趟时驱动只在编之前的失败上补位；第八段起编不过之后也补位（换上的行编不过再换，编之前的失败与编不过合起来至多三次），组里不含 E171（三oracle合谋测试） 装置自己的变异行，`row-result.tsv` 记 `substituted_from` 与每一个试过而没用上的 `failed_attempt_<序>`；开关 `--keep-compile-failures-as-they-are` 只给 `replay.sh` 复跑 s4、F4 两份产物用，补位那一份产物不带它。


```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:36-83`（整段抄，未转述）**

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

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:84-99`（整段抄，未转述）**

```markdown
### 三条全放行的 7 行

读数取 s4 的导出目录（`scripts/experiments/target/e171-dumps/rows-s4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比；草稿脚本 `quiet_rows_s4_fua.py`、`returned_check.py` 只读导出目录，不入库）。三分照 `.claude/rules/mutation-sampling.md` 的口径落到这一批上：**对这一批等价**＝两条流、批 1 批 2 的每个态连同实现在态上的恢复都在条款允许的集合里（写依据；会分开的历史写在旁边，没跑）；**取样点不敏感**＝两条流里有条款不许的态，取样规则没取到；**真盲区**＝取到的态里有条款不许的，三条都放行。「镜像不同」数的是 `states.tsv` 里镜像摘要与基线那一态不同的态；「择到新版」数的是其中实现看 journal 那一遍恢复择到的根 txg 不小于「第一处不同之后写出的第一条根」的态。三条各自看什么：O1（内存模型对拍（HashMap 理想文件系统））-对拍比每次调用之后实现交回的 `ObservedRoot`（`crates/singlefs-harness/src/model.rs` 里它的定义：根的 key、journal 计数器、根带的 F、文件内容、实例表、每块盘的分配记录），O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官比实现恢复择到的根与读回内容；O2（独立解析器 + checker） 判单个镜像的结构；O3（独立规约执行器） 的允许集合 A(c) 由这一行自己写出的根槽写构成（`scripts/experiments/e170_second_run_o3_spec_executor/version_table.py` 文件头说明），一版只投影根的六样（`root_ring.py` 里 `VersionProjection` 的说明：实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F），判的是它自己从镜像恢复出的那一版在不在 A(c) 里（`executor_core.py` 的 `judge_state` 里 `landed_is_allowed =` 那一行）。「三条全放行的 7 行」那张表末列的三分是三轮结论判决之后的现值（旧三分列的改动写在「历史版本」一节），各行归属的全貌在「三轮判决之后的三分」一节。

| 行（变异） | 和基线不同在哪 | 落在哪种状态里 | O1（内存模型对拍（HashMap 理想文件系统）） / O2（独立解析器 + checker） / O3（独立规约执行器） 看不看得到 | 三分 |
|---|---|---|---|---|
| 15（用户数据落点不从这张图的单元区起点起扫） | 两条流第 22 条 op 起：用户数据单元两块盘各 32 KiB 落点 822149120 → 823132160，内容逐字节相同；指向它的 16 KiB 节点里指针与校验和随之变；同一次发布的十一个提交内生块（t2–t12）也整体后移 64 槽，开放段 [50240, 50304) 整段后移到 [50304, 50368)：设备 0 上 12 条写换了落点（第 22 条 op 50180 → 50240，第 24 条 50240 → 50304，第 26 条 50242 → 50306，……，第 44 条 50252 → 50316；两条流相同，`row15-offsets.out` 里两行 `name=row15_first_publish`）；流二第 4 次调用起返回点后移 4 条 op（`mount_writable` 92–179 → 92–183），共 555 条 op 不同；调用结局同；看 journal 那一遍恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到新版：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 不带落点，读回同，看不到；O2（独立解析器 + checker）：落点在单元区里、记账自洽，判绿；O3（独立规约执行器）：投影不带落点，「在」 | **真盲区候选**：依据是 D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md`「已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序」的定案）、射程「改新池新建文件的字节：是」「十一个提交内生块（默认几何）的落点整体重排」（第 226 行）、字节表「t2 到 t12 依次 50240、50242–50243、50244、…」（`.claude/kb/layout/01-first-txn.md`「五、空间记账与分配」里「提交内生块」那一行）；变异下用户数据落到 50240、十一个提交内生块整体挪 64 槽，不在 ⑤ 钉的落点上，三条都放行。另一条依据（用户 2026-10-03 定案之后才有）：D3（空间分配） 已定项 8 射程「例外是**从 mkfs 水位第一次建树的那一版**」「新池新建文件是这一类」、t1@50180「登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里，算格式级，偏离它写出的这一类镜像不合法」（`.claude/kb/decisions/03-空间分配.md`「已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽」的射程），变异把 t1 从 50180 挪到 50240，本身就不合法；三轮判决定去留时还没有这一条，只靠 ⑤。会分开的历史（推的）：单元区刚好够用的池，跳过开头一段之后发布在模型说该成的地方报空间不够，O1（内存模型对拍（HashMap 理想文件系统））-对拍会红 |
| 23（分裂出来的右半容器号不是触发分裂那条新记录的 inode 号） | 两条流第 26 条 op 起：第一个文件那次发布写的 inode 叶（码 3，两块盘各 32 KiB）偏移 53 一字节 0x01 → 0x00（容器号 1 → 0），头校验和随之变；同一次发布第 28 条 op（16 KiB 节点）偏移 52、60、131、149 等处 0x01 → 0x00；之后重写 inode 树的每次发布都带着（流二 155 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到第一个文件那一版（txg 3）及之后：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：不读盘，`ObservedRoot` 里没有容器号，读回同；O2（独立解析器 + checker）：I-9.4（容器号不超最小 key）只要「inode 号 ≥ 容器号、沿叶序严格递增」，一片叶时 1 ≥ 0 成立，判绿；O3（独立规约执行器）：投影不进 inode 树，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 6 定案写「**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md`「已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用」的定案），射程写「每个容器号都是某条记录首次插入时的 inode 号」（第 205 行）；容器号 0 不是任何一条记录的 inode 号。择到新版的那些态不在条款写得出的镜像里，三条都放行。会让 O2（独立解析器 + checker） 红的取样点（推的，没跑）：一片叶装满之后的分裂，右半容器号 0 小于左半，I-9.4（容器号不超最小 key） 的「沿叶序严格递增」判红 |
| 27（记录头读者把末条标志位 0 读反） | 写流与调用结局全同；只在看 journal 那一遍恢复上不同：流一批 1 6 态、流二批 1 51 态（批 2 0 态）里，基线施加 1 条记录、择到新的一版，变异把带末条标志的记录读成「后面还有」、施加 0 条、停在前一版；读回内容不同的 3 / 30 态（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 发布写到一半的态：被丢掉的那一版的调用没返回、它的根槽没落（`returned_check.py` 57 态全是 `call_returned=False its_root_slot_landed=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：前一版是模型提交过的、不比盘上已持久的最新根槽旧、读回与那一版相同，判绿是对的；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用不经恢复；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同，各自不看实现的恢复 | **另一类：三条不判实现恢复里靠记录重建的那一段**。态本身合法（调用没返回，新旧两版都在允许集合里），错在实现的恢复少施加一条记录，违反 D13（验证路线） 已定项 1 的 M4「`Recover` 返回的根恰是 j\*」与 D23（journal 的角色与格式） 已定项 15「由记录重建那次发布的根」、已定项 14「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」；O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官的判据（模型提交过的某一版、不比盘上已持久的最新根旧）比 M4 弱，O2（独立解析器 + checker） 判恢复之前的镜像，O3（独立规约执行器） 不跑实现的恢复；M4 的落点是 E165（恢复重放有界穷举与变异杀伤界） 与 Verus，不是这三条。会分开的取样点（推的，没跑）：一次已经返回、提交点只在 journal 记录里（根槽还没写）的发布之后的态，变异会丢掉一次已返回的发布；两条流的发布都在返回之前写根槽，这样的态不存在。那时 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官的「不许比盘上已持久的最新根槽旧」也挡不住（根槽没写），会不会三条都放行没量过 |
| 34（抬 F 生效之后不放开扣住的槽） | 流一无不同；流二第 498 条 op 起（抬 F 之后那次 `publish`）：用户数据单元落点 822083584 → 822411264，内容相同；指向它的节点与分配记录随之变（51 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 68 态、批 2 21 态；择到 txg 17 及之后的根：批 1 60 态、批 2 14 态 | 同第 15 行：O1（内存模型对拍（HashMap 理想文件系统）） 不比落点，O2（独立解析器 + checker） 判绿（记账自洽），O3（独立规约执行器）「在」 | **另一类：分配政策，镜像合法**：抬 F 之后那次发布换了一个空闲槽、没复用回收的槽，每个态的分配记录与记账自洽；D3（空间分配） 已定项 8 射程「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法，拦它只能靠运行时读数」（`.claude/kb/decisions/03-空间分配.md`「已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽」的射程；新池新建文件那一格例外，这一行的改动在流二抬 F 之后那次发布上，不在其中），三条按设计都不判分配政策。会分开的历史（推的）：抬 F 之后空间只够复用扣住那几槽的池，发布报空间不够而模型说该成，O1（内存模型对拍（HashMap 理想文件系统））-对拍会红；这一行在 `crates/mutations.tsv` 点名的测试 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 直接断言复用 |
| 42（卸载把 F 抬到现行那一版带的 F 而不是它的 txg） | 流一无不同；流二第 537 条 op 起（卸载那次调用的第一条写：两块盘的系统配置槽，偏移 481 一字节 0x11 → 0x0b，即 F 17 → 11，连同校验和），之后卸载推的空发布写的根与节点里同样是 11（24 条 op 不同）；卸载照常 `ok`；恢复逐态同 | 镜像不同：流二批 1 48 态、批 2 14 态，全持久态在内；择到卸载写出的根（txg ≥ 18）：批 1 32 态、批 2 7 态 | O1（内存模型对拍（HashMap 理想文件系统））：卸载没有模型的答（登记读法 6），这次调用不对拍、写落在卸载里的态崩溃判官记不适用，看不到（若有卸载的答，`ObservedRoot` 带根的 F，会比出 11 对 17，推的）；O2（独立解析器 + checker）：系统配置的 F 与根带的 F 都是 11，I-7.12（系统配置 F 不低于同盘根上的 F）成立，单个镜像不知道该是 17；O3（独立规约执行器）：投影带 F，但 A(c) 就是这一行自己写的 F = 11 的根，「在」 | **真盲区候选**：D16（发布语义） 已定项 1 定案的字段表「正常卸载」一行写「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md`），射程「正常卸载的代价」那一条写「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」。变异下卸载返回之后的全持久态 F = 11，回退候选集里留着 txg 11–16 的根，不在条款允许的镜像里，三条都放行。O1（内存模型对拍（HashMap 理想文件系统）） 那一半是登记读法 6 写明的已知缺口（模型没有卸载的答） |
| 92（inode 记录的写入时间留成 A 的） | 流一无不同；流二第 61 条 op 起（第二次 `publish`，覆盖写那次的 inode 叶，两块盘各 32 KiB）：叶内偏移 200、208、216 三个字节 0x3c → 0x00（第 0 条记录起点 136，+64 / +72 / +80 是 atime、mtime、ctime 秒：基线 60，变异留 A 的 0），头里偏移 10–13（头校验和）与 89–92 另有不同；之后重写这片叶的发布都带着（139 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 344 态、批 2 112 态；择到覆盖写那一版（txg 4）及之后：批 1 336 态、批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedFile` 只有内容，`model.rs` 与 `model_comparison.rs` 里 `write_time` 零命中，D13（验证路线） 已定项 6 表 O1（内存模型对拍（HashMap 理想文件系统）） 行判「内容、大小、目录结构」，看不到；O2（独立解析器 + checker）：单个镜像不知道这次发布的写入时间参数，判绿；O3（独立规约执行器）：投影不进 inode 树，「在」 | **另一类：条款在字节表与里程碑验收，决策正文欠一句**：D8（核心索引结构） 已定项 6 的字段表只定了偏移 64「atime / mtime / ctime 秒」的位置；「写入时间是发布参数」写在 `.claude/kb/milestone/01-first-txn.md`「步 3　inode 树与其余几棵树」的现状一段与 `02-second-txn.md`「步 1　覆盖写：同一个文件、同一个池、第二次发布（发布 B）」里 inode 记录那一条，字节表写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md`「四、文件元数据（inode）」里「inode 记录（偏移 64）」那一行），验收写「inode 记录的写入时间留成 A 的 ⇒ 判红」（`02-second-txn.md` 同一步里「变异：extent 叶记录的指针忘了换」那一条），决策正文里没找到取值的一句。择到新版的态（写入时间 0、这次发布的参数是 60）不合验收，三条按设计看不见（没有 API 交回写入时间），不当独立性失败 |
| 112（前缀跨实例边界） | 写流与调用结局全同；看 journal 那一遍恢复在两条流各 3 态（批 1 第 9–11 态，段 1：暖机的第一条写——新实例 1 的 journal 记录——落了、它的根槽没落）不同：基线择 `0,0`、施加 0 条，变异把实例 1 的记录接到实例 0 的根上、施加 1 条、得到 `1,1`；读回都没有文件 | 暖机写到一半的态（`returned_check.py` 6 态全是 `call_returned=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：`1,1` 是模型提交过的一版、不比已持久的最新根槽旧、读回同，判绿是对的；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段（多施加）**：暖机没返回，`0,0` 与 `1,1` 都在允许集合里，错在实现的恢复把别的实例的记录也接上，违反 D23（journal 的角色与格式） 已定项 14 注 1「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md`「已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换」），依据只有这一条注；这一行的变异已在门禁 checker-tier-formal-verification 的红例钉表里（`.claude/gate.d/e165-pinned-red-examples.tsv` 里第 1 列是 `371a71190dce62e0…` 的红例），由 E165（恢复重放有界穷举与变异杀伤界） 的有界穷举判。会分开的取样点（推的，没跑）：环里留着别的实例、不在所选根那条时间线上的记录（崩溃恢复抛弃了根之后的新实例），接进来之后恢复落到模型没提交过的一版，O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官会红；两条流里没有这种态 |

第 102 行（抬 F 先写系统配置那一步不带新 F）是批 2 多出来的那一行全放行（`name=q1_alpha … rows_that_differ_from_batch_1=102`）：流二第 440 条 op（抬 F 先写的系统配置槽，偏移 481 一字节 0x0b → 0x00）与第 537 条 op（卸载先写的系统配置槽，0x11 → 0x0b）不同，4 条 op；O2（独立解析器 + checker） 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 60、73 段各 6 态（`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`）。批 2 在流二只取 17 段（7、11、16、20、24、28、32、36、40、44、48、52、57、61、65、70、74），第 60、73 段一个态都没取（`awk -F'\t' '$3==60\|\|$3==73' states.tsv` 只出批 1 的 8 + 8 态）：**取样点不敏感**，会分开的取样点就是第 60、73 段的任一态。这一格两批翻面，记「不稳定」；Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 8 行仍 ≥ 1；只靠一条的 10 行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:100-119`（整段抄，未转述）**

```markdown
### Q3 只靠一条的 11 行

只写够不到的对象与没判的字段，不判结论。读数取 s4 的导出目录（草稿脚本 `caught_rows_s4.py`，不入库）与产物的 `name=trajectory` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 32（可写挂载不写实例表的行，实例表照抄） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 判 `disagrees:这一版的实例表`（模型要有上一个实例那一行，实现交回空表），模型停在这次调用，之后流二 324 态（批 1）崩溃判官记不适用 | O2（独立解析器 + checker）：这一行 0 态红（批 1 流二 397 态都绿），实现了的不变量里没有一条拿根的实例代号去对实例表的行（由读数推的，没逐条查 invariants.md）；O3（独立规约执行器）：投影里没有实例表，A(c) 是这一行自己写的根，「在」 |
| 45（挂载那一刻就读不出的槽也重读、仍坏就拒） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 12 次调用 `raise_floor` 交回 `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，模型说该成 | O2（独立解析器 + checker）、O3（独立规约执行器）：被拒之前的写流与基线逐条相同（流二停在第 440 条 op，0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，单镜像的判官没有对象可判 |
| 108（码 2 读者补齐字节从最后一个载荷字节算起） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（胶水解不开实现交回的单元：「补齐字节非零」）；流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（流一 0 条 op 不同，流二停在第 92 条 op 之前也 0 条不同）；O2（独立解析器 + checker） 用自己的解析器读裸字节、不调 core 的读者，O3（独立规约执行器） 用 Python 自己的读者，两边看到的镜像与基线逐字节相同。崩溃点重放那份手写版本表的 oracle 与记录核对器红（`layer0_or_record_checker_red=true`，不是三条之一） |
| 115（取前缀核锚点筛选不看 txg） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 32 态、流二 52 态「走读失败：没择到根」（实现恢复报 `RootPublishCarriesMoreThanOneLastRecordFlag…`），段为正 8/12、12/16；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery`，模型说该成 | 写出的字节与基线相同（流一 0 条 op 不同）；O2（独立解析器 + checker） 判镜像结构、O3（独立规约执行器） 用它自己的前缀规则从镜像恢复，都不看实现的恢复，看到的镜像与基线逐字节相同。崩溃点重放那份 oracle 与记录核对器红 |
| 8（空闲不随分配减） | O2（独立解析器 + checker）：I-5.2（空闲统计对得上）批 1 流一 9 态、流二 353 态，段为正 2/12、69/79；批 2 流二 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 里没有记账行，调用都成、内容都对；O3（独立规约执行器）：投影不进记账树，「在」 |
| 65（blocks 向下取整） | O2（独立解析器 + checker）：I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）批 1 流一 9 态、流二 353 态 | O1（内存模型对拍（HashMap 理想文件系统））：`model.rs` 与 `model_comparison.rs` 里 `blocks` 零命中，读回内容同；O3（独立规约执行器）：投影不进 inode 树 |
| 86（写系统配置槽时 F 那 8 字节写成 0） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 97 态，段为正 19/79；批 2 28 态 | O1（内存模型对拍（HashMap 理想文件系统））：比的是根带的 F（`ObservedRoot` 的 `rollback_floor`），不比系统配置槽里的 F；O3（独立规约执行器）：投影的 F 取自根，系统配置槽只拿来择几何，A(c) 是这一行自己写的根，「在」 |
| 87（core 那份在飞上限把 F 多算 1） | O2（独立解析器 + checker）：I-8.1（环几何够大）批 1 流一 51 态、流二 385 态，I-7.13（系统配置池级字段在读者收的范围里）流一 2 态、流二 12 态；段为正 12/12、79/79 | O1（内存模型对拍（HashMap 理想文件系统））：不比系统配置槽的字段；O3（独立规约执行器）：同第 86 行，投影里没有在飞上限 |
| 90（新池新建文件的改动计数写回 1） | O2（独立解析器 + checker）：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流一 9 态、流二 240 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比 inode 记录的改动计数；O3（独立规约执行器）：投影不进 inode 树 |
| 99（defer 行写 0） | O2（独立解析器 + checker）：I-3.11（已分配减 defer 等于最新根走读）批 1 流一 9 态、流二 353 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比记账行；O3（独立规约执行器）：投影不进记账树 |
| 102（抬 F 先写系统配置那一步不带新 F） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 12 态，第 60、73 段各 6 态；批 2 不红（取样点不敏感，见「三条全放行的 7 行」一节末） | O1（内存模型对拍（HashMap 理想文件系统））：`raise_floor` 的答比的是写出的根（根带的 F 对），不比系统配置槽；O3（独立规约执行器）：同第 86 行 |

O3（独立规约执行器） 在 s4 计入分母的 19 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`），而它在每行两条流的 450 个批 1 态上都给出了严格判定（`o3_judged_states_strict=450`，`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。由执行器的写法读出来的一条（读码，没另跑）：A(c) 只由这一行自己写出的根槽写构成，投影只有根的六样，所以 O3（独立规约执行器） 只在「实现自己的恢复规则与条款不同、或一次发布在崩溃之后半施加」这一类上红；实现把某个字段一致地写错，O3（独立规约执行器） 照样判「在」。O3（独立规约执行器） 在坏镜像那一族（Q2）上红得多：P 行 26 张红 22、盲测 40 张全红、已知坏镜像 41 张严格红 28（`name=q2_family` 四行）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:120-123`（整段抄，未转述）**

```markdown
### s3 整表作废

s3 那一次整表（输出根 `scripts/experiments/target/e171-dumps/rows-s3/`）两条线道各自跨行复用编译目录；还原改过的文件用 `copy2`、同步用 `rsync -a`，都把修改时间带回快照那一刻，cargo 按修改时间判新旧，线道 1 跑过第 112 行（replay-prefix 的变异）之后、跑第 114 行之前的 45 行，编出来的装置都带着第 112 行的变异。登记修订 5 照实记了这 45 行、cargo 1.98.0 上的机理实测与驱动的改法；s3 整表、s3 的 Q3-核各趟与 s3 比对器产物不再当 Q1、Q3、Q4 的读数。留档不删：`research/results/` 下的 `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out`（第五段对第 2、27、42、65 行的干净复核），与 `scripts/experiments/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`。s3 读到的「Q1 全放行 40 行、分母 52」里 38 行是被带着跑的；s4 在修过的驱动上重跑全部 117 行，判决行 `rows_compiled_without_the_rebuild_check=0`。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:124-132`（整段抄，未转述）**

```markdown
### F4 扩表

登记第十节 F4：「触达行太少，Q1 的『没有』不够判：进第三段扩表（5.2），扩完仍不够记『不够判』，不记『没有』」，触发观测「Q1-分母 D < 30」；s4 的 D = 19。扩表照登记 5.2「扩表」：步长 4 的挑法比步长 8 多出来的 164 行（与步长 8 重叠的 56 行不重跑），按挑法脚本的输出顺序接在 117 后面编号 118–281、刀号 3。

- 行文件 `scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`（sha256 `f1bd082e…`），由同一目录的 `expansion_table.py` 起挑法脚本两次（步长 8、步长 4）造出；表取 `crates/mutations.tsv` 去掉 E171（三oracle合谋测试） 装置自己的变异行（与 a6、a7 同一个去法）。在 s4 冻住的树的那一份与 2026-10-03 仓里的那一份上造出来逐字节相同；按文件：admission 5、allocation_record_tree 2、allocator 12、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 6、mount 44、mounted_read 3、mounted_session 2、records 2、recovery 28、root_record 1、system_configuration 3、transaction 34、unit 2、replay-prefix 8。登记 5.9 只钉了 117 行的行文 sha256，扩表各行的行文摘要取造表那一刻的表。
- 逐行驱动加 `--other-selected-rows`：补位时跳过 117 行与扩表两张表里的每一个名字（登记 5.2「扩表与补位都不重挑已跑过的行」）。
- 比对器 `--rows-root` 可以给两份（117 行那份与扩表那份，同一个行号在两份里都有就拒），`--expansion-rows` 给扩表时多两行结果：`name=f4_expansion`（扩表与挑法脚本在 `--mutations-table` 那张表上现算的逐行相同、164 行、序号连续、刀号 3、与 117 行不重名）与 `name=q1_denominator_by_table`（分母按表分开再报）；Q1 的分母与三种读法、Q3 都按并起来的 281 行算。拿 s4 的输入重跑改过的比对器（只给 rows-s4），E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）。
- 扩表那一趟在 s4 那一份冻住的树上跑（快照摘要 `d1586465…`，与 s4 五趟驱动的相同），两半才能并成同一批；树不重新冻。主 agent 照第七段交回的脚本跑完七步（空变异一行对 `baseline-run1` 未触达、扩表与挑法脚本在冻住的树的表上现算的逐字节相同、这一次两趟驱动与 s4 的快照摘要只有一种），扩表那一趟两条线道 7 607.9 秒。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:133-165`（整段抄，未转述）**

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

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:166-180`（整段抄，未转述）**

```markdown
### F4 新出的全放行行

读数取 F4 的导出目录（`scripts/experiments/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比）；草稿脚本 `quiet_rows_f4.py`（第七段 `quiet_rows_s4_fua.py` 改输出根）、`extra_checks_f4.py`、`order_by_segment_f4.py`（都只读导出目录，不入库）。三分的口径与「三条全放行的 7 行」一节相同。顺序类的两行（第 242、270 行）写流的条数或次序变了，逐态与基线比没有意义，改用一条判法在这一行自己的态上数「条款不许的态」，同一判法在基线上数一遍都是 0（判法分得出）：切段用比对器自己的 `baseline_checks.segments_of_the_write_stream`（FUA 写放行它那块盘、屏障放行它那块盘）。

| 行（变异） | 和基线不同在哪 | 条款不许的态（批 1 / 批 2） | O1（内存模型对拍（HashMap 理想文件系统）） / O2（独立解析器 + checker） / O3（独立规约执行器） 看不看得到 | 三分 |
|---|---|---|---|---|
| 242（发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前） | 两条流第 8 条 op 起：每次取号、暖机、发布末尾的「根槽 FUA → 两块盘的系统配置槽 → 屏障」换成「两块盘的系统配置槽 → 根槽 FUA → 屏障」，写的内容逐条相同、次序变了（流一 9 条、流二 57 条 op 按下标比不同）；基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段（流一 8 段里 3 段、流二 59 段里 19 段，基线 0 段）；调用结局同 | 同一段里系统配置写已落、根槽 FUA 没落全：流一 21 / 3 态，流二 133 / 19 态；基线 0 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：根槽没落，恢复择到前一版，前一版是模型提交过的、不比已持久的最新根槽旧，判绿；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用结局同；O2（独立解析器 + checker）：这一行 0 态红（单个镜像里系统配置槽比根早一次发布，没逐条查 invariants.md 有没有管这一样的）；O3（独立规约执行器）：A(c) 由这一行自己写的根构成，恢复落到的版本在里面，「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」「系统配置槽在根槽持久之后再更新」（`.claude/kb/decisions/16-发布语义.md`「已定项 7：发布的持久顺序」）；系统配置先于它见证的根落盘的态，照条款的写序造不出来，三条都放行 |
| 269（见证值取成槽世代号） | 流一第 0、1 条 op（mkfs 之后第一次取号写的两块盘系统配置槽）偏移 469 一字节 0x00 → 0x01，连同校验和（155–158）；流二另有第 92、93 条 op（可写挂载取号那一写）0x04 → 0x06。偏移 469 是 journal tail（`crates/singlefs-checker/src/lib.rs` 的 `SYSTEM_CONFIGURATION_TAIL_OFFSET`）；调用结局同；看 journal 那一遍恢复逐态同 | 镜像与基线不同的态：流一 31 / 0，流二 78 / 14（全持久态相同：之后的发布把那一槽写成正确的 tail） | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 不带系统配置，恢复不先信 tail（D23（journal 的角色与格式） 已定项 3），读回同；O2（独立解析器 + checker）：读了 tail（`crates/singlefs-checker/src/lib.rs` 的 `check_system_configuration_slot`），没有一条不变量判它的值；O3（独立规约执行器）：执行器只在锚点断言里读 tail（`scripts/experiments/e170_second_run_o3_spec_executor/executor_main.py` 的 `anchor_assertions` 里），只核全持久态，判定不用它 | **真盲区候选**：D23（journal 的角色与格式） 已定项 14 写「取号那一写……改写取号那一刻按同一取法读到的见证值 c_见证」（全部自证过的槽 journal_tail 取最大）、「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md`「已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换」）；tail 写成 1 的系统配置槽不在条款写得出的镜像里，三条都放行 |
| 270（发布在系统配置轮换之后不过屏障就返回） | 每次发布末尾那道屏障没了：流一 57 → 53 条 op、流二 595 → 565 条，之后的返回点整体前移；写的内容同；系统配置轮换与下一次调用的第一批写切进同一段 | 一次调用的系统配置写没落全、之后的调用的写已落：流一 7 / 6 态，流二 70 / 59 态；基线 0 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：那次发布的根槽 FUA 已落，恢复择到它，判绿；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用结局同；O2（独立解析器 + checker）：系统配置落后根一次发布在单个镜像里合法（I-7.12（系统配置 F 不低于同盘根上的 F） 只比 F）；O3（独立规约执行器）：「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」（第 181 行），射程「屏障口径」写明系统配置轮换之后那一道屏障是「发布返回前加屏障」（第 186 行）；返回之后系统配置还没落、后面的写先落的态照条款造不出来，三条都放行 |
| 271（deleted_inodes 的树表条目头 ID 写 0） | 两条流第 44、45 条 op（树表单元，两块盘各 16 KiB）偏移 1647、1649 两字节 0x01 → 0x00（头 ID 65537 → 0），连同单元校验和；指向它的节点（第 48、49 条 op）与根槽（第 52 条 op）的指针与校验和随之变；流二之后重写树表的发布都带着（85 条 op 不同）；调用结局同；恢复逐态同 | 择到新版（根 txg 不小于第一处不同之后写出的第一条根）：流一 12 / 0 态，流二 356 / 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：不读树表；O2（独立解析器 + checker）：树表条目只解树 ID、种类、诞生 txg（`crates/singlefs-checker/src/walk.rs` 的 `tree_table_entry_of_the_walk`），头 ID 不判；O3（独立规约执行器）：投影只带树表单元指针，A(c) 是这一行自己写的根，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 8 定案「头 ID 8 字节的取值：……`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md`「已定项 8：树表条目的字段表」）；择到新版的态里头 ID 是 0，三条都放行 |
| 277（取前缀施加一次发布之后新根段不换） | 写流与调用结局全同；看 journal 那一遍恢复在流一批 1 6 态、流二批 1 51 态（批 2 0 态）不同：基线施加 1 条记录、落到新的一版，变异同样施加 1 条、却停在所选根那一版（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 0：57 态里被丢掉的那一版的调用都没返回、它的根槽都没落（`extra_checks_f4.py`：全是 `call_returned=False its_root_slot_landed=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：前一版是模型提交过的、不比已持久的最新根槽旧、读回与那一版相同，判绿是对的；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段**：调用没返回，新旧两版都在「根 + journal 任意前缀」（D16（发布语义） 已定项 12）里，态本身合法；错在实现的恢复施加记录之后新根段不换，违反 D13（验证路线） 已定项 1 的 M4 与 D23（journal 的角色与格式） 已定项 15（同第 27 行），三条不判；M4 的落点是 E165（恢复重放有界穷举与变异杀伤界） 与 Verus。会分开的历史（推的，没跑）：一次已返回的发布之后根槽读不出、恢复要靠 journal 重放追上那一版（D16（发布语义） 已定项 7 射程「同一实例里退一代，靠 journal 重放追得上」），变异会停在前一版；两条流里没有读坏根槽的态 |

- **第 233 行（由记录重建的根不照抄所选根的文件系统标识，改写全零），只在丙里全放行**：写流与调用结局全同；看 journal 那一遍恢复在施加了记录的态上走读失败（`InvariantViolated`，`invariant` 字段是 I-1.4（块头 fsid 一致），`detail` 是「树根 fsid 不符」），流一批 1 3 态、流二批 1 48 态（其中 6 态写落在卸载里、崩溃判官记不适用）。甲、乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官判红（流一 3 态、流二 42 态，段为正 2/12、28/79），只靠 O1（内存模型对拍（HashMap 理想文件系统））；丙只取 O1（内存模型对拍（HashMap 理想文件系统））-对拍，调用不经恢复，三条全放行。在丙里是**真盲区候选**：实现在合法的崩溃镜像上恢复不出任何一版，D13（验证路线） 已定项 1 的 M2 写重放结果的根「其余字段等于所选根」（`.claude/kb/decisions/13-验证路线.md`「已定项 1」表里 M2 那一行）；O2（独立解析器 + checker） 判恢复之前的镜像（与基线逐字节相同），O3（独立规约执行器） 跑的是它自己的恢复。崩溃点重放那份 oracle 与记录核对器在这一行上红（`layer0_or_record_checker_red=true`，不是三条之一）。批 2 不红：两条流批 2 一个「基线施加了记录」的态都没有（`extra_checks_f4.py` 输出 `b2_base_records_applied_gt0: 0`），**取样点不敏感**，会分开的取样点是任一个施加了记录的态；这一格两批翻面，记「不稳定」。
- **第 265 行（抬 F 那一串不先写系统配置），批 2 多出来的全放行**：只有流二第 440 条 op 起不同（抬 F 先写系统配置那一步整步没了，149 条 op 按下标比不同）；O2（独立解析器 + checker） 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 59、71 段各 6 态（`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`）。批 2 在流二取的段是 7、11、16、20、24、28、32、36、40、44、48、52、56、60、64、68、72，没有第 59、71 段（`extra_checks_f4.py` 输出 `batch2_states_in_segments_59_71=0`）：**取样点不敏感**，同第 102 行；记「不稳定」。条款是 D16（发布语义） 已定项 7 定案「抬 F 那一串……先把新 F 写进每块盘的系统配置、过一道屏障」。Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 15 行，只靠一条的行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:181-197`（整段抄，未转述）**

```markdown
### 三轮判决之后的三分

三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`；第三轮是最终轮，以它第三节「三轮之后的结论」与第五节为准）把这批读数重新三分。口径：**真盲区候选**＝这一批取到的态在已定条款下不合法、三条都放行（E171（三oracle合谋测试） s4 树）；**另几类**＝三条按设计不判、或错不在态上。执行员原判「对这一批等价」的五行（第 15、27、34、112、277 行）一行都不再是等价。

| 类 | 行 | 一句话依据（各行条款原文在「F4 扩表的结果（整行抄自产物）」与「F4 新出的全放行行」两节的行里） | 三轮里的翻动 |
|---|---|---|---|
| 真盲区候选 | 第 15 行 | D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：变异下十一个提交内生块整体挪 64 槽（`row15-offsets.out`），⑤ 与 `layout/01-first-txn.md` 里「提交内生块」那一行钉的是这些落点 | 第一轮从「对这一批等价」挪进来（D3（空间分配） 已定项 10 ③）；第二轮维持，依据写 ⑤ 落点表与 `layout/01-first-txn.md` 里「用户数据落点政策」那一行的 t1@50180；第三轮留名单、依据换成 ⑤ 的 t2–t12 落点（主 agent 现查 ⑤ 表 t2@50240 与第 220 行「改新池新建文件的字节：是」，复跑 `row15-offsets.out` 同） |
| 真盲区候选 | 第 23、42、271 行 | 第 23 行 D8（核心索引结构） 已定项 6 的容器号；第 42 行 D16（发布语义） 已定项 1 卸载抬 F；第 271 行 D8（核心索引结构） 已定项 8 头 ID | 三轮都在名单 |
| 真盲区候选 | 第 242、269、270 行 | 第 242、270 行 D16（发布语义） 已定项 7 的写序；第 269 行 D23（journal 的角色与格式） 已定项 14 的见证值 | 第一轮被改出名单（按已定项 12 合法、写序义务无人判），第二轮翻回，第三轮没被打中 |
| 另一类 | 第 27、112、277 行 | 三条不判实现恢复里靠记录重建的那一段（D13（验证路线） 已定项 1 的 M4、D23（journal 的角色与格式） 已定项 14 注 1 与已定项 15）；第 112 行已在 `.claude/gate.d/e165-pinned-red-examples.tsv` 里 | 第一轮采正推改归这一类，第二轮站住、措辞收窄，第三轮没变 |
| 另一类 | 第 34 行 | 分配政策，镜像合法：D3（空间分配） 已定项 8 射程 | 第一轮与第 15 行一起挪进真盲区候选，第二轮翻回，第三轮维持 |
| 另一类 | 第 92 行 | D8（核心索引结构） 已定项 6 已补写入时间取值（2026-10-04）；三条 oracle 按设计判不到，由钉写的用例判 | 第一轮归「条款没定且不可见」不进名单，第二轮改理由，第三轮维持 |
| 另一类（零轮） | 第 110、162、174 行 | 运行时读数（写账、分支名），三条按设计都不判，与第 34 行同类 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |
| 另一类（零轮） | 第 49、252 行 | 交回值或内存状态错、写流不变，要更长的历史才分得开 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |

第 155、161 行是已知的那两行（O1（内存模型对拍（HashMap 理想文件系统））-对拍判红），不在这几类里；第 233 行（丙里全放行、甲乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官抓到）、第 265 行（批 2 取样点不敏感）的读法在「F4 新出的全放行行」一节，第 102 行（批 2 取样点不敏感）在「三条全放行的 7 行」一节末，三轮没有改。「另一类（零轮）」两行的意思是这一类是第三轮才冒出来的，没有再开轮，进欠账，不改登记的触达定义。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:198-218`（整段抄，未转述）**

```markdown
### F4 新出的只靠一条的 12 行

只写够不到的对象与没判的字段，不判结论。读数取 F4 的导出目录（草稿脚本 `caught_rows_f4.py`，第七段 `caught_rows_s4.py` 改输出根，不入库）与产物的 `name=trajectory`、`name=row_matrix` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 172（交进来的盘数等于 w 的下限也拒） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:WritableDeviceCountBelowTheStripeWidthLowerBound`，模型说该成；流二停在第 92 条 op | 被拒之前的写流与基线逐条相同（0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，O2（独立解析器 + checker）、O3（独立规约执行器） 没有对象可判（同第 45 行） |
| 233（由记录重建的根不照抄所选根的 fsid） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 3 态、流二 42 态走读失败（I-1.4（块头 fsid 一致） 树根 fsid 不符），段为正 2/12、28/79；批 2 0 态 | 错在实现的恢复，写流与基线逐字节相同；O2（独立解析器 + checker） 判恢复之前的镜像，O3（独立规约执行器） 跑它自己的恢复。批 1 与批 2 红的列不同：批 2 没有施加了记录的态，取样点不敏感（「F4 新出的全放行行」一节） |
| 237（解系统配置槽时根环起点读错了位置） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 53 态、流二 73 态（段为正 12/12、16/16），批 2 7、14 态，「没择到根」（`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`）；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（0 条 op 不同）；O2（独立解析器 + checker）、O3（独立规约执行器） 各用自己的解析器读系统配置，看到的镜像与基线逐字节相同 |
| 249（一个数据单元的发布也切成两条记录） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:jsn`（模型说那一版的 jsn 3，实现 4），模型停下 | 两条记录合起来仍是一次完整的发布：O2（独立解析器 + checker） 在这一行 0 态红（没逐条查 invariants.md 有没有限一次发布几条记录的）；O3（独立规约执行器） 自己重放记录，落到的版本在 A(c) 里，投影不带 jsn |
| 250（数据单元的净荷字节倒序写） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（读回 3000 字节、内容倒序） | 单元的校验和按写下去的字节算，结构自洽：D13（验证路线） 已定项 6 表 O2（独立解析器 + checker） 行「验不到」一栏写的「一个块被覆盖成另一份合法内容」；O3（独立规约执行器） 的投影不带文件内容，不拿写入的内容去比 |
| 281（择根取第一个自证过的槽就不再换） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 41 态、流二 61 态（段为正 9/12、13/16），批 2 7、14 态，「走到实例 0 第 0 代根」（比盘上已持久的最新根槽旧）；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:NewerStateStillUnreadableAfterOneReread` | 错在实现的择根，写流与基线逐字节相同（0 条 op 不同）；O2（独立解析器 + checker） 判镜像结构，O3（独立规约执行器） 用自己的择根规则恢复，看到的镜像与基线相同 |
| 135（复活 defer 里那一条时 defer 槽数不减） | O2（独立解析器 + checker）：I-3.11（已分配减 defer 等于最新根走读）批 1 流二 225 态，段为正 44/79；批 2 70 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot`（`crates/singlefs-harness/src/model.rs` 里它的定义）没有记账行；O3（独立规约执行器）：投影不进记账树 |
| 234（根记录写者不照卸载记号写 flags） | O2（独立解析器 + checker）：I-7.9（回退下界 F 不高于抬 F 的上限）（按卸载记号分两支）批 1 流二 29 态，第 73 段起段为正 6/79；批 2 7 态 | O1（内存模型对拍（HashMap 理想文件系统））：写落在卸载那次调用里，模型没有卸载的答（登记读法 6），对拍不比、崩溃判官记不适用，`ObservedRoot` 也不带根的 flags；O3（独立规约执行器）：投影不带卸载记号（`root_ring.py` 里 `VersionProjection` 的说明：「卸载记号与根记录其余字段不进投影」） |
| 238（改动计数留 1） | O2（独立解析器 + checker）：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比 inode 记录的改动计数；O3（独立规约执行器）：投影不进 inode 树 |
| 245（覆盖写的反向链取上一版记录自己的反向链） | O2（独立解析器 + checker）：I-8.6（反向链算法）（反向链不等的记录不进重放）批 1 流二 336 态，段为正 67/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 只带 journal 计数器，不带反向链；O3（独立规约执行器）：它的前缀规则核反向链（`scripts/experiments/e170_second_run_o3_spec_executor/replay_prefix.py` 的 `replay_prefix`，不等就停），这一行严格、暂代都判「在」；停下之后落到的那一版仍在 A(c) 里是推的（根槽已落、不靠重放），没逐态查它停在哪 |
| 258（不释放旧落点） | O2（独立解析器 + checker）：I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读） 批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedUnitAllocationRecords` 只带这一版每个角色的单元的分配记录（`model.rs` 里 `ObservedUnitAllocationRecords` 的定义），旧落点释没释放不在里面；O3（独立规约执行器）：投影不进分配记录树与记账树 |
| 265（抬 F 那一串不先写系统配置） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F） 批 1 流二 12 态（第 59、71 段各 6 态）；批 2 0 态 | O1（内存模型对拍（HashMap 理想文件系统））：比的是根带的 F，不比系统配置槽；O3（独立规约执行器）：投影的 F 取自根，A(c) 是这一行自己写的根。批 1 与批 2 红的列不同：批 2 没取第 59、71 段，取样点不敏感（「F4 新出的全放行行」一节） |

O3（独立规约执行器） 在 F4 计入分母的 37 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`），每行两条流的批 1 态都给了严格判定（`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:219-233`（整段抄，未转述）**

````markdown
### 没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行

`name=untouched_but_red rows=2 row_numbers=155,161`：两行的变异是同一处（`crates/singlefs-core/src/mount.rs` 里抬 F 上限 `newest_on_every_device.min(fourth_newest)` 改成 `.max(…)`，两行原文只差行首空白），触达三样都不同（`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`），O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二第 12 次调用 `raise_floor` 判 `disagrees:抬 F 的上限`（`call-outcomes.tsv`：模型「上限 Some(11)（D16（发布语义） 已定项 1）」，实现「实现报上限 13」），批 1、批 2 都红；O2（独立解析器 + checker）、O3（独立规约执行器） 不红。

- 不是装置、比对器或取样的缺陷：比对器的触达照登记第一节读法 4 字面算（「任一次调用的结局（成功 / 哪一种错误）」，`collusion_matrix.py` 的 `outcome_words` 只比调用名与结局列），这一次调用两边都是 `ok`；O1（内存模型对拍（HashMap 理想文件系统））-对拍比的是成功的调用交回的值（这里是抬 F 的上限），写流不变是因为这一次请求的 F 不超过 11，上限放宽没起作用。O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用算，与取样点无关，两批都红。
- 缺口在登记读法 4 的触达定义：它把「成功的调用交回的值」排在三样之外，却写「三样都相同的行……在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红」；这两行交回值不同，那句前提不成立。装置的 `call-outcomes.tsv` 只在对拍判出不同时记两边的值，成功的调用交回什么没有单独记，入库装置没有这一列。触达定义没改（产物跑过之后不改登记），分母仍 37。
- 别的未触达行有没有同类差异：攻方（第三轮）在 s4 树的副本上量了（副本探针，s4 树，入库装置没有这一列）。探针照流二的驱动在内存稀疏盘上跑一遍，记每次成功调用交回值的 `{:?}` 文本摘要、末了的分配器与现行那一版；基线与 E171（三oracle合谋测试） 导出的 595 条 op 逐条相同；再把未触达的行逐行打进副本、重编重跑、还原。跑了 232 行（238 行去掉 `make_filesystem.rs` 的 6 行，mkfs 不是一次调用，探针不记它的交回值，第 149–154 行没跑），7 行不同：第 49、110、155、161、162、174、252 行。第 155、161 行就是前面说的那两行（这把尺子在同一次第 12 次调用 `raise_floor` 上抓到它们，上限 11 → 13，是现成的阳性对照）；另 5 行 O1（内存模型对拍（HashMap 理想文件系统））-对拍不比那几个字段所以不红：第 49 行可写挂载交回的见证值 4 → 3，第 110 行写账的结构种类，第 162 行影子账分支名，第 174 行抬 F 交回的写账，第 252 行内存里的分配器少了根槽登记（第 49、252 行交回值或分配器不同，写流不变）。汇总行整行抄自攻方 `return-rows.out` 的 `name=return_summary` 那一行：

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

  主 agent 复跑（`tools/rerun.sh … all`）这 7 行逐字同；`return-rows.out` 只差变异表第 28–31 行那四条 `name=return_row`（复跑是 `status=no_output`、汇总 `hidden_difference=7 identical=221 no_output=10`，攻方是 `identical`）：攻方在决定去掉 `make_filesystem.rs` 的行之前已跑过这四行，`rerun.sh` 的 `awk` 把它们滤掉，复跑就不再跑（`research/prompts/e171-conclusion-r3-main-verification.md` 第四节）。害处没量：第 49、252 行推的是要更长的历史才分得开。
- 对其余读数的影响：两行不进分母，也不进任何 Q1、Q3 名单。照「触达」把它们算进去，D = 39，两行都只有 O1（内存模型对拍（HashMap 理想文件系统）） 红，Q3 只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的行多两行（第 155、161 行），Q1 三种读法的全放行行不变。把 7 行交回值不同的行与第 125 行（「它答不了的」）都算进来，D 由 37 变 45（推的，算术，没在入库装置上跑），那 5 行 O1（内存模型对拍（HashMap 理想文件系统）） 不红，会成为新的全放行行。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:234-261`（整段抄，未转述）**

````markdown
### 编不过的 6 行补位（已跑）

登记第一节读法 5「替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）」；s4 与 F4 那一版驱动只在编之前的失败上补位，第 33、59、127、163、206、243 行编不过之后照原样记编不过、没有补位。第八段把驱动改成编不过之后也补位（换同一文件组里下标 j + 1、两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点，编不过再换，编之前的失败与编不过合起来一行至多三次；组里不含 E171（三oracle合谋测试） 装置自己的变异行），加 `--rerun-compile-failed-from`（只跑几个输出根里结局是编不过的行）；比对器加 `--substitution-rows-root`（补位那一趟的行换掉原来编不过的同号行，别的情形拒），多出 `name=substitution` 与 `name=substitution_summary` 两种结果行。

- 补位那一趟在 s4 那一份冻住的树上跑（快照摘要与 s4 五趟的相同，空变异一行对 `baseline-run1` 未触达），只跑两份输出根里结局是编不过的行（`rerun_compile_failed rows=33,59,127,163,206,243`），两条线道、每条 2 线程，六步都退 0，补位 6 行挂钟 1 277.9 秒；比对器把 117 + 164 + 补位并成 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（4795 行，末行 `E7RESULT name=done emitted=4795`），耗时行在 `…-s4-f4-substitutes-rows-timing.out`（292 行）。
- 补位的六行（整行抄自 `…-s4-f4-substitutes-comparator.out` 里六行 `name=substitution`）、补位汇总（`name=substitution_summary`）与分母（`name=q1_denominator rows_run=`）：

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

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:262-290`（整段抄，未转述）**

````markdown
### Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像（整行抄自产物）

`…-s4-f4-substitutes-comparator.out` 第 4759、4760 行（补位之前的 `…-s4-f4-comparator.out` 第 4662、4663 行同两行，`sed -n '4662,4663p'` 与 `sed -n '4759,4760p'` 各自喂 `md5sum` 都是 `07b56b24014dac5d9eb543ef15384aa9`）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 张是 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 严格判「不在」。从 `name=q2_row` 逐行数同一批、按补丁种类分（命令与原样输出；`B` 是前面那份产物）：

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

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:291-393`（整段抄，未转述）**

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

`e171-three-oracle-collusion-2026-10-03-s1-comparator.out`，基线与对照（Z1 两趟 26 个文件逐字节相同、S2 与 E170（O3规约执行器第一版） 导出装置 18 项逐字节相同、S5 两个数比对器自己现算、a3 基线 0 红）：

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

Q2 的 O2（独立解析器 + checker） 一列（O3（独立规约执行器） 两列、盲测族没跑）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=N rows=10 o2_red=2 o2_quiet=8 o2_quiet_rows=new:14,new:15,new:16,new:17,fix:14,fix:15,fix:16,fix:17
E7RESULT name=q2_family family=K rows=41 o2_red=41 o2_quiet=0 o2_quiet_rows=none
E7RESULT name=q2_blind_family rows=0 expected_when_run=40 status=not_run why=seed_needs_the_frozen_executor_s7
E7RESULT name=q2_negative_bases bases=8 expected=8 o2_red_bases=0 red_base_list=none
E7RESULT name=a9_a10_a8 p_rows=26 p_rows_expected=26 n_rows=10 n_rows_expected=10 known_bad_rows=41 known_bad_rows_expected=41 blind_rows=0 q2_table_rows_when_the_blind_family_exists=107
E7RESULT name=q2_all_quiet status=not_run why=o3_frozen_version_not_available_s7 o2_quiet_rows_total=16
```

判决行（比对器自己的那一行；O3（独立规约执行器） 一律 `not_run`）：

```
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=6 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 known_bad_rows_with_o2_quiet=0 o3_columns=not_run q1_alpha=not_run q1_beta=not_run q2_all_quiet=not_run q3_alpha=not_run
```

`e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out`（第一刀第 6 行：分配记录树按槽号找叶时叶宽算成两倍，实现在第一次 `publish` 里 panic，装置记「实现自崩」、这条流不导出）：

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

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:394-400`（整段抄，未转述）**

```markdown
### 对照与自检

- 阴性（登记 5.6）：基线两趟、三批的每一态与每一次调用上，O1（内存模型对拍（HashMap 理想文件系统））-对拍、O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官、O2（独立解析器 + checker） 判红都是 0 处（a3，`name=a3_baseline_reds`）；空变异（拿表里第一行、把替换文换成原文本身，走完整条逐行驱动）判未触达、四列不红（`name=row row=0`、`name=row_touch row=0`、`name=row_matrix row=0`）；8 张不打补丁的底态（两条流的 F、Mk、Mk⁺、Mk⁻）O2（独立解析器 + checker） 不红（`name=q2_negative_bases`）。O3（独立规约执行器），第二段起：冻结的执行器在基线两条流批 1、批 2 的 576 态上判红 0 处，s4 产物原行 `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`；8 张底态 O3（独立规约执行器） 甲乙都不红（`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`）。s4 整表开跑前，同一棵冻住的树上的空变异一行对 `baseline-run1` 未触达（主 agent 跑的整表脚本第 3 步：`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`）。
- 阳性（装置一侧三条，登记 5.6）：O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二第一次 `publish` 的观测里把那一版文件内容的末字节翻一位，对拍判出一处；O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官在每条流批 1 第一个「实现读回了文件」的态把读回内容末字节翻一位，每条流各一处、合计 2 处（登记字面是「第一个」，装置按每条流各翻一次，对照要的「红」没变）；O2（独立解析器 + checker） 在流一全持久态上叠已知坏镜像族第一条，1 处。基线一趟与空变异一趟各跑一遍，红都一路带到 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`（比对器的 `name=positive_control` 六行）。O3（独立规约执行器） 的阳性对照（第二段起）：流一全持久态叠 P1 补丁交冻结的执行器，基线与空变异两趟各判红 1 处（s4 产物 `name=positive_control oracle=O3（独立规约执行器）` 两行）。翻过的那次对拍让模型停在那次调用，此后的调用不比、写落在它之后的态 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官记「不适用」，所以 O1（内存模型对拍（HashMap 理想文件系统））-对拍那一趟的 `o1_crash_judge_red_states` 是 0。
- 停机条款：s1 本地常量与 crates 回比 42 条全过；s2 两条流的八个文件与 `impl-verdicts.tsv`（去掉耗时列）逐字节等于 E170（O3规约执行器第一版） 导出装置在同一棵树上 `dump` 写出的；s3 O2（独立解析器 + checker） 直接调 `check_pool_image` 与 `judge_crash_image` 内那一遍判红的不变量集合在基线每个取样态上相同（`o2_matches_judgement_false=0`），在 76 行坏状态上与 E170（O3规约执行器第一版） `bad-impl-verdicts.tsv` 池级 checker 那一列逐行相同；s4 已知坏镜像族 41 条、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的、补丁叠回与直接改坏判红集合相等、目标不变量都红；s5 比对器自己现算的两个数（批 1 的态数与序号集合、O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数）与装置的计数相等；s6 没触发：E170（O3规约执行器第一版） 导出装置在 2026-10-03 的树上 `dump` 退 0、36 条 `holds=true`、0 条 `holds=false`；s7 在第一段触发（O3（独立规约执行器） 列不跑），第二段起 O3（独立规约执行器） 用登记修订 2 抄下冻结行的那一版，s4 比对器开头与收尾两次冻结核对都是 0 处不同；s8 的三处调用在「装置、研究一侧与变异表」一节给了行号。逐行那一趟的 S5（登记修订 3、4）在 s4 117 行上 0 行不过：判决行 `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`，`name=q1_denominator … device_stop=0`。
- 被测的是跑那一刻的树：s4 的空变异、117 行与 Q3-核三趟从同一份冻住的树拍快照（五趟驱动的 `snapshot_digest` 只有一种，`d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`）；开跑前树核对拿它与第六段读码时的清单逐文件比，只差一个别的 bin（`e168_delete_ring_window_borrow.rs`，不挡）。第一段的基线、对照与试跑用的是 `crates/` 在跑那一刻的状态；E170（O3规约执行器第一版） 导出装置一侧用的是 `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 取出的暂存版本（sha256 `8c526293…`，登记第三节钉的那一份）在草稿目录里编出来的，它的 `dump` 与 `bad-states` 输出与工作区里 E170（O3规约执行器第一版） 第 1 次重跑改过的版本逐字节相同（`cmp` 无输出），所以复跑驱动直接用工作区里的版本。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:401-406`（整段抄，未转述）**

```markdown
### 单测与变异

- Python 一侧单测 110 条（研究一侧目录里 `grep -c '    def test_' test_*.py` 四个文件 32 + 29 + 24 + 25），`python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` 连跑 5 轮每轮 `Ran 110 tests` / `OK`（手造数据、确定性，5 轮只说明没有隐藏状态）：矩阵（触达三样、O3（独立规约执行器） 两列的读法、全放行谓词、关一条、门槛、轨迹）、切段与取样公式现算、导出目录读回、坏实现表（锚点、补位、空变异）、挑法脚本的行数、冻结核对、坏镜像的 O3（独立规约执行器） 判法、逐行驱动的编译记录与重编核对（真 `rsync -a` 与 `copy2`、假 cargo）、F4 扩表（挑法脚本两次的差集与编号、比对器的逐行核对、两个输出根的合并与重号拒绝、分母按表分开、补位跳过两张表的名字）、编不过之后的补位（手造组 r0–r5、表里有 r0 与 r2、装置行插在中间：编不过依次换到 r1、r3、r4，第四次补不上记编不过；换上的行锚点不过与编不过合算三次；空变异不补位；组里没有装置行；驱动在同一个行目录里重编换上的行、`compile-failed-1.log` 与 `failed_attempt_1` 同号、复跑开关不补位；比对器只拿补位行换掉编不过的同号行、别的情形拒、补位行的结果行写换上的名字）。每个会被变异弄坏的谓词都配了钉绝对值的断言（例：扩表在手造表上必须是 `(118, a4)`、`(119, a12)`、`(120, b2)` 三行；补位的整行 `name=substitution row=3 … substituted_by=c_换 failed_attempts=1 …` 写死）。
- Python 一侧变异 67 条，表 `scripts/mutation/tables/e171_three_oracle_collusion.tsv`，跑法 `mutation_driver.py`（登记第九节写「经 `scripts/mutation/mutate.sh` 跑」，那条脚本只起 cargo test，起不了 Python 的单测，登记修订 1 第 1 行记了这一处）：`e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 里 `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`，抓到 / 无效 / 没红 = 67 / 0 / 0（结局符号逐个数：✅ 67、⏭ 0、❌ 0、⏱ 0、💥 0、⚠️ 0、🧱 0）。x60–x74 是编不过补位那一批（编不过不补位、不计次数、组里算进装置行、换上的行锚点不过时结局不记编不过、补位那一趟领了不是编不过的行、不记试过的名字、编不过之后不换行直接跑装置、比对器不核原行是编不过、不核补位自的名字、结果行写原名、合计把补不上的算换上、行分析不套补位、日志与试过的名字不同号、复跑开关不起作用、补位那一趟同一行在两个根里不拒）。同名的 `…-r6.out` 是同一批的前 13 条加进来时的全表跑（65 条，65 / 0 / 0），之后补了 x73、x74，留档。按「没红的三类」想过而没加的一条：编不过之后的补位从表里原名往下找、不从编不过的那一行往下找——补位跳过表里与已用过的名字，原名与编不过那一行之间的每个名字都在这两样里，两种找法在所有输入上同值，是等价变异，没加进表，等价性写成一条留档测试（`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`，手造组上逐个起点核）；加了这条测试之后变异表重跑一遍，输出与 `-r7.out` 逐字节相同。x52–x59 是 F4 扩表那一批（扩表不去掉重叠行、从 117 编号、把装置自己的变异行算进挑法、两个输出根同号不拒、补位不跳过别的表、扩表核对只比行数、分母按表对调、只读第一个输出根），每条都由 `F4ExpansionTests` 里对应的那条单测抓到。同名的 `…-r4.out` 是同一张表的一次中途被停的跑（外面套的 `timeout 230` 在第 8 条变异时停了它，61 行、没有判决行），不算数、留档。更早的 `…-s1-comparator-mutations.out`、`…-s2-comparator-mutations.out`、`…-s3-comparator-mutations.out` 与 `…-s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` 是表在 9、26、37、40、44、44 条时的全表跑，各自全抓。
- 装置内联单测 31 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）：编过（`scripts/hosts/compile-then-swap.py` 的 `cargo build` 与 `cargo test --no-run`），没跑（checker 档，归提交）。装置一侧变异 15 行追加在 `crates/mutations.tsv` 末尾（名字以「E171（三oracle合谋测试） 装置」起头），三个数待提交时门禁 checker-tier-crates-mutation-replay。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:407-851`（整段抄，未转述）**

````markdown
### 第 1 次重跑第一段·修订 6 那一趟（已跑：第一段的量跑完，Q5 告警自证已过、判甲；Q2 的集合与 Q3 的分母在第二段那一节）

登记 `research/prompts/e171-r1-prereg.md`（修订 6、7 与其后各条「主 agent 认定」，最后一条是「`-resume2` 跨机一致对照 S17 停机的处置」）；问题单 `e171-r1-questions.md` 的 Q1–Q5。树是修订 6 树 `scripts/experiments/target/e171-r1-frozen-tree-revision-6/`（快照摘要 `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`，284 个文件；第一段新树只换装置，不带第 31 批的 core 断言），装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 在树里那一份 sha256 `67e7a549…`。第 0–10 步后缀 `-r6-r3`，第 11、12 步（16 行）多机跑、后缀 `-resume2`，第 13 步没起（超时 0 行）。O3（独立规约执行器） 是 E170（O3规约执行器第一版） 第 2 次重跑的 v2-487（登记修订 1）。修订 6 那一趟的判定只属于修订 6 树；修订 8 那一趟（最终主线）还没跑，这里一个数都不推到它上面。比对器只读这一趟的产物前缀 `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`、后缀 `-r6-r3.out` 与导出根 `scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3/`，给到别的前缀就退出（登记修订 8 第 8.5 节第 1 条）。

| 行 | 判定（修订 6 树） | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| q1 | 甲：七行在这一批三条流上每行至少一条红（批 1 定判，严格与暂代两种 O3（独立规约执行器） 读法都是甲；七行都触达、三条流都跑完，无效行 0） | `…-s1-comparator-resume2-r6-r3.out` 的 `name=q1 reading=strict streams=three_streams batch=1 … verdict=jia` | 红在哪一格：第 15 行 O2（独立解析器 + checker），三条流，I-5.5（从 mkfs 水位建树那一版的 t1 落点）、第 23 行 O2（独立解析器 + checker），只在流三，I-1.1（块头自述逻辑地址）、I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外）、第 42 行 O1（内存模型对拍（HashMap 理想文件系统））-对拍（流二第 14 次调用 `unmount`，`disagrees:根带的 F`）、第 242、269、270 行 O3（独立规约执行器），严格、暂代同数、第 271 行 O2（独立解析器 + checker），三条流，I-9.27（树表条目的头 ID 取值）。只取流一、流二（与 s4 同一批历史的口径）时第 23 行三列全不红（`streams=two_streams … all_quiet_rows=23 verdict=yi`）：第 23 行只靠流三那段跨叶的历史才被抓。批 2 的判定同是甲，6 格在两批之间翻面（「q1 逐格（修订 6 那一趟）」一节）。O1（内存模型对拍（HashMap 理想文件系统）） 的两个崩溃子列在七行上一态都没红 |
| Q2 | 第一段只报七行（描述量）；第二段整表判乙（「第 1 次重跑第二段·修订 6 那一趟」一节） | `name=q2_named_row` 14 行（整行抄在「Q2 七行、对照与 Z1（修订 6 那一趟）」一节） | 七行在流一、流二上都未触达；第 155、161 行各 1 次调用交回值不同、胶水交给模型的观测也不同、O1（内存模型对拍（HashMap 理想文件系统））-对拍红 1 次；第 49、110、162、174、252 行交回值或内存对象不同而观测相同、O1（内存模型对拍（HashMap 理想文件系统））-对拍不红 |
| q3 | 甲：第 125 行计入分母（c = 12，触达） | `name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 … touched=true verdict=jia` | panic 在流二第 14 次调用 `unmount`（`attempt to subtract with overflow`），之前第 2–13 次调用 O1（内存模型对拍（HashMap 理想文件系统））-对拍都 `agrees`；流一没有 panic，比对器只数有 panic 的那条流（把流一那 1 次 `agrees` 也算上是 13，两种读法都 ≥ 1、判定不变）。分母 D = 39 出自第二段整表（「第 1 次重跑第二段·修订 6 那一趟」一节），第 125 行在那里同样计入 |
| Q4 | 基线甲（（ii）0 态）；第 242 行乙（（ii）12 态，第二截全判不合法）；第 269 行乙（（ii）58 态，全判不合法） | `name=q4_trip` 三行（整行抄在「第 1 次重跑第一段·修订 6 那一趟」一节判决行之后的代码块） | 三趟 × 三条流 × 两批 18 格每格都探过（`cells_without_a_probe=0`）。Q4-对账两数 18 格都是 0（`witness_refusals_without_either_branch=0`、`counter_branch_mounted=0`），S11 不触发。（i）判不出那一支另报：基线 1 340 态、第 242 行 2 650 态、第 269 行 1 340 态（两批合计，`undecidable_branch=` 之和），不进候选。（ii）的逐态清单与按段、按建 inode 排开的轨迹在「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节 |
| Q5 | 甲，告警自证已过（主取样点与反向取样点上 RefFS 红的态数都等于 L，模型崩溃判官 0 态红，两点同判） | `name=q5_judged` 两行（整行抄在「第 1 次重跑第一段·修订 6 那一趟」一节判决行之后的代码块）；告警自证见「作数的条件与 Q5（修订 6 那一趟）」一节末条 | 修订 5 第 5.6 节第 7 条与修订 6 第 6.6 节要的告警自证（U1–U9、U13–U16 与原有的两条内联用例绿，`crates/mutations.tsv` 里 V17–V31、V33–V36 全抓；U10–U12、V32 不进这一条）由主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 提前跑过，作数（登记末节「主 agent 认定：第一段（修订 6 树）Q5 告警自证」）。主取样点的 L 是按等价类推出的（`derived_by=equivalence_classes judged_states=144`），等价关系只在 k = 6 的 4 608 态上逐态核过（`mismatching_states=0`），其余态按类核对 ③ 与修订 5 甲推（修订 5 第 5.2 节）。按末段落了几个写排开的轨迹在「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节 |

比对器的判决行（`research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out` 的 `name=verdict` 那一行；补了逐态清单与轨迹之后的 `…-s1-comparator-resume2-r6-r3-2026-10-07.out` 的 `name=verdict` 那一行与它逐字节相同，这一节抄的判决行与 Q1、Q3、Q4、Q5、阴性、覆盖各行在两份里都有）：

```
E7RESULT name=verdict q1=jia q3=jia q4_baseline=jia q4_row_242=yi q4_row_269=yi q5_main=jia q5_reverse=jia q5_same_at_both_sampling_points=true a1_baseline_red=0 a8_failing_controls=0 a11_failing_streams=0 z1_differing_files=0 f7=false a1_empty_mutation_red=0 empty_mutation_touched=false
```

同一份产物的 Q1、Q3、Q4、Q5 与阴性、覆盖各行（整行抄）：

```
E7RESULT name=a1 batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1 batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=o3_third_stream_coverage stream=first_file_unlinked_then_inodes_across_one_leaf sampled_states=6459 strict_in_or_not_in_states=6459 f7=false
E7RESULT name=a11 stream=new_pool_file_creation states=82 crash_judge_applicable_device=82 crash_judge_applicable_recomputed=82 reference_filesystem_voting_device=82 reference_filesystem_voting_recomputed=82 holds=true
E7RESULT name=a11 stream=fixed_script_through_e_then_unmount states=890 crash_judge_applicable_device=890 crash_judge_applicable_recomputed=890 reference_filesystem_voting_device=890 reference_filesystem_voting_recomputed=890 holds=true
E7RESULT name=a11 stream=first_file_unlinked_then_inodes_across_one_leaf states=11695 crash_judge_applicable_device=11695 crash_judge_applicable_recomputed=11695 reference_filesystem_voting_device=81 reference_filesystem_voting_recomputed=81 holds=true
E7RESULT name=a1_empty_mutation batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation_touch row=0 streams_present=3 touched=false
E7RESULT name=q1 reading=strict streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=strict streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 states_judged=514 recovery_panic_states=0 touched=true verdict=jia
E7RESULT name=q4_trip trip=baseline counter_branch=0 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=jia
E7RESULT name=q4_trip trip=row_242 counter_branch=12 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q4_trip trip=row_269 counter_branch=58 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q5_judged sampling_point=main lost=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
E7RESULT name=q5_judged sampling_point=reverse lost=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
```

#### Q1 逐格（修订 6 那一趟）

批 1 每行每条流红的列（O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用，其余按态；括号里是轨迹：峰值、为正段数、期末值、单点红的段数）。在仓根跑的命令（只读 `name=q1_row … batch=1` 那 21 行，打出非 0 的列）：

```
grep 'name=q1_row .* batch=1 ' research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} if (f["o1_comparison_red_calls"]!="0") o=o" O1-对拍="f["o1_comparison_red_calls"]; n=split("O1-模型崩溃判官 O1-RefFS O2 O3甲 O3乙",c," "); for(j=1;j<=n;j++){k=c[j]; if (f[k"_red"]!="0") o=o" "k"="f[k"_red"]"(峰"f[k"_peak"]",正段"f[k"_positive_segments"]",末"f[k"_last"]",单点"f[k"_single_points"]")"} print f["row"], substr(f["stream"],1,9) (o==""?" 全不红":o)}'
```

原样输出：

```
15 new_pool_ O2=9(峰8,正段2,末1,单点1)
15 fixed_scr O2=240(峰8,正段47,末0,单点11)
15 first_fil O2=480(峰8,正段96,末0,单点24)
23 new_pool_ 全不红
23 fixed_scr 全不红
23 first_fil O2=29(峰8,正段6,末1,单点2)
42 new_pool_ 全不红
42 fixed_scr O1-对拍=1
42 first_fil 全不红
242 new_pool_ O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
242 fixed_scr O3甲=8(峰4,正段2,末0,单点0) O3乙=8(峰4,正段2,末0,单点0)
242 first_fil O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
269 new_pool_ O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
269 fixed_scr O3甲=41(峰8,正段10,末0,单点3) O3乙=41(峰8,正段10,末0,单点3)
269 first_fil O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
270 new_pool_ O3甲=3(峰3,正段1,末0,单点0) O3乙=3(峰3,正段1,末0,单点0)
270 fixed_scr O3甲=30(峰3,正段10,末0,单点0) O3乙=30(峰3,正段10,末0,单点0)
270 first_fil O3甲=714(峰3,正段238,末0,单点0) O3乙=714(峰3,正段238,末0,单点0)
271 new_pool_ O2=9(峰8,正段2,末1,单点1)
271 fixed_scr O2=353(峰8,正段69,末1,单点17)
271 first_fil O2=4749(峰8,正段950,末1,单点238)
```

- 对着登记读法 12（C673（三 oracle 合谋测试的补 oracle 改法没做） 改法落点）：第 15 行 ← ⑥ O2（独立解析器 + checker） 新池新建文件落点，O2（独立解析器 + checker） 红（I-5.5（从 mkfs 水位建树那一版的 t1 落点））；第 23 行 ← ⑦ 流三，O2（独立解析器 + checker） 只在流三红（I-1.1（块头自述逻辑地址）、I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外））；第 42 行 ← ④ 模型的卸载答，O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二那次 `unmount` 上红；第 242、269 行 ← ② O3（独立规约执行器） 谓词 W，O3（独立规约执行器） 红；第 270 行 ← ③ O3（独立规约执行器） 见证值不低于 tail，O3（独立规约执行器） 红；第 271 行 ← ⑤ O2（独立解析器 + checker） 树表条目头 ID，O2（独立解析器 + checker） 红（I-9.27（树表条目的头 ID 取值））。七个落点在这一批上都红了。O2（独立解析器 + checker） 红的不变量名取自各行导出 `oracle-verdicts.tsv` 第 8、9 列（两批合计）。
- 批 1 对批 2（第八节几何敏感性）：命令 `grep 'name=q1_row' <同一份产物> | awk '{for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} n=split("o1_comparison_red_calls O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官_red O1（内存模型对拍（HashMap 理想文件系统））-RefFS_red O2_red O3（独立规约执行器）甲_red O3（独立规约执行器）乙_red",c," "); for(j=1;j<=n;j++){k=f["row"]" "substr(f["stream"],1,9)" "c[j]; r[k, f["batch"]]=(f[c[j]]!="0"); keys[k]=1}} END{t=0; m=0; for(k in keys){t++; if (r[k,1]!=r[k,2]) {m++; print k, "batch1_red=" r[k,1], "batch2_red=" r[k,2]}} print "cells=" t, "flipped_between_batch_1_and_2=" m}' | sort`，输出末行 `cells=126 flipped_between_batch_1_and_2=6`，翻面的 6 格是第 15、271 行流一的 O2（独立解析器 + checker），第 269 行流一与流三的 O3（独立规约执行器） 严格、暂代——这 6 格记「不稳定」（批 2 没取到红的那几段）；各行在批 2 上照样有别的格红，Q1 批 2 仍判甲。
- 第 23、42 行的红只在一条流、一种 oracle 上；第 42 行只有 1 次调用（O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用算，没有轨迹）。
- 三份逐行产物里第 252、271 行的 `line_matches_the_registration=false`（行文的 sha256 与登记 5.9 不同，照冻结树里那一行跑、另记），其余十四行是 `true`；`substituted_from=-`、`failed_attempts=0`，没有补位。

#### Q2 七行、对照与 Z1（修订 6 那一趟）

```
E7RESULT name=q2_named_row row=49 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=49 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=155 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=155 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=161 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=161 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=1 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=a8 control=O1-对拍 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-对拍-卸载 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-崩溃判官 products=1 counted=o1_crash_judge_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=O2 products=1 counted=o2_red_states expected=1 observed=1 holds=true
E7RESULT name=a8 control=O3 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=RefFS products=1 counted=o1_reference_filesystem_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=panic products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=交回值 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=内存对象 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=见证 products=1 counted=counter_branch_states expected=1 observed=1 holds=true
E7RESULT name=z1_unstable_memory_objects names=-
```

- 每次调用逐行（`name=q2_named_row_call`，哪几个顶层字段、哪个内存对象不同）在同一份产物里，529 行，不抄；流三上第 110、252 行 238 次调用里都是写数（`writes`）与 `current_version` 不同。
- a8 那 4 样 `checked=false` 的（O3（独立规约执行器）、panic、交回值、内存对象）由按序跑法第 3 步自己的判法判过（`…-s1-positive-control-o3-p1-r6-r3.out` 的 `o3_control_verdict strict=not_in base_digest_matches=true red_strict=true`，交回值、内存对象的调用比对与 panic 截断各一核）；空变异那一趟十项（`…-s1-empty-control-*-driver-r6-r3.out`）同由第 4 步判，比对器不读它们。
- Z1：39 行 `name=z1` 全是 `identical=true`（`grep -c 'name=z1 .*identical=true'` 与 `grep -c 'name=z1 '` 都是 39），基线两次之间没有不稳的内存对象（`names=-`）。

#### Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）

登记第六节 Q4-（ii）要的逐态清单与第八节 Q4、Q5 的轨迹由比对器补出，在 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`（811 行）里。与 `…-s1-comparator-resume2-r6-r3.out`（679 行）比：前 677 行与判决行（新那份第 810 行）逐字节相同，只在 Q5 判定之后、判决行之前多出 132 行（`name=q4_counter_branch_state` 70、`name=q4_trajectory` 18、`name=q4_trajectory_by_inode_creation` 6、`name=q5_trajectory_cell` 36、`name=q5_trajectory` 2），收尾行由 `E7RESULT name=done emitted=679` 变成 `E7RESULT name=done emitted=811`（`diff` 只报 `677a678,809` 与 `679c811`）。轨迹三样（峰值与它第一次出现的格、为正的格数、期末值）照「端点不是轨迹」报；期末值是格号最大的那一格，Q4 按段排开时是这一批取到的最后一段，按建 inode 排开时是第 235 次建 inode，Q5 是末段全落那一格。

逐态清单按格归并（列的态：趟、流、批、段、这一段由第几次调用发出、计数那一支是哪一句、挂载结局、第二截判定；每一态另有序号、状态号、落地串的 sha256、镜像摘要、c_见证、所选版本、拒的成员、单子上的理由与没满足的条件，在产物里）：

```
grep 'name=q4_counter_branch_state ' <新那份> | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="trip"||a[1]=="stream"||a[1]=="batch"||a[1]=="segment"||a[1]=="call_of_the_segment"||a[1]=="counter_sentence"||a[1]=="mount"||a[1]=="second_crop") o=o" "$i}; print o}' | sort | uniq -c | sort -k2,2 -k3,3 -k4,4
      2  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=new_pool_file_creation batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=18 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=21 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=15 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=17 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=19 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
```

三条流的第 0、1 次调用是取号、暖机，流二第 4 次调用是可写挂载（各行 `calls.tsv` 第 2 列）。没满足的条件按计数那一支一一对上：`RecordAtTheWitnessedCounterIsNewer` 对 `witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion`，`AboveTheLastRecordOfTheSelectedVersion` 对 `…:WitnessedCounterAboveTheLastRecordOfTheSelectedVersion`，`NeitherRecordReadable` 对 `…:NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable`。每格列出的态数与同一格 `name=q4` 行的 `counter_branch=` 相等（18 格，0 格不等），轨迹各格之和也与它相等（18 格，0 格不等）。

按段排开（第 242、269 行；基线 6 行都是 0）：

```
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=1 cells=segments segments=9 first_segment=0 last_segment=8 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=2 cells=segments segments=4 first_segment=2 last_segment=7 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=60 first_segment=0 last_segment=59 counter_branch_by_cell=2:2,14:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=36 first_segment=2 last_segment=58 counter_branch_by_cell=2:1,14:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=720 first_segment=0 last_segment=719 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=478 first_segment=2 last_segment=718 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=1 cells=segments segments=12 first_segment=0 last_segment=11 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=2 cells=segments segments=1 first_segment=7 last_segment=7 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=79 first_segment=0 last_segment=78 counter_branch_by_cell=0:2,1:3,2:1,15:2,16:8,17:3,18:1,19:3,20:8,21:1 counter_branch_peak=8 counter_branch_peak_at=16 counter_branch_positive_cells=10 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=17 first_segment=7 last_segment=74 counter_branch_by_cell=16:7,20:7 counter_branch_peak=7 counter_branch_peak_at=16 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=960 first_segment=0 last_segment=959 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=238 first_segment=7 last_segment=955 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

流三按第几次建 inode 排开（235 次建 inode 各一格；段不在建 inode 调用里的（ii）态记 `counter_branch_outside_create_inode_calls`，全持久态那一段记 `counter_branch_after_every_call`）：

```
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=2 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=1 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=6 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

Q5 按末段里落了几个写排开（主取样点 30 格、反向取样点 6 格；装置的 `name=q5_trajectory` 行只出有 L 态的格，比对器补成第 0 格到末段写数那一格、没有的记 0，各格之和与 `name=q5` 行的 L、两个红数核过）：

```
E7RESULT name=q5_trajectory sampling_point=main cells=30 last_segment_writes=29 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:28,2:376,3:3224,4:19826,5:93104,6:347116,7:1054504,8:2658271,9:5633804,10:10133156,11:15571184,12:20533756,13:23297344,14:22762456,15:19137104,16:13810511,17:8518004,18:4460896,19:1965304,20:719026,21:214544,22:50876,23:9224,24:1201,25:100,26:4 reference_filesystem_red_peak=23297344 reference_filesystem_red_peak_at=13 reference_filesystem_red_positive_cells=27 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory sampling_point=reverse cells=6 last_segment_writes=5 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:4,2:4 reference_filesystem_red_peak=4 reference_filesystem_red_peak_at=1 reference_filesystem_red_positive_cells=3 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=13 lost_states=23297344 reference_filesystem_red=23297344 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=29 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=reverse writes_landed_in_the_last_segment=5 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
```

读法（只描述，不改任何判定）：
- Q4 的（ii）态在两行上都集中在取号、暖机那两次调用的段（段 0–2）与流二可写挂载那次调用的段（第 242 行段 14，第 269 行段 15–21）；三趟每条流每批的期末值都是 0，第二截判合法的态在每一格都是 0。第 269 行流二批 1 的峰值 8 在段 16（段 20 同为 8），为正段数 10。
- 流三的（ii）态全落在建 inode 之外的调用里（第 242 行批 1 2 态、批 2 1 态，第 269 行批 1 6 态），235 次建 inode 每一次 0 态：这一批上看不出（ii）随叶容器分裂变多变少，按建 inode 排开的那一格读数是 0，不是「分裂前后都有」。
- Q5 的 RefFS 红态数在每一个有 L 态的格上都等于那一格的 L 态数（主取样点 27 格、反向取样点 3 格），模型崩溃判官在每一格都是 0；末格（末段全落）的期末值是 0，因为那一格没有 L 态（`lost_states=0`），不是 RefFS 在那一格判绿。
- `q5_verdict` 在 L 个态上没有一态红、也没有一态两者都绿时报的结局由 `not_enough` 改成 `not_enough_f8`（登记修订 5 第 5.1 节第 7 条：照 F8 记不够判）；这一趟两个取样点都有红、到不了那一支，`q5_main=jia q5_reverse=jia` 不变。

#### 作数的条件与 Q5（修订 6 那一趟）

Q1–Q4 作数要第十一节原文加 S15 ①②③⑤⑥ 都不触发（修订 6 第 6.6 节）；Q5 另要主取样点两次 S1 全过、`s13=false`、阳性对照「判态路 panic 单列」恰为精确数、两次逐字节相同（Z7），反向取样点 S1 全过、S15 ④ 不触发，再加告警自证（已过，「作数的条件与 Q5（修订 6 那一趟）」一节末条）。整行抄（文件名省前缀 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`）：

```
frozen-tree-r6-r3.out:61: E7RESULT name=verdict s15_1_extra_files_against_the_first_segment_tree=0 s15_1_extra_files_against_the_q5_tree=0 s15_2_holds=true s15_3_unassigned_hunks=0 s15_3_hunks=24 s15_3_table_rows=24 a12_failed_checks=0 a12_checks=12 triggered=none
stream-constants-r6-r3.out:82: E7RESULT name=verdict s1_failed_checks=0
s2-r6-r3.out:20: E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18
s15-5-step-1-r6-r3.out:26: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-5-step-7-r6-r3.out:28: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-6-step-7-r6-r3.out:3: E7RESULT name=verdict s15_six_problems=0 triggered=false
s15-4-step-6-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-1-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-2-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
baseline-run1-r6-r3.out:2035: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0 probed_states=12667 counter_branch_states=0 counter_branch_states_judged_legal=0 undecidable_branch_states=1340 witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 piece_panic_states=- unattributed_panic_states=0
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:19: E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=6 states=4608 product_of_the_choices=4608 class_states=4608 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:23: E7RESULT name=q5_piece_panic_control states=4608 pool_checker=1152 ignored_recovery=1152 record_checker=576 other_pieces=0 consulted_recovery=0 consulted_recovery_of_the_plain_pairing=0 o1_columns_differing_from_the_plain_pairing=0 mismatching_states=0 s13=false holds=true
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:25: E7RESULT name=q5_shape sampling_point=main_before_the_first_barrier_of_call_3 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=81 operations_kept=77 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:54: E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:200: E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:228: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false piece_panic_states=- unattributed_panic_states=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:19: E7RESULT name=q5_shape sampling_point=reverse_at_the_return_point_of_call_2 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=57 operations_kept=53 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:47: E7RESULT name=q5 sampling_point=reverse_at_the_return_point_of_call_2 states=72 last_segment_writes=5 last_segment_tearable_writes=2 product_of_the_choices=72 a10_states_equal_the_product=true lost_the_confirmed_version=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=63 not_lost_o1_red=0 states_without_a_root=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:51: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=72 s13=false piece_panic_states=- unattributed_panic_states=0
```

- Z7：主取样点两次的 E7RESULT 行逐字节相同（`diff <(grep '^E7RESULT' …-run-1-r6-r3.out) <(grep '^E7RESULT' …-run-2-r6-r3.out) | wc -l` 输出 0，连耗时字段都不用去）。
- 第 11、12 步两份合成产物（`…-s1-rows-probe-driver-resume2-r6-r3.out`、`…-s1-rows-driver-resume2-r6-r3.out`）的 `freeze_check` 前后都是 `differences=0`，`snapshot_digest=7726cdd6…`，16 行 `status=ok`、`device_exit=0`、`o3_status=ran`，没有 253（第 13 步不起）。
- S17（跨机一致对照）：原产物 `…-s1-rows-cross-host-check-resume2-r6-r3.out` 判 `E7RESULT name=verdict cross_host_identical=false row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=o3`，作废，不进判定：主 agent 认定（登记末节「`-resume2` 跨机一致对照 S17 停机的处置」第 2–4 条）那一格的不同只在冻结执行器的工作进程号、工作进程数与多进程行序，是对照器缺陷，不是两台不一致。作数的是新入口重出的 `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out`（`E7RESULT name=verdict cross_host_identical=true row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=none`）；`…-2026-10-07-r2.out` 是同一个入口在仓里那一份上重出、补了收尾行 `E7RESULT name=done emitted=8` 的一版，前 7 行与 `…-2026-10-07.out` 逐字节相同；入口改成读同一目录里入库的编排器之后重出的 `…-2026-10-07-r3.out` 是复跑登记版，与 `-r2` 只差第 2 行的 `entry_sha256=`。
- 告警自证已过，Q5 判甲作数：主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 提前跑（用户 2026-10-07「把能做的往前做」），单元 `e171-q5-alert-selfproof`，日志 `~/.local/state/singlefs/detached-runs/e171-q5-alert-selfproof/log`；登记末节「主 agent 认定：第一段（修订 6 树）Q5 告警自证」原样抄的读数：仓里装置 sha256 `ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df`，装置内联用例 `test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s`（含 U1–U9、U13–U16 与原有两条），`crates/mutations.tsv` 全部 49 行「E171R1 装置」（含 V17–V31、V33–V36）`计数：抓到 49 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条（共 49 条）`。提交时这两样照旧在 checker 档复跑；那时仓里装置若已不是 `ab35105f…`，以那一次为准另记。

#### 比对器、复跑与入库的跑法（修订 6 那一趟）

- 比对器 `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py` 相对第一段新树那一份改了八处，判据一字没动（①–⑥ 由主 agent 认，登记末节「主 agent 认定：第一段（修订 6 树）比对器交回」第 1 条；⑦ ⑧ 是那一条第 3 条点名要补的）：① 读 `oracle-verdicts.tsv` 改成 25 列（`run_directory.py`，修订 5 第 5.1 节第 5 条在末尾加了 `piece_panics`，24 列的读法在这一趟的导出上读不动）；② a11 的现算改成「最后一个落了的写所在的调用」（与逐个写取最大值同值，单测对拍），流三 11 695 态由推的十分钟上下降到整份比对器 13 秒；③ 必须给 `--dumps-root`、`--product-prefix`、`--product-suffix`，输入不在这一趟里就退 2、不写产物（修订 8 第 8.5 节第 1 条）；④ 加「流三覆盖（O3（独立规约执行器））」一行与判决行的 `f7=`（第六节、f7）；⑤ 加空变异那一行的 a1 与触达（第七节 a1 后一半）；⑥ `q5_judged` 行多带两列 `judge_panic` 的态数（修订 5 第 5.1 节第 7 条，只描述）；⑦ 补第六节 Q4-（ii）逐态清单与第八节 Q4、Q5 轨迹（只多出新行，「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节）；⑧ `q5_verdict` 在 L 个态上没有一态红、也没有一态两者都绿时报 `not_enough_f8`（原来报 `not_enough`；RefFS 全弃权先走 f8 的次序不变）。
- 单测 41 条（`grep -c '    def test_' test_segment_one.py test_device_timing.py` 为 33、8），`python3 -B -m unittest test_segment_one test_device_timing` 连跑 5 轮都是 `Ran 41 tests … OK`（确定性的纯函数，5 轮只说明没有隐藏状态）。研究一侧变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv` 37 条（原 17 条，加 R14–R19 盯 ①–⑥，加 R20–R33 盯 ⑦ ⑧），整表 37 条全抓、0 无效、0 没红、0 超时，收尾已还原、基线仍全绿：`E7RESULT name=verdict mutations=37 caught=37 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（`…-s1-research-mutations-2026-10-07-r3.out`）。同一天先跑的两份 `…-s1-research-mutations-2026-10-07.out`（22 条，22 / 0 / 0）、`…-2026-10-07-r2.out`（23 条，23 / 0 / 0）是加 R19、R20–R33 之前的，留档；`…-s1-research-mutations.out` 是第一段新树那一趟的。原有的 R4（Q5 全弃权不走 F8）在 ⑧ 之后只在「RefFS 全弃权而模型崩溃判官有红」那一种输入上与原式不同，单测钉了这一格（F8 排在甲之前，原有次序）。
- 多机跑法入库（拷自草稿，原草稿留着不动）：`rows_orchestrator.py`（`7bfbdb84…`）、`peer_side.py`（`27cea9e7…`）、`check_product.py`（`3f237083…`）与 `` 三个文件与草稿里的逐字节相同；`cross_host_o3_check.py`（`afc9a9b0…`，草稿那份 `04376c00…`）改成读同一目录里入库的编排器（开跑照旧核 `7bfbdb84…`），`run_segment_one_on_the_revision_6_tree_multi_host.sh`（`567a8682…`，草稿那份 `ec89ec78…`）的编排器与 `check_product.py` 改指仓里这一目录（编排器按它自己的目录取 `peer_side.py` 与 ``）；冻结树、导出根、日志与续跑清单，与没入库的 `s2_compare.py`、`s15_compare.py`、`frozen_tree_lines_revision_6.py`、`resume_check.py` 照旧在草稿 `e171-r1-segment1-rev6/` 里。仓里那一份上的自证：编排器末行 `自证 34 项，红 0`，跨机对照入口末行 `自证 38 项，红 0`；`peer_side.py` 与跑法脚本没有 `--selftest`，跑法脚本过了 `bash -n` 与 `detached-run.sh long-run-check`（长活声明三个键齐）。修订 6 那一趟实际跑的是草稿里那几份（登记里的认定按草稿的 sha256 记）。
- 复跑：`scripts/experiments/replay.sh` 里加三行，都只在这一趟本机留存的导出与产物上重出，不起装置与 cargo：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_r1_s1_revision_6_comparator` | `…-s1-comparator-resume2-r6-r3-2026-10-07.out` | 比对器，导出根 `scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3/`；约 11 秒 |
| `driver_e171_r1_s1_revision_6_cross_host_check` | `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` | 跨机对照入口 `compare`，补收尾行 |
| `driver_e171_r1_s1_research_mutations` | `…-s1-research-mutations-2026-10-07-r3.out` | 研究一侧 37 条变异，约 40 秒 |

  三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同、`name=done emitted=` 与 E7RESULT 行数相等（811、8、78）。跨机对照那一份 `-r3` 与 `-r2` 只差第 2 行的 `entry_sha256=`（入口自己的 sha256，`04376c00…` → `afc9a9b0…`），判决行逐字节相同；三行原来指的 `…-comparator-resume2-r6-r3.out`、`…-2026-10-07-r2.out` 两份留着。导出根不进仓，它没了前两行就判红。第 0–13 步本身（冻结树上的装置各趟、逐行驱动、O3（独立规约执行器） 执行器、两台机器的编排）推的挂钟几十小时、要第二台，不登记。

#### 它答不了的（修订 6 那一趟）

- 判定只属于修订 6 树（第一段新树换装置，第 16 批合入之后、根环线 10b 之前那一份实现，不带第 31 批 core 断言）；修订 8 那一趟换成最终主线，两趟的对照另出（登记修订 8 第 8.5 节），这里不推。
- Q1 的甲只说「七行在这一批三条流、批 1 上每行至少一条红」，不说共同盲区补上了：表里没登记的坏法与没取到的历史照样可能三条一起放过。第 23 行只在流三上被 O2（独立解析器 + checker） 抓、第 42 行只在 1 次 `unmount` 调用上被 O1（内存模型对拍（HashMap 理想文件系统））-对拍抓；流一、流二的口径下第 23 行仍三条全放行。
- Q2 第一段只有七行的描述；集合与判定出自第二段整表（流一、流二，281 行加补位，集合正是这七行、判乙）。第 49、110、162、174、252 行交回值或内存对象不同而观测相同：差在胶水没交给模型的字段上，O1（内存模型对拍（HashMap 理想文件系统））-对拍按设计看不见。
- Q3 第一段只判了第 125 行计不计入；分母 D = 39 出自第二段整表。c 只数有 panic 的那条流（12）；把没有 panic 的流一也算进来是 13，判定不变。
- Q4 的（ii）态全在第 242、269 行上、第二截全判不合法；基线（修订 6 树上的实现那一趟）（ii）0 态。（i）判不出那一支不进候选，各趟上千态。三趟都是崩溃点重放截断出的态，没注入读故障。
- Q4 的逐态清单与轨迹只在两行坏实现（第 242、269 行）上有（ii）态，流三的（ii）态全在建 inode 之外：按建 inode 排开 235 格都是 0，说不出（ii）与叶容器分裂有没有关系，只说这一批上分裂前后的建 inode 调用里没有取到（ii）态。
- Q5 的主取样点 L = 150 994 944 是按等价类推出来的（逐态判了 144 态），等价只在 k = 6 的 4 608 态上逐态核过；反向取样点 72 态逐态判。告警自证已过，两点都判甲；告警自证只证 Q5 判决路上的告警会红，不扩大等价关系逐态核过的范围。
- O3（独立规约执行器） 一列是 v2-487；在流三上严格读法 6 459 态全给出「在 / 不在」（F7 不触发）。七行里只有第 242、269、270 行 O3（独立规约执行器） 红，其余四行 O3（独立规约执行器） 一态都没红。
- 第 252、271 行跑的是冻结树里那一行的行文（`line_matches_the_registration=false`），与登记 5.9 写下时的行文不同；它们照名字取到、锚点恰 1 次、没补位。

#### 产物（第 1 次重跑第一段）

修订 6 那一趟的产物（`ls research/results | grep '^e171-r1-' | grep -- '-r6-r3'`，81 份，含补了逐态清单与轨迹的比对器产物 `…-comparator-resume2-r6-r3-2026-10-07.out` 与入口改路径之后重出的跨机对照 `…-cross-host-check-resume2-r6-r3-2026-10-07-r3.out`）与研究一侧变异表的五份产物（`grep 'research-mutations'`）。`-timing.out` 是 E7TIMING 行；`…-rows-probe-driver-r6-r3.out`、`…-rows-probe-driver-resume1-r6-r3.out` 是第 11 步前两次起跑停下留的半份（各 1 行，不进判定）；`…-rows-cross-host-check-resume2-r6-r3.out` 判 `false` 的那一份作废（「作数的条件与 Q5（修订 6 那一趟）」一节 S17 那一条），不进判定：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-6-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-6-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-research-mutations.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations.out
```

修订 6 树之前几棵树上的第一段产物（85 份，`ls research/results | grep '^e171-r1-' | grep -v -- '-r6-r3' | grep -v 'research-mutations'`）：`2026-10-04` 起头的是修订 1 那棵树，`2026-10-05` 不带后缀的是第一段新树（`4d071c12…`，第 7 步 a1 在流三上不成立，修订 6 第 6.1 节），`revision-5-q5-` 是 Q5 树，`-r6`、`-r6-r2` 是修订 6 树前两趟停在第 0 步的记录。都保留为参照、不进任何判定（修订 6 第 6.1 节末表与「主 agent 认定：修订 6 起跑前仓里装置与写法偏离」第 3、5 条）：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-comparator-streams-1-2-before-the-rows.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-crash-judge-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-reference-filesystem-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-main-before-the-first-barrier-of-call-3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-on-the-draft-copy-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:852-952`（整段抄，未转述）**

````markdown
### 第 1 次重跑第二段·修订 6 那一趟（整表已跑：Q2 乙，Q3 分母 D = 39）

登记 `research/prompts/e171-r1-prereg.md` 第五节 5.5 第二段、第六节 Q2-集合与 Q3-分母两行、第一节读法 4 与读法 5、修订 5 第 5.1 节「已有产物的读法」末条（第二段只读 O1（内存模型对拍（HashMap 理想文件系统））-对拍与调用，两个崩溃子列不进这两行）、修订 6 第 6.6 节「第二段（整表）照原文，在修订 6 树上跑」，末节「主 agent 认定：第二段整表多机跑法交回」。树同第一段（快照摘要 `7726cdd6…`）；281 行，流一、流二，批 1 批 2，不跑执行器、不带探针，每行限时 1800 秒（T = 75.274 秒，3T 不到 1800）；两台机器跑完，超时 0 行，包装的结局 0 行，跨机一致对照第 201 行判同。

Q2 的「与基线不同」比的是修订 6 那一趟第 1 步 S2 新装置基线（`--streams 1,2`、不带探针，导出 `scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2/`）：修订 6 第 6.1 节末表说 Q2 的集合拿第 1 步这一趟比（旧装置那一份带不了根环表，比出来每行都会是「内存对象不同」），修订 2 第 2.5 节第二段的 T 也出自它。读法 4「基线连跑两次摘要不同的对象拿掉」：第 1 步只跑了一次，比对器另拿同一棵树、同一份装置的第 7、9 步两次三流基线（`baseline-run1/`、`baseline-run2/`）只比流一、流二，八个文件（写流四件、`states.tsv`、`oracle-verdicts.tsv`、`call-outcomes.tsv`、`call-return-values.tsv`）三份逐字节相同，不稳的对象 0 个。

| 行 | 判定（修订 6 树） | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| q2 | 乙：集合 7 行，O1（内存模型对拍（HashMap 理想文件系统））-对拍红的 2 行（第 155、161 行），没红的 5 行（第 49、110、162、174、252 行） | `…-2026-10-07-s2-comparator-r6.out` 的 `name=q2`（`set_rows=7 o1_red_rows=2 … verdict=yi`） | 集合正好是 C672（交回值或内存状态错而写流不变的一类） 点名的七行，整表别的 227 行未触达行交回值与内存对象都与基线同。53 次不同的调用里，胶水交给模型的观测也不同的只有第 155、161 行抬 F 那一次（`ceiling`），O1（内存模型对拍（HashMap 理想文件系统））-对拍在这两次上 `disagrees`；其余 51 次观测相同：不同的是模型看不到的字段（`writes`、`output`、`current`、`publishes`、`allocator`、`system_configuration_writes_before_the_first_publish`）与内存对象（`mount_output`、`current_version`、`allocator`），O1（内存模型对拍（HashMap 理想文件系统））-对拍都 `agrees`。七行逐次调用与第一段在三流带探针那一趟上算出的逐字同 |
| Q3 分母 | D = 39（第 1 次重跑读法 5、批 1 定触达） | `name=q3_denominator` 四行 | 原登记读法 5（调用 panic 与态上的恢复 panic 一律自崩）D = 38，差的是第 125 行；批 2 定触达 D = 35（原登记读法 5 是 34），差的是第 27、112、233、277 行（只在批 1 某态看 journal 那一遍的恢复报告上与基线不同）。不进分母：未触达 234 行、自崩 6 行（第 6、7、85、95、235、272 行）、找不到 2 行（第 107、168 行）；编不过的 5 行都补上了，超时 0 行。第 125 行在登记算法两批都计入，在原登记算法两批都不计入 |

判决行与 Q2、Q3 各行（整行抄自 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`，第 6、7、349、410、411、414、415、416、421、426、431–441、447–450 行）：

```
E7RESULT name=unstable_memory_objects stability_baselines=2 names=-
E7RESULT name=driver_product row_time_limit_seconds=1800 snapshot_digest=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4 rows=281 table_rows=281
E7RESULT name=q2_observation set_rows=7 calls_with_a_difference=53 calls_whose_observation_differs=2 calls_whose_observation_is_the_same=51
E7RESULT name=q2 set_rows=7 o1_red_rows=2 rows_o1_red=155,161 rows_o1_not_red=49,110,162,174,252 verdict=yi
E7RESULT name=q3_denominator reading=registration batch=1 rows=281 counted_D=39 counted=39 untouched=234 self_crash=6 not_found=2
E7RESULT name=q3_denominator_class reading=registration batch=1 class=self_crash rows=6 row_numbers=6,7,85,95,235,272
E7RESULT name=q3_denominator_class reading=registration batch=1 class=not_found rows=2 row_numbers=107,168
E7RESULT name=q3_denominator reading=registration batch=2 rows=281 counted_D=35 counted=35 untouched=238 self_crash=6 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=1 rows=281 counted_D=38 counted=38 untouched=234 self_crash=7 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=2 rows=281 counted_D=34 counted=34 untouched=238 self_crash=7 not_found=2
E7RESULT name=q3_self_crash_row row=6 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=7 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=85 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=95 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=125 where=fixed_script_through_e_then_unmount:call_14:judged_12 registration=counted original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=235 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=272 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_row_125 row=125 registration_batch_1=counted counted_registration_batch_1=true registration_batch_2=counted counted_registration_batch_2=true original_reading_5_batch_1=self_crash counted_original_reading_5_batch_1=false original_reading_5_batch_2=self_crash counted_original_reading_5_batch_2=false
E7RESULT name=recovery_panic_sum rows=0 states=0 by_row=-
E7RESULT name=not_located_row row=107 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=not_located_row row=168 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=untouched_row_red row=155 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=untouched_row_red row=161 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=driver_product_rows rows_in_the_product=281 rows_in_the_root=281 rows_whose_status_or_exit_differs=-
E7RESULT name=verdict q2=yi q2_set_rows=7 q2_o1_red_rows=2 d_registration_batch_1=39 d_registration_batch_2=35 d_original_reading_5_batch_1=38 d_original_reading_5_batch_2=34 row_125_counted_registration_batch_1=true rows_read=281 table_rows=281 rows_not_located=107,168 unstable_memory_objects=0 stability_baseline_files_differing=0 rows_compiled_without_the_rebuild_check=0 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 untouched_rows_with_a_red_column=2 rows_whose_directory_differs_from_the_driver_product=0
```

集合七行每次调用不同在哪（在仓根跑 `grep 'name=q2_set_call' research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out | sed -E 's/.* row=([0-9]+) .* fields=([^ ]*) objects=([^ ]*) observation_differs=([a-z]*) .*/\1 \2 \3 \4/' | sort -k1,1n -k2 | uniq -c`，列是次数、行、交回值不同的顶层字段、不同的内存对象、观测同不同）原样输出：

```
     10 49 - mount_output false
      1 49 output mount_output false
      1 110 #0 current_version,mount_output false
      1 110 current,output current_version,mount_output false
      1 110 - current_version,mount_output false
      1 110 publishes current_version,mount_output false
      3 110 writes current_version false
      7 110 writes current_version,mount_output false
      1 155 ceiling - true
      1 161 ceiling - true
     10 162 - mount_output false
      1 162 output mount_output false
      1 174 system_configuration_writes_before_the_first_publish - false
      1 252 allocator allocator false
     13 252 - allocator false
```

- 两种读法下自崩那几行：第 6、7、95 行在两条流的第一个文件那次发布（第 2 次调用）panic，之前只有取号、暖机两次 `not_modelled`，可判 0 次；第 85、235、272 行在第一次调用之前就 panic（断言是系统运行配置、系统不可变配置写出的字节数与码 2 头的偏移与字段表对不上，推是 mkfs 那一步，没核到调用栈），两条流都没导出、一次调用没发（装置 `name=stream … exported=false not_exported_why=pipeline_panicked:… calls=0`）。读法 5 收窄的原文是「第一处调用 panic 之前可判的调用为 0」，第一次调用之前就 panic 的这三行可判的调用同样是 0，比对器记自崩（原比对器把 `pipeline_panicked` 也记自崩）；这一处读法待主 agent 认，换成另一种读法这三行也不进分母（没有导出可比，触达判不出）。
- 找不到的第 107、168 行：两行的变异名在冻结树的 `crates/mutations.tsv` 里命中 0 次（行文在登记之后改过名），驱动补位要从「这一名在今天这个文件组里的位置」往下找，名字不在组里就找不到起点，记补不上（`failed_attempts=1`、`substituted_from=-`）；登记 5.2「补不上照实报」照办，不进分母。照登记表里记的组内下标往下找是另一种读法，待主 agent 认；那样至多多两行，D 是 39–41，Q2 的乙翻不了（集合里已经有红的行，也有不红的行）。
- 编不过而补上的 5 行（第 33、59、127、163、206 行，第 127 行换了两次）都照补位规则换成同组下一行，换上的都未触达（`name=substitution` 五行与 `…-s2-rows-substitution-coordination-r6.out` 判决行 `substitute_collisions=0`）。
- 校验：七行逐次调用的不同（行、流、调用、字段、对象、观测同不同）与第一段比对器在三流带探针那一趟上算出的 `name=q2_named_row_call` 流一、流二部分逐字相同（两份各取 53 行、去掉第二段多出的标签与 O1（内存模型对拍（HashMap 理想文件系统））-对拍两列、排序之后 `cmp` 无输出）。两份走的是同一个比较函数，独立的只是两趟装置运行与两份基线。
- 作废与停机：Z6 的 `rows_compiled_without_the_rebuild_check=0`；S5 的比对器一侧两数都是 0；两批的看 journal 那遍恢复 panic 0 态（修订 5 第 5.1 节数法，第二段不读两个崩溃子列，读数不变）；Z3 的包装结局与超时 0 行（合成产物 `check_product.py rows_without_executor` 记 `timed_out=none wrapper_outcomes=none`）；Z5 合成产物第 2 行快照摘要 `7726cdd6…`。

比对器、单测、变异与复跑：

- 比对器 `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/segment_two_comparator.py`，另写一份、不改第一段比对器（第一段判决、单测与变异表都不许变；第二段的输入是一份基线、一个行输出根与一份合成产物，与第一段不同）。触达、交回值差、可判调用数、自崩、S5 两数照用 `collusion_matrix.py`；只多批 2 定触达（只留那一批的态、改记批 1 再交给同一个触达函数）、两种读法的分类与汇总。只读这一趟的导出根（`revision-6-2026-10-05-r3/` 与 `revision-6-segment-2/`）与产物前缀 `…-2026-10-07-s2-`、后缀 `-r6.out`，别的退 2。整表约 16 秒。
- 单测 `test_segment_two.py` 17 条（`grep -c '    def test_' test_segment_two.py`），手造流与临时目录，每一类判定至少一条：批 1 与批 2 定触达、驱动与装置结局的各类、自崩与触达的先后、两种读法、mkfs 那一步 panic、分母与各类逐行、Q2 集合（触达的行不进、只批 1 触达的行不进、不稳对象拿掉）、Q2 甲乙丙与 F4、导出根，加一条 8 行整表落盘跑 `main`、判决行整行钉死。连跑 5 轮都是 `Ran 17 tests … OK`（确定性的纯函数，5 轮只说明没有隐藏状态）。
- 研究一侧变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv` 28 条（W1–W28），经同目录 `mutation_driver.py` 带 `--unit-test-module test_segment_two` 跑（驱动加了这个可选参数，不给时照旧跑第一段那两个模块：第一段那张表重跑一遍与 `…-s1-research-mutations-2026-10-07-r3.out` 逐字节相同）：`E7RESULT name=verdict mutations=28 caught=28 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（`…-2026-10-07-s2-research-mutations-r6.out`），抓到 / 无效 / 没红 = 28 / 0 / 0，连跑 5 轮输出逐字节相同。第一张表 37 条的锚点在加了新文件之后仍各命中 1 次。
- 复跑：`scripts/experiments/replay.sh` 加两行，都只在已有的导出与产物上重出、不起装置与 cargo：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_r1_s2_revision_6_comparator` | `…-2026-10-07-s2-comparator-r6.out` | 第二段比对器；读修订 6 那一趟第 1、7、9 步的导出与 `revision-6-segment-2/segment2-rows/`，本机留存、不进仓，没了就判红 |
| `driver_e171_r1_s2_research_mutations` | `…-2026-10-07-s2-research-mutations-r6.out` | 研究一侧 28 条变异，约 40 秒 |

  两个驱动函数照 `replay.sh` 的口径在草稿目录里跑过，输出与入库产物逐字节相同。整表本身（281 行逐行改坏、编、跑装置，两台机器）不登记。

第二段的产物（`ls research/results | grep '2026-10-07-s2-'`，8 份）：合成产物 `…-s2-rows-driver-r6.out`（565 行：281 行 `row`、281 行 `merged_shard` 与开头结尾几行）与它的 `-timing.out`；补位协调 `…-s2-rows-substitution-coordination-r6.out`；跨机一致对照 `…-s2-rows-cross-host-row-201-driver-r6.out`、它的 `-timing.out` 与判定 `…-s2-rows-cross-host-check-r6.out`（`E7RESULT name=verdict cross_host_identical=true row=201 host_of_rows_step=2 host_of_cross_host_run=1 parts=4 compared_parts=3 differing_parts=none`，o3 一格 `not_run`）；比对器 `…-s2-comparator-r6.out`；研究一侧变异 `…-s2-research-mutations-r6.out`：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-check-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-substitution-coordination-r6.out
```

#### 它答不了的（第二段）

- Q2 的乙只说「在流一、流二这两条历史上，未触达而交回值或内存对象不同的 7 行里 O1（内存模型对拍（HashMap 理想文件系统））-对拍红 2 行」：没红的 5 行差在胶水不交给模型的字段与对象上，O1（内存模型对拍（HashMap 理想文件系统））-对拍按设计看不见；它不说这 5 行的状态合不合法，也不说别的历史上集合多大。
- D = 39 是描述量，Q3 的甲乙由第一段第 125 行判（c = 12）；第二段不改第一段任何判定。D 的几个口径只差在读法 5 的自崩那一类与定触达的那一批，四个数都写。
- 判定只属于修订 6 树；修订 8 那一趟（最终主线）的第二段在哪棵树上跑另判（「主 agent 认定：修订 8 交回」第 8 条）。
- 第 85、235、272 行与第 107、168 行的归类是两处读法（「第 1 次重跑第二段·修订 6 那一趟」一节里「两种读法下自崩那几行」与「找不到的第 107、168 行」两条），待主 agent 认；两种读法都不让 Q2 翻面。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:953-969`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 批 1 的态数（装置） | 取样三批的公式作用在崩溃点重放枚举表给的每段序号区间上，再加撕裂扫描；枚举点，不是全量 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1359` `sample_ordinals` | `layer0_enumeration_tables` 的段区间与哪几次写取三态（来自 `crates/singlefs-checker-tier/src/crash.rs`） |
| 2 | 批 1 的态数（比对器现算） | 从 `ops.tsv` 按切段规则切段、`torn.tsv` 的写序号当三态写，每段状态数 = 各写取几态的乘积减 1，再按同一套取样公式现算序号集合 | `scripts/experiments/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | 取样公式（E170（O3规约执行器第一版） 登记 5.4 的文字）；`torn.tsv` 与 `ops.tsv`（装置从 crates 的枚举表与写表写出）：两条路径共用「哪几次写取三态」 |
| 3 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数（装置） | 一态里持久或撕裂的写属于的最后一次调用不早于模型停下的那次调用，这一态不适用 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1937` `crash_judge_is_applicable` | 调用表的返回点与写表的流序下标 |
| 4 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 的落地向量数写落在 `unmount` 那次调用里的态 | `scripts/experiments/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | `calls.tsv` 的返回点（装置写出）；模型只停在 `unmount` 这一条读法（登记第一节读法 6） |
| 5 | O2（独立解析器 + checker） 在坏状态行上的判定（装置） | 从中性格式按落地向量重建 `MemoryPool`、叠 E170（O3规约执行器第一版） 写的补丁、调 `check_pool_image` | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4751` `e170_bad_rows_on_both_streams` | 池级 checker 本身、E170（O3规约执行器第一版） 写的 `bad-patches.bin` 与落地向量 |
| 6 | O2（独立解析器 + checker） 在坏状态行上的判定（E170（O3规约执行器第一版） 导出装置） | `CrashImage` 上把补丁接在写表末尾当持久的系统配置槽写，调 `judge_crash_image` 取池级 checker 一项 | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:3014` `observe_bad_row` | 池级 checker 本身；同一批补丁字节与落地向量 |
| 7 | 第 1 次重跑：O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官适用的态数与 RefFS 投票的态数（装置） | 适用：一态落了的写属于的最后一次调用早于模型停下的那次调用；投票：RefFS 翻译没停在这一态的崩溃时刻之前 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3888` `crash_judge_is_applicable`；同文件 `:3172` `reference_filesystem_vote_of_a_state`（行号是仓里 `ab35105f…` 那一份的；修订 6 树里那一份在第 3475 行之后各少 7 行） | 调用表的返回点与写表的流序下标；`reference_filesystem_translation.rs` 的翻译停点 |
| 8 | 第 1 次重跑：O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官适用的态数与 RefFS 投票的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 现算：每个写落在哪次调用、最后一个落了的写所在的调用早于模型停下的那次；崩溃时刻取关上这一段的那一步，在 `unlink` 那几步或没成功的回退之前才投票；三条流全量态，枚举点不是扫遍 | `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py:176` `recomputed_applicable_and_voting` | 装置写出的 `calls.tsv`（第一条 op、返回点）、`ops.tsv`、`states.tsv` 的落地串与段号、`stream-status.tsv` 的 `model_stopped_at_call`；切段规则比对器自己照 `SegmentClosingRule` 写一份 |

路径 7 与 8 在第 1 次重跑比对器的产物里逐流比（`…-s1-comparator-resume2-r6-r3.out` 的 `name=a11` 三行都是 `holds=true`），同样没有 `assert` 把两条直接连起来；两条共用装置写出的调用表与落地串，它们错了两条一起错。

路径 1 与 2、3 与 4 在产物里逐格比（`name=s5_batch_1_states`、`name=s5_o1_applicable` 八行 `equal=true`），路径 5 与 6 逐行比（`name=e170_bad_row` 76 行 `o2_equals_e170=true`）；这几处比对写在产物里，没有 `assert` 把两条路径直接连起来，还欠这一条。共用项里最要紧的是池级 checker 本身（路径 5、6 同一份 `check_pool_image`）：它错了两条一起错，所以这一格不能当 O2（独立解析器 + checker） 判得对的证据；路径 1、2 共用「哪几次写取三态」，它错了两份态数一起错。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:970-1089`（整段抄，未转述）**

````markdown
### 复跑

装置与 E170（O3规约执行器第一版） 导出装置都是 checker 档的入库装置，经内存包装起；研究一侧是 Python。`scripts/experiments/replay.sh` 里 E171（三oracle合谋测试） 原来的十一行登记（第 1 次重跑另三行不在这张表里，在「第 1 次重跑第一段·修订 6 那一趟：复跑与入库的跑法」一节），各带自己的临时目录，产物只留 E7RESULT 行、exact 比：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | 装置 `baseline` 一趟 |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | 基线、E170（O3规约执行器第一版） 导出装置 `bad-states`（占位种子）、装置 `known-bad-images` |
| `driver_e171_s2_bad_states_export`、`driver_e171_s2_known_bad_images`、`driver_e171_s2_images_o3`、`driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`、`…-s2-known-bad-images.out`、`…-s2-images-o3.out`、`…-s2-o3-control-baseline.out` | 坏镜像三族在冻结种子下的造法、O2（独立解析器 + checker） 与冻结执行器的判定、O3（独立规约执行器） 阳性对照 |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | 第二段比对器那一串（九行试跑）；比对器在第三至六段改过判决行与 config 行，这份复跑对不上，没改指 |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | 研究一侧 67 条变异，`mutation_driver.py`，约一分钟 |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | 第二段那一串前置、修过的逐行驱动跑全部 117 行（两条线道、每条 2 线程，一份冻住的树）、Q3-核三趟、比对器；推的约两小时，没真跑过这个驱动函数（重型，归提交时的全部实验复跑）。117 行的读数读现行的 `crates/`，实现、O2（独立解析器 + checker）、O1（内存模型对拍（HashMap 理想文件系统））、装置或那 117 行的行文变了产物就变。整表那一步带 `--keep-compile-failures-as-they-are`：这份产物是编不过不补位的那一版驱动跑的 |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | 先整条跑 `driver_e171_s4_comparator`，再在它那一份冻住的树与工作根上跑扩表 164 行（同样带 `--keep-compile-failures-as-they-are`），比对器并两半、带扩表、段名 s4-f4；推的约四小时，没真跑过（`bash -n` 过；比对器那一半用同一组参数在已有导出上重跑过、与产物逐字节相同） |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | 先整条跑 `driver_e171_s4_f4_comparator`（它的草稿目录留着 s4、F4 的导出与冻住的树），再在同一份冻住的树与工作根上只跑两份输出根里结局是编不过的行（`--rerun-compile-failed-from`，补位跳过两张表的全部名字，两条线道、每条 2 线程，不带 `--keep-compile-failures-as-they-are`），比对器并 117 + 164 + 补位、带 `--substitution-rows-root`、段名 s4-f4-substitutes；推的约四个半小时（s4 与 F4 那一串约四小时，加补位 1 277.9 秒），没真跑过（`bash -n` 过；各步参数照补位那一趟实际跑的命令逐条对过，没有重跑驱动或比对器）。读现行的 `crates/`，实现、O2（独立解析器 + checker）、O1（内存模型对拍（HashMap 理想文件系统））、装置或那六行的行文变了产物就变 |

拿 s4 的输入（已有的导出目录与驱动输出）重跑 2026-10-03 的比对器，E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）：比对器这一半复得出；装置与驱动那一半要整表重跑才核得到。第一段几样的单条命令：

```
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd scripts/experiments/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

第一段比对器那份（`…-s1-comparator.out`，第一段的 `driver_e171_s1_comparator` 已由 s2 那一份取代、不再登记）要先跑齐基线两趟、E170（O3规约执行器第一版） 导出装置的 `dump`、关一条两趟、阳性对照六趟（基线三趟，空变异三趟经 `row_driver.py --empty-mutation --positive-control <名>`）、逐行驱动三行（`row_driver.py --empty-mutation --rows 1,6`），再 `python3 comparator.py …`，大约十几趟装置（每趟约一分钟），驱动函数把这一串写死；比对器的 config 行只写目录的名字，所以复跑目录的名字要与驱动里的相同。比对器的 a4–a7 读现行的 `crates/mutations.tsv`：表里别处加了核心实现的行或改了那 117 行，这几行会变，不是装置坏了。占位种子 `0000000000000000` 造出的盲测行 E171（三oracle合谋测试） 不用；E170（O3规约执行器第一版） 在第 1 次重跑里加了第二套坏状态表（`bad-states-second-run`），`bad-states` 仍是第一套，两个版本的导出装置在第一套上的输出逐字节相同（「对照与自检」一节最后一条）。

复跑驱动的自检：基线、已知坏镜像、变异驱动三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同（`cmp` 无输出）；比对器那一个驱动函数没有整条真跑（十几趟装置，前台跑不完），它的胶水（各步命令行、目录的名字、传给比对器的参数）用「装置、E170（O3规约执行器第一版） 导出装置与逐行驱动换成从已有导出目录拷」的桩跑过，比对器照真的跑，输出与入库产物逐字节相同；各步命令本身在做产物时逐条真跑过。整条真跑由提交时的复跑做。

产物（都在 `research/results/` 下，名字前缀 `e171-three-oracle-collusion-2026-10-03-s1-` 之后接）：

| 文件名（前缀之后） | 内容 |
|---|---|
| `baseline-run1.out`、`baseline-run2.out` | 基线两趟的 E7RESULT 行（两份逐字节相同） |
| `baseline-timing.out` | 两趟的 E7TIMING 行（每批每段耗时、每条流驱动、判、写各花多久） |
| `known-bad-images.out` | 装置 `known-bad-images` 模式：41 条已知坏镜像与 E170（O3规约执行器第一版） 的 76 行坏状态上的 O2（独立解析器 + checker） |
| `comparator.out` | 比对器：登记锚点、基线与对照、关一条、三行试跑、Q1 分母、Q2 的 O2（独立解析器 + checker） 一列 |
| `rows-timing.out` | 逐行驱动与比对器的 E7TIMING 行（Q4 单行数） |
| `comparator-mutations.out` | Python 一侧 9 条变异 |
| `e170-pinned-dump.out` | E170（O3规约执行器第一版） 暂存版本导出装置的 `dump`（S2 的对照） |
| `e170-bad-states-placeholder-seed.out` | 同一版本的 `bad-states`（P、N 行的出处，盲测行不用） |
| `positive-control-o1-comparison.out`、`positive-control-o1-crash-judge.out`、`positive-control-o2.out` | 三条阳性对照在基线那一趟的装置输出 |
| `disable-o1.out`、`disable-o2.out` | 关一条的开关自检（基线上各一趟）的装置输出 |
| `row-0-device.out`、`row-1-device.out`、`row-6-device.out` | 三行试跑的装置输出（row-0 是空变异） |

全名（`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`，17 份）：

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s1-comparator.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

第二段起的产物（`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`，29 份，名字前缀 `e171-three-oracle-collusion-2026-10-03-` 之后接）：

| 文件名（前缀之后） | 内容 | 算不算数 |
|---|---|---|
| `s2-bad-states-export.out`、`s2-known-bad-images.out`、`s2-images-o3.out`、`s2-o3-control-baseline.out`、`s2-o3-baseline.out` | 坏镜像三族的造法与 O2（独立解析器 + checker）、O3（独立规约执行器） 判定、O3（独立规约执行器） 阳性对照、冻结执行器在基线上（a3 的 O3（独立规约执行器） 一份） | 算：s4 比对器读的就是这几份 |
| `s2-comparator.out`、`s2-rows-timing.out` | 第二段比对器（Q2 全表、九行试跑）与它的耗时 | Q2 的数同 s4；九行试跑由 s4 整表取代 |
| `s2-comparator-mutations.out` | 变异表 26 条时的全表跑 | 历史 |
| `s3-baseline-after-the-fix.out`、`s3-row-0-device.out` | 修订 3 修过取号、暖机记红之后的装置基线与空变异一行（与修之前逐字节相同） | 算：修订 3 的实核 |
| `s3-k4c-attack-row-device.out`、`s3-k4c-check-driver.out` | 修订 3：取号恒拒那一行经逐行驱动跑一行（O1（内存模型对拍（HashMap 理想文件系统）） 红） | 算：修订 3 的实核 |
| `s3-comparator.out`、`s3-rows-timing.out` | s3 整表 117 行的比对器与耗时 | 作废留档（「s3 整表作废」一节） |
| `s3-clean-check-comparator.out` | 第五段在干净工作根上复核第 2、27、42、65 行 | 作废的佐证，留档 |
| `s3-comparator-mutations.out`、`s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` | 变异表 37、40、44、44 条时的全表跑 | 历史 |
| `s3-comparator-mutations-2026-10-03-r4.out` | 52 条那一次中途被停（61 行、没有判决行） | 不算数，留档 |
| `s3-comparator-mutations-2026-10-03-r5.out` | 变异表 52 条的全表跑，52 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r6.out` | 变异表 65 条的全表跑（x60–x72 加进来之后），65 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r7.out` | 变异表 67 条的全表跑，67 / 0 / 0 | 算：复跑登记指它 |
| `s4-comparator.out`、`s4-rows-timing.out` | s4 整表 117 行、Q3-核三趟、比对器与耗时 | 算：117 行那一半的读数（F4 产物里第 1–117 行的结果行与它逐字节相同） |
| `s4-f4-comparator.out`、`s4-f4-rows-timing.out` | F4 扩表 164 行与 s4 的 117 行并表的比对器与耗时 | 算：补位之前的读数（复跑登记里带 `--keep-compile-failures-as-they-are` 的那一份） |
| `s4-f4-substitutes-comparator.out`、`s4-f4-substitutes-rows-timing.out` | 117 + 164 行并上编不过 6 行的补位之后的比对器与耗时（补位那一趟 1 277.9 秒） | 算：补位之后的 Q1、Q3、Q4 读数（Q1、Q3 名单与补位之前逐字同），三轮判决引的产物行号取自这一份 |

全名：

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s3-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1090-1113`（整段抄，未转述）**

```markdown
### 它答不了的

- Q1 的「有」是照字面判的（≥ 1 行）；12 行（丙 13 行）里哪几行算真坏，三轮结论判决给了三分（「三轮判决之后的三分」一节）：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），状态不合法所依的条款句写在「三条全放行的 7 行」「F4 新出的全放行行」两节；另几类 5 行里第 92 行那一句在字节表与里程碑验收、不在决策正文。三分只是「这一批取到的态」上的读法，各行会分开的历史多数是推的、没跑。分母 D = 37 补位之前与之后相同（补位的六行都未触达），Q1 的「有」没翻。
- 281 行里 238 行未触达（补位之后；补位之前 232 行，另 6 行编不过）：这两条流的历史只走到实现的一小部分（取号、暖机、发布、可写挂载、管理员回退、抬 F、正常卸载各一两次，没有分裂、没有删除、没有空间紧的池），未触达只说那一处改动的写流在这两条流上没变，不说它被哪条 oracle 罩着；其中 7 行改动走到了、只是没写到盘上（第 49、110、155、161、162、174、252 行，「没触达却红的两行与未触达行里交回值不同的七行」一节）。三轮之后没有一行全放行行还记「对这一批等价」。
- 触达的定义（登记读法 4）不比成功的调用交回的值，这一条没改（登记不改），分母仍 37：第 155、161 行抬 F 的上限交回 13（模型 11）、写流与结局都同，记未触达，而 O1（内存模型对拍（HashMap 理想文件系统））-对拍判红；攻方在 s4 树的副本上量了 232 行未触达行的成功调用交回值，7 行不同（第 49、110、155、161、162、174、252 行），汇总行 `name=return_summary hidden_difference=7 identical=225 no_output=6`（副本探针，s4 树，入库装置没有这一列；整行与来历在「没触达却红的两行与未触达行里交回值不同的七行」一节）；mkfs 的 6 行（第 149–154 行）没量。
- 编不过的 6 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）；s4、F4 两份产物是补位之前的驱动跑的，复跑它们的登记行带 `--keep-compile-failures-as-they-are`、照旧不补位。
- 顺序类的条款（第 242、270 行）是在崩溃点重放的切段模型上数的：「条款不许的态」靠切段规则（FUA 写只放行它自己那块盘、屏障放行它那块盘）分得出，真机上一次 FUA 写与随后的写怎么排，不在这一批里。
- O3（独立规约执行器） 在 37 行上一行都没红：它的 A(c) 由实现自己写出的根构成，抓的是「恢复落到的那一版不在 [ack, begun] 里」这一类，实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判「在」。这是读执行器代码读出来的，没有另造一行只让 O3（独立规约执行器） 红的坏实现去核；坏镜像那一族里 O3（独立规约执行器） 红得多（Q2）。
- 三行试跑（第一段）不是表的抽样，s4 整表取代了它们当 Q1 的读数；第 1 行未触达只说那一处改动在这两条流的历史上没走到。
- 基线上 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官有一部分态不适用：流二最后一次调用 `unmount` 模型没有答，写落在这次调用里的态不判；基线批 1 流二 397 态里 48 个、批 2 的 119 态里 14 个不适用（`o1_crash_judge_not_applicable`），这些态上 O1（内存模型对拍（HashMap 理想文件系统）） 一列是空的。流一没有不适用的态。
- 阳性对照 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官每条流各翻一次（合计 2 处），登记字面是「批 1 里第一个」；O2（独立解析器 + checker） 的对照只在流一的全持久态上叠第一条已知坏镜像，没有别的取样点。对照证明管子把红带到矩阵，不证明每条 oracle 对每种改坏都分得出。
- Q2 的 O2（独立解析器 + checker） 一列取自两条路径（路径 5、6）；P 行的补丁字节与落地向量是 E170（O3规约执行器第一版） 导出装置写的，期望是 E170（O3规约执行器第一版） 设计员推的（登记 a2、F3）。Q2 全放行的 4 张全是 P8，那一判归 F3。41 条已知坏镜像是 O2（独立解析器 + checker） 自己的回归语料，O2（独立解析器 + checker） 41 条全红是构造的结果，不是 O2（独立解析器 + checker） 独立性的证据。
- 逐行驱动与装置读跑那一刻的 `crates/`：s4 五趟驱动是同一份冻住的树，但 `crates/` 里别处在变，整表跑出来的数只说那一份树。
- 坏镜像上实现的恢复读回什么（登记第一节读法 11，附带）、旧种子盲测族的回归都没跑。Q1-丁（三条全放行而记录核对器或崩溃点重放手写版本表的 oracle 红的行）跑了：F4 的 12 行全放行（甲、乙）里一行都没有（`name=q1_delta … =none`）；比对器只按甲乙算这一格，丙多出的第 233 行上崩溃点重放那份 oracle 与记录核对器红（`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`）；崩溃点重放那份 oracle 与记录核对器另在只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的第 108、115 行上红。
- 装置内联单测没跑（checker 档），装置一侧变异 15 行的三个数没有；Python 一侧的变异只证明比对器与驱动的谓词会红，不证明它们读的装置输出是对的（装置输出由两趟基线、S2、S5 与对照另行钉）。
- E171（三oracle合谋测试） 的 O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS（登记把崩溃判官算进 O1（内存模型对拍（HashMap 理想文件系统）），另写 E171（三oracle合谋测试） 不用 RefFS），与 D13（验证路线） 已定项 6 表 O1（内存模型对拍（HashMap 理想文件系统）） 行把崩溃之后允许看到的集合归 RefFS 不同；两者的下界也不同（盘上已持久的最新根，对最后一次返回成功的发布）。「已确认而丢了」那一类在 E171（三oracle合谋测试） 的四列里只有 O3（独立规约执行器） 判，没有在 37 行上跑过 RefFS（读码读出来的）。RefFS 的数只在三轮的第二轮判决里（`research/prompts/e171-conclusion-r2-main-verification.md` 第二节 R6 一格，读数在 `judge-summary.out`，攻方副本探针，s4 树 core，RefFS 直调 crate、崩溃落点手给、没经翻译层）：暖机那一形 O3（独立规约执行器） 判「不在」27 态，RefFS 判绿的也是这 27 态（27 / 27）；带内容的同形写流 O3（独立规约执行器） 判「不在」的 27 态 RefFS 全红，RefFS 红的共 36 态，另 9 态 RefFS 红而 O3（独立规约执行器） 判在（两边归属崩溃时刻的规矩不同，推的）。这几个数不是 E171（三oracle合谋测试） 装置读数。
- Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的 29 张（P 族 12、盲测 B 族 17，「Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像」一节）全是结构自洽的伪造根、伪造续接镜像，不是坏实现；补丁与期望同出 E170（O3规约执行器第一版） 的改法代码（E170（O3规约执行器第一版） 页「它答不了的」），源头独立在 Q2 只量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；流一 P2、P5 两种补丁的 O2（独立解析器 + checker） 一列随底态翻面。这 29 张的复跑读现行的 `crates/`，不还原 s4 的读数：基线流与 s4 树不同，数不保证仍是 12 与 17。D13（验证路线） 依据段只指到 E171（三oracle合谋测试） 页的这个数。
- 这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树：它与现行的 `crates/` 不是同一份，页里的名单不能直接当现行实现的名单；`replay.sh` 的 E171（三oracle合谋测试） 复跑行读现行的 `crates/`，复跑一次出的是复跑那一刻那棵树的读数。s4 那一版入库装置（`scripts/experiments/e171_three_oracle_collusion/`）上除 Q2 的 29 张之外，这几个数都不能原样重做（这一版装置没有挂载模式、没有 RefFS 列、panic 的流不导出；第 1 次重跑的装置 `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/` 有 RefFS 那一列，挂载模式与 panic 的流导不导出两项没核）。
- 第 125 行是自崩的 6 行之一：登记读法 5 把 panic 记自崩、不进分母，D = 37。它在 panic 之前有 13 次调用可判：副本探针在 s4 树上驱动、打第 125 行变异，第 4–13 次调用的全持久态 O2（独立解析器 + checker） 判 I-3.11（已分配减 defer 等于最新根走读） 红，第 14 次（`unmount`）panic（`drive-s4-m125.tsv`），装置对 panic 整条流不导出。把 panic 与报错同样截断照判，计入分母 D = 38（改口径的预演，不是装置读数，37 与 38 都写）；若触达也把成功调用的交回值算进去，含第 155、161 行，D = 40（37 加第 125、155、161 行）；7 行交回值不同的行都算进来、含第 125 行，D = 45（推的，算术）。
- 谓词 W（正确实现的可写挂载会把这一态读成「见证过的更新一版读不出」）在第 242、269 行取到的崩溃态上为真的态各 15、63 个，基线 0 / 576；s4 树 core 上两次可写挂载的见证类拒 61 个（第 242 行 9、第 269 行 52）。这些数是攻方副本脚本（W）与副本探针（挂载）在 s4 树上量的，入库装置没有 W、也没有挂载模式。2026-10-03 的 core 读 s4 写出的镜像大面积 `refused:Undecidable`（基线流一 44 / 60、流二 500 / 516）：格式变了（journal 头 311 → 487、新根段 188 → 364、点名项 67 → 64），旧记录在新解析器下读不出，不是回归；所以 W 的 15 / 63 与 61 个见证拒只对 s4 core 成立，在现行的 core 上没有对照意义。
- 用户数据落点的取值算不算 layout 登记的字节，这一页不判；三轮判决交用户（`research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行，零轮），用户 2026-10-03 定「算」：D3（空间分配） 已定项 8 射程改成新池新建文件上的落点取值（t1@50180）算格式级，检查在 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥，已还清（见欠账表已还清表）。它不改这一页的三分：第 15 行的依据是 ⑤ 的 t2–t12 落点；第 34 行的落点改动在流二抬 F 之后那次发布上，不是新池新建文件，仍归分配政策。
- 新树重跑装置（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`，这一页没记它的读数）打的 `name=batch`、`name=verdict` 两种行里，`piece_panic_states=` 按件计、不按态：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`；同一次 panic 在 `layer0_oracles` 与 `crash_judge` 两列各记一次（第 27–29 批代码轮判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第三节第 3 条，两腿同判，不改装置）。
- 变异表旧第 38 行已改名「步 4：回退候选集的 F 用目标根自己带的 F 而不是 F_生效」（改名后在 `crates/mutations.tsv` 里，旧名写「最新根自己带的 F」）；两份 `bad-implementations-117.tsv`（`scripts/experiments/e171_three_oracle_collusion/` 与 `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/` 下）第 33 行按旧名引用，从现行的树取快照重跑时这一行按旧名找不到、走补位、不静默失效（正在跑的修订 6 读冻结树里的旧表，不受影响；第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P4「旧第 38 行改名」一行与第三节第 5 条）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1114-1132`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D13（验证路线） 已定项 1 | 备料 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D13（验证路线） 已定项 7 | 备料 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 7 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 12 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D23（journal 的角色与格式） 已定项 3 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-11 不受影响：知识腐烂治理只改过时的现状句，结论与产物不变 |
| D3（空间分配） 已定项 10 | 不影响 | 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |

```
