# e203-b8-criterion-r1：B8 的判据改成按这一次碰到的单元算（2026-11-09）

<!-- doc-lint:not-numbers B1 B4 B8 F2 Q6 Q8 P2a -->

## 一、题面

用户 2026-11-09 弹窗定：B8 的判据改成按这次改动，也就是步数除以这一次碰到的单元数，看随总量涨不涨。是新一轮，原判据下的输先记下（`q8_peak axis=data_units alpha=0.978`，F2 字面打中）。改判据不等于选 B8，选臂另问。这一轮问：新判据写成什么才不是放宽。

原判据与读数：`research/prompts/e203-preregistration.md` 第十二节修订 8；`research/prompts/e206-preregistration.md` Q8；`.claude/kb/experiments/206-发布固定点轮数的代价.md`「B8 那一格的读数」；来源调查 `~/.local/share/singlefs/work/e206-b8-data-units-peak/result.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

`crates/` 今天没有 B8 的实现，步数是模型 `scripts/experiment-apparatus/e206_fixed_point_cost.rs` 按 E203 登记第五节式子算的。主工作区固定点已是 P2a。方案按 `crates/` 今天的实现来谈。

## 三、要回答的问题

1. 分母写成哪个量：这次释放的单元数、重写的角色数，还是两者之和；有没有办法用一个大改动把分母撑大、遮掉随总量涨的项。
2. 按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在跑前写死」三步：记一次输、只许收严、写明收严在哪。这个新判据比原判据多要求了什么，说不出就是放宽。
3. 用现有 40 份产物，新判据对 B1、B4、B8 各判什么；`files_then_big_*`（总量在、改动不在）还要不要量。
4. `.claude/rules/fs-design.md`「记账是事务的副产品」运行时决策路径「代价不许随盘容量增长」这句，新判据还罩不罩得住。

## 四、压着的条款

D8（核心索引结构） 已定项 16；D28（挂载期承诺量） 已定项 1。

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

### 小节清单：`.claude/kb/experiments/203-准入之前扫类的步代价.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E203 准入之前扫类的步代价 —— 部分已跑（够判）：B1、B4 够判；B8 等固定点那一笔；Y1 留给不指定范围的实验复跑 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E203 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 现状 | 抄 | 默认全抄（脚本生成） |
| ### 三段结果 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 判据与失败条款 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-11-08 | 不抄 | 历史节的子节 |
| ### 2026-11-05 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 核心索引结构 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：记账取幂等完整值，整批换代 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：节点大小 16 KiB，钉成常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：key 布局 —— 改名与索引分开处理 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：树表条目的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：树的种类码的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：第一版记账直落叶，`seq` 的取值规则 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：码 2 的载荷布局与 key 比较序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：write buffer 条目的三条硬要求 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/28-挂载期承诺量.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 挂载期承诺量 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：准入不等式的权威形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：准入的在飞合成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：实例切换的预留 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：checkpoint 保留池进准入 | 抄 | 默认全抄（脚本生成） |
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

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:1-8`（整段抄，未转述）**

```markdown
## E203 准入之前扫类的步代价 —— 部分已跑（够判）：B1、B4 够判；B8 等固定点那一笔；Y1 留给不指定范围的实验复跑

**中文** · [English](203-step-cost-of-scans-before-admission.en.md) · [日本語](203-受理前スキャン系のステップコスト.ja.md)

跑前登记 `research/prompts/e203-preregistration.md`（第十二节修订 1–8）。岔路单 `research/prompts/m4-admission-delta-forks.md` 三行：B1 今天、B8 逐项按差量、B4 改射程。量每次发布在两道准入之前扫类的步数与挂钟，跨三个数量级的单元区与记录数，含每盘 16 TiB 单元区的 X2（2³⁰ − 1408 槽，u、D、F 各 10）。

**这一页记修订 7 重跑的 18 个点（计数、计时两个构建各一份，共 36 份产物）与它们的合并报告（任务 10180）。** 判决两轮：`research/prompts/e203-forks-r1-main-verification.md`、`research/prompts/e203-forks-r2-main-verification.md`，不一致的以第二轮为准。

```

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:9-15`（整段抄，未转述）**

```markdown
### 现状

- 三段都跑了：第一段 8 个点（U 轴与 u 轴）、第二段 9 个点（D 轴、F 轴与对角点 X1）、第三段 X2。第二段的 Y1（`many_unlinks_unit_area_2_pow_16`）没跑：修订 6 把它挪进不指定范围的实验复跑。
- 够判的两行：B1、B4（B4 只够覆盖写类）。B8 记「不稳定」（修订 8），等固定点那一笔：岔路单 `research/prompts/e203-forks-r2-forks.md`（P1、P2、P3 三行），还没登记实验。
- 这一页不判结论能不能推翻或确立决策；两轮判决的「交用户表」还等用户定。
- 装置：量的那一路 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py`、`scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs`、入口 `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh`；推的那一路 `scripts/experiment-apparatus/e203_admission_scan_cost.rs`（bin 名 `e203-admission-scan-cost`），变异表 `scripts/mutation/tables/e203_admission_scan_cost.tsv`。

```

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:16-49`（整段抄，未转述）**

````markdown
### 三段结果

下面的行整行抄自 `research/results/e203-admission-scan-cost-report-2026-11-07.out`。这份产物是任务 10180 日志里以 `name=` 起头的 2643 行，每行加 `E7RESULT ` 前缀，末尾加一行 `name=done`（复跑表的完整性闸要这个形状）；抄的时候没有改别的字。

第一段（U 轴、u 轴）：

```text
E7RESULT name=q1 axis=U state=same_mount kind=overwrite row=whole_allocator_copy_before_the_judgement low_end=101959 high_end=103286290 alpha=0.998444 band=GrowsWithTheAxis
E7RESULT name=q2 axis=U state=same_mount kind=overwrite low_end=350496 high_end=309991237 alpha=0.978861 band=GrowsWithTheAxis
E7RESULT name=q2 axis=u state=same_mount kind=overwrite low_end=2503416 high_end=10171494 alpha=0.202951 band=DoesNotGrow
E7RESULT name=q3 axis=u state=same_mount kind=overwrite alphas=0.772315,0.619106,0.641893,0.670220,0.626462 band=GrowsWithTheAxis
E7RESULT name=q10 axis=u state=same_mount kind=overwrite segment=before_landing alphas=0.787311,0.757060,0.771941,0.790239,0.789375 band=GrowsWithTheAxis
```

第二段（D 轴、F 轴、X1）：

```text
E7RESULT name=q2 axis=D state=same_mount kind=overwrite low_end=2503416 high_end=8252955 alpha=0.172692 band=DoesNotGrow
E7RESULT name=q2 axis=F state=same_mount kind=overwrite low_end=2503416 high_end=163060965 alpha=0.604606 band=GrowsWithTheAxis
E7RESULT name=q6 axis=F state=same_mount kind=overwrite low_end=2102 high_end=152344 alpha=0.620064 band=GrowsWithTheAxis
E7RESULT name=q6 axis=F state=remounted kind=overwrite low_end=2270 high_end=4574 alpha=0.101423 band=DoesNotGrow
E7RESULT name=q5 axis=diagonal state=same_mount predicted_steps=34830876 measured_steps=62106461 nanos_ratios=1.178,0.933,1.000,1.347,0.709 passed=false
```

第三段（X2）：

```text
E7RESULT name=q6_top_rows point=user_scale_unit_area state=same_mount kind=overwrite rows=allocator_release_linear_find:1012,record_intersection_filter:354,mapped_units_whole_table:322
E7RESULT name=q9 point=user_scale_unit_area state=same_mount kind=overwrite steps_between=13220563557 nanos_of_each_valid_round=2959498030,2964803058,3551101363,3550223622,3542668057
E7RESULT name=q9 point=user_scale_unit_area state=remounted kind=overwrite steps_between=19830857527 nanos_of_each_valid_round=4475982759,2961183237,3020373367,3135067922,3604622355
```

X2 上调用方那一份整份拷（`whole_allocator_copy_before_the_judgement`）一行 1.390–1.719 秒（量的：三种发布 × 两种挂载状态 × 五轮共 30 个读数，取自计时产物的 `exclusive_nanos`）。

````

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:50-59`（整段抄，未转述）**

```markdown
### 岔路表

| 候选 | 状态 | 依据 | 还差 |
|---|---|---|---|
| B1 今天 | 够判：留不住 | U 轴随盘容量涨（Q2 六格 α 0.95–1.01，量的），对 `.claude/rules/fs-design.md` 运行时决策路径「代价不许随盘容量增长」本来就红；X2 整份拷一行 1.390–1.719 秒（量的）。F1 记 unknown：判决不许拿 X2 读数改判 F1，这个读数只作下界 | ③2 字面差 2 格读不到（见「判据与失败条款」）；u 轴到 13 个容器为止，暂定 |
| B8 逐项按差量 | 不稳定，不判翻面也不判站住（修订 8） | 只剩一格：Q6 F 轴同一次挂载覆盖写 α 0.620，只出在 files_10000 第 1 轮（固定点 22 轮），同一点第 2–5 轮与重挂载都不随。照 F2 字面仍是打中，原样记。修订 7 把 s36 改成按差量问，对原判据是放宽，新定义的 B8 被攻过零轮 | 固定点轮数那一笔的计数实验；量完照原判据再判 |
| B4 改射程 | 够判（覆盖写类）：只能当改条款交 | Q8 的 B4 臂六格×四轴都不随，F5 没打中；但覆盖写的式子判就是 D28（挂载期承诺量） 已定项 1 的 ENOSPC 准入，需求取自两道之间算出的结果，读法越位，两轮判决都这样判。结论只对清单内的步（F3 打中）。落点判之前那一段的残差在 u 轴覆盖写五轮都涨，正落在射程里，来源没量 | 删除类发布的读法被攻过零轮；Q9 用户规模的数不作数（修订 8） |

分族计数（一条命令从产物里数，命令与原样输出在执行员报告里）：Q2 的 U 轴 6 格都随，u、D 轴 6 格都不随，F 轴 1 格随、1 格介于、4 格不随；Q3 的 U、F 轴 6 格都随，u 轴 5 格随、1 格不稳定，D 轴 2 格随、4 格不稳定；Q6 的 U、D 轴 6 格都不随，u 轴 5 格不随、1 格介于，F 轴 5 格不随、1 格随；Q8 的 B4 臂四轴 6 格都不随，B1 臂 U 轴 6 格都随、另三轴 6 格都不随。

```

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:60-73`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 合并报告：36 份产物重算一遍，与留存产物逐字节比（exact）；不重跑历史
bash scripts/experiment-apparatus/replay.sh E203
# 18 个点里的一个：量的那一路，源树用修订 7 那一趟的冻结副本
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build counts --work <新目录> --source <冻结源树> --item base
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build timing --work <新目录> --source <冻结源树> --item base
# 合并报告的原始命令
cargo run --release --bin e203-admission-scan-cost -- report <18 份计数产物> --timing <18 份计时产物>
```

每条命令由主 agent 经 `task-server/task submit` 交。Y1 那一行登记在 `scripts/experiment-apparatus/replay.sh`，只在不指定范围的实验复跑里跑（修订 6），指定范围的 `replay.sh E203` 不点到它；跑出读数之后，读容器数那根轴的判定照原判据再判一次。

````

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:74-126`（整段抄，未转述）**

````markdown
### 口径

- 合并报告：`research/results/e203-admission-scan-cost-report-2026-11-07.out`。
- Y1 那一行的留存预期：`research/results/e203-admission-scan-cost-many-unlinks-completion.out`（手写的三行完成标记，不是量到的数；复跑表要一份对照物，两个构建都做完才打出同样的三行）。
- 还在工作区的 12 份原始输出：
  - `research/results/e203-admission-scan-cost-counts-diagonal-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-diagonal-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_1-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_100-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_1000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-data_units_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_1-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_100-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_1000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-files_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-unlinks_10000-2026-11-07.out`
  - `research/results/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07-r2.out`
- 已归档的 24 份，每份指到它的完成标记那一行：

```text
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-base-2026-11-07.out 第 6067 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_10000-2026-11-07.out 第 6164 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_1000-2026-11-07.out 第 6326 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_100-2026-11-07.out 第 5930 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-data_units_1-2026-11-07.out 第 5922 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_10000-2026-11-07.out 第 7550 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_1000-2026-11-07.out 第 6476 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_100-2026-11-07.out 第 6126 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-files_1-2026-11-07.out 第 6028 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_14-2026-11-07.out 第 5758 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_20-2026-11-07.out 第 6067 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unit_area_2_pow_24-2026-11-07.out 第 6236 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_10000-2026-11-07.out 第 7002 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_1000-2026-11-07.out 第 6204 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_100-2026-11-07.out 第 6006 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-unlinks_1-2026-11-07.out 第 6004 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07-r2.out 第 6236 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-base-2026-11-07.out 第 1659 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_14-2026-11-07.out 第 1661 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_20-2026-11-07.out 第 1659 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unit_area_2_pow_24-2026-11-07.out 第 1651 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_1000-2026-11-07.out 第 1651 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_100-2026-11-07.out 第 1665 行（name=item_done）
产物行见 git show 84b4ae23f:research/results/e203-admission-scan-cost-timing-unlinks_1-2026-11-07.out 第 1663 行（name=item_done）
```

- 「准入之前」切成三段各报各的：调用方段（进门到调 prepare 之前）、落点判之前、两道之间；准入之后只报数。扫类的步按最内层循环体每进一次记 1 步，整份拷按被拷值里每个集合的元素个数之和记。
- 计时用 release 构建、单线程、去掉 `lookup-work-counters` 特性；每个点在同一次挂载与重挂载之后各量五轮。
- s25 两处查找都记调用数（修订 7），补记的只是调用数，步数一格不变（修订 8；主 agent 比新旧 base 计数产物：共有行 1412，calls 不同 60 行，steps 不同 0 行）。
- s36 的查询次数取这一轮进出 `mapped_units` 的角色数（修订 7）；照修订 7 之前的式子，任务 9595 的 F2 打中、D、F 两轴 12 格记「随」，已记一次输。
- 计时报告的点轴值取自同名计数产物（修订 7）。
- 阳性对照：每条臂有一个把省掉的那一步关掉、退化成随单元区一次涨的开关；阴性对照：落点那一判拼单元的步数只随这一次的角色数。

````

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:127-154`（整段抄，未转述）**

````markdown
### 判据与失败条款

- 判据：增长阶 α 三档（α ≥ 0.5 随这根轴涨、α ≤ 0.25 不随、之间不判）；用户关心的规模上一次覆盖写准入之前的扫类挂钟五轮都 ≥ 2 毫秒记「≥ 1 毫秒」，都 ≤ 0.5 毫秒记「< 1 毫秒」，否则不稳定。
- 判决行（照抄合并报告）：

```text
E7RESULT name=verdict quantity=controls all_passed=true
E7RESULT name=verdict quantity=three_two cells_checked=14038 disagreements=0 cells_over_the_bound=0 cells_with_the_scale_unread=2 rows_reconciled=26 rows_equal_only=18 rows_with_a_bounded_part=8 rows_not_reconciled=0
E7RESULT name=verdict clause=stop_2 constants_not_matching_or_unread=0 three_two_disagreements=0 three_two_cells_over_the_bound=0
E7RESULT name=verdict quantity=Q6 result=GrowsOnSomeAxis
E7RESULT name=verdict quantity=Q8_b4_arm result=DoesNotGrowOnAnyAxis
E7RESULT name=verdict quantity=Q4 cells=AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond,AtLeastOneMillisecond reference_only_because_data_or_files_axes_are_missing=false not_judged_because_the_extrapolation_is_not_trusted=true
E7RESULT name=verdict quantity=Q5 cells_passed=2 cells_failed=8
E7RESULT name=verdict clause=F1 fired=unknown meaning=all_four_Q4_cells_at_least_one_millisecond_flips_B1
E7RESULT name=verdict clause=F2 fired=true meaning=Q6_B8_still_grows_on_some_axis
E7RESULT name=verdict clause=F3 fired=true meaning=residual_grows_on_some_axis_in_all_five_rounds
E7RESULT name=verdict clause=F4 fired=false unlinks_not_two_publishes=0
E7RESULT name=verdict clause=F5 fired=false meaning=B4_arm_before_the_landing_judgement_still_grows
E7RESULT name=verdict clause=F6 fired=true meaning=some_Q5_cell_failed
```

- F1：`fired=unknown`。F6 打中，Q4 记「外推不可信，不判」（`not_judged_because_the_extrapolation_is_not_trusted=true`）；`reference_only_because_data_or_files_axes_are_missing=false` 说的是 D、F 两轴都有读数，不是 Q4 判了。
- F2：`fired=true`，只有 B8 剩下的那一格，这一格记「不稳定」（修订 8，岔路表）。
- F3：`fired=true`。结论只对清单内的步。残差占段挂钟的比几乎不随轴变，登记给的出路「补清单」未必碰得到；残差来源没量。
- F4：`fired=false`，被量的 unlink 都判了 2 次。F5：`fired=false`，B4 臂在四根轴上都不随。
- F6：`fired=true`，Q5 十格 8 格不过；照登记进了第三段，Q4 仍只有 U 那一部分用量的。
- ③2：`cells_with_the_scale_unread=2` 不是 0，修订 4 写「读不到的格不为 0 不算③2 全过」，这两格是 data_units_10000 重挂载覆盖写第 4 轮的 `multi_level_tree_planning`、`mapped_units_whole_table`。这一条照字面没满足；判决把 B1 判够判，靠的是 U 轴与整份拷的读数，不靠这两格。

````

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:155-166`（整段抄，未转述）**

```markdown
### 它答不了的

- 只在 6 MiB 环、两块盘、覆盖写与 unlink 上；环长、盘数、截断、快照、整理搬迁没扫过（登记第八节第 4 条）。
- X2 只把单元区推到每盘 16 TiB，记录数没推；固定点在 X2 上只走 2–3 轮，别处量到 22 轮、源码上限 64。每盘 16 TiB、10⁶ 次 unlink 的形状是外推，Q5 核不过：Q4 与 Q9 用户规模的数都不作数。
- 容器数只量到 13 个墓碑容器（Y1 没跑），读容器数那根轴的判定都标暂定；13 个以上的增长这一段量不到。
- 22 轮是怎么来的、轮数上界代价多少，这里没量（固定点那一笔）。
- 整份拷在基线上占扫类合计 95.7%（攻方，推的），压平了别的行：计数 Q2 在 u、D 轴不随、计时 Q3 随，这两个读数不矛盾，只记录，不改判。F2 列的三行是模型按四轴之和挑的 X2 点，不是 F 轴高端。
- 照抄字节的代价归 E199（发布照抄上一版的代价），这一页只问扫类。
- 几处读数是装置自己定的操作化（登记没逐字写）：B8 的 `m_round` 取固定点那一轮释放的落点数加新分配的角色数；`dirty` 取一次调用前后内容不同的记录数；`dm` 取重写角色数的两倍（一出一进）；维护里 `m_rec` 取最后一轮同一个和、`m_seg` 取它作上界；树高从各树根节点头现读。
- s9、s10、s17 只记记录条数，形状拷贝在 Verus 块里不能插；s3 的计时罩住写者调用整条语句（拷贝在 Verus 块里），含写入本身的非拷贝部分；这四行是照抄类，不进扫类合计。
- s32 只记释放实例表链那一处（取样点上的覆盖写与 unlink 不写行，多半走不到，推的，没量过）。

```

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:167-175`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 量的那一路（B1 计数） | 在拷来的实现上只插计数，历史用例逐点逐状态打 `E203RESULT` 行；取样点是 18 个枚举点（Y1 没跑），不是扫遍定义域 | `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:653 edits`、`scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs:674 run_the_point` | 被测代码本身；实现自己的常量；`PoolUnderTest` 搭池 |
| 2 | 推的那一路（计数模型） | 纯算术：B8、B4 的步数式子、常驻内存、α 与判档、外推；锚点单测钉绝对值；读产物的报告接 Q1–Q13 与失败条款 | `scripts/experiment-apparatus/e203_admission_scan_cost.rs:187 by_delta_rows`、`scripts/experiment-apparatus/e203_admission_scan_cost.rs:471 narrowed_steps_by_segment`、`scripts/experiment-apparatus/e203_admission_scan_cost.rs:3403 full_report` | 本地常量（出处写在源文件头，产物头里实现报的值逐个比）；登记第五节的 B8 一列 |

两条路径的对账（登记第七节 ③2，修订 4）26 行都接了：18 行逐个相等，8 行有上界部分，`rows_not_reconciled=0`；不等 0 格，超上界 0 格，读不到 2 格（上一节）。

```

**出处 `.claude/kb/experiments/203-准入之前扫类的步代价.md:176-182`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 备料 | 2026-11-08 不受影响：这一页只记读数，定案与射程没改；「准入之前的步代价不随盘上容器总数、记录总数涨」在 U 轴与 F 轴的量上不成立（B1），改不改射程是 B4 那一行，等用户定；这条分项的依据没引 E203 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-08 不受影响：判决引它判式子判就是 ENOSPC 准入，条款不改 |

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:1-6`（整段抄，未转述）**

```markdown
## D8 核心索引结构 —— 已定

**中文** · [English](08-core-index-structure.en.md) · [日本語](08-コアインデックス構造.ja.md)

D8（核心索引结构） 管核心索引长什么样：一套 btree 实现配多个独立 keyspace 与两个前端、节点大小与它的上限判据、key 布局、whiteout 删除与无界操作的分批、inode 树的形态与 inode 记录的盘上字段表、write buffer 条目的 `seq`、树表条目的字段表与树的种类码登记表、码 2 索引节点的载荷布局与 key 比较序、前端与节点消息缓冲区的衔接、同一次发布里同一个 key 的状态折叠。**不预设 B-tree**，调查见 [prior-art.md](../prior-art.md) 第五节。不管的：记账统计量的载体、key 与合并规则在 D5（快照 / 空间记账机制）；多可写头下 key 要不要加一维快照、快照的实现模型在 D6（快照实现模型）；消息缓冲区的 ε 布局轴在 D11（索引节点要不要留消息缓冲区）；单元头、单元类型码与打包记录单元的装载纪律在 D18（块里携带什么信息）；指针的字段与宽度在 D19（块指针的结构与宽度预算）；发布语义与 checkpoint 边界在 D16（发布语义）；格式冻结的层次在 D15（格式冻结政策）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:30-42`（整段抄，未转述）**

```markdown
#### 已定项 1：记账取幂等完整值，整批换代

**定案**：**记账条目一律是「某个统计量在某一代的完整值」**，写入时整格覆盖；**不用增量 Δ**。合并规则按统计量分（D5（快照 / 空间记账机制） 已定项 3 第 3 行、登记表见 D5（快照 / 空间记账机制） 已定项 10）：非单调统计量「后者胜」（后者由 `seq` 定），单调统计量取最大值。取最大值同样幂等、可交换、可结合，本项的论证（幂等完整值胜过增量 Δ）不受影响。整批换代：每条根的记账树只装这条根那一代的行，不留多代（除 inode 号水位的台阶行随根保留外，D5（快照 / 空间记账机制） 已定项 2）；key 数量不随代数单调累积，两块盘的池上除台阶行外常态 15 行。

**射程**：它定的是记账条目 value 的语义；载体（记账住一棵独立 keyspace 的 btree、条目是「(统计量, 维度元组, 代) → 完整值」）在 D5（快照 / 空间记账机制）。⚠️ **key 数量不随代数单调累积**：每条根只留自己那一代的行（D5（快照 / 空间记账机制） 已定项 2，C548（记账树每次发布整批换代与每统计量每代一条不符）），两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂。⚠️ **兜底**：若某一类账确实做不到幂等（本项不预判有没有这一类），那一类必须另有一条与运行时**不共享判定代码**的独立重算路径。⚠️ 它同时改变 D11（索引节点要不要留消息缓冲区） 代价 1 的成立条件——该代价的判据是「缓冲里放的是幂等完整值还是增量 Δ」，本项把答案钉在幂等那一侧 ⇒ **代价 1 在本工程不成立**；D11（索引节点要不要留消息缓冲区） 的代价 2（可重建性退化）不受影响。

**依据**：

- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：同一棵树同一负载注入四类语义 bug，幂等完整值形态下 I-3.1（已分配统计对得上） 全抓、零盲区，增量 Δ 形态下一条都没抓到——Δ 形态下账本身就是权威内容，checker 只能拿同一个累加函数重放同一份 Δ 日志，它在和自己比。
- 用户定案 2026-08-29：取幂等完整值，接受 key 数量上升（原话在变更史）。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:43-58`（整段抄，未转述）**

```markdown
#### 已定项 2：节点大小 16 KiB，钉成常量

**定案**：**节点大小 16 KiB。mkfs 把 16 KiB 写进系统配置，实现按常量走，不按设备特性算**——格式里是一个常量，不是一条公式。
<!-- format-const: NODE_BYTES = 16384 -->

**射程**：⚠️ **登记 `format-const` 不等于冻结这个数**（2026-08-31 用户定案：「这个数需要可以调整」）。那条标记只保证 **kb 与实验源码写的是同一个数**，改它照常走 D8（核心索引结构） 的流程；它写进系统配置，仍然是 mkfs 参数。⚠️ **它现在只钉得住一部分源码**：门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）的形态是 `const 名 = 整数字面量`，`e56_epsilon.rs` 的 `16 * 1024`、`e43_extension_point_budget.rs` 与 `e52_head_mechanisms.rs` 的数组形式都不认。⚠️ **16 KiB 是 CPU 缓存轴上的选择，不是设备 I/O 轴上的**：E7（离线索引 harness） 量设备 I/O 时说小节点两端都赢、方向相反，两者量的是不同资源，不矛盾。上限那一维归已定项 4；E1（节点大小 vs 可恢复性） 的恢复时间那一半一旦给出比 16 KiB 更紧的上限，本项重开。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：唯一超出 L3 的工作集上 16 KiB 是最小点；同一实验里 40 字节的指针头部在 4 KiB 节点付出的相对代价远高于 16 KiB 节点，大节点吸收指针变宽；同一条曲线上 8 KiB 的反常六轮稳定且无人解释 ⇒ 在解释掉它之前不许把节点大小写成「按设备特性算」。
- E6（加密算法选型）：加密 / 校验单元的增益在 16 KiB 之后基本饱和。
- E73（节点 key 区间的扇出代价）：码 2 头带上 key 区间之后树高一格都没涨，16 KiB 不用重开。
- 用户定案 2026-08-30：16 KiB 钉成常量（原话在变更史）。

**欠**：C56（没登记的格式常量仍会漂）、C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:59-89`（整段抄，未转述）**

```markdown
#### 已定项 3：key 布局 —— 改名与索引分开处理

**定案**：**key = `(locality_id, inode, offset)`**，`locality_id` 在创建时从父目录继承，**改名时故意不更新**。offset 段是**文件字节偏移**（这个单元第一个字节在文件里的偏移，第 n 个单元从 0 起是 n × 净荷容量 32634）。

- 命名走 dirent 树（`parent, name` → inode），改名就是删一条加一条，**O(1)**
- 数据按 inode 编 key，与名字彻底解耦
- scan 局部性来自继承的 `locality_id`，改名后**局部性缓慢退化，正确性不受影响**
- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担

**批量改名**：`mv 大目录 别处` 在这个设计下**只改一条 dirent**。三处会把它重新变成无界操作，必须一开始就堵死：

1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。
2. **改名覆盖已存在的目标 = 一次删除**，必须走已定项 5（whiteout 删除）的 intent log。
3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。

**推论（可当 review 判据用）**：**任何让 rename 触及后代的设计改动，一律否决。**

**inode 记录必须存一份 `locality_id`**：它既是数据 key 的首段、又永不随改名更新，而命名树的 value 只有 inode、数据单元头的五元组里也没有它，所以盘上只有 key 自己带着它——读者打开一个改过名的文件时必须有地方拿到它，否则改名之后数据只能靠扫描才找得到。落点是已定项 6（inode 记录的盘上字段表）；这个值按本项从父目录继承，而第一版没有目录，所以「没有父目录时取什么」也要一起定——第一版取 0，登记在已定项 6 的记录字段表。扫描重建时重编出来的 key 首段按 inode 记录 > 幸存 extent key > 0 给这个对象的全部 key 编同一个首段（2026-09-16 用户定案，D18（块里携带什么信息） 已定项 3 那条加字段的提议不收）。

**射程**：它管数据 key 怎么编与改名怎么不触及后代，不管命名树自己的 key（dirent 树按 `(parent_inode, name)` 编，见已定项 5）。scan 局部性不一定要住在 key 序里：可以住 key 序（全路径索引，改名是 O(子树深度)）、住分配器（把子目录的数据放到父目录附近）、住第二索引（仍然贵，且要多维护一份一致性），本工程取「出生时继承、改名不更新的 locality 前缀」（较好，改名免费）。⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：作为地址首段它的取值精确且定义性。⚠️ **E9（key 编码对遍历局部性的影响） 的遍历局部性数只在一个口径下成立，引用必须连口径一起引**：**交错创建**（`interleave`，老化的代理）**加上 `cache_leaves = 8`**；`locality_id` 的收益是**新树的收益**，不是稳态收益，改名强度的现实性未校准——曲线可用，绝对刻度不可用。⚠️ 全路径索引的遍历优势也量出来了，但**不许拿它去替 D9（加密） 泄漏面与 D1（数据可移动性 / 反向索引） 稳定性做判断**——E9（key 编码对遍历局部性的影响） 对那两样一个字没说。⚠️ 「每次改名平均重写约 2700 个 key」这个数**缺口径**（查不到是哪一档改名、哪个种子），标为未核实；同表里 ext4 Orlov 分配器「遍历内核树快 30%」也是未核实——kb 内无出处无口径，要用先补。⚠️ 三段 key 在 D6（快照实现模型） 取多可写头之后曾经不完整（两个可写头各写一次同一个 `(locality_id, inode, offset)` 会落在同一个 key 槽、后写覆盖先写），2026-08-30 D6（快照实现模型） 已定项 1 取「每头一棵自己的树」之后消解——每头一棵树之下三段 key 在树内唯一，**两个头不再落进同一个 key 槽**。⚠️ **nonce 那条义务仍在**：D9（加密） 已定项 4 已定 nonce 每 extent 存，**生成时仍必须保证两个可写头之间不撞**（候选形态是按头划分 nonce 空间），它不再由身份唯一性免费得到，否则 I-6.1（nonce 不重用） 从「由身份唯一性保证」变成「由生成器保证」而没人保证它；该义务是推论，未走三方论证。⚠️ 后台重新聚簇是只搬物理位置还是重编 key、重编时怎么分批、进度从哪续做，全仓没有定义。

**依据**：

- E9（key 编码对遍历局部性的影响）：同一棵目录树同一个 LRU 上四条臂对比，`locality_id` 前缀比按 inode 编 key 少发设备读，值得做；depth-first 创建时三条真实臂逐位相同，缓存装得下整棵树时四条臂全部相等、效应消失；收益随改名次数衰减到几乎为零。
- 机制论证（2026-08-25 定案时写下，原文在变更史）：全路径索引在本工程还要多付三条 BetrFS 不必付的代价——分隔 key 就是路径会把 D9（加密） 已定项 5 认定的敏感信息从一棵树扩到所有按路径编 key 的树、引用方身份是路径会让 D1（数据可移动性 / 反向索引） 的反向索引条目随改名全部失效、lifting 对 key 编码的额外要求与 D9（加密） 的 nonce 载体和 D6（快照实现模型） 的快照 ID 位置抢同一段 key 空间。
- 用户定案 2026-09-16：扫描重建时给一个对象的全部 extent key 编同一个首段（原话在变更史）。
- E157（并行线一两条条款的计数模型） 量出：定位一页的节点读取次数、树高与叶片数、换容量后字节区间还原那一格，两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点那一格，文件字节偏移下 N 个 key 全算对、单元序号下只有 1 个算对。
- D9（加密） 已定项 6 逐字「锚点偏移 = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off`」，而同一条决策已定项 12 第 2 项写明 AAD 字段组成是 day-1 永久契约 ⇒ 条款本来就预设了这个单位。

**欠**：C346（后台重新聚簇没有定义）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:90-107`（整段抄，未转述）**

```markdown
#### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值

**定案**：大节点省 seek、树更浅，但**重扫代价随节点大小线性放大**，所以**不照抄 128K–256K**。**节点大小换的是「恢复时间」，不是「数据安全」，目标线该定在恢复时间上。取值靠 E1（节点大小 vs 可恢复性） 定：在「重扫代价 ≤ 目标线」的前提下取查找代价最低的那档。** 三条约束：

1. **它是 mkfs 时的格式参数，不是挂载参数。** 自动选取只发生在 mkfs 一次。改它 = **重建索引**，不是重建文件系统（D21（权威态与派生态的分界） 已定索引是派生态、冻结第 2 层降级为本地缓存格式）。
2. **默认策略必须是确定性的、可查的**，且 mkfs 要把「为什么选了这个值」打出来。
3. **测试钉死的是「模拟设备的参数」，不是节点大小本身**——固定一组设备档位让自动规则照常跑，**连选取规则本身也一起被测**。要求只有一条：同一档位必须得到同一取值。

**射程**：⚠️ **判据不是「全盘扫描重建成功率 ≥ 目标线」**——索引坏的永久损失恒 0 ⇒ 成功率这一维与节点大小**无关**，拿它当判据夹不出任何上限。⚠️ **目标线本身仍未定**，它是一个恢复时间预算（多久之内要把被破坏的那部分重扫回来），本项自己算不出来；E1（节点大小 vs 可恢复性） 剩下那一半仍欠 mkfs、checker 与一个能定点破坏镜像的工具。⚠️ 前置：索引可从单元自描述重建——E28（明文映射层能不能只靠单元自描述重建） 实测**元数据侧成立、数据块侧不成立**（缺口恰等于数据块总数，欠 D18（块里携带什么信息） 已定项 3）；⚠️ **「改它 = 重建索引」取消的是定性，不是代价**：整棵树按新尺寸重建是一次满量级的全盘 I/O，只是不丢数据；而且这个取消有未满足的前置——数据块自描述补上之前，「等于重建文件系统」在数据块那一侧仍然成立。桥接依据是索引树重建与明文映射层重建**读同一份输入**（每个单元自带「我是谁、我属于谁」），⚠️ 这一步是推理，E28（明文映射层能不能只靠单元自描述重建） 只测了明文映射层那一个实例；D11（索引节点要不要留消息缓冲区） 「不能拖」的全部理由就是这半句。⚠️ **第一期只有字节主导那一端在射程内**：D12（目标介质） 已定项 5 定第一条布局线是纯 SSD ⇒ 结论不变，**支撑从两端变一端**；寻道主导那一端要等 HDD 布局线开出来才重新在射程里。⚠️ **节点大小的自动选取判据里必须含指针宽度这一项**（E20（指针变宽的 CPU 缓存代价）：大节点吸收指针变宽，方向与重扫代价相反），E20（指针变宽的 CPU 缓存代价） 之前没有。⚠️ **「按设备特性取相反方向」这个理由不成立**：两端最优方向相同，都偏小；节点大小仍是 mkfs 参数，要保留按介质调档需另找依据。

**依据**：

- E29（坏一个节点的爆炸半径）：索引坏的永久损失恒为 0（D21（权威态与派生态的分界） 已定索引是派生态、丢了只是慢），数据单元坏的损失与节点大小无关，只有重扫代价随节点大小线性放大（第 1 层节点的重扫量恰等于扇出）——所以上限只能定在恢复时间上，「爆炸半径」这个说法被它纠正。
- E7（离线索引 harness）：**固定内存预算下大节点三个轴全都更差**（量的是 io/op、字节/次、点查读/次），机制是节点越大、同样的内存装得下的节点越少，命中率掉得比「每次读更多」赚回来的更快；寻道主导那一端方向相同，大节点仍差 ⇒ 「不照抄 128K–256K」在字节主导与寻道主导两端都得到支持。
- 用户定案 2026-09-06：口径改写成「上限由恢复时间定」（原话在变更史）。

**欠**：C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:108-167`（整段抄，未转述）**

```markdown
#### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log

**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表已定（D26（后台整理与放置回收） 已定项 2 的整理意图条目字段表），回退时取哪一版、态别留到第一个写者（整理）落地时定（C364（整理意图记录的字段没有逐字段对账））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。已定项 5 第 2 条与三态表、C1（意图续做做成常规路径） 的字面按承诺类写，整理落地时按类分开写（C621（整理落地时要动的删除条款没有清单外的检查））。

**硬要求**：无界规模的操作一律不许做成单个事务；意图必须**幂等**；whiteout 只在节点重写合并 bset 时清除，**不许有别的清除路径**。

意图本身就是一条普通 key，骑在同一个事务层上，断电后只有三种状态：

| 断电时机 | 恢复后看到 | 怎么收场 |
|---|---|---|
| 意图提交之前 | 没有意图 | 操作从未发生，用户那边也没返回成功 |
| 意图已提交、工作做了一半 | 意图还在 | 恢复时**续做**，已做完的部分被幂等地重跑 |
| 工作做完、意图未删 | 意图还在 | 续做一遍发现无事可做，然后删掉意图 |

不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。

1. **意图声明目标状态，绝不声明增量。**「把文件 X 截到 N 字节」可以重复执行，「再删 1000 个 extent」不能。
2. **续做是每次恢复都跑的常规动作，不是修复手段**（否则会出现「平时不跑、真出事时才第一次跑」的代码路径）。
3. **中间状态必须每一个都是合法的文件系统状态**——「删了一半 extent 的文件」本身就得合法。
4. **一批里每个动作施加时先核它要删的东西还在**（随 D5（快照 / 空间记账机制） 已定项 12）——幂等的字面：一批按批开始时的状态为几个意图各算一份计划、施加时不复核，两个区间重叠的级联会把同一块放两次，统计量减两次、I-3.1（已分配统计对得上） 红。

**范围重建 vs 逐项修改：两条都要有，且要有显式判据**（`.claude/rules/fs-design.md`「不追求万能方案」）：

| 走法 | 代价 | 适用 |
|---|---|---|
| 逐项修改 | K × 随机访问 | 受影响的 key 稀疏散布 |
| 范围重建 | N × 顺序访问 | 受影响的 key 在 key 序上连续 |

**真正的开关不是 K/N，是「受影响的 key 连不连续」**：连续 ⇒ 范围重建可整节点丢弃或重写；不连续 ⇒ 只能逐项。**由此得到一条布局规则：让一起死的东西在 key 序上挨在一起。**

- dirent 按 `(parent_inode, name)` 编 key → 删整个目录是范围操作
- extent 按 `(inode, offset)` 编 key → 截断是范围操作
- 同一快照的 key 带同一 snapshot 前缀 → 删快照有机会变成范围操作

**这顺带从结构上解掉了 whiteout 膨胀**：范围删除直接丢弃整个节点，不用插 N 个墓碑。

**射程**：朴素的交叉点是 `K/N > 顺序代价/随机代价`（机械盘约 1%、SSD 约 10%）。⚠️ 这两个数是**推理，不是实测**，说的是**耗时**，要靠 E4（范围重建 vs 逐项修改的交叉点） 量，那一半要 QEMU。⚠️ **第一期只取 SSD 那一档**（D12（目标介质） 已定项 5：第一条布局线是纯 SSD）。⚠️ 写放大那一半量到的交叉点，自变量是「布局」与「节点大小」而不是介质，不能拿它去证伪「机械盘约 1%、SSD 约 10%」——那说的是耗时。按 `.claude/rules/fs-design.md` 的五条要求，这个分支还欠：判据写成可计算的式子、留一个能强制走指定分支的测试开关、两条分支各自跑崩溃点重放、运行时能看出走了哪条。⚠️ **切换判据里不许出现「当前填充率」这类操作中途会变的量**——判据只能用操作期间稳定的输入。**与 D3（空间分配） 的交互（容易漏）**：意图未完成期间那些空间还没真的释放，空间统计必须把「待删除但未完成」计进去，否则会出现「我删了一大堆，`df` 却不动」。

**依据**：

- E30（交叉点的写放大那一半）：**写放大那一半已量**——在写放大口径下量出条目连续与条目散布两种布局在 4 KiB / 16 KiB 节点上的交叉点，两者相差两个数量级以上——开关是布局与节点大小，不是介质。
- 用户定案 2026-09-16：加第 4 条硬要求「一批施加时先核要删的东西还在」（原话在变更史）。
- 三方 `research/prompts/m3-intent-r1-main-verification.md`：意图分承诺类与提示类（两腿独立得出，一轮）；logged_ops 树零根注册是用户定案 K-I1（原话在变更史；被攻过零轮）。
- 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节「截断续做与计划」「拒绝边界」两格与「全池活截断条目至多一条」那一格：截断推广到任意活 inode，续做按条目点名的号认领，全池至多一条留着（被攻过零轮）；用户定案 2026-10-26：射程「任意活 inode 都能缩短与变长」（原话在变更史）。

**欠**：C2（无界操作做成单事务）。

**截断意图的字段表**（logged_ops 种类码的唯一登记在 D26（后台整理与放置回收） 已定项 2；小端，条目恒 44 字节）：

| 段 | 字段 | 宽 | 偏移 | 取值 |
|---|---|---|---|---|
| key | 种类 | 2 | 0 | 恒 2 = 截断 |
| key | 签发 txg | 8 | 2 | 意图发布的 txg |
| value | inode 号 | 8 | 10 | 目标活对象的 inode 号 |
| value | 对象出生代 | 8 | 18 | 与目标 inode 记录一致 |
| value | 目标 size | 8 | 26 | 目标字节数；意图先发布目标大小，随后回收范围外单元 |
| value | 补零 | 10 | 34 | 非零拒收 |

**截断入口边界**：`crates/singlefs-core/src/transaction.rs` 的 `publish_size_change` 对任意活 inode 允许真正变长或缩短（用户 2026-10-26 定的射程，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`）：截断的两种批带 inode 号，单元数、尾单元重写的对象与角色、旧净荷来源、映射 key、续做认领全按条目点名的 inode；下段计划的入参是这个文件自己的下段节点表；意图那次把罩着被重写单元的叶到根都重写；全池活截断条目至多一条照旧；删完第一个文件之后才建的 inode 改 size 照任意活 inode 办。现行代码照这一条做：单元数按 inode 取（`data_units_of_the_file_in_the_version`），别的文件改 size 走 `ChangeOfAFileOtherThanTheFirst::SizeChange`。意图编码与解析见 `crates/singlefs-core/src/logged_operations.rs`，设计来源为 `research/prompts/m4-trunc-r1-main-verification.md` 与 `research/prompts/m4-trunc-r2-main-verification.md`；inode 的 `T`、`N_w` 字段在已定项 6。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:168-252`（整段抄，未转述）**

```markdown
#### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**定案**：inode 树 = 码 2 内部节点（派生态，不留消息缓冲区，条目 120 字节、扇出 135）+ 码 3 打包记录单元做叶（打包记录类型 2，权威态单元，一叶 233 条 140 字节记录）；根恒为码 2；容器索引就是这棵树的内部节点；已发布的 (inode 号, 对象出生代) 不复用，下一个号是记账里一条取最大值合并的统计量。码 2 根装满时长一层——新根 1 个码 2、分出的右半 1 个码 2（右半没有容器身份），inode 树与墓碑树（D18（块里携带什么信息） 已定项 24）共用这一条；长高那次发布多 2 个 16 KiB 节点 × 副本数。根长高之前代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒（推的单元数，零轮）。

| 部件 | 是什么 | 态别 |
|---|---|---|
| 叶 | 一个 32 KiB 码 3 打包记录单元（D18（块里携带什么信息） 已定项 11），打包记录类型 2，装 233 条 140 字节记录；自带类身份段、记录数、记录宽、载荷校验和，扫描路径上的载荷完整性证据（C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 对码 2 的洞）对它不存在 | 权威态（单元） |
| 内部节点 | 码 2 索引节点。头带 key 区间（min / max 各 8 字节，D18（块里携带什么信息） 已定项 2 留给 inode 树的那一格由此定「带」：理由与记账树同一条——一个节点单独捡起来能自证覆盖段，key 是 inode 号不是文件名、敏感性论证够不到）。**头是块头，落 D15（格式冻结政策） 第 1 层（冻结），与记账树节点头同一格（C102（索引节点头没有预留区，加字段不是 compat））；载荷布局（条目 120、ε = 0）落第 4 层，不永久** | 派生态，丢了从叶重建 |
| 根 | 恒为码 2 索引节点，哪怕树只有一片叶，哪怕树是空的（零条目，key 区间写 [0, 0]，0 是无效 inode 号）。理由：D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，树表指针指向的根若类会随树长大而变，读者按 16 KiB 读 32 KiB 是静默误读 | 派生态 |
| 容器索引 | 就是这棵树的内部节点，不另立树：物理指针只住内部节点，dirent 树引用的是 inode 号。更新一条记录 = COW 叶 + 全部祖先（D23（journal 的角色与格式） 已定项 1 甲） | — |

**内部节点条目 = 分隔 key 8 + 身份引用 26 + 子指针 86 = 120 字节**（扇出按头 131 算 ⌊(16384 − 131) / 120⌋ = 135）。身份引用就是 D18（块里携带什么信息） 已定项 11 给「别的树引用容器」的那 26 字节：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。**类型段兼作「子节点是什么」**：0 ⇒ 子节点是码 2 索引节点（容器号、出生代两段恒 0；登记表里的 0 在容器头里仍是无效，两处含义靠所在结构区分），2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏。合法集合按树分：inode 树 {0, 2}（I-9.2（条目身份与子头相符））、墓碑树 {0, 1}（I-9.23（墓碑树条目身份与子头相符））、谱系树 {0, 5}（I-9.17（谱系树条目身份与子头相符））、退役树 {0, 3}（不变量随第 50 批与退役树的实现一起立，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条），四棵树的内部条目同一形态。读者由此知道读几个槽；码 3 那条边的 AAD 五段（标签由类型 ≠ 0 推出，其余四段照抄条目）全部来自查找路径，D9（加密） 的 A6（AAD 期望值不来自明文侧）在这条边上不需要树的种类；码 2 那条边（内部节点指内部节点）的 AAD 仍是 C107（索引节点的 AAD 身份无落点）。出生树逐条目携带，是因为克隆头（D6（快照实现模型） 已定项 1）共享 origin 的叶，同一个内部节点里会混着出生树不同的条目；D19（块指针的结构与宽度预算） 已定项 7 让子指针头部也带出生树之后，身份引用里的出生树**照旧保留**——两份并存，加密开不开都占位、不挪作他用；类型段 0 条目身份引用里的出生树恒 0，与容器号、出生代两段同形；码 3 那条边 AAD 的出生树取身份引用那份，不取子指针那份。**分隔 key** = 该孩子创建时的最小 key，左从右借的那次发布改成被借的右孩子这一版的最小 key；条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**内部节点不留消息缓冲区，inode 树的 ε = 0**——这是对 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴增补（布局轴「全部布局共用同一组 ε」不动）。分支变量是 D11（索引节点要不要留消息缓冲区） 已定项 4 那条谓词：**这棵树待下刷的消息能不能只从权威态（单元（带写序）+ 实例表 + 祖先表 + 墓碑单元）重算**。能 ⇒ 留缓冲（ε = 0.65）；不能 ⇒ 不留。inode 树不能：属性更新（mtime / nlink / mode 的新值）在到达叶之前没有权威态载体，缓冲进派生态节点就撞 D21（权威态与派生态的分界） 硬约束 6。extent / dirent / 反向索引能（E59（缓冲消息能不能只从单元重算） 的「单元 + 记账」一行三类全 0）；记账树按 D21（权威态与派生态的分界）「离线可由遍历重算」判能，但 E59（缓冲消息能不能只从单元重算） 那一列是神谕、未实测。它落 D15（格式冻结政策） 第 4 层（派生态），不是格式分支。**态别按重算性判，不按「树」这个词判**：一棵树的内部节点可从它的叶重算 ⇒ 派生态；叶若装着算不回来的值 ⇒ 权威态。记账是同一条判据的已定实例，权威记录点在 D21（权威态与派生态的分界）。

**装载纪律按记录类型分**：「一容器一个代际」与「快照发布前先关闭开放容器」只对按代际回收的打包记录类型 1（墓碑）成立；类型 2 的记录逐条死亡，容器由 B 树分裂 / 合并回收，一片叶活很多代、反复重写，快照只是共享它。**死亡分三层**：① 类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113（扫描重建时多版单元的现行版本判定无输入）；② 叶容器的身份消失（删空摘掉、合并时被吸收的右半、跨头首次 COW 重生时的旧身份）写退役记录：打包记录类型 3，带死亡 txg，住专用的退役树（D18（块里携带什么信息） 已定项 11）；退役记录的写者只追加：同一身份跨管理员回退可以有多条，不去重、不覆盖；死亡 txg = 身份消失那次发布的 checkpoint_txg；读者与 checker 只认死亡 txg 在这一版谱系上的退役记录；③ 对象整个删除由 unlink 意图那次写的整对象墓碑体现，从意图那次发布起生效（D18（块里携带什么信息） 已定项 24 第 2 条、已定项 16 第 1 条）。文件变短（覆盖写成更少的单元、截短）不写新记录，数据单元的死活在扫描重建时按重建出的 inode 记录判（「重建」一段）。

**容器身份不需要分配器**：inode 号在每一条时间线上单调（管理员回退的新根取 R_old 那一版的 inode 树、水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)；崩溃恢复回到所选根的树与它的水位；两种之后发的号都大于那棵树里每一个 key）⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**——左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）；第一个容器的号 = 第一条记录的 inode 号。内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份。合并：**左吸收右与左从右借两样都做**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份；被吸收的右半走 COW 释放路径，同一次发布给它的身份写一条退役记录；左从右借时借的条数取均分（借完两片条数相差不超过 1），右片保留身份、不写退役记录，它的分隔 key 改成它这一版的最小 key；合并时机是写路径策略（单测 / 模型对拍），不进格式。**记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目），删空摘掉的容器同一次发布给它的身份写一条退役记录。**跨头首次 COW 的重生**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P6）：新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录。重生只对类型 2：类型 1 容器归写者树独占、克隆不收进自己的容器索引（D18（块里携带什么信息） 已定项 11），从不跨头 COW。两条不同的断言并存：「容器被 COW 重写时身份不变」（内容同、位置变）与「分裂后左半身份不变」（内容少了、身份不变）。类型 1 的容器号仍按 C103（容器索引无落点） 那套单调分配，两条规则按记录类型分。

**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)，位宽 64，声明生命周期创建总数上界 2^63。**已发布的 (inode 号, 对象出生代) 不复用**（D6（快照实现模型） 判据 9 的戒律）：`deleted_inodes` 后台回收窗口里复用会让新旧对象的 extent key 三段逐字相同。⚠️ **「已发布的」这四个字不是为了对齐措辞，是让现行设计自洽**：水位住记账（D5（快照 / 空间记账机制） 已定项 4 第 12 项），被抛弃 checkpoint 的那笔账随它一起丢 ⇒ **每一次普通崩溃**都会让那一代发过的 inode 号被重发。管理员回退是挂着时的一次向前发布，那次发布的 inode 号水位取 max(现行那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)，已发过的号不重发（D23（journal 的角色与格式） 已定项 14）；崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里已发过的号会被重发，而恢复之后每一个新对象的出生代都大于环里读得出的每一条根与记录的 txg（D23（journal 的角色与格式） 已定项 14 注 3）——要几个故障才撞得上被抛弃时间线里已发布对象的出生代，崩溃恢复那一形没量过，见 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）；对外暴露的对象身份 = (inode 号, 对象出生代)，凡按 inode 号认对象的结构都要带上出生代或随根回退。`deleted_inodes` 回收窗口里 extent key 三段逐字相同那一句说的是同一条时间线里的复用（旧对象已发布、它的 extent 在树里等后台回收），照旧被禁；崩溃恢复之后重发同一个号，新对象的 extent 落在所选根那棵 extent 树里，旧对象的 extent 随被抛弃的根一起不在树里——**这靠 extent 树与 `deleted_inodes` 都随所选根**；管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树，`deleted_inodes` 也取 R_old 那一版的（否则 R_old 之后删掉、随 R_old 的 inode 树回来的对象会被后台回收再收一次；主 agent 2026-09-26 定，推的，被攻过零轮）；`deleted_inodes` 的形态（已定项 16）守这两条；盘上分开两者的是单元五元组里的对象出生代（D9（加密） 已定项 6）。**inode 记录在回收完成前留在 inode 树里（nlink = 0），`deleted_inodes` 只存号**。

**下一个 inode 号是记账里的一条统计量**（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：key = (统计量, 树 ID, 设备段, 代)，不带设备维，value = 下一个可用号，**合并规则取最大值**（幂等、交换、结合）；条目照样带 seq（已定项 15 第 3 条是格式要求，一棵 write buffer 树一套条目格式），只是**合并结果不依赖它**。选它而不是「后者胜」的理由是崩溃后 txg 重发那一格：同一个代段里先后出现大小两个值，取最大值给大的（浪费号、不复用号）；「未发布的等于没发生」只对常规挂载路径成立，扫描重建看的是盘。「标签 → 合并规则」是盘上语义，统计量登记表立成独立冻结组件（D15（格式冻结政策））。水位行只为可写头维护，照台阶行的规矩写：水位变了的那次发布留一行台阶行，随根 COW 一直留着（D5（快照 / 空间记账机制） 已定项 2 每条根的记账树只装这条根那一代的行、每次发布把这一代的行全写一遍，对它开了例外，见 D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」那一条），只读树没有这一行；读现行值 = 取该维度元组下代号最大的那一行，空 ⇒ 从未维护过、不是 0。克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。

**记录字段表（定长 140，小端；对齐口径是记录内偏移，容器内第 n 条起点 136 + 140n（码 3 头 107 + nonce / MAC / 算法类型预留位 29，D18（块里携带什么信息） 已定项 16）不按 8 对齐，按非对齐读）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| 偏移 | 字段 | 宽 |
|---|---|---|
| 0 | inode | 8 |
| 8 | 对象出生代（创建那次发布的 checkpoint_txg；inode 号不复用后它不再承担复用判别，按五元组 / AAD 的 day-1 契约保留，扫描时与该对象每个数据单元头逐字段对照） | 8 |
| 16 | locality_id（第一版无父目录取 0） | 8 |
| 24 | mode / uid / gid / nlink，各 4 | 16 |
| 40 | size / blocks / rdev，各 8 | 24 |
| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |
| 88 | 改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲；unlink 的意图那次与每一批也算改动，写本次发布的 checkpoint_txg，不照抄上一版；创建算一次改动，所以改动计数不低于记录偏移 8 的对象出生代，I-9.26（inode 改动计数落在出生代与容器诞生代号之间）） | 8 |
| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |
| 108 | 填充（恒 0） | 4 |
| 112 | flags（恒 0） | 8 |
| 120 | 截断字段 `T`（最近一次需要回收数据单元的截断意图发布 txg；无则 0） | 8 |
| 128 | 截断字段 `N_w`（该次截断的目标字节数；无则 0） | 8 |
| 136 | 预留（恒 0） | 4 |

**写入时间（偏移 64 起三个秒字段、偏移 96 起三个纳秒字段）取写这条记录的那一次发布的参数，不取系统时钟**：每次发布调用带一个写入时间，同一个参数两次发布逐字节相同（可复现）。三个秒字段都写参数的秒数；三个纳秒字段写参数的纳秒部分，参数只带秒数，所以写 0。**新建**（新池新建文件、新建 inode）写创建那次发布的参数；**改写内容**（覆盖写、顺序写）重写这条记录时写这一次发布的参数，不照抄上一版的值；**unlink 的意图那次**写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值；**不改这条记录内容的发布**——同一片叶里别的记录改动、这条被连带重写，回退——照抄这条记录上一版的值。

**`blocks` 是逻辑长度的 512 字节块数**：`blocks` = ⌈`size` ÷ 512⌉，按文件长度向上取整，**不表示分到的空间**，不按这个对象分到几个数据单元算。新池新建文件那条记录 3000 字节 ⇒ 6；新建、还没写过数据的 inode ⇒ 0。

不放 extent 指针（D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0），btime 不进记录。**flags / 填充 / 预留的零值政策**：偏移 108–119 与 136–139 恒 0；偏移 120–135 这 16 字节恒按截断字段 `T`、`N_w` 读取（截断字段无条件带上，池的 incompat 位 2 是布局身份的一部分，D15（格式冻结政策） 已定项 4）。违反对应零值要求 ⇒ 该记录 EIO（对象级，不拒整个容器）——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。将来给任一位 / 任一字节赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat；格式锁定之后退役的字节永久报废，锁定之前可回收、可重排。**记录内不存在 compat 扩展路径**：inode 属性都有语义依赖，D15（格式冻结政策） 判定表第一行够不着，且记录被反复重写、旧读者会把不认识的内容丢掉。原预留 20 字节的宽度不承重：记录内任何赋义都是 incompat，宽度只决定那次要不要连记录宽一起改。**溢出路线按属性逐个判档**：无语义依赖的属性走新增 keyspace（D15（格式冻结政策） 判定表给 compat），有依赖的走 flags 位 / 预留（incompat）；属性 keyspace 没有落点（xattr 被推迟）。**记录宽必须等于登记表给类型 2 的宽**，不等判损坏（I-1.7（打包容器合法与判定顺序） 里在「类身份段」之后、I-1.2（块头写序已发布） 之前插一步，只需头字段）；改宽是 incompat。记录格式、码 3 类身份段对类型 2 的用法、inode 树的 key 编码在格式锁定之后是永久字节：**立成一个独立的冻结组件**（C9（冻结后偷改格式） 按组件记），冻结时刻是格式锁定那一次提交（里程碑 17 收尾之前不存在格式锁定，格式锁定的定义在 D15（格式冻结政策） 的射程）；不动 D15（格式冻结政策） 的四层；类型 1 的记录格式（宽 40，D18（块里携带什么信息） 已定项 24）各自独立。

**截断字段的兼容性与写法**：偏移 120–135 的赋义由 D15（格式冻结政策） 已定项 4 登记的 incompat 位 2 标识；当前写者从 mkfs 起置位。需要回收数据单元的截断在意图发布写入 `(T, N_w) = (该次 txg, 目标 size)`，批次及最后一批保留；不释放单元的截断、变长、等长请求及 unlink 携带已有值；整份覆盖/顺序内容替换清零，新 inode 为零，回退恢复目标版本的值。读法 Z 用该字段排除已截掉后又变长时的旧单元；规则来源为 `research/prompts/m4-trunc-r2-main-verification.md`，读写实现为 `crates/singlefs-core/src/records.rs` 与 `crates/singlefs-core/src/transaction.rs`。

**不变量**：I-9（inode 树结构） 一类，十四条，见 [invariants/09-inode树结构.md](../../../crates/singlefs-checker/invariants/09-inode树结构.md)；状态列统一带前置「树的种类可识别（已定项 8）」，第一版镜像上可由类型 2 容器头里的出生树反推哪棵树是 inode 树。nlink == dirent 条数不在这里立——它的门（dirent 树存在）没有输入，随 dirent 树上线时一起立。第一版的唯一文件取 inode 1（0 无效），nlink = 1，不进孤儿回收。分裂 / 合并的身份传递与合并方向以外的写路径纪律是操作断言，走单测 / 模型对拍与崩溃点重放。

**重建**：内部节点丢了，收集打包记录类型 2 的容器，按 (出生树, 容器号, 容器出生代) 分组，同身份多版的择现行 = C113（扫描重建时多版单元的现行版本判定无输入）；退役树里有退役记录指认的身份不收，退役记录带死亡 txg、按谱系滤（与 I-9.18（删除队列里的号有谱系上的墓碑） 对墓碑同一读法），退役树、墓碑树、谱系树这几棵随现行的树只按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 择版本；每叶 key 范围由记录算，内部节点的 key 区间由叶算；产物落 D18（块里携带什么信息） 已定项 5 的级 1（只读挂载），级 2 的见证者不存在。**extent 树的重建**：对象按 (树 ID, inode) 认、只看本头的 extent 树；这个对象的全部 extent key（幸存的与找回的）编同一个首段，取值 inode 记录 > 幸存 extent key > 0，记录完好时重建视图里的值就是记录里的值；取错只丢局部性。**数据单元的死活**：按重建出的 inode 记录的对象出生代与 size 折成的单元数判，文件变短不写新记录（读法 Z）。I-9.9（locality 三值对照） 在重建之后按侧判：一侧是从另一侧编出来的就输出「不可判定」。

**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）；一条记录（inode 1、出生代 3、locality 0、nlink 1、改动计数 3、三段 0）；记账里一条「inode 号水位 = 2」。

**射程**：**同一条时间线上 inode 号不复用**——「时间线」指所选根的祖先链：含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发），不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）。**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在崩溃恢复落到较旧的根之后可以重发（管理员回退那次发布的水位取 max、不重发，D23（journal 的角色与格式） 已定项 14），所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而恢复之后新实例的每一个 txg 都大于环里读得出的每一条根与记录的 txg，D23（journal 的角色与格式） 已定项 14 注 3），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **当时挡着「新池新建文件写」的九样（里程碑一），① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（已定项 16）；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改新池新建文件的字节：那条记录偏移 48 的 8 字节从 64 变 6。 ⚠️ **写入时间单镜像判不出**：取值是发布参数，单个镜像里看不出参数是什么——O1（内存模型对拍（HashMap 理想文件系统）） 的模型与对拍不带写入时间，O2（独立解析器 + checker） 只判单个镜像，O3（独立规约执行器） 的投影不进 inode 树；判它的是按写路径钉写的 harness 用例（覆盖写、顺序写、建 inode、unlink 各一条），多单元顺序写、回退、同一片叶里被连带重写的别的记录三格没有用例（C677（写入时间在三格写路径上没有用例））。

⚠️ **死亡分三层的两笔代价**：墓碑从意图那次生效，unlink 途中扫描重建读不到那条 nlink = 0 的记录与剩下的数据单元，崩溃点上重建视图里不多出意图已发布的对象（E178（叶容器与记录死亡的计数） 第二、三段）；借时右片保留身份，借之后它的容器号可以不大于左片的最大 key，I-9.4（容器号不超最小 key） 第三句（左容器的最大 key < 右容器号）随这一形删掉，不变量清单与 checker 已跟着改（2026-10-09 第 50 批）；代价是「分裂时右半容器号给小了、仍大于左邻号」那一类镜像上没有判据拦，只剩 `inode_tree` 单测。

**依据**：

- E98（inode 记录与 inode 树的几何）：inode 记录的盘上形态——打包进码 3 不触发 D21（权威态与派生态的分界） 硬约束 6、住索引节点触发（14 个字段里 11 个算不回来）；改动计数单靠 checkpoint_txg 排不了同窗口内两个版本，所以内部节点不缓冲、同一发布窗口只有最后一版落盘。
- E103（inode 打包路 ① 的更新代价）：路 ①（独立容器索引 + inode 树按身份引用）的读代价比容器索引并进内部节点高、还多一次单元读，而单次写不变——容器索引不另立树。
- E104（扫描重建的现行版本判定）：不认退役记录时退役容器里已死的 inode 会复活；模型打出「重生取最小 inode 号当容器号会撞车」，所以重生保留容器号、只换出生树与出生代。
- 四轮三方论证后用户定案 2026-09-05（材料 `_d8-item6-path.md`、`_d8-item6-r2-amended.md`、`_d8-item6-r3-conclusion.md`、`_d8-item6-r4-conclusion.md`、`_d8-item6-r5-landing.md`，四轮三腿输出 `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` 与 `research/results/d8-item6-r{1,2,3,4}-local.out`）。
- 用户定案 2026-09-12（C297（inode 内部条目里出生树两份没有定处理），两轮三方）：身份引用里的出生树与子指针里的那份两份并存，原话在变更史。
- 用户定案 2026-09-16（C143（inode 号水位在崩溃恢复落到旧根后退回去重发），三轮三方）：唯一性收窄到 (inode 号, 对象出生代)，原话在变更史。
- 用户定案 2026-09-23：`blocks` 按文件长度向上取整、不表示分到的空间，`st_blocks` 挪到做 POSIX 面的里程碑（主 agent 的提议，C480（inode 记录的 blocks 怎么算全仓没有条款）），原话在变更史。
- 用户定案 2026-10-01：K-R3-3「写根长高」，`research/prompts/m3-delete-r3-main-verification.md` 第三节 G6-墓碑树根满（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K3：写入时间在各写路径上的取值（两腿同，攻方在副本上量过、主 agent 复跑坐实），三条 oracle 都判不到。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」）、时间字段取「乙 意图带时间、三格同改」，原话在变更史；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n3 那一行（收窄掉「ctime 跟着改」没回问用户，记录偏移 64 起三格写同一个值）。
- 用户定案 2026-10-05：射程加「同一条时间线上 inode 号不复用」（「写吧」），判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T15 那一行，原话在变更史。
- E176（inode 树叶合并三候选的容量与写代价）：合并买到成倍容量（Q1r，对负载不稳定），左吸收右与左从右借多重写的叶片按位置记法不翻面、按内容记法可忽略——合并与借都做；借到刚好够下限时内容记法的多重写由「可忽略」变「不翻」——借取均分。
- E177（逐树不写墓碑的复活数）：删空的叶容器不落盘、什么都不写时，照 kb 择现行的扫描重建读法下 inode 树复活——容器身份消失要另有载体（死亡分三层的第 ② 层）。
- E178（叶容器与记录死亡的计数）：退役记录带死亡 txg、按谱系滤时，借与吸收、管理员回退的历史上容器复活与误判死亡都是 0；文件变短按重建出的 inode 记录判死活时复活与误判死亡都是 0；墓碑从意图那次生效时 unlink 途中漏 1 条记录与剩下的单元、崩溃点上不多出对象；借时给右片换身份每次比保留身份多写码 3、码 2 各一个与一条退役记录。
- 三方判决 `research/prompts/m3-leaf-death-r1-main-verification.md`（第二节 K1：「死亡由现行叶版本体现」不整条推翻、改字面；K3：均分借）、`research/prompts/m3-leaf-death-r2-main-verification.md`（第二节：退役记录带死亡 txg、随现行的树只按 P2 择版本；分隔 key 取孩子这一版的最小 key；均分借维持）、`research/prompts/m3-leaf-death-r3-main-verification.md`（第二、五节）。
- 用户定案 2026-10-08：「M3 连借一起做」（岔路单 `m3-3-5-forks.md` M 那一行）；容器身份消失写退役记录·甲、文件变短取读法 Z、墓碑从意图那次生效、借时分隔 key 取孩子这一版的最小 key 并删 I-9.4（容器号不超最小 key） 第三句（岔路单 `m3-leaf-death-forks.md`，判决 `research/prompts/m3-leaf-death-r3-main-verification.md` 第五节）。原话在变更史。
- 退役记录的写者只追加、死亡 txg 取身份消失那次发布、读者与 checker 只认谱系上的：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K2 的「死亡 txg 取哪次」「同一身份退役几次」两行（两次攻方抽样都坐实：死亡 txg 取 unlink 意图那次或最后一版的诞生代号会误判死亡，写者「已有就不追加」或「按身份覆盖」会复活）与第三节第 3、4 条；用户定案「开池就建（推荐）」，原话在变更史。

**欠**：C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）、C102（索引节点头没有预留区，加字段不是 compat）、C103（容器索引无落点）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C106（开放容器数上界）、C107（索引节点的 AAD 身份无落点）、C110（跨头共享与加密后元数据类的块头一致性读法）、C113（扫描重建时多版单元的现行版本判定无输入）、C115（inode 属性的溢出路线无落点）、C119（冻结组件没有 spec 文件）、C347（祖先表读不出时克隆头认领什么没有条款）、C677（写入时间在三格写路径上没有用例）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:253-270`（整段抄，未转述）**

```markdown
#### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value

**定案**：**`seq` 宽 4 字节，住 value 不住 key**，射程覆盖记账 / 反向索引 / LRU 三棵 write buffer 树，**一棵树一套条目格式、三棵同宽**。宽度定死之后因果倒过来：**`T_dirty` 的上界从此是挂载时判定**（4 字节 = 32 位窗口，按 34 字节/条对应约 136 GiB）；D16（发布语义） 已定项 5 那句「`T_dirty` 是可调值，不是格式常量」原样成立，改的是它多了一个由格式定的上界。**在 seq 里再放一份 txg 是把 key 里已有的段抄第二遍**，不许。两条骑手：

1. **反向索引与 LRU 定 key 时必须让去重窗口同样被 `T_dirty` 界住**（代进 key，或别的等效办法）。4 字节够用的依据是「同一个 key 的待去重条目必然同属一个发布窗口」，而那一条在记账树上由 D5（快照 / 空间记账机制） 已定项 5 的 key 末段是「代」推出来；那两棵树的 key 布局全仓一个字都没有，不兑现这条骑手，4 字节在它们身上没有依据。
2. **同一个 key 的跨批次插入顺序必须与 `seq` 同向**，不许留成隐含假设——btree 层同 key 是「谁后写谁赢」，不是「谁 `seq` 大谁赢」，而按叶分区的并行 flush 下两个批次的物理落地先后不受 `seq` 约束。

**射程**：它定的是条目里这一段的宽度与落点，不定别的段（记账 key 的 `(统计量类型标签, 树 ID, 设备, 代)` 四段在 D5（快照 / 空间记账机制） 已定项 5）。⚠️ 这一段不是与别段正交的：那四段在**同一个 checkpoint 内不唯一**（幂等完整值会被写很多次），去重规则「后者胜」逐字要求「后者」由条目自己携带 ⇒ 不定它，那四段合起来还不是一个定得下合并结果的 key。它**立在 D8（核心索引结构） 不立在 D5（快照 / 空间记账机制）**，因为 write buffer 覆盖「反向索引 / LRU / 记账」三类，立在记账那边会让另外两棵树各自重问一遍。⚠️ 记账条目因此从 30 字节（key 22 + value 8）变 **34**，E97（记账与分配记录的条目编码） / E71（记账 key 的条目数与宽度） / E54（记账的「代」要保留几代） / E99（write buffer 条目与 seq 的去重） 四页的数要重算。⚠️ 「inode 号水位」那条记账条目照样带 `seq`（已定项 15 第 3 条是格式要求），只是它的合并结果取最大值、不依赖 `seq`。⚠️ 取值规则（从几起、一次发布里怎么数）在已定项 10；前端与节点缓冲怎么衔接在已定项 12。

**依据**：

- E99（write buffer 条目与 seq 的去重）：量的正是同一个 key 的待去重条目在一个发布窗口内能有多少条、去重要几位——窗口需求落在 32 位以内，4 字节够。
- E17（write buffer 合并上限）：第一版漏了条目序号，两条臂与真值当场对不上，逼出「条目必须自带 `seq`」这个可判定形态；按叶分区可并行的实测同时说明并行 flush 下物理落地先后不受 `seq` 约束，骑手 2 因此必须显式写下来。
- 三条已定条款的字面冲突（三方论证，主 agent 逐条现查）：D5（快照 / 空间记账机制） 已定项 2 逐字「不用找 ⇒ 是点删不是扫描；触及的 key 数 = 统计量数」——`seq` 进 key 之后同一前缀下有多行，点删当场不成立；已定项 6 逐字「条目照样带 seq……只是**合并结果不依赖它**」——单调统计量取最大值要求两条相遇，而 `seq` 进 key 之后它们是不同的 key、根本不会相遇；D5（快照 / 空间记账机制） 已定项 5 的 key 定义本身不含 `seq`，而「后者由 `seq` 定」只有在 `seq` 不进 key 时才自洽。主 agent 补的第四条：`seq` 进 key 会让 key 不可构造——折叠后同一代只留一条，读者想查 `(标签, 树, 设备, 代)` 的值时不知道 `seq`，点查退化成前缀扫。
- 用户定案 2026-09-06：宽度取 4 字节并把因果倒过来、射程扩到三棵树、`seq` 住 value（原话在变更史）。

**欠**：C161（缓冲两级并存而衔接没定）、C162（seq 进 value 之后四个实验的数要重算）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C416（去重窗口被 T_dirty 界住那条骑手没人查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:271-315`（整段抄，未转述）**

```markdown
#### 已定项 8：树表条目的字段表

**定案**：**树表条目合计 200 字节** = **树 ID 8**（打头）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76。每层 ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** 棵，二层封顶 81² = 6561 棵。⚠️ **分母的口径写死在这里**：16384 减**含预留位的码 2 头**（131 是树表单元自己的码 2 头，key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18；条目区起点是含预留位的头末尾，已定项 11），不减别的数。

- **① 快照进树表**，每个只读快照两条条目：码 14（快照 inode 根）与码 15（快照 extent 根），根指针逐字抄建快照那一版活头的两个根；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**，不是根节点的诞生代号（那是它最后一次 COW 的号，这棵树自己算不出来）。它给 I-3.7（活快照可枚举） 一个载体。
- **② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**。逐字形态：`新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。「根环里全部根记录」读作根环里全部自证过的根（被抛弃时间线上的也算），并上环里全部自证通过的 journal 记录新根段带的该字段（D23（journal 的角色与格式） 已定项 15）：根所在的槽读不出而它那条记录还在时，恢复本来就从记录把那条根重建出来，记录带的水位就是那条根的水位；journal 记录要等它对应的树写好、根落盘之后才会被绕环盖掉。根与它那条记录都读不出（双重介质故障）的那一格盖不住，记在 C342（树 ID 水位在根读不出时退回去重发）。从 mkfs 之外的水位发第一个文件版本时，十三棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65548（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops、退役树）。⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。
- **③ `previous_snapshot_txg` 每头一份，住树表条目**（8 字节）。它是**每个头一个标量**不是每个单元一份，答的是「这个头从哪个快照分出来」，与单元自己的内容无关 ⇒ 按 D1（数据可移动性 / 反向索引） 已定项 4 的判据它进不了单元的自包段。一条树表条目上有**两个各自独立的 8 字节 txg 字段**——诞生 txg（这条自己发布的号）与 `previous_snapshot_txg`（它前驱那个快照的号），对活头条目与快照条目都有定义。
- **④ 兼容档取「树表条目自带长度」这一层，不取「索引节点每条目自带长度」那一层**：前者让旧读者**跳过**一棵不认识的树；后者让旧读者**解开**新树的节点，而**没有消费者**——它不是被判成本高，是被判**买不到要的东西**。
- **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行原样带到新代**（值不变、只换代号；有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错）。台阶行不换代段（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），上一版那次发布水位变了时同一行既是现行行又是台阶行。
- **㊁ flags 其余 15 位**：**未知位一律拒收——该条目 EIO**（对象级，不拒整个池）。**不取**「忽略未知位、继续按已知语义解释」，那是静默误读。配两条纪律：**必须被旧读者看见的改动走系统配置 incompat 位，不许塞进每条目的 flags**；**checker 不许走「跳过」**——I-3.1（已分配统计对得上） 逐字「已分配空间统计 == 实际遍历**所有**引用得到的和」无限定，checker 跳过一棵不认识的树，遍历和就短一截，判出来的是假阳性红而不是漏检，它遇到不认识的树的正确行为是**报「无法完整遍历」并停**。
- **排序契约与嵌套层数**：**条目按树 ID 升序排；嵌套层数不设格式上限**。排序取树 ID 的理由：它是五元组、记账 key、容器索引里到处用的那个键，且 ② 让它随创建序严格递增。不设上限的理由：设上限等于多一个失败模式，而那个上限还随条目宽变。
- **平局判别子取 `(诞生 txg, 树 ID)`，零字节**：D16（发布语义） 已定要用 `(txg, 快照 id)` 打破并列，而 ② 改住根记录之后树 ID **严格随创建序递增**（水位累计、只增不减，崩溃只烧号不回退——I-7.8（根记录树 ID 水位不低于全池最大树 ID) 把这一条变成可判的）⇒ `(诞生 txg, 树 ID)` 就是与创建序一致的全序，而树 ID 本来就在条目里。
- **头 ID 8 字节的取值**：inode 树条目写自己的树 ID（65537），extent 树条目写它服务的那个头（65537），`deleted_inodes`、墓碑树与退役树写 inode 树的树 ID（65537），谱系树写 0（全池一份），logged_ops 写 0（整理意图条目的字段表已定，见 D26（后台整理与放置回收） 已定项 2；C364（整理意图记录的字段没有逐字段对账） 仍欠字段表与实现的逐字段对账，I-4.4（意图只描述目标态） 的字段白名单判法已实现），分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属，与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数）；中央映射树不进树表。见 D5（快照 / 空间记账机制） 已定项 9。 ⚠️ 括号里的 65537 是树 ID 从 mkfs 水位 65536 起发的那一版（新池新建文件）上的取值，判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、`deleted_inodes`、墓碑树、退役树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0。树 ID 不是从 mkfs 水位起发的那一版（水位已被抬过，或一个克隆头的 inode 树）上，头 ID 随这张表里 inode 树的树 ID 走，不再是 65537（D5（快照 / 空间记账机制） 已定项 9「可写头的 ID = 该头 inode 树的树 ID」）。判它的是 I-9.27（树表条目的头 ID 取值）。
- **预留 76 字节**：已知认购者按「一段字节只有一个意思」排，**逐条登记在树表条目预留的认购表里**（门禁 doc-registries 的 tree-table-reserve 格按它算和：认购合计不许超过 76，表里每条要指得到一个还开着的欠账或一条已定分项）。合计 48，余 28。

<!-- gate:tree-table-reserve total=76 -->

| 认购者 | 住哪条条目 | 宽度 | 出处 |
|---|---|---|---|
| livelist 每头计数器 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 第二道销毁闸的活快照计数 | 头条目 | 8 | C272（第二道销毁闸没有 O(1) 的检查法） |
| origin 链接 / 克隆时刻 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 快照谱系的区间标号 | 快照条目 | 16 | C144（快照谱系的区间标号没有载体） |
| `children`（子快照计数） | 快照条目 | 8 | D5（快照 / 空间记账机制） 已定项 18 |

⚠️ **inode 号水位不在表里**：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 2026-09-16 定案「不搬水位，水位照旧住记账」（D5（快照 / 空间记账机制） 已定项 4 第 12 项）⇒ 它不认购这里的字节。

**改新池新建文件的字节：是**——树表单元 t12 的条目宽字段（200）、声明长度（12 × 200 = 2400）与 12 条条目的排布。

**射程**：⚠️ **㊀ 的射程只到「树 ID 不认识」那一半，另一半的处方是相反的**：不认识**统计量标签**那一半由 C71（不认识的统计量标签静默忽略） 管，而它逐字要求「挂载时遇到不认识的统计量标签**必须按 compat_ro 拒绝可写挂载**」——㊀ 不覆盖也不推翻它。两半分得开的机制依据：D5（快照 / 空间记账机制） 已定项 5 的 key 定宽 ⇒ 不认识标签**含义**的写者照样能按固定偏移切出各段；成立的机制是已定项 1「记账条目取幂等完整值」，一条行的值对搬运者不透明也搬得动，而写者不认识那棵树就改不了它 ⇒ 值本来就不变。不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的行没人重写，下一次发布就消失。⚠️ **暴露面很窄**：只有一个统计量带树维（D5（快照 / 空间记账机制） 已定项 4 第 12 项，只为可写头维护）⇒ 不是可写头的树一行都没有；再加一个带树维的统计量，不搬的丢失量就按树数长。⚠️ **㊁ 拒收不丢数据依赖 ④ 的条目自带长度**：条目 EIO 但**位置还在** ⇒ 写者知道「那儿有一棵树我读不了」⇒ 不做跨它的 deadlist 合并；没有长度字段就定位不到下一条，这条推导不成立。⚠️ **㊁ 与本工程对另外两个 flags 字段已定的政策是同一条**：D18（块里携带什么信息）「偏移 7 是 8 个 flag 位……读者遇到非 0 位一律拒收……本工程不给这 8 位留「忽略陌生位」的通道」；已定项 6「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**」，已落成 I-9.7（记录三段恒零）⇒ 三个 flags 字段并成一条，不用再解释它们为什么不同。⚠️ **「位」和「种类」是两根轴**：D15（格式冻结政策） 已定项 2 那条 compat（新增 keyspace 对旧读者只是看不到新树）管的是**种类**，㊁ 管的是**位**——一棵旧读者自以为认识的树被加了位，它的语义变了，那不是加法。⚠️ **嵌套层数不设上限的代价是性能悬崖**，按 `.claude/rules/fs-design.md` 硬要求 4 必须可观测：「每次发布 COW 的树表单元数 = 层数」要能在运行时读出来——走错层数不报错，没有观测手段就永远发现不了。⚠️ **① 交付的那张表不够用**：D5（快照 / 空间记账机制) 点名的「全部根的 txg 列表」写在 D6（快照实现模型） 已定项 1（每头一棵自己的树）之前，而 D6（快照实现模型） 用一个反例打穿了它的前提（主线 txg=10 建可写快照 X、在 X 里于 txg=30 写一个块、主线 txg=35 建快照 Y，按 D5（快照 / 空间记账机制） 的条件判「Y 引用它」而 Y 根本看不见它）⇒ 要的不是「每个根一个 txg 标量」，是**谱系**；诞生 txg 这 8 字节仍然非有不可（I-3.5（引用区间的精确性） 要按 txg 升序枚举全部快照根与活头、I-3.6（deadlist 紧界） 要 `prev(S).txg`），欠的是**在它之上再加一层标号**（嵌套区间标号是两个数，而且随谱系变化要回头改写，与「诞生 txg 写一次不再动」是完全不同的格式性质）。同一个前提的三个消费者里定案只兑现了 I-3.7（活快照可枚举） 一个：I-3.5（引用区间的精确性） 的可判定形式建在「全部根按 txg 排成单一全序」上，谱系成树之后那一条恒假；D18（块里携带什么信息） 的代际回收走同一个区间测试，方向是过判 ⇒ 少回收 ⇒ 泄漏。⚠️ **③ 只补了活头头部的半格**：D5（快照 / 空间记账机制） 逐字「每个可写头各持一份 `previous_snapshot_txg` **与一份 deadlist**」「建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的」⇒ 快照条目也要能指到一份 deadlist，而 deadlist 的盘上落点全仓一处都没有。⚠️ **丙-2 唯一盖不住的那一格判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元；判它不构成错误的依据是孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」）、I-1.2（块头写序已发布） 逐字「判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留」、扫描重建的两道闸与孤儿回收条件都不看树 ID、孤儿与活版共用五元组本来就是常态（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 加 10 字节写序正是为此）、D6（快照实现模型） 判据 9 的理由句说的是「树里的 key 变成可见」而孤儿不在任何树里。由此 D6（快照实现模型） 判据 9 的射程按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。⚠️ **inode 号水位另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去；2026-09-16 定案不搬水位，把唯一性收窄到「已发布的 (inode 号, 对象出生代) 不复用」（已定项 6）。⚠️ **树表容量在两处按不同条目宽算**：E79（根记录的容量） 与 D22（单元原子性怎么合成） 已定项 7 报的每层棵数是另一个模型（它把树表条目当成「树 ID 8 + 根指针 59」= 67 字节），方向不变（间接层躲不开、超过一层再嵌一层），只是门槛比 D22（单元原子性怎么合成） 正文写的低。⚠️ 预留里那两笔已知认购（deadlist 的落点、快照谱系的嵌套区间标号）**没有顺手命名成字段**——命名等于把两个未定项一并定了，那是两个决定。排序契约（① 树 ID 严格升序，② 合发号次序）由 I-9.16（树表条目按树 ID 严格升序且合发号次序） 判：重建与冷走读在读任何一棵树之前拒，池级 checker 另判一份（实审 A3b Q2 与树表排序实现，用户 2026-09-27 定立不变量并同步）。第一版同时活着的只读快照至多 10 个（用户定案，常量 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`；树表单节点封顶 34 个）。快照按码 14 条目的树号点名（第一版没有目录）。有活快照（现行版或目标版树表里有码 14 / 15 条目）时管理员回退一律拒（用户定案，D23（journal 的角色与格式） 已定项 14）。

**依据**：

- E112（旧写者遇到不认识的树）：跑前写死判据与接受条款下六格的静默结局与搬运代价——`Skip + Carry` 在挂得上的那些里零静默结局，`Ignore` 那一格记录被按错编码解释，`Refuse` 全挂不上且与 D15（格式冻结政策） 已定项 2 的 compat 档不相容；搬运代价每代只多写两行，跑前定的作废阈值未触发。
- E113（不认识的树·补齐臂集）：臂集补成 5 flags × 3 记账、选法改成跑前写死的字典序之后，`RejectEntry` 在全部扫描点里一次都没输过，`Skip` 在第 1 关就输（隐形快照使 deadlist 合并认错邻居）、`Ignore` 输在第 2 关、攻击腿提的按位分档 `PerBit` 输在它没有第三种行为——㊁ 由此从「跳过该条目」重定为「该条目 EIO」；记账那一维取 `Carry` 不取 `Exempt`（当时 `Exempt` 要在 D5（快照 / 空间记账机制） 已定项 2 的 K 代上开例外，那笔代价一个格子都没占；2026-09-23 K 代点删撤回，`Exempt` 臂失去对象）。
- 用户定案 2026-09-06：四问（快照进树表、树 ID 水位住根记录、`previous_snapshot_txg` 每头一份、兼容档取条目自带长度）、㊀ ㊁、排序契约与嵌套层数、平局判别子、预留取 32 字节（原话在变更史）。
- 用户定案 2026-09-23：② 的 max 也并上环里记录新根段带的水位（C342（树 ID 水位在根读不出时退回去重发） 前置那一问；C511（回退到无文件那一版之后诞生代怎么接） 第 3 步的实现员先选了这一边，用户同日认下，原话「journal 日志就是为了保证树能完整写入的」），原话在变更史；发号次序是主 agent 同日照格式常量次序定的，被攻过零轮。
- 用户定案 2026-09-14：树 ID 挪到条目最前、根指针 83 → 86、预留里切 8 字节做头 ID（原话在变更史；依据是已定项 11 定码 2 条目里 key 一律是条目的前 key 宽 个字节，树 ID 不打头就要给通用解码规则加例外）。
- 用户定案 2026-09-16：条目从 148 加宽到 200、预留 76，代价是每单元从 109 棵降到 81 棵、二层封顶从 11881 降到 6561，用户知情接受（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1：头 ID 读同表关系、不读字面 65537——字面读法在一段合法的高水位历史上误红 4 条，还放行头 ID 写死 65537 的坏实现（攻方在副本上量过，主 agent 复跑坐实）；判它的 I-9.27（树表条目的头 ID 取值） 按表判，树表有条目时才判。
- 退役树的头 ID 写 inode 树的树 ID、树表声明长度 12 × 200 = 2400：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条（树表条目的头 ID 写 inode 树的树 ID，同墓碑树，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。
- 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（每个快照两条新种类的树表条目，甲出局；单节点树表封顶 34）、S8（有快照时回退拒）与第七节用户定案（岔路 1 上限 10、岔路 3 有快照就拒；岔路 5 快照按码 14 的树号点名是主 agent 定的），原话在变更史。

**欠**：C144（快照谱系的区间标号没有载体）、C148（树表条目变宽把间接层门槛压到二层）、C149（根槽重发之后 birth 口径松一格）、C156（记录编码误读没有不变量）、C158（checker 跳过不认识的树会判假阳性）、C159（E112 的臂集缺三条且天平不含挂不上的代价）、C342（树 ID 水位在根读不出时退回去重发）、C675（同一种树在树表里至多一条没有不变量）、C865（树表条目未知 flags 位让整张树表读不下去）、C866（F2 退回不分字节丢了与内容拒收）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:316-353`（整段抄，未转述）**

```markdown
#### 已定项 9：树的种类码的登记表

**定案**：树表条目里「树的种类」那 2 字节的取值，唯一登记位是已定项 9 这一节的登记表，0 保留为「无」；**格式锁定之前没有旧盘，码可以重排、重用**，改码时 `crates/` 与 checker 在同一次改动里跟着改。

| 码 | 树 | 出处 |
|---|---|---|
| 0 | 无（树表条目不用它） | — |
| 1 | extent 树 | D4（校验和位置） 已定项 7 |
| 2 | inode 树 | 已定项 6 |
| 3 | 分配记录树 | D3（空间分配） 已定项 3 |
| 4 | 记账树 | D5（快照 / 空间记账机制） 已定项 3 |
| 5 | 中央映射树 | D19（块指针的结构与宽度预算） 已定项 5 |
| 6 | livelist 共享树 | D6（快照实现模型） 已定项 2 |
| 7 | 稀疏旁表树 | D5（快照 / 空间记账机制） 已定项 6 |
| 8 | deadlist 树 | D5（快照 / 空间记账机制） 已定项 11 |
| 9 | `deleted_inodes` 树 | 已定项 16 |
| 10 | 墓碑树 | D18（块里携带什么信息） 已定项 24 |
| 11 | 谱系树 | D18（块里携带什么信息） 已定项 24 |
| 12 | logged_ops 树 | 已定项 5 |
| 13 | 退役树 | D18（块里携带什么信息） 已定项 11 |
| 14 | 快照 inode 根 | 已定项 8 |
| 15 | 快照 extent 根 | 已定项 8 |

新池新建文件写出的 **12** 条树表条目照此表（码 1..4、6..13 各一条；中央映射树的根住根记录、不进树表，D19（块指针的结构与宽度预算） 已定项 11；码 5 照登记）；E142（新池新建文件的干跑） 的 `TREE_KIND_*` 常量是码 1..8 那八个值（码 9..15 它没有）；`crates/singlefs-core/src/records.rs` 与 checker 的 `walk.rs` 用同一组常量（码 5 不进树表，那里没有它）。

**射程**：它只登记「树的种类」这一段的取值，树 ID 是另一个量，两个量的值域不相交（树 ID ≥ 65536，取值在已定项 11）。与 D18（块里携带什么信息） 已定项 11 的单元类登记表是两张表、两个概念（用户不取合并）。码 8（deadlist）与 livelist 共享树、稀疏旁表同一个先例——都是「销毁快照时要读、而销毁不许全盘扫描」的结构，day-1 注册进树表比第一次建快照时惰性创建少一条「首次建树」路径；条目形态另立 D5（快照 / 空间记账机制） 已定项 12。**改新池新建文件的字节：是**——加码 8 那一次让树表从 6 条变 7 条、树 ID 水位从 65543 变 65544（树 ID 从 65536 起编，deadlist 是 65543）。

**依据**：

- 无实验：码值是取值约定，没有可量的量；登记表由用户 2026-09-13 定案（码 0..7）、2026-09-14 定案加码 8，两次都没有走三方论证、可推翻，原话在变更史。
- 码 9..12：判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行（两腿各自得出）；树 ID 与种类码值域不相交由用户定案 2026-10-01 取（岔路单 `m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：新池新建文件写出的树表条目数按实现现值（判决当时 11 条，加退役树之后 12 条，`crates/singlefs-format/src/lib.rs` 的 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。
- 码 13 退役树、新池新建文件写 12 条：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条（种类码追加登记、树表排最后、day-1 零根注册，取值照墓碑树与谱系树的先例，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。
- 码 14 快照 inode 根、码 15 快照 extent 根：判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1（取两条新种类的树表条目，追加登记、不重用码；甲出局两腿一致，具体字段零轮）。
- 格式锁定之前码可以重排、重用：用户定案 2026-10-31（里程碑 17 收尾、格式锁定之前没有旧盘，不兼容旧格式），原话在变更史。

**欠**：C315（树的种类码没有登记表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:354-365`（整段抄，未转述）**

```markdown
#### 已定项 10：第一版记账直落叶，`seq` 的取值规则

**定案**：**第一版记账直落记账树叶，不走 write buffer 前端**；D5（快照 / 空间记账机制） 已定项 3 第 3 条读成「前端是后续优化」。**`seq` = 同一个 `(标签, 树, 设备, 代)` 在一次发布里被写进树的第几次**（不是运行期那个统计量被改的次数），**从 1 起**；第一版禁边插边 COW（D19（块指针的结构与宽度预算） 已定项 12 的固定点：每个码 2 / 码 3 一个实例一个 checkpoint 只写出一次，发布末尾一次性 COW）⇒ 每个 key 每次发布只写进树一次 ⇒ **恒 1**；合并规则取最大值的统计量也照写。

**射程**：它定的是第一版走哪条路与 `seq` 怎么数，不定 `seq` 的宽度与落点（已定项 7）、也不定前端与节点缓冲的衔接（已定项 12，第一版两级都不开）。放开固定点、开前端或节点缓冲之后「恒 1」还成不成立，由已定项 13 的状态折叠接着管。

**依据**：

- 无实验：第一版走不走前端是实现排期的选择，没有可量的量；由用户 2026-09-13 在写代码前的总审核里定案，没有走三方论证、可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:366-399`（整段抄，未转述）**

```markdown
#### 已定项 11：码 2 的载荷布局与 key 比较序

**定案**：码 2 节点头自描述（预留 2 之后是条目数 2 + 条目宽 2，key 宽 1 排在 key 区间之前，偏移表见 D18（块里携带什么信息） 已定项 18），基础字节数 81 → 86；含 29 字节预留位后头 = `115 + 2 × key 宽`。条目区从含预留位的头末尾（`115 + 2 × key 宽`）起，**定宽条目连续排列**，**声明长度 = 条目数 × 条目宽**；条目之后的空闲字节恒 0 且参与载荷 CRC。载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`（起点是明文头末尾，口径与码 1 的 105、码 3 的 107 同）。**条目里 key 一律是条目的前 key 宽 个字节**——「读条目宽 → 按条目宽切条目 → 取前 key 宽 个字节当 key」是一条对全部码 2 树都成立的通用解码规则，扫描重建路径上不用另查一张「这棵树的 key 在条目里排第几」的表。key 的全序按逐字段无符号整数、字段自左向右比较，**盘上小端存储不构成 memcmp 序**。树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，两个量的值域永不相交）：extent 65536、inode 65537、分配记录 65538、记账 65539、中央映射 65540、livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547、退役树 65548；mkfs 时水位 65536，第一个文件版本那次之后 65549。

**改新池新建文件的字节：是**——六个码 2 单元的条目区起点（`115 + 2 × key 宽`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）、声明长度，以及树表条目的字段次序；树 ID 字段（树表条目里的树 ID 与头 ID、码 2 头里的树 ID、记账 key 的树 ID 段）与树 ID 水位的值也变，随树 ID 从 65536 起编。

**多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

- **一次发布里先删后插**：上一版有、这一版没有的 key 先按 key 升序删，再按 key 升序插这一版新有的；只决定形状，不决定写序（写的是最终态，已定项 13）。
- **从盘上重建的树分隔 key 坏了**：重建只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），不核分隔 key；下一次发布按分隔 key 删不掉时规划交回，可写挂载在取号之前拒，盘上不变。遇到这种盘之后怎么办（修复、只读挂载）没定。
- **叶条目宽不由读树判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的；内部条目窄于 key + 86 由读树报。

**内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。分配记录树与 extent 树按已定项 14 另起按 key 空间寻址的结构，结构在三方 `m2-keyspace-r1` 判；照码 2 btree 做时它们的内部条目是 96 与 110（extent 110 是用户 2026-09-24 早先定的）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**射程**：它管码 2 节点的载荷怎么排与 key 怎么比，不管头里各字段的偏移（D18（块里携带什么信息） 已定项 18）、也不管每棵树的条目里除 key 之外还有什么（各树自己的分项）。唯一因「key 打头」需要重排的是树表条目，已按这条改成树 ID 打头（已定项 8）。「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」在中央映射条目回到定宽 55 之后不用动（D19（块指针的结构与宽度预算） 已定项 10）——条目宽字段只有一个 u16，按类两宽时这两句顶不住。两条 2026-09-14 补的定案都是用户弹窗定案，没有走三方论证，可推翻。⚠️ 分裂、收缩、分裂点、写序四样被攻过零轮，三方原型只抽了一次样；「低于一半合并」与「追加时末尾切」在崩溃历史上与选中的那两样零对零，没选的理由是用户的取舍，不是安全性。「分裂单开一次发布」出局：只改结构的发布挤掉 D16（发布语义） 已定项 1 的「最近 4 个可退到的不同状态」。inode 树的叶是码 3 打包记录单元，不是码 2 节点：「多层码 2 树怎么长、怎么收」里 ② 的「叶装不下时从中间切」与 ③ 的「不做低于一半合并与借条目」管码 2 节点，够不着 inode 树的叶层；叶层的末尾分裂、左吸收右与左从右借（均分借）照已定项 6。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：量的正是码 2 头自描述要加的那几段与映射树 key 宽的代价——加到 86 之后扇出、树高、解头那几节都不掉档，自描述值得做。
- 用户定案 2026-09-13：码 2 头自描述、基础 86、条目区起点、载荷 CRC 覆盖范围、key 比较序（原话在变更史）。
- 用户定案 2026-09-14：条目里 key 一律是前 key 宽 个字节；「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」原样成立（原话在变更史）。
- 用户定案 2026-09-24：extent 树内部条目 110 字节（原话在变更史）；E157（并行线一两条条款的计数模型） 在 110、136 两个取样点上结论都不翻面。
- 无实验：分裂、收缩、分裂点、写序四样是三方原型（`research/prompts/m2-treesplit-r1-opus-model/`）在同一批崩溃历史上零对零之后由用户选的，没立实验号；分隔 key 跟着维护是那一轮主 agent 按打中定（原型 t1.out：沿用 D8（核心索引结构） 已定项 6 的「分隔 key 创建后不变」24 格全红、跟着维护 24 格全 0），判决 `research/prompts/m2-treesplit-r1-main-verification.md`。
- 用户定案 2026-09-24：写序同段照常 COW、只摘空节点且根剩一个孩子时降高、从中间切、内部条目 = 本树 key + 子指针 86（原话在变更史）。
- `m2-treesplit-implementer-report.md` 第五节 B1、B5、B6。
- 用户定案 2026-10-01：树 ID 整体挪到 65536 起，与种类码的值域不相交（岔路单 `m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行：新树的种类码取 9..12（两腿各自得出）。
- 退役树的树 ID 65548 与第一次发布后的水位 65549：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条与第五节（取值照先例，被攻过零轮）；用户定案「开池就建（推荐）」，原话在变更史。

**欠**：无（C306（码 2 头不自描述 key 宽） 2026-10-11 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:400-413`（整段抄，未转述）**

```markdown
#### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级

**定案**：走 write buffer 前端的三棵树（记账 / 反向索引 / LRU）**ε 恒 0、内部节点不留消息缓冲区**；这三棵树上的写**一律进前端**——含已定项 8 ㊀ 那类旧写者搬运的记账行：有前端的写者把它当前端条目写，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错；**发布前把前端排空到叶**，落叶时同 key 比同一窗口的到达序；发布集的前端条目在同一次发布里排空、按事务切，不跨发布（fsync = 一次全量发布）。其余树按 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴留不留缓冲。**第一版两级都不开**（已定项 10、D11（索引节点要不要留消息缓冲区） 已定项 7），不改新池新建文件的字节。

**检查**：前端树的写路径只暴露前端入口（类型层）；运行时断言——一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目，窗口中途 flush 的与发布时排空的都算、实例切换重发时保留；重组、索引重建、损坏恢复重建三个例外各配一条「前后每个 key 读出来的值不变」。它只能是运行时的：盘上分不出经前端排空与直写落叶。按字面写的三种断言（只认发布时排空的、只认这一次尝试的、按物理新叶数）在合法历史上会误报（`c161-r3-model-output.txt` 的 `G3_publish_only` / `G3_attempt` / `G3_physical` 行）。

**射程**：**代价**：每次发布排空到叶的写出是「排空到根缓冲」那类形态的 3.4–10.7 倍（覆盖写负载 4.8–10.6 倍，同一产物 `part` 列的第三段），发布集的条目都得在这次发布里落叶、不能按预算分批；同一份脏页预算下前端装的条目少一个量级——`T_dirty` 按什么算没有口径。**买到的**：前端树上没有同 key 跨级并存，读的次序最短；陈旧检出（C52（陈旧当成合法））在这三棵树上没有对象。**没出局但没选的**：甲-R4（前端直落叶、节点缓冲只收不走前端的写）与乙-D（前端进根缓冲逐层下推、按位置定新旧）各只在塌缩上被打中一次，它们的陈旧检出要每条消息带发布号（与已定项 7「不许再放一份 txg」冲突），节点布局进 D15（格式冻结政策） 第 4 层。**连带**：重组时节点缓冲怎么搬（射程是全部留缓冲的树，这一项只省了前端三棵）；墓碑与叶上现值的折叠起点（已定项 13）；反向索引 / LRU 的时间线索；D11（索引节点要不要留消息缓冲区） 已定项 6 树轴的分支变量与已定项 5 的措辞。

**依据**：

- 无实验：这一格量的是设计形态在合法历史上会不会被分辨臂打中，依据是三轮三方（`research/prompts/c161-r1-main-verification.md`、`c161-r2-main-verification.md`、`c161-r3-main-verification.md` 各自第四节）——这一形在三轮试过的全部形状（乱序 flush、K 代点删、分裂与合并、部分发布、排空做到一半崩溃、删除、树高 4、索引重建、实例切换、管理员回退）上没被打中；出局的乙 + seq 全池单调两轮被打中（计数器不落盘；实例切换时计数器从所选根重新载入），还要 8 字节 seq 与落盘计数器。用户 2026-09-16 收尾弹窗定案，原话在变更史。

**欠**：C161（缓冲两级并存而衔接没定）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C348（T_dirty 按什么算没有口径）、C349（重组时节点消息缓冲区怎么搬没有条款）、C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:414-426`（整段抄，未转述）**

```markdown
#### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶

**定案**：一棵索引树上的同一个 key 在同一次发布里被改了多次（A → B → C → D → E）时，落叶只做一次：叶里这个 key 的条目直接写成最终态 E，A–D 不写进任何节点。「同一周期」是一次发布：同一个 `checkpoint_txg`，fsync 触发的也是一次发布（D16（发布语义） 已定项 6）。**E 按同一次发布里的到达序取最后一次改动，不按值取**；E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大（C383（同一次发布里同 key 的中间态折叠没有检查） 实测这种朴素折叠四条候选 16 / 16 丢点删，`b.out`）。**折叠的起点是叶上这个 key 在上一次发布留下的现值（没有则无）**：排空 = 以它为起点、按到达序依次应用本次发布里该 key 的前端条目，墓碑 = 删；起点算在 A–D 之前，不算同一次发布里的一个中间态。开前端之后，记账行的「上一代离开」落成墓碑，对着的就是这个起点。到达序在内存里由条目自己带着（已定项 15 第 3 条），不靠按 key 排序之后的相对位置。

**射程**：**全部索引树的全部落叶路径**。第一版的直落叶（已定项 10）之外，以后开的 write buffer 前端排空（已定项 12：落叶时同 key 比同一窗口的到达序，比出来的最后一条就是 E）、节点消息缓冲区（D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）、边插边 COW（D3（空间分配） 已定项 5 允许、D19（块指针的结构与宽度预算） 已定项 12 第一版不取的那条实现）都照这个规矩落叶。**第一版已经满足它**：D19（块指针的结构与宽度预算） 已定项 12 的固定点让已定项 10 推出「每个 key 每次发布只写进树一次」、`seq` 恒 1；已定项 13 把这件事从固定点的推论提成独立条款，放开固定点之后照样成立，不改新池新建文件的字节。实现今天没有对象：`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 一次覆盖写就是一次发布（txg、事务号各加一），没有「同一次发布里几个事务改同一个 key」的路径；同一个 checkpoint 里装两个对象的用例改的是不同的 key。⚠️ **没答的两条**：① A–D 在树之外的去向不归它管——A–D 是同一个逻辑块被覆盖写几次得到的数据单元版本时，按 D16（发布语义） 已定项 2 的丢失窗口与切分纪律「一事务一单元」，A–D 各是一个事务，各有一个落了盘的数据单元与一条 journal 记录（D23（journal 的角色与格式） 已定项 7：一条记录一个事务号），这几个单元没有根引用、也不在 I-7.4（近 K 代块未被复用） 的保护里，它们要不要写、写了在同一次发布里怎么释放、记账怎么算，决策正文、欠账表、不变量清单里都没有管它的条款；② 已定项 7 的盘上 `seq` 在折叠之下恒 1（已定项 10 的「恒 1」从第一版扩到所有版本），而认 E 用的是内存里的到达序，盘上那 4 字节 `seq` 还拿不拿来当 `T_dirty` 上界的依据（已定项 7 定案里「`T_dirty` 的上界从此是挂载时判定」那一句），没答。⚠️ 「单调统计量在同一次发布里不减、所以两种取法给同一个数」是主 agent 的推论，没有检查。

**依据**：

- 无实验：用户定「这个是决策 记录 暂时不需要做和验证」，没开三方、没建实验，也没有可量的量；用户 2026-09-19 定案与同日对「同一周期 = 一次发布」的确认，原话在变更史。A–D 可以不落的三条支点是已有条款（主 agent 2026-09-19 现查）：D16（发布语义） 已定项 4 与 D23（journal 的角色与格式） 已定项 14 第五条（施加的单位是一次发布，合法前缀停在一次发布的两个事务之间时那次发布整体不施加，所以 replay 出来的只有发布边界上的状态）、D16（发布语义） 已定项 6（fsync 触发的也是一次发布，被 fsync 确认过的状态都落在发布边界上）、D16（发布语义） 已定项 14（快照发布点对齐 checkpoint 边界、钉在这个 checkpoint 全部数据变更之后，没有快照引用 A–D）。
- 折叠起点一句与墓碑怎么折并进已定项 13：判决 `research/prompts/m3-close-c693-r2-main-verification.md` 第二节 V4 一格（缩题并进已定项 13 站住，两腿一致）与 `research/prompts/m3-close-c693-r3-main-verification.md` 第二节 T4 一格（站住；检查取排空对拍，台阶 H 严格递增那一判随里程碑十四的分树做）；改法照正推腿 `research/prompts/m3-close-c693-r2-sonnet-output.md`「V4.5 要动的条款、欠账」那一节。正推腿模型 `research/prompts/m3-close-c693-r2-sonnet-model/out/v4_fold_checks.out` 第 1、3 行：单头两次建 inode 的历史上 20 次发布之后，重建落法终态 18 行，朴素取最大 35 行、17 行陈旧（第二轮判决核查第 4 条主 agent 复跑 identical=1）。

**欠**：C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:427-484`（整段抄，未转述）**

```markdown
#### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构

**定案**：**一套 btree 实现 + 多个独立 keyspace + 两个前端，管权威态的树与照分裂做的树；派生态的分配记录树与 extent 树（D21（权威态与派生态的分界） 已定项 5、9）可以另起按 key 空间寻址的结构，格式可以与码 2 btree 不同。**

| 层 | 做什么 |
|---|---|
| 独立 keyspace | 每类数据一棵树（extent / inode / dirent / 反向索引 / 空闲空间 / 记账 …），互不干扰 |
| 日志结构节点 | 节点内新 key 追加不插入，内存里定期重排；写顺序、可在只读锁下回写 |
| **write buffer** 前端 | 高频小更新（反向索引、LRU、记账）批量攒起来排序去重后批量插入 |
| **key cache** 前端 | 同一批 key 反复读写的（分配信息）走哈希表，不做全树查找 |

**两棵派生树的结构**：

- **分配记录树**：按绝对槽号按位置寻址。每片叶罩一段固定的槽 `[k × W, (k + 1) × W)`，W 取偶数（两槽数据单元永远不跨叶），且 W ≤ 叶条目容量 812（一条记录至少罩一个槽，叶永远装得下）；记录按起点槽落叶。往上各层按位置寻址，子节点罩哪一段由它在父节点里的位置算出。**没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空（用户 K1）。checker 逐节点核「这个节点罩的就是它的位置规定的那一段」与「记录的末槽不越过它所在叶的末槽」。
- **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。**上段规范形**：上段形状只由有条目 inode 的号集合定，写与删共用这一条规则：节点取各有条目 inode 的根路径（从它那片叶到根那一串）之并，根层级取罩得住最大号的最低一层，集合为空时取层级 0 的零条目根；完整的重写规则在已定项 16 第 8 条。
- **挂载怎么读**：分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。
- **实现取值**（实二一取，主 agent 2026-09-25 定，被攻过零轮）：
  - **分配记录树**：
    - 叶宽 W = 812（取叶条目容量 (16384 − 135) ÷ 20 本身），内部条目 96 字节（key 10 + 子指针 86），扇出 169。
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - 层级 L 的节点罩 W × 169^L 个槽。
    - 内部节点与根只写有孩子的格：每个孩子一条内部条目（key 10 + 子指针 86），条目的 key 取这个孩子按位置罩的那一段的起点（孩子所在的盘, 那一段的第一个槽号），按 key 排，没有孩子的格不占条目；读者按这个 key 认孩子，槽号不落在孩子那一层的格点上、盘不在池里、或认出的孩子不在这个父节点名下的，这条条目不指任何孩子（`crates/singlefs-core/src/allocation_record_tree.rs` 的 `child_named_by_entry_key`）。根节点头的 key 区间罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)。
    - 根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。
  - **extent 树**：
    - 上段叶条目 113 字节 = key 24（(0, inode, 0)）+ 标签 1 + 载荷 88。标签 0 全零：inode 有记录、一个数据单元都没有时的条目——第一个文件（inode 1）没有数据单元时一律写它（size 0，或 size > 0 整个文件都是洞）；号 ≥ 2 的 inode 变长到 size > 0 而一个单元都没有时写它，之后缩短、缩到 0、再变长都留着，到 unlink 最后一批才去掉，`publish_new_inodes` 建出的 size 0 inode 不写；写者重写上段叶照写这一条；这一形上 size 改大、改成原值、截短（截到 0 也算）都只改 inode 记录、标签留 0、不加单元；标签 1 是下段根的节点指针 86 字节加 2 字节零；标签 2 是那一个数据单元的数据指针 88 字节。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - 上段叶罩 143 个 inode，内部条目 110 字节（key 24 + 子指针 86），扇出 147。
    - 上段叶节点头的 key 区间照 D18（块里携带什么信息） 已定项 2 射程段写这片叶按位置罩的那一段，不写叶里第一条与最后一条条目的 key：闭区间，第 k 片（从 0 起）smallest_key = (0, 143k, 0)，largest_key = (0, 143k + 142, 2^64 − 1)。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - 下段叶罩 144 个单元，下段 key 的字节偏移 = 单元号 × 32634。上段与下段的内部条目，key 都取这个孩子按位置罩的那一段的起点（`crates/singlefs-core/src/extent_tree.rs` 的 `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **发布路径的固定点**：猜重写集 → 在分配器拷贝上走释放（释放弄脏的叶先并进重写集）→ 取落点 → 算脏节点 → 不在集合里就并进去重来。第 64 轮还没收拢，就在任何落盘之前返回 `AllocationRecordTreeRewriteSetDidNotSettle`；64 没有依据，见新立的欠账。
  - **盘上槽数**：取盘上整槽数，盘尾不足一槽的零头不算；写侧与读侧只有一处定义（`crates/singlefs-core/src/allocator.rs` 的 `absolute_slot_count_of_device`），分配记录树根层两侧同一个数（实二六取，主 agent 2026-09-25 定，被攻过零轮；用例 `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`）。
  - **打开文件时自报的读数**：`ExtentTreeReadsAtOpen` 数的是读了几个 extent 树节点，不是块层的设备读次数；提示过期、经中央映射回退时每跳多 2 次设备读，两个数不相等（实二六查出，主 agent 2026-09-25 认这个口径、不另加设备读数，被攻过零轮；用例 `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` 钉相等的那一格，`a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` 钉不等的那一格，都在 `crates/singlefs-verification/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`）。
- 两棵树的节点照样带 D18（块里携带什么信息） 已定项 18 的码 2 头（树 ID、层级、key 区间、诞生代号），树高照 D8（核心索引结构） 已定项 11「树高 = 根节点头层级 + 1」读。

**射程**：⚠️ 异构只放给派生态、可重建的两棵树：分配记录树与 extent 树；记账树（权威态）、中央映射树、树表、实例表、inode 树照一套码 2 btree 做（D8（核心索引结构） 已定项 11 的分裂与收缩）——inode 树只有根与内部节点是码 2，已定项 11 管的是这一层；叶是码 3 打包记录单元，叶层的末尾分裂、左吸收右与左从右借（均分借）照已定项 6，已定项 11 的「叶装不下时从中间切」与「不做低于一半合并与借条目」够不着它。异构结构照样要：单个节点自描述（哪棵树、哪一层、管哪一段 key、第几代，D18（块里携带什么信息））、损坏可检出（D21（权威态与派生态的分界） 已定项 11）、每一种新写序进崩溃点重放崩溃点、checker 另有一套判它的代码。那两棵的结构在三方 `m2-keyspace-r1` 判。⚠️ 「日志结构节点」说的是**内存里的排布**——盘上每次落盘是整单元 COW 重写，按已定项 11 的单一定宽条目数组写（声明长度恰等于条目数 × 条目宽），不是多段追加的 bset。⚠️ **write buffer 前端第一版不实装**（已定项 10：记账直落记账树叶）；前端与节点消息缓冲区怎么衔接在已定项 12，条目要满足的三条硬要求在已定项 15。索引结构可以多样、事务层不许多样那条纪律在 `.claude/rules/fs-design.md`，本项只定索引这一侧。⚠️ 叶宽 W 的具体取值、上段扇出、上段叶条目的完整字段表，条款没写到的由实现员照上面几条取、交回里写明，主 agent 定后补进这一节。⚠️ 三种新写序（长高多层时位置 0 那一串节点同一次发布写、矮下去多层时同一串一起释放、内联与下段互换）要进崩溃点重放，checker 判定随实现一起做。⚠️ 这一节的四项都被攻过零轮；「全局按位置＋内联」的崩溃历史没扫（用户定全量崩溃最后统一跑）。⚠️ 一个数据单元都没有的 inode 带标签 0 条目这一形（第一个文件 size 0，或第一个文件 size > 0 整个文件都是洞），现有写路径自己写不出来（每条写路径都至少落一个单元；截到 0 只有读进来的标签 0 文件走得到），只来自读进来的那一版；写者对它的义务是重写上段叶时照写这一条、不丢。号 ≥ 2 的 inode 变长到 size > 0 而零单元的那条写路径已有：照判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节「别的文件的洞」那一格写粘住的标签 0（`crates/singlefs-core/src/transaction.rs` 的 `plan_the_files_other_than_the_first_after_this_publish`）。「有记录、没有条目、size > 0」在挂载态读者、三处冷读者、池级 checker 与 GPU 检查、恢复两个内核上判同一个结论：读者拒、checker 红。⚠️ 被否掉的候选：全局按 key 排（上段要分裂）、每文件一棵根住 inode 记录（要改 inode 记录这个独立冻结组件）、全局按位置不内联（一次挂载里连续建到第 14625 个小文件就假性报满，违反 D3（空间分配） 已定项 9 第 1 条）——理由在判决第三节。

**依据**：

- 无实验：一套 btree 实现配多 keyspace 的形态没有可量的量，依据是 2026-08-29 现查 `linux-6.17/fs/bcachefs/bcachefs_format.h` 的 `BCH_BTREE_IDS()` 宏（数 `x(` 行与带 `BTREE_IS_write_buffer` 的行，源码固定在 `~/code/fs-refs/`）：bcachefs 用**一套 btree 实现 + 21 棵树**，其中 **5 棵**走 write buffer（`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`）；比「按访问模式选不同结构」好在**只有一套结构要调对**——异构方案意味着 N 套实现、N 套崩溃一致性 bug、N 套 checker。prior-art.md 的 bcachefs 那一行以这一段为准。
- 用户定案 2026-09-24：派生树可以异构（原话在变更史）；多出来的那套实现、崩溃一致性与 checker 由崩溃点重放流与 checker 各自覆盖，不再拿「只有一套结构要调对」挡。
- 无实验：三方原型（`research/prompts/m2-keyspace-r1-opus-model/`）在同一批历史与代价模型上量过，没立实验号，选哪一个由用户定，判决 `research/prompts/m2-keyspace-r1-main-verification.md`。
- 用户定案 2026-09-24：K1、K2、K4（原话在变更史）。
- 实二一报告 `m2-keyspace-implementer-report.md`：实现员取值，主 agent 认，被攻过零轮。
- 实二六报告 `m2-e26-implementer-report.md` 第六节 Q3、Q4：盘上槽数与打开文件自报读数两条取值，主 agent 认，被攻过零轮。
- E142（新池新建文件的干跑） 第十五次跑第五段（`e142-r15-step5-runner-report.md` 第四节）：分配记录树内部与根节点那两格、extent 上段叶 key 区间那一格，独立装置的变体臂只有这一种读法逐字节对上 crates；这是归因，不是在装置主读法上验过，主读法按这几格改完的重跑在后面。
- 判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「上段规范形」：上段形状只由有条目 inode 的号集合定，写与删共用一条规则（两腿各自推出同一条规则；只升不降那条规则下同一个集合会出两种形状）。
- 用户定案 2026-10-22：标签 0 配 size > 0 的第一个文件，写者接得住这一形（弹窗选「丑：写者接得住这一形 (Recommended)」，原话在变更史）；判决 `research/prompts/m4-validate-code-r1-main-verification.md` 第四节第一行与「用户定案」一节，调查员报告 `~/.local/share/singlefs/work/m4-investigate-t5-tag-zero/report.md` 问一（现有写路径写不出这一形）与附带发现一节（标签 0 镜像上重写上段叶的发布丢掉标签 0 条目，写出的那一版三处读者都拒）。无实验：不改新池新建文件写出的字节，只定写者对读进来的这一形照写。
- 用户定案 2026-10-24：第一个文件没有数据单元时 size 0 也写标签 0（弹窗选「甲：一律写标签 0 (Recommended)」，原话在变更史）；判决 `research/prompts/m4-validate-code-r2-main-verification.md` 第四节与「用户定案」一节；第三轮判决 `research/prompts/m4-validate-code-r3-main-verification.md` 第二节「改法甲本身」那一格（两条腿各一次没打中）。无实验：不引实验号，只记用户定案与两轮判决。
- 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节「别的文件的洞」那一格与第三节：别的 inode 照第一个文件的标签 0，取正推腿的粘住形，读者与池级 checker 对「有记录、没有条目、size > 0」判同一个结论（被攻过零轮）；用户定案 2026-10-26：射程「任意活 inode 都能缩短与变长」（原话在变更史）。无实验：只把标签 0 的对象从第一个文件推广到任意 inode。

**欠**：C211（节点落盘形态没有正文）；C920（有记录无条目 size 非零那一形几处判定没对拍）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:485-502`（整段抄，未转述）**

```markdown
#### 已定项 15：write buffer 条目的三条硬要求

**定案**：

1. **排序会摧毁局部性而不是创造它 ⇒ 节点缓存必须是抗扫描的**（2Q / LRU-K 之类）。按叶排序之后每次 flush 就是一次覆盖整个叶子空间的顺序扫描，而 **LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0**。**这是硬要求，不是建议。**
2. **条目形态必须是幂等完整值，不许是增量 Δ**。记账那一棵咬得最紧——「释放了 4096 字节」的自然表达就是 Δ，处置见已定项 1 的定案。本工程因此与 bcachefs 分道：其 accounting 树是 delta 形态（`disk_accounting_format.h` 原文「updates are _deltas_」）。
3. **条目必须自带序号（seq），这是格式要求**。flush 时按 key 排序**丢弃时间序**，而去重规则是「后者胜」⇒ **「后者」必须由条目自己携带**；用不稳定排序时同 key 条目的相对次序未定义，去重结果因此不确定。

**射程**：三条管的是 write buffer 这条路上的条目与缓存策略，宽度与落点在已定项 7、取值规则在已定项 10、与节点消息缓冲区的衔接在已定项 12。⚠️ 要求 2 与要求 3 是两条不同的要求：幂等管「重复应用不出错」，序号管「谁覆盖谁是确定的」。⚠️ **已知代价**（bcachefs 原文）：write buffer 的更新无序且最终一致，flush 时按 key 排序丢弃时间序，且这个 sort-merge-sweep **本质单线程**，在多线程负载下它的效率是关键路径。这句有两种读法，实测结果相反——**线程读法**（合并的 CPU 吞吐撑不住多线程负载）**不成立**：单线程合并吞吐按每 4 KiB 用户写 3 条条目算远超盘的带宽，按叶分区还可并行；**I/O 读法**（flush 的树插入 I/O 是关键路径）**成立，且是三项里最大的一项**。⇒ **关键路径上真正卡住的是 D9（加密） 的加密，不是合并**；⚠️ E17（write buffer 合并上限） 只量了合并这一段，不含树插入与 I/O。

**依据**：

- E7（离线索引 harness）：本工程这一侧的 io/op 在缓存占树 2% / 6% / 25% 三档上逐位相同，干预证实保持同叶聚合不变、只打乱组的处理顺序就能让 25% 缓存档的读次数下降，100% 缓存档打乱前后完全相同（阴性对照）——LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0，所以要求 1 是硬要求；同一装置量出 flush 的树插入 I/O 折算下来远大于加密与合并，I/O 读法成立。
- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：增量 Δ 形态下 I-3.1（已分配统计对得上） 判别力归零，幂等完整值形态下全抓——要求 2 由它钉住。
- E17（write buffer 合并上限）：第一版漏了 `seq`，两条臂与真值当场对不上，逼出要求 3 的可判定形态；同一实验的单线程合并吞吐与按叶分区的并行倍数把「sort-merge-sweep 本质单线程」的线程读法判为不成立。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:503-535`（整段抄，未转述）**

```markdown
#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空

**定案**：

1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。
2. **分批**：对任一 inode N，从尾部按数据单元个数分批，每批释放 N 尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」，剩下单元数取 N 在 extent 树里的单元数（没有的取 0，只有最后一批）；K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），测试由只供测试的旋钮压小；删除批只重写 N 被截掉那一片下段叶与它的祖先，加上段里 N 那一串（K-R3-1，extent 写法的新分支，要过 `.claude/rules/fs-design.md` 五条硬要求）；最后一批释放 N 剩下全部单元、删 N 的 inode 记录、删 `deleted_inodes` 那条 key、把 N 的条目从上段去掉（不写标签 0），同一次发布；只有这一条路，不设一次做完的快路。意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。`deleted_inodes` 根是单节点：码 2 头的层级字节写 0，条目就是队列里的号；池级 checker 的删除态扫描只按层级 0 读它（`crates/singlefs-checker/src/walk.rs` 的 `deleted_inodes_queue_without_judging`），层级不是 0 的根判不了。
4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。
5. **删空**：「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`；判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树、谱系树与退役树。删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第三节 G5）。删空的一版上覆盖写、顺序写、unlink 拒成已有的「没有文件版本可改」那一种拒绝（`UserChangeRefused::NoFileVersionToChange`）。删完第一个文件、别的文件还在的一版归 `WithFile`：inode 树还有任一记录或 extent 树还有条目的一版是带文件的一版，不管 inode 1 在不在；第三个成员只指文件集合真的空。
6. **续做**：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加 `WritableMountPhase::Continuation`（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V3）。会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己；挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）。会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试；每一次被拒的删除类发布各算一次准入（一次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 次发布，`crates/singlefs-core/src/mount.rs`）；推满仍放不下交回「抬 F 之后仍没空间」的拒绝（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的留在盘上。挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同；推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常做成，靠的是 D16（发布语义） 已定项 1「正常卸载」那一行当场可用的那一截回收出来的槽（与 D16（发布语义） 已定项 1「准入」那一行同一定案）。任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。
7. **幂等的判据五条**：① 计划相等；①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、根与单元头里的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11。另加 harness 层每种发布的重写角色集合钉绝对值（零轮）（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V1；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G2）。不同操作序列的终态不要求逻辑内容相同（例如不同删序的墓碑死亡 txg 不同），要求上段形状只由有条目 inode 的号集合定、每个崩溃态 checker 全绿。
8. **上段规范形**：extent 树上段形状只由有条目 inode 的号集合定，写与删共用一条规则：节点取各有条目 inode 的根路径（从它那片叶到根那一串）之并；根层级取罩得住最大号的最低一层；集合为空时取层级 0 第 0 格一个零条目根兼叶；非根节点不许零条目。一次发布里，进出集合或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，别的照抄，上一版有而新形状没有的换下；长高那次旧根照抄成根之下的节点，降低那次上一版里降后层级的第 0 个节点照抄成根，旧根与脊线上方摘掉的节点换下（已定项 14 的 extent 树一条写同一个形状，`research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「上段规范形」）。

**射程**：第一版按 inode 号删任一 inode、无目录（用户 2026-09-28 定的是单文件、按 inode 号、无目录、不带 truncate，任一号由多文件删除的判决放开）；截断与变长不走这一条，走已定项 5 的截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md`），照这一条的走空与幂等纪律；删完第一个文件之后（K-U5，记欠账）按 inode 号写数据只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode，删之前就有的别的文件照常能写；取这个死亡 txg 只认这一版父链上的墓碑：「父链」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义，从这一版沿谱系记录往上走，只排掉被这条父链上的回退抛下的那几段 (T_old, T_rb) 开区间里的死亡 txg，不在父链上的谱系记录不排，再取最大；inode 1 的记录还在时不判；这一读法依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24），墓碑回收落地时要换读法（C618（删完之后不能再建带数据的文件））；崩溃流接在 `crates/singlefs-verification/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表里运行时决策路径「代价不许随盘容量增长」，在这一格读成只管准入之前（用户 2026-10-31 定，原话在变更史）：准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径。

**依据**：

- 无实验：数都在三方腿的副本原型上量（三份删除判决），入库装置「删除借保留量的环窗口」第一段已在套补丁表的副本上跑完（2026-10-03），第二段要在真实现的删除批上重跑、还没跑。
- 三方判决 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- 用户定案 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节队列上界与接口那一行：回退到意图那一版会把号带回队列，只有「会话里每次用户改动先走空」与「回退之后续做」两个臂在放开用户动作的历史上队列长度与有队列时的写次数两项都守住。
- 用户定案 2026-10-01：会话里每一次用户改动先走空（岔路单 `m3-blanks-r1-forks.md` 的「会话里的续做点」一行，原话在变更史）。
- 用户定案 2026-10-02：会话走空被空间拒推抬 F 再试；2026-10-03：每一次被拒的删除类发布各算一次准入、挂载续做被空间拒同样推抬 F 再试、推满仍放不下照 D16（发布语义） 已定项 1 做成可写、删除类发布各占一个事务号（原话在变更史）。
- 代码轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第二节 K2 那一行（续做、走空与推抬 F）与第四节（挂载续做推满仍放不下的结局交用户）；实现：会话走空推抬 F 是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，事务号是 `crates/singlefs-core/src/transaction.rs` 的 `publish_deletion_step`。
- 主 agent 2026-10-02 定 Q-L 照收（调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」）：删空的一版上覆盖写、顺序写、unlink 拒成已有的那一种拒绝。
- 用户定案 2026-10-03：「卸载必须做成」，续做推满仍放不下之后正常卸载靠 D16（发布语义） 已定项 1「正常卸载」那一行放开的那一截回收出来的槽做成，单盘幸存那一问取「甲 GA（推荐）」（原话在变更史）；设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`，代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n1 各行。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」），时间字段取「乙 意图带时间、三格同改」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n3 那一行。
- 判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节「删任一号」「上段规范形」「删第一个文件」与第五节用户定案：删任一 inode 的各步、上段规范形、K-U5 取「β 时序级」（用户定案原话在变更史）。
- 代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第二节 W4 与第三节第 1 条：β 取墓碑死亡 txg 只认这一版父链上的墓碑（攻方量过：删 inode 1、建 inode 3、回退到删之前、再删 inode 1、回退到建 inode 3 那一版，写 inode 3 做成，池级 checker 零红）。
- 判决 `research/prompts/m4-close-code-r1-main-verification.md` 第二节「T7 容器小树代价」那一行（判决当时 `ancestors_of_containers` 按全部容器数开标记表）；用户定案 2026-10-31：代价条款「管，只管准入之前」（无实验：用户定案，原话在变更史）。

**欠**：C618（删完之后不能再建带数据的文件）、C619（墓碑树根长高那次发布没有崩溃流）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C118（`deleted_inodes` 树的形态无落点） 还清。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:1-6`（整段抄，未转述）**

```markdown
## D28 挂载期承诺量 —— 已定

**中文** · [English](28-mount-time-commitments.en.md) · [日本語](28-マウント時コミットメント量.ja.md)

D28（挂载期承诺量） 管**准入这一格**：可写挂载在挂载期间承诺给自己的空间（不落盘、每次挂载重算），以及分配准入怎么读数。不管的：可写挂载的准入合取在 D2（RAID 条带策略） 已定项 13；checkpoint 保留池的权威句在 D23（journal 的角色与格式）「死锁 2 与假性 ENOSPC 是两种故障」一节，这里只定它怎么进准入（已定项 4）；准入不等式与在飞合成原住 D3（空间分配） 已定项 4 / 6，那两处只留指针、编号不动。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:16-73`（整段抄，未转述）**

````markdown
#### 已定项 1：准入不等式的权威形式

<!-- gate:admission-formula -->

```
可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)
```

**定案**：分配准入按这个式子算可用。它是「任何增加空间占用的操作，进门前先算最坏情况下解开自己需要多少」（D3（空间分配） 第 3 条）的可执行形式。

- **逐设备算，每块盘各自满足**（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，一块盘不够就拒，不拿别的盘的富余补。前五项（含挂载期承诺量里的切换预留、被抛弃根独占量）取这块盘自己的值——两份副本各落一块盘，一块盘满了不能靠别的盘补；后三项——待删占用、已承诺预留、checkpoint 保留池——是全池的承诺量，**按全部副本之和记字节**（物理字节，与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」同一口径），摊到每块盘各扣「副本之和 ÷ 副本数」。这个分界决定 D5（快照 / 空间记账机制） 已定项 4 里哪几个统计量要带设备维。「需求」按盘字节计、每盘相同（与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」逐盘物理字节同口径）。
- **挂载期承诺量**（第五项）：挂载期间常量、只住内存、不落盘、每次挂载重算，按设备分摊。成员两个：实例切换的预留（已定项 3，按设备摊）与 checkpoint 保留池（已定项 4，全池的量、每次发布现算，式子里单列第八项、逐设备摊扣）。
- **checkpoint 保留池**（第八项）：挂载期承诺量的池级成员，按当时各记录树的高之和 + 记账节点现算，每次发布重算、不设常数上限、只住内存；按全部副本之和记，摊到每块盘扣「副本之和 ÷ 副本数」（固定点写出的每一块按 D2（RAID 条带策略） 已定项 6 落两列，每块盘各落一份）；它保证 checkpoint 自己的固定点写得出去。
- **被抛弃根独占量**（第九项）：取值 = 分配器实际隔离的集合的槽数：只被被抛弃根引用的槽（被抛弃根引用的槽 − 回退候选集里的根引用的槽，下界 0），加可写挂载时预隔离的槽：水位之上没施加的 journal 记录点名的落点、那次发布的账里的落点、根环里比所选那一版新、按实例表有效、读得出的根的账里的落点（账读不出时照 D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」从中央映射现算出来的落点也在内），以及那条较新的根账读不出而记录齐时照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行并进它那一槽的回退复活那一份（回退目标 R_old 比所选那一版旧，它账里的落点）与宽护那一份（现行账里已释放的落点），不论它们有没有被回退候选集里的根引用，只除去分配位置着的槽（含已释放而还在 defer 窗口里的；那些算在「已分配」里，不在这里再算一次，挂载那一刻不隔离，`crates/singlefs-core/src/allocator.rs` 的 `DeviceFreeMap::isolate_the_slots_not_allocated`）；这次挂载里回收它们时分配器照还没解开的那一份补隔离，有两条路：挂载内记根时（`PoolAllocator::record_root_written_by_this_process`）、抬 F 那一串的回收（`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」；判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、`research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第二节 K1-3），与分配器实际隔离的集合是同一个数；各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算，被抛弃的根被轮转覆写时清零（C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放） 那一形与第九项字面不符：抬 F 那一串读盘得来、根环表没记着的被抛弃根，补的隔离写掉它那一槽也不放，回收时补不上、照旧整段隔离，与「已分配」双扣）；被抛弃的根只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。影子账开着时，分配记录树两块盘各多写一片叶，准入读数两边之差 = 被隔离的槽 + 这几个节点（实二一报告「停下交主 agent」第 8 条）。
- **待删占用**：恒 0 的保留位——删除中 inode 没释放的单元本来就在「已分配」里，再记一次是重复扣。
- **环窗口**（常驻扣项）：删除各批新写的兜底，环窗口 = (根环槽数 3 × S + 1) × W，S 是系统配置每区槽数（S = 4 / 8 / 16 时环窗口 13W / 25W / 49W）；与 checkpoint 保留池同类，只住内存、不进记账行，每次发布按几何现算，每块盘各扣这一份。W 是一次删除类发布每盘新占槽数的上界，W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，K 是删除批大小，N_ℓ 是分配记录树第 ℓ 层两块盘合计罩到的位置数，H_退役 是退役树的内部层数（从退役树根节点码 2 头现读，零根与只有根时 0）；6 + H_退役 是一次发布新写的退役单元上界：至多跨两片退役容器，2 × 2 + 1 + H_退役 + 1（两片容器各 2 槽、根 1、内部层 H_退役、根满长高多出的右半 1），不论合并连不连锁；这个上界在删除批只改尾叶时成立（D8（核心索引结构） 已定项 16）。删除队列非空时，可写挂载前那一判把环窗口算作可动用。

**接线**（实二五，被攻过零轮）：

- 发布路径在 `transaction::prepare_the_version_publish` 里判：算定这次发布的形状之后、读盘核与动分配器之前。
- 可写挂载在 `mount::establish_instance` 里判：取号与预演之前，这一刻的需求逐盘记 0。
- 拒了在任何写之前返回：发布路径报 `PublishError::SpaceAdmissionRefused`，可写挂载报 `MountError::SpaceAdmissionRefusedBeforeAcquisition`。
- 需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）；判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判（用户 2026-09-25 定 ND+A1n，被攻过一轮，安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`）；删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判；谱系容器（D18（块里携带什么信息） 已定项 24）归固定点、进 ckpt_cost 的 Σ 名单。读数取分配器此刻的计数。
- ckpt_cost 照已定项 4 的 Σ 名单算。
- 发布经挂着的会话调（`crates/singlefs-core/src/mounted_session.rs`，实五）时，准入拒或落点被拒由会话照 D16（发布语义） 已定项 1「准入」那一行推抬 F 的空发布、再重发；可写挂载在 `mount::establish_instance` 写行与暖机之后推。

**射程**：式子里每一项是不是被维护的统计量，逐项对照写在 D5（快照 / 空间记账机制） 已定项 4 的「式子逐项对照」表里（那里是唯一的对照位，五处例外——设备容量、挂载期承诺量、被抛弃根独占量、checkpoint 保留池与环窗口——也写在那里）；D5（快照 / 空间记账机制） 已定项 4 的统计量清单相对这个式子判完备。式子里的统计量读的是已发布值（挂载期承诺量、被抛弃根独占量、checkpoint 保留池与环窗口是挂载时或每次发布现算的内存量，不在此列），准入却发生在窗口中间，读数怎么合成归已定项 2。待删占用恒 0、W 上界、管理员回退不判式子与谱系容器进 Σ 名单、删除队列非空时挂载前把环窗口算作可动用被攻过零轮；删除类发布不判式子被攻过一轮。「删除批只改尾叶」这个前提在第 50 批之前的删空路径上不成立：删空一片叶时 `crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers` 曾按位置重写它右边的每一片，两次抽样副本都量到删空那次新占的槽数超过 W（100 MiB 盘 W = 36、删空那次每盘新占 45 / 85 槽；4 GiB 盘 30 片叶 62 槽、135 片 272 槽），第 50 批随 M3 改删除路径已恢复这个前提：删空、吸收一片时右边挪了位置、内容没变的片照抄上一版（`InodeLeafContainerOrigin::CarriedFrom`），只重写内容变了的叶与祖先；发布路径上 20 片与 40 片两种几何下第一片吸收第二片那一次每盘新写同为 14 槽（`m3-batch50-m3b-implementer-report.md`）。

**依据**：每一项为什么在式子里——

| 项 | 为什么在式子里 |
|---|---|
| 容量、**已分配** | I-3.1（已分配统计对得上） 的被审计对象 |
| **不可回收** | zoned 上一个活快照钉住的物理空间是 zone 粒度而引用的块数是块粒度，[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 逐字要求「另立一个不可回收量」 |
| **待删占用** | I-3.4（可用空间扣待删占用）；D5（快照 / 空间记账机制） 与 D8（核心索引结构） 分批删除定「待删除结构算作活引用」 |
| **已承诺预留** | D3（空间分配） 已定项 15；D3（空间分配） 已定项 2 逐字「墓碑就是解开自己的一部分，计入那个量」 |
| **挂载期承诺量** | 用户定案（C126（切换预留的最坏量没有口径） 第一轮）：实例切换的预留不落盘、按设备算，塞不进「已承诺预留」那一格 |
| **被抛弃根独占量** | 用户定案（C318（影子账隔离的单元没进准入不等式））：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账之后，只被被抛弃的根引用的单元在当前账里是空闲的、分配器却不许拿它们，直到被抛弃的根离开根环；它们既不在「已分配」里也不在「defer 待释放」里，不扣就高估可用。取值口径见 `research/prompts/m2-step45-code-r3-main-verification.md` 第六节 1 |
| **checkpoint 保留池** | 用户定案（已定项 4） |
| **环窗口** | 用户定案 K-R3-2（常驻扣项）：删除各批新写的兜底不进已承诺预留，在式子里另立一项、按几何现算 |

实验：

- E19（defer 窗口下的假性 ENOSPC）：保留池不进准入时 checkpoint 会卡死——第八项。
- E82（准入的在飞合成）：只读已发布值会同窗超卖——这个式子要配已定项 2。
- E141（切换预留的挂载准入自证）：按设备判与按池级标量判，在一块盘短 1 块而两块盘加起来够的池上分得开——括号里按设备求和。
- E150（回退复用被抛弃的根引用的单元）：带第九项的可用等于分配器发得出的槽数，不带就多报被隔离的单元——第九项。
- 用户定案 2026-09-23：准入逐设备合取（C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁）；待删占用与已承诺预留按全部副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款） 已还清，第三轮三方写回：一份副本那一读法让这个式子与 I-3.4（可用空间扣待删占用） 当场为假）；判决 `research/prompts/c355-c363-r3-main-verification.md` 第二、三、五节，原话与被推翻的那句 2026-09-13 原话在变更史。
- 实二五报告 `m2-e25-implementer-report.md`。
- 删「defer 待释放」、需求改成新写的全部槽：用户 2026-09-25 定案（原话在变更史），安全设计轮第二轮判决 `research/prompts/m2-safety-r2-main-verification.md`（「defer 待释放」与「已分配」重复扣）、第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`。
- 环窗口、待删占用恒 0、删除类发布与管理员回退不判式子、谱系容器进 Σ 名单、删除队列非空时挂载前那一判：`research/prompts/m3-delete-r3-main-verification.md` 第三节 G1-删除类发布怎么判、G1-标签 4、G1-W 上界、G1-窗口位置、G1-崩在删除中间的重挂载、G3-回退六行；W 上界的式子在辩方报告 `m3-delete-r3-sonnet-output.md` 3.1 节；用户定案 K-R3-1「只改尾叶（推荐）」、K-R3-2「常驻扣项（推荐）」，原话在变更史。
- 环窗口里的根数读根环槽数 3 × S：`research/prompts/m3-blanks-r1-main-verification.md` 第二节根环里的根数那一行（一轮）。
- 环窗口 W 加 6 + H_退役、删空路径重写右边每一片的前提不成立：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4 的「一次发布至多几条退役记录」「环窗口 W 里那 24 槽」两行与第三节第 6、7 条（上界式子是推的、零轮；删空那次超 W 两次抽样都在副本上量到）；用户定案「两样都做（推荐）」「并进 M3 改（推荐）」，原话在变更史。
- 第九项的取值扩成分配器实际隔离的集合（加水位之上没施加的记录点名、不被回退候选集里的根引用的槽）：判决 `research/prompts/m3-c694-r2-main-verification.md` 第二节「K1 条款空白定位」那一行与第三节第 3 条（`crates/singlefs-core/src/admission.rs` 的 `AdmissionReading::of_allocator` 把 `device_map.isolated_slots()` 整个当第九项），被攻过零轮；预隔离那部分不再扣回退候选集里的根引用的槽：`research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「t1 C 的候选豁免」与「t1 C 的盲区」两行（与根环线 `research/prompts/m3-batch10y-code-r1-main-verification.md` K4-1 同一形），被攻过零轮；预隔离再并上较新可读根的账与从映射现算的那一份：判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 3 条，用户定案 2026-10-08「四形都修」（原话在变更史），只在攻方模型上量过、被攻过零轮。
- 「需求」按盘字节计、每盘相同：用户定案（原话在变更史）。

**欠**：C70（跨统计量的和没人盯）：按这个式子写那条向量等式；C87（准入读的合成值无定义）。

````

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:74-89`（整段抄，未转述）**

```markdown
#### 已定项 2：准入的在飞合成

**定案**：准入读数 = 已发布统计量 − 在飞已批准的预留；窗口内产生的释放不发信用（D16（发布语义） 已定项 15 的准入侧形态）；「最坏情况下解开自己需要多少」含墓碑那一笔（D3（空间分配） 已定项 2 的可执行读法）。

- 在飞 overlay 是纯内存派生态：崩溃后按定义清零（在飞事务全部作废），不落盘、零格式位。
- 实例切换重建内存态时 overlay 按重放结果重算：施加一条记录把它点名的码 1 单元登记成已分配；所选根之后写出的码 2 / 3 单元按已发布谓词（D18（块里携带什么信息） 已定项 11）未发布、不登记、由重做重写；失败那次 checkpoint 的固定点分配与号 > W 事务的分配作废、重做时重新准入。

**射程**：切换那一句同时定了 C284（施加一条记录在指针层上做什么没有定义） 空间侧在切换这一格的读法；指针层其余部分仍归 C284（施加一条记录在指针层上做什么没有定义）。

**依据**：

- E82（准入的在飞合成）：三种直觉写法各出一类可数违例——只读已发布值同窗超卖、把窗口内释放记成可用会吃进 defer 该扣住的块（D16（发布语义） 已定项 15 被静默绕过）、忘了墓碑盘满后删除全数死锁。
- 切换那一句：C126（切换预留的最坏量没有口径） 第一轮三方论证（`research/prompts/c126-r1-main-verification.md`）之后的用户定案，原话在变更史。

**欠**：C87（准入读的合成值无定义）：E82（准入的在飞合成） 的三个红样例照抄进准入路径的单测与崩溃点重放，overlay 做成可强制清零的测试开关（清零后并发准入必须超卖、检查必须抓到——双向证）；`crates/singlefs-core/src/admission.rs` 的注释写明「在飞 overlay 没有实现」（准入读数的在飞合成今天没有代码）。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:90-111`（整段抄，未转述）**

```markdown
#### 已定项 3：实例切换的预留

**定案**：预留 = (N_switch + 1) × 一次切换的最坏量，按设备算；一次切换的最坏量 = 实例表链重写 + 暖机空发布。

- **链重写**：副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节。副本数 = 2（实例表单元指针的两个位置条目，D22（单元原子性怎么合成） 已定项 7；两份落两块盘，所以按设备各算一份 32768 × 片数）；每片行数 = ⌊(32768 − 135) / 行宽⌋ − 1（链指针记录恒为一片的最后一条，D18（块里携带什么信息） 已定项 11；行宽 88 ⇒ 369 行）；rows0 = 挂载时读到的行数加写行那次要写的行数（新实例 − max(所选根的实例, 1)，挂载时就知道；每次可写挂载都写行）。
- **暖机**：至多 R 次空发布 × 现算 c_max（R = 3，c_max 按已定项 4 每次发布现算），预留里这一半是 (N_switch + 1) × R × c_max；第一个可运行目标两块盘互为镜像，空发布的 c_max 块每块盘各占一份，所以按设备各加一份。
- **N_switch + 1 份**：N_switch = 3（D23（journal 的角色与格式） 已定项 14）；多的一份给写行那次发布的元数据——写行那次是新实例的第一次发布，之前不推抬 F 的空发布，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC。N_switch 那几份是逐次累加的上界：连续切换的前提是上一次的根没发布、重建态从同一个根来，上一次的链页在重建态里不存在，实际不累加。
- **固定点重做与号 > W 的数据单元重写不另要空间**：切换 = 挂载内一次恢复（D23（journal 的角色与格式） 已定项 14），重建态从所选根 + 施加到 W 的记录来；失败那次 checkpoint 的固定点单元与号 > W 事务的单元从没进过已发布的分配记录、也不被施加登记（已定项 2 的切换那一句）⇒ 重建态里它们是空闲的，重做在同一片空间上再做一遍，用的是 checkpoint 的保留池；号 > W 的用户数据分配失败按 D23（journal 的角色与格式） 已定项 14 向调用者返回 ENOSPC，不阻塞切换。
- **发布路径用 rows0 + 1**：发布准入里的切换预留按下一次可写挂载的 rows0 算（这次挂载的 rows0 + 1，下一次挂载必写一行）；可写挂载自己判时照旧按这次的 rows0（用户 2026-09-26 定，攻方在副本上量过：实例表行数到 366 那一格崩了再挂被拒 8 → 0；被攻过零轮）。
- **`df`**：可用已经扣掉这一项，`df` 报可用（D3（空间分配） 已定项 9 第 1 条）；没扣时出的是假性 ENOSPC（D16（发布语义） 已定项 1）。

**射程**：式子按「两份副本各落一块盘」写，只对两块盘的池没有歧义（D2（RAID 条带策略） 已定项 9 的第一个可运行目标）；**三块盘以上时两份落哪两块盘没写**，挂载判定该要求「任意两块」「空闲最多的两块」还是「分配器会选的那两块」都没定，空发布落哪块盘同样没写。第一版不跑 zoned，zoned 上 rows0 无界（C121（zoned 上实例表链长无上界））；非 zoned 上 rows0 由行回收压住，但回收要「无读失败 ∧ 全盘在线」且删行要可写挂载，近满盘 + 坏扇区的池上「预留拿不到 ⇒ 只读」可能成吸收态（推的，每 369 行才多一片）；每次可写挂载都写行、行回收要根环每个槽都读成功且自证过之后（D18（块里携带什么信息） 已定项 11「回收」那一条，清扫器 `crates/singlefs-core/src/instance_table_sweep.rs` 照它不删），一个根槽持续读不出、读回全零或重读仍自证不过时行只增不减，这个吸收态不再要求近满盘（C597（实例表行只增不删，删行的条件没有条款））。D16（发布语义） 已定项 5 只写 T_dirty 是触发条件、没写重做期间开放的 checkpoint 会不会堵住写者，那是 D16（发布语义） 的口径空白。

**依据**：

- E141（切换预留的挂载准入自证）：挂载准入按这个式子，在只差 1 块的两个池上于份额处翻，预留置 0 时都可写，池级标量判不出短的那块盘，七个错法各有取样点分得开；新池新建文件几何下每块盘的份额与各片边界的取样都在那一页，只建了两块盘。
- E148（提交固定点按两棵记录树重算）：c_max 的现算口径，新池新建文件规模的数在那一页。
- 三方：C126（切换预留的最坏量没有口径） 第一轮（`research/prompts/c126-r1-main-verification.md`），链重写的式子由两条云端腿改正；多的那一份来自 C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方之后的用户定案；暖机那一半连带 D16（发布语义） 已定项 8 的用户定案。
- 发布路径用 rows0 + 1：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md` 第二节「两处结构性分叉」那一行与第三节。

**欠**：C126（切换预留的最坏量没有口径）：实现侧的对拍与三块盘以上的落点；C121（zoned 上实例表链长无上界）；C597（实例表行只增不删，删行的条件没有条款）。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:112-142`（整段抄，未转述）**

```markdown
#### 已定项 4：checkpoint 保留池进准入

**定案**：checkpoint 保留池不设常数上限，按当时的结构现算，池多大它就多大。

- **形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算；每棵码 2 树当前的高 = 发布那一刻从它的根节点码 2 头里现读的层级 + 1（D18（块里携带什么信息） 已定项 18 的层级字段），不用内存里另存一份；按 key 空间定形状的分配记录树与 extent 树的高怎么读，随三方 `m2-keyspace-r1`。单位是 16 KiB 元数据块，代进已定项 1 那条按字节写的式子时乘 16384（式子里的「已分配」是「已分配字节」、「容量」是单元区大小，D5（快照 / 空间记账机制） 已定项 7）；它是运行时算出的量，盘上没有字段。
  - 分配记录树这一项按 K1 算：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L）；根层级是根节点码 2 头里现读的层级（树表 0 条、写过行的那一版取池几何的根层级），不读这一版的状态、不依赖 ckpt_cost，恒是上界：一次发布改几片叶的不动点收不了口（开放段装不下开新段、没有全空段回落、换下很久以前落在别处的节点，实审 A4c 量到一块盘改 3–4 片叶），拿得准的上界只有整棵树；代价随单元区线性涨：两块 4 GiB 盘 529 个节点，每块盘多扣约 2.7%（1 TiB 约 3.2%）。中央映射树这一项进 Σ，按它现有的删插式子算，删插数取「分配记录树这一项 + 记账树节点数 + 管理员回退那次发布多写的谱系容器、谱系树根各一把和谱系树内部层每层一把 + 退役容器、退役树根各一把和退役树内部层每层一把」（退役那几把随 ckpt_cost 的退役项，判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2，推的，零轮）（谱系容器是一个 32 KiB 单元、一把 key，按单元数算、不按块数），随分配记录树这一项的式子变而跟着变；
  - 记账树不进 Σ，单列，按每发布的节点数；
  - 树表单列一项，每次发布 1 个单元（D16（发布语义） 已定项 9：每次发布重写它，它没有高）；
  - 谱系容器单列一项，按一次谱系容器 COW 连祖先算：容器 2 块 + 谱系树根 1 块 + 谱系树内部层数，零根与只有根的谱系树 3 块、长高一层 4 块（管理员回退那次写它，D18（块里携带什么信息） 已定项 24；按一次谱系容器 COW 算被攻过零轮，连祖先算是用户 2026-10-02 定的，没攻过）；
  - 退役容器单列一项，按一次退役容器 COW 连祖先算：容器 2 块 + 退役树根 1 块 + 退役树内部层数，零根与只有根的退役树 3 块（最后一批写它，D18（块里携带什么信息） 已定项 11 类型 3；用户定案「两样都做（推荐）」，与 D3（空间分配） 已定项 2 标签 6 的退役项合起来这一形被攻过零轮）；
  - 实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留。
- **归属**：算挂载期承诺量，进已定项 1 的式子（单列第八项），落在已定项 1 分界的后三项那一侧：按全部副本之和记（ckpt_cost × 16384 × 副本数），逐设备各扣「副本之和 ÷ 副本数」。
- D16（发布语义） 已定项 1 的保留池（= ckpt_cost，也是 c_max）、推空最坏残留（5 + 6 c_max）与 D16（发布语义） 已定项 8 的切换预留都按同一个现算的 c_max 取。

**射程**：保留池本身（`reserve ≥ ckpt_cost`、死锁 2 与假性 ENOSPC 的区分）的权威句在 D23（journal 的角色与格式） 已定项 24，这里只定它怎么进准入。条款里不写常数：分裂合并、条带成员表、livelist 都没进 E148（提交固定点按两棵记录树重算） 的模型，它的 9 块是模型数。退役容器那一项只罩一次发布的暂态（一次发布新写的退役容器与它的祖先）；永久长出的槽由 D3（空间分配） 已定项 2 标签 6 的退役项预付，不在 ckpt_cost 里；「删 → 回退」环下两样合起来仍超，记在 C617（删后回退留下的墓碑没有预付罩着）。

**依据**：

- E148（提交固定点按两棵记录树重算）：聚簇提交块时固定点是与用户块数无关的常数，用户块随机或全散布时没有可用上界——所以现算、条款里不写常数。
- E19（defer 窗口下的假性 ENOSPC）：预留恰等于 ckpt_cost 时 checkpoint 就不卡死——保留池必须进准入。
- E173（上限卡在读坏槽的三种处置）：第 1 次重跑在 9 个几何上量了保留池按 ckpt_cost 现算与常数式 10 + 7 c_max 的差——现算只在小池上多拒 152 格、常数式一格都不更好（产物 `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out` 第 28382、28411 行）；用户 2026-11-02 据此定保留池按 ckpt_cost 现算，D16（发布语义） 已定项 1 同日改。
- 用户定案：不设常数上限、现算，原话在变更史。
- 用户定案 2026-09-23：逐设备合取（C355（checkpoint 保留池池级扣而固定点每块要落两列） 取丁），原话与它推翻的那句 2026-09-13 原话在变更史；判决 `research/prompts/c355-c363-r3-main-verification.md` 第五节第 2 题。
- 用户定案 2026-09-24：树高从根节点头现读（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T5）。
- `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2、第四节第 4 条。
- 用户定案：分配记录树这一项换成 K1（整棵树），2026-09-27 弹窗选「K1 整棵树」；代价与候选式子表在 `m2-rev-a4c-implementer-report.md` 第二节「候选式子」（K0 不是上界、实测三种情形一块盘改 3–4 片叶、不动点不收敛）。
- 用户定案 2026-10-02：谱系容器那一项「乙 连祖先算（推荐）」，中央映射树那一项的删插数「乙 一并算上（推荐）」，原话在变更史；调度记录 `records/2026-10-01-删除与并发事务.md`「十六　批 8」。
- 退役容器那一项：判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4 的 P-a、P-b 两行与第六节「两样都做」（照谱系单列一项单做，不回退也超卖，第一次抽样在副本上量到、主 agent 复跑重现；两样合起来那一形被攻过零轮）；用户定案「两样都做（推荐）」，两次弹窗的原话在变更史。

**欠**：C83（提交固定点没人回答）：实现后记固定点轮数，超过当时现算的量判红；分配记录树按 K1 的式子已写进 `admission.rs`（实审 A4d，`m2-rev-a4d-implementer-report.md`），交主 agent 的两处设计问题记欠：C573（固定点上界没算有普通分配那次发布多出的删插）、C574（落得下那一判不核两槽对齐与各盘同槽）。C576（三条用例被准入先拒连带红，改法待选） 已还清：不等设备的两条用例分别钉 `SpaceAdmissionRefused`、`PlacementRefused`，分配器回滚那条变异改指专门用例。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:8-29`（整段抄，未转述）**

```markdown

**D8（核心索引结构）的分项只有一套编号**，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **记账怎么表达成幂等完整值** | 记账取幂等完整值，不用增量 Δ；每条根装自己那一代的行（整批换代，不留多代），台阶行随根留 **状态：已定。** **进度：已实现。** |
| 2 | **节点大小取多少** | 16 KiB：mkfs 写进系统配置，实现按常量走，不按设备特性算 **状态：已定。** **进度：已实现。** |
| 3 | **key 布局** | key = `(locality_id, inode, offset)`，`locality_id` 创建时从父目录继承、改名时故意不更新 **状态：已定。** **进度：已实现。** |
| 4 | **节点大小的上限由什么定** | 上限由恢复时间定，不由可恢复性定；取值在「重扫代价 ≤ 目标线」下取查找代价最低的那档，靠实验 **状态：已定。** **进度：已实现。** |
| 5 | **whiteout 删除** | 接受墓碑；可能修改无界数量 extent 的操作先写意图再分批推进；第一版 unlink 的意图住 `deleted_inodes`（已定项 16） **状态：已定。** **进度：已实现。** |
| 6 | **inode 记录的盘上字段表** | inode 树叶是码 3 打包记录单元（类型 2，一叶 233 条定长 140 字节记录），内部节点码 2、条目 120 字节、无缓冲；(inode 号, 对象出生代) 发布后不复用 **状态：已定。** **进度：已实现。** |
| 7 | **write buffer 条目自带的 `seq`** | 宽 4 字节、住 value 不住 key，射程覆盖记账 / 反向索引 / LRU 三棵树，`T_dirty` 的上界因此是挂载时判定 **状态：已定。** **进度：已实现。** |
| 8 | **树表条目：树 ID 的分配与编码、树的种类、根指针以外还有什么** | 条目合计 200 字节、树 ID 8 打头；快照进树表；树 ID 水位取根环全部根记录该字段的 max；未知 flags 位拒收该条目 **状态：已定。** **进度：未实现。** |
| 9 | **树的种类码的登记表** | 码 0 无、1..8 是八棵既有树、9..15 是 deleted_inodes、墓碑树、谱系树、logged_ops、退役树、快照 inode 根、快照 extent 根；格式锁定前码可重排、重用 **状态：已定。** **进度：已实现。** |
| 10 | **第一版记账走不走 write buffer 前端；`seq` 的取值规则** | 第一版记账直落记账树叶、不走前端；`seq` = 同一个 key 在一次发布里被写进树的第几次，从 1 起，第一版恒 1 **状态：已定。** **进度：已实现。** |
| 11 | **码 2 索引节点的载荷布局、种类到条目宽的表、key 比较序** | 码 2 头自描述、基础 86 字节，条目区从含预留位的头末尾起定宽连续排列，key 是条目的前 key 宽 个字节 **状态：已定。** **进度：已实现。** |
| 12 | **write buffer 前端与节点消息缓冲区的衔接** | 一棵树只准一级：走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；第一版两级都不开 **状态：已定。** **进度：未实现。** |
| 13 | **索引树更新做不做状态折叠** | 同一次发布里同一个 key 改了多次时落叶只做一次、直接写最终态 E，中间态不写进任何节点，E 按到达序认 **状态：已定。** **进度：已实现。** |
| 14 | **索引的总体结构** | 一套 btree 实现、多个独立 keyspace、两个前端（write buffer、key cache）；派生态的分配记录树与 extent 树可以另起按 key 空间寻址的结构 **状态：已定。** **进度：已实现。** |
| 15 | **write buffer 条目的三条硬要求** | 节点缓存必须抗扫描；条目必须是幂等完整值、不许是增量 Δ；条目必须自带序号 `seq`，这是格式要求 **状态：已定。** **进度：未实现。** |
| 16 | **第一版 unlink 怎么删** | 第一版 unlink：只立 `deleted_inodes`，从尾部按 144 单元分批，挂载里续做到空 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:8-15`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **准入不等式的权威形式** | 可用逐设备算、每块盘各自满足：五项取这块盘的值，待删占用（恒 0）、已承诺预留、保留池按全部副本之和摊到每块盘，另扣每盘的常驻环窗口 **状态：已定。** **进度：已实现。** |
| 2 | **准入的在飞合成** | 准入读数 = 已发布统计量 − 在飞已批准；窗口内释放不发信用；「解开自己」含墓碑 **状态：已定。** **进度：已实现。** |
| 3 | **实例切换的预留** | (N_switch + 1) × 一次切换的最坏量（实例表链重写 + 暖机空发布），按设备算 **状态：已定。** **进度：已实现。** |
| 4 | **checkpoint 保留池进准入** | 保留池按当时的结构现算、不设常数上限，算挂载期承诺量，按全部副本之和记、逐设备摊扣进已定项 1 **状态：已定。** **进度：已实现。** |

```
