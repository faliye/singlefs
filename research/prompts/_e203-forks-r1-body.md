# e203-forks-r1：准入之前扫类的三条岔路（B1、B8、B4）拿 E203 的数判，推论轮第一轮正文（2026-11-07）

<!-- doc-lint:not-numbers B1 B4 B7 B8 X1 X2 Y1 Q4 Q5 Q6 Q8 Q9 F1 F2 F5 F6 -->

## 一、题面

岔路单 `git show ff5dca7d22b1:research/prompts/m4-admission-delta-forks.md`（主 agent，2026-11-05）三行：B1 今天、B8 逐项按差量、B4 改射程（各行的候选定义、翻面观测与够判条件以那份文件的原文为准，用这条 `git show` 取）。E203（准入之前扫类的步代价）照跑前登记 `research/prompts/e203-preregistration.md` 跑完三段（18 个点，Y1 照修订 6 挪进不指定范围的实验复跑），合并报告是任务 9595（命令 `cargo run --release --bin e203-admission-scan-cost -- report <计数产物 18 份> --timing <计时产物 18 份>`，在 `scripts/experiment-apparatus/` 起）。用户 2026-11-07 看了数之后说「这个一次拷贝 不是在预料中吗  我们没有优化这部分吧」，定「开三方判读再交你定」。

这一轮要回答：三行各自站不站得住，交用户定的候选该是哪几个、各自的代价数。判的是岔路，不改登记的判据。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

## 二、实现今天的样子（主 agent 的观测，2026-11-07）

- E203 量的是冻结源树 `~/.local/share/singlefs/work/e203-runner-segment-2-r2/pristine-source/`（第二段那一刻主工作区的 `crates/` 等几份，从计数构建副本反打插计数的 diff 还原、核过 sha256，停机条款第 3 条）。26 行扫类每行在 `crates/` 里的文件:行与类别，在跑前登记第三节（`research/prompts/e203-preregistration.md` 第三节清单）。
- 今天的实现就是 B1：两道准入（落点判、式子判）之前照今天全量扫、全量建索引、整份拷分配器。B8、B4 在 `crates/` 里都没有实现；E203 里 B8、B4 的步数是计数模型 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` 按第五节式子在量到的规模上算的（推的），不是改过的代码跑出来的。`crates/` 里没有差量视图、撤回日志、释放判定增量索引：`grep -rn 'delta_view\|undo_log\|incremental_release_index' crates/singlefs-core/src/` 0 命中。
- 方案按 `crates/` 今天的实现来谈。

## 三、量到的数（产物行整行抄；出处是任务 9595 日志 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-9595.attempt-1.log` 与 `research/results/e203-admission-scan-cost-*.out`）

判据与停机：

```
name=verdict quantity=controls all_passed=true
name=verdict quantity=three_two cells_checked=14038 disagreements=0 cells_over_the_bound=0 cells_with_the_scale_unread=2 rows_reconciled=26 rows_equal_only=18 rows_with_a_bounded_part=8 rows_not_reconciled=0
name=verdict clause=void_6 points=18 points_with_the_done_marker=18
name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond reference_only_because_data_or_files_axes_are_missing=false not_judged_because_the_extrapolation_is_not_trusted=true
name=verdict quantity=Q5 cells_passed=2 cells_failed=8
name=verdict clause=F1 fired=unknown meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1
name=verdict clause=F2 fired=true meaning=Q6_B8_still_grows_on_some_axis
name=verdict clause=F5 fired=false meaning=B4_arm_before_the_landing_judgement_still_grows
name=verdict clause=F6 fired=true meaning=some_Q5_cell_failed
```

B1：X2（每盘 16 TiB 的单元区、历史各轴取 10）直接量的，不外推（`research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07.out`，`row=whole_allocator_copy_before_the_judgement` 的 `exclusive_nanos`）：同一次挂载覆盖写五轮 1420202116、1419989321、1414134016、1422421328、1419390647；重挂载五轮 1319344068、1321940719、1325601594、1318225192、1321239046。登记第十二节修订 3：「X2 量过之后照严的读法 Q4 不判」。

B8（Q6）：

```
name=q6 axis=U state=same_mount kind=overwrite low_end=1504 high_end=1980 alpha=0.039670 band=DoesNotGrow
name=q6 axis=u state=same_mount kind=overwrite low_end=1748 high_end=11396 alpha=0.271404 band=Between
name=q6 axis=D state=same_mount kind=overwrite low_end=1748 high_end=429120 alpha=0.796679 band=GrowsWithTheAxis
name=q6 axis=F state=same_mount kind=overwrite low_end=1748 high_end=3213078 alpha=1.088127 band=GrowsWithTheAxis
name=q6_top_rows point=user_scale_unit_area state=same_mount kind=overwrite rows=allocator_release_linear_find:506,mapped_units_whole_table:376,record_intersection_filter:354
name=q6_user_scale state=same_mount kind=overwrite data_units_user=10 by_delta_steps=1055418 by_delta_axes_not_extrapolated= today_steps=20675377896 today_axes_not_extrapolated=
name=q6_user_scale state=same_mount kind=overwrite data_units_user=268435456 by_delta_steps=11693260552 by_delta_axes_not_extrapolated= today_steps=149317555187 today_axes_not_extrapolated=
```

D、F 两轴在三种发布、两种挂载状态上 α 都在 0.66–1.09（整行在日志里，`grep 'name=q6 '`）。

B4（Q8、Q9）：

```
name=q8 arm=b4 axis=U state=same_mount kind=overwrite low_end=36 high_end=36 alpha=0.000000 band=DoesNotGrow
name=q8_user_scale state=same_mount kind=overwrite data_units_user=10 narrowed_steps=66 narrowed_axes_not_extrapolated= today_steps=6618613940 today_axes_not_extrapolated=
name=q9 point=user_scale_unit_area state=same_mount kind=overwrite steps_between=13220563557 nanos_of_each_valid_round=2778493178,2792344478,2781414849,2776512072,2765885729
name=q9_user_scale_nanos state=same_mount kind=overwrite data_units_user=10 nanos_of_each_round=7623811388,4394849652,4266465554,5332990357,4434986126 axes_not_extrapolated=
name=q9_user_scale_nanos state=same_mount kind=overwrite data_units_user=268435456 nanos_of_each_round=403797602001,288264437866,277411979953,292008599061,289442017787 axes_not_extrapolated=
```

## 四、压着的条款

- `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」（第 17 行起）：运行时决策路径（分配、ENOSPC 准入、defer 窗口、生命周期判定）不许遍历，且代价不许随盘容量增长。
- `.claude/kb/decisions/08-核心索引结构.md` D8（核心索引结构） 已定项 16（第 503 行起，到下一个 `####` 为止）。
- 跑前登记第六节判据表（Q4–Q9）、第十节失败条款 F1–F6、第十二节修订 1–6。

## 五、格

1. B1：判据是 Q4 外推（四格都 ≥ 1 毫秒则翻面），Q5 核不过、严读法不判；X2 直接量到整份拷分配器一行 1.32–1.42 秒。拿 X2 的直接读数判 B1 翻面，是不是在改跑前写死的判据（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列」与「判据自己也会写错：打中之后先判是哪一种」）；X2 的记录数各轴只到 10，「用户规模上只会更大」要靠什么（扫类步数随记录数单调不减）成立，那个前提在冻结源树里核得到吗。
2. B8：F2 打中在 D、F 两轴。贡献最大的三行（`allocator_release_linear_find`、`mapped_units_whole_table`、`record_intersection_filter`）是 B8 定义里本该按差量改掉、模型没改到（模型的式子漏了），还是 B8 的定义本身就留着它们；按 `evidence-discipline.md`「记一次输 / 只许收严 / 写明收严在哪」，B8 能不能出一个收严的新形态，那就是新候选、被攻过零轮。
3. B4：落点判之前不涨（F5 没打中）。两道准入之间那一段在 X2 上每次覆盖写约 2.77 秒（量的），外推到用户规模 4.3–404 秒（推的）。B4 把这一段读成不在 `fs-design.md` 运行时决策路径一格里；式子判本身算不算「ENOSPC 准入」，这一读法站不站得住。
4. 有没有候选漏了（例：落点判之前按 B4、两道之间按 B8 的差量），各自要什么代价数才能交用户；新冒出来的组合记零轮，写成要登记的实验，不在这一轮判它赢。
