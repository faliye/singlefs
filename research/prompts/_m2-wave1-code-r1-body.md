# 背景材料：里程碑「第二个事务」增补 1 与增补 2 第一波代码改动的代码三方对抗第一轮（2026-09-17）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 U7 Y1 Y2 Y3 Y4 Y5 Y6 -->

## 一、被判的对象

里程碑 [02-second-txn.md](../../.claude/kb/milestone/02-second-txn.md)「增补 1」第 1 件与「增补 2」收口表第 19、20、21、23、28、30 行，四个实现员 2026-09-17 17:10–17:42 UTC 交回。工作区没有提交点，被判的范围按「文件::项名」给（门禁 56 号按路径认）：

| 文件 | 项 |
|---|---|
| `crates/singlefs-core/src/write_accounting.rs`（新） | 全文件：`WrittenStructureKind`、`WriteCallsAndBytes`、`WritesByStructureKind` |
| `crates/singlefs-core/src/transaction.rs` | `CommitStep::WriteUnitToEveryDevice` 的 `identity` 字段；`PoolWriter::perform`、`PoolWriter::write_superblock_slot` 的记账；`WarmUpOutput`、`warm_up`、`warm_up_after_journal_counter`；`publish_first_file` 的 jsn；`PublishError::AccountingEntriesExceedOneNode`、`publish_version` 里记账树准入、`accounting_entry_count`；`build_file_version_units`、`publish_admitted` 里的 `BirthSequenceAllocator` |
| `crates/singlefs-core/src/recovery.rs` | `rebuild_version` 里重建版本的空账 |
| `crates/singlefs-core/src/lib.rs` | `write_accounting` 模块声明 |
| `crates/singlefs-core/src/allocator.rs` | `PlacementRefusal`、`agreement_across_devices` 与 `DeviceAgreement`、`DeviceFreeMap::lowest_commit_generated_fallback_slot`、`DeviceFreeMap::commit_generated_answer_without_open_segment`、`PoolAllocator::try_allocate_user_data`、`PoolAllocator::try_allocate_commit_generated`（`allocate_user_data` / `allocate_commit_generated` 改成 `try_*().ok()`） |
| `crates/singlefs-harness/src/crash.rs` | `Layer0Tally::checker_not_applicable_states`、`Layer0Tally::checker_counts_by_invariant`、`evaluate_state_for_versions` 里的计数 |
| `crates/singlefs-harness/src/scenario.rs` | `ScenarioPoint`、`run_first_transaction` 的两处回调 |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` | `name=publish_writes` 与 `name=publish_writes_against_device` 两种结果行、`second-instance` 模式 |
| `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs` | 回调签名跟着改 |
| 用例 | `second_transaction_supplement_one_write_accounting.rs` 两条；`second_transaction_supplement_two_commit_generated_fallback.rs` 四条；`second_transaction_supplement_two_unequal_devices.rs` 三条；`second_transaction_supplement_two_warm_up_counter.rs` 一条；`second_transaction_supplement_two_accounting_node_full.rs` 两条；`second_transaction_step_zero_layer0.rs` 的 `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it` 与 `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state` |
| `crates/mutations.tsv` | 第 77–106 行（名字以「增补 1：」「增补 2：」「增补 2 第」「步 0：」「步 6：」开头的 30 行） |

今天的状态（实现员报告，主 agent 没有复跑）：各自 `check.sh` 绿（最后一个 17:40 UTC）；每条新变异在草稿副本上红在点名的测试上；门禁 54、55、57、59 号没跑。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- 写量计数（`write_accounting.rs` 全文、`transaction.rs` 第 111–164 行）：`PoolWriter` 在设备报成功之后按种类记一笔；单元写的种类由 `CommitStep::WriteUnitToEveryDevice` 新加的 `identity`（`TransactionUnit`）经 `WrittenStructureKind::of_unit` 给，journal 记录、根槽、超级块槽由步骤成员定；一次发布的账是发布前后两次快照之差。合计只从记过的账里加、不经报数清单。取号与 mkfs 的写不在任何一次发布的账里。
- 回落与按盘取落点（`allocator.rs` 第 150–173、430–468、695–745 行）：开放段装不下时每块盘各答一个去处（最低全空段，没有就回落到单元区里起点最小、整个跨度都不被 `is_blocked_for_commit_generated` 挡的落点，两槽按 32768 对齐）；各盘答案一致才分配，一块都答不出报 `NoFreeSlotOnAnyDevice`，部分盘答不出报 `SomeDevicesFullDeviceSetSelectionUndefined`，答案不同报 `CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined`；回落时把开放段置空。发布层（`transaction.rs` 第 1466 行）把所有拒绝报成 `NoSpaceFor`。用户数据同样按盘答、不一致拒绝。
- 暖机（`transaction.rs` 第 405–460 行）：`warm_up` 调 `warm_up_after_journal_counter(…, LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM)`；后者 txg 照格式常量走 1、2，记录号从传入的计数接着数，tail 跟记录号。今天只有 mkfs 之后同一进程里调。
- 记账树准入（`transaction.rs` 第 1195–1227 行）：行数 = 3 + 6 × 盘数，大于一个节点的容量就在拷分配器、动分配器之前返回 `AccountingEntriesExceedOneNode`。树表条目在发布路径里写死 7 条，没有准入。
- 出生序号（`transaction.rs` 第 1450 行）：`publish_admitted` 每次发布建一个 `BirthSequenceAllocator`，文件对象的单元与固定点结构都从它取号。
- 层 0（`crash.rs` 第 505–556 行）：每条不变量报「评估过 / 判违例 / 不适用」；残留记录那条流基镜像里种一条实例 1、jsn 5、txg 5、事务号 3 的记录，31 个状态里 19 个恢复落在 (1, 5)；陈旧 tail 那条流是固定脚本到 E 再覆盖写一次（txg 18，数据单元落回 50180），录制流里每次超级块槽写的 tail 改成 2，8 个状态。两条流上用例按现状钉住两处现象：残留记录流 12 个状态 I-3.1（已分配统计对得上） 判红（收口表第 23′ 行），陈旧 tail 流 8 个状态记录核对器报 `record_claimed_state_missing_unit`（第 23″ 行）。
- 真设备二进制：每次发布一行 `name=publish_writes`，每段窗口一行 `name=publish_writes_against_device` 与设备一层计数比，对不上 `matches=false` 并退出码 1；`second-instance` 模式在发布 B 之后丢掉句柄、冷重开、`mount_writable`、发布 C、冷恢复。已有四个模式的 stdout 只由实现员读代码核过不变，没在虚机里跑。
- `crates/` 里没有的：树表条目的输入（实现员报 `transaction.rs` 第 1781 行写死 7 条，`PublishPlan` 没有树清单字段）；拒绝原因在发布层的区分；盘不等大时按盘不同槽的 `Placement`。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| U1 | 发布路径按 12 种结构计写调用与写字节，一次发布的账 = 前后快照之差，与设备一层逐次相等 | D17（实现分层与第三方管道） 已定项 2 / 已定项 5；`.claude/kb/layout/01-first-txn.md`「零、盘面地图与第一个事务的写清单」 |
| U2 | 提交内生块段耗尽时回落到单元区里不被挡的最低落点 | D3（空间分配） 已定项 8 ②、已定项 10 ⑤；C369（提交内生块段耗尽时没有回落） |
| U3 | 用户数据、开段、回落三处由每块盘各答一个、一致才分配，不一致或部分盘答不出在动状态之前拒绝 | D3（空间分配） 已定项 8；D2（RAID 条带策略） 已定项 2、已定项 10；C368（分配器落点只看盘 0，盘不等大时断言失败） |
| U4 | 暖机记录号按上一条记录接着数、tail 跟记录号；`publish_first_file` 的记录号取上一条 + 1 | D23（journal 的角色与格式） 已定项 18、已定项 19；D16（发布语义） 已定项 8；C366（暖机路径把 txg 写进计数器与 tail） |
| U5 | 出生序号的作用域是一次发布 | D19（块指针的结构与宽度预算） 已定项 9 |
| U6 | 记账树行数超过一个节点在动分配器之前报错 | D5（快照 / 空间记账机制） 已定项 4 的登记表；里程碑步 3 决策点那句「其它树装不下也要报错不 panic」 |
| U7 | 层 0 两条新流与按不变量计数：残留记录只做正例；陈旧 tail 按「恢复完成且终态与 tail 没坏时相同」判 | C42（残留记录冒充合法前缀）、C77（重放起点未定义）；D23（journal 的角色与格式） 已定项 3、已定项 14；`.claude/kb/verification-build.md`「崩溃点重放第一版」必红用例表 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| Y1 回落与挡位 | U2 / U3 下有没有可达的历史让提交内生块或用户数据落在影子账隔离、抬 F 扣住、defer 里、或别的候选根还引用的槽上；回落把开放段置空之后，用户数据会不会落进刚才那一段而那一段里还有这次发布的提交内生块；回落不看 `R`（第一版 `R` 为空）在哪一步会错 | 一段可达历史让一次分配落在被挡或仍被引用的槽上，或同一次发布的两个单元同槽 |
| Y2 拒绝与失败路径 | U3 的三种拒绝在对称的两盘池上走不走得到；走到时分配器与盘上是不是逐字节不变；发布层统一报 `NoSpaceFor` 会不会让调用方做错决定（重试、抬 F） | 对称池上走到拒绝，或拒绝之后分配器指纹、录制流有变 |
| Y3 写量计数 | U1 有没有一次写不经过 `PoolWriter`（回卷写、恢复写、挂载里的写）而进了设备计数、没进账，或反过来；快照差在发布失败、重试时对不对；`identity` 字段会不会被一条路径填错而合计仍对 | 一次发布按种类的合计与设备一层不等，或某一种被记到别的种类而合计相等 |
| Y4 计数与准入的算术（本地腿逐格填表） | U4 在起点记录号 0、1、40 下两次空发布各自的 txg、记录号、tail；U6 在 1、2、78、79、80 块盘下的记账行数、节点容量与准入结果；U5 在同一次发布里连着要四个序号时各自的值 | 表里任一格与代码或条款推出的值不同 |
| Y5 代码与条款 | U2–U7 每条做的是不是条款逐字说的：回落的「同样排除 `R`」、按设备取的「在每一块被选中的设备上各自取」、tail 存「jsn 的 48 位计数器」、出生序号「作用域换到下一个 checkpoint 时清零」；U7 的残留记录正例、陈旧 tail 判法与 C42 / C77 的「怎么拦」一列是不是一回事 | 一处代码按条款的另一种读法写，或一条用例测的不是条款说的那件事 |
| Y6 写回 | 里程碑收口表第 19、20、20′、21、23、23′、23″、28、30 行与步 6 现状那句必红计数，与代码、用例、变异表对不对得上 | 一句与代码不符 |

**反向接受条款**：Y1 / Y2 / Y3 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮；Y4 打中 ⇒ 核是代码错还是条款没写，前者改代码；Y5 打中且落在条款没写的地方 ⇒ 记进收口表、标预想交用户；Y6 打中 ⇒ 改里程碑文字。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb，主 agent 在入库装置上重做才引。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史或镜像打穿 U1–U3 | Y1、Y2、Y3 | Y4 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按事实表逐格算，找代码与条款推出的值不一致的格 | Y4 | Y1、Y2、Y3 |
| Sonnet 正推（`three-way-forward`） | 核 U2–U7 的代码与用例做的是不是条款说的 | Y5、Y6 | 不替攻方找反例 |

两条攻方腿的攻击面：Opus 攻「落点落错、拒绝路径动了状态、计数漏记或记错种类」，都要造一段可达历史；本地攻方只算三张数表（暖机的 txg / 记录号 / tail、记账行数与准入、出生序号），不造历史。

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-wave1-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-wave1-code-r1-main-verification.md`）。前几轮判决可读：`m2-step6-checker-r1-main-verification.md`、`m2-step45-code-r3-main-verification.md`、`m2-code-r2-main-verification.md`。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
