# m4-publish-without-whole-version-copy-design-r3 攻方腿报告

日期：2026-11-18。设计推论轮第三轮（最终轮），攻方腿（Opus）。前几轮判决：`research/prompts/m4-publish-without-whole-version-copy-design-r1-main-verification.md`、`research/prompts/m4-publish-without-whole-version-copy-design-r2-main-verification.md`、`research/prompts/m4-gpugc-publish-growth-r1-main-verification.md`。
攻击面：「已定」项撑不撑得住 / 未定项清单的完整性 / 第二轮报出的数字能不能核。不再攻三条候选谁最脆弱、甲三处难点的解法、测试与装置里的读者分类。
行号都是主工作区此刻的树；补丁行号是 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch` 自己的行号（只读）。没改主工作区的 `crates/`，没读禁读清单里的文件。
数的来源分三种，逐处标：「量的·核过」= 第二轮任务 19957 的日志或第二轮产物里逐字有；「量的·草稿」= 我在草稿目录对第二轮产物跑自己的脚本读出来的，待主 agent 复跑（下面「要主 agent 跑的命令」第 1 条）；「推的」= 读码推的。

## 复跑命令与文件指纹

模型目录 `research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/`：

```
b5c9f9cf913cea1182785045063731e00cda47bcab5a92fccfe99a725ca3bd11  probe/compaction-probe-on-the-cpu-arm.diff
1087455f1968a348297499ac175903689d8d73c2c3051e254e6a4536d9b5cda6  probe/publish-change-probe.diff
89a12a1124a4499b8d5ea450bea47d100284b3275902e718d924fd9c36f03f58  tools/recompute_copy_ratio_by_block_size.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
bf1504b83b4634d9ade4c3952765544c7336ac49397d18352a96a9101b1509da  tools/rerun.sh
041c66009c30c147d8b5d7286417965acbb6d0aaae1210f6efd38d792123cf34  tools/summarize_compaction_probe.py
```

- `probe/publish-change-probe.diff`：第二轮的探针原样（sha256 与第二轮报告里的相同）。
- `probe/compaction-probe-on-the-cpu-arm.diff`：打在第二轮探针之上的两处（只在副本里）：装置 `construction-timing` 每个 P 造完起步镜像之后，照 `geometry-check` 的那套阶梯在 CPU 臂上不计时地交替跑（不要卡），每一级跑完取走探针表、打 `phase=compaction_cpu_arm_user_changes_<次数>` 的逐次发布行与 `name=compaction_probe` 一行；探针另数分配记录按分配记录树的叶分块时碰到几片叶、那几片叶里几条记录、有记录的叶几片（`records_leaves_touched` 那三项）。
- `tools/recompute_copy_ratio_by_block_size.py`：按块长 16（只给上下界）、64、256 与「按叶分块」重算拷贝比，判 T1、T4；块长 64、256 的中位与装置打的分布行逐格比（两条路）；另报已回收集占比最大的那一行落在哪一段、已回收集在相邻两次发布之间缩了几次。
- `tools/summarize_compaction_probe.py`：整理那一段把带搬迁的发布（`relocated_units > 0`）单拿出来，按 P 报三张表，判 C1–C4；完整性闸（产物行数与装置报的数到的次数逐个相等、至少一个 P 量到带搬迁的发布）。造出来的合成产物上证过两种红（行数少一、没有带搬迁的发布，都退 1）。
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`：对 `--frozen` 那棵树建最小副本、依次打两份 diff、`cargo run` 装置 `construction-timing --unit-area 2288MiB --live-data-units 1024,2048,4096 --chunks shared`、删编译目录、跑两份汇总。模板的 `--selftest` 在假仓上起不了 `minimal-copy.py`，这一份的 `--selftest` 会红（推的，没跑）。
- 复跑：「要主 agent 跑的命令」第 4 条（外包一层 `rerun-from-a-clean-work-directory.sh`）。
- 探针的内存与时间（推的，没跑）：第二轮同档 P=4096 造池「construction_seconds=198.777」（第二轮产物 `name=construction_timing` 行）；P=1024、2048 按交错写每步随版本涨估各几十秒；每个 P 的 CPU 臂至多三级、每级 24–96 次用户改动加整理批，按第二轮交错写末尾每步约 180 毫秒估每级一到三分钟。全量估 10–20 分钟加编译，低于 40 分钟，不缩取样。常驻约第二轮那一趟的「resident_bytes=1092743168」再加挂载时拷的一份池，约 2 GiB 量级。
- 这一条腿不枚举崩溃状态：探针不录写流、不调 `enumerate_crash_states`。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 已回收集是什么、谁加谁删 | 没打中（读码核过） | 已释放、已回收、还没被复用的落点；回收时加、复用或复活时删、重挂清空；是记录表的一个子集，项数不超过记录条数 |
| 已回收集为什么涨、该不该有界 | 打中（第二轮判决的说法） | 「只涨不缩」不成立：第二轮产物里它在 1849 次发布上缩过（量的·草稿）；它涨是因为记录表里留着已死的记录（D3 已定项 7「条目留到该落点被重新分配时覆盖」），上界是槽数，不随发布次数无界涨 |
| 已回收集归这一题还是另一缺陷 | 没打中（另一缺陷）；改法换了 | 不是泄漏、不是另一缺陷；归这一题只因它进发布前那份克隆。第二轮攻方「换持久」的改法我撤回，改成「删掉这份集合、按记录与位图判」（零轮） |
| T5 的读数落在哪 | 打中（数对、归属错） | 0.861538 不在交错写段，在第二轮自己说「不判」的「放开垫片的槽」那一段（txg 6574）；交错写段最大 0.791809（量的·草稿）；翻面结论不变 |
| d 的定义：换下标、只改落点 | 没打中 | 三张表都按键比，换了下标不算；只改落点的项单元表、映射表、记录表都算进 d。映射表的值只有落点，比甲那张 `mapped_units`（角色 → key）多算，是往大里数 |
| d 的定义：拷贝式子与甲的块 | 打中一半（推不出） | 式子是「一层块指针整份拷」的二层表；块长 16 时块指针一项就占 1/16，T1 的 0.065 几乎全是它。甲最自然的块是分配记录树的叶（812 槽一片），这种块第二轮没量；按叶分块要拷的比上界推不出，补了计数（命令第 3、4 条） |
| 块长 16、256 时 T1、T4 翻不翻 | 没打中 | 块长 256：T1 0.032199、T4 0.161365；块长 16：T1 在 [0.064896, 0.072056]、T4 在 [0.077396, 0.121765]，都在 0.25 下面（量的·草稿，256 那两个数装置自己也打过） |
| 整理发布上的 d：装置有没有路 | 打中（走得到、但量不到） | 不要卡、走得到整理发布的只有 `geometry-check`；它三档全造、P 取登记值、不收收窄的参数，第二轮探针接上之后在它上面一行探针行都不打。补了副本改动与命令（第 3 条），跑前写死 C1–C4 |
| 数能不能核 | 没打中 | 第二轮判决第二节表 45 个数、翻面行 10 个记号、三段发布次数 3 个，逐字都在任务 19957 的日志里（58 处，0 处对不上） |
| 未定项清单还漏了什么 | 打中三样 | 验证包库代码里读版本的 40 行没进改动面；版本里分配记录每次发布 `to_vec` 加整排，两批之后照旧；发布前那份整份克隆两批之后照旧、还多带补丁的两份索引 |
| 「甲与不动形态两批并得住」 | 打中 | 补丁把记录表的主次序定成「记下的先后」、按键次序是附带的索引，甲要的是反过来；补丁新加的两份随记录涨的索引进发布前那份克隆，两批之后 T5 那一格的分子至少多一个记录表 |

## 格一：已回收集（`reclaimed_placements`）

### 它是什么、谁加、谁删（读码，主工作区此刻）

- 定义：「/// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。」（`crates/singlefs-core/src/allocator.rs:2295`），字段「reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,」（`crates/singlefs-core/src/allocator.rs:2296`）。
- 加：回收那一刻。`reclaim_released_records_up_to` 在全部记录里挑已释放、释放代不大于门槛的，「if !self.reclaimed.insert(key) {」（`crates/singlefs-core/src/allocator.rs:3347`）跳过已在集合里的，其余「device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));」（`crates/singlefs-core/src/allocator.rs:3355`）把槽放回空闲。
- 删，两处：新落点与已回收记录相交时，「let was_reclaimed = reclaimed.remove(&(device, record_slot));」（`crates/singlefs-core/src/allocator.rs:1568`），起点相同的那条记录改写成这次的分配、别的删掉；回退复活时「if self.reclaimed.remove(&(device, placement.slot)) {」（`crates/singlefs-core/src/allocator.rs:2878`）。
- 清空：重挂从记录重建分配器，「let mut allocator = Self::new(devices);」（`crates/singlefs-core/src/allocator.rs:2674`），`new` 里「reclaimed: BTreeSet::new(),」（`crates/singlefs-core/src/allocator.rs:2359`）；已释放的记录重进 defer 队列，下一次回收再进集合。
- 记录本身留着不删，条款在同一份文件的注释里：「/// 删完之后就没有记录，与从没分配过的槽同样（D3（空间分配） 已定项 7：条目留到该落点被重新分配时覆盖，而它罩住的槽正被这次分配重新占用）。」（`crates/singlefs-core/src/allocator.rs:1543`）。
- 我在 `allocator.rs` 里搜了改 `records` 与 `reclaimed` 的全部写法（`records.retain`、`self.records =`、`reclaimed.insert/remove/clear`），只有上面几处；没有「记录删了、集合里还留着」的路（只核到这份文件）。

所以已回收集 ⊆ 记录表的键（每项都有一条已释放的记录对着），项数不超过分配记录条数；每块盘上记录互不相交（I-5.4），记录条数又不超过槽数。它是「记录已释放 ∧ 槽已回到空闲」这一子集的索引，推的：可以由「记录 `is_released` 且那几槽在位图上未分配」判出来，复活那一支的分叉条件就是位图上占没占（`mark_allocated` 与 `mark_resurrected_out_of_the_defer_queue` 两条）。

### 为什么涨到与记录同阶（量的·草稿，第二轮产物）

`tools/recompute_copy_ratio_by_block_size.py` 对第二轮产物跑出的原样行（草稿目录 `block-size-scratch-run3.txt`）：

```
name=reclaimed_set_steps phase=sequential_half grew=1143 shrank=537 unchanged=368
name=reclaimed_set_steps phase=interleaved_half grew=2191 shrank=1180 unchanged=853
name=reclaimed_set_steps phase=unlinking_the_spacer_files grew=132 shrank=122 unchanged=6
name=reclaimed_set_steps_overall shrank=1849 publishes=6573
```

另从同一份产物读（草稿脚本 `scratch/reclaimed.py`）：交错写段开头到末尾，已回收集 17708 → 51790、分配记录 23030 → 65980，涨的比 34082 ÷ 42950 ≈ 0.79；末行（`releasing_the_spacer_slots`，txg 6575）已回收 56992、记录 66244、两盘空闲图槽数 292864。

读法：
- 「只涨不缩」（第二轮判决第二节、背景材料第 2 条的提法）不成立：6573 次发布里 1849 次是缩的。它缩在新落点落到已回收的槽上（上面第一处删）。
- 它跟着记录表一起涨：每次发布释放几十条旧记录（覆盖写换下的单元），它们过了 defer 窗口就进集合；造池一直往没写过的空间写（聚簇段从最低的全空段开），被回收的槽很少被再用，于是「死记录」越积越多。记录表本身就是活记录加死记录：活的约 2 ×（版本里单元数 4516）≈ 9032 条，66244 − 56992 = 9252（推的，差的那点是 defer 窗口里的）。
- 有界：上界是两盘槽数（292864），由 I-5.4 压着；造池那一趟只用到四分之一不到，池写满一圈之后它应当停在「写过又空出来的槽」那么多（推的，没量过绕满一圈的历史）。它不随发布次数无界涨。

### 该不该有界、归哪一题

- 不能「随回收门槛推进清掉」：相交那一处的断言要它（「…不是已回收的：分配器只发位图上空闲的槽，位图与记录分叉了」那条 panic），复活那一支靠它分叉。清掉它要么让断言误报，要么让复活走错分支（推的）。
- 不是另一个缺陷：没有泄漏的路，项数被记录条数压着。它与池同阶，根子是 D3 已定项 7 让已释放的记录留到被覆盖为止——那是盘上格式的决定，不归这一题。
- 归这一题只有一处：发布前那份整份克隆（`crates/singlefs-core/src/transaction.rs:13213`「let allocator_before_this_publish = allocator.clone();」）把它整份拷一次，T5 量到的就是这件事。
- 改法（零轮，只读码推过）：甲下不必把它也做成持久集合，删掉它，凡问「这个落点回收过没有」都改成看记录的 `is_released` 与位图；记录表与空闲图两样已在甲的持久化之列，这一项就跟着变成零代价。回收那一处「insert 返回 false 就跳过」改成先判位图。前提是「已回收 ⟺ 已释放 ∧ 位图上空」在今天每条路上都成立；它没有断言，要先在测试构建里加一条逐项比（与集合并存一段，比对不上就红），再删。

### T5 的读数落在哪一段（量的·草稿）

```
name=leftover_over_records_of_a_phase phase=interleaved_half max=0.791809 txg=6295 reclaimed_placements=51634 cluster_segments=448 abandoned_root_placements=0 allocator_records=65776
name=leftover_over_records_of_a_phase phase=unlinking_the_spacer_files max=0.845898 txg=6572 reclaimed_placements=55972 cluster_segments=62 abandoned_root_placements=0 allocator_records=66242
name=leftover_over_records_of_a_phase phase=releasing_the_spacer_slots max=0.861538 txg=6574 reclaimed_placements=57008 cluster_segments=62 abandoned_root_placements=0 allocator_records=66242
name=leftover_over_records_overall max=0.861538 phase=releasing_the_spacer_slots txg=6574
```

- 第二轮攻方报告的跑前登记写「放开垫片的槽」那一段「上一趟 `publishes=0`，不判」，汇总脚本的 T5 却在全部段的每一行上取最大（第二轮 `summarize_publish_change_probe.py` 的 `every_row`）。这一趟那一段有 2 行探针行（装置报的 `publishes=0`，探针数到 2 次），最大值就出在那里。背景材料与派发提示把 0.861538 与「交错写段最大 51790 项」写在一句里，读起来像同一段；交错写段自己最大是 0.791809，那一行的已回收是 51634、不是 51790（51790 是交错写段末尾那一行）。
- 不改翻面结论：0.791809 仍远过 0.05 那条线。改的是归属与「全部段」的射程，判决引的时候写清楚。

## 格二：d 的定义与甲真要复制的块

### 探针数的是什么（读第二轮 diff，`research/prompts/m4-publish-without-whole-version-copy-design-r2-opus-model/probe/publish-change-probe.diff` 新文件 `publish_change_probe.rs` 那一段）

三张表都是「两份按键升序的表逐条比」，d = 加 + 删 + 值变：

| 表 | 键 | 值 | 照抄但换了下标 | 只改了落点 | 与甲那张表的差 |
|---|---|---|---|---|---|
| 单元 | 角色（`TransactionUnit` 派生次序，同一角色取最先一项） | （落点，字节那份 `Arc` 的地址） | 不算（按键比，下标不进值） | 算（落点在值里） | `PublishedUnit` 只有 `slot`、`identity`、`bytes` 三个字段，值罩全了；照抄换了角色的算删一加一，与按角色当键的表要做的相同 |
| 中央映射条目 | 映射 key | 第一份位置项的槽（进映射的角色取它在 `units` 里的落点） | 不算 | 算 | 版本里没有这样一张表：`mapped_units` 是（角色 → key），落点不在里面。探针拿 `units` 的落点拼成 key → 槽来数，只改落点的项甲在 `mapped_units` 上一项不用拷，这里却算进 d——往大里数 |
| 分配记录 | （设备，槽号） | 整条记录 | 不算 | 算（改写、释放、隔离记号都在值里） | 与甲的记录表一一对上 |

没打中的那一半：「换了下标」三张表都不算，「只改落点」三张都算；对按键寻址的甲，这正是要拷的那些项，口径对得上（映射那张偏大，不影响「不翻」的结论）。

打中的那一半在「块」上，不在「项」上：
- 拷贝比的式子是「碰到的块数 × 块长 + ⌈N ÷ 块长⌉ 个块指针」÷ N（装置 `probe_lines_of_the_phase` 的注释与第二轮汇总脚本同一个式子）。这是只有一层块指针、每次整份拷块指针的二层表。块长 16 时光块指针就是 N ÷ 16，比值至少 0.0625：下面 T1 块长 16 的 0.064896–0.072056 几乎全是块指针。换成多层的持久树，块指针那一项是「碰到的路径 × 层数 × 扇出」，小得多（推的）。所以块长越小，这个式子越高估；结论「不翻」对多层树只会更稳。
- 甲最可能取的块不是「按键次序每 64 条」，而是分配记录树的叶：「- **叶**：盘 d 上第 k 片叶罩槽 `[k × W, (k + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]（偶数、≤ 叶条目容量 812）；」（`crates/singlefs-core/src/allocation_record_tree.rs:3`）。不动形态第二批已经按这个块做了跨发布带着走的每叶共享（补丁第 1296 行「+struct LeavesOfTheLastPublishedVersion {」那一段，上面两行注释说每片叶的记录共享、分配器整份拷贝时只拷引用，转述），版本里那张表在盘上也是按叶写的。按叶分块要拷的条数 = 碰到的叶里的全部记录，与块长固定的式子不是一个量：两盘各约 146432 槽 ÷ 812 ≈ 181 片叶，交错写段记录中位 43582 条，平均每片约 120 条（推的）；d 中位 72 条若落在 72 片不同的叶里，要拷约 72 × 120 ÷ 43582 ≈ 0.20，加块指针约 0.21——离 0.25 不远，而碰到的若是满叶（至多 812 条）就会过线。第二轮产物没有这个数，推不出翻不翻。
- 所以在副本的探针里补了按叶分块的三项计数（`records_leaves_touched`、`records_in_the_leaves_touched`、`records_leaves`），跑前写死 T1 按叶分块的翻面线（同 0.25），命令在第 3、4 条。

### 块长 16、256 时 T1、T4 翻不翻（量的·草稿；块长 256 的中位装置自己打过，两条路对上）

```
name=block_size_flip live_data_units=4096 reading=T1_allocation_records block=16 copy_over_n_median_low=0.064896 copy_over_n_median_high=0.072056 threshold=0.25 flipped=no
name=block_size_flip live_data_units=4096 reading=T1_allocation_records block=64 copy_over_n_median=0.025178 threshold=0.25 flipped=no
name=block_size_flip live_data_units=4096 reading=T1_allocation_records block=256 copy_over_n_median=0.032199 threshold=0.25 flipped=no
name=block_size_flip live_data_units=4096 reading=T4_central_mapping_entries block=16 copy_over_n_median_low=0.077396 copy_over_n_median_high=0.121765 threshold=0.25 flipped=no
name=block_size_flip live_data_units=4096 reading=T4_central_mapping_entries block=64 copy_over_n_median=0.074861 threshold=0.25 flipped=no
name=block_size_flip live_data_units=4096 reading=T4_central_mapping_entries block=256 copy_over_n_median=0.161365 threshold=0.25 flipped=no
```

块长 256 的中位与装置 `name=publish_change_probe_phase` 行里的 `copy256_over_n_median` 逐格对上（脚本末行「✓ 块长 64、256 两条路的中位逐格对上」）；那个数第二轮没进汇总、没进判决。块长 16 产物里没有逐行的碰到块数，只给界（下界：碰到的 16 条块不少于碰到的 64 条块；上界：不多于 4 倍、也不多于 d），上界都在线下，翻不了。
另两段只报不判：顺序写段单元、映射两张表块长 256 的中位是 0.513944、0.493321，已过 0.25（那一段表还小，N 中位一千出头，256 条一块只有四五块）；这说明块长与表大小有关，块长要随 N 选或用多层树，不能写死一个大块长（推的）。

结论：「64 条一块」换成 16 或 256，交错写段的 T1、T4 都不翻（没打中）；按叶分块推不出，待第 3、4 条。

## 格三：整理发布（带搬迁的发布）上的 d

### 装置走不走得到（现读 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`）

- 不要卡就走到整理发布的，只有 `geometry-check`：「"geometry-check" => CommandOfTheApparatus::GeometryCheck { product },」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:9285`），它开跑时 `CardsOfTheCommand::NotNeeded`（`:8067`），每档造完起步镜像在 CPU 臂上交替跑「run_lockstep_once(&setting, LOCKSTEP_ARMS[0], 0).map(|summary| summary.batches);」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/stage_three_cost_subcommand.rs:13`）。CPU 臂的整理发布最后经第二轮探针的接点 `publish_version_of_trees_relocating_units_after_a_check`（`transaction.rs` 里调它的五处有两处在整理那一段，`:11110`、`:11180`；读调用处推的，副本那一趟的 `relocating_publishes=` 是判据），探针数得到。
- `lockstep` 给 `--arms` 只点 CPU 臂也要卡：它开跑时写死「CardsOfTheCommand::TakenBeforeOpeningTheDevices,」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/concurrent_runs.rs:31`），服务不交卡就打 `clause=no_cards_from_the_task_server` 退 78。`construction-timing` 不跑 GC。
- 但 `geometry-check` 在第二轮的探针副本上量不到：
  1. 一行探针行都不打。第二轮 diff 只在造池的钟开段、收段时取走探针表（diff 第 163 行「+        let counted_between_the_phases = singlefs_core::publish_change_probe::take_the_counts();」），而钟的探针行只在 `construction-timing` 里写进产物（diff 第 345 行「+        for (name, fields) in &clock.probe_lines {」）。`geometry-check` 造池用的钟在 `construct_a_starting_image_of_the_size` 里建、用完就丢；之后交替跑里的整理发布推进进程里的表，没人取。
  2. 跑不完。它三档全造（「for tier in UnitAreaTier::EVERY_TIER {」，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/stage_three_cost_subcommand.rs:53`），P 取登记值（「construct_a_starting_image_of_the_size(tier, tier.device_bytes(), tier.live_data_units())」，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/starting_image_construction.rs:70`；背景材料说最小档 36608，转述），不收 `--unit-area` 与 `--live-data-units`。第二轮探针副本上 2288MiB 档 P=4096 造池 198.777 秒，交错写每步从头八分之一 46.598 毫秒涨到末八分之一 179.602 毫秒（第二轮产物 `phase=interleaved_half` 那一行）；P 涨到 36608 约 9 倍，按每步随版本线性涨，单交错写一段就是几小时（推的），过任务服务四小时上限；E202 现状一节记的第一次几何核也是这样没跑完。
- 所以判「装置走得到、量不到」：缺两样——整理那一段跑完之后取走探针表、写进产物；一个能收窄档与 P 的入口。

### 补的（只在副本里）与命令

副本 `/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/copy`：`python3 scripts/hosts/minimal-copy.py <仓根> <副本> --with-test-inputs` 从主工作区建，`patch -p1` 依次打第二轮 diff 与模型目录的 `probe/compaction-probe-on-the-cpu-arm.diff`。后者做两件事：
- `construction_timing_command` 每个 P 造完、打完造池那几行之后，调新加的 `probe_the_compaction_publishes_on_the_cpu_arm`：先把造池剩在表里的取走丢掉；几何没成形打一行 `geometry_formed=false ran=no` 就不跑；成形的照 `geometry-check` 那套阶梯（用户改动 24、48、96，批长 4，够 64 批就停）在 CPU 臂上交替跑，每一级从同一份起步镜像拷池，跑完取走探针表，打逐次发布行（`phase=compaction_cpu_arm_user_changes_<次数>`）、分布行与一行 `name=compaction_probe`（`outcome`、`batches`、`units_relocated`、`publishes_counted`、`seconds`）。
- 探针多数三项（按叶分块，见格二）。
P 取 1024、2048、4096：第二轮那一趟 P=4096 的造池产物里 `written_segments=128 written_segments_with_half_live=128`，碎片段成形，起步镜像能拿来跑；取三个 P 是为了 C4（d 随不随 N 涨）。几何成不成形以产物的 `geometry_formed=` 为准，我没跑过。
这两份改动我没编过（子 agent 不交任务）；`rustfmt --check` 两份文件都过（只判得了能解析）。

### 跑前写死的翻面线（在副本跑出任何一行之前写下）

判在带搬迁的发布（`relocated_units > 0`）上；同一段里的用户改动与签、删意图的发布另报不判。

| 读数 | 翻面线 | 翻成什么 |
|---|---|---|
| C1 分配记录，量到的最大 P，`copy64_over_n` 中位 | ≥ 0.25 | 「甲在整理发布上也值得做（分配记录表）」→「不值得」 |
| C1 按叶分块 同上，拷贝比 =（碰到的叶里的记录 + 有记录的叶数）÷ N | ≥ 0.25 | 同上，块取分配记录树的叶 |
| C2 中央映射条目，同上 | ≥ 0.25 | 同上，映射那一张 |
| C3 单元，同上 | ≥ 0.25 | 同上，单元那一张 |
| C4 每张表，最大 P 对最小 P：d 中位的倍数 ÷ N 中位的倍数 | ≥ 0.5 | 「整理发布的 d 与全版无关」→「随全版涨」 |
| 作废 | 产物行数与 `publishes_counted` 有一处不等，或一个 P 都没量到带搬迁的发布 | 这一趟作废、退 1，不判 |

结果反过来我接受的写法：C1–C4 都不翻，第一轮判决「整理发布没量」那一条的射程收到「E202 造池加 CPU 臂整理、一个档、三个 P」；有一条翻，甲的分步终稿里那张表要另起一步，先在整理那一支换表示，或不换。建这份探针时我的倾向：C1–C3 不翻（一批搬 4 个单元，d 是个位到几十）、C4 不翻；按叶分块 C1 不确定。倾向不进判定。
取样的限度：一个档（2288MiB）、三个小 P、批长只取主取样点 4、只 CPU 臂；E202 正式跑的 P 与 GPU 臂没量。整理批长产品默认 16（`compaction_plan.rs` 的 `COMPACTION_BATCH_UNITS_BY_DEFAULT`），装置主取样点是「const BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT: usize = 4;」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:287`）。

## 格四：第二轮判决第二节表里的数，在任务 19957 的日志里逐字找

草稿脚本 `scratch/verdict_numbers.py` 读判决第 26–34 行的表（每行五个数：d 中位、最小、最大，N 中位，d ÷ N 中位）对日志的 `name=distribution` 行逐字段比，再把判决里反引号括着的 `键=值` 与三段发布次数在日志里逐字找。输出（草稿目录 `verdict-numbers.txt`）末行原样：

```
checked=58 mismatched=0
```

日志原件与草稿目录里那份拷贝（第 5 条命令读拷贝）的 `sha256sum` 原样：

```
b7243ae85d1448e56d33ef377051e03097f137f98b80989afb6be4356ae3bf50  /home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-19957.attempt-1.log
b7243ae85d1448e56d33ef377051e03097f137f98b80989afb6be4356ae3bf50  /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/task-19957-summary-log-copy.log
```

45 个表格数、10 个 `键=值`（`copy64_over_n_median=0.025178`、`carried_under_a_new_role_median=0`、`rewritten_roles_median=18`、`repeated_roles_max=0`、`order_flips_among_common_roles_max=0`、`descents_within_a_role_kind_max=0`、`copy64_over_n_median=0.074861`、`leftover_over_records_max=0.861538`、`reclaimed_placements_max=51790`、`cluster_segments_max=450`）、三段 `apparatus_publishes=2048/4224/260 holds=yes` 都找到。没打中。
日志里核不出、要回产物核的只有一样：0.861538 落在哪一段（格一末节）。日志不报，判决与背景材料的上下文把它读成交错写段，产物说是「放开垫片的槽」那一段。

## 另攻一：未定项清单还漏了什么

前两轮留下的未定项：容器自己写还是引包、分步先换哪张表、已回收集为什么只涨不缩、乙强与丙强零轮、整理发布与「一次改很多」的发布没量。逐样看派发点名的四处：

1. 验证包的内存池与崩溃引擎读版本的地方：没打中。`crates/singlefs-verification/src/memory_pool.rs`、`crash.rs`、`crash_facts.rs`、`crash_judge_tables.rs`、`crash_identity.rs`、`crash_verify_gpu.rs` 里的 `.units` 都是录制流的写表或崩溃事实表（例：`crates/singlefs-verification/src/crash_facts.rs:825` 遍历的是事实表的 `units`），不读 `TransactionOutput`；`crash_amplification.rs` 的命中是 `refusal.units()`（判器单元）。崩溃引擎不读版本，与第一轮判决第一节第 4 条（攻方读、主 agent 没核）一致；我按文件 grep 核过这几份（`grep -nP '\.units\b'` 与 `grep -n TransactionOutput`）。
2. 但验证包的库代码（`src/` 下、不在 `src/bin/`）读版本的那几份，第二轮的读者分类表没进：它的范围是 `crates/*/tests` 与 `crates/*/src/bin`。现数（`grep -rnP '\.allocation_records\b|\.units\b' crates/*/src --include=*.rs`，去掉 `src/bin/`、`singlefs-core`、`singlefs-unit-graph`，再去掉上一条那几份不读版本的）：`history.rs` 22 行 `.allocation_records`、`model_comparison.rs` 12 行 `.units` 与 5 行 `.allocation_records`、`on_device_modes.rs` 1 行、`crash_injection.rs` 读版本 8 处（`TransactionOutput`）。里面有甲下要改写法的：「previous_records.clone_from(&output.allocation_records);」（`crates/singlefs-verification/src/history.rs:7552`）、「let records_on_disk_before = current.allocation_records.clone();」（`crates/singlefs-verification/src/history.rs:8387`）、「units: output.units.clone(),」（`crates/singlefs-verification/src/model_comparison.rs:1038`）（把版本的 `Vec` 拷进自己的结构）。甲下这几处要么跟着换类型，要么在那里 `to_vec` 照拷（代价不变、只在验证包）。打中的是改动面的清单少了这一块，不是新的难点；`crates/singlefs-checker/src/walk.rs` 的命中是 checker 自己的结构（checker 只依赖 `singlefs-format`，第一轮判决已核）。
3. 读写并发时旧版的释放：推不出、不打中。甲下旧版是值，最后一个持有者放手时只放它独有的块，代价与 d 同阶；放在哪个线程上、读者持着旧版时常驻多涨多少，归里程碑六（「读写并发都归里程碑六」（`.claude/kb/milestone/04-fourth-txn.md:5`）），今天串行、没有读者线程。今天跨调用持着版本的：一次用户改动交回的那几版（核心层 `UserChangePublished` 的 `versions_published`，验证包 `crates/singlefs-verification/src/history.rs:2664` 那一处读它）、冻结发布那一份；甲下它们共享没改的块，常驻只降不涨（推的）。
4. 甲之后建树函数还整棵遍历：打中一处、两批之后仍在。版本里的分配记录每次发布从分配器整份拷出再整排：「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」（`crates/singlefs-core/src/transaction.rs:16232`）、「allocation_records.sort_by_key(AllocationRecord::sort_key);」（`crates/singlefs-core/src/transaction.rs:16233`）；补丁里这两行没有改动（补丁里搜不到删它们的行），是 O(R log R)。甲只有在分配器那张表本身按（设备，槽号）排、版本直接共享它时才去掉它——这一条把「分配器的记录表与版本的分配记录是甲的同一步」（第二轮攻方三·1）钉到了具体行上，并且和下面「并得住」那一条撞在一起。紧挨着的那条恒开断言（`:16245` 起）比整表，补丁改成只比改过的叶，归两批的代码轮判（第二轮判决已记）。

## 另攻二：「甲与不动形态两批并得住」在哪一样上不成立

补丁 `~/.local/share/singlefs/work/e202-pool-construction-speedup-writer-1/patch/crates.patch` 只读。三处：

1. 记录表的主次序反了。补丁把 `PoolAllocator` 的「-    records: Vec<AllocationRecord>,」（补丁第 1273 行）换成「+    records: AllocationRecordsIndexedBySlot,」（补丁第 1277 行），那个结构的主表是「+    records_in_recording_order: Vec<AllocationRecord>,」（补丁第 742 行），按键的次序只是附带的「+    recording_sequence_of_each_slot: BTreeMap<(DeviceIdentity, SlotNumber), u64>,」（补丁第 746 行），另配「+    recording_sequence_of_each_position: Vec<u64>,」（补丁第 744 行）做二分。补丁结构的文档注释（第 734 行起）写明 `PoolAllocator::records` 交出的就是记下的先后，「删一条不动别的记录的先后」（转述）。甲要版本与分配器共享同一张按（设备，槽号）排的持久表（上面另攻一第 4 条），主次序得是键；补丁花力气保住的「记下的先后」在甲下要么丢掉（先证没有读者依赖它：补丁自己的 `slots_of_the_records_intersecting` 还特地按表里的先后重排交回），要么在持久表里再挂一个序号。哪一种，补丁那三份并行的表都不是甲的一部分，要整份重写，不是「正好是甲的一部分」。
2. 补丁加的索引进发布前那份克隆。补丁不碰发布前的整份克隆（主工作区 `crates/singlefs-core/src/transaction.rs:13213`「let allocator_before_this_publish = allocator.clone();」；补丁里搜 `allocator_before_this_publish` 0 处）。`AllocationRecordsIndexedBySlot` 派生 `Clone`，两批之后那份克隆在今天的记录表与已回收集之外，多拷一份与记录同样多项的 `BTreeMap`、一份同样长的 `Vec<u64>`，与「+    released_and_not_reclaimed: BTreeSet<(CheckpointTxg, DeviceIdentity, SlotNumber)>,」（补丁第 751 行）。第二轮的 T5 是在没打补丁的主工作区副本上量的（第二轮报告二·1），两批之后同一格的分子至少再加一个记录条数：按第二轮读数推，比值从 0.86 到 1.86 以上（推的，没在打了补丁的树上量）。`released_and_not_reclaimed` 只装 defer 窗口里的，不随池涨（推的）。
3. 只读的预判路径上整份克隆仍在：补丁在合发几次数据改动那条预判入口里写「+    let mut allocator_for_the_rehearsals = allocator.clone();」（补丁第 6365 行），因为 `rehearse_and_undo` 要可改的分配器；主工作区那里是在判定函数里面每轮各拷一份（`transaction.rs:14076`「let mut rehearsal = allocator.clone();」），补丁收成调用处一份。持久化之后这一份是常数，撤销账整个多余——这一半第二轮攻方三·1 已报，不重复；新的是：撤销账要 `&mut`，把只读路径逼成先整份拷，两条线先合撤销账、后做持久，这一处要改两遍。

合入次序（推的）：补丁里与表示无关的那几样（中央映射树只按改到的 key 规划、按叶分组带着走的缓存）先合；`AllocationRecordsIndexedBySlot` 与撤销账不合，等甲的分配记录那一步直接做成按键的持久表、演练回到「拷一份、改、丢」。这与第二轮攻方「先合撤销账、后做持久，那一套要整删」同向，多出的是记录表那一份也在整删之列。

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 改法在打中的格上还中不中 |
|---|---|---|---|---|
| 已回收集「只涨不缩」不成立、上界是槽数 | 不分辨：三条候选都不碰它（第二轮判决已记「不分辨臂」） | 看得到：项数就在 `PoolAllocator` 上，探针逐次打了 | 判据格「已回收集」：「是什么、为什么涨、该不该有界、归这一题还是另一个缺陷」 | 第二轮「换持久或不随发布涨」：换持久起作用但多一样容器；「随门槛清掉」不起作用（断言与复活分支要它）；我提的「删掉、按记录与位图判」起作用（推的） |
| T5 的 0.861538 归错段 | 不分辨，也不改翻面 | 看得到：产物逐行有 `phase=` | 判据格「数能不能核」 | 不涉改法；判决引它时写段名与「全部段」 |
| 拷贝式子与甲的块对不上（按叶分块没量） | 分辨：只碰甲（乙、丙没有块） | 看得到：探针能数碰到的叶 | 判据格「d 的定义对不对得上」：「探针数的与甲真要复制的是不是一回事」 | 补的计数能判；块长 16/256 那两个改法不翻，对这一格无话可说 |
| 装置走得到整理发布、但量不到 | 不分辨 | 看得到：`geometry-check` 不打探针行是读码可见的 | 判据格「整理发布上的 d」：「给一条能量的命令」 | 副本改动起作用（没编过）；照原样跑 `geometry-check` 不起作用 |
| 补丁的记录表主次序与甲相反、索引进克隆 | 分辨：只碰甲（乙、丙不共享分配器的表） | 看得到：补丁里的类型定义 | 正文「已定」第 1 条：「这几样哪些白做了、哪些要重写、哪些正好是甲的一部分」 | 「先合补丁的记录表再换持久」不起作用（要重写）；「补丁里只合与表示无关的那几样」起作用（推的） |
| 改动面少了验证包库代码那几份 | 分辨：只碰甲 | 看得到 | 正文「未定项清单的完整性」第 4 条（验证包读版本的地方） | 不涉改法，进分步终稿的改动面 |

## 我提的改法（只在我的读码与草稿脚本上推过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 删掉已回收集，「回收过没有」改成看记录 `is_released` 与位图；删之前在测试构建里与集合并存、逐项比 | 已回收集进发布前克隆（T5 的大头） | 推的，没实现没跑 |
| T5 判决引用写段名：交错写段最大 0.791809，全部段最大 0.861538 在「放开垫片的槽」段 | 数能不能核 | 量的·草稿，待主 agent 跑第 1 条 |
| 按分配记录树的叶分块量 T1（探针三项计数、`block=allocation_record_tree_leaf` 那一行） | d 的定义 | 推的，待主 agent 跑第 2–4 条 |
| 副本里 `construction-timing` 造完之后 CPU 臂交替跑、取走探针表 | 整理发布上的 d | 推的，待主 agent 跑第 2–4 条（没编过） |
| 不动形态两批里 `AllocationRecordsIndexedBySlot` 与撤销账不合，等甲的分配记录那一步 | 并得住 | 推的 |
| 分步终稿的改动面加验证包 `history.rs`、`model_comparison.rs`、`on_device_modes.rs`、`crash_injection.rs` 读版本的那几处 | 未定项完整性 | 读码核过行数，改法推的 |

## 要主 agent 跑的命令

1. 第二轮产物上按块长重算（判决要引格一、格二的数时用）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus -- python3 /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/tools/recompute_copy_ratio_by_block_size.py /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r1-model-rerun/publish-change-probe-2288MiB-P4096.out`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus`（草稿目录）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里「✓ 块长 64、256 两条路的中位逐格对上」、六行 `name=block_size_flip … block=16|64|256` 都是 `flipped=no`、按叶那一行 `flipped=not_judged`（第二轮产物没有叶计数）、`name=leftover_over_records_overall max=0.861538 phase=releasing_the_spacer_slots txg=6574`、`name=reclaimed_set_steps_overall shrank=1849 publishes=6573`。`exit=1` 看 `✗` 行。
2. 副本里编、跑装置（整理那一段与按叶分块）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/copy -- cargo run --release -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- construction-timing --unit-area 2288MiB --live-data-units 1024,2048,4096 --chunks shared --product /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/compaction-probe-2288MiB-P1024-2048-4096.out`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/copy`（副本，`.deliverables` 写成「交接」）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；产物里三行 `name=construction_timing`、每个 P 至少一行 `name=compaction_probe`（`geometry_formed=true ran=yes outcome=ok` 才算量到）、`phase=compaction_cpu_arm_user_changes_` 开头的探针行带 `records_leaves_touched=`。两份改动我没编过：`exit=101` 多半是编译错，`task-server/task log <号> --all` 里 `error[` 指到 `publish_change_probe.rs` 的 `leaves_of_the_records_touched` 或装置的 `probe_the_compaction_publishes_on_the_cpu_arm`，新派攻方腿承接修。
3. 第 2 条那份产物的两份汇总（等第 2 条退 0 再交，各一条）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus -- python3 /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/tools/summarize_compaction_probe.py /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/compaction-probe-2288MiB-P1024-2048-4096.out`
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus -- python3 /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/tools/recompute_copy_ratio_by_block_size.py /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/compaction-probe-2288MiB-P1024-2048-4096.out`
   - 在哪起：草稿目录（同上）。
   - 看哪几行：整理那一条 `exit=0` 且「✓ 完整性闸过、量到了带搬迁的发布」算有效，读 `name=compaction_run`、`kind=relocating` 的 `name=compaction_distribution` 与 `name=compaction_flip` 各行，照格三的表判；`exit=1` 看 `✗` 行（作废）。块长那一条读三个 P 的 `name=block_size_flip`，`block=allocation_record_tree_leaf` 那一行就是按叶分块的 T1；`live_data_units=4096` 的块长 64 那两行应与第二轮的 0.025178、0.074861 同阶（同一套造法再跑一次，不要求逐字相同）。
4. 从干净运行目录整份复跑（判决要引第 2、3 条的数时用）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus-run /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r3-opus-model`
   - 在哪起：草稿目录。第二个参数是冻结副本：给仓根时复跑的是交任务那一刻的主工作区；判决要对开工那一刻，换成 `snapshot-tree.py` 倒推出的树。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=rerun_probe_exit block_size_exit=0 compaction_exit=0`、`name=apparatus_seconds seconds=<秒>`、两份汇总的成功句与翻面行。非 0 时运行目录里 `patch.log`（两份 diff 打不上：主工作区那几份文件在我建副本之后被改过）、`apparatus.log`、`block-size.out`、`compaction.out` 分别看。运行目录用完由主 agent 删。
5. 第二轮判决数字的逐字核对（格四；可选，主 agent 也可以自己逐个 grep）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/scratch/verdict_numbers.py /home/fy5090/code/singlefs/research/prompts/m4-publish-without-whole-version-copy-design-r2-main-verification.md /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/task-19957-summary-log-copy.log`
   - 在哪起：草稿目录。日志原件在服务状态目录下、不在共享根，我拷了一份进草稿目录（两份 sha256 相同：`sha256sum` 见格四），命令读拷贝。
   - 看哪几行：末行 `checked=58 mismatched=0`。

## 没打中的形状

- 已回收集有没有泄漏：在 `allocator.rs` 里找「记录删了、集合里还留着」与「集合里有、记录不是已释放」的路，按 `records.retain`、`self.records =`、`reclaimed.insert/remove/clear` 全数过一遍；只有相交让位、复活、重建三处，每处成对改。没打中（只核到这份文件）。
- 已回收集会不会无界：项数 ⊆ 记录键，记录每盘互不相交，上界两盘槽数；产物里它 1849 次缩过。没打中。
- d 漏数：单元的值罩全了 `PublishedUnit` 三个字段；`Arc` 地址比相等，上一版在比的那一刻还活着，地址不会被新分配复用（没有 ABA）；映射表往大里数。没找到往小里数的口子。
- 块长：16（只有界）、64、256 三种，交错写段 T1、T4 都不翻；顺序写段的单元与映射在 256 上过线，那一段不判。
- 判决表的数：58 处逐字对上，0 处对不上。
- 验证包内存池与崩溃引擎：`memory_pool.rs`、`crash*.rs` 读的是写表与事实表，不读版本。
- 冻结重发拿哪一份：不攻（正推那一格，第二轮攻方已报「挪动不拷贝」）。

## 撤回了什么

- 第二轮攻方（上一条同立场腿）的改法「分配器的已回收集……一并换持久」（第二轮判决第三节第 3 条记的零轮改法里已回收集那一半）：撤，改成「删掉这份集合、按记录与位图判」。理由：它是记录表的子集索引，记录表与位图已在甲的持久化之列，再给它单配一个持久集合是多一样容器、买不到东西。聚簇段集与被抛弃根那两样不在此列，照旧。
- 第二轮攻方 T5 的跑前登记写「放开垫片的槽」那一段不判，汇总脚本却在全部段上取最大：撤回「全部段」的射程，T5 只判造池三段时最大是 0.845898（删垫片段），交错写段 0.791809；翻面结论不变。
- 第二轮攻方三·1 与第二轮判决里「已回收集随发布次数涨」的说法：撤回「随发布次数」，它随记录表里的死记录涨，上界是槽数（格一）。
- 背景材料与第二轮判决的「已回收集为什么只涨不缩」这一问：问题本身的前提不成立（1849 次缩），建议判决改问「它为什么与记录表同阶」，答案是 D3 已定项 7 留着死记录。

## 这条腿自己的限度

- 副本改动与两份汇总脚本都没编、没在真产物上跑（子 agent 不交任务）；格二按叶分块与格三全部是推的，读数要等第 2–4 条。汇总脚本只在我造的合成产物上跑过，证过两种红。
- 「量的·草稿」那些数是我在草稿目录对第二轮产物跑自己脚本的拷贝读出的；第二轮产物本身是副本装置跑的，不是入库装置。
- 块长 16 只有上下界；块的模型仍是「一层块指针」的二层表，没给多层树建臂（推的：多层树更低）。
- 已回收集可由「记录已释放 ∧ 位图上空」判出来，是读码推的，没写断言、没跑。
- 补丁的结论只读了 `allocator.rs` 那一段的类型与注释、`transaction.rs` 那一段搜了几处，没逐行读完 8945 行；「两批之后 T5 至少 1.86」是按类型推的，没在打了补丁的树上量。
- 只看了一个档（2288MiB）；整理那一段只取三个小 P、批长 4、CPU 臂；GPU 臂与 E202 正式 P 没量。
- 已回收集上界那一句只在造池（池没写满一圈）的产物上看过，绕满一圈之后停在哪是推的。

## 没做什么

- 不判正推、辩方那几格（分步终稿、冻结重发、内存与旧版释放的估算）；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没改主工作区的 `crates/`；补丁只读；没读禁读清单里的文件。
- 没删 /home/fy5090/.local/share/singlefs/work/m4-publish-without-whole-version-copy-design-r3-opus/copy：主 agent 要在上面跑命令（第 2 条），已写进草稿目录 `.deliverables`，种类「交接」，用完由主 agent 删（删前 `du -sh` 是 57M，没有编译目录）。
- 草稿目录里另留：两份打改动之前的原件备份（`e202-after-the-probe-diff.rs.orig`、`e202-after-the-compaction-probe.rs.orig`、`publish_change_probe.rs.orig`）、副本改动的 diff、任务 19957 日志的拷贝、`scratch/` 下的草稿脚本与合成产物、各次草稿运行的输出；都已写进 `.deliverables` 或是普通小文件，没有编译目录与库拷贝。
