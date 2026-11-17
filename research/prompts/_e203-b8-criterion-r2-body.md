# e203-b8-criterion-r2：严格判据下 B8 量出了什么、量不出的那一格怎么办（2026-11-13）

<!-- doc-lint:not-numbers B1 B4 B8 F1 F2 F3 F5 F6 F7 F9 F10 F11 Q0 Q1 Q7 Q8 Q9 Q11 P1 P2a W1 W2 W4 W5 W6 W7 K100 K1000 -->

## 一、题面

第一轮判决（`research/prompts/e203-b8-criterion-r1-main-verification.md` 第三节）定下新登记的形态，「这些条件全部零轮」；用户 2026-11-09 采用，登记成 E207（严格判据下的扫类步数），尖峰成因另立 E209（尖峰成因）。两个实验能量的都量了。这一轮是这一题的第二轮，攻第一轮的修补本身、产生结论的方法、没被看过的地方。

用户 2026-11-13 定（原话在 `records/2026-10-13-里程碑四预开发.md` 第 1688 行）：「这个影响什么 又不需要均匀 这样先这样没问题 其他的走三方」。「这个」指 E207 复量那一格：已记的翻面不动，复量判 `unstable`，这一格留开着，不再加点。这一轮不重判这一格的去留，只判它之外的四件。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

- `crates/` 今天没有 B8 的实现；E207、E209 的步数由装置往拷出来的实现里插计数得到（`scripts/experiment-apparatus/e203_admission_scan_cost_counters.py`、`scripts/experiment-apparatus/e207_strict_scan_steps.rs`、`scripts/experiment-apparatus/e209_spike_cause.rs`）。方案按 `crates/` 今天的实现来谈。
- 每次发布按全部分配记录做的几处（主 agent 现读行号，各处占多少用时没量）：`crates/singlefs-core/src/transaction.rs:13213` `let allocator_before_this_publish = allocator.clone();`；固定点循环 `:14031` `for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {`，每轮 `:14076` `let mut rehearsal = allocator.clone();`，`:14085` 与 `:14105` 各一次 `nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records())`，`:14106` `nodes_holding_records(&geometry, rehearsal.records())`。登记里计数的扫类步数不数这几处的拷贝与比较。
- 重挂载的写行发布走 `crates/singlefs-core/src/mount.rs:5841` 的 `row_publish_plan_on_file_version`。
- 被量的三份实现文件此刻的 sha256 前 16 位：`transaction.rs` d6b2083d2d950d7f、`allocator.rs` eb822f0920329af4、`mount.rs` ae24940185ad0102。
- 会话与读数状态没有存盘与读回的入口：执行员 `grep serde` 无命中（`~/.local/share/singlefs/work/e207-b8-strict-runner-24/result.md`，主 agent 没复查）。

## 三、量到的（整行抄自留存产物，主 agent 逐行读过）

E207，`research/results/` 下：

| 读数 | 文件与行 | 原文 |
|---|---|---|
| 第一段，单个单元改动档总判 | `e207-strict-scan-steps-report-first-2026-11-13-r4.out`（留存形态；原始 `-r3.out` 第 5252 行） | `name=verdict quantity=q1 reading=one_unit_change_against_the_total segment=new tier=one_user_unit cells=36 cells_unread=0 row_one_holds=false row_two_flips=true only_the_first_grows=true scan_control_void=false linear_control_void=false` |
| 第一段，多单元改动档作废 | 同上原始第 5250 行 | `name=verdict quantity=void_clauses segment=new tier=user_units_equal_declared applicable_cells=54 scan_control_red=12 linear_control_red=54 scan_control_void=true linear_control_void=false` |
| 贴线那一格 | 同上原始第 4552 行 | `name=cell segment=new set=K100 state=same_mount tier=one_user_unit pick=later_peak filter=steady arm=tested ends=7167/1/25346,22641/1/250190 looser=no alpha=0.502 band=grows raw_alpha=0.502 alpha_by_mapped_units=0.453 local_slopes=two_points linear_row=allocator_release_linear_find` |
| 复量主格 | `e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13.out` 第 2636 行 | `name=series_cell state=same_mount tier=one_user_unit pick=later_peak filter=steady window=common_window arm=tested primary=yes points=6 ends=5974/1/14152,7167/1/25346,9394/1/43844,9108/1/76932,14948/1/136064,15167/1/241260 alphas=0.312,0.494,-0.055,0.869,0.025 beta=0.345 beta_band=between grows=1 between=2 does_not_grow=2 unread_segments=0 beta_by_mapped_units=0.321 verdict=unstable` |
| 对账分辨 | `e207-strict-scan-steps-report-reconciliation-split-2026-11-13.out` 第 679 行 | `name=verdict quantity=reconciliation_split point=files_1000 outcome=tree_changed reason=head_differs=false,gap_negative=false,rerun_differs=false,changed_tree_signature=true,five_items_differ=true` |
| 对账分辨的两条阳性对照 | `e207-strict-scan-steps-report-reconciliation-split-slots-placed-undercounted-2026-11-13.out` 第 336 行；`…-image-witness-flipped-2026-11-13.out` 第 338 行 | `name=verdict quantity=placement_gap point=base operations=162 negative=38 first_negative=same_mount/W1/1 zero=0 positive=124`；`name=verdict quantity=head_against_plain point=base operations=162 differing=1 first_differing=remounted/W2/5 remount_before_the_crash_same=true remount_after_the_mount_same=true` |
| 第二段，尖峰三种重现假设 | `e207-strict-scan-steps-report-second-2026-11-13.out` 第 3892 行 | `name=verdict quantity=q8 outcome=none_of_the_three` |

第三段（`files_30000` 加扩展窗口）：同一次挂载 401 个操作旗上 1 个（W1 第 1 次），扩展窗口 321 个操作旗上 0 个，重挂载 80 个旗上 0 个（执行员读 `e207-strict-scan-steps-report-third-2026-11-13.out`，主 agent 没逐行读）。

第四段（登记已定项 1「加大点，分段跑」）：较小两点量完；中间点 `files_100_units_1000_unit_area_2_pow_21` 整项一片 434.584 秒、内存峰值 3.64 GiB（任务 15662，量的）。最大点 `files_1000_units_1000`（活单元 10⁶、发布约 4000 次）两次没量出：稀疏盘那一趟片峰值到 45.55 GiB 装不下（任务 15482）；换紧凑盘之后内存够（`user_changes=1220` 时 21.4 GiB），整项一片跑 142 分钟到 `publishes_done=1837`（共 4000），进度行每五分钟依次 551、843、1051、1223、1331、1439、1551、1641、1740、1837（任务 15682 的子任务 15693 的日志，量的）；最后一段每次发布约 9.7 秒（执行员量的），整段外推约 56000 秒（执行员推的，拟合指数 2.48）。任务服务的单项上限是四小时。

E209，`research/results/e209-spike-cause-report-third-2026-11-13.out`（留存形态，1052 行；行号是去掉前缀后的原始行号）：

| 读数 | 行 | 原文 |
|---|---|---|
| 总判 | 1051 | `name=verdict items_read=74 items_without_item_done=zero items_refused=zero constants_mismatches=zero history_mismatches=zero script_sequence_mismatches=zero closure_model_mismatches=zero final_set_not_covering=zero final_nodes_mismatches=zero edge_walk_mismatches=zero live_units_shortfalls=zero negative_control_over_four_rounds=zero twin_differences=zero plain_differences=zero reproduction_at_the_training_point=reproduced` |
| 复现闸，不复现的那一点 | 86 | `name=reproduction_gate point=files_10_units_1000 operations=10 median_final_nodes=7 line_at_the_main_floor=28 flagged_at_14=0 flagged_at_28=0 flagged_at_56=0 verdict=not_reproduced` |
| 行一四个读法 | 831–834 | 四行都 `verdict=does_not_hold`；窄读法两行 `training_failures=0 … script_mismatches=1 script_mismatch_labels=files_100_units_100__head__old_mount_old#1:fresh=42:observed_flag=0` |
| 行二 | 848 | `name=ordinal_shift point=files_100_units_100 walk_forward_first_flagged_ordinal=1 fillers=8 verdict=follows_the_operation` |
| 行四 | 849 | `name=mount_carries point=files_100_units_100 first_after_the_mount_flagged=false nodes_of_the_spike=53 carried_by_the_mount=51 share=0.962 verdict=mount_carried_the_nodes remount_publishes=6` |
| 行四，不复现的那一点 | 930 | `name=mount_carries point=files_10_units_1000 first_after_the_mount_flagged=false nodes_of_the_spike=17 carried_by_the_mount=15 share=0.882 verdict=partly_carried remount_publishes=4` |
| 行三预清扫 | 986 | `name=presweep point=files_100_units_100 flagged_counts=0 unflagged_counts=1,2,4,8,16,32,64 between=0..1 verdict=threshold_between_two_sample_points` |
| 几何敏感性 | 1048 | `name=geometry_sensitivity quantity=ordinal_equality points=6 per_point=files_100_units_100:false,files_100_units_100_unit_area_2_pow_17:false,files_100_units_100_unit_area_2_pow_20:false,files_10_units_1000:true,files_200_units_100:false,files_75_units_100:false stable=false` |

执行员照登记判的（当线索，腿要自己核）：E209 问题单 `research/prompts/e207-spike-cause-questions.md` 行一、二、三、五不成立，行四在五个复现点上成立（share 0.949–0.981，定位到写行那一步）；E207 岔路单第 9、10 行没够判，第 11 行仍开。

## 四、要回答的问题

1. **岔路单三行的去向**（`research/prompts/e203-b8-criterion-r1-forks.md` 第 9、10、11 行）：照第三节的读数，三行各该记「开着 / 够判 / 用户已定」里的哪一个；第 10 行除了用户已定留开的那一格，别的格的读数（单个单元改动档 36 格里 `row_one_holds=false row_two_flips=true only_the_first_grows=true`）说明什么、不说明什么；第 11 行有了 E209 之后还差不差东西（E209 是在小几何上量的，E207 观测到尖峰的大点没放进去；`files_10_units_1000` 不复现）。
2. **量不出的那一格**：多单元改动档在前三段作废、第四段最大点在四小时内跑不完。三条出路各自的代价与买到的东西：整条跑过四小时（约 56000 秒独占一台，推的；阳性对照在第四段也贴线，登记写的 `C1` 约 0.51 对 0.5，推的）；在 `crates/` 加会话与读数状态的存读入口（要决策与实现员，改的是产品代码）；改登记里的最大点（换成跑得完的规模，那一格的总量跨度变小）。有没有第四条路；哪一条在不放宽判据的前提下站得住；都站不住的话这一格该怎么记。
3. **第二节那几处「每次发布按全部分配记录做一遍」**：登记的扫类步数没数它们。B8 那一行「准入之前的步数只随这一次碰到的单元、容器、记录数涨」与 `.claude/rules/fs-design.md`「记账是事务的副产品」里「代价不许随盘容量增长」那一句，罩不罩这几处；罩的话 E203、E206、E207 的步数口径少数了什么，不罩的话依据是哪一句。这是没被看过的地方。
4. **E209 三处登记没写死的口径**（执行员整理在 `~/.local/share/singlefs/work/e209-spike-cause-runner-7/result.md` 第四节）：挂载窗里只有一两个操作时旗线退化（登记读法 1）；行五的可比操作不限挂载次序（登记读法 7）；不复现尖峰的点算不算几何敏感性的取样点。各自改不改得动已判的五行；按 `.claude/rules/evidence-discipline.md`「臂的定义也在跑前写死之列」与「判据自己也会写错」那一节，哪一处属于「判别子观测不到」或「打中不分辨臂」，要不要另立一轮登记。
5. **对已定条款的关系**：E207、E209 两页的「影响的决策」现在对 D8（核心索引结构） 已定项 16 写「备料」。照 `.claude/rules/format-evolution.md`「实验页的影响的决策」的四种关系，这两份实验对它、对 D28（挂载期承诺量） 已定项 1 各该写什么；E203 选臂（第一轮判决「交用户」第三行）现在动不动得了。

## 五、压着的条款

D8（核心索引结构） 已定项 16；D28（挂载期承诺量） 已定项 1；E203（准入之前扫类的步代价）、E206（发布固定点轮数的代价）、E207（严格判据下的扫类步数）、E209（尖峰成因） 四个实验页；登记 `research/prompts/e207-preregistration.md`、`research/prompts/e209-preregistration.md`。
