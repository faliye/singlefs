# 背景材料：里程碑「第二个事务」步 4（管理员回退）与步 5（抬 F 与延迟重用）那批代码的三方对抗第一轮（2026-09-17）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 S11 S12 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 P1 P2 -->

## 一、被判的对象

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`（`mount_rollback`、`rollback_floor_ceiling`、`raise_rollback_floor`、`establish_instance`）、`crates/singlefs-core/src/allocator.rs`（`isolate`、`reclaim_released_up_to`、`record` 改写、`mark_reclaimed`、`format_time_tree_table`）、`crates/singlefs-core/src/recovery.rs`（`readable_roots`、`allocation_records_under_root`、`instance_table_of_root`、`rollback_high_water_of_root`、`effective_rollback_floor`、`replay_journal` 的回退行封顶与链首无锚点规则）、`crates/singlefs-core/src/transaction.rs`（`format_time_tree_table_to_release`）、`crates/singlefs-core/src/instance_table.rs`（从 `mount.rs` 挪出）、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`（I-3.1 / I-2.1 的候选集）、`crates/singlefs-harness/src/crash.rs`（记录核对器的 `record_lost`）；用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`（新，4 条）、`second_transaction_step_five_reuse.rs`（新，4 条）、`second_transaction_step_zero_layer0.rs`（固定脚本到 E）、`second_transaction_step_three_second_instance.rs`（链首无锚点那条）、第一个事务与步 1 / 步 3 用例里随 mkfs 树表释放改的钉数；变异表 `crates/mutations.tsv`（40 行：步 4 八条、步 5 六条、步 3 改法一条）。diff 与新文件全文在附录二 `research/prompts/_m2-step45-code-r1-diff.md`（基准：提交 5f9e449；步 0 / 步 3 那一半已由 `research/prompts/m2-step3-code-r1-main-verification.md` 判过，这一轮只判在它之后加的）。
落的是 `.claude/kb/milestone/02-second-txn.md` 步 4 与步 5，连带步 0（固定脚本到 E）；两步的「现状」段是主 agent 按代码写的，也在被判之列（X10）。
今天的状态（主 agent 现查，2026-09-17）：`check.sh` 绿；命名纪律绿；步 4 四条、步 5 四条验收用例绿；层 0 快的那条 108 个状态零违例，全量 2104413 个状态交门禁 54 号（还没跑）；门禁 59 号 40 条变异各红；52 号第二条流的登记与用例相符；E142（第一个事务的干跑） 第十次跑同步了 mkfs 树表释放与链首规则，产物只差 `defer_queue_per_device` 与 `mkfs_generation_records` 两处。
按 `.claude/rules/implementation-workflow.md`，这批代码提交前要过这一轮；上一轮（步 3）云端攻方打中的 X1 改法（链首无锚点时只认 txg + 1）被攻过零轮，并入这一轮的 X8。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- `crates/singlefs-core/src/mount.rs` `mount_rollback(parameters, devices, RollbackTarget { instance, checkpoint_txg }, ShadowLedger::{On, Off})`：`choose_superblock` → `choose_root`（最新根）→ `scan_journal` → `readable_roots`（根环全部可读根）→ 目标不在环里 `RollbackTargetNotInRing` → 候选集：最新根指着的实例表（`instance_table_of_root`）里若有目标实例的行且目标 txg > 那一行的 T ⇒ `RollbackTargetNotACandidate`；目标 txg < 最新根的 `rollback_floor` ⇒ 同错 → 目标根自己那条记录（同实例同 txg）读不出 ⇒ `RollbackRecordUnreadable` → `rebuild_version`（R_old 那一版）→ `JournalScanReport` 只记 `valid_records`（一条不施加）→ `first_txg_of_new_instance` = max(根环最高 txg, 记录最高 txg) + 1 → `rebuilt_allocator`（从 R_old 的分配记录重建，再按 `effective_rollback_floor` 回收）→ 影子账（`ShadowLedger::On`）：对根环里 (txg, 实例) > (R_old.txg, R_old.实例) 的每条可读根读 `allocation_records_under_root`，R_old 自己的记录里没有的 (盘, 槽) 逐个 `isolate_abandoned` → `establish_instance`：`acquire_instance`（取号 + 屏障）→ 行：被退回的实例 (r_old, T_old, 0, 回退位)、中间实例 (i, 0, 0) → `publish_version`（`InstanceTablePlan::Rewrite`，`counter` = R_old 那条记录的 jsn + 1，事务号 0、反向链 0，`rollback_floor` = F_生效）→ 暖机循环（同可写挂载）。
- `crates/singlefs-core/src/allocator.rs`：`DeviceFreeMap` 加 `isolated` 位图（`is_free` 与 `lowest_empty_segment` 绕开它；不进 `allocated_slots` / `deferred_slots` / `free_slots`）、`mark_reclaimed`（位图清、`allocated_slots -= span`、`deferred_slots -= span`、`free_slots += span`、runs 增量）；`PoolAllocator` 加 `reclaimed: BTreeSet<(盘, 槽)>`、`reclaim_released_up_to(floor)`（`is_released && generation <= floor` 的记录逐个回收、去重）、`record()` 在 `reclaimed` 里命中时改写那条记录（代、跨度、已释放位清）而不追加、`format_time_tree_table: Option<Placement>`（`mark_format_time_units(instance_table, tree_table)` 记下）。
- `crates/singlefs-core/src/transaction.rs` `publish_version`：`previous` 为 `None`（第一个文件版本）且这次重写树表 ⇒ 释放 mkfs 那片树表单元（`format_time_tree_table_to_release`）；其余照旧。
- `crates/singlefs-core/src/recovery.rs`：`effective_rollback_floor` = 各盘（按根的 txg 算区域再算盘）所带 F 最大值的最小值，没根的盘不算、一条根都没有时 0；`rollback_high_water_of_root`：所选根自己指着的实例表里有本实例的回退行 ⇒ Some(W)；`replay_journal(…, rollback_high_water)`：`high_water == 0 || record.transaction > high_water` 即停；链首：所选根自己那条记录读得出 ⇒ jsn + 1，读不出 ⇒ 只认 `checkpoint_txg = 根 txg + 1` 的那条。
- `crates/singlefs-core/src/mount.rs` `rollback_floor_ceiling`：有效根 = 可读 ∧ txg ≥ 今天的 F ∧ 按最新根实例表有效；每块盘上最新的有效根取 min；非空 = 环里有它自己那条记录且事务号 ≠ 0，按 txg 降序取第 4 个，不足 4 个取最旧的有效根；上限 = 两者的 min。`raise_rollback_floor(parameters, devices, allocator, current, new_floor)`：超上限 `RollbackFloorAboveCeiling`；先 `reclaim_released_up_to(new_floor)`，再连推 `rollback_floor = new_floor` 的空发布（`InstanceTablePlan::Carry`、文件照抄）直到每块盘上都落了一条、上限 `ROOT_RING_REGIONS`；返回 `RaisedFloor { ceiling, publishes, reclaimed }`。
- `crates/singlefs-checker/src/walk.rs`：最新根走完之后，别的根只在「按最新根实例表有效 ∧ txg ≥ 最新根 record_bytes[130..138] 的 F」时走（I-2.1 只在这些根上判，I-3.1 的并集也只含它们）。
- `crates/singlefs-harness/src/crash.rs` `check_records`：一份记录「丢了」= 不在盘上且没有更晚落在同一位置的写在盘上；`root_without_record` 要全部记录都丢了才判。
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`：`Script::{SecondVersionOnly, ThirdVersion, RollbackToFirstVersion, ReuseAfterRaisingFloor}`；到 E 的流 54 段（`…18,2,1,4,10,2,1,10,2,1,18,2,1,18,2,1,18,2,1,18,2,1,10,2,1,10,2,1,18,2,1,2`）、279 次写、闭式 2104413；版本表加 (3,9)、(3,10) 第一次内容、(3,11)–(3,14) 四版、(3,15)、(3,16) 第四版、(3,17) E；快的那条 108 个状态。
- `crates/` 里没有的（主 agent 现查 grep 零命中）：抬 F 的正常触发（准入不够）、I-7.4 / I-4.8 的 checker、实例表第二片的写者、fsync 接口、影子账的盘上字段（只住内存）。

## 二、代码做了哪些选择（每条都是可以被攻的推论）

| 编号 | 选择 | 压着的条款（原文在附录一） |
|---|---|---|
| S1 | 回退候选集：按最新根指着的实例表判（无 i 的行，或有行且 T ≤ Ti）∧ txg ≥ 最新根带的 F；目标根不在环里、不在候选集、自己那条记录读不出各报错 | D23（journal 的角色与格式） 已定项 14 回退段；D16（发布语义） 已定项 1 回退候选集那一行 |
| S2 | 回退之后新实例的第一条 jsn = R_old 自己那条记录 + 1（C340 取 P1）：被抛弃发布的记录槽被新实例的记录盖掉；记录核对器把「被后来的记录合法覆盖」与「记录流有洞」分开 | C340（回退之后记录链从哪条之后接没有定义）；D23（journal 的角色与格式） 已定项 14 第 3 条「计数器全池接着走」 |
| S3 | 回退行 (r_old, T_old, 0, 回退位) 与中间实例行 (i, 0, 0) 写在 R_old 那一版实例表上，与第一个新根同一次发布；回退行的 W 恒 0 | D23（journal 的角色与格式） 已定项 14 回退段；D18（块里携带什么信息） 已定项 11 |
| S4 | 影子账（窄读法）：只隔离「(txg, 实例) 大于 R_old 的可读根引用而 R_old 不引用」的槽；只住内存，不进已分配 / defer / 空闲，只让用户数据落点与开放段绕开；被抛弃根独占量逐盘报数、不进记账行；`ShadowLedger::Off` 是只供测试的开关 | D23（journal 的角色与格式） 已定项 14 影子账那一句；D28（挂载期承诺量） 已定项 1 第九项；C314（回退可以复用被抛弃的根引用的单元）；C318（影子账隔离的单元没进准入不等式） |
| S5 | 前缀第五条：所选根的实例在它自己指着的实例表里有回退行 ⇒ 只施加到 W 为止（W = 0 一条不施加、事务号 0 的记录也不施加）；只读所选根自己的表 | D23（journal 的角色与格式） 已定项 14 六条口径第五条 |
| S6 | 可再分配 ⟺ 已释放 ∧ 释放代 ≤ F_生效（环里最旧有效根第一版恒 0）；回收 = 位图清、占着与 defer 各减、空闲加；回收过的落点被再分配时那条记录改写、槽号每盘唯一 | D16（发布语义） 已定项 1 可再分配那一行；D3（空间分配） 已定项 7；D5（快照 / 空间记账机制） 已定项 4 |
| S7 | 抬 F 的上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根)，非空按「环里有它自己那条记录且事务号 ≠ 0」认（预想）；不足 4 个取最旧的有效根；超上限报错 | D16（发布语义） 已定项 1 抬 F 的上限那一行 |
| S8 | 抬 F：先回收再推带新 F 的空发布，直到每块盘上都有一条（生效）；一条根带的 F 与它的记账行说同一件事；重开时 F_生效 = 各幸存盘所带 F 最大值的最小值，既给分配器回收、也写进新实例的根（F 可以被写低） | D16（发布语义） 已定项 1 生效那一行；D5（快照 / 空间记账机制） 已定项 4 |
| S9 | checker：I-3.1 的并集与 I-2.1 的判定只在候选集里的根上（按最新根实例表有效 ∧ txg ≥ 最新根的 F） | I-3.1（已分配统计对得上） 读法甲；I-2.1（校验和与内容匹配）；D16（发布语义） 已定项 1 |
| S10 | 第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放（记录改写成已释放、释放代 3，defer 待释放 16384）；第一个事务的字节表五随之改 | D3（空间分配） 已定项 7；I-3.1（已分配统计对得上）；E142（第一个事务的干跑） 第十次跑 |
| S11 | 层 0 到 E：oracle 版本表按 (txg, 实例) 找版本；回退之后 (3, 9)、(3, 10) 读第一次内容；抬 F 的两条根读第四版；E 读自己 | D13（验证路线） 已定项 4；E77（发布的持久顺序） 判据 1 |
| S12 | 上一轮改法：所选根自己那条记录读不出时链首只认 `checkpoint_txg = 根 txg + 1` 的那条（一次发布一条记录、txg 每次加一） | D23（journal 的角色与格式） 已定项 14 六条口径第一条 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| X1 候选集与回退目标 | S1 的三个判定与 D23 回退段、D16 候选集那一行逐句对；「按最新根指着的实例表判」在最新根本身是被抛弃时间线的根、或最新根的实例表单元读不出时怎么办；F 之下的根 | 一段历史让不该退到的根被退到，或该退到的被拒 |
| X2 P1 与记录槽复用 | S2：新实例的记录盖掉被抛弃发布的记录槽之后，恢复（普通、回退再回退）、记录核对器、层 0 的记录流有洞判定各在哪一格错；「与没发起回退时相同」（崩在 D 之前）在 B 的记录被 D 的记录盖掉之后还成不成立 | 一段历史让恢复施加了被抛弃的记录，或核对器判错 |
| X3 影子账 | S4 窄读法隔离的集合对不对（被抛弃根引用而 R_old 也引用的槽不隔离；被抛弃根之间共享的槽只隔离一次）；隔离的槽在回退实例的根被轮转覆写之前一直隔离——第一版根环 24 槽走不到；`ShadowLedger::Off` 那两格里另一格（回退那次发布崩在单元写完之后、记录之前，恢复挂上 C 的根）在这条几何上造不造得出 | 一个只被被抛弃根引用的槽被发出，或一个 R_old 引用的槽被隔离 |
| X4 前缀第五条 | S5 只读所选根自己指着的实例表，而回退行写在新实例的表里——什么历史让所选根自己的表里有本实例的回退行；「事务号 0 的记录也不施加」对不对 | 一段历史让回退被静默撤销 |
| X5 回收与复用 | S6：回收之后位图、三个计数、记录三者的一致；回收过的落点被复用时记录改写、`reclaimed` 集合与盘上记录的分歧（重开之后 `reclaimed` 空、盘上记录仍写着已释放、按 F_生效 再回收一遍）；跨度不同的复用（回收 2 槽、再分配 1 槽） | 一个状态让 I-3.1 / I-5.2 红，或同盘同槽两条记录 |
| X6 上限与生效 | S7 / S8：「非空」按记录事务号认，记录撕了怎么办；第 4 新的非空根的读法与 D16 用户定案「最近 4 个」是不是同一个数；先回收再推空发布——第一条带新 F 的根落了、第二条没落就崩，重开后 F_生效 回到旧值、账里的空闲已经算过回收的槽；F 写低（新实例的根写 F_生效）与「F 平时不动、只在准入不够时抬」冲不冲突 | 一段历史让 F 之下的根的单元被复用，或让记账行与根带的 F 说两件事 |
| X7 checker 的候选集 | S9：F 之下的根不走 ⇒ 它们引用的单元坏了没人判；被抛弃时间线的根不走 ⇒ 同上；I-2.1 的射程比条款原文（任一被引用的块）窄了，条款要不要改 | 一份坏镜像 checker 判绿 |
| X8 链首无锚点 | S12：`checkpoint_txg = 根 txg + 1` 与「一次发布一条记录」绑死，第六条（一次发布多条记录）落地时怎么改；回退之后（P1 盖槽）所选根自己那条读不出的形态；与 E142 装置同一条 | 一段历史让不该接的记录接上，或该接的没接 |
| X9 第一个事务改字节 | S10：释放 m2 之后第一个事务的记录、记账与走读（分配记录逐盘核、已释放记录的代判「不晚于根」）各怎么变；kb 字节表五与 E142 第十次跑对不对 | 一处字节表与产物或代码不符 |
| X10 写回 | `02-second-txn.md` 步 4 / 步 5 / 步 0 的「现状」、`layout/01-first-txn.md` 八回退与抬 F 两行、第二条流那句、`invariants.md` I-3.1 / I-2.1 的读法，与代码对不对得上 | 一句现状与代码不符 |

**反向接受条款**：X1–X9 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮；打中的格落在没有条款的地方（P1、W 恒 0、窄读法、非空的读法、F 写低）⇒ 记进 `02-second-txn.md` 对应步的决策点、标预想交用户，代码按最保守的读法改；X10 打中 ⇒ 改现状。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（四条腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史打穿 | X1、X2、X3、X4、X8 | X5、X6、X7、X9（本地攻方的） |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 同上，另一组攻击面 | X5、X6、X7、X9 | X1、X2、X3、X4、X8 |
| Sonnet 正推（`three-way-forward`） | 核「代码做的是不是条款说的」：S1–S12 逐条对附录原文；X10 | — | 不判攻方的格 |
| 本地辩方（`three-way-local-defense`，英文，抽两次） | 替七处实做时定下的取法辩护（P1；回退行 W 恒 0；影子账窄读法；非空按记录事务号认；先回收再推带新 F 的根、新实例的根写 F_生效；链首无锚点认 txg + 1；第一个文件版本释放 mkfs 树表）：每处先写最强的反对再答；另核步 3 那一轮判决第四节 X2 ① 的处置（射程记决策点、代码不动）在回退与抬 F 进来之后还成不成立 | — | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-step45-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-step45-code-r1-main-verification.md`，这一轮结束才有）。前几轮判决可读：`research/prompts/m2-code-r1-main-verification.md`、`m2-code-r2-main-verification.md`、`m2-step3-code-r1-main-verification.md`。本地腿的提示避开 Rust 路径 `::`（字词损坏闸把它判成粘连）。
