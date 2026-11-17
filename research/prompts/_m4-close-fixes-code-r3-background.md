# m4-close-fixes-code-r3：r2 判决第四节与 P2a，第三轮，最终轮（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 P1 P2 P2a P3 -->

## 一、题面

`research/prompts/m4-close-fixes-code-r2-main-verification.md` 第四节五条（读全部接上计数、映射 key 集合不等就拒、账的三处、名字与死链、测试）与用户 2026-11-09 定的 P2a（固定点每轮把释放链走到闭包，E206（发布固定点轮数的代价））已打进主工作区，都被攻过零轮。这是这一题的第三轮，也是最终轮，之后不再开第四轮。

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。这一轮只攻前两轮站住的形态与这批改动；这一轮里新冒出来的零轮形态写进判决的交用户表，不再为它开一轮。

改动全文：`research/prompts/_m4-close-fixes-code-r3-diff.md`（`crates/singlefs-core` 7 个文件块）。实现员报告：`~/.local/share/singlefs/work/m4-close-fixes-r2-writer/result.md`、`result-reds-and-closure.md`、`result-unkilled-rows.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `allocator.rs`、`compaction_plan.rs`、`instance_table_sweep.rs`、`mount.rs`、`mounted_session.rs`、`recovery.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-fixes-code-r3-snapshot/crates-sha256.txt`）。这几份里不是这批的块（别的会话的改动）当上下文读，腿只攻 diff 里有的块。
- 验证到的数（量的）：主工作区 clippy 任务 11330 退 0；带 gpu：`unit_read_…` 37 条过、`crash_points_…` 4 条过、`states_judged_by_the_gpu_here=182`（任务 11331）；P2a 目标 `allocation_record_tree_settling_walks_the_release_chain_to_its_closure` 1 条过（任务 11332）；崩溃枚举用例缩域到 5：`states=177 … exhaustive=false violations=0`（任务 11334，缩过域不算门禁通过）。变异在副本上都证过红（任务 11220、11288、11307、11311）；「释放处不走闭包」红在 `覆盖写 inode 1 的固定点走了 29 轮`。全量域 GPU、不指定范围的门禁都没跑。
- 压着的条款：D4（校验和位置） 已定项 4，用户定案「按字面全接上」（`research/prompts/m4-close-fixes-code-r1-main-verification.md` 第三节）；映射 key 不等就拒（r2 判决第三节）；P2a 的定义 `research/prompts/e206-preregistration.md` 第五节 P2a 那一行；E206 产物与判定 `.claude/kb/experiments/206-发布固定点轮数的代价.md`（P1 逐轮读数按作废条款 1 作废，只作线索）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. **P2a 的前提**（登记写「被并进去的节点都在 P1 的不动点里」）：`merge_into_the_rewrite_set_walking_the_release_chain` 一带，走闭包会不会并进 P1 不动点之外的节点（多写单元），读上一版指针的时机对不对。量到的只有 16 个形状（第二段 33 项没跑），测试只有一段历史；什么形状 P2a 仍走满上界 64 拒。实现员报告写了写行那一份（树表 0 条那一版）两处并入没有证红，判这个盲区。
2. **映射 key 集合不等就拒**（`recovery.rs` 一带）：有没有合法的盘被拒——留存条目、搬迁中崩溃、回退之后、快照销毁、整理半途这些状态，key 集合的等式都成立吗；GPU 判器少判的一形（实现员报告第 2 条）；只读路径不判留下什么。
3. 读全部接上之后：穷举表（实现员报告）里没接的 `transaction.rs:668`、`mount.rs:1407` 一带的依据站不站得住；接了线而没有变异行的 6 处（`instance_table_sweep.rs:743`、`:921`，`mounted_session.rs:4115`，`mount.rs:10789`、`:10901`、`:5350`）。
4. 账的口径统一之后：去重的范围、「只试一次」写实之后那一句，与 D4（校验和位置） 已定项 4 对不对得上。
5. 未定项清单：r1、r2 判决与实现员「停下交主 agent」里没有去向的条目，有没有漏；攒批会话读不到待发写（r2 判决第五节）要不要进条款。
6. 数字能不能核：正文第二节每个数对产物；E206 页引的行。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告原样：「补丁在产品代码里没有新加 `todo!`、`unreachable!`、`assert!`，也没有以 Undecided、Unsupported、OutsideTheFirstVersion 收尾的错误成员（`grep` 补丁的加行，量的）。删掉的那一支（`transaction.rs` 旧 `:11953`）的理由写在第 4 条。」第 4 条：「账上的组都是这次发布之前读到的」。P2a 那一批报告写「走不到的理由：0 处」。

### 检查单：类型与名字

新加的名字由腿看 diff；`merge_into_the_rewrite_set_walking_the_release_chain`、`PoolWriter::rounds_of_the_last_allocation_record_tree_settling`（只读观测）、映射 key 集合那一判的拒因、改名后的开关。`expect` 依赖的不变量逐处看。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；P2a 不留开关。腿现跑 `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r3-diff.md`。

### 检查单：不新建判定库

本轮改动没有这一类：产品代码不写判定。

### 检查单：所有任务多机可用

产品代码没有这一类；新测试目标由腿看读不读本机状态。

### 检查单：测试优先多线程

P2a 目标单历史逐次判，次序就是被测对象；由腿核。

### 检查单：带 --item 的命令与项

带 `--item` 的 crates-mutation-replay 这批没跑（任务 11017 要记 45.6 GiB、等整台腾空，取消），整表归提交时；变异行都走 `prove-red.sh` 证过。

### 检查单：新崩溃流多罩了什么

`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping` 182 个状态加「坏盘 × 部分落盘」与卸载之后判真盘；GPU 缩域新流多罩建上一版的下界与映射 key 集合。腿核是不是真比已有的多。

## 五、格

1. 代码做的是不是条款说的。
2. 第三节六条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。

### 小节清单：`.claude/kb/experiments/206-发布固定点轮数的代价.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E206 发布固定点轮数的代价 —— 部分已跑：第一段 40 项（39 项完整、1 项作废）；作废条款 1 记一次输，P1 逐轮读数作废、仅作线索；F1 打中，P1 翻面；P2a 够判；P3 前提不成立（读数出自 P1 逐轮读数，仅作线索）；第二段 33 项够判后未跑 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E206 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 现状 | 抄 | 默认全抄（脚本生成） |
| ### 第一段结果 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表 | 抄 | 默认全抄（脚本生成） |
| #### P2a 的读数 | 抄 | 随父节带到 |
| #### P3 的读数 | 抄 | 随父节带到 |
| ### B8 那一格的读数 | 抄 | 默认全抄（脚本生成） |
| ### 对照与作废的读数 | 抄 | 默认全抄（脚本生成） |
| ### 判据与失败条款 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-11-09 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/decisions/04-校验和位置.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D4 校验和位置 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D4 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：校验和粒度与随机小读的张力 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：不足一个单元的 extent 怎么算 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：凑不满一个单元的写怎么写 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：读—改—写读不回旧单元时怎么办 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：32 KiB 指的是含头还是净荷 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：extent 声明长度住哪、多宽 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：逻辑 extent 与单元的关系 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：加密关闭时 Merkle 字段取什么；已定项 1 的 5% 重开判据基线 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:1-10`（整段抄，未转述）**

```markdown
## E206 发布固定点轮数的代价 —— 部分已跑：第一段 40 项（39 项完整、1 项作废）；作废条款 1 记一次输，P1 逐轮读数作废、仅作线索；F1 打中，P1 翻面；P2a 够判；P3 前提不成立（读数出自 P1 逐轮读数，仅作线索）；第二段 33 项够判后未跑

**中文** · [English](206-cost-of-the-publish-fixed-point-rounds.en.md) · [日本語](206-パブリッシュ固定点ラウンド数のコスト.ja.md)

<!-- doc-lint:not-numbers P1 P2 P3 P2a P2b B1 B4 B8 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 F1 F2 F3 F4 F5 F6 F7 F8 W1 W2 W3 W4 U -->

跑前登记 `research/prompts/e206-preregistration.md`（第十二节修订 1 记了产物之后补的三件事），岔路单 `research/prompts/e203-forks-r2-forks.md` P1、P2、P3 三行。问的是一次发布的固定点走多少轮、什么形状的历史让它走很多轮、连续发布逐次怎么变；压轮数（P2）与降每轮代价（P3）各花多少。这一笔量完，E203（准入之前扫类的步代价） 的 B8 那一格照原判据再判（E203（准入之前扫类的步代价） 登记第十二节修订 8），读数在下面「B8 那一格的读数」一节。

**这一页记第一段 40 份产物与合并报告（任务 11102）。** 合并报告 `research/results/e206-fixed-point-cost-report-2026-11-09.out` 末行 `name=report_done items=40 complete=39`。第二段 33 项与第三段没跑（够判后未跑）。这一页不判结论能不能推翻或确立决策，也不选 P1、P2、P3 哪一条；「选哪条」交用户。

```

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:11-20`（整段抄，未转述）**

```markdown
### 现状

- 第一段 40 项：16 个点各 p1、p2a 两臂共 32 项，`single_step`（`base`、`files_100`、`files_1000`）与 `closure_off`（同三点）6 项，`plain`（`base`、`files_1000`）2 项。分三组 `--item` 跑（登记修订 1），判定经 prune-server。39 项有 `name=item_done`；`files_1000_units_100__p1` 没有，它的量窗第一次覆盖写被拒（下一条）。
- **F1 打中**：`files_1000_units_100__p1` 的量窗第 1 次覆盖写，固定点走满 `fixed_point_rounds=64`，交回 `AllocationRecordTreeRewriteSetDidNotSettle`。这是产品路径上的拒绝（上界 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT = 64`，`transaction.rs:13466`），登记 F1 要求当场交主 agent，已写进执行员报告。登记第四节第 12 条的推算（`files_1000_units_100` 今天的实现会交回这个错）这一点上打中；同一条推算对 `files_30000` 的预期（约 74 轮、走过上界）没打中，实测 38 轮定下（作废，仅作线索）。F1 那一次拒绝是 P1 臂原样的固定点循环的结局，不依赖 `single_step` 对照，照报。
- **作废条款 1 记一次输**：`single_step` 对照三点 `holds=false`（合并报告 569、571、573 行）字面触发作废条款 1，P1 的逐轮读数整段作废，判据不放宽。下面凡出自 P1 臂逐轮读数的数（`arm=p1`、`item=…__p1` 的行，含 Q1 的峰值 38、29、22、11）都是作废的读数，只作线索；整行照抄，不删。P2a 的选择依据是它自己的读数与不动点比对（主 agent 的判定，`records/2026-10-13-里程碑四预开发.md` 第 1579 行）。
- 登记作废条款第 6 条：这个点这条臂作废（`name=void_6`）。**那个点上 P2a 没有比对对象**：`files_1000_units_100__p2a` 完整（`name=item_complete … complete=true`），可是 Q5 的不动点相同率与净步数、Q6 的写放大都是拿 P2a 与同点的 P1 比，P1 没有读数，这三个量在该点上没有行。
- 用户 2026-11-09 弹窗定「现在实现 P2a」，第二段 33 项不先跑（够判后未跑）。第二段是 12 个点的 p1、p2a 共 24 项，加三个轴末点的 P2b16、P2b256、`whole` 共 9 项，登记第五节数下来是 33 项；弹窗写的 36 项是错的（`records/2026-10-13-里程碑四预开发.md` 第 1579 行）。
- 合并报告 `fork_row` 三行：P1 `enough_readings=false`、P2 `enough_readings=true`、P3 `enough_readings=false`。这个标志只数「第一段 16 个点上 p1（P1、P3）或 p2a（P2）的读数是不是都完整」（`fork_rows_with_enough_readings`，`e206_fixed_point_cost.rs:1872`）：P1 与 P3 是因为 `files_1000_units_100__p1` 作废，不是因为别的点缺读数。
- 装置：量的那一路 `scripts/experiment-apparatus/e206_fixed_point_cost_counters.py`、`e206_fixed_point_cost_histories.rs`、入口 `e206-fixed-point-cost-run.sh`、判定库 `e206_fixed_point_cost_library.py`；推的那一路 `scripts/experiment-apparatus/e206_fixed_point_cost.rs`（bin 名 `e206-fixed-point-cost`）。变异表：模型 `scripts/mutation/tables/e206_fixed_point_cost.tsv`、插计数脚本 `e206_fixed_point_cost_counters.mutations.tsv`、历史用例 `scripts/mutation/tables/e206_fixed_point_cost_histories.tsv`（11 行，**未证红**，登记修订 1）。

```

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:21-111`（整段抄，未转述）**

````markdown
### 第一段结果

下面的行整行抄自 `research/results/e206-fixed-point-cost-report-2026-11-09.out`（任务 11102 日志里以 `name=` 起头的 753 行；同目录 `e206-fixed-point-cost-report-2026-11-09-r2.out` 是每行加 `E7RESULT ` 前缀、末尾加 `name=done` 的留存形态，复跑表比的是这一份，行号不变）。抄的时候没有改别的字；产物行引自第一段 40 份产物本身（每行带 `E206RESULT ` 前缀）。出自 P1 臂逐轮读数的行（作废条款 1）照抄不删，在各段引导句里标「作废，仅作线索」。

**F1：撞上界那一次**（P1 循环的结局，不依赖 `single_step` 对照，照报；`research/results/e206-fixed-point-cost-files_1000_units_100-p1-2026-11-09.out` 的最后三行，共 3535 行，没有 `name=item_done`）：

```text
E206RESULT name=publish item=files_1000_units_100__p1 phase=window state=same_mount op=W1 op_index=1 publish=same_mount/W1/1/0 had_caller_segment=true fixed_point_rounds=64
E206RESULT name=window_refused item=files_1000_units_100__p1 phase=window state=same_mount op=W1 op_index=1 change=overwrite cause=AllocationRecordTreeRewriteSetDidNotSettle
```

```text
name=void_6 item=files_1000_units_100__p1 problem=refused:AllocationRecordTreeRewriteSetDidNotSettle voided=this_point_this_arm
name=q1 item=files_1000_units_100__p1 point=files_1000_units_100 arm=p1 peak_rounds=64 at=same_mount/W1/1 verdict=hits_the_limit first_overwrite_rounds=64 window_rest_peak_rounds=0 history_peak_rounds=22
name=f8 item=files_1000_units_100__p1 refused=AllocationRecordTreeRewriteSetDidNotSettle refused_for_not_settling=true
name=peaks item=files_1000_units_100__p1 peak_round_work=19004 peak_final_roles=171 peak_final_nodes=415 peak_effective_rounds=19.082
name=q9 item=files_1000_units_100__p1 state=same_mount publishes=1 peak=64 rounds_at_least_3=1 rounds_at_least_5=1 rounds_at_least_9=1 last=64 falls_back=false last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=remounted publishes=0 peak=0 rounds_at_least_3=0 rounds_at_least_5=0 rounds_at_least_9=0 last=0 falls_back=true last_window_at_most_4=true recurs=false
name=q10_many_rounds item=files_1000_units_100__p1 at=same_mount/W1/1 rounds=64 release_chain_depth=85 release_chain_nodes=431 active_leaves=[273, 273] rewritten_nodes=415
name=q10 item=files_1000_units_100__p1 chain_explained_ratio=0.000 release_exit_share=1.000 placement_exit_share=0.000
name=f2 item=files_1000_units_100__p1 rounds_at_least_5=6
name=q7 item=files_1000_units_100__p1 largest_round=3036 sum_of_rounds=60850 sum_over_final_roles=355.848 premise_ratio=0.045 premise=holds memory_bytes=48056 maintenance_steps=19458 unread=0
name=q8 item=files_1000_units_100__p1 state=same_mount first_publish=2701726 peak=2701726 between_rounds_first_publish=61910 between_rounds_peak=61910 unread=0
name=q8 item=files_1000_units_100__p1 state=remounted first_publish=unread peak=unread between_rounds_first_publish=unread between_rounds_peak=unread unread=0
```

**什么形状的历史让固定点走很多轮**（Q1 判「多」的四个点与撞上界的一个点，p1 与 p2a 各一行；p1 的峰值 11、22、29、38 作废，仅作线索，撞上界那一行是 F1 的拒绝，照报）：

```text
name=q1 item=data_units_10000__p1 point=data_units_10000 arm=p1 peak_rounds=11 at=//0 verdict=many first_overwrite_rounds=3 window_rest_peak_rounds=3 history_peak_rounds=11
name=q1 item=data_units_10000__p2a point=data_units_10000 arm=p2a peak_rounds=3 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_10000__p1 point=files_10000 arm=p1 peak_rounds=22 at=same_mount/W1/1 verdict=many first_overwrite_rounds=22 window_rest_peak_rounds=6 history_peak_rounds=6
name=q1 item=files_10000__p2a point=files_10000 arm=p2a peak_rounds=4 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=4
name=q1 item=files_1000_units_100__p1 point=files_1000_units_100 arm=p1 peak_rounds=64 at=same_mount/W1/1 verdict=hits_the_limit first_overwrite_rounds=64 window_rest_peak_rounds=0 history_peak_rounds=22
name=q1 item=files_1000_units_100__p2a point=files_1000_units_100 arm=p2a peak_rounds=4 at=remounted/W2/13 verdict=not_many first_overwrite_rounds=3 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_1000_units_10__p1 point=files_1000_units_10 arm=p1 peak_rounds=29 at=same_mount/W1/1 verdict=many first_overwrite_rounds=29 window_rest_peak_rounds=3 history_peak_rounds=5
name=q1 item=files_1000_units_10__p2a point=files_1000_units_10 arm=p2a peak_rounds=3 at=//0 verdict=not_many first_overwrite_rounds=2 window_rest_peak_rounds=3 history_peak_rounds=3
name=q1 item=files_30000__p1 point=files_30000 arm=p1 peak_rounds=38 at=same_mount/W1/1 verdict=many first_overwrite_rounds=38 window_rest_peak_rounds=7 history_peak_rounds=8
name=q1 item=files_30000__p2a point=files_30000 arm=p2a peak_rounds=5 at=//0 verdict=not_many first_overwrite_rounds=3 window_rest_peak_rounds=4 history_peak_rounds=5
```

全部 `name=q1` 行按臂和档数（命令与原样输出）：

```text
$ grep '^name=q1 ' e206-fixed-point-cost-report-2026-11-09.out | sed -E 's/.*arm=([a-z_0-9]+) .*verdict=([a-z_]+) .*/\1 \2/' | sort | uniq -c
      3 closure_off not_many
      1 p1 hits_the_limit
      4 p1 many
     11 p1 not_many
     16 p2a not_many
      1 single_step many
      2 single_step not_many
```

峰值轮数落在哪一次发布（p1 的读数，作废，仅作线索）：p1 的 `files_10000`、`files_1000_units_10`、`files_30000`、`files_1000_units_100` 四个点，峰值都在 `same_mount/W1/1`（量窗第 1 次覆盖写）。`files_1000_unit_area_2_pow_24__p1` 的峰值 6 在 `same_mount/W2/16`；`data_units_1000__p1` 的 4 在 `same_mount/W3/2`；`data_units_10000__p1` 的 11 在历史里（`at=//0` 是历史段，没有量窗位置）。同一次挂载与重挂载：

```text
name=sensitivity_mount point=files_1000 same_mount_peak=4 remounted_peak=4
name=sensitivity_mount point=files_10000 same_mount_peak=22 remounted_peak=6
```

连续多次发布逐次怎么变（Q9，p1 的四个多轮点与撞上界的点；p1 的读数，作废，仅作线索）：

```text
name=q9 item=data_units_10000__p1 state=same_mount publishes=105 peak=5 rounds_at_least_3=14 rounds_at_least_5=1 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=data_units_10000__p1 state=remounted publishes=105 peak=3 rounds_at_least_3=18 rounds_at_least_5=0 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_10000__p1 state=same_mount publishes=105 peak=22 rounds_at_least_3=52 rounds_at_least_5=13 rounds_at_least_9=1 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_10000__p1 state=remounted publishes=105 peak=6 rounds_at_least_3=51 rounds_at_least_5=10 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=same_mount publishes=1 peak=64 rounds_at_least_3=1 rounds_at_least_5=1 rounds_at_least_9=1 last=64 falls_back=false last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_100__p1 state=remounted publishes=0 peak=0 rounds_at_least_3=0 rounds_at_least_5=0 rounds_at_least_9=0 last=0 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_10__p1 state=same_mount publishes=104 peak=29 rounds_at_least_3=40 rounds_at_least_5=1 rounds_at_least_9=1 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_1000_units_10__p1 state=remounted publishes=104 peak=4 rounds_at_least_3=38 rounds_at_least_5=0 rounds_at_least_9=0 last=2 falls_back=true last_window_at_most_4=true recurs=false
name=q9 item=files_30000__p1 state=same_mount publishes=104 peak=38 rounds_at_least_3=58 rounds_at_least_5=26 rounds_at_least_9=3 last=4 falls_back=false last_window_at_most_4=false recurs=true
name=q9 item=files_30000__p1 state=remounted publishes=104 peak=7 rounds_at_least_3=54 rounds_at_least_5=25 rounds_at_least_9=0 last=2 falls_back=false last_window_at_most_4=false recurs=false
```

长轮那几次的释放链（Q10；`chain_explained_ratio` 是 `|rounds − (release_chain_depth + 2)| ≤ 2` 的占比，只在 `rounds ≥ 9` 的发布上判；p1 的读数，作废，仅作线索）：

```text
name=q10_many_rounds item=files_10000__p1 at=same_mount/W1/1 rounds=22 release_chain_depth=21 release_chain_nodes=73 active_leaves=[44, 44] rewritten_nodes=73
name=q10_many_rounds item=files_1000_units_100__p1 at=same_mount/W1/1 rounds=64 release_chain_depth=85 release_chain_nodes=431 active_leaves=[273, 273] rewritten_nodes=415
name=q10_many_rounds item=files_1000_units_10__p1 at=same_mount/W1/1 rounds=29 release_chain_depth=28 release_chain_nodes=87 active_leaves=[47, 47] rewritten_nodes=87
name=q10_many_rounds item=files_30000__p1 at=same_mount/W1/1 rounds=38 release_chain_depth=36 release_chain_nodes=153 active_leaves=[129, 129] rewritten_nodes=155
$ grep '^name=q10 ' e206-fixed-point-cost-report-2026-11-09.out | grep -v not_seen
name=q10 item=files_10000__p1 chain_explained_ratio=1.000 release_exit_share=0.955 placement_exit_share=0.000
name=q10 item=files_1000__single_step chain_explained_ratio=0.000 release_exit_share=0.756 placement_exit_share=0.152
name=q10 item=files_1000_units_100__p1 chain_explained_ratio=0.000 release_exit_share=1.000 placement_exit_share=0.000
name=q10 item=files_1000_units_10__p1 chain_explained_ratio=1.000 release_exit_share=0.966 placement_exit_share=0.000
name=q10 item=files_30000__p1 chain_explained_ratio=0.667 release_exit_share=0.895 placement_exit_share=0.053
```

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:112-210`（整段抄，未转述）**

````markdown
### 岔路表

每格数据指到上面或下面的整行；「剩下的量」指登记第五节第二、三段。合并报告末尾的岔路单三行与收尾行：

```text
name=fork_row row=P1 enough_readings=false
name=fork_row row=P2 enough_readings=true
name=fork_row row=P3 enough_readings=false
name=report_done items=40 complete=39
```

| 候选 | 状态 | 依据 | 还差 |
|---|---|---|---|
| P1 今天：固定点轮数不另设目标，只有源码现有的上界 | **翻面**（登记 Q1 与 F1：任一点撞上界，P1 直接翻面）。F1 那一次拒绝是 P1 循环的结局，不依赖 `single_step` 对照，照报。**作废条款 1 记一次输**：对照 `single_step` 三点 `holds=false`，P1 的逐轮读数作废，判据不放宽，见「对照与作废的读数」；Q1 另有 4 个点判「多」（峰值 38、29、22、11）、Q2 的 `data_units` 轴 α 0.695 记「随」、Q4 的 `not_held_down` 点 7 个，这三样出自作废的读数，仅作线索。`fork_row row=P1 enough_readings=false` 只因 `files_1000_units_100__p1` 作废 | `name=verdict question=q1 arm=p1 … points_at_the_limit=files_1000_units_100`；`name=q2 arm=p1 axis=data_units … alpha=0.695 band=grows`；`name=q2 arm=p1 axis=files … alpha=0.496 band=between` | 已翻面，剩下的量只能补「什么形状」那一半：第二段 33 项（含 12 个次序点：顺序、集中、打散、建完再删等）不先跑，够判后未跑；`files_1000_units_100` 的 P1 读数补不全（撞上界即停） |
| P2 压轮数（P2a 每轮把释放链走到闭包；P2b 定期整棵重写） | **P2a 够判**（`enough_readings=true`）：Q4 `held_down`（16 个点 p2a 最大 5 轮、`E*` 最大 3.520）；Q5 不动点相同率 p2a 的 15 行都是 `ratio=1.0000`；Q6 写放大 15 行 `ratio=1.0000`。**净步数**：线性查找形态下 5 个点净多、10 个点净省，按索引形态也是同样的 5 个点净多（下面 Q5 一节）；净步数是 P2a 与 P1 逐轮读数之差，P1 一侧作废，仅作线索。P2b 没跑（登记：P2a 翻面时才要） | `name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-`；`name=q5_same_final_set`、`name=q5_net_steps`、`name=q6` 的 `__p2a` 行 | 登记 Q5 写「相同率 < 1 或索引形态下净多，P2a 翻面，P2 只剩 P2b」；净多的 5 个点都是 P1 峰值不超过 3 轮的小点，登记没写「净多」是按点还是按全部点判，这一页不判。用户已定先实现 P2a，P2b 的 Q6 不先量 |
| P3 留轮数、降每轮代价（按式子，模型算） | **Q7 前提不成立**（Q7 的输入是 P1 臂逐轮读数，作废，仅作线索）：`ρ` 在 15 个完整的 p1 点上都大于 0.5（3.026 至 149.898），`name=f6 … premise_ratio_over_the_bound=true` 15 行；唯一 `ρ=0.045` 的是作废的 `files_1000_units_100__p1`（只有一次被拒的发布）。按登记 Q7「任一不满足，P3 翻面」与 F6 字面：翻面。`fork_row row=P3 enough_readings=false`，原因同 P1 | 下面 Q7 一节的 `name=q7`、`name=f6` 整行 | 还差 `files_1000_units_100__p1` 的读数，P1 在该点撞上界，补不出；剩下的量不会让 15 个点的 `ρ` 掉到 0.5 以下 |

#### P2a 的读数

```text
name=verdict question=q1 arm=p2a points_many=- points_at_the_limit=-
name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-
name=verdict question=q2 arm=p2a grows_on=data_units
name=verdict question=q10 arm=p2a points_the_chain_does_not_explain=-
name=q2 arm=p2a axis=files low_end=39 high_end=95 alpha=0.129 band=does_not_grow local_slopes=0.000,0.208,0.178
name=q2 arm=p2a axis=data_units low_end=39 high_end=1272 alpha=0.504 band=grows local_slopes=0.931,0.578,0.004
name=q2 arm=p2a axis=unlinks low_end=39 high_end=57 alpha=0.055 band=does_not_grow local_slopes=0.000,0.032,0.133
name=q3_final_roles arm=p2a axis=files low_end=22 high_end=84 alpha=0.194 band=does_not_grow local_slopes=0.000,0.056,0.526
name=q3_final_roles arm=p2a axis=data_units low_end=22 high_end=441 alpha=0.434 band=between local_slopes=0.714,0.570,0.017
name=q3_final_roles arm=p2a axis=unlinks low_end=22 high_end=26 alpha=0.024 band=does_not_grow local_slopes=0.000,0.000,0.073
name=f4 arm=p2a points=-
```

Q4 的 P2a 行 `points_not_held_down=-` 的意思：16 个点上 `R* ≤ 8` 且 `E* ≤ 4`。`R*` 最大的是 `files_30000__p2a` 的 5 轮（P1 同点 38 轮），其余点 3 到 4 轮。`q2` 的 `data_units` 轴 P2a 也记「随」（α 0.504）：那条轴上真要写的角色数本身在涨（`q3_final_roles` 的 `data_units` 轴 α 0.434，两臂相同），P2a 动不了它，F5 三根轴都 `triggered=false`。

Q5：同一段历史上 P2a 与 P1 的不动点逐个比（命令与原样输出）：

```text
$ grep '^name=q5_same_final_set item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'ratio=1.0000 first_difference=None same_image_at_the_history_end=true final_set_differs=false'
15
$ grep '^name=q6 item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'ratio=1.0000 verdict=acceptable'
15
$ grep '^name=q5_net_steps item=.*__p2a ' e206-fixed-point-cost-report-2026-11-09.out | grep -o 'linear=[a-z_]* indexed=[a-z_]*' | sort | uniq -c
      5 linear=costs_more indexed=costs_more
     10 linear=saves indexed=saves
```

净步数整行（`net_steps_linear` 是线性查找形态，`net_steps_indexed` 是按索引；负数是省）：

```text
name=q5_net_steps item=base__p2a publishes_with_at_least_3_rounds=26 net_steps_linear=1338 net_steps_indexed=936 linear=costs_more indexed=costs_more
name=q5_net_steps item=data_units_10000__p2a publishes_with_at_least_3_rounds=32 net_steps_linear=-10334626 net_steps_indexed=-10349720 linear=saves indexed=saves
name=q5_net_steps item=data_units_1000__p2a publishes_with_at_least_3_rounds=37 net_steps_linear=-3836827 net_steps_indexed=-3838860 linear=saves indexed=saves
name=q5_net_steps item=data_units_100__p2a publishes_with_at_least_3_rounds=29 net_steps_linear=1492 net_steps_indexed=1044 linear=costs_more indexed=costs_more
name=q5_net_steps item=files_10000__p2a publishes_with_at_least_3_rounds=103 net_steps_linear=-372091510 net_steps_indexed=-372193553 linear=saves indexed=saves
name=q5_net_steps item=files_1000__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-21674834 net_steps_indexed=-21681554 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_14__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-6144074 net_steps_indexed=-6147399 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_20__p2a publishes_with_at_least_3_rounds=82 net_steps_linear=-140290898 net_steps_indexed=-140297618 linear=saves indexed=saves
name=q5_net_steps item=files_1000_unit_area_2_pow_24__p2a publishes_with_at_least_3_rounds=86 net_steps_linear=-2071221471 net_steps_indexed=-2071231185 linear=saves indexed=saves
name=q5_net_steps item=files_1000_units_10__p2a publishes_with_at_least_3_rounds=78 net_steps_linear=-328136791 net_steps_indexed=-328222922 linear=saves indexed=saves
name=q5_net_steps item=files_100__p2a publishes_with_at_least_3_rounds=47 net_steps_linear=2432 net_steps_indexed=1692 linear=costs_more indexed=costs_more
name=q5_net_steps item=files_30000__p2a publishes_with_at_least_3_rounds=112 net_steps_linear=-2662924587 net_steps_indexed=-2663330038 linear=saves indexed=saves
name=q5_net_steps item=unlinks_10000__p2a publishes_with_at_least_3_rounds=51 net_steps_linear=-41746236 net_steps_indexed=-41750799 linear=saves indexed=saves
name=q5_net_steps item=unlinks_1000__p2a publishes_with_at_least_3_rounds=27 net_steps_linear=1390 net_steps_indexed=972 linear=costs_more indexed=costs_more
name=q5_net_steps item=unlinks_100__p2a publishes_with_at_least_3_rounds=23 net_steps_linear=1186 net_steps_indexed=828 linear=costs_more indexed=costs_more
```

#### P3 的读数

```text
name=q7 item=base__p1 largest_round=1391 sum_of_rounds=2647 sum_over_final_roles=120.318 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7 item=data_units_10000__p1 largest_round=1033610 sum_of_rounds=1379231 sum_over_final_roles=3127.508 premise_ratio=149.898 premise=fails memory_bytes=496920 maintenance_steps=308502 unread=0
name=q7 item=data_units_1000__p1 largest_round=85778 sum_of_rounds=107776 sum_over_final_roles=254.189 premise_ratio=78.074 premise=fails memory_bytes=51704 maintenance_steps=25396 unread=0
name=q7 item=data_units_100__p1 largest_round=7690 sum_of_rounds=10104 sum_over_final_roles=88.632 premise_ratio=12.286 premise=fails memory_bytes=6920 maintenance_steps=2894 unread=0
name=q7 item=files_10000__p1 largest_round=6089 sum_of_rounds=16116 sum_over_final_roles=191.857 premise_ratio=3.290 premise=fails memory_bytes=13224 maintenance_steps=5996 unread=0
name=q7 item=files_1000__p1 largest_round=2373 sum_of_rounds=5609 sum_over_final_roles=224.360 premise_ratio=3.379 premise=fails memory_bytes=4984 maintenance_steps=2164 unread=0
name=q7 item=files_1000_unit_area_2_pow_14__p1 largest_round=2041 sum_of_rounds=4488 sum_over_final_roles=195.130 premise_ratio=3.333 premise=fails memory_bytes=4016 maintenance_steps=1775 unread=0
name=q7 item=files_1000_unit_area_2_pow_20__p1 largest_round=2375 sum_of_rounds=5617 sum_over_final_roles=224.680 premise_ratio=3.379 premise=fails memory_bytes=4984 maintenance_steps=2164 unread=0
name=q7 item=files_1000_unit_area_2_pow_24__p1 largest_round=2617 sum_of_rounds=6244 sum_over_final_roles=231.259 premise_ratio=3.037 premise=fails memory_bytes=5640 maintenance_steps=2460 unread=0
name=q7 item=files_1000_units_100__p1 largest_round=3036 sum_of_rounds=60850 sum_over_final_roles=355.848 premise_ratio=0.045 premise=holds memory_bytes=48056 maintenance_steps=19458 unread=0
name=q7 item=files_1000_units_10__p1 largest_round=7041 sum_of_rounds=18889 sum_over_final_roles=190.798 premise_ratio=3.243 premise=fails memory_bytes=15552 maintenance_steps=7029 unread=0
name=q7 item=files_100__p1 largest_round=1457 sum_of_rounds=2823 sum_over_final_roles=128.318 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7 item=files_30000__p1 largest_round=13346 sum_of_rounds=36363 sum_over_final_roles=211.413 premise_ratio=3.026 premise=fails memory_bytes=27112 maintenance_steps=13760 unread=0
name=q7 item=unlinks_10000__p1 largest_round=2147 sum_of_rounds=6351 sum_over_final_roles=244.269 premise_ratio=3.360 premise=fails memory_bytes=6056 maintenance_steps=2626 unread=0
name=q7 item=unlinks_1000__p1 largest_round=1473 sum_of_rounds=2899 sum_over_final_roles=131.773 premise_ratio=3.440 premise=fails memory_bytes=2872 maintenance_steps=1026 unread=0
name=q7 item=unlinks_100__p1 largest_round=1395 sum_of_rounds=2651 sum_over_final_roles=120.500 premise_ratio=3.714 premise=fails memory_bytes=2600 maintenance_steps=914 unread=0
name=q7_sum_of_rounds arm=p1 axis=files low_end=2647 high_end=16116 alpha=0.262 band=between local_slopes=0.028,0.298,0.458
name=q7_memory_bytes arm=p1 axis=files low_end=2600 high_end=13224 alpha=0.235 band=does_not_grow local_slopes=0.000,0.283,0.424
name=q7_sum_of_rounds arm=p1 axis=data_units low_end=2647 high_end=1379231 alpha=0.906 band=grows local_slopes=0.582,1.028,1.107
name=q7_memory_bytes arm=p1 axis=data_units low_end=2600 high_end=496920 alpha=0.760 band=grows local_slopes=0.425,0.873,0.983
name=q7_sum_of_rounds arm=p1 axis=unlinks low_end=2647 high_end=6351 alpha=0.127 band=does_not_grow local_slopes=0.001,0.039,0.341
name=q7_memory_bytes arm=p1 axis=unlinks low_end=2600 high_end=6056 alpha=0.122 band=does_not_grow local_slopes=0.000,0.043,0.324
$ grep '^name=f6 ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'premise_ratio_over_the_bound=true roles_changed_over_twice_the_final=false'
15
name=f6 item=files_1000_units_100__p1 premise_ratio_over_the_bound=false roles_changed_over_twice_the_final=true
```

`ρ` 是登记 Q7 的 `Σ_r records_changed_between_rounds_r ÷ W`，表中 `premise_ratio`；`premise=fails` 是 `ρ > 0.5`。`Σ_r` 的 α：`files` 轴 0.262（介于）、`data_units` 轴 0.906（随）、`unlinks` 轴 0.127（不随）；内存峰值的 α：0.235、0.760、0.122。登记 Q7 要三根轴都不随、前提成立、内存不随，P3 才站住。这些是推的：P3 的式子写在登记第五节，输入是量的（轮间差读数），出自 P1 臂逐轮读数，作废，仅作线索。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:211-268`（整段抄，未转述）**

````markdown
### B8 那一格的读数

E203（准入之前扫类的步代价） 登记第十二节修订 8：B8 剩下的一格（`files_10000` 同一次挂载覆盖写第 1 轮，固定点 22 轮，Q6 α 0.620）记「不稳定」，固定点这一笔量完照原判据再判；判决 `research/prompts/e203-forks-r2-main-verification.md` 第三节「B8」一行把「判仍随还是不稳定」交用户定读法。**原判据**（E203（准入之前扫类的步代价） 登记 Q6、F2 与第八节第 1 条）：

- Q6：每根轴一档，全部轴 α ≤ 0.25 记「B8 不随总数涨」，任一轴 α ≥ 0.5 记「B8 仍随总数涨」，之间不判。
- F2：任一根轴、任一种挂载状态、任一种被量的发布上 α ≥ 0.5，B8 行翻面。
- 第八节第 1 条：两种挂载状态落在不同档的，那一格记「不稳定」。

E206（发布固定点轮数的代价） 的 Q8 把每次发布逐轮的输入喂 E203（准入之前扫类的步代价） 的 B8 列。`q8_first_publish` 取量窗 W1 第 1 次覆盖写（同一次挂载），`q8_peak` 取量窗全部发布（同一次挂载加重挂载）的最大值，两个读法都报；`q8_p3_*` 是同样的发布换成 P3 的轮间差输入（登记 Q7、第五节 P3 式子）。整行（P1、P2a 两臂，三根轴）：

```text
name=q8_first_publish arm=p1 axis=files low_end=2102 high_end=152344 alpha=0.620 band=grows local_slopes=0.045,0.475,1.340
name=q8_peak arm=p1 axis=files low_end=3763 high_end=152344 alpha=0.536 band=grows local_slopes=0.030,0.356,1.221
name=q8_p3_first_publish arm=p1 axis=files low_end=1787 high_end=16538 alpha=0.322 band=between local_slopes=0.048,0.314,0.605
name=q8_p3_peak arm=p1 axis=files low_end=2794 high_end=16538 alpha=0.257 band=between local_slopes=0.030,0.297,0.445
name=q8_first_publish arm=p1 axis=data_units low_end=2102 high_end=5958 alpha=0.151 band=does_not_grow local_slopes=0.036,0.374,0.043
name=q8_peak arm=p1 axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
name=q8_p3_first_publish arm=p1 axis=data_units low_end=1787 high_end=3449 alpha=0.095 band=does_not_grow local_slopes=0.037,0.208,0.040
name=q8_p3_peak arm=p1 axis=data_units low_end=2794 high_end=1379969 alpha=0.898 band=grows local_slopes=0.565,1.023,1.107
name=q8_first_publish arm=p1 axis=unlinks low_end=2102 high_end=12789 alpha=0.261 band=between local_slopes=0.031,0.083,0.670
name=q8_peak arm=p1 axis=unlinks low_end=3763 high_end=12789 alpha=0.177 band=does_not_grow local_slopes=0.000,0.063,0.467
name=q8_p3_first_publish arm=p1 axis=unlinks low_end=1787 high_end=6577 alpha=0.189 band=does_not_grow local_slopes=0.026,0.078,0.462
name=q8_p3_peak arm=p1 axis=unlinks low_end=2794 high_end=6577 alpha=0.124 band=does_not_grow local_slopes=0.001,0.045,0.326
name=q8_first_publish arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526
name=q8_peak arm=p2a axis=files low_end=3763 high_end=15440 alpha=0.204 band=does_not_grow local_slopes=0.030,0.350,0.233
name=q8_p3_first_publish arm=p2a axis=files low_end=1787 high_end=16185 alpha=0.319 band=between local_slopes=0.048,0.312,0.598
name=q8_p3_peak arm=p2a axis=files low_end=2794 high_end=16185 alpha=0.254 band=between local_slopes=0.030,0.181,0.551
name=q8_first_publish arm=p2a axis=data_units low_end=2102 high_end=4040 alpha=0.095 band=does_not_grow local_slopes=0.036,0.201,0.047
name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255
name=q8_p3_first_publish arm=p2a axis=data_units low_end=1787 high_end=3430 alpha=0.094 band=does_not_grow local_slopes=0.037,0.206,0.040
name=q8_p3_peak arm=p2a axis=data_units low_end=2794 high_end=1379969 alpha=0.898 band=grows local_slopes=0.565,1.023,1.107
name=q8_first_publish arm=p2a axis=unlinks low_end=2102 high_end=7184 alpha=0.178 band=does_not_grow local_slopes=0.031,0.083,0.420
name=q8_peak arm=p2a axis=unlinks low_end=3763 high_end=7184 alpha=0.094 band=does_not_grow local_slopes=0.000,0.063,0.217
name=q8_p3_first_publish arm=p2a axis=unlinks low_end=1787 high_end=4062 alpha=0.119 band=does_not_grow local_slopes=0.026,0.078,0.252
name=q8_p3_peak arm=p2a axis=unlinks low_end=2794 high_end=4186 alpha=0.059 band=does_not_grow local_slopes=0.001,0.045,0.130
```

由上面整行抄出的 α 与档（不是新算的）：

| 臂 | 轴 | `q8_first_publish` | `q8_peak` |
|---|---|---|---|
| P1 | `files` | 0.620 随（2102 → 152344） | 0.536 随（3763 → 152344） |
| P1 | `data_units` | 0.151 不随 | 0.978 随（3763 → 3231038） |
| P1 | `unlinks` | 0.261 介于 | 0.177 不随 |
| P2a | `files` | 0.289 介于（2102 → 15440） | 0.204 不随（3763 → 15440） |
| P2a | `data_units` | 0.095 不随 | 0.978 随（3763 → 3231038） |
| P2a | `unlinks` | 0.178 不随 | 0.094 不随 |

照原判据的字面：

- **P1**（输入是 P1 臂逐轮读数，作废，仅作线索）：两个读法下都有轴 α ≥ 0.5（`q8_first_publish` 的 `files` 0.620，`q8_peak` 的 `files` 0.536 与 `data_units` 0.978），Q6 记「B8 仍随总数涨」，F2 打中。`files` 轴的 2102 → 152344 与 E203（准入之前扫类的步代价） 页同一格同数（`files_10000` 同一次挂载覆盖写第 1 轮，22 轮）。
- **P2a**：`q8_first_publish` 读法下三根轴都没到 0.5，但 `files` 轴 0.289 大于 0.25，不满足「全部轴 ≤ 0.25」，两档都不是（原判据对这一档没给结论）；同一格的发布，P1 是 152344，P2a 是 15440。`q8_peak` 读法下 `data_units` 轴 α 0.978 ≥ 0.5，Q6 记「仍随」，F2 打中。
- `data_units` 轴峰值 3231038 在 P1 与 P2a 两臂完全相同（`name=q8 item=data_units_10000__p1` 与 `__p2a` 的 `state=same_mount … peak=3231038`），量窗里 `final_roles` 最大的发布是 `same_mount` W2 第 1 次（66 个角色，两臂都 3 轮，用 `name=settle` 行排出来的，没读 `q8_peak` 取的是哪一次，**未核**）；P2a 动不了这一格，它不来自固定点轮数。E203（准入之前扫类的步代价） 页 Q6 的 D 轴 6 格都「不随」；E206（发布固定点轮数的代价） 的 `q8_peak` 取量窗里全部发布（含 W2 冷文件、W3 建删），发布集合与 E203（准入之前扫类的步代价） 的五轮覆盖写 / unlink 不同，两个口径不能直接对比。
- `q8_p3_*` 行（P3 轮间差输入，P1 臂一侧作废，仅作线索）：P1 臂的历史上，`files` 轴 first_publish 0.322、peak 0.257，都介于；`data_units` 轴 0.095 不随、0.898 随；`unlinks` 轴 0.189、0.124，都不随；这些是推的。
- 重挂载一栏在逐点 `name=q8 … state=remounted` 行里，两臂相同或 P2a 更小；`files_10000` 重挂载 first_publish 8818，同一次挂载 152344（P1）与 15440（P2a）。第八节第 1 条「两种挂载状态落在不同档记不稳定」在这一格是否触发，要把 `state=remounted` 的 α 另算，报告没打这一行，**没判**。

这一节只写数与原判据；「选读法」「B8 判仍随还是不稳定」「P1 与 P2a 选哪条」都不在这里。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:269-340`（整段抄，未转述）**

````markdown
### 对照与作废的读数

登记第十一节作废条款逐条的结局（整行见上、下；每条的命令与原样输出在执行员报告）：

| 条款 | 读数 | 结局 |
|---|---|---|
| 1 `single_step` 对照 | 三个点 `holds=false` | **记一次输**：P1 的逐轮读数按字面整段作废，判据不放宽；成因见下 |
| 2 `closure_off` 对照 | 三个点 `holds=true` | P2a 不作废 |
| 3 P3 对照 | `control_p3` 38 行 `holds=true` | 不作废 |
| 4 阴性对照 | p1、p2a 三根轴 `holds=true`；p2b 六行 `result=unread`（P2b 没跑） | p1、p2a 不作废 |
| 5 `--build plain` 比盘 | `base`、`files_1000` 两点 `same=true` | 不作废 |
| 6 缺 `item_done` | `files_1000_units_100__p1` | 这个点这条臂作废 |
| 7 对账 | 24 行里 8 行 `equal=false`（全是 W2，30 对 32） | 查明是判据少算，不是计数错；第 2 组不作废，见下 |

```text
name=anchor_measured anchor_identifier=files_10000_first_overwrite rounds=22 expected_rounds=22 round_work=1201 expected_round_work=1201 per_round_roles=[11, 24, 28, 32, 36, 40, 42, 44, 46, 50, 54, 58, 62, 64, 66, 70, 74, 76, 78, 80, 82, 84] matches=true
name=anchor_measured anchor_identifier=files_10000_second_overwrite rounds=Some(2) expected_rounds=2 matches=true
name=anchor_measured anchor_identifier=base_first_overwrite rounds=2 expected_rounds=2 round_work=21 expected_round_work=21 per_round_roles=[7, 14] matches=true
name=control_single_step point=base holds=false voids=p1_round_readings_if_false
name=control_closure_off point=base holds=true voids=p2a_if_false
name=control_single_step point=files_100 holds=false voids=p1_round_readings_if_false
name=control_closure_off point=files_100 holds=true voids=p2a_if_false
name=control_single_step point=files_1000 holds=false voids=p1_round_readings_if_false
name=control_closure_off point=files_1000 holds=true voids=p2a_if_false
name=void_5 point=base plain=Some("a97bb56210604ecb68cd4f689ec0f525a646552bc144c6b0c303b5d4079306bf") today=Some("a97bb56210604ecb68cd4f689ec0f525a646552bc144c6b0c303b5d4079306bf") same=true voids=all_measured_if_false
name=void_5 point=files_1000 plain=Some("c82a604c4590edda8a2f38f2f8bbad0096f5ccae6508337bf5bf6f574025af19") today=Some("c82a604c4590edda8a2f38f2f8bbad0096f5ccae6508337bf5bf6f574025af19") same=true voids=all_measured_if_false
name=control_negative arm=p1 axis=files low_final_roles=10 high_final_roles=13 alpha=0.038 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p1 axis=data_units low_final_roles=10 high_final_roles=10 alpha=0.000 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p1 axis=unlinks low_final_roles=10 high_final_roles=14 alpha=0.049 rounds=2 holds=true voids=p1_if_false
name=control_negative arm=p2a axis=files low_final_roles=10 high_final_roles=13 alpha=0.038 rounds=2 holds=true voids=p2a_if_false
name=control_negative arm=p2a axis=data_units low_final_roles=10 high_final_roles=10 alpha=0.000 rounds=2 holds=true voids=p2a_if_false
name=control_negative arm=p2a axis=unlinks low_final_roles=10 high_final_roles=14 alpha=0.049 rounds=2 holds=true voids=p2a_if_false
$ grep '^name=control_p3 ' e206-fixed-point-cost-report-2026-11-09.out | grep -c 'holds=true'
38
```

**`single_step` 对照三点 `holds=false`**。模型的 `single_step_control_holds`（`e206_fixed_point_cost.rs:1259`）四个条件：不动点逐次相同、每个定下的固定点轮数不少于节点数、量窗至少一次比 p1 多轮、历史末 `image_sha256` 相同。执行员另写一份脚本逐条核（脚本在执行员草稿目录 `single-step-control-check.py`，起于 `research/results` 目录；括号里依次是历史第几次、p1 轮数、`single_step` 轮数、节点数，量窗的一项依次是 op、op_index、轮数、节点数）：

```text
$ python3 -I single-step-control-check.py
point=base history_calls=44/44 same_set_history=True window_publishes=210/210 same_set_window=True window_slower=210 image_sha256_equal=True settled_with_rounds_below_final_nodes=1 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5')]
point=files_100 history_calls=134/134 same_set_history=True window_publishes=208/208 same_set_window=True window_slower=208 image_sha256_equal=True settled_with_rounds_below_final_nodes=1 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5')]
point=files_1000 history_calls=1043/1043 same_set_history=True window_publishes=208/208 same_set_window=True window_slower=208 image_sha256_equal=True settled_with_rounds_below_final_nodes=2 [(0, 'p1', '2', 'single_step', '2', 'final_nodes', '5'), (917, 'p1', '3', 'single_step', '8', 'final_nodes', '9')]
```

不动点逐次相同（历史与量窗的 `final_set_hash` 与角色数）、量窗每一次都更慢、历史末盘面相同三条在三个点上都成立；不成立的是第二条：`base` 与 `files_100` 各有 1 个、`files_1000` 有 2 个定下的固定点，`single_step` 的轮数小于它的节点数。第 0 个是历史里的第 1 次固定点（两臂都是 2 轮、5 个节点、同一个哈希，`single_step` 没有让它变慢；是不是 `:1903` 那一份同形的行发布固定点，没核，那一份的补丁登记第五节没写）；`files_1000` 的第 917 个是 p1 3 轮、`single_step` 8 轮、9 个节点，补丁每轮只并一个节点，第 1 轮规划自己罩住的节点不在差集里，所以 8 轮对 9 个节点，登记「至少一个节点一轮」的假设与循环写法不符。这是执行员读码推的成因，登记没写，也没有别的读数证它。

判定（主 agent，`records/2026-10-13-里程碑四预开发.md` 第 1579 行）：三点 `holds=false`（合并报告 569、571、573 行）字面触发作废条款 1，记一次输，P1 逐轮读数作废，判据不放宽；F1 那一次拒绝是 P1 循环的结局，不依赖这个对照。

**对账 W2 的 8 行 `equal=false`**（30 对 32，`base` 与 `files_100` 的 p1、p2a、`single_step`、`closure_off` 全是同一对数）：

```text
name=reconciliation item=base__closure_off op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__p1 op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__p2a op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=base__single_step op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__closure_off op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__p1 op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__p2a op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
name=reconciliation item=files_100__single_step op=W2 image_changed_unit_slots=30 slots_placed=32 equal=false
```

`plain` 构建不插计数，它的 W2 `image_changed_unit_slots=30` 与 p1 的盘面读数相同（`research/results/e206-fixed-point-cost-base-plain-2026-11-09.out`），所以盘这一边是 30，计数打的 `slots_placed` 是 32（W1 与 W3 都相等）。登记作废条款 7 要先按停机条款 2 两边查、查明是计数错才作废。两边查了：探针任务 11147 的整行（抄自 `records/2026-10-13-里程碑四预开发.md` 第 1580 行），与调查员跑前写下的预期逐字段相同：

```text
E206PROBE name=op_summary item=base__p1 phase=window state=same_mount op=W2 op_index=1 counter_placed=32 allocator_placed=32 image_changed=30 placed_slots=32 placed_unchanged=2 changed_not_placed=0 placed_changed_without_a_write=0 placed_unchanged_without_a_write=0 placed_unchanged_not_all_zero=0
```

反事实任务 11148（W2 第 1 次内容加长、尾槽不全零）：`counter_placed=32 allocator_placed=32 image_changed=32 placed_slots=32 placed_unchanged=0`。判定：不是计数错，是判据少算——`image_changed_unit_slots` 量的是内容变了的槽，全零尾槽再写一遍不算变（32 个槽放下去、2 个没变），所以第 2 组不作废；要动判据写登记修订，这一次没动。W3 那 32 行 `counter_placed=116 allocator_placed=34 image_changed=116`：`counter` 与 `image` 相等，`allocator` 少是探针取的分配器范围（推的，没核）。探针副本已删。

**停机条款 3（产物头记的源码 sha256 与登记不符）**：40 份产物里 34 份头行记的 `transaction.rs`、`allocator.rs` 与登记相同，`files_30000`、`files_1000_units_100`、`files_1000_unit_area_2_pow_24` 三点的 6 份记的是别的值（`transaction.rs` 都是 `602f9bbb…`，`allocator.rs` p1 是 `3c91258f…`、p2a 是 `ff676177…`），三点内 p1 与 p2a 的 `allocator.rs` 也不同。成因：入口 `scripts/experiment-apparatus/e206-fixed-point-cost-run.sh:170` 的 `header_for` 读 `$source_tree`（仓），头行记的是写头行那一刻的仓，不是编进去的副本，仓里 `crates/` 那几个文件之后被改过。执行员核了副本：三组副本（`small`、`bounds`、`ends`）里这三个文件都等于 `git show HEAD:` 的原文套上 `research/results/e206-fixed-point-cost-counters-diff-2026-11-09.diff`（1989 行，三组副本里的同名 diff 逐字节相同），而 HEAD 的三个文件 sha256 正是登记写的。所以编进去的是登记的源码：头行记错，不是编错。这一条证据是「三组副本现状」，不是编译当时的记录。调查员的补丁（头行改读副本）没打：读副本之后 `counts` 产物每趟都触发停机条款 3。欠账去向是登记修订。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:341-384`（整段抄，未转述）**

````markdown
### 判据与失败条款

判决行（整行，合并报告里全部 `name=verdict`）：

```text
name=verdict question=q1 arm=p1 points_many=data_units_10000,files_10000,files_1000_units_10,files_30000 points_at_the_limit=files_1000_units_100
name=verdict question=q4 arm=p1 verdict=not_held_down points_not_held_down=data_units_10000,files_10000,files_1000_unit_area_2_pow_24,files_1000_units_100,files_1000_units_10,files_30000,unlinks_10000
name=verdict question=q2 arm=p1 grows_on=data_units
name=verdict question=q10 arm=p1 points_the_chain_does_not_explain=files_1000_units_100,files_30000
name=verdict question=q1 arm=p2a points_many=- points_at_the_limit=-
name=verdict question=q4 arm=p2a verdict=held_down points_not_held_down=-
name=verdict question=q2 arm=p2a grows_on=data_units
name=verdict question=q10 arm=p2a points_the_chain_does_not_explain=-
```

- F1：打中（上面），照报，不依赖 `single_step` 对照。
- F2（源码注释「三四轮就定」）：打中，只记注释与读数不符，不判臂。`rounds ≥ 5` 的发布次数，p1 臂各点（`0` 的不列；含 `single_step`、`closure_off` 对照；p1 与 `single_step` 的读数，作废，仅作线索）：

```text
$ grep '^name=f2 ' e206-fixed-point-cost-report-2026-11-09.out | grep -v 'rounds_at_least_5=0'
name=f2 item=base__single_step rounds_at_least_5=253
name=f2 item=data_units_10000__p1 rounds_at_least_5=2
name=f2 item=files_10000__p1 rounds_at_least_5=232
name=f2 item=files_1000__closure_off rounds_at_least_5=4
name=f2 item=files_1000__p1 rounds_at_least_5=4
name=f2 item=files_1000__single_step rounds_at_least_5=1250
name=f2 item=files_1000_unit_area_2_pow_14__p1 rounds_at_least_5=7
name=f2 item=files_1000_unit_area_2_pow_20__p1 rounds_at_least_5=4
name=f2 item=files_1000_unit_area_2_pow_24__p1 rounds_at_least_5=13
name=f2 item=files_1000_units_100__p1 rounds_at_least_5=6
name=f2 item=files_1000_units_10__p1 rounds_at_least_5=2
name=f2 item=files_100__single_step rounds_at_least_5=341
name=f2 item=files_30000__p1 rounds_at_least_5=1202
name=f2 item=files_30000__p2a rounds_at_least_5=1
name=f2 item=unlinks_10000__p1 rounds_at_least_5=29
```

- F3（P2a 不是原不动点）：没打中，P2a 与 P1 的不动点 15 行逐个相同、历史末盘面相同（「P2a 的读数」一节的命令）。
- F4（P2a 碰不到轮数）：没打中，`name=f4 arm=p2a points=-`。
- F5（三臂共用部分在涨）：没打中，三根轴 `triggered=false`；`q3_final_roles` 的 `data_units` 轴 α 0.434 介于。
- F6（P3 前提不成立）：打中（读数出自 P1 臂逐轮读数，作废，仅作线索），15 个完整 p1 点的 `ρ` 都大于 0.5；`roles_changed_over_twice_the_final` 15 个点是 `false`。
- F7（释放链解释不了长轮）：打中（出自 P1 臂逐轮读数，作废，仅作线索），`name=verdict question=q10 arm=p1 points_the_chain_does_not_explain=files_1000_units_100,files_30000`：`files_1000_units_100` 的 64 轮被上界截断（`release_chain_depth=85`，+2 超过 64），`files_30000` 的 `chain_explained_ratio=0.667`（`release_exit_share=0.895`、`placement_exit_share=0.053`）。`files_10000` 与 `files_1000_units_10` 的 `chain_explained_ratio=1.000`。登记第四节第 12 条的假设（轮数约等于每盘活跃叶数）在这两点的预测（`files_30000` 约 74、`files_1000_units_100` 约 247）没对上：实测 38 轮（作废，仅作线索）、64 轮被拒（F1，照报），活跃叶数 129、273。
- F8（历史走不完）：被拒的只有 F1 那一次，是量窗里被拒、不是历史里；`history_refused` 在 40 份产物里 0 行（`grep -c 'name=history_refused'`，没有非 0 的文件）。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:385-398`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 合并报告：40 份产物重算一遍，与留存产物逐字节比（exact）；不重跑历史。登记行写好，还没跑过
bash scripts/experiment-apparatus/replay.sh --item E206.1
# 40 项里的一项：量的那一路，副本由入口拷
bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --item files_10000__p1 --work <新目录>
bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --list-items
# 合并报告的原始命令
cargo run --release --bin e206-fixed-point-cost -- report <40 份产物>
```

每条命令由主 agent 经 `task-server/task submit` 交；工作目录放共享根下新建。第一段整趟另有 `--segment first`。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:399-452`（整段抄，未转述）**

````markdown
### 口径

- 合并报告：`research/results/e206-fixed-point-cost-report-2026-11-09.out`（753 行，任务 11102 日志）；留存形态 `research/results/e206-fixed-point-cost-report-2026-11-09-r2.out`（每行加 `E7RESULT ` 前缀、末尾加 `name=done emitted=754`，由上一份用同一个 awk 得到，不是重跑）。
- 插计数的 diff：`research/results/e206-fixed-point-cost-counters-diff-2026-11-09.diff`（1989 行，计数构建的 38 份产物头行 `diff_file=`）；`research/results/e206-fixed-point-cost-counters-diff-2026-11-09-r2.diff` 是空文件，是 `plain` 构建的 diff（两份 `plain` 产物头行 `diff_file=e206-fixed-point-cost-counters-diff-2026-11-09-r2.diff`，`diff_lines=0`）。
- 40 份产物与它们的完成标记在工作区里（都没归档出去）：

```text
$ ls e206-fixed-point-cost-*-2026-11-09.out | grep -v 'report'
e206-fixed-point-cost-base-closure_off-2026-11-09.out
e206-fixed-point-cost-base-p1-2026-11-09.out
e206-fixed-point-cost-base-p2a-2026-11-09.out
e206-fixed-point-cost-base-plain-2026-11-09.out
e206-fixed-point-cost-base-single_step-2026-11-09.out
e206-fixed-point-cost-data_units_10000-p1-2026-11-09.out
e206-fixed-point-cost-data_units_10000-p2a-2026-11-09.out
e206-fixed-point-cost-data_units_1000-p1-2026-11-09.out
e206-fixed-point-cost-data_units_1000-p2a-2026-11-09.out
e206-fixed-point-cost-data_units_100-p1-2026-11-09.out
e206-fixed-point-cost-data_units_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_10000-p1-2026-11-09.out
e206-fixed-point-cost-files_10000-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000-closure_off-2026-11-09.out
e206-fixed-point-cost-files_1000-p1-2026-11-09.out
e206-fixed-point-cost-files_1000-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000-plain-2026-11-09.out
e206-fixed-point-cost-files_1000-single_step-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_14-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_14-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_20-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_20-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_24-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_unit_area_2_pow_24-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_units_100-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_units_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_1000_units_10-p1-2026-11-09.out
e206-fixed-point-cost-files_1000_units_10-p2a-2026-11-09.out
e206-fixed-point-cost-files_100-closure_off-2026-11-09.out
e206-fixed-point-cost-files_100-p1-2026-11-09.out
e206-fixed-point-cost-files_100-p2a-2026-11-09.out
e206-fixed-point-cost-files_100-single_step-2026-11-09.out
e206-fixed-point-cost-files_30000-p1-2026-11-09.out
e206-fixed-point-cost-files_30000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_10000-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_10000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_1000-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_1000-p2a-2026-11-09.out
e206-fixed-point-cost-unlinks_100-p1-2026-11-09.out
e206-fixed-point-cost-unlinks_100-p2a-2026-11-09.out
```

- 点：`U` 默认 2^17，`u`、`D`、`F` 各轴见登记第五节取样点表；历史次序与量窗 W1 到 W4 见登记第五节。固定点的「一次」「轮」「轮功」「有效轮数」的定义逐字在登记第一节「读法」。
- 每轮重写的角色数 `A_r` 与节点数 `N_r` 都报；判据只吃 `A_r`。
- 不量挂钟（登记读法第 8 条）。

````

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:453-464`（整段抄，未转述）**

```markdown
### 它答不了的

- 只在 6 MiB 环、两块盘、登记第五节那几种历史形状上量；第二段 33 项（12 个次序点：顺序、集中、打散、建完再删、一次一个 inode、每 1000 次重挂载；加三个轴末点的 P2b16、P2b256、`whole`）与第三段全量点没跑，是够判后未跑，「什么形状」的问法在那些次序上翻不翻第一段的判，这一页答不了。
- P2b 没量：P2b 的写放大（Q6）只有 P2a 的 15 行（比值 1.0000，P2a 本来就不多写），P2b 两档的写放大没有读数。
- P3 是式子，不是实现：Q7 的 `ρ`、轮间差之和、内存是模型从逐轮读数算的，没有实现过「每轮只碰变了的」。
- 逐轮读数作废：`single_step` 对照三点 `holds=false`（上一节），作废条款 1 记一次输，P1 逐轮读数整段作废，仅作线索，判据不放宽。F1 那一次被拒是一次观测到的结局（`window_refused`），它依赖的是 P1 臂原样的循环，不依赖 `single_step` 臂的补丁。
- 对账 W2 的 32 对 30 查明是判据少算，不是计数错（上一节）；判据没动，要动写登记修订。W3 的 `allocator_placed=34` 对 `counter_placed=116` 是探针取的分配器范围，推的，没核。
- 产物头行的 `source_sha256` 在 6 份产物上记了写头行时的仓，不是编进去的副本（上一节），要查编译当时的源码只能靠三组副本现状；头行读副本的补丁没打，欠账去向是登记修订。
- `files_1000_units_100` 的 P1 读数补不全：固定点到 64 轮仍不定，错误交回之后那次发布没发成，P1 在这个点的 Q5、Q6、Q7、Q9（只有 1 次发布）都只有这一次。
- 不判结论能不能推翻或确立决策，也不选 P1、P2、P3。
- 重挂载的 B8 α 没单独算（上一节）。

```

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:465-473`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 量的那一路 | 拷来的实现上插计数，跑历史用例，打逐轮读数；取样点是登记第五节取样点表的枚举点（第一段 16 个点各 p1、p2a，对照 3 个点），理由是三根轴上跨三个数量级加形状与几何敏感性，不是扫遍定义域 | `scripts/experiment-apparatus/e206_fixed_point_cost_histories.rs:640 run_point_under_arm` | 被测代码本身；每叶 812 槽、扇出 169、上界 64、两块盘都取实现的值（产物头 `name=constants` 16 行相同：`leaf_slots=812 internal_fanout=169 settling_rounds_at_most=64 disks=2`） |
| 2 | 推的那一路 | 本地式子吃量的那一路的读数，算 P3 与 B8 列；抽象固定点在登记第七节那几张图上另算一遍 | `scripts/experiment-apparatus/e206_fixed_point_cost.rs:1084 p3_reading` | 每叶 812 槽、扇出 169、上界 64、两块盘写成本地常量，与产物头逐个比；逐轮读数；E203（准入之前扫类的步代价） 登记的 B8 式子 |

两条路径的比对接在 `model_mismatches`（`e206_fixed_point_cost.rs:1327`：轮功等于逐轮角色数之和、产物头常量等于本地常量）；合并报告里没有 `name=stop_condition_2` 行。登记第七节 ② 第 1、2 条的已有读数在这次产物上复现（上面三行 `name=anchor_measured … matches=true`）：`files_10000__p1` 同一次挂载 W1 第 1 次 22 轮、轮功 1201、逐轮角色数 11 到 84；第 2 次 2 轮；`base__p1` 2 轮、轮功 21、角色 7、14。E203（准入之前扫类的步代价） 的 B8 四个数（2102、2098、152344、4226）在本次报告里没有对应行，由模型的 `e203-anchors` 子命令核，**这一次没跑**。

```

**出处 `.claude/kb/experiments/206-发布固定点轮数的代价.md:474-480`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 备料 | 2026-11-09 不受影响：这一页只记读数，定案与射程没改；射程里「准入之前的步代价不随盘上容器总数、记录总数涨」要看固定点：P1 在 `files_1000_units_100` 撞上界（F1，照报），`files` 轴 22、38 轮与 B8 在 P1 臂仍随（上面 B8 一节）出自作废的 P1 逐轮读数，仅作线索，P2a 臂下 `files` 轴介于；这条分项的依据没引 E206（发布固定点轮数的代价），该不该升成支撑或推翻交主 agent |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-11-09 不受影响：条款没改；该分项接线写的固定点走在两道准入之间，P1 在 `files_1000_units_100` 走满 64 轮交回 `AllocationRecordTreeRewriteSetDidNotSettle`，是发生在该分项接线两道准入之间的一次拒绝；这条分项的依据没引 E206（发布固定点轮数的代价），该不该升成支撑或推翻交主 agent |

```

**出处 `.claude/kb/decisions/04-校验和位置.md:1-6`（整段抄，未转述）**

```markdown
## D4 校验和位置 —— 已定

**中文** · [English](04-checksum-location.en.md) · [日本語](04-チェックサム配置位置.ja.md)

D4（校验和位置） 管校验和住哪、罩多大：校验和内联进指向该块的父指针（父节点存子节点校验和），整棵树成 Merkle 树、自验证，能检测读到正确但过期的块，不另建 btrfs 那种独立的 csum tree。加密开启时这个 Merkle 字段存的是 MAC（D9（加密） 已定项 2），校验和粒度就是 MAC 粒度、加密不另开一层树；加密关闭时它取什么在已定项 8。指针里的完整性字段是两个，不是一个：Merkle 字段给带密钥的一侧，不要密钥的密文校验和（D9（加密） 已定项 1、D19（块指针的结构与宽度预算） 已定项 2、I-6.8（密文校验和匹配））给无密钥的一侧。不管的：指针各段的宽度与偏移归 D19（块指针的结构与宽度预算） 已定项 11 的偏移表（字节表 [layout/01-first-txn.md](../layout/01-first-txn.md) 第三节），指针总宽度不当约束；节点大小归 D8（核心索引结构） 已定项 2 / 已定项 4。过期块这一格没有检查，欠在 C7（过期块无检查）；节点大小结论按最终指针宽度复跑的账在 C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:20-35`（整段抄，未转述）**

```markdown
#### 已定项 1：校验和粒度与随机小读的张力

<!-- format-const: DATA_UNIT_BYTES = 32768 -->

**定案**：数据 extent 的校验和 / MAC 覆盖 32 KiB。内联让校验和粒度与 extent 粒度**耦合**，大 extent 的随机小读被迫读取并校验一整个单元——这条张力不消失，是被接受了。定案在数上等价于「接受在随机小读上付约一成」，不等价于「32 KiB 在这根轴上更快」，不许把它引用成后者。

**射程**：只管数据侧，粒度绑 extent，不绑节点。它与节点大小是两个数，不相等是允许的：D8（核心索引结构） 已定项 2 的 16 KiB 只管索引节点；节点自己就是一个单元，它的校验和粒度随节点走 ⇒ 元数据侧 16 KiB、数据侧 32 KiB。重开条件：随机小读的设备侧倍数 ≥ 1.136（已定项 8；按设备侧的倍数判，不按含 AES 的总倍数）；换介质、目标负载换成热点工作集、或 D25（目标负载优先级） 的负载权重改动时，适用前提变了，重跑 E58（校验和粒度的端到端代价） 再看。

**依据**：

- E58（校验和粒度的端到端代价）：32 KiB 相对 16 KiB 的代价落在随机小读上、大半在设备侧，顺序侧的收益只来自 AES 分块、设备侧为零——张力真实存在，付的是约一成。
- E6（加密算法选型）：热缓冲下大单元在 CPU 侧买不到吞吐；这句只对热缓冲成立，随机小读那条路缓冲恒冷、买得到的那点吞吐小于同一条路上的设备代价，方向不变。
- 用户定案，日期与经过在变更史。

**欠**：C238（位置条目从 11 变 14 之后「53」这个派生数没人回扫）：字节口径的临界随机小读占比按现行的指针宽（指向码 1 是 88 字节，D19（块指针的结构与宽度预算） 已定项 11）重算；C357（同一根轴上的两条重开判据没人合过）：重开判据与已定项 5 的交叉点落在同一根轴，没有合成规则。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:36-49`（整段抄，未转述）**

```markdown
#### 已定项 2：不足一个单元的 extent 怎么算

**定案**：单元恒为 32 KiB。短 extent（小文件、尾块、被截断的 extent）补齐到整单元，校验和 / MAC 覆盖补齐后的整单元 ⇒ 格式里没有「变长单元」这条分支。

**射程**：代价是空间放大，全部落在小文件上，经 D5（快照 / 空间记账机制） 的记账进入 ENOSPC 准入。补齐之后写路径上不把多个小文件打包进一个单元，这一句只在单次操作口径下成立（C168（E66 的 pack 臂只覆盖最差的一种实现））；写路径之外由后台整理把小对象搬进共享容器，归 D27（小数据打包容器），写入时照旧按已定项 2 补齐；内联阈值归 D14（双轨（大小文件 / 持久临时）） 已定项 3（定成 0）。

**依据**：

- E65（校验和粒度的写侧代价）：`pad` 臂的恒等式给出补齐的空间放大，只落在小文件上、随单元大小成正比。
- E66（补齐之后小文件怎么落）：补齐之后写路径上的打包在 I/O 上没有一格赢、只买空间，内联点读赢而写贵一个数量级——补齐没有推翻独立 extent 这条落法。
- 用户定案，日期与经过在变更史。

**欠**：C168（E66 的 pack 臂只覆盖最差的一种实现）：给打包加攒批臂与后台整理臂，在同一台设备上重跑并报绝对值。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:50-68`（整段抄，未转述）**

```markdown
#### 已定项 3：凑不满一个单元的写怎么写

**定案**：先把旧单元整个读回来并验 MAC，改完重算整单元的 MAC，再整单元写到新位置（COW，不原地覆盖）。首次写不设例外：所有凑不满一个单元的写都走 RMW，不给「首次写」开第二条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」同向）；首次写不读回会快得多，这笔代价是知情接受的，买到的是写路径只有一条。

**射程**：

- 新分配的单元没有旧单元可读：「走 RMW」在这一格读这次发布分到的新槽的残留一次，字节丢弃、旧内容按零，残留那一读出错记数、不拒写（用户 2026-10-26 定）；残留字节一律不进新单元（残留常是同一文件的旧单元）。一次部分写的读回次数 = 触及的单元里净荷没被整单元盖满的个数，有旧单元的读旧单元（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节「首次写」那一格）。C385（首次写要不要读回，两条已定项说反话） 由用户 2026-10-26 定「首次写也读回」，D14（双轨（大小文件 / 持久临时）） 已定项 3 的收益栏跟着改。
- **实现边界**：`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite_of_an_inode` 与 `publish_sequential_write_of_an_inode` 接收整份内容、从头替换；`build_data_unit`（`crates/singlefs-core/src/unit.rs`）在新缓冲上构造单元。没有任意偏移的部分写入口；截断尾单元的专用读改写不能代替通用部分写。未加密路径执行 CRC，不执行 MAC。
- 旧单元读不回来或 MAC 验不过时，连「只想改其中 4 KiB」的写也做不成（重算整单元的 MAC 需要其余原文）；该怎么办归已定项 4。

**依据**：

- E65（校验和粒度的写侧代价）：量出凑不满一个单元的写走 RMW 的字节与时间代价，字节放大远大于时间代价——已定项 3 知情接受的那一笔。
- E66（补齐之后小文件怎么落）：首次写不读回比读回整个单元再写快一个数量级，差的是那一次设备往返、不是字节——「首次写不设例外」付的代价。
- 判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节「首次写」那一格：新分配单元读残留一次、字节丢弃、旧内容按零的逐格读回规则（正推腿提，被攻过零轮）；用户定案 2026-10-26：C385（首次写要不要读回，两条已定项说反话）「首次写也读回」、残留读出错「记数、不拒写」（原话在变更史）。
- 用户定案，日期与经过在变更史。

**欠**：C57（零填充顶掉原文）：失败耦合那一格要故障注入才验得了；C385（首次写要不要读回，两条已定项说反话）：读法已定（射程第一条），读回次数与读序列钉成用例待实现。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:69-78`（整段抄，未转述）**

```markdown
#### 已定项 4：读—改—写读不回旧单元时怎么办

**定案**：旧单元读不回来、或读回来 MAC 验不过时，那个部分写失败并返错。「读不回来」指这个单元的两份都读不回：指针里的两条位置条目与中央映射里的两条落点都读不出，或整单元校验和对不上。一份读错、另一份能读时，返回能读的那份，那个部分写照常做。两份以最新那一版里指着这个单元的指针为准：整单元校验和与那一份自己的位置条目相符、单元头里的出生 txg 与写序与指针相符的，才是要读的那一份；不按两份单元头里谁的代号大来挑——被后来的实例盖过的那一份代号更大，却不是这个指针要的单元。读到只坏一份时计数；可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离，走同一个事务层，不开第二条写路径；挂载这一串里读到的，在挂载之后第一次用户改动发成之后搬，挂载之后一直没有用户改动就不搬；冷恢复读与只读挂载只计数、不搬。不许拿零填充凑出一个新单元——那会把「一段数据丢了」变成「一段合法的零」，而后者在盘上与真实数据逐字节不可区分。补齐字节恒为 0，且校验和覆盖整个单元含补齐。

**射程**：补齐字节为零那一半单镜像可判，是不变量 I-2.3（补齐字节为零且参与校验和）（[invariants/04-校验和.md](../../../crates/singlefs-checker/invariants/04-校验和.md)），池级 checker 在判（`crates/singlefs-checker/src/walk.rs`）；「读不回来时必须失败」那一半单镜像判不了——一个已经零填充重建出来的单元，校验和是自洽的——要故障注入才验得了。加密开启时 RMW 要先解密旧单元才改得动 ⇒ 无密钥侧做不了部分写；D9（加密） 已定项 5 定为「只做 scrub」，无密钥侧不搬。节点那一侧的可恢复性归 D8（核心索引结构） 已定项 4，不在这一格。最新已提交那一版的单元，两份是同一次发布用同一块缓冲、在屏障与根槽之前逐盘写出的；写一半出错就冻结、原样重发（`crates/singlefs-core/src/transaction.rs` 的 `persist_publish_writes` 与 `FrozenPublish`）。所以只坏一份只来自设备故障，或者读的是旧版本、它的槽释放之后又让一次没发布成的写盖掉了一块盘上那一份；只坏一份时读另一份是故障处置，不是常态。各读路径读到只坏一份时的处置（用户 2026-11-04、2026-11-05 定，原话在变更史）：可写挂载里的读都计数并搬，包括部分写与截断的读回、可写挂载时建上一版那一读、可写挂载期间别的读；冷恢复读与只读挂载只计数、不搬，只读挂载读到只坏一份不修。挂载这一串里读到的（可写挂载时建上一版那一读），挂载交回之前记下，挂载之后第一次用户改动发成之后，并进那次改动之后另发的那一批搬迁（快照下读回的旧单元本来就走这一批），被空间拒时同样推抬 F 再试一遍；那一批搬没搬成都只试这一次，结局随那次改动报出，之后的用户改动不再接着搬；挂载之后一直没有用户改动就不搬。豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，不进计划：它们在挂载这一串的发布里本来就换落点，旧落点走释放之前读盘核那一路（推的，多片实例表链里不被改写的那几片没核）。有活快照时读回的旧单元进 deadlist、坏副本留到快照销毁，这一形经已有的搬迁路径另发一批搬迁修好，另发那批被空间拒时推抬 F 再试一遍。

**依据**：无实验：「读不回就失败」是正确性规则，没有可量的量，要验只能靠故障注入的检查，不是计数实验。补齐字节约束成零而不是「随便什么都行」，是判别力要求：不约束的话 checker 无法把「合法的补齐」与「被零填充顶掉的原文」分开，I-2.3（补齐字节为零且参与校验和） 会对两种情形一起判绿。判决 `research/prompts/m4-close-code-r1-main-verification.md` 第四节 T3.1 那一行：已定项 4 的字面不分一份与两份，交用户。判决 `research/prompts/m4-close-code-r3-main-verification.md` 第二节判条款与代码落差的那一格与第四节：已定项 4 的字面不分读路径，交用户。用户定案（2026-08-31 立这一格；2026-10-31 定一份与两份的读法与计数加搬迁修复；2026-11-04 定快照下与只读挂载的读法；2026-11-05 定冷恢复读与可写挂载时的读法，并定可写挂载里读到的什么时候搬），原话在变更史。

**欠**：C57（零填充顶掉原文）：把某个单元的读弄坏，随后对该 extent 发一个小于单元的写，那个写必须失败；C946（只坏一份的读没有计数也没有搬迁修复）：读到只坏一份时计数、经搬迁把这个单元整个搬到新槽，这两样未实现。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:79-92`（整段抄，未转述）**

```markdown
#### 已定项 5：32 KiB 指的是含头还是净荷

**定案**：32 KiB 含头。单元恒 32768 字节，自描述头、声明长度、净荷、补齐全在这 32768 之内。

**射程**：已知代价是净荷不是 2 的幂：用户数据粒度 = `32768 − 头 − 扩展点声明值 N`，文件偏移 → 单元要做除法而不是移位，且 N 随每条线声明而变；读路径上文件的 4 KiB 页有一成多跨在两个单元上，净荷从偏移 h 起、页缓存页与 O_DIRECT 缓冲拿不到零拷贝。这两层与随机页读占比有交叉点，D25（目标负载优先级） 没有数值权重，不据此重开已定项 5。

**依据**：

- I-2.3（补齐字节为零且参与校验和） 逐字「校验和 / MAC 覆盖**整个单元含补齐**」，已定项 1 逐字「覆盖 32 KiB」⇒ 被覆盖的那 32768 就是单元的全部；另一种取法（32768 指净荷、盘上单元 = 32768 + 头 + N 再对齐）会让「覆盖 32 KiB」指向一个比单元小的区间，与 I-2.3（补齐字节为零且参与校验和） 直接冲突。单元大小是一个常量、不随头长度或 N 变 ⇒ 分配器、根槽对齐、`physical_block_size` 整除关系都只对一个数负责。
- E140（单元含头逼出的跨单元页与读侧拷贝）：含头在读路径上的两层代价（跨单元页、读侧拷贝）与补齐形态在随机与顺序两格反向，只报得出交叉点，不构成重开的理由。
- 用户定案，日期与经过在变更史。

**欠**：C357（同一根轴上的两条重开判据没人合过）：交叉点与已定项 1 的重开判据落在同一根轴。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:93-102`（整段抄，未转述）**

```markdown
#### 已定项 6：extent 声明长度住哪、多宽

**定案**：extent 声明长度住单元自描述头，宽 2 字节：值域 0..32768，16 位够。不取 4 字节——4 字节是按 128 KiB 单元档估的，而已定项 1 已把单元定成 32768。

**射程**：单元大小若将来改到 > 64 KiB，这个字段要跟着重开。加密开启时它是密文（D18（块里携带什么信息） 已定项 13，臂庚）：位置与宽度不动，只是从三类共用的明文前缀白名单里去掉；已定项 6 要求的是它住单元自己身上，不要求明文。码 1 的声明长度从含预留位的头末端 134 起算，上界 32768 − 134（D18（块里携带什么信息） 已定项 16）；实现写在共同前缀偏移 8 的 2 字节（`crates/singlefs-core/src/unit.rs` 的 `write_common_prefix` 与 `declared_length`）。

**依据**：无实验：住哪由 I-2.3（补齐字节为零且参与校验和） 推出，宽度是值域算术，没有可量的代价。I-2.3（补齐字节为零且参与校验和） 要在没有索引的前提下判声明长度之后到单元末尾恒为 0——那时读不到索引 key 的区间 ⇒ 声明长度只能住单元自己身上；不放它，那条判定在扫描重建路径上无从判起，而 C57（零填充顶掉原文） 正压在这条判定上。用户定案，日期与经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:103-126`（整段抄，未转述）**

```markdown
#### 已定项 7：逻辑 extent 与单元的关系

**定案**：一个逻辑 extent 的占盘长度恒 32768 字节，一个 extent 一个单元，不跨多单元；声明长度可变、≤ 32768。两个长度必须分开写，I-2.3（补齐字节为零且参与校验和） 的补齐起点用的就是声明长度：

| 长度 | 值域 | 谁在用它 |
|---|---|---|
| **声明长度** | ≤ 32768，用户数据 | I-2.3（补齐字节为零且参与校验和） 的补齐起点；已定项 6 定了它住哪 |
| **占盘长度** | **恒 32768** | 校验和 / MAC（已定项 1）；D19（块指针的结构与宽度预算）「每个逻辑 extent 一份」按它计 |

切分是预留特性，第一版关闭，`extent_off` 恒为 0。位留着不删，依据是 D19（块指针的结构与宽度预算） 已定项 3 定宽留位这条政策，不是因为第一版用得上；将来唯一可能让切分复活的是压缩——等到真要把多个压缩 extent 打进同一个单元时，`extent_off` 才会有非零值。

**射程**：可变 vs 不可变这个取舍不重开：声明长度可变一个字节都不省，占盘还是 32768，要省只能动占盘长度，而已定项 2 逐字「格式里没有『变长单元』这条分支」；重开不许拿倍数说事（`.claude/rules/fs-design.md`「比谁省不构成任何判据」），要拿一个小文件负载量出 ENOSPC 准入什么时候先撞墙。切分这条定案不动新池新建文件的字节（那 2 字节用不用都占着），它改的是「一份 MAC 会不会被复制成 k 份」，第一版恒为一份。压缩那一侧第一版留位、恒 0：指针头部的压缩算法码与压后长度（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11），`crates/singlefs-core/src/pointer.rs` 的 `PointerHead::write_to` 写 0。

**依据**：

- 四处已定条款，不是「两端相夹」那个推理（那个推理被三方论证的本地腿打穿，经过在变更史）：D19（块指针的结构与宽度预算） 开头「逻辑属性（MAC、nonce、算法类型、extent 距起点偏移）在头部，每个逻辑 extent 一份」——单数；D19（块指针的结构与宽度预算） 已定项 3 的 MAC 与 nonce 是恒定占位的定宽槽位，装不下随长度而变的 N 份；D9（加密） 已定项 4「指针头部为每个逻辑 extent **存一份** 96 位 nonce」；已定项 1「校验和粒度与 extent 粒度**耦合**」——耦合即相等，跨多单元是脱钩。
- 旁证：D19（块指针的结构与宽度预算） 已定项 11 给 extent 距起点偏移 2 字节（偏移 32），装不下 128 KiB extent 要的 17 位；D18（块里携带什么信息） 已定项 6 逐字「单元（extent）级」，括号里直接拿 extent 注解单元。
- E58（校验和粒度的端到端代价）：源码按父节点侧指针载荷摊，已定项 1 引的摊薄比正是按一个 extent 一个单元算的——旁证。
- E66（补齐之后小文件怎么落）：写路径上按子单元粒度打包出局，切分在写路径上没有来源。
- 切分：已定项 3 规定不满一个单元的写要整单元读回来、改完再整单元写到新位置，加上一个 extent 就占一个单元，任何操作都造不出「一个物理单元被好几个指针从不同偏移引用」这种局面；C72（extent 和单元什么关系没人答） 那一轮三方论证（`research/prompts/c72-extent-split-*.md`）把 reflink、去重、RAID 重建、journal 重放、快照销毁、重条带、墓碑打包挨个查过，一个反例都没找到。
- 用户定案，日期与经过在变更史。

**欠**：C72（extent 和单元什么关系没人答）：门禁比对「每 X 一份」这类字段频率，X 必须指到一条已定条款给出的大小（`extent_off` 第一版恒 0 的检查已有：池级 checker 的 `judge_pointer_extent_offset_is_zero`（`crates/singlefs-checker/src/image.rs`）在每条数据指针上判，core 读盘上的指针头读者都拒非 0）。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:127-139`（整段抄，未转述）**

```markdown
#### 已定项 8：加密关闭时 Merkle 字段取什么；已定项 1 的 5% 重开判据基线

**定案**：加密关闭时父指针里的 Merkle 校验和就是位置条目里那 4 字节 CRC32C，覆盖被指单元的全部字节（码 1 / 码 3 是 32768，码 2 是 16384）、每物理位置一份，不另开字段；第一版整棵树的自验证强度是 32 位，知情接受。已定项 1 的重开判据是「随机小读设备侧倍数 ≥ 1.136（E58（校验和粒度的端到端代价） 实测 1.086 再高 5 个百分点）即重开」，按设备侧的倍数判。

**射程**：加密开启时 Merkle 字段存 MAC（D9（加密） 已定项 2），不在这一格。实现：写侧 `location_entries`（`crates/singlefs-core/src/make_filesystem.rs`）对整单元算 CRC-32C 写进两份位置条目，读侧 `crates/singlefs-core/src/recovery.rs` 的 `read_unit_via_locations` 拿整单元的 CRC-32C 比位置条目里的 `unit_checksum`。

**依据**：

- E58（校验和粒度的端到端代价）：摘掉校验后的设备侧倍数就是重开基线。
- 写代码前的总审核（`records/2026-09-13-总审核.md` 的 D4（校验和位置） 两节）按「指针头部里没有第三个校验和槽位」推荐，用户定案，日期与经过在变更史。

**欠**：C357（同一根轴上的两条重开判据没人合过）：这条重开判据与已定项 5 的交叉点没有合成规则。

```

**出处 `.claude/kb/decisions/04-校验和位置.md:8-19`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **校验和粒度与随机小读的张力** | 数据 extent 的校验和 / MAC 覆盖 32 KiB；随机小读因此多付约一成，知情接受 **状态：已定。** **进度：已实现。** |
| 2 | **不足一个单元的 extent 怎么算** | 补齐到 32 KiB：单元恒为一个大小，校验和 / MAC 覆盖补齐后的整单元 **状态：已定。** **进度：已实现。** |
| 3 | **凑不满一个单元的写怎么写** | 读—改—写：读回旧单元验 MAC，改完重算整单元 MAC，整单元写到新位置；首次写不设例外 **状态：已定。** **进度：未实现。** 已有整份内容写与 COW，任意偏移的部分写 RMW 尚无入口。 |
| 4 | **读—改—写读不回旧单元时怎么办** | 两份都读不回时那个写失败并返错，不许零填充重建；一份读错就用能读的那份，计数并经搬迁修掉；补齐字节恒为 0 且参与校验和 **状态：已定。** **进度：未实现。** 只读挂载与冷恢复读只计数；可写挂载时的搬迁修复未实现。 |
| 5 | **32 KiB 指的是含头还是净荷** | 含头：单元恒 32768 字节，自描述头、声明长度、净荷、补齐全在这 32768 之内 **状态：已定。** **进度：已实现。** |
| 6 | **extent 声明长度住哪、多宽** | 住单元自描述头，2 字节 **状态：已定。** **进度：已实现。** |
| 7 | **逻辑 extent 与单元的关系** | 一个逻辑 extent 占盘恒为一个单元（32768 字节）、不跨多单元；声明长度可变、≤ 32768；切分预留、第一版关闭 **状态：已定。** **进度：已实现。** |
| 8 | **加密关闭时 Merkle 字段取什么；已定项 1 的 5% 重开判据基线** | 位置条目里那 4 字节 CRC32C 就是 Merkle 校验和、覆盖被指单元全部字节；随机小读设备侧倍数 ≥ 1.136 即重开已定项 1 **状态：已定。** **进度：已实现。** |

```
