# 根环实现第 10z 批第十六至十九块：代码轮第一轮正文（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 m7 m8 R C P Q F S -->

## 一、题面

这一轮攻四块：

| 块 | 照什么改 | 落点 |
|---|---|---|
| 第十六块（core） | 用户 2026-10-08 弹窗定「显式推环命令加带出路的错」（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md`）：准入被拒、推抬 F 停在上限、而 C 那一圈挡着时，`NoSpaceAfterRaisingTheFloor` 的新字段 `wide_protection_at_the_ceiling` 报 `StuckBehindTheWideProtection`（R 的 txg、每盘挡着的槽数、要推几次、`ways_out()` 两条出路）；新入口 `MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states` 先在分配器拷贝上预演、再发 F 不变的空发布直到 R 那一槽被写掉，取不到落点时在任何写之前拒；另把被抛弃根那一圈与根槽读坏那一圈改成只隔离分配位没置着的槽、回收时照根环表补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第三节） | `crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` |
| 第十七块（core 测试） | 同一份判决 K2：访问器 `slots_counted_both_as_allocated_and_as_isolated` 的阳性对照；卸载那一支「当场放开那一截」（m7）补隔离的用例 | `allocator.rs` 测试模块、同一份 harness 测试 |
| 第十八块（GPU 测试） | 同一份判决 K4：卡池模块拆词拒同名模块与 `extern crate` 等；叫停各层整句比加顶层一句；`GpuCard::device()`、`device_condition()` 方法体整个钉住 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 的测试模块 |
| 第十九块（core 测试） | 卸载那一支「扣到生效那一截」（m8）补隔离的用例 | 同一份 harness 测试 |

条款与用户原话：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 29 行（回退候选集保留最近 4 个可退到的不同状态）与第 37 行「根槽这一次读坏」那一行（现行写法已写回）；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第 27 行第九项；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 390 行；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；欠账 `.claude/kb/checks-owed.md` C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）、C762（抬 F 给根环表没记着的被抛弃根补的隔离一直不放）。
- 用户 2026-10-08 弹窗原话「显式推环命令加带出路的错」：不自动推环；推环只在用户显式调用时、调用即确认回退候选集掉到少于 4 个不同状态。

改动全文：`research/prompts/_m3-batch10z-pqrs-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-p/patch/report.md`（第十六块，第二节改到的函数与行号、「要你定或知道的」五条）、`…/batch10z-q/patch/report.md`、`…/batch10z-r/patch/report.md`、`…/batch10z-s/patch/report.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-09）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/`，取在四块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-pqrs-code-r1-snapshot/crates-sha256.txt`。
- 第十六块关键位置（主工作区合后）：`mounted_session.rs` 第 171 行字段、第 196 行 `StuckBehindTheWideProtection`、第 211 行 `ways_out`、第 285 行 `wide_protection_at_the_ceiling`、第 331 行 `RootRingPushRefused`、第 690 行推环入口、第 876 行填字段；`mount.rs` 第 1558 行 `isolate_slots_referenced_only_by_abandoned_roots`、第 5915 行 `parameters_of_the_pool_agreeing_with_the_caller_inputs`；`allocator.rs` 第 2237、2254 行。冻结副本行号以快照为准。
- 合后验证：
  - 第十六块（`~/.local/share/singlefs/work/m3-r1/merge-10z-p/summary.txt`）：merge-and-verify rc=0；mutation-tables 0 行筛不到，static-check `CHECKED 3508 322 10`；e158 `95 passed`；E171 装置 sha 前 12 位 ab35105f861d；harness 档慢档 34 / 23 / 16 passed、第九项相关 10 份全绿；checker 档快档 61 份 0 失败。
  - 第十八块（`…/merge-10z-r/summary.txt`）：merge-and-verify rc=0；static-check `CHECKED 3517 322 10`；带 gpu 的 lib 与 12 份上卡测试全过；合后副本上 checker 档 9 行 prove-red 全抓到。
  - 第十七、十九块（`…/merge-10z-qs/summary.txt`）：见那份汇总。
- 实现员自定、被攻过零轮的：
  - 第十六块：「被宽护卡住」做成字段不做成员（e158 第 19119–19136 行、e168 第 1724–1730 行对 `UserChangeRefused`、`FloorRaiseStop` 穷举 match）；按 C 那一圈整体判、分不出宽护与别的几份；推环入口收盘表走抬 F 那一串的核（`caller_inputs_agreeing_with_the_disk`）；被抛弃根那一圈按「回收时补不补得上」分两支；整行替换的 3 行旧变异（「步 4」「增补 2 收口第 27 行」「第10z批第九块 K3 loop-span1」）；
  - 第十六块已知留着的：抬 F 那一串读盘得来、根环表没记着的被抛弃根回收时补不上隔离、照旧整段隔离与双扣（C762）；推环入口「收盘表被拒」「真发时块设备错」两支没有用例；
  - 第十八块：卡池拆词与叫停花括号层数各写一份。
- 已知没做、不在这一轮被判范围的：C762、C554；384 槽盘之外的几何没在推环入口上量；新用例不枚举崩溃状态。
- 用户已定、报出来不算打中的：tri 的判法本身与它在「不知道」那一支上多护的那一截；显式推环会让回退候选集掉到少于 4 个不同状态（用户点头的代价）；两进程同读一份空闲的竞态；G2 乙。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第十六块 带出路的错

1. 报 `StuckBehindTheWideProtection` 的判据：有没有一形卡在宽护上却报成别的（`NotWhatStopsTheAdmission`），或不是宽护卡住却报成它？挡着的槽数、R 的 txg、要推几次算得对不对？
2. 两条出路：照抄出路走，是不是都走得通？卸载重挂那条在 R 的账再挂时仍读不出时会怎样，出路的话有没有误导？

### K2　第十六块 推环入口

1. 预演与真发：预演说推得到、真发半路失败（块设备错、空间被别的抢走）时，盘、分配器、会话是不是一致？崩溃落在推环中间任一次空发布时，恢复与再挂对不对、I-7.4 / I-4.8 照旧不红？
2. 推环会不会提前盖掉别的被抛弃根或回退候选，让别的隔离提前解开、I-7.4 红？R 不是环里最新的根、环里另有被抛弃根、C 有多槽时呢？
3. 推完之后：R 那一槽写掉、宽护解开、准入恢复；有没有一形推完仍卡（推不动、推错槽、F 被抬到不该到的地方）？

### K3　第十六块 被抛弃根那一圈与根槽读坏那一圈只隔离分配位没置着的槽

1. 回收时补得上那一支：每一条回收路（挂载内记根、抬 F 那一串、卸载那一格两截）是不是都补上了？有没有一形 R 还在环里时那几槽被发出去？
2. 补不上那一支（照旧整段隔离）：判据对不对？有没有该照旧整段隔离的被跳过？

### K4　判别力

第十六到十九块新加的用例与变异（第十六块追加 13 行、替换 3 行；第十七块 2 行；第十八块 9 行；第十九块 1 行），能不能拦住改回去？推环入口没有用例的两支、第十八块扫源码用例仍拦不住的写法，各是哪几种？

### K5　只防 OOM、不压并发（第十八块只动测试，查有没有把产品行为带偏）

第十八块改的扫源码用例与新用例，会不会把合法的产品写法判红、逼出别扭的产品代码？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `research/scripts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试。
