# m4-close-batch2-code-r1 判决：里程碑四收口第二批合入的代码轮第一轮（2026-11-10）

<!-- doc-lint:not-numbers R -->

## 一、核查

- 腿：正推 `research/prompts/m4-close-batch2-code-r1-sonnet-output.md`；攻方 `research/prompts/m4-close-batch2-code-r1-opus-output.md`、承接 `research/prompts/m4-close-batch2-code-r1-opus-s2-output.md`（只修 A2 的探针造法）。各抽一次，「没打中」不支撑结论。
- 开工快照 `sha256sum -c research/prompts/m4-close-batch2-code-r1-snapshot/crates-sha256.txt`：五份 `OK`，写判决时没有对不上的。
- `python3 scripts/agents/cite-check.py` 正推与攻方两份：`✓ 核了 40 处引文、64 处裸行号，对上 39 处，没判 65 处（其中裸行号没有引文可比 49 处；逐处列在上面）`。判决引的行（`mounted_session.rs` 第 2624、3226–3233、3379、4276、4450、5872–5884 行，`mount.rs` 第 7732 行，`recovery.rs` 第 193 行，`compaction_plan.rs` 第 636–641 行，D26 已定项 13 第 4 条「在任何写之前就报停着」那一句）主 agent 现读过，与腿引的一致。
- 复跑：攻方两条探针在副本里跑（任务 12031，模型拷到 `~/.local/share/singlefs/work/m4-close-batch2-code-r1-main-rerun/model`），快照核过 `name=m4_close_batch2_attack_snapshot rc=0 expected=0`。探针一绿；探针二在造前提处红「在 CPU 上只发了签发那一次，意图在强制选的那一段上：Stopped(NoEligibleSegment)」，A2 没量到。承接腿改了造法，新探针任务 12045 在排。
- 检查单第六格复跑：门禁 code-tooling 的 heavy-thread-defaults 格任务 12029 不再点名这一批的两份测试文件（剩下 3 处在 `scripts/` 下，不是这一批的，归别的会话、已修）；两份测试改线程取法后任务 12030 `5 passed`、`10 passed`。

## 二、逐格判

| 格 | 判 |
|---|---|
| A1 后台入口停着时不删意图 | 打中，量的。任务 12031：`background_entry_while_stopped not_started_while_stopped=[true, true, true] not_started_after_a_user_change=true live_intent_on_disk=Some((CheckpointTxg(11), …))`，同一刻显式入口 `explicit_entry_while_stopped outcome=Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError) versions_published=1 live_intent_on_disk=false`。正推 1.2 同报。用户 2026-11-10 定「后台入口也删」：判出停着、现行那一版有活意图时起跑一趟只删意图，之后照旧不写；D26 已定项 13 第 4 条「在任何写之前就报停着」限在切到一张卡都不剩、手里没有活意图那一刻 |
| A2 暂停删意图那一次的落点受 R 排他 | 打中，量的。任务 12045（承接腿的探针，副本里只有 `mount.rs` 与开工快照不同，是别的会话打进的补丁、不在删意图路径上）：`drop_refused_setup … free_slots_in_the_region=[34, …] free_slots_outside_the_region=[0, …]`，两趟都是 `Failed(Batch(Publish(PlacementRefused { unit: AllocationTreeNodeBelowTheRoot(…), refusal: NoFreeSlotOnAnyDevice }))) refused_short_of_space=true versions_published=0`，意图与 R 排他都留着；对照臂 `drop_contrast_with_the_region_open … published=true detail=txg=CheckpointTxg(17) live_intent_on_disk=false rewritten_slots_in_the_region=10`。正推「面 1」同报这一边角。用户 2026-11-10 定「删意图那一次放开 R」 |
| A3 落后支两处经裸读者调 | 打中，读码。`mounted_session.rs:2624`、`mount.rs:7732` 传的是盘表，`recovery.rs:193` 默认实现什么都不记；窗口之外坏的那一份放行而不计数，之后那次发布的轮换推平 tail，再也看不见。与 D4 已定项 4「可写挂载里的读都计数并搬」对不上 |
| 正推 1.3 删意图免式子的射程比定案字面宽 | 采纳实现的结构判据（只删整理意图、不搬单元）：另三种（R 已全空的删意图、丢意图、零搬迁的最后一批）盘上写的与暂停那一次相同，落点那一判照判。写回 D28 已定项 1 时写明这四种 |
| 正推 1.4 抬 F 第一次空发布被拒时那一读不进账 | 不改这一批，登欠账：之后每次发布写配置槽前再读同一张实例表，差的是一次计数（推的） |
| 四·1 `budget == 0` 走不到 | 成立（两腿三个入口都追过）。留作根环预算护栏；A1 改完后后台入口判出停着走删意图那一趟，预算照入口给 |
| 四·2 类型与名字 | 成立。`mount.rs:7162` 新 `expect` 依赖 `JOURNAL_RECORD_BYTES` 恒 4096；注释里的序数写法与形参 `devices: &Reader` 门禁不判，不改 |
| 四·3、四·4、四·5 | 成立 |
| 四·6 测试优先多线程 | 合入那一版不成立（两腿都报），已改：三处改成 `singlefs_verification::heavy_threads::default_heavy_threads()`，见第一节复跑 |
| 四·7 带 --item 的命令与项 | 成立（读登记行，`--list-items` 没跑） |
| 四·8 新崩溃流 | 两句成立。一趟开头停着删意图（排在删除续做之前、队列不空时照抄队列）与补跑之后下一批开头删意图两处没有录制流，A1 改完后台入口那一处同形，一并交实现员补一条录制流 |
| 三·2 同实例点名窗口 | 两腿都没打中（一次抽样） |
| 三·4 交叉 | 并进 A3 |

## 三、交实现员（下一批）

1. A1：`mounted_session.rs` 后台入口判出停着而现行那一版有活意图时起跑一趟，只发一次删意图（与显式入口一趟开头同一段），之后照旧 `NotStarted`；三处文档串（`mounted_session.rs` 第 3230、4276 行，`compaction_plan.rs` 第 639 行）写实。验收：探针一的历史下后台入口那一次发一版删意图、盘上意图与 R 排他都不在；变异「后台入口停着不删」会红。
2. A3：`mounted_session.rs:2624` 与 `mount.rs:7732`（`caller_inputs_agreeing_with_the_disk` 及其五个调用方）的落后支判断经计数层的读者调，窗口之外坏的那一份进账、搬。验收：用例造挂着换上一块落后盘、窗口之外坏一份，先挂着回退或抬 F，卸载前账上有这一组；变异「换回裸读者」会红。
3. 四·8：给停着删意图（后台入口与一趟开头两处）补一条录制流或小用例钉住与暂停流逐步相同。
4. A2：删意图那一次（`CompactionIntentStep::Drop` 与达成删意图）取落点不排除 R，删意图那一版落盘之后才撤分配器侧的意图。验收：探针二的前提下暂停删意图发成、意图与 R 排他都不在；删意图途中崩溃的状态里 I-5.6 成立；变异「删意图照旧排除 R」会红。被攻过零轮，下一轮代码轮攻它。

## 四、交用户

| 项 | 状态 |
|---|---|
| A1 后台入口也删 | 用户 2026-11-10 已定 |
| A2 删意图那一次放开 R | 用户 2026-11-10 已定，被攻过零轮 |

## 五、延后（去向）

随 kb 写回登 `.claude/kb/checks-owed.md`：① 抬 F 第一次空发布被拒时那一读不进账（正推 1.4）；② 零单元发布 `publish_without_units` 不转交那一支没有用例点名（正推 1.4）。D28 已定项 1 不判名单、D26 已定项 3、D26 已定项 13 第 4 条、D23 第 405 行的写回在这一批 kb 写回里做。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs
- crates/singlefs-core/src/compaction_plan.rs
- crates/singlefs-core/src/mount.rs
- crates/singlefs-core/src/mounted_session.rs
- crates/singlefs-core/src/transaction.rs

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 不影响 | 2026-11-10 不受影响：后台入口停着删意图由用户定案，写回随 kb 写回批 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-10 不受影响：不判名单的射程写回随 kb 写回批 |
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-10 不受影响：A3 是实现没接全，条款不改 |
| D18（块里携带什么信息） 已定项 11 | 不影响 | 2026-11-10 不受影响：窗口两腿都没打中 |
