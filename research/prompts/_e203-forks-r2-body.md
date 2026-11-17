# e203-forks-r2：准入之前扫类的三条岔路（B1、B8、B4），修订 7 重跑之后的推论轮第二轮正文（2026-11-07）

<!-- doc-lint:not-numbers B1 B4 B8 B11 X1 X2 Y1 Q3 Q4 Q5 Q6 Q8 Q9 Q10 F1 F2 F3 F5 F6 -->

## 一、题面

第一轮判决 `research/prompts/e203-forks-r1-main-verification.md` 打中三处装置与登记的错（s36 式子与同一行做法不相容、s25 第二处查找漏记调用数、F3 读不到点轴值时报了 `fired=false`）。用户 2026-11-07 定「①②和 s36 都修、全部重跑」，登记 `research/prompts/e203-preregistration.md` 第十二节修订 7 照此写下（先记一次输）。修订 7 的装置改动与 36 条重跑做完，合并报告是任务 10180（命令 `cargo run --release --bin e203-admission-scan-cost -- report <18 份计数产物> --timing <18 份计时产物>`，在 `scripts/experiment-apparatus/` 起；36 份产物在 `~/.local/share/singlefs/work/e203-runner-revision-7/report-inputs/`，`research/results/` 下那一批已被别的会话删掉，内容在提交 84b4ae23f 里）。

这一轮要回答的与第一轮相同：三行各自站不站得住，交用户定的候选是哪几个、各自的代价数。判的是岔路，不改登记的判据。岔路单原文：`git show ff5dca7d22b1:research/prompts/m4-admission-delta-forks.md`。

这是这一题的第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不许重复第一轮已攻过的角度（第一轮判决第二节那七格）。

## 二、实现今天的样子（主 agent 的观测，2026-11-07）

- 量的仍是冻结源树 `~/.local/share/singlefs/work/e203-runner-segment-2-r2/pristine-source/`；`crates/` 今天的实现就是 B1，B8、B4 在 `crates/` 里都没有实现，步数是计数模型 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` 按第五节式子算的（推的）。
- 修订 7 改了：计数模型 s36 的查询项改成全部轮里进出 `mapped_units` 的角色数 × lg(M)（函数 `mapped_units_queries_by_delta`）；插计数脚本 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 补记 s25 第二处查找（冻结源树 `allocator.rs` 第 2262–2267 行一带）；计时报告的点轴值从同名计数产物取，取不到报 unknown。另：入口 `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh` 把进度行转到标准输出（任务 9845 被服务判卡住之后）。
- 方案按 `crates/` 今天的实现来谈。

## 三、量到的数（任务 10180 日志 `/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-10180.attempt-1.log`，整行抄）

```
name=timing_point_axes points_of_the_timing=18 taken_from_the_counts=18 unread=0
name=verdict quantity=controls all_passed=true
name=verdict quantity=three_two cells_checked=14038 disagreements=0 cells_over_the_bound=0 cells_with_the_scale_unread=2 rows_reconciled=26 rows_equal_only=18 rows_with_a_bounded_part=8 rows_not_reconciled=0
name=verdict quantity=Q6 result=GrowsOnSomeAxis
name=verdict quantity=Q8_b4_arm result=DoesNotGrowOnAnyAxis
name=verdict clause=F1 fired=unknown meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1
name=verdict clause=F2 fired=true meaning=Q6_B8_still_grows_on_some_axis
name=verdict clause=F3 fired=true meaning=residual_grows_on_some_axis_in_all_five_rounds
name=verdict clause=F5 fired=false meaning=B4_arm_before_the_landing_judgement_still_grows
name=verdict clause=F6 fired=true meaning=some_Q5_cell_failed
name=q6 axis=F state=same_mount kind=overwrite low_end=2102 high_end=152344 alpha=0.620064 band=GrowsWithTheAxis
name=q3 axis=U state=same_mount kind=overwrite alphas=0.691719,0.638257,0.692836,0.687398,0.664968 band=GrowsWithTheAxis
name=q10 axis=U state=same_mount kind=overwrite segment=between alphas=0.461720,0.409846,0.480794,0.481663,0.449168 band=Between
```

D、F 两轴其余 11 格 `band=DoesNotGrow`（`grep 'name=q6 axis=[DF] '`）。

## 四、压着的条款

同第一轮正文 `research/prompts/_e203-forks-r1-body.md` 第四节（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」第 17 行起；`.claude/kb/decisions/08-核心索引结构.md` D8（核心索引结构） 已定项 16 第 503 行起；D28（挂载期承诺量） 已定项 1；登记第六、十、十二节）。

## 五、格

1. 修订 7 本身：s36 改成「进出 `mapped_units` 的角色数 × lg(M)」是不是收严（与登记第五节 s36 行做法一致、比原式子多要求了什么），还是借修订把 B8 放宽了；s25 补记、F3 读计数产物的轴值，改法有没有带进新错（核模型与插计数脚本的改动、自证与变异）。
2. 产生结论的方法：B8 只剩 F 轴、同一次挂载、覆盖写一格 α=0.620。这一格的涨来自哪几行、是固定点轮数的乘数还是别的；它是 B8 定义里该有的，还是装置（历史用例、F 轴取点）造出来的。F3 打中（两道之间残差在 U、u 轴五轮都涨），它分不分辨臂（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」四问）。
3. 从未被看过的地方：第一轮两腿都没碰的扫类行、历史用例的取点、计时与计数两个构建是不是同一份实现（停机条款第 3 条 `stop_3 state_fields_that_differ=0`）以外还有没有不一致；X2 只取历史各轴 10 这一取样点够不够代表用户规模的 B4 两道之间那一段。
4. 交用户之前，第一轮判决第三节交用户表里的候选（只换整份拷、分块共享、B8 收严、留 B1 加例外、式子判吃最坏数）哪几个这一轮的数已经够判、哪几个仍缺代价数。
