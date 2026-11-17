# m4-gpugc-mount-runtime-code-r2：判决（2026-11-09）

正文 `research/prompts/_m4-gpugc-mount-runtime-code-r2-body.md`，改动全文 `research/prompts/_m4-gpugc-mount-runtime-code-r2-diff.md`，开工快照 `research/prompts/m4-gpugc-mount-runtime-code-r2-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端辩方（Sonnet） | `research/prompts/m4-gpugc-mount-runtime-code-r2-sonnet-output.md`，模型 `research/prompts/m4-gpugc-mount-runtime-code-r2-sonnet-model/` | 满盘撤回「运行库救得出来」够不着；余量取 S 没比过别的取法；两处新缺口；类型与名字不成立（低危） |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-mount-runtime-code-r2-opus-output.md`，模型 `research/prompts/m4-gpugc-mount-runtime-code-r2-opus-model/` | 最重是冻结着 fsync 答成功；重发成了之后删除队列没人走；满盘加 fsync 零发布；类型与名字不成立 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 11221 退 0，树在 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/m4-gpugc-mount-runtime-code-r2-tree`。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-mount-runtime-code-r2-opus-output.md research/prompts/m4-gpugc-mount-runtime-code-r2-sonnet-output.md --root <那棵树> --background research/prompts/_m4-gpugc-mount-runtime-code-r2-background.md` 末行「✓ 核了 27 处引文、154 处裸行号，对上 16 处，没判 165 处（其中裸行号没有引文可比 96 处；逐处列在上面）」，没有对不上的。没判的是快照树之外的路径（`transaction.rs`、第一轮报告、工作根下的读数）与裸行号，判决依赖的那几处主 agent 现读过：`crates/singlefs-core/src/mounted_session.rs` 第 3345–3412 行（起跑判据）、第 4316–4350 行（删除续做）、第 6296–6320 行（待发空答 `None`）、第 6578–6590 行（fsync 入口）。
2. 攻方探针任务 11213 退 0，辩方探针任务 11210 退 0。两份探针腿都没编过，在主 agent 这里第一次编过、第一次出数（量的）。输出 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/run/attack_r2_probes.out`（sha256 `8d325919…`）、`.../run/attack_r2_full_pool.out`（`69e1d7e3…`）、`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-run/readings.out`（`0ca92877…`）。
3. 被判的代码与主工作区此刻逐字节相同的说法是腿量的；主工作区合入之后只多了测试里两处改名与一处格式，不碰被判的产品代码。合入后验证任务 11209：「✓ 测试全绿（16 个测试二进制，共 531 条测试；跑了 16 项（4 个包），--item 点名的；按条复用：现判 530、复用 251、拒算 1）」「✓ clippy 过（2 个特性组合：default --all-features；每个带 check.sh 的编码纪律 lint 7 条）」；fmt 红在那一处改名撑长的行，已修；写这份判决时这个任务还在跑后面的步骤，它的结局不算进这一轮。

「没打中」只各抽一份样本，只当线索，不撑结论。

## 二、判决

逐条，读数都是量的：

| 条 | 读数 | 判决 |
|---|---|---|
| 冻结着 fsync 答成功 | `name=fsync_while_frozen froze_in=PublishWhenDueOfAMaintenancePass after=CrashWithTheDiskStillFailing … fsync_reported_nothing_to_do=true fsync_outcome=Ok(None) frozen_when_the_fsync_returned=true frozen_at_the_crash=true inode_watermark_before=2 inode_watermark_after_the_remount=2`；真线程那一版 `fsync_outcome=Ok(None) … inode_watermark_before=2 inode_watermark_after_the_remount=2`；`AnEarlierFsync` 第二种进法同样 `Ok(None)` | **打中，最重**。违反 `.claude/kb/decisions/16-发布语义.md` 第 215 行「fsync 等系统配置轮换持久之后才返回」。改法见第三节第 1 条 |
| 重发成了之后删除队列没人走 | `name=queue_after_a_resent_unlink watermarks=ProductDefaults action=NothingButAccept … queue_empty_after_the_resend=false passes=20 simulated_seconds=600 accept_went_through_at_pass=None … last_not_started=Some(NotDueByTheWatermarksAtTheTimerFallback)`；`watermarks=OverTheLine` 那一臂 `accept_went_through_at_pass=Some(1)` | **打中**。违反运行库「调用方挂载之后不调任何后台入口」。第 2 条 |
| 满盘加 fsync 零发布 | `name=full_pool_fsync_pass action=UnlinkedOneFile fsync_waiting=true pass=2 simulated_seconds=60 publishes_of_the_pass=1 total_publishes=2 … below_the_floor=false accept_went_through=true`；`action=UnlinkedFourFiles fsync_waiting=true pass=1 … total_publishes=0 … accept_went_through=true`；`name=full_pool_on_the_thread threads_calling_fsync=1 … accept_after_measuring_went_through=true`，`threads_calling_fsync=2` 同样 | **撤回**，攻方跑前的翻面条件「40 拍之内 `accept_went_through=true`」打中。删 1 个文件那一臂此刻 `below_the_floor=true`，破例救的；删 4 个那一臂写本来就进得去 |
| 低于底额让路那一趟不留着 | 同上 `UnlinkedOneFile` 第 2 拍（60 模拟秒）写得进；`TruncatedFilesToHalf` 那一臂 `below_the_floor_after=false`，并入上一条，第 1 拍写得进 | **撤回**（翻面「头两拍写得进」）。限 320 MiB、240 段这一个池 |
| 满盘撤回「运行库救得出来」（第一轮） | `name=defense_arm files_unlinked=1 passes_before=0 size=thirty_two_units accept_went_through=false`，`passes_before=1 size=thirty_two_units accept_went_through=true … pending_published=[ok published_something=true]`；`files_unlinked=4 passes_before=0 size=thirty_two_units accept_went_through=true … pending_published=[ok published_something=true]` | **站得住，比第一轮写得实**：删 1 个文件是整理救的，32 单元也收得下、发得成；删 4 个不用整理。辩方「功劳分不清、只试一单元」的质疑被它自己写的翻面条件解了。限这一个池 |
| 卸载那句文档 | `name=unmount_resend_failed failed_at=TheLastBarrierOfThePublishAfterTheRootLanded … verdict=frozen_publish_resend_failed inode_watermark_before=2 inode_watermark_after_the_remount=3`；`TheFirstWriteOfThePublish` 那一行 `=2` | **打中（文档）**。行为不改，第 3 条 |
| 重发成了不算一次用户改动 | `name=resent_user_change_budget write_fails_once=true … frozen_publishes_resent=1 compaction_publishes=0 publishes_since_the_last_user_change=1000 last_not_started=Some(PublishBudgetOfTheLastUserChangeSpent)`，`false` 那行 `compaction_publishes=10` | **打中（轻）**。第 4 条 |
| 前台 unlink 整段持锁 | `name=lock_hold_of_a_queue_walk walked_by=foreground_publish_user_change publishes_in_total=41 most_publishes_between_two_holds=41`，`background_entry` 那行 `=2` | **不在这一批**。不违反条款（D8 定了一次调用走完），拆入口是持锁时长的优化，登进里程碑四优化项，归优化里程碑 |
| 重发没有退避 | `name=resend_without_a_backoff data_units=1 … writes_of_the_pass=28 bytes_of_the_pass=524288` | **不改**。原位同字节，盘上状态无害；前台在这期间本来写不进，重发越快恢复越快。写进 D26 射程 |
| 余量取 S | S=5、K=16：第 3 次挂载 `states_after_the_mount=3`，第 4 次 1；S=6、S=7：第 4 次 3；S=8：第 5 次 3。`margin_until_the_stop` 只有 `s=5 batch_units=4` `intents_completed=0 live_intent=true` 一行走不完 | **照旧（用户定的）**。攻方算术表「S<8 第 1 次就穿」被读数改掉：这几条历史都在第 3 次之后。辩方提的 `min(8, 3S−9)` 在 S=6、7 上多保一点，默认 S=8 上两者相同，不为它重开。批大小小于 16 走不完不是死局（一次用户改动重开），写进 D26 射程。这几条历史崩溃时都不带满额活意图，不算最坏 |
| 两份删除次序 | 今天三臂相同（两腿读码） | **采**：下沉成一个函数，第 5 条 |
| 建实例 panic 占额度 | `name=opener_maker_panics opening=5 outcome=Some(InstanceLimitOfThisProcessReached { instances_created: 4 }) instances_created=4` | **改**：panic 没占住实例，额度退回，第 6 条 |
| 开卡器并发 | `name=opener_makers_in_flight most_in_flight=1 … limit=4`；`name=opener_several_at_once_summary deadline_milliseconds=200 sorted_waits=[151, 303, 454, 605]` | 上限没打中。排队超期限是旧行为、低危，文档写明，第 6 条 |
| 卸载重发失败交回运行库（辩方） | 读码，没量 | **不采**：结局是重挂恢复，不错，只是多一次挂载 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | 成立（两腿 9 处逐处追过） |
| 类型与名字 | **不成立**：停因 `YieldedToAForegroundFsync` 兼表停线程；枚举名 `ForegroundFsync` 没跟着变。第 7 条 |
| 不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程 | 成立（两腿一致）。wrong-card 登欠账，不在这一批 |
| 带 --item 的命令与项 | 崩溃点重放项成立；形式化项由主 agent 现列，这一批没碰被验区域就报没有可点的项 |
| 新崩溃流多罩了什么 | 删除一步一放、破例前移不加流，加小用例（第 5 条）。**重发变成常态而没有流：采，加一条冻结、重发中途崩的崩溃流**，第 8 条 |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-mount-runtime/src/lib.rs` | **要改**（fsync 先重发、卸载文档、读数名） |
| `crates/singlefs-core/src/compaction_plan.rs` | **要改**（停因、枚举改名） |
| `crates/singlefs-core/src/mounted_session.rs` | **要改**（fsync 查冻结、队列算起跑、次序下沉、重发记账） |
| `crates/singlefs-core/src/mount.rs` | 照旧 |
| `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` | **要改**（panic 退额度、文档） |

## 三、改法（下一批的条款）

1. **fsync 不许在冻结时答成功。** 核心层 `publish_pending_now` 进门查冻结，冻结着就拒（不走「待发空答 `None`」）。运行库的 `publish_pending_now` 持锁时冻结着先重发一次：发成了照常往下，答成功；没发成交回错误。验收：`fsync_while_frozen` 四格与真线程那一格 `fsync_outcome` 是 `Err` 或重发成了之后水位多 1，不再有「答成功而崩溃后水位不涨」。
2. **队列不空算起跑。** 起跑判据里删除队列不空就起跑、只走队列不发整理；不看水位、不被预算挡。验收：`queue_after_a_resent_unlink watermarks=ProductDefaults action=NothingButAccept` 第 1 拍或第 2 拍写得进。两次故障叠加（写盘报错加写满）时删除步被空间拒：实现员造用例量，走不动而整理也轮不到就停下交主 agent，不自己定。
3. **卸载文档**改成「那次可能成了也可能没成，读现行根判」（D16 已定项 17），运行库两处同改，配用例钉 `TheLastBarrier…` 水位多 1。
4. **重发成了记账。** 冻结时记那次是用户改动还是后台批，重发成了照种类记账：用户改动那种重给预算、清上一条完成的意图（D26 已定项 13 第 6 条）。
5. **删除次序下沉**成 `transaction.rs` 里一个选下一步的函数，两处都调；加小用例：unlink 进队列、走一步后插一次前台 `publish_user_change`，终态与一次走空逐字节相同；另两种队列（销毁中的快照、截断）各走一次至多一步的入口。
6. **开卡器**：建实例 panic 退额度；文档写明多个挂载同时开卡时后到的排队，总等待可超期限。
7. **名字**：停因拆出停线程那一种；枚举 `ForegroundFsync` 改成说出它是「这一批之前调用方的回答」的名字，全仓同改；`observations` 的 `compaction_publishes` 把删除步分开数或改名。
8. **新崩溃流**：一次发布写盘报错、冻结，重发，在重发的每个写上崩。不重开挂载之外的步骤，登记 `crash-case:` 与崩溃点重放合表，正文写明比已有流多罩的状态。

## 交用户的

- 第 1、2、4、8 条是腿提的形态，被攻过零轮，第三轮攻。
- 第三轮是这一题的最后一轮，之后不再开。

## 这一轮被判过的文件

- crates/singlefs-mount-runtime/src/lib.rs：判了 fsync 先重发、卸载文档与读数名，要改
- crates/singlefs-core/src/compaction_plan.rs：判了停因与枚举改名，要改
- crates/singlefs-core/src/mounted_session.rs：判了 fsync 查冻结、队列算起跑、次序下沉与重发记账，要改
- crates/singlefs-core/src/mount.rs：判了这一批的改动，照旧
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了 panic 退额度与文档，要改

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 7 | 不影响 | 2026-11-09 不受影响：代码没守它，改代码，条款不动 |
| D16（发布语义） 已定项 17 | 不影响 | 2026-11-09 不受影响：运行库文档把「可能成了」写成「没成」，改文档 |
| D26（后台整理与放置回收） 已定项 13 | 改了 | 2026-11-09 改了：队列不空算起跑、重发成了照种类记账、重发不退避、批大小小于 16 的边界写进射程 |
