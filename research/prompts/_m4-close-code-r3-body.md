# m4-close-code-r3：里程碑四收口修补二批第二段的代码轮正文（2026-11-05）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T3.1 -->

## 一、题面

攻代码轮 m4-close-code-r2 判决（`research/prompts/m4-close-code-r2-main-verification.md`）第三节交下来、与用户 2026-11-04 三条定案合进主工作区的五份实现员补丁里的产品代码。射程是 `crates/singlefs-core/src/` 下这六份：`recovery.rs`、`transaction.rs`、`mounted_read.rs`、`mounted_session.rs`、`mount.rs`、`compaction_plan.rs`（`container_tree.rs` 只改一行测试注释，不在射程）。准入之前那三处不在这一轮：它另走了设计轮 m4-admission-delta（判决 `research/prompts/m4-admission-delta-r3-main-verification.md`）。

| 来源（补丁目录，`~/.local/share/singlefs/work/` 下） | 关的 | 落点（产品代码） |
|---|---|---|
| m4-close-code-r2-fix-reader-writer | r2 判决 T4 改法甲（读回照 I-9.12 判叶间严格升序，片内不升序判 I-9.4）；T3.1 计数加搬迁修复的计数半 | `recovery.rs`、`transaction.rs`、`mounted_read.rs` |
| m4-close-code-r2-fix-session-writer | 另两个读者（只读挂载、快照冷读）判交叠；只坏一份按一次可写挂载累计；有活快照时另发一批搬迁；只读挂载只计数 | `mounted_read.rs`、`mounted_session.rs`、`recovery.rs`、`transaction.rs`、`compaction_plan.rs` |
| m4-close-code-r2-fix-refusal-writer | 快照下另发那批被空间拒时推抬 F 再试；挂载时读到只坏一份进累计（只计数不搬） | `mounted_session.rs`、`mount.rs`、`compaction_plan.rs` |
| m4-close-code-r2-fix-branch-switch-writer | 另发那批两条被拒分支（`RefusedNotForSpace`、`FloorRaiseFailedWhilePushingForSpace`）补只供测试的强制进入开关 | `mounted_session.rs`、`compaction_plan.rs` |
| m4-close-code-r2-fix-sampling-writer | 只改测试代码（准入计数测试放宽取样），不在射程 | — |

改动全文：`research/prompts/_m4-close-code-r3-diff.md`（五份补丁里落在 `crates/singlefs-core/src/` 的 hunk，照打补丁的次序原样抽出）。

被判条款：D4（校验和位置） 已定项 4 定案（`.claude/kb/decisions/04-校验和位置.md` 第 71 行）与 C946（只坏一份的读没有计数也没有搬迁修复）（`.claude/kb/checks-owed.md` 第 595 行）；不变量 I-9.12、I-9.4（`crates/singlefs-checker/invariants/09-inode树结构.md`）；用户 2026-11-04 三条定案（原话记在 `records/2026-10-13-里程碑四预开发.md`）：有活快照时坏副本「另发一批搬迁修好」、只读挂载「只读挂载只计数，不修」、另发那批被拒时「推抬 F 再试一遍」；`.claude/rules/fs-design.md`「五条硬要求」第 2 条（每条分支能被测试强制进入）；D8（核心索引结构） 已定项 16 第 6 条会话走空被空间拒推抬 F 那一套。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- 冻结副本：`~/.local/share/singlefs/work/m4-close-code-r3/repo/`，取五份补丁都打进之后的主工作区（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`prune-server`）。腿读代码读这一份；行号以它为准。开工快照 `research/prompts/m4-close-code-r3-snapshot/crates-sha256.txt`（六份）。
- 这一段的验证（主 agent 经服务跑的，原话记在 `records/2026-10-13-里程碑四预开发.md`）：
  - 两包测试（任务 6513，`cargo test --no-fail-fast -p singlefs-core -p singlefs-harness -- --include-ignored`）：336 个测试二进制「passed=3096 failed=13」；13 条里 11 条多报 I-3.21（立 I-3.21 那一路的期望集没跟），2 条抬 F 上限偏低经五个副本对照坐实出自别的会话的批 19（拿掉这五份照旧红，拿掉批 19 四份转绿），都不是这五份的；
  - 合入后验证（任务 6232）：clippy --all-features 红在这一批新加的测试 `crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs:167`（显式生命周期可省）；checker 档快档带 gpu 那一格红一条 `the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order`（12 个状态 GPU 与 CPU 不一致：CPU 恢复新加的叶序判法 `recovery.rs` 的 `inode_records_of_the_leaf_ascend_after_the_leaves_before`，GPU 恢复内核 `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` 没跟）。这两处已认、并进下一批修补；它们不在产品代码射程里，攻方不为它们开格。
- 不许动的一处：`mounted_session.rs` 里攒批会话排空被拒那一支在起整理之前 `return Err`（`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs` 整理那道门的前提压在它上面，出处 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 690–700 行），三份改 `mounted_session.rs` 的补丁报告都贴了它前后逐字相同。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　四个读者判 inode 叶交叠
可写挂载重建、冷走读、只读挂载 `open_pool_for_read`、快照第一个文件冷读四处都调 `inode_records_of_the_leaf_ascend_after_the_leaves_before`。核：四处判的是同一件事、拒成同一种失败吗；读 inode 树的读者还有没有第五处没接（mount 的读阶段缓存、checker 之外的走读、回退读上一版）；交叠判在空叶、单条叶、叶跨盘副本不一致时会不会误拒合法镜像。

### T2　只坏一份的计数
四条读路径（部分写读回、截断读回、挂载态 `read_at`、冷恢复读）加挂载时建上一版那一读，各自在哪里计、计的是读还是单元、累计在哪（`PoolWriter`、`ReadPathObservation`、`PoolReaderCountingUnitReadsWithOneCopyFailing`、`CompactionOfThisMount` 的私有字段）。核：同一份坏副本会不会在一次操作里被数两次；两份都坏时照旧返错、不计数吗；计数不加任何盘上字节吗；按一次可写挂载累计的数在会话被重开、续做、整理之后对不对。

### T3　有活快照时另发一批搬迁与被拒推抬 F
用户改动发成之后、同一次调用里另发一批（经 `publish_the_compaction_step_once`），被空间拒时推一串抬 F 再试，推满交回 `NoSpaceAfterRaisingTheFloor`、不写字节；不是空间不够交回 `RefusedNotForSpace`、不重试；抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`；三个只供测试的开关。核：另发那批走的是同一个事务层、没开第二条写路径吗；推抬 F 的预算口径与会话走空那一套（D8（核心索引结构） 已定项 16 第 6 条）相同吗；推抬 F 之后会话的 `current` 换到推到的那一版，后续用户改动看到的是对的那一版吗；三个开关在生产路径上恒为关吗；排空被拒那一支真没动吗。

### T4　`publish_the_change_once` 的挪动
为了不让变异表第 2471 行锚点失效，按改动分发的 `match` 原样挪进新函数 `publish_the_change_on_the_writer`，加了 `allow(clippy::needless_borrow)`。核：挪动前后每一种改动走的路逐个相同吗；`allow` 有没有掩住真问题。

## 四、分工（照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）

这是这一段产品代码改动的第一轮代码轮。正推腿逐格核「代码做的是不是条款与用户定案说的」。攻方腿第一轮攻过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱：每格找一个反例（历史或镜像、走到的冻结副本文件:行、期望与实际）。
