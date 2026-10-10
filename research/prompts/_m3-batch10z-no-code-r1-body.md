# 根环实现第 10z 批第十四、十五块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 H1 H2 Z1 Z6 Z8 T1 T2 T3 T4 T5 X4 X5 X18 R C P Q F -->

## 一、题面

这一轮攻两块，碰的文件不交：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第十四块（core） | 第十二、十三块代码轮判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3（H2 双扣）与 K2、第三节第十四块：C 那一圈（宽护、回退复活、丙并中间各支）在挂载那一刻与抬 F 重算时只隔离分配位没置着的槽，分配位置着的（现行账仍分配、已释放而还在 defer 窗口里的）不再算进第九项，等回收时补隔离；抬 F 那一串的回收新接 `reclaim_released_up_to_isolating_what_the_isolation_still_protects`；新访问器 `slots_counted_both_as_allocated_and_as_isolated`；K2 读盘一层两处与墓碑片读不出的用例 | `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第十五块（GPU） | 同一份判决第二节 K3-3、K4、K5-2：借之前那一步看池里每张卡，留着的设备丢了就照重起那一套记账，没丢且列出来才 poll；正推 T1–T5 收进库单测；叫停两层换成行为用例，另四层与卡池接线保留扫源码、改整行比，拦不住的写法写在用例旁；七种骗过扫源码的写法做成 9 行变异 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |

条款与用户原话：
- `.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项（「只除去现行账里仍分配着的槽（那些算在「已分配」里，不在这里再算一次；这次挂载里释放、回收时分配器照补隔离）」）；`.claude/kb/decisions/16-发布语义.md` 已定项 1 第 29 行（回退候选集保留最近 4 个可退到的不同状态）与第 37 行「根槽这一次读坏」那一行（H2 丙、「丙继续补洞」、tri）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 390 行；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；欠账 `.claude/kb/checks-owed.md` C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放）。
- 用户 2026-10-08 定 tri；同日定 384 槽盘去掉双扣之后仍卡死那一形「给写掉 R 那一槽开路」，查清代价后再定「显式推环命令加带出路的错」（实现在第十六块，不在这一轮被判范围）。
- G2 岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` G2 行；用户原话「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；GPU 总口径 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

改动全文：`research/prompts/_m3-batch10z-no-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-n/patch/report.md`（第十四块，第六节 H1 重量表、第七节停下交主 agent 的八条）、`…/batch10z-o/patch/report.md`（第十五块）。第十四块的量法源码拷在 `~/.local/share/singlefs/work/m3-r1/batch10z-n/sweep/`。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/`，取在两块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-no-code-r1-snapshot/crates-sha256.txt`。
- 第十四块关键位置（冻结副本 `crates/singlefs-core/src/allocator.rs`）：`DeviceFreeMap::isolate_the_slots_not_allocated` 第 598 行、`isolate_slot_at_index` 第 613 行、`slots_counted_both_as_allocated_and_as_isolated` 第 625 行、`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects` 第 2087 行、`isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records` 第 2158 行、`isolate_reclaimed_slots_the_isolation_still_protects` 第 2435 行；`crates/singlefs-core/src/mount.rs` 第 1649–1656 行、第 4719–4730 行；新用例在测试文件第 2894–3088 行一带。
- 第十五块关键位置：`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1448 行 `take_in_the_devices_kept_before_booking`。
- 合后验证：
  - 第十五块（`~/.local/share/singlefs/work/m3-r1/merge-10z-o/summary.txt`）：merge-and-verify rc=0；mutation-tables 0 行筛不到，static-check `CHECKED 3480 322 10`；checker 档带 gpu lib 与 12 份挂 gpu 的测试全过、`NOT_RUN_NO_GPU_ADAPTER` 0 行；合后副本上 checker 档 11 行 prove-red 全抓到。
  - 第十四块（`~/.local/share/singlefs/work/m3-r1/merge-10z-n/summary.txt`）：merge-and-verify rc=0；mutation-tables 0 行筛不到，static-check `CHECKED 3490 322 10`；e158 `95 passed`；E171 装置 sha 前 12 位 ab35105f861d；harness 档慢档三份（24 / 23 / 16 passed）与第九项相关 10 份 `--include-ignored` 全绿；checker 档快档 61 份结果 0 失败。
- 实现员自定、被攻过零轮的：
  - 第十四块：跳过按槽判（落点里分配位置着的那几槽不隔离、其余照隔离）；补隔离两条路（挂载内记根、抬 F 那一串）；抬 F 那一条只在「R 的根槽读不出（C694 的 C）」那一形上是唯一保护；`referenced_by_the_current_account` 按起点槽豁免照旧留着；
  - 第十四块量到的（副本）：1536、6144 两档每一形 0 拒；攻方那一形 2242 → 252 槽、40 拒 → 0；384 槽盘仍有 22 形写全被拒（第十六块的事，不在这一轮被判范围）；`racct,oldacct` 三形改前改后都在挂载时拒（`FloorRaiseFailedAfterTheMountsPublishes`，原因没查）；
  - 第十五块：K3-3 改法；丢了的设备上活着的借据照整笔记到持有者丢掉它；`check_selected_blocks` 没有行为用例。
- 已知没做、不在这一轮被判范围的：384 槽盘卡死那一形（第十六块）；被抛弃根那一圈与根槽读坏那一圈遇到 defer 槽也双扣（第十六块）；只供测试的复用窗口置 0 那条路回收时不补隔离；C762；C554。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态、一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口；G2 乙本身；H2 丙本身；tri 的判法本身与它在「不知道」那一支上多护的那一截；C762。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第十四块 双扣改法做到没有

1. 挂载那一刻跳过分配位置着的槽之后，这些槽在之后被释放、回收时，是不是每一条回收路都补得上隔离？有没有一条回收路（挂载内记根、抬 F 那一串、正常卸载之前、别的回收入口）漏补，让被抛弃根 R 还在环里时那几个槽被发出去、I-7.4 红？
2. 按槽跳过：一个落点一部分槽分配位置着、一部分没置时，补隔离补全了没有？`referenced_by_the_current_account` 按起点槽豁免与新的按槽跳过叠在一起，有没有一形该隔离的被豁免？
3. 准入与第九项：改完之后第九项与「已分配」还有没有一形重复扣，或者反过来少扣一截（该算进第九项的被当成已分配、而它其实已经不在已分配里）？

### K2　第十四块 判别力

新用例与 10 行变异、2 行整行替换，能不能拦住改回去？`slots_counted_both_as_allocated_and_as_isolated` 这一访问器本身会不会算错让用例照绿？

### K3　第十五块 借之前那一步看每张卡

1. 看池里每张卡之后，有没有一形多给（账给的量大于卡上真能再给的）？丢了的设备旧克隆还占着卡、两线程同时借同一张卡、重起正在进行时呢？
2. 有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住（每次借都看每张卡、多 poll 一次）？

### K4　第十五块 判别力

T1–T5、叫停两层的行为用例、改成整行比的扫源码用例与 9 行变异，能不能拦住改回去？保留扫源码的那几处，用例旁写的「拦不住的写法」写全了没有？

### K5　只防 OOM、不压并发两头

1. 有没有一形账给的量大于此刻真能用的？
2. 除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或并发被压住？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 24G。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
