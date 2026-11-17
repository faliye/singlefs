# m4-gpugc-mount-runtime-code-r2：攻方腿（2026-11-09）

**这条腿没编、没跑（子 agent 不交任务）：探针只过了 `rustfmt`（语法）。下面每个带 `name=` 的读数都是「推的，待主 agent 跑」；读码的结论写「读码」。** 翻面条件写在跑之前，结果反过来照翻。

复跑命令见「要主 agent 跑的命令」。模型目录 `research/prompts/m4-gpugc-mount-runtime-code-r2-opus-model/` 每个文件的 sha256 见本节末。引代码的行号以冻结副本为准；`mounted_session.rs`、`compaction_plan.rs`、`mount.rs`、`transaction.rs`、`gpu_opener_of_this_process.rs`、运行库 `lib.rs` 与主工作区此刻逐字节相同（`cmp`、`sha256sum`，量的）。

```
a663f778a3b2d0f3769002294bebcfce3103b6a4e61a5f3f71d7a53c6944ab7c  src/attack_r2_full_pool.rs
aa83a8d3669989a101b41f41918de778a315520bd23247d565512ea14ed9e936  src/attack_r2_gpu_opener.rs
0e158b90fdd92f51c8709594d991a5ee09478c3013ee3f5d9cf3f2eb742055e1  src/attack_r2_probes.rs
2fd7c6f44725887272e817b85f036116d4992ad56231477a2b76fd60634413f6  tools/make_the_small_pool_fixture.py
6a2fdaa0c48f7601b1c804c67f790494be3c3a169993f01b019eab2f911a691f  tools/patch_the_fixture.py
407e31a9625e258ff43ad76278f6fcda0060cda6b06b809cbeb4fb859b72a48b  tools/rerun-from-a-clean-work-directory.sh
39c578f139a7df0275590749cebb715adad19bc4a84349431b56e1c63b43765f  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 证据状态 |
|---|---|---|
| T3 冻结着 fsync 答成功（**最重**） | 打中：待发已并进冻结的那一次时 `publish_pending_now` 答 `Ok(None)`，之后崩溃那次改动没了 | 读码确定；`name=fsync_while_frozen` 推的，待主 agent 跑 |
| T3 重发成了之后删除队列没人走 | 打中：unlink 意图写盘报错一次、重发成了，产品水位下维护线程不走队列，`accept` 一直被拒 | 读码确定；`name=queue_after_a_resent_unlink` 待跑 |
| T1 破例排在让路之前 | 打中（够不着）：全空段不低于底额而写不进时一个破例都没有，前台 fsync 一直在等整理就零发布 | `name=full_pool_fsync_pass` 待跑，第一轮撤回的那一格加上 fsync |
| T1 让路那一趟不留着（实现员补的） | 打中（轻）：低于底额加前台 fsync 时每个周期一次发布，一条意图约 150 秒；第一轮「不改」的依据没量过这一臂 | 同上，`action=TruncatedFilesToHalf` 待跑 |
| T1 卸载先重发 | 打中（文档）：`FrozenPublishResendFailed` 写「没落盘、回到之前那一版」，根已落盘时不成立，也与 D16 已定项 17 相反 | `name=unmount_resend_failed` 待跑 |
| T1 重发成了不算一次用户改动 | 打中（轻）：`B_gc` 不重给、上一条完成的意图不清 | 读码；`name=resent_user_change_budget` 待跑 |
| T1 重发没有退避 | 量代价，不算打中：盘持续坏时每个 P 整份写一遍 | `name=resend_without_a_backoff` 待跑 |
| T1 余量取 S | 不算新打中：S 小于 8 时最坏一次崩溃就掉到 4 以下，文档自己写了；只缺读数交用户 | `name=margin_crash_loop` 待跑 |
| T1 删除走空放锁之后前台插进来 | 没打中语义；`accept` 被拒的窗口变长（见下） | 读码 |
| T1 开卡器 | 没打中上限；并发开卡总等待可超期限（低危，没改过的旧行为） | `name=opener_*` 待跑 |
| T3 放锁之外的路 | 打中（轻）：前台 unlink 一次调用整段持锁走完队列，修补碰不到这条路 | `name=lock_hold_of_a_queue_walk` 待跑 |
| T3 两份次序 | 今天逐字相同，没有反例；没有用例拦 | 读码 |
| T3 `observations()`、`CallerIsStoppingItsThread` | 没打中 | 读码 |
| T2 | 只报一条：第一轮一条腿把 fsync 冻结时的行为读错了 | 读码（T2 主攻归辩方腿） |
| 走不到的理由二次查实 | **成立**（9 处逐条核过；另 1 处判决外的 `expect` 也走不到） | 读码 |
| 类型与名字 | **不成立**（一处）：停因 `YieldedToAForegroundFsync` 照旧兼表停线程；水位读数层仍是裸 `u64` | 读码 |
| 不兼容旧代码与旧格式 | **成立** | `grep` |
| 不新建判定库 | **成立** | 读补丁 |
| 所有任务多机可用 | **成立**（wrong-card 不必这一批解决，理由见节内） | 读码 |
| 测试优先多线程 | **成立** | `grep` |
| 带 --item 的命令与项 | 崩溃点重放项**成立**；形式化项**推不出**（项名现算，不跑列不出） | 读登记 |
| 新崩溃流多罩了什么 | **推不出要不要流**；判「删除走空一步一放」不用加，「冻结着 fsync」要加用例（不是流） | 读码 |

## T3：冻结着 fsync 答成功（最重）

历史（每一步都不用只供测试的开关，`name=fsync_while_frozen_on_the_thread` 是真线程真钟那一版）：

1. 前台 `accept` 收下一次改动，留在待发（`crates/singlefs-core/src/mounted_session.rs:6380`）。
2. 过了 `T_time`，维护线程 `publish_when_due` 发它，写盘报错，发布冻结；待发并进冻结的那一次、清空：「冻结了时待发并进了冻结的那一次」（`crates/singlefs-core/src/mounted_session.rs:6296`），报错只送事件接收方（`crates/singlefs-mount-runtime/src/lib.rs:485`）。调用方没收到任何错误。
3. 调用方 fsync：`crates/singlefs-mount-runtime/src/lib.rs:854` 进 `crates/singlefs-core/src/mounted_session.rs:6578`，直接到 `crates/singlefs-core/src/mounted_session.rs:6303`：「if self.pending.is_empty() {」，交回 `Ok(None)`。这条路上没有查冻结的那一判；`accept` 有（`crates/singlefs-core/src/mounted_session.rs:6344`）。
4. 调用方读成「这次 fsync 成了」。盘还坏着，这一次重发也失败，机器掉电或进程崩：那次改动没落盘。

第二种进法：第一次 fsync 自己撞上写盘报错、收到错误（待发同样清空），同一线程重试或别的线程的 fsync 答 `Ok(None)`。

对不上的是：「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:215`）。这一格 fsync 返回时那次发布冻结着，没持久。

反向核：`crates/singlefs-mount-runtime/tests/` 与 `crates/singlefs-verification/tests/publish_triggers_by_time_and_dirty_journal_bytes.rs` 里冻结之后只试 `accept`、不试第二次 fsync（`grep -n 'publish_pending_now\|frozen'`，范围就是这两处）。第一轮正推腿写的是：「publish_pending_now 与 publish_user_change 同样被拒（它们走同一条发布路径）」（`research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-output.md:112`）——待发空时走不到发布路径，这句对 `publish_pending_now` 不成立。

判别量：`fsync_reported_nothing_to_do=true`、`frozen_when_the_fsync_returned=true`、`CrashWithTheDiskStillFailing` 那两行 `inode_watermark_after_the_remount` 等于 `inode_watermark_before`。对照：`DiskHealsThenAPassThenCrash` 那两行水位多 1（重发修得好时不丢，所以打中的是「盘没修好就崩」这一窗）。

**推翻条件**：`fsync_outcome` 是 `Err`，或崩溃之后水位多 1。

四句：分辨臂（只有 fsync 这一个入口中，`accept`、`publish_user_change` 都拒；窗口是冻结到重发成功之间，盘持续坏时没有上界）；系统看得到（`frozen_publish()` 就在分配器上，`crates/singlefs-core/src/mounted_session.rs:6344` 已经在读）；分句是 D16 已定项 7「fsync 等…持久之后才返回」；改法：`publish_pending_now` 进门查冻结，冻结着就拒（新的拒绝成员，或走 `UserChangeRefused` 已有的冻结那一种），这样修这一格。修在核心层，不是运行库。只在我的模型上量过（其实还没量）、被攻过零轮；改法那一格是「推的，待主 agent 跑」。

修补的关系：第一轮的「每拍先重发」把窗口从周期缩到 P，盘持续坏时窗口照旧。这一格不是这一批带来的，是这一批之前就有、第一轮读错了没人攻。

## T3：重发成了之后删除队列没人走

历史（不用只供测试的开关，水位取产品默认）：

1. `publish_user_change(Unlink)`，意图那次发布写盘报错一次，冻结（`crates/singlefs-core/src/transaction.rs:1598`）。调用方收到错误。
2. 下一个 P 重发成了（`crates/singlefs-mount-runtime/src/lib.rs:522`）：意图落盘，文件进 `deleted_inodes`，走空那几步这次调用没走到。事件文档写：「一批写盘报错把发布路径冻结了，这一次重发成了：前台的写从此放行。」（`crates/singlefs-mount-runtime/src/lib.rs:172`）
3. 之后每一次 `accept` 数据写被拒：`crates/singlefs-core/src/mounted_session.rs:6423`，成员 `QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`。
4. 维护线程不走队列：起不起跑只看「该跑了」与水位（`crates/singlefs-core/src/mounted_session.rs:3345`、`crates/singlefs-core/src/mounted_session.rs:3371`），队列不是起跑条件；走队列在起跑之后（`crates/singlefs-core/src/mounted_session.rs:4319`）。产品水位没越线的池上 `NotDueByTheWatermarksAtTheTimerFallback`，一直不走。

用户动作放开扫（`action=`）：只 `accept`（一直拒，20 拍 600 模拟秒）；`publish_user_change`（走空，放行）；经 `accept` 交屏障（走空，放行）；重挂（挂载续做走空，放行）。水位越线那一臂（`watermarks=OverTheLine`）一拍走空：所以打中的是「水位没越线」这一半。

判别量：`watermarks=ProductDefaults action=NothingButAccept` 那一行 `accept_went_through_at_pass=None`、`queue_empty_at_the_end=false`、`last_not_started=Some(NotDueByTheWatermarksAtTheTimerFallback)`。

**推翻条件**：那一行 `accept_went_through_at_pass=Some(_)`，或意图那次没冻结（`frozen_after_the_unlink=false`，那就是探针没摆对，不算推翻也不算打中）。

四句：分辨臂（水位越线不中）；系统看得到（`deletion_queue_of_the_version_is_empty` 就在 `crates/singlefs-core/src/mounted_session.rs:2165`）；分句：运行库文件头「调用方挂载之后不调任何后台入口」（`crates/singlefs-mount-runtime/src/lib.rs:2`）加事件文档那一句；只用 `accept` 的调用方在这里出不来。改法：甲，起跑那一判把「队列不空」算起跑、只走队列不发整理（预算那一判也要让开，实现员报告已点到预算这一半）；乙，运行库重发成了之后自己走一遍队列；丙，`accept` 碰到队列不空时自己走空（语义变成 accept 里发布）。甲修这一格、也修预算扣完那一半；乙只修重发这条路，修不了 `Failed(DeletionContinuation)` 之后的队列。都没实现，推的，待主 agent 跑，被攻过零轮。

同一族的三处（实现员报告留的「预算扣完而队列不空」就是其中之一，读码）：删除续做被拒之后这一趟不留着（`Failed` 不进 `crates/singlefs-core/src/mounted_session.rs:3498` 那一支）；预算扣完不起跑（`crates/singlefs-core/src/mounted_session.rs:3405`）。这两处水位越线时也中。

低于底额再叠一层（读码，没量）：起跑之后删除续做排在整理之前，后台这一步不推抬 F（`crates/singlefs-core/src/mounted_session.rs:2083` 与 `crates/singlefs-core/src/mounted_session.rs:2104` 都不推，前台那条路 `crates/singlefs-core/src/mounted_session.rs:5751` 才推）。删除步被以空间拒时 `crates/singlefs-core/src/mounted_session.rs:4344` 直接 `Failed`，走不到整理，破例也轮不到。要两次故障叠加（写盘报错加写满），这一格我没造，写成线索。

## T1：破例排在让路之前，够不着「不低于底额而写不进」

第一轮撤回「只用 accept 写的走不出来」靠的读数：`full_pool_pass topped_up=true files_unlinked=1 pass=1 total_publishes=10 … empty_by_the_records=5 openable=false accept_went_through=true`（量的，`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-run/readings.out`，第一轮的代码）。这一拍没有前台 fsync。

这一格加上 fsync（`action=UnlinkedOneFile fsync_waiting=true`，用户动作与第一轮同一段）：

1. 记录口径全空段 5，不低于 `N₀ = 1`（`crates/singlefs-core/src/compaction_plan.rs:1883`），`foreground_cannot_write` 假（`crates/singlefs-core/src/mounted_session.rs:3317`），这一趟不带破例。
2. 放掉的槽在 defer 里，开不成段（`openable=false`），`accept` 被拒。要环转过去才开得成，转环靠整理发布。
3. 每一批之前问 fsync：`crates/singlefs-core/src/mounted_session.rs:4443` 那一支让路，一次都不发。待发空的 fsync 也算（`crates/singlefs-mount-runtime/src/lib.rs:860`）。第一轮量过一条线程反复空 fsync 就 `compaction_publishes=0`（`research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md:26`）。
4. 只要前台 fsync 一直在，整理一直零发布，`accept` 一直拒。

判别量：`fsync_waiting=true` 各行 `total_publishes=0`、`accept_went_through=false`，走满 40 拍；`full_pool_after_the_fsync_left` 第一拍就写得进。真线程那一臂 `name=full_pool_on_the_thread threads_calling_fsync=1`：`publishes_measured` 接近 0，`accept_after_measuring_went_through=false`，线程数 0 的对照发得动、写得进。

**推翻条件**：`fsync_waiting=true` 的行 40 拍之内 `accept_went_through=true`；或真线程那一臂 `threads_calling_fsync=1` 与 0 没差别。

四句：分辨臂（没有 fsync 就不中，有 fsync 不论几条都中）；系统看得到（`lowest_empty_segment()` 就在同一个分配器上，读数 `openable` 就是它；破例判的是记录口径，这是判据选错量，不是看不到）；分句：「只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）。「低于底额」这个合取不满足，破例字面不来：**代码照条款写的，洞在条款**，第一轮攻方报告已写过「这一格不低于底额，条款没罩到」，判决因满盘探针发得动而撤回；探针没摆 fsync。这一条是撤回的条件变了，不是重复。

改法与它们在这一格上（都没实现，推的，待主 agent 跑，被攻过零轮）：

| 改法 | 这一格 | 别的格 |
|---|---|---|
| 甲：破例判据加「开不成段」（`lowest_empty_segment().is_none()`） | 推的：修 | 要改 D26 已定项 13 第 5 条；回退门在这一格也被绕过 |
| 乙：让路设上限（连续让路 N 次发一批） | 推的：修 | fsync 多等一批；不低于底额也生效 |
| 丙：待发空的 fsync 不登记「在等」 | 推的：修空 fsync 这一臂 | 有数据的 fsync 照让；要在取锁之前知道待发空不空 |
| 第一轮的「破例排在让路之前」 | **不修**（不低于底额，破例不来） | 低于底额那一格它修了 |

### 让路那一趟不留着（实现员补的）

低于底额（`action=TruncatedFilesToHalf`，放开「删」换成「截短」：有洞、记录口径没有全空段；截短要是把全空段也做出来，这一臂就退回上一格，读数 `below_the_floor` 分得出）加 fsync：第一次调用破例发 1 次（`crates/singlefs-core/src/mounted_session.rs:4442`）、第二次调用不是破例、让路、这一趟不留（`crates/singlefs-core/src/mounted_session.rs:3487`），`is_a_trip_under_way` 假，线程隔一个周期再来（`crates/singlefs-mount-runtime/src/lib.rs:613`）。不低于底额时同样的让路隔 P 再来。低于底额时前台更急，等得反而比不低于底额时久 30 倍（默认 30 秒对 1 秒，推的算术）。

一条意图至多 5 次发布，头一次只签意图不搬东西：约 150 模拟秒才腾出一段，还要转环。第一轮判「不改」的依据是：「前台可达的情形由满盘探针量过第 1 拍就过」（`research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md:35`），那个探针没有 fsync，也没有低于底额（读数 `empty_by_the_records=5`）。这一臂是这句够不着的地方。

判别量：`fsync_waiting=true` 各拍 `publishes_of_the_pass=1`、`simulated_seconds` 每拍涨 30、`accept_went_through` 第一次为真在第几拍；同一 `action` 的 `fsync_waiting=false` 是对照。

**推翻条件**：`fsync_waiting=true` 头两拍写得进；或这一臂 `below_the_floor=false`（那就只是上一格）。

四句：分辨臂（没 fsync 时一拍推到门）；系统看得到；分句「每趟至少发 1 次」字面满足，条款没写多久一趟；改法：低于底额让路时 `is_a_trip_under_way` 报真（隔 P 不隔周期）。代价：破例挤状态从每 30 秒一次变每秒一次，低于底额时根环转得快 30 倍（推的，被攻过零轮）。第一轮攻方提过同一个改法；这里补的是它在 fsync 这一臂才要紧。

**T1 问的反方向「破例会不会一直抢锁」**：没打中。一趟只破例一次，新一趟要等定时兜底或一次用户改动（`crates/singlefs-core/src/mounted_session.rs:3345`）；用户改动触发的那一趟发的那一次与用户改动一比一，没有放大（读码）。

## T1：卸载先重发之后的状态

修补的行为对（`unmounting_with_a_frozen_publish_*` 两条入库用例）。打中的是它写下的那句话：「卸载先重发冻结的那一次，重发又报错：盘交回，内存不接着用，等价于崩溃（冻结的那一次没落盘，下一次挂载走恢复，回到它之前最后发成的那一版）。」（`crates/singlefs-mount-runtime/src/lib.rs:302`）

历史：一次发布写到最后一道屏障报错（根槽 FUA 已成、系统配置已轮换，`crates/singlefs-core/src/transaction.rs:1660` 到 `crates/singlefs-core/src/transaction.rs:1668`），冻结；卸载重发，屏障又报错，`FrozenPublishResendFailed`；盘修好再挂，恢复读到那一次的根。

判别量：`name=unmount_resend_failed failed_at=TheLastBarrierOfThePublishAfterTheRootLanded` 那一行 `inode_watermark_after_the_remount` 比 `inode_watermark_before` 多 1（文档说不多）；`failed_at=TheFirstWriteOfThePublish` 是对照，不多。

**推翻条件**：那一行水位不多 1。

条款是：「fsync 报错之后，这次发布的数据可以出现也可以不出现」（`.claude/kb/decisions/16-发布语义.md:376`）。文档把「可能成了」写成「没成」，正是 D16 已定项 17 依据里点名的那种读法。调用方照文档想「那次改动没了」、重做一遍，就是重复施加。

四句：分辨臂（根落盘之后才中）；系统看不看得到：运行库这一刻分不清（要读现行根，已定项 17 写了判据），所以改的是文档，不是行为；分句是已定项 17 第一句；改法：文档改成「可能成了也可能没成，读现行根判」，第 973 行同改。只改注释，没量。

走过的别的形状（读码，没中）：卸载第二次调用（`PendingChangesNotPublished` 交回的运行库上）`of_the_session` 新建一份 `Shared`，`stopping` 是假（`crates/singlefs-mount-runtime/src/lib.rs:819`），`run_one_maintenance_pass` 照走整理，「腾出空间再卸载」走得通；观测计数清零（`crates/singlefs-mount-runtime/src/lib.rs:823`），文档没写，低危。

## T1：重发成了不算一次用户改动

历史：闲置池上预算扣完；一次用户改动写盘报错、冻结；下一拍重发成了，那一版就是一次用户改动的结果。`resend_the_frozen_publish`（`crates/singlefs-core/src/mounted_session.rs:3222`）不调 `note_a_user_change_published`（`crates/singlefs-core/src/mounted_session.rs:2958`）也不清上一条完成的意图（`crates/singlefs-core/src/mounted_session.rs:2963`）。文档只说整理的计数差一次。

对不上：D26 已定项 13 第 6 条「每次用户改动发成就清掉它」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）；重发成了算不算「发成」条款没写，我读成算（用户那次改动此刻就在现行那一版里）。

判别量：`name=resent_user_change_budget write_fails_once=true` 比 `write_fails_once=false` 少发（`compaction_publishes`、`publishes_since_the_last_user_change`、`last_not_started=PublishBudgetOfTheLastUserChangeSpent`）。

**推翻条件**：两行发布数相同。

四句：分辨臂（只有冻结过的用户改动中）；系统看不看得到：看不到——冻结的那一次是用户改动还是整理，`FrozenPublish` 没记（`crates/singlefs-core/src/transaction.rs:1536`），要记就要加字段；分句见上；改法：冻结时记一个种类，重发成了照种类记账。低危：下一次用户改动就补回来。推的，待主 agent 跑。

## T1：重发没有退避

盘持续坏时每个 P 重发一次（`crates/singlefs-mount-runtime/src/lib.rs:619`），整份单元照写一遍，一次失败一个事件。不算打中：字节不变、位置不变（`crates/singlefs-core/src/transaction.rs:1625`），对盘上状态没有害；前台在这期间本来就写不进。代价只有三样：写量、持锁、事件量。

`name=resend_without_a_backoff` 量每拍写的次数与字节（第一道屏障报错的情形：单元全写过一遍才报错）。推的：`data_units=32` 每拍约 2 MiB（32 单元 × 32 KiB × 2 块盘），默认 P 下每小时约 7 GiB（算术，待跑）。写第一笔就报错的盘写量是 0。持锁：每次重发持会话锁写完一整份（`crates/singlefs-mount-runtime/src/lib.rs:492` 到 `crates/singlefs-mount-runtime/src/lib.rs:498`），读路径同一把锁，盘慢时读每秒被挡一次（读码，没量）。

**推翻条件**：`writes_of_the_pass` 是 0（那就是这个形状摆错了）。

要不要退避交主 agent：条款没定（实现员报告已列）。前台被拒期间重发快慢决定恢复多快，退避换来的只是写量。我的判断是可以不加，若加只退避写量，不要超过几秒。被攻过零轮。

## T1：余量取 S（S = 5、6、7、8，K = 16、8、4，连崩）

门放 `2S − 4` 次（`crates/singlefs-core/src/compaction_plan.rs:1532`）。文档自己写了：「S = 8 时一次挂载落在余量里；S 更小时一次挂载就可能用完余量。」（`crates/singlefs-core/src/compaction_plan.rs:1534`）

`name=margin_until_the_stop`、`name=margin_crash_loop` 量门关之前发几次、意图做不做得完、门关着带活意图崩溃之后每次挂载还剩几个状态。推的（读码算术，待跑）：

| S | K=16 门放 | K=16 一条意图（整段上界 5 次） | K=8（上界 9 次） | K=4（上界 17 次） | 最坏第几次崩溃掉到 4 以下（按一次挂载发满 8 次算） |
|---|---|---|---|---|---|
| 5 | 6 | 走得完，第二条半路门关 | 走不完 | 走不完 | 第 1 次（挂载至多 8 次发布，余量 5） |
| 6 | 8 | 走得完 | 走不完 | 走不完 | 第 1 次 |
| 7 | 10 | 走得完 | 走得完 | 走不完 | 第 1 次（8 > 7，差 1） |
| 8 | 12 | 走得完 | 走得完 | 走不完 | 第 2 次（第一轮 S = 8 实量是第 4、5 次，见判决 `crash_loop` 那一行：一次挂载发不满 8 次） |

入库用例只扫 K = 16 走不走得完（`crates/singlefs-mount-runtime/tests/mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs:349`），K 是挂载参数（`crates/singlefs-mount-runtime/src/lib.rs:65`）。走不完不是死局：一次用户改动门就重开（第一轮正推腿读码）；闲置时活意图一直占着 R。

**不算新打中**：用户定「余量取 S」时第一轮判决已写「崩溃循环总有尽头」是接受的边界。要交用户看的只有一个数：S = 5、6、7 上最坏一次崩溃就穿（实际第几次看读数），不是 S = 8 上量到的那个样子。第一轮的崩溃循环读数都是 S = 8（`gate_close`、`crash_loop`）。

**推翻条件**（表里第一次穿的那一列）：`states_after_the_mount` 第一次低于 4 的挂载号与表不同，就照读数改。挂载实际发几次取决于带不带活意图、续做几步，表里是上界。

## T1：删除走空放锁之后前台插进来

造的交错（读码，每一条入口都核了）：步与步之间前台 `publish_user_change`、经 `accept` 的屏障：先走空（`crates/singlefs-core/src/mounted_session.rs:3570`），再做自己的；队列比后台走得快，不冲突。`accept` 数据写：拒（`crates/singlefs-core/src/mounted_session.rs:6423`）。`publish_pending_now`：待发是空的（队列不空时收不进待发），答 `Ok(None)`，这里答得对。截断步的 `names_the_live_record`（`crates/singlefs-core/src/transaction.rs:12508`）：前台改 size 要先走空，碰不到。步之间读：读到中间版本，那些版本本来就是合法状态。**没打中语义。**

一处行为变化（读码）：改之前前台在整段走空期间等锁、等到就过；改之后这期间 `accept` 被拒。窗口只在队列不空时才有，队列不空只从崩溃或写盘报错来（挂载走空、前台走空，见上一节）。

`deletion_queue_of_the_version_is_empty` 为假而 `publish_at_most_one_step_of_the_queue` 选不出一步、空转把这一趟留着：两边判的是同三样（`crates/singlefs-core/src/transaction.rs:4134`、`crates/singlefs-core/src/mounted_session.rs:2115`），走不到。

删除步不问 fsync、不受写带宽：**不算打中**，条款写明：「它只约束整理批（每一批之前问调用方一次），整理开头先走的那一遍删除不是整理批，不受它约束」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。

## T1：开卡器

`name=opener_makers_in_flight`（8 条线程反复开卡、丢实例，期限 1 毫秒）判别量：`most_in_flight=1`、`instances_created` 不超过 `limit=4`。读码：闸与 `one_opening_at_a_time` 两道，同一时刻至多一条工作线程，推的 `most_in_flight=1`。**推翻条件**：`most_in_flight` 大于 1。

`name=opener_maker_panics`：建实例 panic 也用掉一个额度（额度记在建之前，`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:191`），第 5 次开卡报 `InstanceLimitOfThisProcessReached`。这是文档写的取舍（挂住也算用了一个）；挂住的确实占着实例，panic 的不占。低危，不算打中。

`name=opener_several_at_once`：`OpensTheBackgroundGpu` 文档写：「期限、闸、进程级实例与泄漏账都在实现里；这里只收到一个结局，所以这一次调用要在实现自己的期限之内返回。」（`crates/singlefs-mount-runtime/src/lib.rs:151`）四次挂载同时开卡、每次 150 毫秒（都不超 200 毫秒期限）：后到的排在 `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:210` 上，从调用起算等到约 600 毫秒（推的）。这一行是旧行为，这一批没动。低危。**推翻条件**：`sorted_waits` 最大值不超过 200。

放闸之后的二次开卡、建实例途中丢实例：入库用例量过（实现员报告证红表），我没另造。`of_this_process` 建两次就是两份额度与两道闸（计数在对象上，`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:138`），文档写「一个进程建一份」，靠调用方守。低危，没量。

## T3：`run_one_maintenance_pass` 之外绕过放锁的路

有一条：前台 `publish_user_change`（`crates/singlefs-mount-runtime/src/lib.rs:876`）与经 `accept` 的屏障。`UserChange::Unlink` 的文档写一次调用走完全部批（`crates/singlefs-core/src/mounted_session.rs:145`），持着会话锁一路走到空（`crates/singlefs-mount-runtime/src/lib.rs:884` 取锁，`crates/singlefs-core/src/mounted_session.rs:3570` 走空）。同样形状还有截断、删快照。第一轮判「删除走空整段持锁」打中，修的是后台那条路：后台那条路只在崩溃或写盘报错之后才有活干（上一节），常态的长队列都走前台这一条。

判别量：`name=lock_hold_of_a_queue_walk`，同一条 40 步的队列（批大小摆成 1 是只供测试的开关，代表一个大文件）：`walked_by=background_entry` 的 `most_publishes_between_two_holds` 接近 1；`walked_by=foreground_publish_user_change` 接近一共发的次数。

**推翻条件**：两行 `most_publishes_between_two_holds` 同一量级。

四句：分辨臂（前台这条路中，后台不中）；系统看得到（走空循环在核心层，放锁在运行库，核心层不持锁，要拆入口）；分句：判决 m4-gpugc-background-runtime-design-r1「每次发布之间放锁」管的是后台与前台经一把锁，字面没说前台自己的多次发布；D8 已定项 16 第 6 条定了「一次调用走完」。**所以我不判它违反条款，判它是「修补碰不到这一格」**：另一个线程的 fsync 与读，在别人 unlink 大文件时等完全程。改法：运行库的 unlink 分步（前台每步放锁，语义不变、返回时走空），要拆核心层入口。代价大，交用户定。推的，待主 agent 跑，被攻过零轮。

别的路（读码）：`with_the_session_and_devices`（闭包持锁，用例与读路径用）；`switch_background_compute`（只记待生效）；`publish_pending_now` 与 `accept` 一次一发；挂载续做在线程起来之前；`session_for_tests_only` 只供测试。`run_compaction_in_the_background` 整趟入口运行库不调（`grep` 核：运行库 `lib.rs` 只调 `_for_at_most_one_publish`）。

## T3：两份次序、`observations()`、`CallerIsStoppingItsThread`

**两份次序**：`crates/singlefs-core/src/mounted_session.rs:2114` 起与 `crates/singlefs-core/src/transaction.rs:12642` 起逐项相同，三臂同形。今天没有反例。没有用例在两份之间拦：实现员报告已写。实现员的变异「整段走到空」换的是整个函数，不罩「只调换次序」。改法：把选下一步下沉成一个函数；代价几行。第三份不同：`crates/singlefs-core/src/transaction.rs:12579` 的预演只走 `deleted_inodes`，文档写「把删除队列从 current 走到空的每一步」；队列里有快照或截断时预演比真走少。这一处不是这一批的，写成线索，没量。

**`observations()`**：单独一把小锁（`crates/singlefs-mount-runtime/src/lib.rs:946`），不取会话锁，发布中途读得到；`update_the_observations` 都在放了会话锁之后（`crates/singlefs-mount-runtime/src/lib.rs:531` 的块结束才记）。`compaction_publishes` 文档写「整理发布次数」，实际数 `versions_published.len()`，删除步也算进去（`crates/singlefs-mount-runtime/src/lib.rs:576`），放锁之后删除步一次一个进这个数。低危：读数名不符。`has_a_background_trip_under_way` 在会话锁里读（`crates/singlefs-mount-runtime/src/lib.rs:537`）。

**`CallerIsStoppingItsThread`**：`foreground_fsync()` 只有一处调用（`crates/singlefs-core/src/mounted_session.rs:4441`），`match` 三臂都写，没有通配（`grep` 范围：`crates/` 下非 `tests/` 的 `foreground_fsync()`）。`e202` 那一处只断言不匹配（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:9322`）。`history.rs` 答的是两个成员之一（`crates/singlefs-verification/src/history.rs:10267`），不答新成员，没有差。这个成员走让路那一支，停因记 `YieldedToAForegroundFsync`；低于底额时这一趟不留、不低于底额时留，反正线程要停了。**没打中。**

**破例还开着时写带宽截住**：`admit_the_next_batch_under_the_write_bandwidth_limit` 在破例之后照判（`crates/singlefs-core/src/mounted_session.rs:4457`），`is_the_exception_of_the_trip_still_open` 留着，下一拍接着破例。这是实现员写明的。写带宽窗口没过之前低于底额也发不出：条款第 6 条的上限，不矛盾。**没打中。**

## T2：只报一条

第一轮正推腿那一句读错了（见「冻结着 fsync 答成功」）；第一轮攻方探针也只试 `accept`（`name=frozen_resend`）。这一格第一轮两条腿都没碰到。满盘探针的外推（320 MiB）归辩方腿；我只补一条：`full_pool_*` 没摆 fsync，见 T1 第一格。

## 检查单

**走不到的理由二次查实：成立。** 实现员报告「走不到的理由」9 处逐条追了调用点，都站得住：

| 处 | 核 |
|---|---|
| `crates/singlefs-mount-runtime/src/lib.rs:576` | 成立：长度到 u64，本仓只跑 64 位 |
| `crates/singlefs-mount-runtime/src/lib.rs:721` | 成立：`spawn` 失败时闭包手里只有接收端，没有 `Arc`；`drop(runtime)` 之后唯一持有者。第一轮「靠标准库丢闭包」那个依赖已没了 |
| `crates/singlefs-mount-runtime/src/lib.rs:987` | 成立：join 过，闭包的 `Arc` 已丢；接收方回调 panic 走展开也丢；`self` 已丢 |
| `crates/singlefs-core/src/compaction_plan.rs:435` | 成立：常量求值 |
| `crates/singlefs-core/src/compaction_plan.rs:1617` | 成立：`.min(整段数)` |
| `crates/singlefs-core/src/compaction_plan.rs:1801` | 成立：`free_runs × per > above × allocated` 推出向下取整的线小于 `free_runs` |
| `crates/singlefs-core/src/mount.rs:4302`、`:4306`、`:4308` | 成立：数不超过 3S |

另一处报告没列、也走不到：`crates/singlefs-core/src/mounted_session.rs:4323`，`deletion_queue_of_the_version_is_empty` 对没有树的一版答真（`crates/singlefs-core/src/mounted_session.rs:2171`），`shared_version_of_trees` 对另两种答 `Some`（`crates/singlefs-core/src/transaction.rs:2513`）。这一处是旧行，判决没点。

**类型与名字：不成立（一处）。** `CallerIsStoppingItsThread` 说得出内容，但它是枚举 `ForegroundFsync` 的成员，枚举名说的是 fsync；判决要改的是「`Waiting` 兼表停线程」，实现员拆开了 `ForegroundFsync`，没拆停因：`CompactionRunStop::YieldedToAForegroundFsync` 照旧兼表停线程（`crates/singlefs-core/src/compaction_plan.rs:620`，文档那一句就写了两样），`is_yielding_while_the_foreground_cannot_write` 在停线程时也真（`crates/singlefs-core/src/mounted_session.rs:3487`）。实现员报告自己写了「名字不全对」。`publish_at_most_one_step_of_the_queue`、`root_ring_slots_left_by_a_compaction_publish`、`EmptyClusterSegmentCount`、`FreeRunCount`、`WatermarkCrossing`：说得出内容。

`WatermarkVerdict` 的单位：结论这一层真分开了（`crates/singlefs-core/src/compaction_plan.rs:1716`）。读数这一层没分：`WatermarkReadingOfADevice` 四个量照旧裸 `u64`（`crates/singlefs-core/src/compaction_plan.rs:1668`），`line_in_segments` 交 `u64`，比较在裸数上。判决点名的是 `Crossed` 那一处，算修到了；读数层是剩的。

**不兼容旧代码与旧格式：成立。** `grep ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH` 冻结副本 `crates/` 与 `.claude/kb/`：0 处。补丁没有认旧版本的分支。

**不新建判定库：成立。** 补丁文件清单（实现员报告「写过的文件」）里没有判定存放处；两个新测试文件不写判定。

**所有任务多机可用：成立。** 真卡用例取 `GPU_CARDS_BY_PRIORITY` 交下来的卡（`crates/singlefs-background-gpu/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs:95`），不绑一台。

wrong-card 判：**不必这一批解决。** 依据（读码）：产品照文档「没参与的卡也开（之后的切换可能点它，切换不重开设备）」（`crates/singlefs-background-gpu/src/background_gpu.rs:837`）；服务只提醒、照跑（`task-server/task_server_lib.py:228`）；别的任务清场回读跳过别的活着的登记（`scripts/crash/gpu-quiesce-run.sh:46`），不会被它拒。真影响：同一刻分到那两张卡的任务显存少 202 MiB（数取背景材料，量的来自任务 11152，我没量）、读数里多一个进程。这一条对量显存或 CPU 时间的实验才要紧（E202 那一类）。要解决就得让开卡只开点名的卡，代价是切换要重开设备，这是产品决定，不归这一批。

**测试优先多线程：成立。** 运行库的六条与真卡一条都有「不切片并行」的理由（`grep -n '不切片并行'`：`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:734`、`:836`、`:868`、`:902`、`:1012`、`:1210`，真卡那一份第 75 行），理由都是「一个池、一条线程、没有可切的区间」。

**带 --item 的命令与项：崩溃点重放成立，形式化推不出。** `.claude/gate.d/crash-point-replay-items.tsv` 里带 compaction 的项在（`compaction-relocation`、`compaction-relocation-on-the-gpu-path`、`compaction-relocation-switching-backends-between-batches`、`compaction-relocating-a-block-shared-with-a-snapshot`、`compaction-deadlist-reclamation`、`compaction-with-a-write-while-the-intent-is-live`，另 `deletion-interleaved-with-compaction-across-a-rollback`）。形式化项由 `--list-items` 现算（`.claude/gate.d/formal-verification.sh:48`），不跑列不出；`mounted_session.rs`、`mount.rs` 里 `grep -n 'verus!'` 没命中，实现员「点盖这两份文件的项」可能一项都点不到：要主 agent 列的时候核。

**新崩溃流多罩了什么：判「删除一步一放、破例前移」不加流。** 一步一放写的字节与次序同整段走空（`crates/singlefs-core/src/mounted_session.rs:2115` 与 `crates/singlefs-core/src/transaction.rs:12643` 同一组发布），多出来的只是步与步之间的前台发布，前台走同一个走空。破例前移只改发不发，不改一次发布里写什么。这两样加小用例就够，实现员加了。推的，没量。

**但这一批让另一条路变常态而没有流**：重发从「整理到期才重发」变成「每个 P、卸载也重发」。`crates/singlefs-verification/tests/` 里崩溃枚举用例名没有带 frozen、resend 的，`crash-point-replay-items.tsv` 里 `grep 'resend\|frozen'` 0 行。重发是原位同字节，单元与记录撕了也是同一份内容（推的），系统配置槽轮换照常（`crates/singlefs-core/src/transaction.rs:1664`）：连续失败时轮换反复写。我判该补一条「冻结、重发中途崩」的流，崩溃枚举归崩溃点重放；我没在原型里枚举（见限度）。

## 没打中的形状

- 破例一直抢锁：一趟一次，新一趟要等定时兜底或用户改动（读码）。
- `publish_at_most_one_step_of_the_queue` 空转：判空与选步判同三样（读码）。
- 两次发布之间前台改队列：所有改动入口先走空；`accept` 拒；`publish_pending_now` 待发空（读码）。试过的入口：`publish_user_change`、`accept` 屏障、`accept` 数据写、`publish_pending_now`、`publish_when_due`、`switch_background_compute`、闭包读。
- 卸载第二次调用：新的 `Shared`，`stopping` 假，整理走得动（读码）。
- 破例留着跨写带宽：照留、下一拍接着破例（读码）。
- `CallerIsStoppingItsThread` 漏 `match`：一处调用、没有通配（`grep`）。
- 开卡器建实例并发：闸加排队锁，至多一条工作线程（读码；`name=opener_makers_in_flight` 取样 8 线程 × 50 轮，待跑）。
- 删除步不让路：条款明写不约束（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。
- 取样范围：池 2 种（4 GiB 两块盘 40 数据单元、320 MiB 写满 173 份左右，后者是第一轮量的份数）；S 取 5、6、7、8；K 取 16、8、4；用户动作每一格放开 3–4 种；崩溃点都在发布边界。

## 这条腿自己的限度

- **一行都没编过、没跑过。** 三份探针只过了 `rustfmt`，API 逐个对着冻结副本 `grep` 过、形状大多抄自能编的入库用例与第一轮探针，仍可能编不过。编不过就新派同立场的腿（带「承接：」）修。
- 崩溃只取发布边界（丢运行库、盘原样留着），段内写的子集没枚举。这一轮打中的前两条不靠段内状态：`fsync_while_frozen` 的报错在第一个写，什么都没落盘；队列那一条没有崩溃。余量那几行是状态个数的上界，不是每个崩溃态都走一遍恢复与 checker。**没调 `enumerate_crash_states`**：要跑就要把验证包接进探针、算状态数；我判这一轮的打中不用它，「冻结、重发中途崩」那条流该由主 agent 决定进不进崩溃点重放。
- 满盘探针沿用第一轮的 320 MiB、240 段，外推到大盘的问题（归辩方腿）我没量。估时（推的）：填盘约 100 秒（第一轮量的 100.8 秒，`full_pool_fill`），6 个按拍的臂每个约 5–15 秒，3 个真线程臂每个 3–10 秒，`attack_r2_probes` 几分钟，加编译，全量远低于 40 分钟。填盘循环里带估时闸，超 40 分钟自己停。没有缩历史或几何。
- `fsync_waiting=true` 用的是把「在等」计数一直持着（`a_foreground_fsync_is_waiting`），是 fsync 一直在的最坏臂；真线程那一臂量的是反复空 fsync。没有量「fsync 占一半时间」这种中间臂：`publishes_measured` 一条线程那一行就是真实比例。周期压到 1 毫秒，量交错不量周期。
- 探针摆的只供测试开关：`CappedForTests(1)`（锁那一格，代表大文件）、预算记成 1000（`resent_user_change_budget`）、强制选段（余量那一格）。前两条打中不用这些。
- 改法都没实现、都没跑，被攻过零轮。
- 只攻了 T1、T3 与检查单；T2 只报一条。
- 「没打中」只抽了这一次样，不够支撑结论（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。

## 要主 agent 跑的命令

**先拷模型目录**（`.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算` 核查第 3 条），在仓根：

```
cp -r research/prompts/m4-gpugc-mount-runtime-code-r2-opus-model /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/opus-model
```

**再交这一条**（在哪个目录起：`--cwd` 给的拷贝；不碰卡）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/opus-model -- bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/run /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check/opus-model
```

跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=` 是 0；日志末行 `name=rerun_exit attack_r2_probes=0 attack_r2_full_pool=0 attack_r2_gpu_opener=0`。读数在 `<草稿目录>/run/readings.out`（全部 `name=` 行），各目标原样输出在同目录 `attack_r2_*.out`。探针不断言结论，退非 0 只说明编不过或挂载 `expect` 炸了：`grep '^error\|panicked at' attack_r2_*.out` 看是哪一种。背景材料说的 `NewerStateStillUnreadableAfterOneReread` 若炸在这里，先看是不是同一个原因。

脚本安全重跑：`rerun-from-a-clean-work-directory.sh` 只在 `--work` 里有 `.attack-rerun-marker` 时清它；`rerun.sh` 拒非空目录。

各格看哪一行、翻面条件：

| 格 | 看哪几行 | 算打中 | 翻面 |
|---|---|---|---|
| 冻结着 fsync | `name=fsync_while_frozen`、`name=fsync_while_frozen_on_the_thread` | `fsync_reported_nothing_to_do=true` 且 `frozen_when_the_fsync_returned=true`；`CrashWithTheDiskStillFailing` 水位不涨 | `Err`，或崩溃后水位涨 |
| 队列 | `name=queue_after_a_resent_unlink`、`name=queue_after_the_action` | 产品水位 `NothingButAccept` 全程拒 | 走得通 |
| 满盘加 fsync | `name=full_pool_user_action`、`name=full_pool_fsync_pass`、`name=full_pool_after_the_fsync_left`、`name=full_pool_on_the_thread` | `UnlinkedOneFile` 与 `UnlinkedFourFiles` 在 `fsync_waiting=true` 下 `total_publishes=0`、40 拍写不进，`false` 臂头几拍写得进 | 40 拍内写得进；或 `accept_before_the_action_went_through=true`（盘没写满，这一臂作废） |
| 低于底额加 fsync | 同上 `TruncatedFilesToHalf` | `below_the_floor=true`，每拍 1 次、每拍 30 秒 | 头两拍写得进；`below_the_floor=false` 则并入上一格 |
| 卸载那句文档 | `name=unmount_resend_failed` | `TheLastBarrier…` 水位多 1 | 不多 |
| 重发不算用户改动 | `name=resent_user_change_budget` | `true` 那行发布数少 | 相同 |
| 重发写量 | `name=resend_without_a_backoff` | 只报数 | `writes_of_the_pass=0` 则形状摆错 |
| 余量 | `name=margin_until_the_stop`、`name=margin_crash_loop` | 只报数，照读数改表 | 表与读数不同就改表 |
| 放锁的路 | `name=lock_hold_of_a_queue_walk` | 前台那行接近一共发的次数，后台那行接近 1 | 同一量级 |
| 开卡器 | `name=opener_makers_in_flight`、`name=opener_maker_panics`、`name=opener_several_at_once_summary` | `most_in_flight=1` 是没打中 | 大于 1 |

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没编、没跑探针；没枚举崩溃状态；没实现任何改法；没写用例；没碰冻结副本；没读实现员补丁目录之外的改动。
- 草稿目录只放了 `model-sha.txt`、`rerun-steps-new.txt`、`cite.txt`，没有仓副本与编译目录。
