# m4-gpu-wait-outside-the-lock-design-r1：判决（2026-11-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 -->

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-design-r1-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-snapshot/`。这是这一题的第一轮（设计推论轮），同一题没有前一轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-output.md` | 三段可写（取段、算段、收段）；读盘放哪、把手锁、CPU 路径拆不拆、戳对不上怎么办、下限各是岔路；活锁「有，破例批上必现」；不新增写序列 |
| 云端攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md` | Q2 弱判据打中、只比 txg 漏掉待发、活锁打中、让路只能在第一段问、锁序打中、卸载照样要等；Q4 严判据下没打中 |

核查：

- 引文：任务 19673 `python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-output.md research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m4-gpu-wait-outside-the-lock-design-r1-background.md`，末行「✓ 核了 71 处引文、159 处裸行号，对上 71 处，没判 159 处（其中裸行号没有引文可比 159 处；逐处列在上面）」。设计轮没有冻结的树，根取主工作区；快照里六份 `crates/` 文件 `git status --short` 量得 0 行，没有未提交改动。没判的裸行号里判决要用的四处主 agent 现读过：`crates/singlefs-core/src/transaction.rs:11170`（txg 取 `previous.root.checkpoint_txg` 的下一个）、`crates/singlefs-core/src/root_ring.rs:172`（根槽只按 txg 算）、`crates/singlefs-core/src/background_compute.rs:1176`（切到 `gpu` 的校验里取 GPU 实现那把锁）、`crates/singlefs-core/src/mounted_session.rs:7528`（「整理是屏障」）。其余裸行号没逐处核。
- 攻方探针：任务 19650（第三次重排后退 0），输出 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/probe-run/out/`。
  - `stale-base.out`：`RESULT name=stale_base arm=stale_base_live_intent_check histories=54 compacted_on_a_moved_version=45 lost_foreground=0 lost_after=0 panicked=0 compaction_refused=0 writable_remount_failed=0`；`RESULT name=stale_base arm=txg_guard histories=54 compacted_on_a_moved_version=0 lost_foreground=0 lost_after=0 panicked=0 compaction_refused=0 writable_remount_failed=0`；阳性对照 `RESULT name=stale_base arm=planted_rollback histories=54 compacted_on_a_moved_version=0 lost_foreground=45 lost_after=0 panicked=0 compaction_refused=0 writable_remount_failed=0`。弱判据那一臂 45 段历史在旧一版上起了整理，逐段行里结局是 `outcome=Failed(Batch(Publish(ReleaseTargetAlreadyReleased_`，其中 30 段之后的前台写被拒（`after_refused` 大于 0）。
  - `lock-order.out`：`LOCK name=lock_order arm=released_before_the_session_lock switch_to=gpu hold_ms=1600 holder_waited_ms=0 switch_waited_ms=1595 read_waited_ms=1585`；`LOCK name=lock_order arm=held_into_the_session_lock_switch_to_gpu hold_ms=50 holder_waited_ms=not_done switch_waited_ms=not_done read_waited_ms=not_done deadline_s=10 deadlocked=1`；`RESULT name=lock_order deadlocked_when_held_into_the_session_lock=1`。
- 正推模型：任务 19652 退 0，日志「自检查了 9 格，红 0 格」「name=rerun_done grid_lines=96」，网格 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-sonnet-run/grid.txt`。它是排队仿真，W（锁外那一段的时长）与前台到达率是摆的，不是量的；弄坏开关那两条没交。
- 今天代码上的基线：任务 19653（副本 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-main/repo-copy`，正推腿的探针拷进 `crates/singlefs-mount-runtime/tests/`），`2 passed`：
  - `name=baseline_foreground_wait route=cpu sleep_ms_per_gpu_call=0 samples=56279 wait_micros_mean=14 wait_micros_p99=0 wait_micros_max=54337 samples_with_a_publish_in_between=46 max_publishes_in_between=5`
  - `name=baseline_foreground_wait route=gpu_double sleep_ms_per_gpu_call=5 samples=55156 wait_micros_mean=21 wait_micros_p99=0 wait_micros_max=77884 samples_with_a_publish_in_between=44 max_publishes_in_between=3`
  - 只跑了一趟、睡眠只取 5 毫秒一档，前台是单线程；腿建议的 0、50 毫秒两档没跑。

## 二、判决

| 格 | 判决 |
|---|---|
| Q1 形态 | 三段（锁里取输入、锁外算、回锁复核再发布）两腿都没否掉，**挂起**：它的每一处细节（读盘放哪、把手锁、CPU 路径）都还是岔路。攻方「最脆弱」四样（发布底是不是现行那一版、在途的批不记在会话上、计数登记在取锁处会让给自己、两把锁的次序）读码坐实前两样的事实依据：发布路径的 txg 只从交进去的那一版取（`transaction.rs:11170`），不核它是不是现行那一版 |
| Q2 弱判据 | **打中，但后果不是攻方说的那一种**。攻方报「盖掉前台已经答了成功的那一次发布」；探针 54 段历史 `lost_foreground=0`，丢失没有出现，出现的是整理发布失败（`ReleaseTargetAlreadyReleased`）与其后 30 段历史里前台写被拒。阳性对照丢 45 段，探针分得出丢失。结论：回锁复核不比现行那一版的 txg 不行；「会丢已答成功的发布」这一句不采 |
| Q2 只比 txg | 攻方「锁外期间 `accept` 收下没发，txg 不变，照发就破了屏障」是读码推的，没探针。`mounted_session.rs:7528` 的原句是「待发里不并进整理的发布……那一趟起跑之前先把待发排空」，照字面复核要加「待发为空」。**挂起**，第二轮攻 |
| Q2、Q3 活锁 | **打中**（两腿一致）。正推网格：整批作废、前台每秒 100 次发布时，W 取 5 毫秒作废 0.415、连着作废最多 11 批，W 取 20 毫秒作废 0.869、连着最多 44 批（`policy=split-discard-on-any-publish` 两行；仿真，W 是摆的）。破例批（全空段低于底额）上前台 `accept` 推抬 F 的空发布让 txg 一直变，整批作废时破例批发不出：读码推的，两腿都没量 |
| Q3 让路 | 让路只在取输入之前问一次：两腿一致，采。算完回锁那一刻再让，算好的一批就白算。计数登记点、停因成员名（`YieldedToAForegroundLockWaiter`）、D26（后台整理与放置回收） 已定项 13 第 6 条改句草案是正推腿提的，零轮 |
| 缺口 2（放锁后前台抢不到） | **基线不支持**。今天的代码上非 fsync 的前台平均等 14 到 21 微秒，最坏 54 到 78 毫秒，拿到锁之前后台最多发 3 到 5 次整理发布，55000 多次取样里中间夹了发布的只有 44 到 46 次。「一拍 64 次都抢不到」没出现。主 agent 2026-11-17 那一句是推的，撤回；最坏那 54 到 78 毫秒是不是持锁一批的时长，这一趟分不出 |
| Q4 崩溃语义 | 严判据（txg 相等且待发为空才用）下两腿都没打中：不多写序列、不要新流。算一次没打中，这一格在严判据下停 |
| Q5 锁序 | **打中，量到**。锁外那一段持着 GPU 实现那把锁时，前台切到 `gpu` 持着会话锁在它上面等满整段（等 1595 毫秒对持 1600 毫秒），读跟着等 1585 毫秒；持着 GPU 锁回来取会话锁就死锁（`deadlocked=1`）。拆开之后切换这条路比今天更糟 |
| Q5 卸载 | 两腿一致：不改 GPU 包里等的那一处，卸载与 Drop 照样等锁外那一批（读码，`crates/singlefs-mount-runtime/src/lib.rs:1136` 的 `Arc::try_unwrap`）。缺口 4 不随拆锁自动消掉 |
| Q5 把手锁中毒 | 正推腿报「算段 panic 毒掉 GPU 把手后，前台切换在持会话锁时 `expect` panic」：读码推的，没量，挂起 |
| Q6 验收 | 正推腿 14 条清单采作底稿；第 1 条已量（基线两行） |

第一轮有打中的（Q2 弱判据、活锁、锁序），照 `.claude/rules/three-way-inference.md`「开不开下一轮（设计推论轮）」开第二轮。

## 三、改法

都是腿提的或主 agent 拼的，逐条标量没量：

1. 回锁复核比现行那一版的 txg，不等就作废：**量过**（任务 19650 `arm=txg_guard`，54 段历史没有一段在旧一版上起整理）。
2. 回锁复核另判待发为空：零轮，没探针。
3. 回锁之前放掉 GPU 实现那把锁：**量过一半**（任务 19650 `arm=released_before_the_session_lock` 不死锁；切换仍等满整段）。
4. 切换的卡清单校验不取 GPU 实现那把锁（缓存卡清单、`try_lock` 拒、校验推迟三候选）：零轮。
5. 活锁的下限（连着作废若干批后下一批回锁里算、活意图那一支只核 R、破例批走锁内三候选）：零轮；仿真里 `floor2` 一档把连着作废压到 2 批、W 取 20 毫秒时前台最坏等从 8 毫秒回到 28 毫秒（仿真）。
6. 会话上记在途的批、拒第二次取输入：零轮。
7. 任何前台等锁都让路的计数、停因成员与反向饿死的下限：零轮。
8. 卸载叫停（改 GPU 包里等的那一处查停标记）：零轮。

标零轮的，写进产品代码之前先在探针里做 A/B。

## 交用户的

第二轮攻完再交岔路；现在先报两条改变了题面的读数：

- 缺口 2 的基线不支持「前台抢不到锁」；「任何前台等锁都让路」换得到的最多是最坏那 54 到 78 毫秒，代价是反向饿死要另设下限。
- 拆到锁外之后前台发布密时锁外算的批大半作废（仿真），要下限；下限的每一种候选都让一部分批回到锁里等 GPU。

## 这一轮被判过的文件

- crates/singlefs-mount-runtime/src/lib.rs：判了会话锁、一拍的取放锁次序、卸载 join 与让路计数在拆开之后各要改什么；基线量的是它今天的行为。
- crates/singlefs-core/src/mounted_session.rs：判了后台入口拆三段的切点、每批之前问调用方那一处、「整理是屏障」对回锁复核的要求。
- crates/singlefs-core/src/background_compute.rs：判了 GPU 实现那把锁与会话锁的次序、切换校验取锁那一处。
- crates/singlefs-core/src/compaction_plan.rs：判了停因与问答成员要加的那两处。
- crates/singlefs-core/src/transaction.rs：判了整理发布的 txg 只从交进去的那一版取、不核现行那一版。
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了等的那一处不查停标记、卸载因此照样要等。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-11-18 不受影响：形态挂起，第二轮之后才写回 |
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-18 不受影响：第 4、5、6 条的改句是零轮草案，没写回 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-18 不受影响：严判据下不多写序列 |
