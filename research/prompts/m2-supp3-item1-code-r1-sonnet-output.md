# 增补 3 第 1 件（随机历史生成器）代码三方第一轮：正推腿报告（Sonnet 5）

分到的格：Z1（条款那一半）、Z6。时刻按 UTC 记（东京 = UTC+9）。开工约 2026-09-18 15:30Z。

## 一、Z1（条款那一半）：生成器调每个入口的方式是不是条款与文档注释允许的

### 1.1 两条前提各自落在哪条条款、哪行注释上（逐条 grep -nF 核对）

**前提 A（抬 F 的目标不低于今天的 F）**：

`crates/singlefs-harness/src/history.rs` 第 184–189 行（`FloorTargetChoice::steps_above_current_floor` 的文档注释）：
```
/// 新 F = 现行版本根记录的 F + `steps_above_current_floor mod (现行 txg − 现行 F + 3)`：最大到现行 txg + 2，
/// 而上限（D16（发布语义） 已定项 1）不超过每块盘上最新的有效根 ≤ 现行 txg，所以超过上限的目标一定取得到。
/// 生成器的前提：目标只取现行 F 及以上，不往下抬。没有条款明写，按入口的文档注释取的读法——
/// `crates/singlefs-core/src/mount.rs` 第 590 行「抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）」，已定项 1 在
/// `.claude/kb/decisions/16-发布语义.md` 第 107 行；往下的目标这个入口接不接、接了算不算合法没有条款（F 回落在增补 2 收口表第 ② 行里打回重议）。
```
现查 `crates/singlefs-core/src/mount.rs:590`（`grep -n` 命中，逐字一致）：
```
590:/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）：正常的触发是准入不够，这里是只供测试的强制入口（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
```
现查 `.claude/kb/decisions/16-发布语义.md:107`（`grep -n` 命中，逐字一致）：
```
107:1. **defer 窗口取多大：取按盘回退下界的收严形态、按状态数读——候选集保留最近 4 个可退到的不同状态，可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)，根记录带回退下界 F 8 字节。** **状态：已定。**（2026-09-13 用户定案 + E139（按盘回退下界的收严形态） + 四轮三方论证）
```
**判定：一致。** 两处引文逐字命中，`history.rs` 自己也如实写「没有条款明写，按入口的文档注释取的读法」——它没有把这条自称为条款，只称为对文档注释的读法，这一点与代码里 `raise_rollback_floor` 的实现相符：函数本身不检查 `new_floor >= current_floor`（第 595–618 行只检查 `new_floor > ceiling`），generator 的这条限制是它自己主动收紧探索范围，不是遵循了一条它读错的条款。

**前提 B（`publish_without_units` 只在没有文件的一版上调）**：

`history.rs` 第 527–533 行（`MissingPrecondition::CurrentVersionWithFile` 的文档注释）：
```
/// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写四个固定点单元，那是挂载内部的 `publish_version`，
/// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 第 506 行
/// 「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」，已定项 9 在 `.claude/kb/decisions/16-发布语义.md` 第 192 行。
```
现查 `crates/singlefs-core/src/transaction.rs:506`：
```
506:/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
```
现查 `.claude/kb/decisions/16-发布语义.md:192`：
```
192:9. **一次空发布写不写单元（2026-09-13，用户定案）：记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行，连带记账树节点、分配记录、映射条目与树表单元；第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。** 正文见 D16（发布语义）「已定项 9」。 **状态：已定。**
```
**判定：一致。** 三处引文逐字命中。代码侧核对：`apply_publish_without_units`（`history.rs:1291-1304`）在调 `publish_without_units` 之前守 `let PoolVersion::WithoutFile(previous) = &session.current else { return StepOutcome::NotApplicable(...) }`，与「只在没有文件的一版上调」一致；这个精确对应 `publish_without_units` 自己文档注释里「树表 0 条 ⇒ 零单元」讲的正是暖机与树表 0 条池上的可写挂载路径。

### 1.2 有没有别的入口前提生成器没守、也没写

`history.rs` 全文只有两处「前提」注释带出处引用（`grep -n "前提"` 命中 14 行，只有第 185、530 两处带 kb / 代码行号引用；其余是转述或统计文案，见下方原样命中）：
```
185:    /// 生成器的前提：目标只取现行 F 及以上，不往下抬。没有条款明写，按入口的文档注释取的读法——
188:    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
530:    /// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 第 506 行
532:    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
```
逐一核了七个被调入口（`publish_first_file`、`publish_overwrite`、`publish_without_units`、`mount_writable`、`mount_rollback`、`raise_rollback_floor`、`recover`）自己的文档注释：`crates/singlefs-core/src/transaction.rs`、`mount.rs` 两个文件里 `grep -n "# Panics"` **零命中**——七个入口全部只有 `# Errors`，没有一个写「调用方必须保证 X，否则 panic」这类契约。这意味着：
- 生成器没有守、也没写的「panic 契约」**不存在**：没有第三条隐藏前提是「文档明说不许违反、否则 panic」而生成器绕过了它。
- `raise_rollback_floor` 自己的 `# Errors` 还列了另外两条生成器没有专门守卫的前提——「现行版本里没有重写过的实例表单元」「`new_floor` 超过上限」——但这两条都在 `# Errors` 之列，代码把它们变成 `Err`（`RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`、`RollbackFloorAboveCeiling`），生成器按 `StepOutcome::Refused` 接住即可，不需要专门的 `MissingPrecondition` 守卫；主工作区实测（见二.2 复跑）计数里两者都出现过（`Err 成员 MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion：34 次`、`RollbackFloorAboveCeiling：122 次`），说明这条路径真的被走到过，不是摆设。
- `publish_first_file` 自己另有一条更细的前提（`transaction.rs` 第 1129–1134 行：只接得上 txg 2、jsn 2 的那一版，否则 `FirstFileVersionNotRightAfterTheSecondWarmUp`），生成器的 `apply_publish_first_file`（`history.rs:1210-1248`）同样不专门守卫，直接调用、让它按 `# Errors` 走 `Err`；`operation_weights` 里 `ExpectedSession::OpenWithFile` 一档仍以权重 4 抽 `PublishFirstFile`（`history.rs:292`），实测计数（二.2）「Err 成员 PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp：58 次」证明这条也真被走到、且以 `Err` 收场，不是 panic。

**判定：一致，且没有找到第三条被隐藏或漏写的前提。** 生成器对「没有专门守卫的入口条件」的处理方式统一：让它们落进入口自己的 `# Errors`、变成 `StepOutcome::Refused`，这与 W2「入口返回 `Err` 算合法结局」的口径一致；两条特意加了 `MissingPrecondition` 守卫（前提 A、B）都对应「不守卫就会把探索样本大量浪费在必然 `Err` 的分支上」（`RaiseNeedsRewritten…`、`FirstFileVersion…` 这类不专门守卫也只是让 `Err` 出现的比例更高，不影响正确性），而不是对应「不守卫就会 panic」——因为两个入口自己都没有 `# Panics` 契约。

**推翻条件**：日后若在 `transaction.rs` / `mount.rs` 任一入口的文档注释里发现一条 `# Panics`（或等价的「调用方必须保证……」硬约束），而生成器没有为它加对应的 `MissingPrecondition` 守卫、也没有在注释里提及，则这条判定作废，要按 Z1 的失败条款记「入口条款有依据、生成器没守也没写」。

### 1.3 入口集合与「只调公开入口」的核对

`.claude/kb/milestone/02-second-txn.md:401`（设想实现 1）逐字列的入口：`transaction::publish_first_file`、`publish_overwrite`、`mount::mount_writable`、`mount::mount_rollback`、`mount::raise_rollback_floor`、`transaction::publish_without_units`、冷启动 `recovery::recover`。`history.rs` 第 28–35 行的 `use` 逐一对上，`grep -c "pub fn "` 核对这七个都是 `pub fn`（现查 `transaction.rs`/`mount.rs`/`recovery.rs` 对应签名行，均为 `pub fn`）。

另发现两处直接调用 `acquire_instance`、`warm_up`（`history.rs:477,479`），不在这七类操作之列。核实：这两处只出现在 `HistoryPool::start` 里，只用于搭「第一个文件之后」这个确定性起点（等价于 `tests/common::build_pool`），不参与 `generate_history` 随机抽取的操作序列，且用 `.expect(...)` 而非静默吞错——注释写明这是与既有测试基础设施共用的固定初始化路径，不是被检验的「历史」的一部分。**判定：一致，不算违反「只调公开入口」**：这两个调用本身也是公开入口（`pub fn`），且只用于起点搭建，不受随机化，不产生「装置捏造的失败」。

## 二、Z6：里程碑「增补 3」现状段、收口表第 ② 行 2026-09-18 追加句、第 43 行与代码/用例/产物是否对得上

### 2.1 逐句核对（现查行号：`.claude/kb/milestone/02-second-txn.md` 第 384 行现状段、第 311 行 ② 行、第 360 行 43 行）

| 句子（原文摘录，来自 kb 现查） | 依据（代码/用例/产物） | 判定 |
|---|---|---|
| 「生成器、执行器、已知红清单、收缩与大档在 `crates/singlefs-harness/src/history.rs`」 | 现查该文件：`generate_history`（392 行）、`execute_history`/`execute_history_observing`（1514/1518 行起）、`KNOWN_RED_FORMS`（760 行）、`shrink_operations`/`shrink_failing_history`（1761/1835 行）、`run_history_campaign`（2004 行，含大档路径）均在此文件 | 一致 |
| 「快档（种子 [0, 96)、每段 30 步、`check.sh` 里约 26 秒）在 `.../second_transaction_supplement_three_random_history.rs`」 | 现查测试文件第 18–20 行：`FAST_TIER_FIRST_SEED=0`、`FAST_TIER_SEEDS=96`、`FAST_TIER_OPERATIONS_PER_HISTORY=30`；本轮在独立副本上实测两次分别 15.8 秒（探针）与 52.0 秒（同机有其他 cargo 进程占用时，见下）、implementer 报告记 25.4–25.9 秒（机器轻载） | 一致（耗时受机器负载影响，「约 26 秒」与report 轻载读数一致） |
| 「随机源手写 SplitMix64」 | 现查 `history.rs:59` 注释与 `next_word` 实现（73–77 行，SplitMix64 的标准混合步骤） | 一致 |
| 「把 `crates/mutations.tsv` 第 41、121 行改回去快档都判红（第 129、130 行钉着；第 121 行那一条只有种子 54、69 判得出，余量薄）」 | 现查 `crates/mutations.tsv`：全文 130 行，第 129/130 行原文与替换文逐字对应第 41/121 行同一处代码（`sed -n '41p;121p;129p;130p'` 核对，见下）；报告 §12：「第 130 行…只有两个种子判得出：54、69」 | 一致 |
| 「第一次跑撞出清单外一类，登记成已知红第 1 条（增补 2 收口表第 43 行）」 | 报告 §4「新发现」→ §9 问题 1 → §12「`closeout_table_row` 填成『增补 2 收口表第 43 行』」；本轮独立复跑（下 2.2）确认打补丁后种子 80 归为 `KnownRed(form=1)` | 一致 |
| 「快档 96 段里 50 段以已知红第 0 条收尾（收口表第 ② 行），那一行不修，随机历史罩不到根环转过之后的状态」 | 本轮独立复跑（下 2.2）：96 段中已知红第 0 条恰为 50 段 | 一致 |
| 「还没做：第 1 件的代码三方、门禁 54 与 59 号…；第 2–7 件」 | 报告 §10「没做什么」：未走三方、未做层 0/QEMU/herd7/crates 变异表整表复跑、未做第 2–7 件 | 一致 |

### 2.2 独立复跑一：快档 96 段（仓的副本，不动主工作区）

草稿目录 `/tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo/` 是仅含 `Cargo.toml`、`Cargo.lock`、`crates/` 的独立副本；`sha256sum` 核对副本里 `history.rs`、测试文件与主工作区逐字节相同（均为 `5e715bbe…`、`4fa5f0f5…`，与实现员报告 §12「打完之后的 sha256」一致）。在该副本上跑：

```
$ cd /tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo
$ nice -n 19 cargo test --manifest-path Cargo.toml -p singlefs-harness \
    --test second_transaction_supplement_three_random_history -- --nocapture \
    random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
```
原样输出（首两行）：
```
── 随机历史快档 ──
种子 [0, 96)，每段 30 步
历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
```
末尾：
```
已知红第 0 条（增补 2 收口表第 ② 行（一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红））：50 段；…
已知红第 1 条（增补 2 收口表第 43 行）：1 段；前几个种子 [(80, Operation(21))]
用时 52.0 秒
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... ok
```
（本次机器上同时有另外几个 `cargo test --release … random_histories_large_tier…` 进程在跑，见下 2.3，用时因此比报告 §12 的 25.7 秒长，但**逐项计数与主工作区实现员报告 §12 逐字相同**：45/50/1/0，种子 80 落在 `Operation(21)`。）

**判定：一致。** 「快档 96 段里 50 段以已知红第 0 条收尾」「第一次跑撞出清单外一类、后登记为已知红第 1 条」两句在独立副本上原样复现。

### 2.3 独立复跑二：大档 3000 段（在草稿副本上跑，尚未跑完/见下）

已用同一副本起一个后台任务（`SINGLEFS_RANDOM_HISTORY_SEEDS=3000 SINGLEFS_RANDOM_HISTORY_OPERATIONS=40`，release，16 线程），命令与日志路径见「没做什么」一节；这一格不用等它跑完也能核：**仓里已经存的产物** `research/prompts/m2-supp3-item1-implementer/proposal/round4-large-tier-3000x40.log` 原样：
```
历史 3000 段：跑完 1041、以已知红收尾 {0: 1929, 1: 30}、新发现 0；根环转过一圈的 2102 段；最高 txg 50
```
与收口表第 ② 行追加句「大档 3000 段里 1929 段以它收尾」、第 43 行「大档 3000 段里 30 段」逐字对得上。**这份产物本身是实现员在 `/tmp/claude-1000/m2-supp3-item1/proposal/repo` 这个副本上跑出的**（日志第一行 `Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/m2-supp3-item1/proposal/repo/crates/singlefs-harness)`），按 `evidence-discipline.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」，这份数没有在入库装置（主工作区）上重做过；本轮在另一个独立副本上起的后台复跑（2.4）是对它的核实，核实结果见下。

### 2.4 独立复现三：第 43 行「换 F 之后红与不红的格子」（本轮自己写的探针，仓的副本上跑，未动主工作区）

第 43 行称：「同一段只换 F，7、11、12 不红，8、9、10 红，红的正好是被抛弃实例的三个 txg」。这段话来自实现员报告 §4 一次性草稿用例（未入库），本轮独立重写一遍同一段 11 步历史（步骤逐项照第 43 行与测试文件里 `raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form` 抄，只把第 10 步 `steps_above_current_floor` 从写死的 8 换成 7..12 六个值），落盘为 `research/prompts/m2-supp3-item1-code-r1-sonnet-model/probe_row43_floor_targets.rs`，跑在 `/tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo`（`crates/singlefs-harness/tests/zzz_sonnet_r1_probe.rs`，与仓里那份逐字节相同）：

```
$ cd /tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo
$ nice -n 19 cargo test --manifest-path Cargo.toml -p singlefs-harness \
    --test zzz_sonnet_r1_probe -- --nocapture
```
原样输出：
```
steps_above_current_floor=7 -> new_floor=Some(CheckpointTxg(7)) ending=Completed outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(7), publishes: 3, reclaimed_placements: 26, … }))
steps_above_current_floor=8 -> new_floor=Some(CheckpointTxg(8)) ending=KnownRed(form=1) outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(8), publishes: 3, reclaimed_placements: 26, … }))
steps_above_current_floor=9 -> new_floor=Some(CheckpointTxg(9)) ending=KnownRed(form=1) outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(9), publishes: 3, reclaimed_placements: 26, … }))
steps_above_current_floor=10 -> new_floor=Some(CheckpointTxg(10)) ending=KnownRed(form=1) outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(10), publishes: 3, reclaimed_placements: 26, … }))
steps_above_current_floor=11 -> new_floor=Some(CheckpointTxg(11)) ending=Completed outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(11), publishes: 3, reclaimed_placements: 31, … }))
steps_above_current_floor=12 -> new_floor=Some(CheckpointTxg(12)) ending=Completed outcome_at_10=Some(Applied(RaisedFloor { new_floor: CheckpointTxg(12), publishes: 3, reclaimed_placements: 35, … }))
test sonnet_r1_probe_row43_floor_targets_seven_through_twelve ... ok
```
（`current_floor=0`，`FloorTargetChoice::steps_above_current_floor` 与 `new_floor` 之间没有取模折叠，见 1.1 式子，所以取 7..12 直接对应 F=7..12。）

**逐格判定**：F=7 → `Completed`（不红）；F=8/9/10 → `KnownRed(form=1)`（红，即已知红第 1 条 / 收口表第 43 行）；F=11/12 → `Completed`（不红）——**与第 43 行「7、11、12 不红，8、9、10 红」逐字吻合**。F=11 时 `reclaimed_placements` 从 26 变 31（+5），与实现员报告 §4「F = 11 时回收从 26 个落点变成 31 个，多出的 5 个与 (2, 7) 那 6 槽的落点数…对得上」的方向一致（该报告没有把「6」与「+5」的差异摊开解释——(2,7) 那条根自己独占的落点里有 1 个在 F=8/9/10 时已经不属于「多出的」范围，超出本轮判定射程，未深究）。

**推翻条件**：若换一台机器、换一个 rustc 版本复跑上表六个值，任何一格的 `ending` 不同，则第 43 行这句作废，要重新核 `raise_after_rollback_leaves_allocated_statistic_above_walked` 与 `KNOWN_RED_FORMS` 第 1 条的匹配条件。**判定：一致。**

### 2.3（续）大档后台复跑已完成：独立确认

同一副本上的后台任务已跑完（`/tmp/claude-1000/m2-supp3-item1-r1-sonnet/large-tier-3000x40.log`，`SINGLEFS_RANDOM_HISTORY_SEEDS=3000 SINGLEFS_RANDOM_HISTORY_OPERATIONS=40 SINGLEFS_RANDOM_HISTORY_THREADS=16`，release）。原样输出：
```
── 随机历史大档 ──
种子 [0, 3000)，每段 40 步
历史 3000 段：跑完 1041、以已知红收尾 {0: 1929, 1: 30}、新发现 0；根环转过一圈的 2102 段；最高 txg 50
…
已知红第 0 条（增补 2 收口表第 ② 行……）：1929 段；……
已知红第 1 条（增补 2 收口表第 43 行）：30 段；前几个种子 [(80, Operation(21)), (174, Operation(10)), (254, Operation(15)), (292, Operation(15)), (435, Operation(17)), (556, Operation(37)), (616, Operation(11)), (659, Operation(26))]
用时 243.0 秒
test random_histories_large_tier_seeds_and_length_from_the_environment ... ok
```
**逐字与仓里已存产物 `research/prompts/m2-supp3-item1-implementer/proposal/round4-large-tier-3000x40.log` 相同**（1041/1929/30/0，种子列表 80、174、254、292、435、556、616、659 一致），且这次是在**另一个独立副本**（`/tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo`，与实现员的 `/tmp/claude-1000/m2-supp3-item1/proposal/repo` 是两份不同的拷贝）上重新跑出来的，不是抄同一份产物。**判定：一致**，收口表第 ② 行「大档 3000 段里 1929 段」与第 43 行「大档 3000 段里 30 段」两句独立坐实。用时 243.0 秒比实现员报告的 105.8 秒长，原因是这次机器上同时有另外若干个 `cargo test … random_histories_large_tier…`（release）进程在跑（`ps` 记录见「没做什么」），不影响计数结果。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z1（前提 A：抬 F 不低于今天的 F） | 一致 | `history.rs:184-189` 的引文与 `mount.rs:590`、`16-发布语义.md:107` 逐字命中；代码本身不检查方向，是生成器自己收紧探索范围，注释如实写「没有条款明写」 |
| Z1（前提 B：`publish_without_units` 只在无文件版上调） | 一致 | `history.rs:527-533` 的引文与 `transaction.rs:506`、`16-发布语义.md:192` 逐字命中；`apply_publish_without_units` 的 `PoolVersion::WithoutFile` 守卫与之对应 |
| Z1（有没有别的入口前提没守没写） | 一致，未发现第三条 | 七个入口全无 `# Panics`；`raise_rollback_floor`、`publish_first_file` 各自另一条 `# Errors` 前提没有专门守卫，但都落进 `Err`（快档实测各自 122/34 次、58 次命中），不构成「违反了条款或文档注释」的非法调用 |
| Z1（入口集合与只调公开入口） | 一致 | 七类操作与里程碑「设想实现 1」逐一对应；`acquire_instance`/`warm_up` 只用于起点搭建，不进随机操作序列 |
| Z6（增补 3 现状段逐句） | 一致 | 六句核心陈述（文件位置、快档规模、随机源、变异判红、已知红登记、还没做什么）逐句与代码/报告/独立复跑对得上 |
| Z6（收口表第 ② 行 2026-09-18 追加句） | 一致 | 「快档 96/50」「大档 3000/1929」两个数在本轮两次独立副本复跑（96 段用 2.2、3000 段用 2.3）上原样复现 |
| Z6（收口表第 43 行整行） | 一致 | 11 步复现、`allocated=1441792`/`walked=1343488`/差 `6×16384`、种子 80/大档 30 段均对得上；F=7..12 的红/不红格子本轮独立复跑（2.4）逐格吻合 |

## 四、没做什么

- 不判 Z2、Z3、Z4、Z5，也不替 Opus 攻方与本地攻方腿找变异（分工表明写）。
- Z1 的「每个入口一行」前提表（文档前提 vs 生成器守的前提逐条对比）归本地攻方腿；本节 1.2 只核了「有没有一条隐藏的 panic 契约」，没有逐字段填那张表。
- 没有验证 W2、W3、W4、W5（分给 Opus 攻方与本地攻方），只在 1.2 里引用了实现员报告里的计数作为「路径真被走到」的旁证，没有自己重新设计变异。
- 大档 3000×40 的独立复跑本轮用的是**同一批随机操作序列**（同一套生成器代码、同一个种子区间），与实现员在另一个副本上的产物是「同一个确定性计算的两次独立执行」，不是两条独立的验证路径（不构成 `implementation-first.md`「校验路径本身也要证明它会红」意义上的独立校验）；要更强的独立性需要一条不共享 `history.rs` 代码的重算路径，本轮没有做，也不在 Z1/Z6 射程内。
- 没有跑门禁 54 号（层 0 全量）与 59 号整表复跑——`crash-verifier` 的职责，报告 §10/§12 已如实写「没做」，本轮不重复判它。
- `crates/singlefs-core/src/mount.rs`/`transaction.rs` 的 `# Errors` 之外是否还存在别的、写在别处（例如 kb 决策文件而非代码注释里）的入口前提未穷举——只查了与 Z1 直接相关的这两处及七个入口自身的文档注释，没有对整个 `singlefs-core` 公开 API 做前提普查（那是「预想实现」第 6 件 panic 面普查的范围）。
- 本轮开跑前 `ps` 看到主工作区上已有 `second_transaction_step_zero_layer0`（release，--include-ignored，另一会话／腿在跑）与两个 `random_histories_large_tier…`（release，--ignored）进程，均为普通 `cargo test`，不属于需要停下等待的性能测量类进程（`qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio`），按规则加 `nice -n 19` 照常在自己的独立副本上跑，没有与它们抢主工作区的构建产物。
- 草稿目录 `/tmp/claude-1000/m2-supp3-item1-r1-sonnet/` 下 `repo/`（仓副本，含 `zzz_sonnet_r1_probe.rs`）与 `large-tier-3000x40.log`（3000×40 大档复跑日志）未清理，供复核。

## 五、文件与哈希

- 报告：`research/prompts/m2-supp3-item1-code-r1-sonnet-output.md`（本文件）
- 模型：`research/prompts/m2-supp3-item1-code-r1-sonnet-model/probe_row43_floor_targets.rs`（sha256 `03e0636b3bf56c7963f001b53776511942d4d42e30b32fd9114306b419aafa4c`）
- 草稿：`/tmp/claude-1000/m2-supp3-item1-r1-sonnet/repo/`（仓副本）、`/tmp/claude-1000/m2-supp3-item1-r1-sonnet/large-tier-3000x40.log`（大档独立复跑日志）
