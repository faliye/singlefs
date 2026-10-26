## E156 alloc-basis 四条岔路的代价数 —— 部分已跑（2026-09-26 第 4 次重跑：前提 1 成立（当天的代码上「记录已持久、根槽没持久」的崩溃之后可写挂载仍造得出洞）、前提 2 不成立（两个 F 口径在抬 F 那一串上合成一个）⇒ 岔路单第 3 行失效、不量；前提 3 只有岔路 2 那一族按旧式子定义、Hh 历史碰不到新准入式子；岔路单第 1 行在 38 条排得下的历史 × 「实」「每」两个回收时点上量完、按登记的够判条件够判：12 个取样点上 Q1c-前 都越过 2 槽（h_缺 = 1 时已 10 槽）、Q1d 都随 h_缺 增长，F1 触发；岔路单第 1 行用户 2026-09-27 定留到里程碑三、要考虑重排算法；入库装置，确定性，第 4 次重跑时 17 单测 / 装置的 22 条变异全抓；岔路单第 2、7 行用户已定不量；2026-10-05 第 5 次重跑：core 断言之后 W7 那一格的阳性对照换成 PC-钉门槛、6 个取样点两列全看到；第 4 次读数在新 `crates/` 上规范化之后 294 行不同（F1 仍触发，Q1d 在「实」列 4 个取样点不再随 h_缺 增长，K10 在 34 个观测点不符），归属跑证明不是 31-1 带来的，哪一份承重主 agent 已认定第 5 次产物（`e156-r5-prereg.md` 末尾「主 agent 认定：第 5 次交回与 S11 两边查」第 2 条；第 4 次产物当那棵树的记录留着，不再承重）；2026-10-05 第 6 次重跑：回收门槛照用户 2026-10-05 定乙读（挂载时读回全零的根环槽钉在它可能住过的最旧 txg）之后，38 格 K10、F24 都是 0，副本里去掉 core 的全零钉住时两项都红在推出来的 34 个观测点上；「实」列与第 5 次逐字段相同，「每」列随之与「实」列相同，Q1d 在「每」列 4 个取样点由随 h_缺 增长变成不单调，F1 仍触发、Q1c-前 判定不变；PC-钉门槛 六格两列与第 5 次逐字节相同；单测 19 条与变异 M45–M47 没跑，归提交时；2026-10-18 第 7 次重跑（问题单 H1，C566（环上有洞时多扣的槽随洞数无界增长） 那一半）：洞数从 1 档扫到环满一圈再跨圈，读法甲（一圈里排满，S = 8、16 的 8 个单位）与读法乙（跨过一圈，12 个单位）下 Q1a 峰值都判「仍在涨」，上界数全落在最大档（≤ 端点，没量到上界；读法乙最大档 S = 16 为 1810 / 1416 槽），H1 按登记的够判条件够判；Hh-后 8 个单位判「不单调」、F32 触发（登记预推每档相同，推错了）；内联单测 25 条由主 agent 跑过，新加的变异 M50–M54 没跑，归提交时）

**中文** · [English](156-cost-figures-for-the-four-alloc-basis-forks.en.md) · [日本語](156-alloc-basis-の四つの分岐に伴うコストの数値-一部実施済み.ja.md)

<!-- doc-lint:not-numbers K1 K8 S1 S2 S3 S4 S5 S9 V1 V2 V3 V6 V7 V8 V10 V11 V13 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F16 Q1 Q2 Q3 Q7 R1 R2 R3 R4 R5 R6 R7 R8 G7 G8 G12 G26 G27 G28 H0 HF K9 U5 U7 U8 U11 U13 F23 F24 M32 M34 A10 A14 F26 F27 F28 F29 F30 K10 K15 L1 L3 M35 M44 S11 S13 S14 U14 U19 V18 W7 W14 P7 Q1r5a Q1r5b Q2d R1a R1b R1c R1d R2a R2b R2c K16 L6 M45 M46 M47 U20 F31 F32 F33 F34 F35 V19 V20 S15 S16 S17 S18 M36 M42 F22 M50 M54 S20 W28 -->
本文里 K1、锚1/锚2/锚3（对应登记「七」第二类锚点表逐行的本地简写，不写成编号形状——kb 别的登记表已经用同一形状登记过别的含义，这里避开撞号）、K8、K9、S1–S5、S9、V1–V8、V10、V11、V13、F1–F14、F16、R1–R8、Q1a–Q1f、Q2a/Q2b/Q2c、Q3a–Q3e、Q7a–Q7f、U5、U7、U8、G7、G27、G12、甲-T1、F-扣、H0、HR、HK、HK-F0、HF、Hh(k)、HX、HY、β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0、Bd(k)、Bk1/Bk2/Bk3
这些编号与代号都是 `e156-preregistration.md`／`e156-r2-prereg.md` 跑前登记自己的锚点 / 判据 / 历史族 / 臂名，登记位在那两份文件里，不是 kb 的 D / E / C / I 编号。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| X8 | 小池耗尽两口径构造 |
| P21 | 释放越早等得越久 |
| I3 | 一次挂载分配记录只增不减 |
| N2 | 释放时改写的分配代字段 |

**备料**：给 alloc-basis 四条岔路（`alloc-basis-forks.md`）交用户之前配代价数——判决收口表第 ② 行因此挡着第 5、60、54、43 行；用户 2026-09-21 定「先跑计数实验，量完一次全交」。第二、七行判决标着「无运行时代价」，第二行量的是「删掉 D28（挂载期承诺量） 已定项 1『− defer 待释放』这一项要同步改的清单能不能被一条命令核」，第七行量的是「G27（给 defer 账加的一条检查）分不分得出今天两条检查判不出的差别」。等 D28（挂载期承诺量） 已定项 1 与 D16（发布语义） 已定项 1 按岔路定案时引用 E156（alloc-basis 四条岔路的代价数）。岔路单第 2、7 行用户已定（岔路单那两行的状态列），第 3 行按第 4 次重跑的前提 2 失效、不量；第 1 行用户 2026-09-27 定留到里程碑三、要考虑重排算法（弹窗原话「可以留到里程碑3 但是要考虑重排算法」），记成欠账 C566（环上有洞时多扣的槽随洞数无界增长）。

**问题**：第 4 次重跑登记 `e156-r4-prereg.md`（回答问题单 `e156-r4-questions.md` 的四节）。实一至实五之后，管理员回退改成挂着时的一次向前发布（挂载时回退与截断删了）、每次发布都做挂载内回收、准入不再扣「defer 待释放」；岔路单第 2、7 行用户已定不量，第 1、3 行先答三个前提：前提 1（今天的代码上根环还会不会出洞）、前提 2（两个 F 口径在今天的抬 F 路径上还分不分得开）、前提 3（哪几个量是按旧准入式子定义的）。前提 1 成立才量第 1 行（登记第五节 5.5 的第二段），前提 2 不成立则第 3 行失效、不设量。装置是**入库装置**：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`，只读驱动真实 mkfs、暖机、新池新建文件、覆盖写、空发布、可写挂载、挂着回退、抬 F，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码。

### 第 7 次重跑（2026-10-18）：洞数从 1 档扫到环满一圈再跨圈，读法甲、读法乙下 Q1a 峰值都判「仍在涨」，上界数全落在最大档（≤ 端点，没量到上界）

重跑登记 `research/prompts/e156-r7-prereg.md`（含执行员在装置写完、产物一次没跑之前写的「修订 1」），问题单 `research/prompts/m3-close-cost-measurement-questions.md` 的 H1 一行，量 C566（环上有洞时多扣的槽随洞数无界增长）与 C380（环上有洞时释放代判不出唯一值）那一半：今天的 `crates/` 上，洞上多扣的槽数随洞数增长是否有上界、上界是多少；够判条件逐字是「洞数至少取三档（含环满一圈），每档报多扣槽数」。H1 的三个候选（甲 可写挂载时补洞、乙 今天的代码上多扣有上界而改写验收并销账、丙 前洞多扣挪出里程碑三）不是登记里的臂（登记 5.1）：这一次只量今天的代码（甲-T1 那一臂，「实」列为准、「每」列并报），哪个候选成立不在这一节判。

**读法写死**（登记 W26、W28）：「环满一圈」两种读法都量、各占一行、判据形状相同——读法甲＝一圈里排满（R7a：前位置 Hh(k, 前)，k = 1 … `k_满(S)`，S = 4 / 8 / 16 时 2 / 4 / 8 档，S = 4 只有 2 档、只报不判）；读法乙＝跨过一圈（R7c：Hh(1, 前) … Hh(`k_满` + 1, 前) 再加 Hc 两档，`k_满` + 3 档）。统计单位 (族类, S, ρ, 时点) 取每档 Q1a 峰值，按档升序：相邻差全 ≥ 0 且最后一个 > 0 判「仍在涨」（没见上界，≤ 端点）；全 ≥ 0 且最后一个 = 0 判「已平」（上界 = 末档峰值）；其余判「不单调」。

**装置与产物**（命令与原样输出是这一次真跑的；单测、挂钟、实现快照三项出自主 agent 的跑产物脚本，标明）：

| 项 | 命令 | 原样输出 |
|---|---|---|
| 装置行数 | `wc -l crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` | `7580`（`git show HEAD:<装置> | wc -l` 是 `6291`） |
| 装置源文件 sha256 | `sha256sum crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` | `bcb356afd4867a27f80c943b10d62962e520cbf74a4d328327722959c6a7cdb3`（与登记「修订 1」记的相同，跑产物前后的主 agent 脚本也记了这一个） |
| 单测条数 | `grep -c '#\[test\]' <装置>` | `25`（`git show HEAD:<装置> \| grep -c '#\[test\]'` 是 19，新加六条） |
| 单测读数 | 主 agent 在 release 下跑装置的内联单测（checker 档，重型，执行员没跑；日志不入库） | `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s` |
| 产物 | `wc -l research/results/e156-alloc-basis-counts-2026-10-18.out`；`tail -n 1 <产物>` | `16669`；`E7RESULT name=done emitted=16669` |
| 产物 sha256 | `sha256sum research/results/e156-alloc-basis-counts-2026-10-18.out` | `1663f1ea28e5b3334f21adfc121bbf899bc66e7e429eea782f1f0c562c776c83` |
| 跑产物的二进制 | 主 agent 的脚本在跑之前记的 `target/release/e156_allocation_basis_counts` | sha256 `eb3903266a015578b10c412a18a7a988eda0dfbf9fc161790658417d142b2d70`；此刻仓根 `target/release/` 下那一份是 `81106e66389b91790a16726e56001648b950cabe720386457a02f9cd65fa05c9`（产物之后又编过，源文件 sha256 不变，产物不是它跑的） |
| 挂钟 | 主 agent 的脚本量 | `挂钟秒数 241`（release，线程上限 31，内存上限 8G） |
| 实现快照（S19、S3） | 脚本在跑前、跑后各取一次 `git diff HEAD -- crates/ \| sha256sum`、`git diff HEAD --stat -- crates/ \| tail -1`、`git status --short crates/ \| sha256sum` | 前后都是 `7fd21f75d5e478c1342b53d0d54b123303ac7993b0e4af473d049dceea10d5ca`、` 190 files changed, 73166 insertions(+), 4603 deletions(-)`、`4d180fd57ae7b242a5579beeca267bdf9441fb36f9cc7442a9246cdc71ba33a4`；登记第三节写的 `847e0e54…` 与修订 1 开工时的 `16ba9b79…` 都与它不同（别的会话在改 `crates/`，登记 S19 第 2 项本来就不停，被测对象是跑的那一刻的树） |

**起跑经过**：执行员交的 `run-product.sh` 不带 `gpu` 特性编装置，而多机配置 `ENABLE_GPU=1`，给定状态表的判段交回占位结论（主 agent 的诊断，那一趟没有独立复现），装置在 `verdict` 函数里 panic（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 里 `PlaceholderOfATableLeftToAnotherHost`、`PlaceholderOfATableRecordedOnly` 两个分支，消息只点名 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 与 `SINGLEFS_CRASH_AMPLIFICATION_SHARE` 两个变量）；那一趟的输出被后一趟覆盖，没留存、不是产物。主 agent 改成带 `--features verdict-store,gpu` 编、经 `scripts/crash/gpu-quiesce-run.sh` 起，跑出 `research/results/e156-alloc-basis-counts-2026-10-18.out` 这一份。S3 前后快照相同，说明这一趟里 `crates/` 没被别的会话改动。

**守门、停机与作废条款**（登记第十一节）：

| 登记项 | 读数 | 结论 |
|---|---|---|
| S20 装置的 Rust 锚点模型与 Python 锚点脚本逐点比 | `anchor_r7_summary` 58 行、`anchor_r7_dump` 3567 行、`anchor_r7_k0_summary` 3 行、`anchor_r7_k0_dump` 384 行，与改正后脚本（`anchors_e156_r7_hdong_window.py`，sha256 `a33d5f8c…`，k0 包装 `anchors_e156_r7_k0.py` sha256 `c9bc6de0…`）的输出 `cmp` 都无差异；执行员开工前的试跑触发过一次 S20（脚本漏了 h_洞 的窗口左端），处置在登记「修订 1」第 2 条 | 不触发 |
| S21 真实分配器读数与锚点对不上（含「峰值不低于锚点树表下界」那一条） | `q1_cell_checks` 116 行里 `a15_every_below`、`a15_real_below` 的和 0，`a14_mismatches`、`a11_mismatches`、`a23_lag_mismatches`、`a24_pinned_mismatches` 的和都是 0；另用独立脚本对 110 格 × 两列（实、每）核峰值 ≥ 锚点 `q1a_tt_peak`，220 个全部不低于 | 不触发 |
| S11 / S12 守门（K10、F24、G-adm） | `verdict stage=second`：`s11_k10_violation_points=0`、`s11_a10_real_mismatch_cells=0`、`s12_g_adm_nonzero=0`；`failure_clauses` 的 `f24_real_every_differ_points=0` | 不触发；PC-c2 对照格 `a14_mismatches=9`、`seen=true` 按设计非 0 |
| V20 判定函数的阳性对照（PC-档序列） | 五个合成序列 `holds=true`，`r7_verdict` 汇总 `tier_sequence_control_cases_wrong=0` | 不触发 |
| V21 PC-同槽 看不到下降 | 六个 (S, ρ) 的 Hs(2, 后) 上「每」列峰值由不改时的 344 / 236 / 712 / 476 / 1460 / 960 降到 10，`seen=true`；汇总 `same_slot_controls_run=6 same_slot_controls_without_a_drop=0` | 不触发 |
| 判别力自证 ② 去掉最后一档重判 | 汇总 `dropping_the_last_tier_changed_units=0_of_24`（R7a、R7c 的 24 个统计单位里没有一个因为去掉最后一档改判；S = 4 的 R7a 4 个单位本来就只有 2 档） | 判定对最后一档不敏感 |
| 判别力自证 ③ 2 槽 / 1 槽门槛挪到两个观测值之间 | 汇总 `threshold_moved_units_flipping=60_of_60` | 做得出，V5 不触发 |
| 其余作废条款（V1、V2、V6、V16、V17） | `verdict stage=second` 里对应字段 `v1_over_withheld_controls_void=0`、`v2_a10_every_mismatch_cells=0`、`v6_a11_mismatch_points=0`、`v16_judge_cases_wrong=0`、`v17_a13_every_nonzero_points=0` 都是 0 | 不触发 |
| 截断 | `truncated_cells=0` | 116 格都跑到登记的终点，没有「未量到」 |
| 旧失败条款照报（F1、F2、F22、F23） | `failure_clauses`（`:16654`）：`f1_front_above_two_groups` 列了 12 组全部取样点，`f2_no_observable_difference=false`，`f22_front_and_back_differ_groups=none`，`f23_back_at_most_two_groups=none` | 与第 6 次同形（F1 仍触发，岔路单第 1 行的判决用户 2026-09-27 已定，不重下） |

**数量自查**（登记第五节 5.2：每个 ρ 上 S = 4 / 8 / 16 各 14 / 18 / 26 条历史，共 116 条；`q1_cell` = 116 + PC-c2 对照 1 + PC-同槽 6 = 123）：

```
$ for n in q1_cell q1_cell_reused q1_cell_checks r7_tier_peaks r7_verdict pc_same_slot pc_tier_sequence anchor_r7_summary anchor_r7_dump anchor_r7_k0_summary anchor_r7_k0_dump; do printf "%s %s\n" "$n" "$(grep -c "^E7RESULT name=$n " research/results/e156-alloc-basis-counts-2026-10-18.out)"; done
q1_cell 123
q1_cell_reused 2
q1_cell_checks 116
r7_tier_peaks 60
r7_verdict 6
pc_same_slot 6
pc_tier_sequence 5
anchor_r7_summary 58
anchor_r7_dump 3567
anchor_r7_k0_summary 3
anchor_r7_k0_dump 384

$ grep '^E7RESULT name=q1_cell ' research/results/e156-alloc-basis-counts-2026-10-18.out | awk '{print $3, $4, $5}' | sort | uniq -c
      2 family=Hc s=16 rho=1
      2 family=Hc s=16 rho=1/4
      2 family=Hc s=4 rho=1
      2 family=Hc s=4 rho=1/4
      2 family=Hc s=8 rho=1
      2 family=Hc s=8 rho=1/4
     18 family=Hh s=16 rho=1
     18 family=Hh s=16 rho=1/4
      6 family=Hh s=4 rho=1
      6 family=Hh s=4 rho=1/4
     10 family=Hh s=8 rho=1
     10 family=Hh s=8 rho=1/4
      6 family=Hs s=16 rho=1
      6 family=Hs s=16 rho=1/4
      6 family=Hs s=4 rho=1
      6 family=Hs s=4 rho=1/4
      6 family=Hs s=8 rho=1
      6 family=Hs s=8 rho=1/4
      1 family=PC-c2 s=4 rho=1
      1 family=PC-same-slot s=16 rho=1
      1 family=PC-same-slot s=16 rho=1/4
      1 family=PC-same-slot s=4 rho=1
      1 family=PC-same-slot s=4 rho=1/4
      1 family=PC-same-slot s=8 rho=1
      1 family=PC-same-slot s=8 rho=1/4
```

每个 ρ 上 S = 4 的 `Hh` 6 + `Hc` 2 + `Hs` 6 = 14，S = 8 为 10 + 2 + 6 = 18，S = 16 为 18 + 2 + 6 = 26，与登记一致；`q1_cell_checks` 116 行（每条历史一行）；`r7_tier_peaks` 60 行 = 5 个量（R7a、R7b、R7c、R7d 前、R7d 后）× 3 个 S × 2 个 ρ × 2 个时点；`q1_cell_reused` 2 行是 S = 4、ρ = 1 的 Hh(1, 前)、Hh(1, 后) 复用第一段的格。`排不下` 的 36 格（`verdict stage=second` 的 `cells_run_without_fitting_the_ring=36`）等于锚点脚本输出里 `fits=False` 的 18 条历史 × 2 个 ρ：

```
$ grep -c 'fits=False' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.out
18
```

S20 与核对脚本（对应守门表里 S20、S21 两行，原样输出）：

```
$ sed -n 's/^E7RESULT name=anchor_r7_summary //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.out && echo SUMMARY_SAME
SUMMARY_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - <(grep '^DUMP ' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.dump | sed 's/^DUMP //') && echo DUMP_SAME
DUMP_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_k0_summary //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.out && echo K0_SUMMARY_SAME
K0_SUMMARY_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_k0_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - <(grep '^DUMP ' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.dump | sed 's/^DUMP //') && echo K0_DUMP_SAME
K0_DUMP_SAME
$ sed -n 's/^E7RESULT name=anchor_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | wc -l
1092
$ sha256sum research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.py research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.py
a33d5f8ce5bb70585f1372ac8fa8209a0c3dac3282026d7fd77c152f673d2b6e  research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.py
c9bc6de064be9e6d89da881cd93d25ec04604e3b7206588fb5b078b3b2016397  research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.py
$ python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_tier_peaks_against_observations.py
r7_tier_peaks 行 60 条，用原始观测点重算后峰值序列不同的 0 条；有观测点的格 123 个
$ python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_peaks_against_anchor_lower_bounds.py
格 × 列 220 个，峰值不低于锚点树表下界的 220 个，低于的 0 个 []
```

两条核对脚本的输出另存在 `research/results/e156-alloc-basis-counts-2026-10-18-support/check_tier_peaks_against_observations.txt` 与 `research/results/e156-alloc-basis-counts-2026-10-18-support/check_peaks_against_anchor_lower_bounds.txt`，各一行，与这两条命令打出的逐字相同。

**H1 的读数：读法甲（R7a）与读法乙（R7c）并排**（登记第六节；每个统计单位占一行，各报各的判定，不合取）。Q1a 在这些族里等于 Δ（`q1_trajectory` 里 `quantity=q1b_real`、`quantity=q1b_every` 的 `peak` 在 123 格里都是 0，`delta_real` 与 `q1a_real` 的 `peak` 逐格相同，`q1_cell_checks` 116 行的 `a13_real_nonzero`、`a13_every_nonzero` 逐行相加都是 0）；「实」列与「每」列在 Hh、Hc、Hs 的 116 条历史上峰值、为正点数、期末值逐格相同，只有 PC-同槽 对照格按设计不同；`peak` 与 `peak_after_first_ring`（环转过新池新建文件那一圈之后的峰值）在 123 格里都相同；`final`（期末值）在 123 格里都是 0（终点取最后一个洞 + 3S + 2，期末为 0 是历史的造法决定的，推的：环转过一圈之后门槛放开；不是「多扣不存在」）。

读法甲：族类 Hh-前，档 = 注入的洞数 k = 1 … `k_满(S)`（整个洞数都落在第一圈 txg < 3S 里）。ρ = 1 与 ρ = 1/4、实与每的峰值序列逐个相同：

| S | 档（k） | 每档 Q1a 峰值 | 相邻差 | W28 判定 | max(P)，是否落在最大档 |
|---|---|---|---|---|---|
| 4 | 1、2 | 10、20 | 10 | 档数不足（`fewer_than_three_tiers`），只报不判 | 20，是 |
| 8 | 1 … 4 | 10、20、28、36 | 10、8、8 | 仍在涨（`still_rising`） | 36，是（≤ 端点） |
| 16 | 1 … 8 | 10、20、28、36、44、52、60、68 | 10、8、8、8、8、8、8 | 仍在涨 | 68，是（≤ 端点） |

读法甲：`r7_verdict quantity=r7a` 12 个统计单位里判了 8 个（S = 8、16 × 两个 ρ × 两个时点），8 个都是「仍在涨」，0 个已平、0 个不单调；S = 4 的 4 个单位只有 2 档。

读法乙：族类 Hh-前 k = 1 … `k_满` + 1 再加 Hc 两档（注入洞数 n 取 `k_满` + 1 的两倍和三倍）。「实」「每」序列相同；ρ 两档分行：

| S | ρ | 档（注入洞数） | 每档 Q1a 峰值 | 相邻差 | W28 判定 | max(P)（族、txg），是否落在最大档 |
|---|---|---|---|---|---|---|
| 4 | 1 | 1、2、3、6、9 | 10、20、146、308、594 | 10、126、162、286 | 仍在涨 | 594（Hc，63），是 |
| 4 | 1/4 | 1、2、3、6、9 | 10、20、110、244、458 | 10、90、134、214 | 仍在涨 | 458（Hc，63），是 |
| 8 | 1 | 1 … 5、10、15 | 10、20、28、36、294、608、914 | 10、8、8、258、314、306 | 仍在涨 | 914（Hc，105），是 |
| 8 | 1/4 | 1 … 5、10、15 | 10、20、28、36、226、476、716 | 10、8、8、190、250、240 | 仍在涨 | 716（Hc，105），是 |
| 16 | 1 | 1 … 9、18、27 | 10、20、28、36、44、52、60、68、594、1208、1810 | 10、8、8、8、8、8、8、526、614、602 | 仍在涨 | 1810（Hc，201），是 |
| 16 | 1/4 | 1 … 9、18、27 | 10、20、28、36、44、52、60、68、458、944、1416 | 10、8、8、8、8、8、8、390、486、472 | 仍在涨 | 1416（Hc，201），是 |

读法乙：`r7_verdict quantity=r7c` 12 个统计单位全判，12 个都是「仍在涨」，0 个已平、0 个不单调，12 个取样点一致（只在这 12 个取样点上）。

R7e（上界数）：R7a、R7c 的每个统计单位 max(P) 都落在最大档上，所以写「≤ 端点」，没量到上界；第一次越过 2 槽与 1 槽的最小档都是 1（`first_tier_above_two_slots=1 first_tier_above_one_slot=1`，60 个统计单位全是），即注入一个洞已经多扣 10 槽。读法甲在第一圈里每多一个洞多扣 8 槽（相邻差 8，S = 8、16 上的第 3 档起），读法乙在第一个落进第二圈的洞处跳到 146 / 294 / 594（ρ = 1，S = 4 / 8 / 16）、此后每多 `k_满` + 1 个洞再多 134–614 槽（相邻差，随 S 与 ρ 变）。

F31（不单调）：R7a、R7c 的统计单位没有一个判「不单调」，不触发。F33（两种读法相反）：同一 (S, ρ, 时点) 上 R7a 与 R7c 没有一个是「已平」，不触发；S = 4 的 R7a 没有判定，不参与比较。

**照报的两行：R7b 触发 F32，R7d 不触发**（登记 R7b、R7d，判定不算判据）：

| 量 | 登记预推 | 读数 | 条款 |
|---|---|---|---|
| R7b Hh-后（k = 1 … `k_满后`，S = 8、16 判了 8 个单位） | 每档峰值相同（只钉回同一圈里最旧那一条根） | 8 个单位全判「不单调」：S = 16、ρ = 1 的峰值 716、710、702、694、686、678、672、666 随洞数逐档下降；S = 16、ρ = 1/4 的 474、478、470、472、466、468、462、468 上下摆；S = 8、ρ = 1 为 366、360、352、344。量级是几百槽（k = 1 时 S = 4 / 8 / 16 为 174 / 366 / 716），比 Hh-前同 k 的 10 槽大一个数量级 | F32 触发（`r7_verdict quantity=summary` 的 `report_only_units_unlike_the_prediction` 列了这 8 个单位）：登记第四节对这一族的推断（每档相同）错了，正文不许按推的写；登记要求交主 agent |
| R7d Hs-前、Hs-后（m = 1、2、3，24 个单位） | 同槽叠洞，那条旧根永远不被写掉，多扣随发布线性涨 | 24 个单位全判「仍在涨」，全落在最大档（Hs-前 S = 16、ρ = 1 为 10、758、1518；Hs-后 716、1460、2134） | 与登记预推相符，F32 不因这一行触发；F34（Hs-后 判「已平」与 D16（发布语义） 已定项 1 射程「一块盘持续写不进根时扣住的单元随发布线性涨、没有上界」对不上）不触发 |

R7b 与 Hh-前 的两个量放在一起看：Hh-后 在 k = 1 就已经是几百槽、随洞数不涨反略降，Hh-前 在第一圈里从 10 槽起每洞 +8，跨圈后是几百到上千槽并继续涨——「随洞数是否有上界」这个问法在不同的洞位置上的形状不同，这是两个观测，不是对读法甲、乙的裁决。

**判定整行抄自产物**（`research/results/e156-alloc-basis-counts-2026-10-18.out`）。`verdict stage=second` 一行（`:16656`）与 `r7_verdict` 六行（`:16559`–`:16564`）：

```
E7RESULT name=r7_verdict quantity=r7a units=12 judged=8 fewer_than_three_tiers_units=[s=4_rho=1_timing=real,s=4_rho=1_timing=every_publish,s=4_rho=1/4_timing=real,s=4_rho=1/4_timing=every_publish] still_rising=8 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_8_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:fewer_than_three_tiers,s=4_rho=1_timing=every_publish:fewer_than_three_tiers,s=4_rho=1/4_timing=real:fewer_than_three_tiers,s=4_rho=1/4_timing=every_publish:fewer_than_three_tiers,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7b units=12 judged=8 fewer_than_three_tiers_units=[s=4_rho=1_timing=real,s=4_rho=1_timing=every_publish,s=4_rho=1/4_timing=real,s=4_rho=1/4_timing=every_publish] still_rising=0 flat=0 not_monotonic=8 geometry_sensitivity=consistent_on_8_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:fewer_than_three_tiers,s=4_rho=1_timing=every_publish:fewer_than_three_tiers,s=4_rho=1/4_timing=real:fewer_than_three_tiers,s=4_rho=1/4_timing=every_publish:fewer_than_three_tiers,s=8_rho=1_timing=real:not_monotonic,s=8_rho=1_timing=every_publish:not_monotonic,s=8_rho=1/4_timing=real:not_monotonic,s=8_rho=1/4_timing=every_publish:not_monotonic,s=16_rho=1_timing=real:not_monotonic,s=16_rho=1_timing=every_publish:not_monotonic,s=16_rho=1/4_timing=real:not_monotonic,s=16_rho=1/4_timing=every_publish:not_monotonic
E7RESULT name=r7_verdict quantity=r7c units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7d_front units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7d_back units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=summary units=60 criterion_units_not_monotonic=[] report_only_units_unlike_the_prediction=[r7b:s=8_rho=1_timing=real,r7b:s=8_rho=1_timing=every_publish,r7b:s=8_rho=1/4_timing=real,r7b:s=8_rho=1/4_timing=every_publish,r7b:s=16_rho=1_timing=real,r7b:s=16_rho=1_timing=every_publish,r7b:s=16_rho=1/4_timing=real,r7b:s=16_rho=1/4_timing=every_publish] sampling_points_where_the_two_ring_readings_disagree=[] same_slot_back_units_flat_against_the_clause_range=[] dropping_the_last_tier_changed_units=0_of_24 threshold_moved_units_flipping=60_of_60 tier_sequence_control_cases_wrong=0 same_slot_controls_run=6 same_slot_controls_without_a_drop=0
E7RESULT name=verdict stage=second cells_run=116 cells_run_without_fitting_the_ring=36 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=0 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0 anchor_lag_mismatch_points=0 anchor_pinned_slot_mismatch_points=0
```

读法甲 R7a、「实」列六行（`:16499`、`:16501`、`:16503`、`:16505`、`:16507`、`:16509`；「每」列是各自的下一行，除 `timing=` 与 `maximum_peak_cell=` 里的时点外逐字相同，`grep 'name=r7_tier_peaks' <产物> | sed -E 's/ timing=(real|every_publish)//; s/_timing=(real|every_publish)//' | sort | uniq -c | awk '{print $1}' | sort | uniq -c` 的输出是 `     30 2`，即 60 行配成 30 对）：

```
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=4 rho=1 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[10,20] adjacent_differences=[10] growth=fewer_than_three_tiers maximum_peak=20 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=16 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=4 rho=1/4 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[10,20] adjacent_differences=[10] growth=fewer_than_three_tiers maximum_peak=20 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=16 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=8 rho=1 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[10,20,28,36] adjacent_differences=[10,8,8] growth=still_rising maximum_peak=36 maximum_peak_tier=4 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=28 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=8 rho=1/4 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[10,20,28,36] adjacent_differences=[10,8,8] growth=still_rising maximum_peak=36 maximum_peak_tier=4 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=28 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[10,20,28,36,44,52,60,68] adjacent_differences=[10,8,8,8,8,8,8] growth=still_rising maximum_peak=68 maximum_peak_tier=8 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=52 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[10,20,28,36,44,52,60,68] adjacent_differences=[10,8,8,8,8,8,8] growth=still_rising maximum_peak=68 maximum_peak_tier=8 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=52 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

读法乙 R7c、「实」列六行（`:16523`、`:16525`、`:16527`、`:16529`、`:16531`、`:16533`）：

```
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=4 rho=1 timing=real tiers=[1,2,3,6,9] families=[Hh,Hh,Hh,Hc,Hc] peaks=[10,20,146,308,594] adjacent_differences=[10,126,162,286] growth=still_rising maximum_peak=594 maximum_peak_tier=9 maximum_peak_family=Hc maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=63 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=4 rho=1/4 timing=real tiers=[1,2,3,6,9] families=[Hh,Hh,Hh,Hc,Hc] peaks=[10,20,110,244,458] adjacent_differences=[10,90,134,214] growth=still_rising maximum_peak=458 maximum_peak_tier=9 maximum_peak_family=Hc maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=63 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=8 rho=1 timing=real tiers=[1,2,3,4,5,10,15] families=[Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,294,608,914] adjacent_differences=[10,8,8,258,314,306] growth=still_rising maximum_peak=914 maximum_peak_tier=15 maximum_peak_family=Hc maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=8 rho=1/4 timing=real tiers=[1,2,3,4,5,10,15] families=[Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,226,476,716] adjacent_differences=[10,8,8,190,250,240] growth=still_rising maximum_peak=716 maximum_peak_tier=15 maximum_peak_family=Hc maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8,9,18,27] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,44,52,60,68,594,1208,1810] adjacent_differences=[10,8,8,8,8,8,8,526,614,602] growth=still_rising maximum_peak=1810 maximum_peak_tier=27 maximum_peak_family=Hc maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=201 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8,9,18,27] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,44,52,60,68,458,944,1416] adjacent_differences=[10,8,8,8,8,8,8,390,486,472] growth=still_rising maximum_peak=1416 maximum_peak_tier=27 maximum_peak_family=Hc maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=201 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

照报的 R7b、「实」列六行（`:16511`、`:16513`、`:16515`、`:16517`、`:16519`、`:16521`）：

```
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=4 rho=1 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[174,168] adjacent_differences=[-6] growth=fewer_than_three_tiers maximum_peak=174 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=27 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=4 rho=1/4 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[114,118] adjacent_differences=[4] growth=fewer_than_three_tiers maximum_peak=118 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=33 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=8 rho=1 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[366,360,352,344] adjacent_differences=[-6,-8,-8] growth=not_monotonic maximum_peak=366 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=51 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=8 rho=1/4 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[234,238,230,232] adjacent_differences=[4,-8,2] growth=not_monotonic maximum_peak=238 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=57 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[716,710,702,694,686,678,672,666] adjacent_differences=[-6,-8,-8,-8,-8,-6,-6] growth=not_monotonic maximum_peak=716 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=99 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[474,478,470,472,466,468,462,468] adjacent_differences=[4,-8,2,-6,2,-6,6] growth=not_monotonic maximum_peak=478 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

PC-同槽六行（`:15950`、`:16002`、`:16090`、`:16178`、`:16338`、`:16498`）与 PC-档序列五行（`:5243`–`:5247`）：

```
E7RESULT name=pc_same_slot s=4 rho=1 k=2 position=back unchanged_q1a_every_peak=344 control_q1a_every_peak=10 control_q1a_real_peak=344 seen=true
E7RESULT name=pc_same_slot s=4 rho=1/4 k=2 position=back unchanged_q1a_every_peak=236 control_q1a_every_peak=10 control_q1a_real_peak=236 seen=true
E7RESULT name=pc_same_slot s=8 rho=1 k=2 position=back unchanged_q1a_every_peak=712 control_q1a_every_peak=10 control_q1a_real_peak=712 seen=true
E7RESULT name=pc_same_slot s=8 rho=1/4 k=2 position=back unchanged_q1a_every_peak=476 control_q1a_every_peak=10 control_q1a_real_peak=476 seen=true
E7RESULT name=pc_same_slot s=16 rho=1 k=2 position=back unchanged_q1a_every_peak=1460 control_q1a_every_peak=10 control_q1a_real_peak=1460 seen=true
E7RESULT name=pc_same_slot s=16 rho=1/4 k=2 position=back unchanged_q1a_every_peak=960 control_q1a_every_peak=10 control_q1a_real_peak=960 seen=true
E7RESULT name=pc_tier_sequence case=rising_one_two_three actual=still_rising expected=still_rising holds=true
E7RESULT name=pc_tier_sequence case=flat_one_two_two actual=flat_upper_bound_2 expected=flat_upper_bound_2 holds=true
E7RESULT name=pc_tier_sequence case=not_monotonic_one_three_two actual=not_monotonic expected=not_monotonic holds=true
E7RESULT name=pc_tier_sequence case=flat_five_five_five actual=flat_upper_bound_5 expected=flat_upper_bound_5 holds=true
E7RESULT name=pc_tier_sequence case=two_tiers_are_refused actual=fewer_than_three_tiers expected=fewer_than_three_tiers holds=true
```

**判决行点名**（`grep -n 'name=verdict' <产物>` 共 14 条，`:5232` 与 `:16656`–`:16668`；命令与原样输出）：

```
$ grep -n 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | cut -d: -f1 | tr '\n' ' '
5232 16656 16657 16658 16659 16660 16661 16662 16663 16664 16665 16666 16667 16668 

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=(false|not_run)(?= |$)' | sort | uniq -c

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=[1-9][0-9]*(?= |$)' | sort | uniq -c
      1 cells_run=116
      1 cells_run_without_fitting_the_ring=36
      6 rho=1
      4 s=16
      4 s=4
      4 s=8

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=[a-z_]+(?= |$)' | grep -E 'void|not_|unseen|ambig' | sort | uniq -c
      1 fork_three=void_not_measured
      1 premise_two_answer=not_holds
      8 q1d_growth=not_monotonic
      1 u13b_flip2_not_applicable=true
```

没有取值为 `false` 或 `not_run` 的字段；违例、不匹配、歧义、失败类的整数计数都是 0（输出里的 `cells_run=116`、`cells_run_without_fitting_the_ring=36` 是格数，`s=`、`rho=` 是分组名，都不是违例计数）。拿不准、逐个点名的四处：`premise_two_answer=not_holds`、`fork_three=void_not_measured`（`:5232`，前提 2 不成立、岔路单第 3 行失效不量，是第 4 次起的既定结果，这一行与第 6 次产物 `e156-alloc-basis-counts-2026-10-05-r2.out` 的 `:1217` 逐字相同，`diff` 无输出）；`u13b_flip2_not_applicable=true`（同一行，名字含 `not_`，取值是 `true`，不是失败）；`q1d_growth=not_monotonic`（`:16657`–`:16668` 里 8 行，是 Q1d 随 h_缺 的判定值，S = 8 与 16 的 ρ = 1 为 `grows_with_holes`，其余为 `not_monotonic`，与第 6 次的 4 个「实」列取样点结论同形，不是违例）。不在 `name=verdict` 行里、但属于「没过」的一项：`r7_verdict quantity=summary`（`:16564`）的 `report_only_units_unlike_the_prediction` 非空，即 F32 对 R7b 的 8 个单位触发（照报表里 R7b 一行）。

**H1 够不够判**（登记「够判停机」：R7a 的 S = 8、16 取样点、R7c 的 12 个取样点都出了 W28 判定，R7e 出了上界数）：

| 读法 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|
| 读法甲 R7a（一圈里排满） | 已够判：S = 8、16 的 8 个单位全部「仍在涨」，每档多扣 10、20、28、36 …（S = 8）与 … 68（S = 16），上界数落在最大档、写「≤ 端点」；S = 4 只有 2 档（10、20），登记规定只报不判 | 不能：读法甲的档上限 `k_满(S)` 由几何定死（洞数再大就进了第二圈，那是读法乙），登记里的量都跑完了，没有「够判后未跑」 | `grep -n 'name=r7_tier_peaks quantity=r7a\|name=r7_verdict quantity=r7a' <产物>` |
| 读法乙 R7c（跨过一圈） | 已够判：12 个单位全部「仍在涨」，最大档（Hc 三倍 `k_满` + 1 个洞）处 914（S = 8、ρ = 1）、1810（S = 16、ρ = 1），上界数落在最大档、写「≤ 端点」，没量到上界 | 没量过：把 Hc 再造到四圈以上、或 S 取更大会不会出现「已平」，这在登记的射程之外（登记第十一节：洞数档到 Hc 三圈为止，「仍在涨」只说到最大档，不外推到「无界」） | `grep -n 'name=r7_tier_peaks quantity=r7c\|name=r7_verdict quantity=r7c' <产物>` |
| 两种读法并排 | 两种读法在 12 个取样点上结论相同（都「仍在涨」），F33 不触发；认哪一种为准登记 W26 留给主 agent，两种的行都已报 | 不会：它们同向 | `grep -n 'name=r7_verdict quantity=summary' <产物>`，看 `sampling_points_where_the_two_ring_readings_disagree=[]` |

这一节只写读数与够不够判，不判候选甲、乙、丙谁成立：登记第六节 R7a 行写的映射是「已平 往乙，仍在涨 往甲或丙，不单调 不支持任何一边」，这一次两种读法都落在「仍在涨」，「已平」（往乙）在这些取样点上没有读数；候选怎么选属于三方推论与用户定案。

**它答不了的（第 7 次）**：① 候选甲（可写挂载时补洞）做完之后的多扣：`crates/` 里没有根环槽位补洞的写路径（登记 5.1），这里只量今天的代码。② 造洞只有「记录已持久、根槽没持久」一种，读不出、自证不过的槽没造；洞数档到 Hc 三倍 `k_满` + 1 个洞为止，「仍在涨」只说到最大档，不外推到「无界」，也不说明增长的函数形状（相邻差 8 与 126–614 是读数，不是拟合）。③ 只在 4 GiB × 2、S ∈ {4, 8, 16}、ρ ∈ {1, 1/4}：同一读法下上界数随 S 与 ρ 变（Hc 最大档 S = 16 为 1810 / 1416），不是常数。④ 每条历史的终点是最后一个洞 + 3S + 2，`final` 全是 0 由这个造法决定；多扣持续多久（为正的观测点数）只在 `q1_trajectory` 里报了，没进判据。⑤ R7b 的预推错在哪（每档峰值为什么随洞数略降或摆动）没查，只记了 F32 触发。⑥ 内联单测 25 条的读数是主 agent 给的，装置的 5 条新变异 M50–M54 还没跑（待轻型），`replay.sh E156`（alloc-basis 四条岔路的代价数） 的整份复跑也没跑；产物只有这一趟，没有同一个二进制再跑一遍的逐字节比对。⑦ 读法甲、乙谁为准没判。⑧ 登记 5.5 的停机判据（S = 16 的 Hc(27) 单条挂钟超过 10 分钟）没有单独量，整趟 241 秒，按推的不会触发。

**装置下一次要改的**（这一次没改装置）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 的 `verdict` 函数里，给定状态表的判段交回占位结论时 panic，消息（`:758`）只点名 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 与 `SINGLEFS_CRASH_AMPLIFICATION_SHARE` 两个变量；这一次真撞上的成因是装置二进制没带 `gpu` 特性编、而多机配置 `ENABLE_GPU=1`（主 agent 的诊断，那一趟输出没留存），消息里没有这一种，起跑的人会白查两个变量。下一次要把这一种成因写进那条消息（并点名补救：带 `--features verdict-store,gpu` 编、经 `scripts/crash/gpu-quiesce-run.sh` 起）。

### 第 6 次重跑（2026-10-05）：回收门槛照定乙读之后 38 格 K10、F24 都是 0，去掉 core 的全零钉住正好红在推出来的 34 点

重跑登记 `e156-r6-prereg.md`（差分形态，以第 5 次登记为底，含末尾「主 agent 认定：第 6 次登记交回」六条），问题单 `e156-r6-questions.md` 三行：第 1 行（装置的回收门槛照定乙读之后，今天的 `crates/` 上 38 格的 K10 与 F24 还有没有不符的点，另在副本里去掉 core 的全零钉住证这一判分得出）、第 2 行（岔路单第 1 行的读数在新门槛下各是什么、与第 5 次产物比哪几格变了，只报不重下判决）、第 3 行（PC-钉门槛 六格两列还都判「不符」、都含 6S − 1 吗）。起因是第 5 次的 K10 34 点不符：core 照用户定乙，把挂载那一刻读回全零的根环槽钉在它可能住过的最旧 txg（`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant::AllZeroAtMount`），装置与锚点模型照的是旧读法。

**起跑前提与开跑前的核**（登记 S15、S2、S3）：

| 项 | 命令与原样输出 | 结论 |
|---|---|---|
| 第 31 批代码轮判决写完 | `ls research/prompts/m3-formal-code-b31-r1-main-verification.md` 在；派发提示写明判决已写完 | 成立 |
| 装置没改过 | `sha256sum crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `e69e423a6fda2d973f1b7d4b741bbb280e54d70723d953fda666bba00e2bfe20`，`wc -l` → `5819` | 成立 |
| core 里有全零钉住 | `grep -c AllZeroAtMount crates/singlefs-core/src/allocator.rs` → `11`；`grep -c '            &ring_slots_reading_back_all_zero,$' crates/singlefs-core/src/mount.rs` → `1` | 成立 |
| 条款没变（S2） | 登记命令一出口 `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`、与第 5 次第二节 `diff` 0 行；命令二出口：定乙四段 `a4ac82034664e49c452d9de59ebe7b8f510f01fe15e8dafc2f6ab6764b3805f7`、问题单 `7889590e8aed051fc02b3edd4ae4686ca0e6d4ba1dd85c9ee191d24ee27a1a8c` | 不触发 |
| 锚点脚本（命令三、四、五） | 汇总 `700ec0b0…`（第 4 次与第 6 次脚本同）、`--dump` 第 4 次 `7bb1e088…` 第 6 次 `47a7d193…`（各 1092 行，`diff` 23 行、`bab6a323…`）、`--dump --pins` `8a222fe4…`、`count_pin_points.py` 输出 `9d5cf9ce…`、`lag_bound_r5.py` 在两份 `--dump` 上都是 `b980971a…` | 与登记第十三节相同 |
| 实现快照（S3） | 编产物二进制之前、主产物跑完之后：`git diff -- crates/ \| sha256sum` 都是 `ad4c31366a677a2df4f08c069d77fc7ebc04345921bcd83371b0f32076ba6ae0`，`git status --short crates/ \| sha256sum` 都是 `8995a6f9b7c12c51e377589e09dc69beb5bb6ebe758392ace82f9825cd129a28`（152 行）；PC-全零钉 跑完之后再取一次，`git diff` 只多了 `crates/mutations.tsv` 末尾追加的三行变异 | 不触发 |

**装置怎么改的**（登记第五节 5.3）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 5819 行 → 6205 行，19 条单测（`grep -c '#\[test\]'`，第 5 次 18）。挂载点在交给 `mount_writable` 之前读切段镜像的 3S 个槽，分自证过、读回全零、读坏三类（`read_root_ring_slot_content`）；挂载交回之后按新实例第一个 txg 给每个全零槽取「落到它、小于第一个 txg 的最小 txg」，整张换进 `HhRunState` 的钉住表，打一行 `name=all_zero_pins_at_mount`。`read_hh_state` 在算门槛之前对钉住表里每一槽读这一刻那一槽的根：根的 txg ≥ 第一个 txg 算这个进程写掉了、< 第一个 txg 算重读读出（另计 `released_by_reread_only`），都不是的照旧钉着、并入门槛；`reclaim_threshold_read_from_the_ring_image` 的函数体与 `let every_publish_floor = ring_threshold;` 一个字没动。锚点模型照登记命令四的补丁改同一处。每格在 `name=q1_cell_checks` 之后打 `name=all_zero_pin_cell`，第二段 `name=failure_clauses` 之后打 `name=all_zero_pin_summary`；已有的行一行都没改格式。新单测 U20 与变异 M45–M47（`crates/mutations.tsv` 里带「M45（第 6 次重跑登记第九节）」「M46（…）」「M47（…）」的三行）。只对这一份文件跑了 `rustfmt --edition 2021`，`rustfmt --check` 无输出（第 4999 行起的 `use` 块随之改好）。`compile-then-swap.py` 在仓副本里 `cargo build` 与 `cargo test --no-run` 都编过才换进主工作区（换上之后 sha256 `3129ec95e2f129ef0a49f1be15a998aae5c4abb508615a8663cdece91d1ef279`）。

**产物**：主产物 `e156-alloc-basis-counts-2026-10-05-r2.out`（4063 行，sha256 `1c8812b19456165b10b6cd01f3e00fb2ddd1e3673f700cedac151f5da778176b`，完成标记 `E7RESULT name=done emitted=4063`），用仓根 `target/release/` 的二进制（sha256 `1227561466a0008a1b1a7004bafe73d1d71cff7b1af1a51eaab19c40416f2a48`）跑，同一个二进制再跑一遍逐字节相同；`replay.sh` 的驱动用 debug 二进制，debug 一遍约 13 分钟，执行员前台跑不完，只跑了前 225 秒，那 1803 行与 release 产物的前 1803 行逐字节相同，整份 debug 复跑交 `replay.sh` 的 exact 比对。判别力对照 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out`（4063 行，sha256 `eaf6b8b63d3b1d899377559a5f09fb32f6774cd14031f41567898c62873679ff`），副本里对 core 的改动 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.patch`（`mount.rs` 一行，挂载时一个全零槽都不装进根环表，与调查报告 x1 补丁的块体逐行相同、只差行号），同一份装置源码在副本里另编 release。

**第 1 行：照定乙读门槛之后 38 格 K10、F24 都是 0**（登记 R1a、R1b）。主产物 `:4050`、`:4048`、`:4049`：

```
E7RESULT name=verdict stage=second cells_run=38 cells_not_fitting=4 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=0 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0
E7RESULT name=all_zero_pin_summary cells=38 cells_with_pins=16 pins_installed=64 threshold_lowered_points=34 released_by_reread_only=0 bad_slots_at_mount=0 bad_slots_still_pinned=0
```

逐格：`grep ' name=q1_cell_checks ' e156-alloc-basis-counts-2026-10-05-r2.out | grep -o 'k10_violation_points=[0-9]*' | sort | uniq -c` → `38 k10_violation_points=0`；同一命令换 `real_every_differ_points` → `38 real_every_differ_points=0`；`failure_clauses` 行（`:4048`）的 `f24_real_every_differ_points=0`。38 格是 S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} 的 k = 0 六格、洞位置为前与为后各 16 格。按登记第八节 8.2 的写法：K10、F24 在这 38 个取样点上一致为 0，只在这 38 个取样点上。

**判别力（登记 R1c，PC-全零钉）**：同一份装置配去掉全零钉住的 core，判别力对照 `:4050` 的 `s11_k10_violation_points=34`、`:4048` 的 `f24_real_every_differ_points=34`（整行在「判决行点名」）。不为 0 的观测点集合用 `e156-alloc-basis-counts-2026-10-05-r6-point-sets.py`（只读两份产物与锚点脚本的两份输出）与登记 L6 × 两个 ρ 比，输出 `e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt` 里 `L6_points_x_rho=` 与 `pc:` 开头的两行：

```
L6_points_x_rho=34
pc: k10_points=34 f24_points=34 k10_eq_L6=True f24_eq_L6=True k10_minus_L6=[] L6_minus_k10=[] f24_minus_L6=[] L6_minus_f24=[]
```

即 K10、F24 都红在前洞格 txg h + 3S − 1 那一个观测点上（S = 4 的 16、21，S = 8 的 28、33、39、45，S = 16 的 52、57、63、69，按 k 取前 k 个），后洞格与 k = 0 的格一点不红；作废 V19 与停机 S16 都没触发。两个输入文件是登记命令四 `--dump --pins` 与命令五的原样输出：`e156-alloc-basis-counts-2026-10-05-r6-anchor-pins.txt`（`8a222fe4…`）、`e156-alloc-basis-counts-2026-10-05-r6-anchor-lowered.txt`（`9d5cf9ce…`）。

**钉住簿记（登记 R1d）**：同一份输出第 4–7 行：

```
R1d_1: hh_mount_rows=68 checked=68 mismatches=[]
R1d_2: cells=38 mismatches=[]
R1d_3: cells_with_reread_release=[]
R1d_4: cells_with_bad_slots=[]
```

68 次挂载（含第一段那两段历史）每次钉的 txg 集合与锚点脚本那一格挂载做完那一行的 `pinned` 相同，后洞格与 k = 0 一槽不钉；每格 `threshold_lowered_points` 等于 L6 那一格的点数；`released_by_reread_only` 全是 0、`bad_slots_at_mount` 与 `bad_slots_still_pinned` 全是 0，停机 S17、S18 不触发。锚点逐行相同（登记第七节 7.2 的最后一条断言）：主产物 `name=anchor_dump` 1092 行去掉前缀之后与命令四 `--dump` 的 `DUMP` 行 `cmp` 无输出（作废 V20 不触发）。

**第 2 行：岔路单第 1 行的读数与第 5 次比**（登记 R2a–R2c，只报、不重下岔路单第 1 行的判决）。比较脚本 `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.py`，输出 `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.txt`（146 行）第 1 行：

```
pairs=102 unequal_pairs=42 only_old=0 only_new=0 differing_fields_real=0 differing_fields_every=122 differing_fields_unsplit=3 judgement_flips=8 other_judgement_changes=12
```

| 读数 | 第 5 次 | 第 6 次 |
|---|---|---|
| F1 | 12 组全在（触发） | 12 组全在（触发），不翻面，F33 不触发 |
| Q1c-前 判定（实、每两列 12 组的 `judgement_two_slots`、`judgement_one_slot`） | 全 `above`，越过 2 槽与 1 槽的最小 h_缺 都是 1，第一次越过在 txg 7 | 同左，不翻面 |
| Q1c-前「每」列的 Δ 峰值（`by_h_que`，S = 4、ρ = 1，h_缺 = 1） | `peak10` | `peak20`，与「实」列相同（「每」列 6 组的 `by_h_que` 都变了） |
| Q1d（`q1d` 的 `growth_by_h_que`）「实」列 | s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 `not_monotonic`，其余 `grows_with_holes` | 同左 |
| Q1d「每」列 | 12 组全 `grows_with_holes` | 与「实」列相同：那四组翻成 `not_monotonic`（F34 触发，照报） |

翻面的 8 行是比较输出第 2–9 行（`q1d` 与 `verdict stage=second_group` 的 `q1d_growth` 各 4 组，都在「每」列）。判定字段变了而不属于 Q1c-前、Q1d 的 12 处在第 10–21 行：`q1c quantity=q1c_not_on_holes` 的「每」列 6 组两个判定都由 `at_most` 变 `above`。R2c：归「实」列的不同处 0 个（`differing_fields_real=0`），「实」列与第 5 次逐字段相同，F35 不触发。第 6 次产物里每组的 `q1c`、`q1d`、`q1c_front_threshold_moved` 行去掉 `timing=` 之后实、每两行逐字相同（`sed 's/ timing=[a-z_]*//' | sort | uniq -c` 每组都是 2）。

**第 3 行：PC-钉门槛**（登记 R3）。主产物 `name=pc_pinned_floor` 六行与汇总行（`:3908`–`:3914`）与第 5 次产物同名七行 `cmp` 逐字节相同：六格实、每两列 `judge_*_mismatch=1`、`contains_bound_*=true`，汇总 `E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6`，`v2_pc_pinned_floor_seen=true`。

**判决行点名**（`grep -n 'name=verdict' <产物>`）：主产物 14 条（`:1217`、`:4050`–`:4062`），没有取值为 `false` 或 `not_run` 的字段，违例、不匹配、失败类的整数计数都是 0；`premise_two_answer=not_holds`、`fork_three=void_not_measured` 与第 5 次相同（前提 2 不成立、岔路单第 3 行失效）。判别力对照 14 条（行号同），不为 0 的两项：`s11_k10_violation_points=34`（登记 R1c 要的那一项）与 `s11_a15_real_below_points=16`（`:4050`；16 个前洞格各 1 点，「实」列 Q1a 低于锚点模型照定乙读算的树表下界，推的：core 不钉时在 txg h + 3S − 1 提前回收了一个树表单元；登记没写这一项的预期，对照不是被测状态）。完整行：

```
产物行见 git show 18b30adbfe94:e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out 第 4050 行（name=verdict）
```

**它答不了的（第 6 次）**：① 内联单测 19 条（含新 U20）与 `crates/mutations.tsv` 里点着这个装置的 26 行变异（含 M45–M47）都没跑（checker 档，归提交时），只核了 26 行原文在新源码里各命中 1 次、M45–M47 三条同时施加在副本里 `cargo test --no-run` 编得过；② 这一族只造第一圈里的前洞，那一槽在 [0, 第一个 txg) 里只住得下一个 txg，「钉在最旧 txg」与「钉在能住的最大 txg」同值，分不出 core 钉的是哪一个；③ 「重读读出根才解开」那条路在这一族上一次没出现（`released_by_reread_only` 全 0），装置只计数、不证它；④ 读坏的槽、准入抬 F 的上限这一族不造；⑤ PC-全零钉 证 core 少钉全零槽时装置看得见，不证装置旁路自己读没读对定乙（那一半由锚点逐行相同与 U20 盯）；⑥ 主产物用 release 二进制跑，debug 那一份只比了前 1803 行；⑦ 「实」列与第 5 次相同，第 5 次之后 `crates/` 别的改动碰没碰这一族之外的量没查；⑧ 两个对照（PC-钉门槛、PC-全零钉）各自分得出它们要抓的那一类，合起来分不出三样：core 钉的是最旧 txg 还是最大 txg、重读解开、读坏的槽（第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P5「两个对照分不分得出」一行，站住）；⑨ 变异 M36（`crates/mutations.tsv` 里带「M36（第 4 次重跑登记第九节）」的那一行：门槛改成「最新有效根 − 环里根数 + 1」、漏看洞槽里的旧根）在前洞格上可能少一块判别力（同一格表里「M36、M42」一行，攻方推的，没量）；M36 与 M42 无互相掩护（读码）。

### 第 5 次重跑（2026-10-05）：W7 那一格的对照换成 PC-钉门槛，第 4 次读数在新 `crates/` 上 294 行不同

重跑登记 `e156-r5-prereg.md`（差分形态，其余各节同第 4 次登记 `e156-r4-prereg.md`），问题单 `e156-r5-questions.md` 两行：第 1 行（core 在分配器没装根环表而记了一条根时断言之后，装置里 W7 那一格的阳性对照 PC-分配器 造不出来，换成什么）、第 2 行（第 4 次已够判的读数在新 `crates/` 上还相不相同）。起因是用户定的 C551（`build_pool` 的分配器没有根环表）：`crates/singlefs-core/src/allocator.rs` 的 `record_root_written_by_this_process` 在没装根环表时 `expect`，消息以「分配器没装根环表却记了一条根」开头。

**起跑前提与开跑前的核**（登记第 5 次的停机 S13、S2、S3）：

| 项 | 命令与原样输出 | 结论 |
|---|---|---|
| 31-1 的断言在主工作区 | `grep -c '分配器没装根环表却记了一条根' crates/singlefs-core/src/allocator.rs` → `2`（`allocator.rs` 里一处是断言消息、一处是 `should_panic`） | 成立 |
| 第 31 批合入验证绿的目录 | 派发提示写明 `/tmp/claude-1000/merge-and-verify/2026-10-05-2822486/` | 成立（目录里的日志执行员没重读） |
| 装置只多 31-3 的改动 | `git show :crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs \| sha256sum` → `f1930d4e…`（登记写时的样子）；`git diff -- 同一文件` 两块，与 `/tmp/claude-1000/m3-formal-31-3/patch/crates.patch` 前两块逐字相同：一块是 `mod tests` 的 `use` 里加 `root_ring_occupancy_after_make_filesystem`（函数体之外），一块是 `new_pool_file_creation_state` 函数体里装根环表 | 登记写的是「只在函数体里」；多出来的那一行 `use` 是同一块补丁带的，执行员认为不改读数、照样开工，写进交回由主 agent 认 |
| 条款没变（S2） | 登记第十三节命令一原样重取五段，`sha256sum` → `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`，与登记第二节同一个 | 不触发 |
| 锚点脚本（命令三、命令四） | `anchors_e156_r4.py` 的输出 `700ec0b0…`、`--dump` `7bb1e088…`（1092 行）、`lag_bound_r5.py` 的输出 `b980971a…`，与登记第十三节同一个 | 相同 |
| 实现快照（S3） | 产物前、第 1 次产物后、第 2 次复跑后各取 `git diff -- crates/` 与 `git status --short crates/` 的 `sha256sum`：`31db3aba132749d5fd643e4ef8e013af15896679f953b73dd4bb8265caccf58d`、`8c727d9e15719e32467975a09ff64b3b445ff1971be7ebf51ef49e7b326a34a0`，三次相同 | 不触发 |

**装置怎么改的**（登记 5.5 的 e1–e4）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 5783 行 → 5819 行。删 `HhAllocatorSource::WithoutRootRingPositiveControl`、`allocator_without_the_root_ring` 与 `main` 里 PC-分配器 那一块；`HhAllocatorSource` 与成员 `ProductPathWithRootRing` 名字不改（`name=q1_cell` 行打 `allocator={:?}`）；加 PC-钉门槛（格表之后、PC-多扣 之前，对格表里 `hole_count == 1` 且位置为后的 6 格用同一个 `judge_hole_history_outcome`，期望取同一 S 的 `anchor_model::tree_table_reclaim_lags`，6S − 1 现算为 `2 * E156_ROOT_RING_REGIONS * slots_per_region - 1`）；`name=verdict stage=second` 里 `v2_pc_allocator_seen` 换名 `v2_pc_pinned_floor_seen`；新单测 U19。`grep -n 'PoolAllocator::new(' crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `5041:        let mut allocator = PoolAllocator::new(`（`new_pool_file_creation_state` 里，第 5071 行 `install_root_ring_occupancy` 紧跟着装根环表）。单测数 `grep -c '#\[test\]' 同一文件` → `18`（第 4 次 17）。`compile-then-swap.py` 在草稿副本里 `cargo build` 与 `cargo test --no-run` 都编过才换进主工作区。

**第 1 行：新对照 PC-钉门槛**。产物 `e156-alloc-basis-counts-2026-10-05.out`（3955 行，sha256 `a3d29fe0e31a570aca19db52a6fda9f7600017f43dbcfa856996d9126085931c`；同一个二进制再跑一遍与它逐字节相同，`cmp` 无输出）：

```
E7RESULT name=pc_pinned_floor s=4 rho=1 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=4 rho=1/4 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1/4 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1/4 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6
```

六格两列逐个报：Q1r5a（判定判「不符」）实、每两列都是 1；Q1r5b（滞后集合含 6S − 1）实、每两列都是 true；`v2_pc_pinned_floor_seen=true`（`e156-alloc-basis-counts-2026-10-05.out` 里 `name=verdict stage=second cells_run=` 那一行）。登记的够判条件满足（作废 V2 没触发、F28、F29 没触发）：新对照在 6 个取样点上证出了「A10 分得出真实分配器把回收推迟一圈」。它只证推迟，不证全程不回收（那一形由变异 M35 改写后的行盯，提交时跑），也不证 W7 那一格的核认得出没装根环表的分配器（那一件由 core 的单测接管）。滞后集合是 3S 到 6S 的整段（S = 4 时 12 到 24），不是只有 6S − 1 一个值；6S − 1 与 6S 是登记命令四算出的释放代 6、5 的滞后（`lag_bound_r5.py` 的输出 `release_first_free_lag=[(5, 28, 24), (6, 28, 23)]`），整段里别的值产物没按释放代拆开，没核（命令 `grep -c '^E7RESULT name=pc_pinned_floor ' e156-alloc-basis-counts-2026-10-05.out` → `6`，汇总行 `grep -c '^E7RESULT name=pc_pinned_floor_summary '` → `1`）。

**第 2 行：第 4 次读数在新 `crates/` 上逐字段重比**（登记 Q2a、Q2b）。规范化：第 4 次产物 `e156-alloc-basis-counts-2026-09-26-r2.out`（4007 行）去掉带 `family=PC-allocator` 项的行、第一项是 `name=pc_allocator` 的行、`name=verdict stage=second` 行里的 `v2_pc_allocator_seen=` 一项；新产物去掉 `name=pc_pinned_floor`、`name=pc_pinned_floor_summary` 行与同一 verdict 行里的 `v2_pc_pinned_floor_seen=` 一项；「第一项」按行首 `E7RESULT ` 之后的第一个空白分隔项算。比对脚本与全部输出都在仓里：

```
python3 e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.py e156-alloc-basis-counts-2026-09-26-r2.out e156-alloc-basis-counts-2026-10-05.out > e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
head -3 e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
old_dropped_rows={'family=PC-allocator': 58, 'name=pc_allocator': 1} old_dropped_items=1
new_dropped_rows={'name=pc_pinned_floor': 6, 'name=pc_pinned_floor_summary': 1} new_dropped_items=1
lines_old=3948 lines_new=3948 equal=3654 unequal=294
```

Q2a 判「不同」（登记写死的判据：`unequal = 0` 才算相同）。登记 L3 那一句核过：新产物去掉 7 行 1 项、第 4 次产物去掉 59 行 1 项，去掉的每一行不是 `pc_allocator` 就带 `family=PC-allocator`，V18 没触发。294 行里有 1 行是 `name=done emitted=`（4007 → 3955，恰好是 4007 − 59 + 7；登记的规范化没有把这个计数项摘掉，按字面算进了 294）。

Q2b 按登记的两类分（判决行名单取登记第六节）：

| 类 | 行数 | 键数 | 行名（行数） |
|---|---|---|---|
| 判决行 | 60 | 160 | `q1c`（24）、`q1_cell_checks`（16）、`q1d`（6）、`q1c_front_threshold_moved`（6）、`verdict`（5）、`u13a_rollback_to_oldest_candidate`（1）、`geometry_sensitivity`（1）、`failure_clauses`（1） |
| 计数行 | 234 | 522 | `q1_observation`（82）、`hole_recovery`（69）、`q1_trajectory`（48）、`q1_cell`（18）、`q1_trajectory_extra`（16）、`done`（1） |

不同的行落在哪几格：`name=q1_cell` 的 Hh 行里洞位置为前的 16 格全部不同（k = 1、2、4 各 6、6、4 格），位置为后的只有 k = 2、k = 4 各 1 格，k = 0 的 6 格一格不同；`hole_recovery` 69 行只有一个键不同（`steps_after_root_slot_write` 在 `system_configuration_slot,system_configuration_slot` 之后多了 `,barrier,barrier`）；其余键见 `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`（979 行，每个不同的行逐键列旧值、新值）。判决层面的几处：

| 字段 | 第 4 次 | 新产物 |
|---|---|---|
| `failure_clauses` 的 `f24_real_every_differ_points` | 0 | 34 |
| `verdict stage=second` 的 `s11_k10_violation_points` | 0 | 34 |
| `geometry_sensitivity quantity=q1d_growth` 的 `result` | `consistent_on_12_sampling_points` | `unstable`（`e156-alloc-basis-counts-2026-10-05.out` 里 `name=geometry_sensitivity quantity=q1d_growth` 那一行，12 个取样点里「实」列的 s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 四点 `not_monotonic`，其余 8 点 `grows_with_holes`） |
| `verdict stage=second_group` 的 `q1d_growth` | 12 组全是 `grows_with_holes` | 4 组 `not_monotonic`（与 `geometry_sensitivity quantity=q1d_growth` 那一格的四点相同），8 组不变 |
| `q1c_front_threshold_moved`（S = 4、ρ = 1、实 那一组，新产物 `e156-alloc-basis-counts-2026-10-05.out`、第 4 次 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 各自的 `name=q1c_front_threshold_moved s=4 rho=1 timing=real` 那一行；「实」列 6 组都不同） | `threshold_between=10` `threshold_at_largest=18` | `threshold_between=18` `threshold_at_largest=20` |
| `u13a_rollback_to_oldest_candidate` 的 `item1_slots` | 374 | 377 |

没变的判决：`f1_front_above_two_groups` 仍是 12 组全在（`e156-alloc-basis-counts-2026-10-05.out` 里 `name=failure_clauses` 那一行），即 F1 在新 `crates/` 上照样触发；`name=verdict stage=first` 行（`:1214`）、`v2_a10_every_mismatch_cells`、`v2_a12_every_nonzero_points`、`v2_a14_mismatch_points`、`v2_a15_every_below_points`、`s1h_root_ring_missing_points` 都与第 4 次相同（`:3942`）。`name=verdict stage=second` 整行（`:3942`）：

```
产物行见 git show 18b30adbfe94:e156-alloc-basis-counts-2026-10-05.out 第 3942 行（name=verdict）
```

**Q2c 归属**：在仓副本里只把 31-1 对 `allocator.rs` 的断言块反着打回去（`record_root_written_by_this_process` 回到 `let Some(occupancy) = self.root_ring.as_mut() else { return; };`），用同一份装置、副本自己的编译目录跑一次，产物 `e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out`（3955 行），`cmp` 与 `e156-alloc-basis-counts-2026-10-05.out` 逐字节相同（两份 sha256 都是 `a3d29fe0…`）。不同不是 31-1 带来的，来自第 4 次产物之后别的批改过的 `crates/`（登记 W14）；登记第四节 P7 的预估（Q2a 相同）错了，F30 的触发条件（Q2c 判出不同来自 31-1）没满足。哪一批带来的没查（登记没立这个量）。

按登记，Q2a「不同」触发 S14 与 F26，K10 在「实」上 34 点不符触发 S11，三条都是交主 agent 的停机，不当结果、不改判据。读码推的一个线索（没跑过、没验）：`crates/singlefs-core/src/allocator.rs` 在第 5 次重跑时把挂载时读回全零的槽记成 `RootRingOccupant::AllZeroAtMount` 并把回收门槛钉在它可能住过的最旧 txg（`allocator.rs` 里 `TxgsARingSlotBadAtMountCanHold` 的文档注释，用户定乙），而装置 `read_hh_state` 里没有 `AllZero` 的处理（`grep -n 'AllZero' crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 无输出）；洞位置为前的格，崩溃那一次的根槽按推算是环第一圈里还没写过的槽（推的，没读产物核），与不同的格大体重合。这一条要对着 `crates/` 与装置两边查，不是这一份产物的结论。

**判决行点名**（`grep -n 'name=verdict' e156-alloc-basis-counts-2026-10-05.out`）：14 条，行号 `1214`、`3942`–`3954`。没有取值为 `false` 或 `not_run` 的字段；整数计数里不为 0 的，名字表示违例的只有一个：`s11_k10_violation_points=34`（`:3942`，K10 在「实」列 34 个观测点不符，34 点全在洞位置为前的 16 格里；第 4 次是 0，不在登记预期内，触发停机 S11、S14）。`cells_run=38`、`cells_not_fitting=4` 是格数，不是违例。判决行之外同一类字段：`name=failure_clauses`（`:3941`）的 `f24_real_every_differ_points=34`（第 4 次 0）与 `f2_no_observable_difference=false`（取值 false 表示有可观测的差，F2 没触发，与第 4 次相同）。

**第 5 次重跑与页内第 4 次各节的关系**：「这一段做了什么」到「它答不了的」各节写的是 2026-09-26 第 4 次重跑，引的读数出自 `e156-alloc-basis-counts-2026-09-26-r2.out`，没有随新产物改写。新产物在 Q2a 的规范化之后有 294 行不同，F1 仍触发而 Q1d 随 h_缺 增长的那一句在「实」列 4 个取样点上不再成立（Q2b 判决层面那一组字段）；哪一份承重主 agent 已在认定为第 5 次产物（`e156-r5-prereg.md` 末尾「主 agent 认定：第 5 次交回与 S11 两边查」第 2 条）；第 6 次重跑照定乙读门槛之后，`f24_real_every_differ_points`、`s11_k10_violation_points` 在第 6 次产物上又是 0（见「第 6 次重跑」一节），但这几节里「Q1d 都随 h_缺 增长」在第 5、6 次产物的「实」列 4 个取样点上仍不成立，不能原样再引。

**它答不了的（第 5 次）**：① 内联单测与变异没跑（执行员不跑 checker 档）：新单测 U19 与整份装置的 18 条单测、M35 改写与 M44 的实际红绿都没有读数，只验了变异两行的原文在源码里各命中 1 次、替换文在副本里编得过；② PC-钉门槛 证推迟一圈，不证全程不回收；③ W7 那一格认得出没装根环表的分配器这一件，装置里不再有对照，靠 core 的单测 `recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming` 与变异「C551（`build_pool` 的分配器没有根环表）：分配器没装根环表记根时静默返回（不断言）」；④ 294 行不同来自哪一批 `crates/` 的改动没查；⑤ `s11_k10_violation_points=34` 是装置读错了还是实现的门槛与条款不同，没两边查完（只有 Q2c 之后那一条读码的线索）。

**要主 agent 在提交时跑的命令**（执行员不跑，重型）：

```
SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 10 cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts
```

看 `test result:` 行是不是 `19 passed`（第 6 次，含新 U20；第 5 次交回时是 `18 passed`（Q2d）），U19 的名字是 `hh_back_hole_at_four_slots_per_region_delays_the_tree_table_reclaim_to_six_ring_slots_minus_one_and_the_a10_judgement_sees_it`；变异：旧 M35（替换文 `allocator_without_the_root_ring` 在装置里已不存在，编不过）已换成改写后的 M35、追加了 M44，都已打进 `crates/mutations.tsv`（`grep -n 'e156_allocation_basis_counts' crates/mutations.tsv` 现查；两行的原样内容在 `e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv` 与 `e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`），由门禁 checker-tier-crates-mutation-replay 跑，三个数分开报。

### 这一段做了什么

2026-09-26 第 4 次重跑，第一段与第二段在一次装置运行里：第一段的停机条款任一触发就只出第一段，都没触发才跑第二段（登记第十二节修订 2 第 11 条）。

1. **开跑前的核**：被测条款五段用登记第十三节命令一重取两次（开跑前、产物之前），与登记第二节逐字节相同，sha256 `af95e554dd52e87acc7095c129194171cac2e122ca1832d36be10882022e5de4`，停机 S2 不触发。命令四锚点脚本原样重跑，输出 sha256 `700ec0b0…`、`--dump` sha256 `7bb1e088…`，与登记相同。实现快照（停机 S3）产物前后各取一次：`git diff -- crates/` 的 sha256 都是 `ecf57f9db7d0c6e79b833cfaab481f68c12b54e3a5b161c13a626d9770a2d6fa`，`git status --short crates/` 的 sha256 都是 `947a8fd6955f503316ba7d663dd22fa40cd9ce2d316a940df62c66bae39a97bc`。
2. **装置整个改写**：挂载时回退的本地替身、HK / HK-F0 / HR / H0 / HF / HX / HY 与挂载时回退那一形的隔离场景全部删掉，岔路 3、7 的量不再跑。新写的：岔路 1 的族（分配器走产品路径 `allocator_after_make_filesystem`，装着根环表；「记录已持久、根槽没持久」的崩溃按录下来的设备写切在根槽那一次写之前造；每个观测点从镜像的根环现读门槛、h_缺、h_洞，两条臂 × 两个回收时点各算一次多扣）；锚点模型（命令四 `anchors_e156_r4.py` 的逐行移植，产物里 `name=anchor_dump` 那 1092 行去掉前缀之后与 Python 的 `--dump` 逐字节相同，命令在「结果」）；前提 2 的录写（抬 F 那一串的设备写，切在系统配置写完之后、每条推空根之后读 F_生效）；两个回退单测的新形态；判定函数（门槛 2 槽、1 槽照登记）。
3. **第一段撞上一次停机 S1(i)，按登记 S1 行「查装置读错了实现（改装置，写进第十二节）」处置**：录写切段镜像与装置原有的「发完只改回根槽」镜像每次都差每块盘一个系统配置扇区——实现在根槽 FUA 之后还轮换每块盘的系统配置槽（`crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`），原有造法的前提「根槽是那次发布最后一笔写」不成立。对照镜像改成「发完把根槽与每块盘的系统配置两槽都改回」，S1(i) 的判据（两份逐字节相同）不动，差在哪几个扇区逐洞照报。这一处改动记在登记第十二节修订 2 第 1 条，要不要认由主 agent 定。
4. **第二段**：S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} × (k, 位置) ∈ {(0, —), (1, 前), (2, 前), (4, 前), (1, 后), (2, 后), (4, 后)} 共 42 格，S = 4 的 (4, 前)、(4, 后) 两格几何排不下（锚点模型 `fits=false`），跑 38 格；每格两个回收时点，共 76 组读数。阳性对照 PC-多扣（每个 (S, ρ) 的 k = 0 与 (2, 后)）、PC-洞、PC-分配器（S = 4、ρ = 1、k = 0，换回不装根环表的分配器）、PC-判定器（10 条合成用例）都跑了。Q1e（附带）只在 S = 8、ρ = 1 的格上算（登记写死），别的格标「够判后未跑」。
5. **单测与变异**：`cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `test result: ok. 17 passed`。`crates/mutations.tsv` 里点着这个装置的 22 行：这一次新加 9 行（M35–M43）、改锚 3 行（M32–M34，测试名跟着改），其余 10 行没动；22 行在草稿目录的仓副本里逐条施加、跑点名的测试，22 行全红（抓到 22 / 无效 0 / 没红 0），门禁 code-source-discipline 的 mutation-tables 格（锚点唯一命中）通过。门禁 checker-tier-crates-mutation-replay 整表复跑没跑（归提交前的整轮）。其中另有 9 行测试名末尾带 `$`，门禁 checker-tier-crates-mutation-replay 按字面找 `test … FAILED` 找不到它们，没改，交主 agent（登记第十二节修订 2 第 10 条）。

### 结果整行抄自产物（产物已移出工作区的行写成一行指针）

产物 `e156-alloc-basis-counts-2026-09-26.out`（4007 行，2026-10-05 现查 sha256 `cbbb5f38033ce0c93baa837044abe261e07471cb7ff1af150457f4ad6546e136`；2026-09-26 跑完时页内记的是 `1eaf5ea6…`，之后提交 `bc57af7a`（按内容起名）改了这份产物里的 160 行，`git show bc57af7a^:e156-alloc-basis-counts-2026-09-26.out | sha256sum` 得 `1eaf5ea6…`，现值与旧值不同）；装置把区域数 R 改成本地常量之后又跑一份 `e156-alloc-basis-counts-2026-09-26-r2.out`，与前一份逐字节相同（`sha256sum` 现值同为 `cbbb5f38…`），第 5 次重跑之前 `replay.sh` 登记的是 r2 那一份。完成标记：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4007 行（name=done）
```

**前提 1（问题单第一节）：成立。** 两段最小复现（S = 4、ρ = 1）上崩溃那次的 txg 在环里没有根，新实例第一个 txg = 洞 + 1；「前」洞的槽从没写过，「后」洞的槽里仍是 txg 4 的根：

```
E7RESULT name=pq1 position=front hole_txg=5 prefix_applied=1 first_txg_of_new_instance=6 ring_has_root_at_hole_txg=false hole_slot_content=never_written a16_applicable=true
E7RESULT name=pq1 position=back hole_txg=16 prefix_applied=1 first_txg_of_new_instance=17 ring_has_root_at_hole_txg=false hole_slot_content=root_txg_4 a16_applicable=true
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 1143 行（name=hole_recovery）
E7RESULT name=pc_c2 ring_has_root_at_hole_txg=true mount_return_h_que=0 a14_mismatches=9 seen=true
```

阳性对照 PC-c2（切段点包进根槽那一次写）在同一段历史上环里有了 txg 16 的根、挂载交回那一刻 h_缺 = 0、门槛与锚点有 9 个观测点不符——这套测量分得出「有洞」与「没洞」。

**前提 2：不成立（两个 F 口径合成一个）⇒ 岔路单第 3 行失效、不设量。** 抬 F 那一串先把新 F（8）写进每块盘的系统配置、过屏障：切在最后一笔系统配置写之后的镜像上 F_生效 已是 8；每条推空根带的 F 与切在它之后的 F_生效 都是 8：

```
E7RESULT name=pq2 raised=true floor_before=0 new_floor=8 effective_floor_before_any_write=0 system_configuration_writes=6 effective_floor_after_last_system_configuration_write=8 system_configuration_writes_after_first_root=4 pushed_roots=txg12:root_floor8:effective8,txg13:root_floor8:effective8 publishes=2 reclaimed_placements=61 the_two_readings_merge=true
```

`system_configuration_writes_after_first_root=4` 是两次推空各自在根槽之后轮换两块盘的系统配置（与前提 1 那一格同一个持久顺序），不是新 F 的先写。

**前提 3：只有岔路 2 那一族按旧式子定义**（登记第一节 1.1，对登记文字的核）。它对岔路 1 的间接影响（Hh 历史碰不碰得到新准入式子）在装置上核：全部 38 格与第一段两段历史上发布被准入拒 0 次、可写挂载都在取号之前判够（`mount_admission=admitted_before_acquisition`）、推抬 F 0 串、F_生效 恒 0（`g_adm_nonzero=0`，见两条判决行）。

**问题单第三节（回退改形态作废的问法）**：挂着回退那一形上 U11、U13 的结构都成立（txg = 现行 + 1、实例代号不变、实例表行数不变、普通重开之后隔离 0 槽、被抛弃的根 0 条）；U13 (a) 回退那一版的记账第 5 项是 356 槽，I-3.11（已分配减 defer 等于最新根走读） 判成立；Q7c① 的转色改在造出来的基底 β_syn 上核：

```
E7RESULT name=u11_forward_rollback txg_before=8 txg_of_rollback_publish=9 instance_before=2 instance_after=2 instance_table_rows_before=1 instance_table_rows_after=1 remount_instance=3 isolated_after_plain_remount=[(0, 0), (1, 0)] abandoned_roots_after_plain_remount=0 forward_rollback_structure_holds=true
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 1211 行（name=u13a_rollback_to_oldest_candidate）
E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true
E7RESULT name=q7c2_threshold_at_least basis=beta_syn k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one
```

第一段判决行：

```
E7RESULT name=verdict stage=first premise_one_reproduced=true premise_two_the_two_readings_merge=true premise_two_answer=not_holds fork_three=void_not_measured g_adm_nonzero=0 s1_failures=0 a11_mismatches=0 a14_mismatches=0 pc_c2_seen=true k11_holds=true forward_rollback_structure_holds=true u13a_structure_holds=true u13b_flip1=true u13b_flip2_not_applicable=true stop_reasons=none
```

**岔路单第 1 行（第二段）**。判决行（作废、停机类的计数全是 0）与 12 个取样点 (S, ρ, 回收时点) 的判定：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3994 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3995 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3996 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3997 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3998 行（name=verdict）
E7RESULT name=verdict stage=second_group s=8 rho=1 timing=real q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
E7RESULT name=verdict stage=second_group s=8 rho=1 timing=every_publish q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4001 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4002 行（name=verdict）
E7RESULT name=verdict stage=second_group s=16 rho=1 timing=real q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
E7RESULT name=verdict stage=second_group s=16 rho=1 timing=every_publish q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4005 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4006 行（name=verdict）
```

- **Q1c-前（第 1 行的判据）**：12 个取样点全部判「越过 2 槽」，也全部越过 1 槽；最小的越过点都是 h_缺 = 1（第一次「前」洞挂载交回的那一刻，txg 7）。「前」洞的 Δ 峰值在 12 个取样点上逐点相同：h_缺 = 1、2、3、4 时 10、18、26、34 槽（S = 4 只排得下 k ≤ 2，到 18）。S = 8、ρ = 1、「实」那一行：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3935 行（name=q1c）
```

- **Q1c-后（照报，不算判据）**：12 个取样点全部越过 2 槽，登记第四节预推的「后洞必然多扣得多」没被推翻（失败条款 F23 没触发）；「后」洞把门槛钉在环里那条旧根上，Δ 随 h_缺 每步涨一次发布换下的量（ρ = 1 时约 14–16 槽、ρ = 1/4 时约 8–10 槽），到最后一个 h_缺 取样点 S = 4 / 8 / 16 分别是 174 / 366 / 716 槽（ρ = 1）、118 / 238 / 478 槽（ρ = 1/4）。
- 各取样点逐个列（一条命令从产物里数，输出原样）：

```
$ grep '^E7RESULT name=q1c ' e156-alloc-basis-counts-2026-09-26.out | grep -E 'quantity=q1c_(front|back) ' | awk '{h=$13; sub("by_h_que=","",h); n=split(h,p,","); split(p[1],a,":"); split(p[n],b,":"); sub("peak","",a[2]); sub("peak","",b[2]); print $3, $4, $5, $6, $8, $10, "h" a[1] "->" a[2], "...", "h" b[1] "->" b[2]}'
s=4 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h12->174
s=4 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h12->174
s=4 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h12->118
s=4 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h12->118
s=8 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h24->366
s=8 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h24->366
s=8 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h24->238
s=8 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h24->238
s=16 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h48->716
s=16 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h48->716
s=16 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h48->478
s=16 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h48->478
```

- **Q1d（随洞怎么长、在什么规模上越过一个单元的槽数）**：12 个取样点都判「随 h_缺 增长」（相邻差全部 > 0），第一次越过 2 槽 / 1 槽都在 h_缺 = 1、「前」洞 k = 1、第 1 次工作负载发布之后的挂载交回、txg 7。第二读法（不进判定）：按注入的洞数 k 分组时 Δ 峰值**不随 k 增长**（S = 8、ρ = 1：k = 1 / 2 / 4 时 366 / 360 / 344 槽），峰值由「后」洞把门槛钉住的那一段撑着，不由洞的个数撑着。S = 8、ρ = 1、「实」那一行：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3937 行（name=q1d）
```

- **几何敏感性**：Q1c、Q1c-洞、Q1c-前、Q1c-后、Q1d 在 12 个取样点上都一致（只在这 12 个取样点上）；「实」与「每」逐观测点相同（`f24_real_every_differ_points=0`）：

```
E7RESULT name=geometry_sensitivity quantity=q1c_front result=consistent_on_12_sampling_points per_sampling_point=s=4_rho=1_timing=real:above,s=4_rho=1_timing=every_publish:above,s=4_rho=1/4_timing=real:above,s=4_rho=1/4_timing=every_publish:above,s=8_rho=1_timing=real:above,s=8_rho=1_timing=every_publish:above,s=8_rho=1/4_timing=real:above,s=8_rho=1/4_timing=every_publish:above,s=16_rho=1_timing=real:above,s=16_rho=1_timing=every_publish:above,s=16_rho=1/4_timing=real:above,s=16_rho=1/4_timing=every_publish:above
```

- **失败条款**：F1（甲-T1 在「前」洞上多扣得多，出路不能只是一句射程，交主 agent）在 12 个取样点上全部触发；F2、F22、F23、F24 都没触发：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3993 行（name=failure_clauses）
```

- **钉绝对值的锚点**：锚点模型每个 S 的 A10 滞后恰为 3S、排得下的格上 r2 两条闭式与按环现数逐点相同、k = 0 的 Q1a 与全部格的 Q1b 恒 0；装置在 38 格的每个观测点上门槛、h_缺、h_洞 与它逐点相同，Q1a 不低于它的树表下界（判决行 `v2_*`、`v6_*`、`s11_*` 全 0）。阳性对照 PC-分配器：不装根环表时树表单元在这一次挂载里一个都发不出：

```
E7RESULT name=anchor_model s=4 k0_end=48 a10_lags=[12] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:false,1back:true,2back:true,4back:false
E7RESULT name=anchor_model s=8 k0_end=72 a10_lags=[24] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:true,1back:true,2back:true,4back:true
E7RESULT name=anchor_model s=16 k0_end=120 a10_lags=[48] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:true,1back:true,2back:true,4back:true
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3859 行（name=pc_allocator）
```

锚点模型的 Rust 移植与 Python 脚本逐行比（Python 输出在草稿目录，不入库；比的是 1092 行 `DUMP`）：

```
$ grep '^E7RESULT name=anchor_dump ' e156-alloc-basis-counts-2026-09-26.out | sed 's/^E7RESULT name=anchor_dump /DUMP /' | sha256sum
5e61a3e939ebd5df09c76c80dde30352482245b7e83a1164e6ccf4e0223794e0  -
$ python3 anchors_e156_r4.py --dump | grep '^DUMP' | sha256sum        # 脚本全文在登记第十三节
5e61a3e939ebd5df09c76c80dde30352482245b7e83a1164e6ccf4e0223794e0  -
```

- **Q1e（附带，只在 S = 8、ρ = 1）**：有洞的观测点上第 4 次重跑时 `check_pool_image` 的 I-3.1（已分配统计对得上） 判红 14 / 148 个状态（都是崩溃恢复之后的 L2），落在 (2, 后) 的 txg 57 与 (4, 后) 的 txg 57–69：

```
$ grep '^E7RESULT name=q1_cell ' e156-alloc-basis-counts-2026-09-26.out | grep 's=8 rho=1 ' | grep -o 'k=[0-9] position=[a-z-]*\|i31_red_on_hole_states=[0-9]*\|i31_judged_hole_states=[0-9]*' | paste -d' ' - - -
k=0 position=- i31_red_on_hole_states=0 i31_judged_hole_states=0
k=1 position=front i31_red_on_hole_states=0 i31_judged_hole_states=21
k=2 position=front i31_red_on_hole_states=0 i31_judged_hole_states=23
k=4 position=front i31_red_on_hole_states=0 i31_judged_hole_states=29
k=1 position=back i31_red_on_hole_states=0 i31_judged_hole_states=21
k=2 position=back i31_red_on_hole_states=1 i31_judged_hole_states=24
k=4 position=back i31_red_on_hole_states=13 i31_judged_hole_states=30
```

- **产物齐不齐**：逐观测点的行按格数与锚点模型 `--dump` 那一格的观测点数逐格相同（38 格 + PC-分配器 46 + PC-c2 25 = 2141 行；k = 0 与 (2, 后) 多带四列 PC-多扣，234 + 180 行 × 2 个 ρ）：

```
$ grep -c '^E7RESULT name=q1_observation ' e156-alloc-basis-counts-2026-09-26.out
2141
$ grep '^E7RESULT name=q1_observation ' e156-alloc-basis-counts-2026-09-26.out | awk '{print NF}' | sort | uniq -c
   1313 40
    828 44
```

### 复跑

```
bash scripts/experiment-apparatus/replay.sh E156
cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts
```

`scripts/experiment-apparatus/replay.sh` 里 E156（alloc-basis 四条岔路的代价数） 那一行现在指 `e156-alloc-basis-counts-2026-10-18.out`（第 7 次重跑的主产物，release 二进制，带 `--features verdict-store,gpu`、经 `scripts/crash/gpu-quiesce-run.sh` 起，挂钟约 241 秒；驱动 `driver_e156` 改成 `cargo run -q --release …`，特性与清场经 `CHECKER_TIER_FEATURES`、`CHECKER_TIER_QUIESCE_PREFIX` 带上，`BINDGEN_EXTRA_CLANG_ARGS` 由仓根 `.cargo/config.toml` 给；多机配置 `ENABLE_GPU=1` 时不带 gpu 特性编的装置会在 `verdict` 函数里 panic，所以特性不能省；`replay.sh E156`（alloc-basis 四条岔路的代价数） 没跑，整份复跑是重型，归提交前；核对用的锚点输入与脚本在 `e156-alloc-basis-counts-2026-10-18-support/`，不进 `replay.sh`）。第 7 次之前那一行指 `e156-alloc-basis-counts-2026-10-05-r2.out`（第 6 次重跑的主产物，执行员用 release 二进制跑出来、debug 二进制前 1803 行与它逐字节相同，`replay.sh E156`（alloc-basis 四条岔路的代价数） 没跑，整份 debug 复跑归提交前；判别力对照 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out` 与它的 `.patch`、比较 `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.py` / `.txt`、点集核对 `e156-alloc-basis-counts-2026-10-05-r6-point-sets.py` / `.txt` 与它读的 `e156-alloc-basis-counts-2026-10-05-r6-anchor-pins.txt`、`e156-alloc-basis-counts-2026-10-05-r6-anchor-lowered.txt` 都不进 `replay.sh`）。在那之前指 `e156-alloc-basis-counts-2026-10-05.out`（第 5 次重跑的产物，留着；此前指 `e156-alloc-basis-counts-2026-09-26-r2.out`，这一份与 `e156-alloc-basis-counts-2026-09-26.out` 逐字节相同、都留着；归属跑 `e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out` 与新产物逐字节相同、不进 `replay.sh`；比对 `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.py` 与它的输出 `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`；变异两行 `e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`、`e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv`）；2026-10-05 的新产物是执行员用同一个二进制直接跑出来的，`replay.sh E156`（alloc-basis 四条岔路的代价数） 没跑（整轮复跑归提交前）。第 4 次登记的复跑写法（驱动 `driver_e156` 在仓根 `cargo run -q -p singlefs-checker-tier --bin e156_allocation_basis_counts`，经内存包装）；2026-09-27 指向旧产物时复跑那一行判「字节一致」，汇总行原样：

```
字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0
```
第 4 次重跑之前的产物一个没删，数字出自已不存在的实现（挂载时回退、挂载内不回收、单节点分配记录树），不再承重：`e156-alloc-basis-counts-2026-09-22-stage1.out`、`e156-alloc-basis-counts-2026-09-22-stage2.out`、`e156-alloc-basis-counts-2026-09-22-stage3.out`、`e156-alloc-basis-counts-2026-09-23-stage1.out`、`e156-alloc-basis-counts-2026-09-24-stage2.out`、`e156-alloc-basis-counts-2026-09-24-stage3.out`、`e156-alloc-basis-counts-2026-09-24-stage4.out`、`e156-alloc-basis-counts-2026-09-25-r3.out`、`e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`（各自当时的读法在本页「历史版本」）。

### 它答不了的

- **G12 那一臂只作旁路谓词评估**：答「同一条真实历史上两条谓词各扣住哪些槽」，答不了「按 G12 跑出来的历史会长成什么样」；G12 的「实」与「每」在这个装置里是同一次计算（第 4 次重跑时每次发布都回收，观测点那一刻的环就是最后一次真实回收用的环），F24 对 G12 那一列触发不了。PC-多扣的四个组合按构造给同一个数。
- **洞只由覆盖写或空发布那一次的崩溃造**：写行、暖机、抬 F 的推空、挂着回退那一次上的崩溃，与「最新根槽挂载时读不出」那一类没造。
- **Hh 里没有抬 F、没有被抛弃根**：F > 0 或影子账开着时两条臂的差没量。
- **只一种间隔、一种池**：造洞之间隔 2 次工作负载发布，池 4 GiB × 2。
- **判定只在这 12 个取样点上**：S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} × 两个回收时点；S = 4 的 k = 4 两格几何排不下没跑。
- **Q1e 只在 S = 8、ρ = 1**：别的格「够判后未跑」。
- **确定性装置**：跑两遍一致不是统计证据；证据强度来自钉绝对值的断言（锚点模型、K1-1、R-4）与 22 条变异。
- **交主 agent 的**：① 抬 F 那一串把扣住位留到最后一次推空之后才放，比 F_生效 取到新值晚（登记第三节 j7、j8），不是岔路单第 3 行的两个候选之一；② 可达状态里记账第 5 项能不能为 0 这一段没量到：第 3 次重跑的可达基底 `beta_hr_rollback_row`（产物 `e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`）第 4 次重跑时走不出来（挂着回退那一版第 5 项是 356 槽），I-3.11（已分配减 defer 等于最新根走读） 的判别力自证只落在造出来的基底 β_syn 上，条款正文照这一句写；③ 问题单第四节点名的 `crates/mutations.tsv` 第 551 行不是这个装置的变异；④ S1(i) 的对照造法改了（「这一段做了什么」第 3 条）；⑤ 点着这个装置的 9 行变异测试名带 `$`。

### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 装置自己那一份引用 | 每个 txg 的那一版的发布输出里全部单元的落点（跨度展开），实例表单元第一次重写之前按根记录的实例表指针补上；零单元那几版按 mkfs 根记录的两条指针 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1778` `facts_of_published_version` | 发布输出（`TransactionOutput.units`、根记录指针）；「两盘同槽」这一前提 |
| 2 | 盘上仍分配记录 | 同一批环里的根，`recovery::allocation_records_under_root` 读出的未释放记录逐盘展开；没有分配记录树的那几版按根记录两条指针的这块盘那一份 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1823` `referenced_slots_read_from_disk` | 盘上的分配记录树与根记录；跨度的定义 |
| 3 | 从镜像的根环现读 | 每个观测点读环里全部自证过的根，门槛 = max(F_生效, 环里最旧有效根)；第 6 次起「环里最旧有效根」照定乙并进这次挂载读回全零、还没写掉的槽（挂载时在切段镜像上读，`root_ring_slots_reading_back_all_zero_at_mount`），h_缺、h_洞 按登记字面现数、不并 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:2096` `read_hh_state` | 根环落点公式（`target_for_publish`）、`effective_rollback_floor`、实例表判被抛弃、`read_root_ring_slot` 与「每个字节都是 0」的全零判法 |
| 4 | 锚点模型 | 命令四 `anchors_e156_r4.py` 的逐行移植，门槛那一处照第 6 次登记命令四 `anchors_e156_r6.py` 改（并进挂载时还没写过的槽）：只用脚本头那几句条款算洞、挂载做完的 txg、门槛、h_缺、h_洞、树表下界 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1356` `anchor_model::run_the_anchor_hole_script` | 根环 txg u 落槽 u mod 3S、区域设备 [0, 1, 0]、「每次发布换下树表单元」、暖机规则、c2 之后首 txg 规则 |
| 5 | 录写切段 | 录下崩溃那次发布的设备写，切在根槽那一次写之前施加到发布之前的镜像上 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1011` `crash_publish_before_its_root_persists` | `FixedGeometry::classify` 认根槽写；同一次发布的输入 |
| 6 | 发完改回 | 真发一次，再把根槽与每块盘系统配置两槽改回发布之前 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:924` `crash_before_root_persists` | 根槽落点公式；系统配置每盘 2 槽、槽宽 4096（本地常量）；同一次发布的输入 |
| 7 | 甲-T1「实」与「每」 | 「实」读真实分配器的空闲图；「每」按门槛对记录现算 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1955` `over_withheld_on_device` | 同一份分配器记录、同一个门槛、同一份引用 |

路径 1 与 2 在每次挂载交回与每格终点逐根逐盘比（`w9_mismatches`，全部 0）；路径 3 与 4 在每个观测点比（判决行 `v6_a11_mismatch_points=0`、`v2_a14_mismatch_points=0`），路径 4 与 Python 脚本逐行比（「结果」里的 sha256）；路径 5 与 6 逐洞比（`cut_matches_root_and_system_configuration_restored_image`，全部 true）；路径 7 两列逐观测点比（`f24_real_every_differ_points=0`）。三组比对各自共用的前提（发布输出与盘上记录同出一次发布、根环落点公式、同一份记录与门槛）一起错时比不出来；锚点模型只用条款句，是唯一不经 `crates/` 的一条。第 7 次重跑（2026-10-18）路径 4 另加了新族的洞 txg、`k_满`、终点、`fits` 与 `k_满后`：装置的 Rust 移植与 Python 脚本逐点比（停机 S20，`anchor_r7_summary` 58 行、`anchor_r7_dump` 3567 行、`anchor_r7_k0_summary` 3 行、`anchor_r7_k0_dump` 384 行，与 `e156-alloc-basis-counts-2026-10-18-support/` 下脚本的输出 `cmp` 都无差异，命令在「第 7 次重跑」一节）；这一比对共用的前提是同一份条款句，两边读错同一处时比不出来（开工前试跑里脚本漏了 h_洞 的窗口左端，装置对、脚本错，两边不同才露出来）。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次重跑量的是根环上有洞时甲-T1 多扣随洞数的涨法（问题单 H1），没量准入不等式里的项，这一分项的定案与依据不受读数影响 |
| D16（发布语义） 已定项 1 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次读数里 Hs-后（一块盘持续写不进根的形）24 个单位全判「仍在涨」，与这一分项射程「扣住的单元随发布线性涨、没有上界」不矛盾，F34 不触发；这一分项的依据段没引 E156（alloc-basis 四条岔路的代价数），这一格该不该升成支撑交主 agent |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次重跑量洞上多扣的槽数，不量 journal 重放集合，C380（环上有洞时释放代判不出唯一值） 里释放代判不出唯一值那一半没碰 |
| D3（空间分配） 已定项 7 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次重跑读的是分配器空闲图与回收门槛，没改一条分配记录的落点粒度与 key、value 宽度 |
| D5（快照 / 空间记账机制） 已定项 4 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次重跑没量统计量清单，只量甲-T1 多扣的槽数随洞数的涨法 |
| D8（核心索引结构） 已定项 14 | 备料 | 2026-10-23 不受影响：只改了产物名的写法，结论、数与产物不变；此前 2026-10-21 不受影响：整行抄的产物行照产物现在的键名重抄（值逐字不变），补点名两份核对输出与编号简称，数、结论与产物不变；2026-10-18 不受影响：第 7 次重跑没碰 keyspace 与前端的分法，只读分配记录树里的记录算多扣 |

## 历史版本

### 2026-09-26：第 4 次重跑之前的正文（挪入，原样）

**第 4 次重跑之前的标题状态**：部分已跑（第 3 次重跑第一段已答问题单第 3 行；岔路 7 的 G27 判别力自证 Q7c① 补了一个可达基底、首次在可达状态上转色，续不续判交主 agent 定；岔路 1、3 的 S=16/ρ=1/4/位置「前」/k=4 与 HF 18 格全扫仍未做）（2026-09-22 第一、二段 + 2026-09-23 R2 第一段 + 2026-09-24 R2 第二、三段 + 2026-09-24 R2 续派两段（s4）+ 2026-09-25 第 3 次重跑第一段 + 2026-09-25 第 3 次重跑续派（补可达基底 beta_hr_rollback_row），入库装置，确定性，11 单测 / 12 条变异全抓；分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）重算钉着旧布局的三个常量与另外六处，H0/HR 不再撞墙、跑满登记步数；岔路 1（Q1d 在 S=4 上「单调」变「不单调」）与岔路 7（`q7d2_min_item5` 「非 0」变「= 0」、F16「触发」变「不触发」）判定变了，岔路 3（Q3c/Q3e/K9）在已核的量上不变；覆盖 S1 对拍（含 S1(c) 完整重放，隔离槽数 34→54）、锚点 K1-1/A-D8/A1/根环槽数锚点/节点容量锚点、H0/HR/HK/HK-F0/HF/Hh/HX/HY 八族历史、岔路 2 全套（Q2a/Q2b/Q2c）、岔路 7 全套（Q7a–Q7f，9 个基底）、岔路 3 一个几何格（Q3a–Q3d 都有值）+ Q3e（X8（小池耗尽两口径构造）-A/HY，真实公开入口构造出来了）、岔路 1 两个 S 取样点（S=8/S=4）× 三个洞数取样点（Q1a–Q1d）+ 步数对齐对照（分开「随洞长」与「随步数长」）；HY 的覆盖写次数改成搜出满足 e ≥ max(8, f) 的那一档，这一轮搜到 0（未满足，见「它答不了的」）；岔路 1、3 的 S=16/ρ=1/4/位置「前」/k=4 与 HF 18 格全扫仍未做）

**问题（第 1–3 次重跑）**：跑前登记 `e156-preregistration.md`（637 行，岔路 2 全套由它交用户定）；岔路 1、3、7 还开着，重跑登记 `e156-r2-prereg.md` 只覆盖这三行（岔路单第 9、11、12 行）。R2 第一段（2026-09-23）只做岔路 7；R2 第二、三段（2026-09-24）各做岔路 3、岔路 1，都是**缩小范围版**：只在 S = 8（mkfs 默认）、ρ = 1 一个几何取样点上各跑一条代表性历史。R2 续派（2026-09-24，同日，s4 执行员，两段）在此基础上补：第一段查清「零个洞时 Δ = 20」的来源（装置诊断代码自己的一处过滤漏洞，已修，不改变任何已报的 Δ 值）、给岔路 1 补 S = 4 第二个几何取样点、给岔路 3 的 K9 前提补现核；第二段（主 agent 续派消息「岔路表里还开着两行」）给岔路 1 补步数对齐对照（把「Δ 随洞数长」与「Δ 随步数长」分开）、给岔路 3 构造出 Q3e（X8（小池耗尽两口径构造）-A/HY，真实公开入口）并补 Q3d。装置放在**入库装置**上：`crates/singlefs-harness` 的只读 bin `e156_allocation_basis_counts`，驱动真实 mkfs / 暖机 / 新池新建文件 / 覆盖写 / 空发布 / 可写挂载 / 管理员回退 / 抬回退下界，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码；G27 另写一份独立实现（R2 起改成只读被判镜像的记账行，做法在「这一段做了什么」第 7 条）。R2 第一段做登记「五、5.7」第一段：①「十一」S1 与 `crates/` 的逐项对拍（含 S1(c) 完整重放）；②锚点 K1、锚1、锚2、锚3；③R3（G27 改读镜像）；④岔路 7 全套 Q7a–Q7f，9 个基底（β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0）。R2 第二段做岔路 3：一个 HF 历史上 G7（公开入口重组）与 F-扣（真实 `raise_rollback_floor`）各自的 D_rel、K9。R2 第三段做岔路 1：三个 Hh(k) 历史（k = 0、1、2）上甲-T1 与 G12 的多扣、Δ。缩小范围与续派各段还差什么见「它答不了的」与本页历史版本 2026-09-24 条目。

#### 这一段做了什么（第 1–3 次重跑）

1. **锚1/锚2/锚3**：单元区每盘 211968 槽（`4 GiB ÷ 16384 − 50176`，与实现侧 `UNIT_AREA_START_SLOT` 回比）、根环 S=8 时 24 槽（与 `target_for_publish` 逐点核对映射一致，0 处不符）、分配记录节点容量 812（`⌊(16384−135)÷20⌋`）——锚1、锚2 在第一、二段用 `python3 -c` 独立复算过；锚3（节点容量）与两个几何锚点这一段改成装置自己在 `main()` 开头现算并 `assert_eq!`，不再是外部脚本核一遍。
2. **K1（S1(a)）**：装置真跑 mkfs→取号→暖机→新池新建文件，盘 0 记账第 1 项 13 槽、第 2 项（空闲）211955 槽、第 5 项 1 槽，逐槽落点 `50180:2,50240:1,50242:2,50244:1,50245:1,50246:1,50247:1,50248:1`，与登记逐字相符（`matches_registered=true`）。
3. **S1(c) 完整重放（2026-09-23 R2 第一段，取代第一、二段的「轻量核」）**：装置自己用公开入口重放 `rollback_by_a_forward_publish.rs` 的 `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation` 场景（A → B(4100B) → 重开取号 2、写行、暖机 → C(2500B) → 回退到 A → 普通重开），逐项现算并 `assert_eq!`：D 的 jsn 接在 C 的 8 之后（`c_counter=8 d_counter=9`）、5 条记录 `(1,4)(2,5)(2,8)(3,9)(3,10)` 原样在环里、普通重开取到实例 4、`isolated_slots_per_device == [(0,34),(1,34)]`、`abandoned_roots_unreadable == 0`——全部通过，是装置自己重新走一遍这个场景、不是确认同名测试还绿。**S1(b)（`reuse_after_raising_the_floor.rs` 两个用例）仍只是现跑测试文件本身，本段没做装置侧重放**，见「它答不了的」。
4. **H0/HR 两个合法状态**：H0（ρ=1，连续覆盖写，无抬 F、无回退）在第 50 步真实撞上 `AllocationRecordsExceedOneNode { records: 820, capacity: 812 }`（`crates/singlefs-core` 今天单节点分配记录树的硬容量——回收只在挂载/回退重建与抬 F 两处发生，登记「三」I3（一次挂载分配记录只增不减），H0 定义里两样都没有，分配记录只增不减）；β1 取 H0 实际跑到的第 49 步（txg 52）。HR 前缀同理止步于 49 步，回退到候选集里 txg 最小的根（txg 29，实例 1，候选集 24 条），尾段止步于 47 步（txg 102，实例 2）；β2 取 HR 实际跑到的最后一步。两个状态都是真实合法状态（`check_pool_image` 在 β1 / β2 两个端点上跑过；逐步核 checker 判定那一格没做，写在「它答不了的」那一节）。
5. **锚6（S1(e)）**：一次覆盖写（O）释放 10 槽、分配 10 槽，H0 全程 49 步逐步精确（`503 = 13 + 49×10`、`491 = 1 + 49×10`），与登记「四」P6 引 E153（账本形态与环上有洞的代价） 那半「10 槽」相符；空发布（E）本段未测（ρ=1 时 H0 不含 E）。
6. **岔路 2（Q2a/Q2b/Q2c）**：只读 D28（挂载期承诺量） 已定项 1 的公式、D5（快照 / 空间记账机制） 已定项 4 的对照表、`.claude/gate.d/doc-registries.sh` 的 admission-terms 格 写出清单 L（4 处），再跑登记指定的命令 `grep -rn 'defer 待释放' .claude/ crates/ scripts/`（命中 45 处，去重 file:line 后 45 处）；`|M∖L|=41 > 0`，`|L∖M|=0`。门禁 doc-registries 的 admission-terms 格的判别力自证在草稿目录仓副本上做：①只删公式那一项 ⇒ 红；②只删对照表那一行（不改别处）⇒ **仍然红**（`03-空间分配.md:82` 的公式副本没跟着改）；③连同公式副本一起改 ⇒ 绿，且门禁 doc-registries 的 admission-terms 格脚本本身一个字节都没动。
7. **R3（2026-09-23 R2 第一段）：G27 改成只读被判镜像的记账行**，不再读内存里的 `PoolAllocator`（第一、二段那一版两侧都不读被判镜像的缺陷，「历史版本」记着撤回的经过，到这里改掉）：新函数 `allocated_minus_deferred_mismatches_referenced(pool, accounting_slot, referenced)` 直接解析记账树叶字节；旧的内存读法 `allocated_minus_deferred_matches_referenced` 保留成变异反面专用（U8）。
8. **HK / HK-F0 两族新历史（2026-09-23 R2 第一段，登记「五、5.2」）**：HK 是真实历史——β0 → O → E → 无崩溃重开（写行、暖机）→ O×4 → 抬 F 到上限 → O → 管理员回退到候选集里 txg 最小的根 → O → c2 崩溃（记录已持久、根槽没持久）并恢复 → O，覆盖 骨架发布 列的全部发布种类；HK-F0 是同一串操作但分配器强制 `ReuseWindow::ForcedToZero`（不可达阳性对照，R7）。两族各自的四个观测点（无崩溃重开、回退、抬 F、c2 恢复之后）连同 β0/β1/β2、β_syn（β0 镜像做 (第 1 项 −1、第 5 项 −1、第 2 项 +1)）、β_F0（HK-F0 新池新建文件之后）凑成 9 个基底，逐个跑 Q7a（Bd(+1)/(−1)/(+8)）、Q7c①②、PC-检查-Bk1/Bk2/Bk3。
9. **发现并修了两处装置自己的实现缺口（写在 `e156-r2-prereg.md`「十二」修订，产物之前）**：① HK 的回退目标选取原样抄了 HR 那段代码（`readable_roots` 里挑 `checkpoint_txg` 最小的一条），没按 `effective_rollback_floor` 过滤候选集——HK 抬过 F 之后 genesis 已经不是候选，直接选最小 txg 会选到一条已经被挡在外面的根，补一条按登记 R7 早就写死、只是先前没写出来的过滤。② `mount_writable`/`mount_rollback` 内部按 `rebuilt_allocator` 重建分配器，`ReuseWindow` 恒回落产品默认，每次挂载/回退之后要重新 `set_reuse_window`，只在最外层设一次不够。
10. **岔路 7 全套（Q7a–Q7f，2026-09-23 R2 第一段，取代第一、二段那版 Q7a/Q7b/Q7c）**：见「结果」。
11. 命名纪律：`naming-lint.sh` 对文件全部通过（0 处违规；R2 这一段又新增并改名了 40 余处单字母/字母加数字的标识符，比如 `g8_prime_is_red` → `allocated_minus_deferred_mismatches_referenced`、`beta_w/r/f/l2` → `beta_after_reopen/rollback/raise_floor/crash_recovery`）。变异表 `crates/mutations.tsv` 对这个 bin 累计 3 条（第一段那条 `referenced_slots` 丢实例表兜底 + R2 新增两条：R8 数分配不数释放、G27 退回读内存分配器的行），三条都手工验证红→绿、已还原、基线仍全绿；门禁 checker-tier-crates-mutation-replay 整张表未跑（归 `gate-triage`）。

**2026-09-24 先做的一步：在今天的 `crates/` 上重跑 R2 第一段，与 2026-09-23 的产物逐字节比**——`crates/singlefs-core` 的 `mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs`（`git hash-object` 与「十二」修订 5 记的哈希比对，5 个文件的哈希都变了）已经被另一条线改动。重跑之后只有 3 处不同，全部集中在基底 `HK_l2`（c2 崩溃恢复之后那个观测点）：`base_i31_red` 由 `true` 变 `false`（该基底的 base 状态 I-3.1（已分配统计对得上） 由红转绿）、连带 3 行 `q7a` 与 `pc_check_bk2`/`pc_check_bk3_is_bd_plus_one` 的 `i31_red` 字段同向翻转、`q7a_summary` 的 `all_red_count` 由 12 变 15；`q7c1_flip_seen`/`q7c2_flip_seen`、全部 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行逐字节不变。**这条差异不改变岔路 7 的任何已报判定**：Q7c①在可达基底上仍从未转色（V3 仍触发），本轮的差异只是 `walk.rs`（checker I-3.1（已分配统计对得上） 实现，另有实现员在改）让 `HK_l2` 的 base 状态从「本来就已经红」变成「真的是绿的合法状态」，使 `HK_l2` 那一格新纳入 `q7a_all_red_count` 的合取判定——这一条不归执行员判断对错，只如实记录读数（派发原话）。
12. **岔路 3（第二段，2026-09-24，`run_hf_single_cell`）：一个 HF 历史（S = 8、ρ = 1）上 G7 与 F-扣 的 D_rel、K9**。历史：β0 → 无崩溃重开（给「现行版本里有重写过的实例表单元」）→ 8 次非空覆盖写 → 从同一份快照分别重建三条分支（探上限、F-扣、G7），先核 V13（抬 F 之前三条分支设备字节与快照逐字节相同）再各自抬 F 到同一个上限（`raise_rollback_floor` 探测得到的 `ceiling`）。G7 用公开入口重组：`raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect` 先推带新 F 的空发布到每块盘都盖到，再 `reclaim_released_up_to(..., ReclaimedReuse::Immediately)`；F-扣 用真实 `raise_rollback_floor`。两条分支的推空发布次数都是 3（与 A9 在 φ 靠近 2 的取样点预测的次数量级吻合），最后一次推空发布的 txg 相同（16），因此两个口径在这一格上 D_rel 完全相同（见「结果」）。**只跑这一个几何格**：登记「五、5.2」HF 的 S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} 共 18 格没有全扫；X8（小池耗尽两口径构造）-A（Q3e，需要 P5 那个 128 槽两段小池几何 HX/HY）这一轮没有构造，产物里那一行标 `status=not_done`。
13. **岔路 1（第三段，2026-09-24，`run_hh_cell`）：三个 Hh(k) 历史（k = 0、1、2，S = 8、ρ = 1，洞的位置固定「后」）上甲-T1 与 G12 的多扣、Δ**。每格：β0 → 工作负载直到环转过一圈（txg ≥ 3S + 3 = 27）→ 造 k 个洞（c2 崩溃 + 一次可写挂载恢复，每个洞之间隔 2 次工作负载发布）→ 最后无崩溃重开一次（甲-T1「实」读法的回收点，也是这一格的测量点）。甲-T1 的多扣（Q1a）= 这一刻记账第 1 项 − 走读引用（健康状态下等于第 5 项/`deferred`）；G12 的多扣（Q1b）= 遍历设备 0 上「已释放」的记录，按 G12 的区间谓词（`环里没有一条根的txg落进[分配代,释放代)`，Hh 没有回退、没有抬 F ⇒`被抛弃∨txg≥F_生效`恒真）逐条判断是否还该扣住，累加扣住的跨度；分配代由装置自己维护的影子账本给（真实记录的 `generation` 字段释放那一刻被改写成释放代，登记「三」N2（释放时改写的分配代字段）；账本实现 `snapshot_allocated_generations`/`record_release_generations`，单测 `allocation_generation_ledger_recovers_the_pre_release_generation` 覆盖，变异 `crates/mutations.tsv` 新增一条并手工验证红→绿）。三个洞数取样点上 Δ 单调增（20 → 70 → 120，每加一个洞 +50），h_缺 也单调增（0 → 8 → 14）；见「结果」与「它答不了的」（h = 0 时 Δ 已经是 20，越过 2 槽 / 1 槽两个门槛，这一条的机制没有查透）。**只跑这一个几何格**：S ∈ {4,8,16} × ρ ∈ {1,1/4} 的其余组合、洞的位置「前」、k = 4 都没有扫。

**R2 续派第一段（2026-09-24，同日，s4 执行员，产物 `…-stage3.out`）：**

14. **查清 `holes=0` 时 Δ = 20 的来源：是装置诊断代码自己的一处过滤漏洞，不是 G12 谓词的真实行为**。`run_hh_cell` 给 Q1b（与逐槽列举 `q1_delta_record`）过 `allocator.records()` 时只按 `is_released` 过滤，没核这条记录是不是已经被真实分配器 `reclaim_released_up_to` 回收过——回收之后 `is_released` 仍是 `true`（不删记录，等下一次同起点槽的 `record()` 才覆盖），但槽的位图已清空（`DeviceFreeMap::is_free` 为真）。`holes=0` 那格唯一一次回收（最后无崩溃重开）回收了 9 条记录（11 槽），全部满足「不再扣住」，被诊断循环误当成「G12 判不再扣住、甲-T1 仍算」的证据。修法：过滤条件加 `!device_map_d0.is_free(record.slot)`。**这条修法不改变任何已报的 Δ 值**——`q1a_over_withheld`/`q1b_over_withheld` 都不受影响，`holes=0/1/2` 三格的 `delta`（20/70/120）修法前后逐字节相同，只是逐槽列举更准确了。新增回归测试 `reclaimed_records_are_not_removed_but_their_slots_become_free` 钉住这条修法的前提（第 6 个单测）。
15. **岔路 1 补 S = 4 第二个几何取样点（方向与 mkfs 默认 8 相反）**：ρ = 1、回收时点「实」、洞位置「后」不变，k ∈ {0, 1, 2} 同 S = 8。新增 K8 锚点（核「本地打算跑的 S 与喂给 mkfs 的系统配置字段一致」）、`q1_geometry_sensitivity_s`（S = 8 与 S = 4 上「Δ 越过 2 槽」这条判定是否一致——结果：一致，两点都在 `holes=0` 就已越过）。
16. **岔路 3 的 K9 前提改成现核**：`run_hf_single_cell` 在抬 F 之前新增 `k9_precondition`，用「first_file 那一次 + 8 次覆盖写各一次」这个已知的非空 txg 集合，现读一次 `readable_roots` 数还留在环上的有几条——量出 `non_empty_valid_root_count=9 ≥ 4`，与代码注释的预估一致，但现在是量出来的。
17. **在今天的主工作区上重跑 R2 已有产物，与并发补丁核对**：`crates/` 的 `allocator.rs`/`mount.rs`/`transaction.rs`/`recovery.rs`/`walk.rs`/`format/lib.rs`/`crash.rs`（S3 跟踪的 11 个文件里 7 个）被另一条并发会话改过；把 `stage2.out`（生成于并发补丁之前）里格式没变的 329 行与今天重新跑出来的同名行逐行比对，**逐字节相同**——这批并发补丁没有改到 E156（alloc-basis 四条岔路的代价数） 这几条历史实际触达的代码路径。

**R2 续派第二段（2026-09-24，同日，主 agent 续派「岔路表里还开着两行」，产物 `…-stage4.out`）：**

18. **岔路 1：步数对齐对照，把「Δ 随洞数长」与「Δ 随步数长」分开**。`run_hh_cell` 新增 `matched_to_holes` 参数——`holes=0` 时若非零，把「环转过一圈」的门槛多加 `matched_to_holes × 6` 步工作负载（6 是实测出的「一次造洞固定消耗 6 个 txg」常量，与 S 无关），让总发布步数与 `holes=matched_to_holes` 那格对齐，但不造任何洞。结果：S = 8、S = 4 各两组（`matched_to_holes ∈ {1,2}`）里，终点 txg 与对应真实 `holes=k` 格逐一相同，而 `delta` **全部还是 20**——不随步数增长。据此 `Δ(k) − Δ(0, 步数对齐)` = 50（k=1，两个 S）、100/90（k=2，S=8/S=4）——**这才是「洞上多扣的槽数」，之前报的 20→70→120 曲线，增长部分不是步数带来的，是洞本身带来的**。另外从 `q1_delta_record` 逐槽拆解出 `holes=0` 那 20 槽的构成：两类各 8 条、各 10 槽（分配代 4→释放代 5、分配代 5→释放代 6），机制是「一次挂载里的回收只做一次、且做在重开的最开头，而重开自己还要再推进 2 个 txg」（I3（一次挂载分配记录只增不减）、I13）与洞无关，S=8/S=4 逐槽一致。
19. **岔路 3：Q3e（X8（小池耗尽两口径构造）-A/HY）用真实公开入口构造出来了，Q3d 也做了**。新建 P5 的 128 槽两段小池（`x8a_parameters`/`build_x8a_prefix`/`run_x8a_cell`），走 mkfs → 无崩溃重开 → 反复覆盖写到装不下为止。**HX**（填到真耗尽）：F-扣（真实 `raise_rollback_floor`）与 G7（新增可失败版重组）**都**在第一次推空发布上失败、**都**卡在同一个单元（`AccountingTree`，`NoFreeSlotOnAnyDevice`）——两者在这具体历史上卡的是同一步、同一个原因，不是 P5 原始简化模型里「扣住失败、不扣住成功」那种更尖锐的对比（G7 的真实定义是「推空全部结束才回收」，在完全耗尽的池上反而更早没有可用空间）；单独重放「回收并扣住」这一步，实测回收且扣住 66 条记录、81 槽，`free_slots` 从 5 涨到 86（P5「空闲计数报着那些槽」逐字应验），但 `lowest_empty_segment` 仍是 `none`（与 P5 那个「单独一段全空」的简化模型不完全一样，真实历史里两个聚簇段都混着仍分配与被扣住的槽）。**HY**（阳性对照，提前停手，`free=75`）：两个口径都成功，确认 HX 的失败是真耗尽导致，不是装置或探测本身的问题。Q3d（`D_acct`/`D_alloc`）是从代码已读的调用顺序推出来的，不是逐次持久根现测的（F-扣 回收在第一次推空之前 ⇒ `D_acct=0`；两个口径「能发」都在推空全部结束之后 ⇒ `D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=pump_publishes−1=2`），据实标注方法的限度。

**（2026-09-25 第 3 次重跑续派）**：20. **补第 10 个基底 `beta_hr_rollback_row`，让 Q7c① 第一次在可达状态上转色**。第 3 次重跑第一段的 `q7d2_min_item5` 第一次在可达状态上读到 `min_item5=0`（`family=HR kind=rollback_row txg=76`）——HR 家族回退到候选集里最旧的根、写行发布之后那一步。把这一步捕成一个 `BasisSnapshot`（`basis_of("beta_hr_rollback_row", …, true)`），并入既有的 `bases` 数组（9→10），不改 `q7c_self_test`/`run_q7a_cell`/`run_pc_check` 任何一行公式，只是多喂一个取样点。详见「历史版本」2026-09-25 续派段与「结果」。

#### 结果（第 1–3 次重跑）

⚠️ **当时承重的产物是 `e156-alloc-basis-counts-2026-09-24-stage4.out`（776 行，R2 第一、二、三段 + 续派两段累计，`replay.sh` 已改指向它）**。它在 `stage2.out`（338 行，2026-09-24，R2 第一、二、三段累计）的基础上，续派第一段追加 Q1 诊断溯源修法、S = 4 几何点、K9 现核（产物 `stage3.out`，682 行，与 `stage2.out` 共有的 329 行「格式没被这两段改动」的行逐字节相同，命令见「复跑」），续派第二段再追加步数对齐对照、20 槽拆解、X8（小池耗尽两口径构造）-A/HY、Q3d（`stage4.out`，与 `stage3.out` 共有的 323 行逐字节相同）。`e156-alloc-basis-counts-2026-09-23-stage1.out`（324 行）、2026-09-22 的三份旧产物（31/35/35 行）原样留着当各自阶段的证据，不再承重；岔路 2 的结论（第 5–8 行）不受这几段改写影响，原样保留在下面。

⚠️⚠️ **`crates/` 在续派第二段收尾时又被改动，这次是大范围、还没做完的改动，实测数值大面积地变了，对 E156（alloc-basis 四条岔路的代价数）跑 `replay.sh` 现在报「对不上」（354 行不同，占全文 776 行近一半）——`git status --short crates/` 显示 `singlefs-core`/`singlefs-checker` 几乎每个源文件都标着 `M`。这不是像修订 10 那次「改了但没碰到 E156（alloc-basis 四条岔路的代价数） 触达的路径」，这次连岔路 7（第一段）的核心读数都变了，「结果」与「这一段做了什么」两节的数字要按「生成产物那一刻的 `crates/`」来读，不能当「现在的 crates/ 会给出的数」**，例如：**HR 家族不再在第 47 步撞墙**（旧读数 `hr_lengths actual_tail=47`，新读数 `actual_tail=72`，跑满了没截断；`legal_state family=HR` 的 `item1/item2/item5` 从 txg=78 起不再单调增长、锁死在 `item1=242 item2=211726 item5=230` 不动，直到 txg=127——这与「一次挂载之内记录只增不减」的既有前提（登记「三」I3（一次挂载分配记录只增不减）、I8）明显不一致，是不是分配记录树的容量/回收机制被改了，这一份不判断，交主 agent）；**`Q7d-2` 的 min_item5 从 1（β0）变成 0（`family=HR kind=rollback_row txg=53`）**——这直接冲击岔路 7 这一整段唯一的正式结论「可达合法状态第 5 项从来不是 0」，如果这份新读数站得住，V3/F16 的前提本身就不成立了；`q7b`（total_l1 167→192）、`q7a_summary`（all_red_count 15→18）、`integrity`（legal_state_rows 168→193）这些汇总数也全部跟着变。**岔路 1（Q1a–Q1d、步数对齐、20 槽分类）与岔路 3（Q3a–Q3d、X8（小池耗尽两口径构造）-A/HY）续派两段量出的全部数字都在受影响之列**（`holes=0` 的 `delta` 从 20 变成 0，`q3e_f_kou` 的失败方式从 `PlacementRefused` 变成一个新增的 `MountError::RaiseFloorSequencePublishFailed` 包装）。**没有任何一段的读数可以确认「不受这次改动影响」**——此前「已经量出并核对过」这类说法，说的是与更早一版 `crates/`（修订 5/10 记的哈希）逐字节核对过，不是与现在这版核对过。这一份不重新生成产物、不改判定，只如实记这条警告；下一步是等这条并发改动落定再重新跑一轮，还是先按哪个版本的读数交用户，交主 agent 定。

**第 3 次重跑第一段（2026-09-25，`e156-r3-prereg.md`，只答问题单第 3 行）**：本页记「对不上」与「min_item5 从 1 变 0」的那两条 ⚠️不是猜测里悬着的问题——这一段在**今天的 `crates/`**（分配记录树已按 D8（核心索引结构） 已定项 14 按位置寻址落地）上重新钉死装置里那三个常量与另外六处，重跑，产物 `e156-alloc-basis-counts-2026-09-25-r3.out`（1076 行，sha256 `1356c1dff9b31cf944516976972d241f7d1ad7df1f77c7c599a785432e5f95cd`）。跑前与跑后各取一次 `crates/` 快照（S3）：两次之间 `crates/mutations.tsv`（并发追加，含这一份自己新增的 6 行）与这个装置文件本身（这一份自己的改动）之外，**没有第三个文件发生变化**——`singlefs-core`/`singlefs-checker`/`singlefs-format` 与其余 harness 文件逐字节未变，产物有效性不受这条并发编辑影响（S3 字面上触发过，核实后不构成停机理由，写明交主 agent 复核）。

「三个常量（与另外六处）改成什么」（第七节 7.2 闭式与命令二独立算出，逐条已在装置上核对成立）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 7 行（name=k1_after_new_pool_file_creation）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 3 行（name=anchor_a_d8）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 6 行（name=anchor_a_d8_root_level）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 9 行（name=s1f_record_count）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 332 行（name=s1ef_summary）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 486 行（name=r5_empty_publish_closed_form）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 522 行（name=s1c_remount）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 524 行（name=q7d1_by_kind）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 630 行（name=r7_hy_cap）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 632 行（name=r7_hy_condition）
```

K1 第 1 项 13→17、β0 走读引用 12→16（`k1_after_new_pool_file_creation` 与 `referenced_slots` 逐字匹配）、S1(c) 隔离槽数 34→54、新池新建文件之后记录条数 20→28：全部按第七节 7.2 的闭式与独立命令算出的值核对成立（`s1ef_summary mismatches=0`：H0 全部 144 步逐次released/记录增量都等于闭式，`steps_by_changed_leaf_count=1:108,2:36` 两个方向——只落 1 片叶与跨 ≥ 2 片叶——都出现过，第八节 8.2 满足）。空发布释放槽数 4→8（`r5_empty_publish_closed_form`）、Q7d-1 覆盖写那一组从字面 `min=10 max=10` 改成逐次现量（`min=14 max=16`）。A-D8：本地叶宽/扇出与 `singlefs_format` 回比一致，本地算出的根层级与实装 `AllocationRecordTreeGeometry` 一致（都是 2），F21 未触发。HY 的覆盖写次数改成搜「e ≥ max(8, f)」：搜了 3、2、1、0 四档都不满足（这一格的开放段在小池 P5 几何上，抬 F 那一刻要么还没建立（cap=0，重开之后 `open_segment` 归零、下一次分配前才重新打开）要么已经消耗殆尽（cap≥1 时小池本身在 1 次覆盖写之后就 `SpaceAdmissionRefused`），装置报 `holds=false`——写进「它答不了的」，这一格没有按登记原意演示出「推空时还留着空间」，交主 agent 判要不要放宽小池几何。

**Q3r.4（岔路 1、3、7 判定变不变，PC-判定器脚本在 `/tmp/claude-1000/e156-r3/pc_judge.py`，`--selftest` 12 条断言全过；产物 `e156-alloc-basis-counts-2026-09-25-r3.out`）**：

```text
岔路     量                stage4 判定                                新产物判定                                    变不变
岔路1    Q1d monotonic    s=4:true,s=8:true                        s=4:false,s=8:true                       变
岔路3    Q3c              差别不大                                     差别不大                                     不变
岔路3    Q3e              hx_两口径一致=True hy_两口径一致=True              hx_两口径一致=True hy_两口径一致=True              不变
岔路3    K9               g7=0 f_kou=0 expected=3                  g7=0 f_kou=0 expected=3                  不变
岔路7    Q7a summary      分得出                                      分得出                                      不变
岔路7    Q7b              不误报                                      不误报                                      不变
岔路7    Q7c              ①②都转色                                    ①②都转色                                    不变
岔路7    Q7d-2            非 0（没有第 5 项为 0 的可达状态）                    =0（可达基底 β_z 存在）                          变
岔路7    F16              触发                                       不触发                                      变
岔路7    Q7e              转色（①）                                    转色（①）                                    不变
```

**问题单第 3 行的答案：变**——岔路 1（Q1d 在 S = 4 上 `q1d_monotonic` 从 `holds=true` 变 `holds=false`，即从「单调」变「不单调」）与岔路 7（`q7d2_min_item5` 从「非 0」变「= 0」、连带 F16 从「触发」变「不触发」）两条岔路的判定翻了面，岔路 3（Q3c、Q3e、K9）在已核的量上不变。**岔路 7 是用户已定案的那一行**（2026-09-23 定「采纳 G27……立成 I-3.11（已分配减 defer 等于最新根走读）」）——这一次的 `q7d2_min_item5=0`（`family=HR kind=rollback_row txg=76`，`item1=18 item2=211950 item5=0 referenced=18 check_is_red=false`，是一个 checker 判绿的合法状态）与用户定案时依据的「可达合法状态第 5 项从来不是 0」这句话矛盾——**用户定案所依的那一格变了，这一份不重开岔路 7，交主 agent**。岔路 1 的翻面：`q1d_monotonic` 在 S = 4 上从 `holds=true`（stage4：三个洞数取样点 20→70→110 单调）变成 `holds=false`（新产物：H0/HR 不再撞墙、Hh(k) 族走的历史更长，相邻两点出现非单调的一跳），第 9 行「随洞的个数怎么长」这句的答案从「单调」变成「不单调，需要看全部点」——这条岔路当时**没有**用户定案（岔路单第 9 行状态仍「开着」），不需要额外交回定案的一格，但正文（若已写成「单调」）要按这份新读数改。

R2 全部段产物收尾行（`grep -n 'name=integrity \|name=done\|name=integrity_r2_segments' e156-alloc-basis-counts-2026-09-24-stage4.out`）：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 323 行（name=integrity）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 1075 行（name=integrity_r2_segments）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 776 行（name=done）
```

分类计数（`grep -o 'name=[a-z0-9_]*' e156-alloc-basis-counts-2026-09-24-stage4.out | sort | uniq -c`）：392 条 `q1_delta_record`（S=8/S=4 各 holes=0/1/2，续派第一段新加的逐槽列举）、177 条 `legal_state`（beta0 1、H0 49、HR 100、HK 18、HK-F0 9）、30 条 `s1d_step`、27 条 `q7a`、10 条 `q1_hh`（S=8/S=4 各 holes=0/1/2）、10 条 `q1_delta_debug`、各 9 条 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least`/`pc_check_bk1`/`pc_check_bk2`/`pc_check_bk3_is_bd_plus_one`/`basis_snapshot`、4 条 `q1_step_matched_diff`（续派第二段）、4 条 `q1d_adjacent_diff`、2 条 `x8a_prefix`/`x8a_held_measure`/`q3e_x8a`/`q3e_g7_remount`（HX、HY 各一条），其余各 1 条或 2 条（`q7a_summary`/`q7b` 等首段量、`q3a_d_rel_g7` 等岔路 3 单格量、`q3d_d_acct_d_alloc`、`anchor_k8`、`k9_precondition`、`q1_geometry_sensitivity_s`）；总计 66 种量名、776 行。

**岔路 2**（第一段，未受这次改写影响）：`|M∖L|=41`（清单漏了 41 处，逐条列在跑前登记「修订」引用的 `/tmp/claude-1000/e156-q2/list-M.txt` 与本报告里，含 `decisions-history` 归档、`milestone/`、`layout/`、多份实验页、`crates/singlefs-core/src/transaction.rs:1488`、`crates/singlefs-harness/src/model.rs:57`、两个测试文件与一份 research 脚本 fixture）；`|L∖M|=0`。门禁 doc-registries 的 admission-terms 格的判别力自证：①红 ②仍红（`03-空间分配.md:82` 的公式副本不跟着改就过不了）③绿；**门禁 doc-registries 的 admission-terms 格脚本本身不需要改**。

**岔路 7（R2 第一段之后：仍不够判，续跑翻不了面，交主 agent 定改判据还是认下）**。R3 改成只读被判镜像之后，逐项判定：

- **Q7b（登记「六」，判据 0）**：`name=q7b red_l1=0 total_l1=167 red_l2=0 total_l2=1`——H0、HR、HK 全部 168 个可达合法状态（L1 167 个、c2 崩溃恢复之后的 L2 1 个）上 G27 零误报。**F5 不触发。**
- **Q7d-1/Q7d-2（新，判据「至少两个第 5 项不同的取样点」「min 值」）**：`name=q7d1_by_kind kind=O min=10 max=10 count=153`、`kind=E min=4 max=4 count=1`——O、E 两种发布自己的释放槽数在全部样本上都是常数（10、4），与 S1(e) 的 assert 一致。`name=q7d2_min_item5 min_item5=1 family=beta0 kind=first_file txg=3`——**168 个可达合法状态里没有一个第 5 项为 0**，最小值 1 出在最早的那个（β0，新池新建文件之后），印证 P21（释放越早等得越久）(g) 的预估：只要发布过一次带记账行的事务，第 5 项就不可能是 0。
- **Q7d-3 / PC-可达（判据必须 = 0）**：`name=q7d3_pc_reachable hk_forced_to_zero_smallest_item5_item5=0 beta_syn_item5=0 holds=true`——**度量方法本身看得见 0**（HK-F0、β_syn 都测到 0），证明 Q7d-2 的「min=1，从未见过 0」不是量法的盲区，是可达状态集合本身的性质。**V11 不触发。**
- **Q7c①②③（登记「十一」V3 原字面）**：①（不减第 5 项）在全部 **7 个可达基底**（β0、β1、β2、βK-w/r/f/l2）上都是 `flips_red_to_green=false`——比第一、二段（只测过 β0/β1/β2 三个）多测了 HK 的四个观测点，结论没有变、证据强了一倍多；只在 **2 个不可达基底**（β_syn、β_F0，第 5 项人为造成 0）上转色。②（`==`→`>=`）在全部 7 个可达基底上都 `flips_red_to_green=true`（`name=q7a_summary all_red_count=15 q7c1_flip_seen=true q7c2_flip_seen=true`，2026-09-24 在今天的 `crates/` 上重跑：`all_red_count` 由昨天的 12 变成 15——差值全部来自 `HK_l2` 那一格的 `base_i31_red` 由 `true` 转 `false`（另有实现员在改 `walk.rs` 的 I-3.1（已分配统计对得上） 实现，见 2026-09-24 记录「先做的一步」那段），`q7c1_flip_seen`/`q7c2_flip_seen` 本身与逐条 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行都逐字节不变）；在 β_syn/β_F0 上**无从执行**（`not_applicable=true reason=deferred_is_zero_cannot_subtract_one`：那两个基底第 5 项已经是 0，Bd(−1) 会把它减成负数，`e156-r2-prereg.md`「十二」修订 3 记了这条发现）。⇒ 按 V3 字面，**Q7a 整张表仍然作废**——①在可达基底上从来看不到转色，根子还是「可达合法状态第 5 项从不为 0」（Q7d-2 已经把这句从「两个基底」的观察扩成「168 个状态、7 个覆盖八种发布种类的基底」全部同一个结论）。

**（2026-09-25 第 3 次重跑续派，`e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`，逐行照抄）**：「可达合法状态第 5 项从来不是 0」这条结论今天被推翻。补一个可达基底 `beta_hr_rollback_row`（HR 家族管理员回退写行那一步，`family=HR kind=rollback_row txg=76`）之后：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 591 行（name=basis_snapshot）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 592 行（name=q7a）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 593 行（name=q7a）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 594 行（name=q7a）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 595 行（name=q7c1_not_subtracting_defer）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 596 行（name=q7c2_threshold_at_least）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 597 行（name=pc_check_bk1）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 598 行（name=pc_check_bk2）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 599 行（name=pc_check_bk3_is_bd_plus_one）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 618 行（name=q7a_summary）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out 第 621 行（name=integrity）
```

`q7c1_not_subtracting_defer` 那一行 `flips_red_to_green=true`——**判别力自证①第一次在一个可达基底上转色**（此前只有不可达的 β_syn 转过色）；`q7c2_threshold_at_least` 与 β_syn/β_F0 同理 `not_applicable`（`deferred==0` 时 Bd(−1) 会把它减成负数，不是两种读法的分歧）。`basis_count` 从 9 变 10、`all_red_count` 从 18 变 20（这个基底 delta=1/8 各命中一次 Q7a 合取，delta=-1 因 `deferred_underflow` 跳过）；`legal_state_rows` 仍是 310（这个基底是对已有 `legal_state` 行的再利用，没有新增走过的历史步）。独立单测 `hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test` 不调用 `main()`，另用同样的公开入口重放同一段真实历史，独立核对到同一个 `item5=0` 与 `flips_red_to_green=true`；核查 agent 的独立探针（`/tmp/claude-1000/e156-fork7-check/probe.out`，第三条互不共享代码的路径）在这个状态上读到 `item1=18 item5=0`、checker 46 条不变量零违例，三条路径一致。**够不够判交主 agent**：这只证明了「存在至少一个可达基底能转色」，没有证明「这是唯一一个」或「这是某个结构性条件的必然结果」。
- **Q7e（新，把「判据在可达状态上没有判别力」与「装置或 G27 写坏了」分开）**：`name=q7e beta_forced_to_zero_flip1=true beta_forced_to_zero_flip2=false beta_syn_flip1=true beta_syn_flip2=false beta_forced_to_zero_i31_red=true beta_forced_to_zero_i52_red=false beta_syn_i31_red=true beta_syn_i52_red=false`。①在两个不可达基底上都转色（**V10 不触发**）；②这一行的 `flip2=false` **读不出「转色失败」还是「不适用」**——本行没有带 `not_applicable` 字段，要去同一份产物里对应基底的 `q7c2_threshold_at_least` 行核对，那两行才写着 `not_applicable=true`。**这是产物本身的一处可读性缺口，交主 agent（下面单列一条）**。`beta_*_i31_red=true` 说明这两个基底为什么不是合法状态：造它们时人为改动了记账字节而没有同步改走读引用能看到的东西，`I-3.1`（已分配统计对得上） 天然判红。
- **Q7f（附带，不解除 V3）**：`name=q7f_count count=7`——**全部 7 个可达基底**上，真实 G27 判绿而「不减第 5 项」的变体判红：G27 比这个特定的坏变体更不容易漏报，但这条不进 V3 的判定，只是留给主 agent 的候选数据（登记「六」Q7f 原话）。
- **S9（停机，不当结果、不作废）**：`name=s9_raise_floor_refused family=HK-F0 error=RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, failure: UnitUnreadable { slot: SlotNumber(50178) } }`——HK-F0 的抬 F 探测上限撞上这个错误：`ForcedToZero` 下 `release()` 会把释放代 ≤ 这次 txg 的全部已释放记录一起回收（`allocator.rs`），genesis 的树表单元在 HK-F0 早期就被这样收走挪作他用，`rollback_floor_ceiling` 判非空要读候选根的树表，读不到就报错——这正是 I-7.4（近 K 代块未被复用） 要挡的那类复用，HK-F0 故意不可达，撞上了停这一格（只影响 HK-F0 后半段：`beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery` 三个字段在 main() 里从没被 HK-F0 引用过，S9 不影响任何已报的判定）。

⚠️ **交主 agent 的两点**：① Q7e 输出行本身应当带 `not_applicable` 标记而不是把「不适用」压成 `false`，同「基底已红」的 PC-检查行一样（登记 5.5 footnote 已经预见了「基底已红不算失败也不算通过」这条，这里同一类问题换了个地方冒出来）；② V3/F16 已经确认到「7 个可达基底、168 个可达状态，Q7c① 从没转过色」，续跑不会再翻面——改判据（不再要求红→绿的严格翻转）还是认下（「G27 的判别力由 Q7b 加上坏镜像那一格干净对照立住」），交主 agent 判。

**岔路 3（R2 第二段，2026-09-24，一个几何格：S = 8、ρ = 1，`run_hf_single_cell`）**：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 614 行（name=v13_hf_prefix_identical）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 615 行（name=q3a_d_rel_g7）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 616 行（name=q3b_d_rel_f_kou）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 617 行（name=q3c_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 618 行（name=k9_newest_batch_d_rel_non_empty）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out 第 330 行（name=q3e_x8a）
```

- **V13 holds=true**：抬 F 之前，探上限 / F-扣 / G7 三条分支各自从同一份快照重建，逐字节相同——两条口径的历史前缀确实一样，Q3a/Q3b/Q3c 不因为「起点就不同」而失去可比性。
- **Q3a（G7）与 Q3b（F-扣）**：同一批被抬 F 处置放开的槽（释放代 = 13，抬 F 之前设备 0 上最大的已释放 generation）在两个口径上「分配器真的能把它发出去」的时刻**完全相同**（`dispensable_txg=16`，两条口径推空发布次数都是 3）——`d_rel_非空 = 0`（16 与 13 之间没有非空发布，只有推空的空发布）、`d_rel_全 = 2`。**这不是巧合**：G7 与 F-扣 在这一格里推空发布的次数只由「哪块盘还没盖到带新 F 的根」决定，与谁先回收无关，两条口径因此在同一段历史上推出同样长的空发布序列，最后一次推空发布的 txg 自然相同。
- **Q3c（两个口径的差）**：`max_diff=0 ≤ 3` ⇒ **差别不大**——两个口径各一个数、差是 0，第 11 行「够判条件」的第一半满足。⚠️ **只在这一个几何格上量到**，没有跑登记「五、5.2」HF 的 18 格全扫（S × ρ × φ），也没有验证「差别不大」在别的 φ（抬 F 起点相位）或别的 S 上是否仍然一致——不许外推成「两个口径在所有几何上都差别不大」。
- **K9（新，released 记录唯一那一批的 D_rel_非空）**：`g7=0 f_kou=0`，**都不是登记预期的 3**。K9 的前提（抬 F 那一刻环里 ≥ 4 条非空有效根）**续派第一段已经现核**（不再是没查过）：`name=k9_precondition non_empty_valid_root_count=9 threshold=4 holds=true`——前提成立，「= 3」不成立不是因为前提没满足，而是这一批（release_generation=13, dispensable_txg=16）之间的 3 次持久发布恰好都是抬 F 自己的推空发布，本来就不算「非空」，与 P21（释放越早等得越久）(e)「越早释放等得越久」是同一方向的现象。K9 只报数，不作为判据，交主 agent 决定是否需要专门构造别的几何来验它的方向性。
- **Q3e（X8（小池耗尽两口径构造）-A）与 Q3d：续派第二段做出来了**，见「R2 续派第二段」小节，不再是 `status=not_done`。

**R2 续派第一段（2026-09-24，同日，s4 执行员，产物 `…-stage3.out`）新增**：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 641 行（name=anchor_k8）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out 第 523 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out 第 581 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out 第 671 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 698 行（name=q1_geometry_sensitivity_s）
```

S = 4（方向与 mkfs 默认 8 相反）上 Δ 同样单调不减（20 → 70 → 110），「越过 2 槽」这条判定与 S = 8 一致（`stable=true`）——S = 4 的 `h_missing` 在 holes=1/2 都停在 8（不像 S = 8 那样 8 → 14），`delta` 在 holes=2 也是 110 不是 120：环更浅时洞的计数方式与多扣的绝对量确实随几何变，但方向性判定没有变。

**R2 续派第二段（2026-09-24，同日，主 agent 续派，产物 `…-stage4.out`）新增**：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 771 行（name=q1_step_matched_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 772 行（name=q1_step_matched_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 773 行（name=q1_step_matched_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 774 行（name=q1_step_matched_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 331 行（name=x8a_prefix）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 332 行（name=x8a_held_measure）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 333 行（name=q3e_f_kou）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 336 行（name=q3e_g7）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 339 行（name=q3e_x8a）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 340 行（name=x8a_prefix）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 634 行（name=q3e_f_kou）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 637 行（name=q3e_g7）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out 第 348 行（name=q3e_x8a）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out 第 619 行（name=q3d_d_acct_d_alloc）
```

- **步数对齐对照（岔路 1 够判条件里「随洞的个数怎么长」这句现在有了干净的对照）**：`Δ(0, 步数对齐到 holes=k)` 在全部四组（S=8/S=4 × k=1/2）里**都还是 20**，与 `holes=0`（未对齐）完全一样——同样的总发布步数、只是不造洞，Δ 不涨。`Δ(k) − Δ(0, 步数对齐)` = 50（k=1，两个 S）、100/90（k=2，S=8/S=4）：**这才是洞本身贡献的多扣，之前的 20→70→120 曲线，增长部分不是步数带来的**。
- **`holes=0` 那 20 槽的分类（上一段「没有查透」这条现在有答案了）**：`q1_delta_record`（S=8/S=4 两个几何点逐槽一致）显示只有两类、各 8 条各 10 槽——分配代 4→释放代 5、分配代 5→释放代 6。机制：ring-fill 循环停在某个 txg 时环里最旧有效根还没推进到那两个释放代，紧接着的「最后一次无崩溃重开」用当时的 floor 回收，但重开自己的写行 + 暖机又把环推进 2 个 txg，使这两批释放留在「已回收但比当时 floor 新」的状态——与洞完全无关，S=8/S=4 逐槽相同（读代码得出的机制，不是新读出的独立事实，详见 `e156-r2-prereg.md`「十二」修订 12）。
- **Q3e（X8（小池耗尽两口径构造）-A/HY）真实构造出来了**：HX（填到真耗尽）上 F-扣 与 G7 **都**卡在同一步（`AccountingTree` 分配不到落点）、同一个原因；HY（阳性对照，提前停手）上两个口径**都**成功——确认 HX 的失败是耗尽导致，不是装置的问题。单独测过「回收并扣住」这一步：`free_slots` 从 5 涨到 86 但 `lowest_empty_segment` 仍是 `none`（P5「空闲计数报着那些槽」逐字应验，但这段真实历史比 P5 的简化模型更混乱——两个聚簇段都混着仍分配与被扣住的槽，不是干净的「一段活一段空」）。**第 11 行「X8（小池耗尽两口径构造）-A 那一格两者各卡在哪」现在有答案**：两者卡在同一步，不是 P5 简化模型暗示的「扣住失败、不扣住成功」那种更尖锐的差别。
- **Q3d**：`D_acct(F-扣)=0`、`D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=2`——从代码已读的调用顺序推出来的（F-扣 的回收在第一次推空之前、两个口径「能发」都在推空全部结束之后），不是逐次持久根现测的，据实标注方法限度。

**岔路 1（R2 第三段，2026-09-24，一个几何格：S = 8、ρ = 1，位置固定「后」，`run_hh_cell`）**：

```text
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 330 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 331 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 332 行（name=q1_hh）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 333 行（name=q1d_adjacent_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 334 行（name=q1d_adjacent_diff）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 335 行（name=q1d_first_crossing）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out 第 336 行（name=q1d_monotonic）
```

- **Q1a / Q1b / Δ 随洞数怎么长**：三个取样点（k = 0、1、2）上 Δ 分别是 20、70、120，**单调增、每加一个洞恰好 +50**（`q1d_monotonic holds=true`）；h_缺 同步单调增（0 → 8 → 14）。**第 9 行「够判条件」逐字满足**：至少两个洞数不同的取样点各有 Δ，且报得出越过 2 槽 / 1 槽两个门槛的规模——**在这三个取样点上，Δ 从 h = 0 起就已经越过两个门槛**（`q1d_first_crossing` 两栏都指向 `holes=0 txg=29 delta=20`）。
- **h = 0 时 Δ = 20 的来源，已在续派两段查清（不再是「没有查透」）**：一是诊断代码的一处过滤漏洞（不改变 Δ 的数值，见「这一段做了什么」第 14 条）；二是这 20 槽本身是「一次挂载只回收一次、重开自己还要再推进 2 个 txg」这条实现事实造成的、与洞无关的基准量（见续派第二段的 20 槽分类）；三是步数对齐对照证明「+50/洞」的边际效应确实是洞本身的贡献、不是步数的贡献。**读这三个数时**：20 是与洞无关的基准（S=8/S=4 逐槽一致），50/洞（k=1）与 100(S=8)/90(S=4)/洞（k=2）才是「洞上多扣的槽数」——第 9 行「洞上多扣的槽数」这句问的正是后者，不是包含基准值的聚合曲线。
- **续派第一段补了 S = 4，其余几何维仍未扫**：S = 16、ρ = 1/4、回收时点「每」、洞位置「前」、k = 4 都没有跑，第三段 a/b 的完整 24+14 条历史仍未做；登记「五、5.6」第六类要求的其余几维（回收时点、洞位置）没有取到方向相反的第二个点。

#### 复跑（第 1–3 次重跑）

```
bash scripts/experiment-apparatus/replay.sh E156
```

当时报「字节一致」（`replay.sh` 的 `driver_e156` 直接 `cargo run -q -p singlefs-checker-tier --bin e156_allocation_basis_counts`，先例同 E142（新池新建文件的干跑），两个 cargo workspace 互相看不到对方，不能合并成一次调用；确定性——同一个二进制跑两遍逐字节一致，V8 未触发，续派两段各自现跑过；产物换到 `…-stage4.out`，sha256 `5311e01a0cec4797bf848ffacbbf3f624b6a01627412491c76a8c92f2cea0f02`）。单测：`cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `6 passed`（续派第一段新增 1 条：`reclaimed_records_are_not_removed_but_their_slots_become_free`，钉住 Q1 诊断修法的前提；此前 5 条不变）。变异：`crates/mutations.tsv` 对这个 bin 累计仍是 4 条（续派两段没有新增变异行——新加的诊断/构造代码要么不改变任何已判的量、要么是新的产品行为组合，不是候选/判据分歧，加变异的收益与工作量都没到位，逐条理由见「它答不了的」），逐条手工改坏、`cargo test` 判红、还原后再判绿（未跑门禁 checker-tier-crates-mutation-replay 整张表，归 `gate-triage`）。**手工验证过一条没红的变异**（G7 重组里 `ReclaimedReuse::Immediately` 改成 `HeldUntilFloorTakesEffect`）：当前的 Q3a/Q3b/Q3c/K9 输出逐字节不变，按 `mutation-sampling.md`「三类」判为**第一类：真盲区**——`run_hf_single_cell` 那条历史在回收之后没有再做一次分配尝试；**X8A（续派第二段）从另一个角度补了这条盲区的证据**：`x8a_held_measure` 直接量出「回收但扣住」之后 `free_slots` 涨了、`lowest_empty_segment` 却仍是 `none`，`q3e_f_kou`/`q3e_g7` 也确实随两种回收方式而表现不同（HX 上二者都失败但是不同代码路径导致），但这仍不是给 `run_hf_single_cell` 那条历史加的变异，两处是两个独立的观测点，不进 `crates/mutations.tsv`。

**第 3 次重跑之后（现状）**：`replay.sh` 报「字节一致」（产物换到 `…-2026-09-25-r3.out`，sha256 `1356c1dff9b31cf944516976972d241f7d1ad7df1f77c7c599a785432e5f95cd`）。单测：`cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `10 passed`（新增 4 条：`pc_closed_form_matches_the_registered_anchor_r3c`、`pc_closed_form_third_group_counts_released_and_allocated_records`、`overwrite_steps_match_the_closed_form_and_cross_a_second_leaf`（U10/U12）、`rollback_isolation_scenario_matches_the_new_layout`（U11）；此前 6 条改了断言里的绝对值（13→17、12→16、34→54）但函数与场景不变）。变异：`crates/mutations.tsv` 对这个 bin 累计 10 条（本轮新增 M27–M32 六条：R-3 闭式漏根/丢 devices 因子/只取分配不取释放、Q7d-1 退回字面 10、K1-1 退回 13、S1(c) 退回 34），逐条手工改坏、`cargo test` 判红、还原后再判绿；此前 4 条的锚点核过在改完的源码里仍唯一命中，函数体未改，不必重新逐条证红。全表复跑仍归门禁 checker-tier-crates-mutation-replay（`gate-triage`）。

**第 3 次重跑续派之后（现状）**：`replay.sh` 报「字节一致」（产物换到 `…-2026-09-25-fork7-selfproof.out`，sha256 `59b1be11c762c057a5260d412da371ee89138d991aae64352bbd88cb793b0b2a`；两次直接 `cargo run` 逐字节一致，见「历史版本」）。单测：`cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `11 passed`（新增 1 条：`hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`；此前 10 条断言与场景不变，`naming-lint.sh` 对新代码零违规）。变异：`crates/mutations.tsv` 对这个 bin 累计 12 条（本轮新增 M33/M34：`q7c_self_test` 里 `without_subtracting_defer_is_red`/`real_check_red_plus1` 的比较符号各改坏一次），逐条手工改坏、`cargo test` 单独跑新测试判红、还原后再判绿；此前 10 条的锚点核过在改完的源码里仍逐条唯一命中（`python3` 用 `content.count(original)` 核过 12 条全部原文，见交回报告）。全表复跑仍归门禁 checker-tier-crates-mutation-replay（`gate-triage`）。
#### 它答不了的（第 1–3 次重跑）

- **S1(b)（`reuse_after_raising_the_floor.rs` 两个用例）仍没有装置侧重放**：只现跑了测试文件本身，不像 S1(c) 那样在装置里重新走一遍那个场景再逐项 `assert_eq!`。
- **S1(d) 只核了 G27 与 `check_pool_image` 两条检查在 H0 前 30 步（3S+6）逐步的判定，没有核第三条独立算法**：`s1d_step` 那 30 行只报 `item1/item2/item5`（镜像读法）与今天两条检查的判定，三者之间没有第三条完全独立的路径互证——`item1/item2/item5` 与 G27 用的是同一次 `mirror_accounting_row_slots` 读法。
- **（2026-09-25 前的结论，续派已推翻）岔路 7 的判别力自证①在可达状态上做不出转色**：这句曾经的结论已被 2026-09-25 续派段推翻——第 3 次重跑第一段的 `q7d2_min_item5` 第一次在可达状态上读到 `min_item5=0`（`family=HR kind=rollback_row txg=76`），续派段把这一步捕成第 10 个基底 `beta_hr_rollback_row` 并入既有循环，`q7c1_not_subtracting_defer basis=beta_hr_rollback_row … flips_red_to_green=true`——判别力自证①第一次在**可达**基底上转色，不再是只有不可达的 β_syn 才转色。够不够判交主 agent，见「结果」续派段那一条与本页顶部标题。
- **续派段仍答不了的**：这一个可达基底是不是唯一一个第 5 项恰为 0 的可达状态，没有扫过全部可能几何与历史长度去确认（只在这一条 HR 历史、这一个取样点上发现）；这一格是不是「巧合」还是「回退到候选集最旧根、且候选集刚好转满一圈」这个结构性条件的必然结果，没有另外论证。
- **Q7e 那一行的 `flip2=false` 读不出「转色失败」还是「不适用」**：字段本身没带 `not_applicable` 标记，要跨行去对应基底的 `q7c2_threshold_at_least` 输出核对——这是产物的一处可读性缺口，不是判定错，见「结果」末尾那条给主 agent 的提醒。
- **H0/HR 的实际长度被 `AllocationRecordsExceedOneNode` 收窄到 49/47 步，不是登记的 144/72+72**：这是装置在今天这份 `crates/` 上量到的真实边界，不是我选的参数。Hh(k) 族避开了这堵墙（每格只跑到 txg 29/35/41 或更小，远没到 812 条记录的墙），但**没有跑登记「五、5.2」要的那种连续 6N 步长历史**，撞不撞墙、撞墙之后的规模没有另外验证。
- **HK-F0 撞上 S9（抬 F 探测上限时候选根树表读不出）之后停在那一格**：`beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery`（HK-F0 那三个观测点）没有真正跑出来，main() 里也从未引用它们，不影响任何已报的判定，但 HK-F0 家族「每一种发布至少出现一次」这条覆盖到抬 F 之后就停了。
- **旁路评估的射程**（登记「十一」）：G12 只在真实历史旁边做谓词评估，不改分配器行为，答不了「按它跑出来的历史会长成什么样」；G7 这一份已经改成用公开入口重组真实历史（R4），不再是纯旁路评估，但仍不改真实分配器的产品路径。
- **岔路 1：S = 16、ρ = 1/4、回收时点「每」、洞位置「前」、k = 4 都没跑**：第三段 a/b 的完整 24+14 条历史仍未做；`Q1c-洞`（把 h_缺 = 0 与 h_缺 ≥ 1 分开报）、`Q1e`（附带）、PC-多扣、PC-洞（阳性对照）都没做；M9 的周期重开表没有实现（这一段的 Hh 族只在最后做一次无崩溃重开，与登记「每 S 次工作负载发布重开一次」不同，见「这一段做了什么」第 13 条与本页「历史版本」）。
- **岔路 3：HF 的 S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} 共 18 格里 17 格没跑**：Q3a–Q3d、K9 只在一个几何格（S=8、ρ=1）上量出；X8（小池耗尽两口径构造）-A/HY 用的是另一个独立小池（P5 几何），只跑了一个「填到耗尽」与一个「提前停手」的取样点，没有扫过介于两者之间的中间态（比如「刚好差 1 槽放不下」那种边界）。
- **Q3d 是推出来的，不是逐次持久根现测的**：`q3d_derived` 的方法限度见「这一段做了什么」第 19 条，更强的验证方法（逐次持久根现读记账行）没有做。
- **X8（小池耗尽两口径构造）-A 在 P5 的简化模型与 2026-09-24 续派第二段的真实历史构造上不完全对应**：P5 原本的对比是「扣住失败、不扣住成功」，2026-09-24 续派第二段的两个真实口径（F-扣 扣住、G7 从不扣住）在完全耗尽的池上都失败、卡在同一步——「不扣住就成功」这个 P5 式的更尖锐对照，这两个真实口径都没有走到（G7 是「从不扣住」但也「从不提前回收」，不是 P5 那种「回收但不扣住」）；有没有必要另建一个「回收但不扣住」的第三口径去对齐 P5 的原始模型，2026-09-24 续派第二段没有做，交主 agent。
- **G7 重组「立刻可发」与「回收但扣住」的差别，作为一条变异仍是真盲区**（`mutation-sampling.md` 第一类）：`run_hf_single_cell` 的测法在回收之后没有再做一次分配尝试，看不出这个差别；X8（小池耗尽两口径构造）-A 的 `x8a_held_measure` 从另一个角度量到了这个差别的效应（`free_slots` 涨但 `lowest_empty_segment` 仍是 `none`），但两处是独立的观测点，没有把这条盲区补进 `crates/mutations.tsv`。
- **门禁 checker-tier-crates-mutation-replay（`crates/mutations.tsv` 整张表复跑）未跑**：这个 bin 累计的 10 条已逐条手工验证红→绿，全表复跑归 `gate-triage` 统一跑。
- **第 3 次重跑：HY 没有演示出登记原意「推空时开放段还留着空间」**：登记要求覆盖写次数不满足 `e ≥ max(8, f)` 时「减少覆盖写次数直到满足」，这一段搜了 3、2、1、0 四档，`open_segment_free_slots` 在全部四档上都是 0（cap=0 时重开之后 `open_segment` 归零、还没有下一次分配把它重新打开；cap ≥ 1 时 P5 这个小池几何本身在 1 次覆盖写之后就 `SpaceAdmissionRefused`，段早被填满）——HY 在这个小池几何上搜不出满足条件的档位，登记的「阳性对照：两个口径在 HY 上都不该卡」这句话没有被兑现出「留着空间」这个前提，交主 agent 判要不要放宽 P5 的小池几何（比如把单元区从 128 槽调大到能装得下 8+ 槽的开放段余量）再补跑这一格。
- **Q3r.5（附带，够判后不跑）没有跑**：把「实二一之前的并发改动」与「HEAD 之后的改动（含实二一）」分开归因，这一段不做——第 3 行已经够判，登记「五」5.4 把它标成「够判后主 agent 判要不要跑」。
- **Q3r.4 只覆盖了 10 个量，不是「六」列出的全部量**：`岔路 1` 只核了 `Q1d monotonic`；`岔路 3` 核了 `Q3c`/`Q3e`/`K9`；`岔路 7` 核了 `Q7a 汇总`/`Q7b`/`Q7c`/`Q7d-2`/`F16`/`Q7e`。`Q1a`/`Q1b`/`Q1c`/`Q1c-洞`/`Q3a`/`Q3b`/`Q3d`/`Q7f` 这几个量的原始数字在「结果」里逐行照抄了，但没有单独写进 `pc_judge.py` 的判定表——已经足以支持「变/不变」的整体结论（每条岔路只要有一行「变」就整条翻面），逐量补全交主 agent 判要不要续跑。

#### 影响的决策（第 3 次重跑续派时的回看，原样）

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-09-25 不受影响：只补编号简称与改指代，结论、数与产物不变 |
| D5（快照 / 空间记账机制） 已定项 4 | 备料 | 2026-09-25 不受影响：只补编号简称与改指代，结论、数与产物不变 |
| D16（发布语义） 已定项 1 | 备料 | 2026-09-25不受影响：关系仍是备料——续派（补可达基底 beta_hr_rollback_row）的读数动到这一分项管的机制本身，`q7d2_min_item5=0`（`family=HR kind=rollback_row txg=76`）让判别力自证①第一次在可达基底上转色（`flips_red_to_green=true`），与这一分项「非空」那段叙述、以及岔路 7 用户定案所依的「可达合法状态第 5 项从来不是 0」都不再站得住；`.claude/kb/decisions/16-发布语义.md` 依据段现查这个实验号仍是零命中，关系维持备料不自升；该不该把这条并入这一分项的重议清单，交主 agent 判（此前 2026-09-25 不受影响：只补编号简称与改指代，结论、数与产物不变） |
| D3（空间分配） 已定项 7 | 备料 | 2026-09-25 不受影响：只补编号简称与改指代，结论、数与产物不变 |
| D8（核心索引结构） 已定项 14 | 备料 | 2026-09-25 不受影响：只补编号简称与改指代，结论、数与产物不变 |

### 2026-09-23

- 撤回「独立于 Q7a 的一格干净信号」那一段。原文：**独立于 Q7a 的一格干净信号**（`stage3.out` 的 `name=q7a basis=beta0_k1` 那一行，字段语义修正之后才读得对）：基底上 G27 成立、`I-3.1`（已分配统计对得上） 与 `I-5.2`（空闲统计对得上） 都判绿；坏镜像（Bd(+1)）上 **G27 不成立（判红）而那两条仍判绿**。⇒ G27 在这一格分得出差别、而今天那两条分不出，且 G27 在基底上不误报。这一格不进 Q7a 的聚合（那张表已按 V3 作废），单独可读。
- 撤回依据：重跑登记的设计员现查、主 agent 复核 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` 的 `run_q7a_cell`——`corrupted_check_holds = allocated == deferred + referenced`，其中 `(allocated, _free, deferred)` 取自调用方传入的 `corrupted_allocator_row`，不读 `corrupted_pool`；基底那一侧的 `base_check_holds` 读的是内存分配器。G27 在第一段里一次都没读过盘上的记账行。

**（同日，R2 第一段，执行员）** 装置改写：R3（G27 改读镜像）、新增 HK/HK-F0 两族历史、9 个基底、Q7d/Q7e/Q7f、S1(c) 完整重放、锚1/锚2/锚3 锚点。结论：岔路 7（岔路单第 12 行）仍不够判，且这一次的证据面比第一、二段宽得多（7 个可达基底、168 个可达状态，横跨 H0/HR/HK 三族、八种发布种类）——Q7c①在可达状态上从来看不到转色，Q7d-2/Q7d-3 排除了「量法本身看不见 0」这条解释，剩下的只有「可达合法状态第 5 项从来不是 0」。续跑这套自证设计翻不了面，改判据还是认下，交主 agent（见「结果」与本页顶的岔路表）。产物换成 `e156-alloc-basis-counts-2026-09-23-stage1.out`（324 行），`replay.sh` 已改指向它；旧的 `stage1/2/3.out`（第一、二段）原样保留，不再承重。

### 2026-09-24

**（R2 第二、三段，执行员）** 先做的一步：`crates/` 在 R2 第一段之后被另一条线改动（`mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs` 五个文件的 `git hash-object` 都变了），在今天的代码上重跑 R2 第一段并与昨天的产物逐字节比对——只有 `HK_l2` 基底的 `I-3.1`（已分配统计对得上） 判定由红转绿（另有实现员在改 `walk.rs` 的 checker 实现）这一处差异，Q7c 的读数与岔路 7 的判定不变，详见「这一段做了什么」与「结果」。

装置改写：新增 `run_hf_single_cell`（岔路 3）、`run_hh_cell`（岔路 1）两个函数，`raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect`（G7 用公开入口重组）、`snapshot_allocated_generations`/`record_release_generations`（岔路 1 的分配代账本，G12 需要的那 8 字节真实记录里没有，装置自己维护）。**两条岔路都只跑了一个几何格**（S = 8、ρ = 1），不是登记「五、5.2」的全量取样——理由是这一份执行员的时间预算：岔路单第 3 行要「两个口径各一个数 + X8（小池耗尽两口径构造）-A 两者各卡在哪」，这一格已经给出前一半（D_rel 相同、差别不大）；岔路单第 1 行要「至少两个洞数不同的取样点」，三个取样点（k = 0、1、2）已经给出、且越过了两个门槛。结论：

- **岔路 3（第 11 行）**：一个几何格上 G7 与 F-扣 的 D_rel 完全相同（都是 0，非空发布计），Q3c 判「差别不大」；K9 没有验到预期的 3（前提「环里 ≥ 4 条非空有效根」没有专门核过，也可能是 P21（释放越早等得越久）(e) 已预估的方向不成立）；Q3e（X8（小池耗尽两口径构造）-A）完全没做。**第 11 行部分够判**：两个口径各一个数拿到了，X8（小池耗尽两口径构造）-A 那一半没有。
- **岔路 1（第 9 行）**：三个洞数取样点上 Δ 单调 +50/洞，从 h = 0 起就越过 2 槽 / 1 槽两个门槛；但 h = 0 时 Δ 已经是 20，这个基线值本身没有查透（只做到聚合数级别，没有逐槽拆解）。**第 9 行的两个够判条件字面满足**（至少两个取样点、报得出越过门槛的规模），但只在一个几何格上、基线机制不明，交主 agent 判是否已经够判到可以交用户，还是要先查清 h=0 的 20 从哪来。

产物换成 `e156-alloc-basis-counts-2026-09-24-stage2.out`（338 行，累计第一、二、三段），`replay.sh` 已改指向它；2026-09-23 的 `stage1.out`（324 行）原样保留（今天在最新 `crates/` 上重跑一份新的、与它只差 HK_l2 那 3 处，未另存文件，差异写进「这一段做了什么」）。单测由 4 条增到 5 条，变异表由 3 条增到 4 条，另手工验证了一条不会红的变异（G7 的「立刻可发」标记改成「回收但扣住」，见「复跑」），按 `mutation-sampling.md` 判为第一类真盲区，不进 `crates/mutations.tsv`。

**（同日，R2 续派第一段，s4 执行员，接续一个撞了会话限额而中断的执行员）** 开工先核现场：交接摘要 `/tmp/claude-1000/handover/ab85369a718bfa7f4.md` 记的后台任务已经跑完（留在 `/tmp` 的 `probe1.out` 与产物同值，未落盘）。先按派发要求在今天的主工作区上重跑已有产物、与并发补丁核对：`crates/` 的 7 个 S3 跟踪文件被另一条并发会话改过，把 `stage2.out` 里格式没变的 329 行与今天重跑的同名行逐行比对，逐字节相同——并发补丁没有改到 E156（alloc-basis 四条岔路的代价数） 这几条历史触达的代码路径（结论详见「这一段做了什么」第 17 条）。查清 `holes=0` 时 Δ = 20 的来源是装置诊断代码自己的一处过滤漏洞（已修，不改变任何已报的 Δ 值），给岔路 1 补 S = 4 第二个几何取样点，给岔路 3 的 K9 补前提现核。产物换成 `e156-alloc-basis-counts-2026-09-24-stage3.out`（682 行），`replay.sh` 已改指向它；`stage2.out` 原样保留。单测由 5 条增到 6 条。

**（同日，R2 续派第二段，执行员，主 agent 就 E156（alloc-basis 四条岔路的代价数） 传消息「接着做这一段，岔路表里还开着两行」）** 给岔路 1 补步数对齐对照（`matched_to_holes` 参数）：k=0 但工作负载步数补到与 holes=k 相同、不造洞，测出 `Δ(0, 步数对齐)` 恒为 20——证明之前报的 20→70→120 曲线，增长部分是洞本身贡献的，不是步数带来的；顺带从 `q1_delta_record` 拆解出 `holes=0` 那 20 槽的构成（两类各 8 条各 10 槽）与机制。给岔路 3 构造出 Q3e（X8（小池耗尽两口径构造）-A/HY）：新建 P5 的 128 槽两段小池，两个真实口径（F-扣、新增可失败版 G7 重组）在填到耗尽的 HX 上都卡在同一步（`AccountingTree` 分配不到落点），在提前停手的 HY 上都成功——确认 HX 的失败是耗尽导致；一并做出 Q3d（推导法，见方法限度）。**结论**：岔路 1（第 9 行）与岔路 3（第 11 行）现在都有了比上一段更完整的证据（岔路 1 补了几何点与步数对照，岔路 3 补齐了 Q3a–Q3e 与 Q3d），但都还不是登记「五、5.2」的全量扫描，够不够判交主 agent 看「岔路表」定。产物换成 `e156-alloc-basis-counts-2026-09-24-stage4.out`（776 行），`replay.sh` 已改指向它；`stage3.out` 原样保留。单测仍是 6 条，变异表仍是 4 条（新增代码没有配套新变异行，理由见「复跑」）。

### 2026-09-25

**（第 3 次重跑第一段，执行员，`e156-r3-prereg.md`，只答问题单第 3 行）** 分配记录树在这之间已按 D8（核心索引结构） 已定项 14 落地成按位置寻址（叶宽 812、内部扇出 169、按盘分流的共享根），装置里钉着旧布局（单节点分配记录树）的三个常量（均以 E156（alloc-basis 四条岔路的代价数） 为前缀：`OVERWRITE_EXPECTED_RELEASED_SLOTS`=10、`OVERWRITE_EXPECTED_RECORD_DELTA`=16、`NEW_POOL_FILE_CREATION_EXPECTED_RECORD_COUNT`=20）与另外六处（K1 第 1 项 13、β0 走读引用 12、空发布释放 4、S1(c) 隔离槽数 34、Q7d-1 字面 `min=10 max=10`、HY「≥ 8」）不再成立。改法：两个本地常量（叶宽、扇出）抄自 kb 登记位、加一条与 `singlefs_format` 同名常量回比的断言；新写一条闭式（`e156_allocation_record_tree_new_node_count` / `_replaced_node_count`：一次发布改写的分配记录树节点数 = `devices × 叶数 + devices × 层级 1 数 + 1`，叶/层级 1 位置从这次发布真实翻动的记录槽号现算，不调用 `crates/singlefs-core::allocation_record_tree` 的任何几何函数），S1(e)(f) 的常量断言换成 H0 全部 144 步逐次核对（观测型，不 panic，汇总 `s1ef_summary`）；K1-1 改成 17/1、β0 走读引用 16、S1(c) 隔离槽数 54（都由独立 Python 命令 `anchors_e156_r3.py` 与装置内单测双重核对）；HY 的覆盖写次数改成搜索满足 `e ≥ max(8, f)` 的档位（`e156_find_hy_cap_satisfying_open_segment_condition`），这一次搜到 0（未满足，见「它答不了的」）。新增单测 4 条（PC-闭式两条、U10/U12 一条、U11 一条），新增变异 6 条（M27–M32，crates/mutations.tsv 末尾），全部逐条手工改坏验红、还原验绿。跑前与跑后各取一次 `crates/` 全量快照（S3）：两次之间只有 `crates/mutations.tsv`（并发追加）与这个装置文件本身在变化，其余源码文件逐字节未变，S3 字面触发但核实后不构成停机理由。

**结果（问题单第 3 行：变）**：分配记录树按位置寻址之后，H0/HR 不再撞 `AllocationRecordsExceedOneNode`、跑满登记要求的全部步数（144/72+72），比 stage4.out 的 49/47+47（撞墙截断）长得多。用 PC-判定器脚本（`/tmp/claude-1000/e156-r3/pc_judge.py`，`--selftest` 12 条自测全过）比对新产物（`…-2026-09-25-r3.out`，1076 行）与 stage4.out：**岔路 1**（`q1d_monotonic` 在 S=4 上从 `true` 变 `false`，H0/HR 跑满之后 Hh(k) 族的相邻两点出现非单调的一跳）与**岔路 7**（`q7d2_min_item5` 从「非 0」（stage4：`min_item5=1 family=beta0`）变「= 0」（新产物：`min_item5=0 family=HR kind=rollback_row txg=76`，是一个 checker 判绿的合法状态），连带 F16 从「触发」变「不触发」）判定翻了面；**岔路 3**（Q3c、Q3e、K9）在已核的量上不变。岔路 7 是用户已定案的一行（2026-09-23，依据正是「可达合法状态第 5 项从来不是 0」）——**用户定案所依的那一格变了，不重开这条岔路，交主 agent**；岔路 1 当时没有用户定案（第 9 行仍「开着」），正文若已写「单调」的地方要按这份新读数改。跑前登记「五」5.4 的够判条件（第七节锚点全对、Q3r.1/Q3r.2 成立、PC-判定器 ①② 过、Q3r.4 表每一行都有两边的判定）第一段已满足，第二段（Q3r.5，实二一之前的并发改动与 HEAD 之后的改动分开归因）够判后未跑，交主 agent 定续不续。

**（第 3 次重跑续派，执行员，`e156-r3-prereg.md`「十二」修订 4，回答岔路单第 12 行 G27 判别力自证的判决打中）** 上一段产物的 `q7d2_min_item5` 第一次在可达状态上读到 `min_item5=0`（`family=HR kind=rollback_row txg=76`）——此前 7 个可达基底（β0/β1/β2/βK-w/r/f/l2）第 5 项最小是 1，是岔路单第 12 行「用户已定案（2026-09-23）」所依的那句「可达合法状态第 5 项从来不是 0」的对象。一名核查 agent 另用独立探针（journal 记录逐条施加、在写行发布那一刻截断、直接跑 `singlefs-checker`）二次核对同一状态，`/tmp/claude-1000/e156-fork7-check/probe.out`：`through_row_root`/`through_row_publish_tail` 两个镜像切点上 `item1=18 item5=0`，checker 46 条不变量零违例，I-3.1（已分配统计对得上）、I-5.2（空闲统计对得上）、I-3.11（已分配减 defer 等于最新根走读） 均判 `Holds`，两条路径读到同一个数。判据不改，逐字沿用岔路单第 12 行原判据：把这一步捕成第 10 个基底 `beta_hr_rollback_row`（`main()` 里 HR 家族管理员回退那一步的写行结果），并入既有 `bases` 数组（9→10，`basis_count` 随之改），让已有的 `q7c_self_test`/`run_q7a_cell`/`run_pc_check` 循环照原样再跑一遍——不新写判定逻辑、不改 `q7c_self_test` 的公式。核心结论：**Q7c① 第一次在一个可达基底上转色**（`name=q7c1_not_subtracting_defer basis=beta_hr_rollback_row k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）——此前只有不可达的 β_syn（人为把第 5 项减到 0）转过色；`name=q7c2_threshold_at_least basis=beta_hr_rollback_row k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one`（Q7c② 在这个基底上不适用，同 β_syn/β_F0 一样，Bd(−1) 会把第 5 项减成负数）；`q7a_summary` 的 `all_red_count` 从 18 变 20（该基底 delta=1/8 各命中一次 Q7a 的合取），`basis_count` 从 9 变 10；`q7c1_flip_seen`/`q7c2_flip_seen` 字面不变（β_syn 已经让它们是 `true`），但今天这一格是**可达基底**上的转色，与只在不可达基底上转色不是同一件事，见「它答不了的」新增两条。新增一条单测（`hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`）独立重放同一段真实历史（mkfs → 新池新建文件 → 72 次前缀覆盖写 → 管理员回退到候选集里最旧的根），不调用 `main()` 里的同一段代码，只借同样的公开入口独立走一遍，核对 `item5==0`、G27 在基底上判绿、`q7c_self_test` 的 flip1 为真、flip2 为假；单测由 10 条增到 11 条。新增两条变异（`crates/mutations.tsv` 末尾 M33/M34，各改坏 `q7c_self_test` 里 `without_subtracting_defer_is_red`/`real_check_red_plus1` 的比较符号），都逐条手工改坏验红、还原验绿，变异表由 10 条增到 12 条。产物落盘之前把跑前登记「十二」修订追加为第 4 条，写明这是新加的一格、为什么加、在看过哪份产物之后加，判据逐字沿用岔路单第 12 行原判据、不改。产物另存 `e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`（1085 行，比 r3.out 多 9 行：1 行 `basis_snapshot` + 3 行 `q7a` + 2 行 `q7c` + 3 行 `pc_check`），`replay.sh` 已改指向它，`r3.out` 原样保留。装置在跑本段产物前后各起一次编译，中途撞过一次 `crates/singlefs-core` 的瞬时编译失败（`mount.rs` 第 1073 行类型不匹配），几分钟后自愈——`git status --short crates/` 显示本机同时有多个会话在改 `crates/singlefs-core`/`singlefs-checker` 的大片文件，那次失败发生在其中一个会话的中间状态上，不是这一份改动引起的；`cargo check -p singlefs-core` 与 `replay.sh` 里这个实验的登记行随后都过、字节一致，交主 agent 知悉。**这一格够不够判交主 agent**：它证明了 Q7c① 这套自证在**至少一个**可达状态上能转色，但没有扫过全部几何与历史长度去确认这是不是唯一的一格、或是不是「回退到候选集最旧根」这类结构性条件的必然结果。
