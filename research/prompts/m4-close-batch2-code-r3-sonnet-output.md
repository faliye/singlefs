# m4-close-batch2-code-r3 辩方腿报告：复核第二轮判决，并答第三轮攻击面与检查单八格（2026-11-10）

<!-- doc-lint:not-numbers R -->

辩方腿（Sonnet）。被复核的判决：`research/prompts/m4-close-batch2-code-r2-main-verification.md`（依据的腿报告：`research/prompts/m4-close-batch2-code-r2-sonnet-output.md`、`research/prompts/m4-close-batch2-code-r2-opus-output.md`、承接腿 `research/prompts/m4-close-batch2-code-r2-opus-s2-output.md`）。
没写模型，没有模型目录与复跑脚本，也就没有要报的 sha256；没往统一任务服务交任务。读数全部来自主 agent 已交的任务日志（任务号逐个核过，见「数字能不能核」一节）。

开工核：`sha256sum -c research/prompts/m4-close-batch2-code-r3-snapshot/crates-sha256.txt`，两份 `OK`（`mounted_session.rs`、`mounted_read.rs` 与开工快照一致）。
读的旧版：暂存区那一份 `mounted_session.rs`（`git show :crates/singlefs-core/src/mounted_session.rs`，blob 前缀 7dc7474，就是第三轮 diff 的「改前」），判决引的行号（停着那一判 3394、挂载预算读根环 3409、一趟开头停着删意图 4505–4515、回退门定义 4626–4630、回退门那一判 4728）在旧版里逐处对得上。下面写「旧版第 N 行」的都指这一份；写 `路径:行号` 的都指此刻工作区。

## 一、结论先说

| 项 | 判 |
|---|---|
| 第二轮判决 B2（停着删意图不过回退门） | 成立（读码加 A/B 读数）；判决交用户表里「条款字面已写明」一句 kb 里没有这句字面，B2 是读法，不是字面 |
| 第二轮判决 H3（停着那一判排在预算读根环之前） | 次序打中成立（读码，且 kb 字面「挂载不再给新预算」撑着）；A/B 那一行 2 对 16 只量到内部计数，没量到起不起跑或写不写，「换得到东西」这一半推不出 |
| A/B 结论（任务 12435） | B2 一半成立、读数核得到；H3 一半口径是计数 |
| 第三节五条改法落到这一版代码 | 五条都落了（逐条见下表），一条破例分支没有用例 |
| 新冒出的、判决清单里没有的 | 两处：停着而门关着的那一趟没有备忘，运行库一拍内最多连叫 64 次；停着删意图那一支里「前台写不进破例」的分支没有用例也没有读数 |
| 检查单八格 | 成立 6 格（其中 1 格附一处没用例的分支），推不出 1 格（任务没出结局），成立（推的）1 格 |

## 二、复核第二轮判决

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| B2：停着那一趟删意图不过回退门 | 站得住 | 旧版第 4505 行起那一支在循环之前发删意图，回退门那一判在旧版第 4728 行的循环里；现在回退门那一判在 `crates/singlefs-core/src/mounted_session.rs:4753`：「if is_the_rollback_gate_judged && !is_this_publish_the_exception {」，循环外的删意图则没有。循环里所有删意图（丢意图、暂停时丢意图）都在 `advance_compaction_by_one_publish` 里、在这一判之后，所以只有停着那一支漏门，不是「同样打中别的臂」。A/B 日志第 34 行（无补丁臂）`first_wakes_publishing_past_a_closed_gate=6`，第 89 行（有补丁臂）`first_wakes_publishing_past_a_closed_gate=0`，六段历史（三种几何各两段）全中 | 有一段历史在门关着、停着、有活意图、预算还剩、前台写得进的前提下，无补丁臂的叫起没发删意图；或有补丁臂发了 |
| B2 的字面依据 | 够不着字面，读法站得住 | 判决 `research/prompts/m4-close-batch2-code-r2-main-verification.md:40` 写「条款字面已写明那一次照样受第 5 条约束」。kb 里这句字面没有：`.claude/kb/decisions/26-后台整理与放置回收.md:253` 只写「那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删」，`.claude/kb/decisions/26-后台整理与放置回收.md:254` 写通则「这次整理发布不发」与唯一的破例「只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次」。靠的是：停着删意图被第 4 条叫「发布」又吃发布预算（253），第 5 条的破例清单里没有它。`grep -nF '照样受第 5 条'` 在 kb 里命中 0 次。读法合理，但是读法 | kb 里有一句把停着删意图划出回退门（或明写它受门）的字面 |
| 被判出局的一方（停着删意图不过门）的理由 | 够不着「条款」，够得着「取舍」 | 旧臂的理由：门关着不发，意图与 R 排他留着，R 里的空槽别的写不能用；破例只开给「全空段低于底额」一档。kb 没排「少一个回退状态」与「R 排他多留」谁重。判决把它列进交用户表（`research/prompts/m4-close-batch2-code-r2-main-verification.md:40`「零轮」）是对的 | 用户裁定旧臂 |
| H3：停着那一判排在挂载预算读根环之前 | 次序成立；行为后果推不出 | 旧版第 3394 行停着判在前、第 3409 行读根环在后；现在读根环在 `crates/singlefs-core/src/mounted_session.rs:3407`：「&& compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()」，停着那一判在 `crates/singlefs-core/src/mounted_session.rs:3425`：「if compaction_stays_stopped(allocator, background_compute) {」。kb 字面撑着次序：`.claude/kb/decisions/26-后台整理与放置回收.md:254`「挂载不再给新预算」。判决写「刚挂载、停着而有活意图时，那一趟预算按」（`research/prompts/m4-close-batch2-code-r2-main-verification.md:17`）已过去 0 次算；A/B 日志第 9 行与第 49 行里挂载那一刻记的是 2（挂载自己的发布），不是 0，口径略松，不影响次序 | 有用例造出「停着、活意图、挂载后头一次调用」，读与不读两种次序下起跑结局相同且预算相同（本腿没造到，反过来也没造到不同） |
| H3 的「换得到东西」（A/B 判据：量它在被打中那一支上换不换得到写进或出路） | 推不出 | 日志第 9 行（无补丁臂）与第 49 行（有补丁臂）两臂都是 `live_intent_after_the_mount=false`、`started=false`、`publishes_of_the_wake=0`；只有 `publishes_since_the_last_user_change_after_the_first_call` 从 2 变 16（第 95 行：`counted_at_the_mount_without_the_patch=2 after_the_first_call_without_the_patch=2 counted_at_the_mount_with_the_patch=2 after_the_first_call_with_the_patch=16`）。这个几何每区根槽数 S=8，预算 `3S−4` 是 20（`crates/singlefs-core/src/compaction_plan.rs:1574`：「(3 * root_ring_slots_per_region)」），读到 16 还剩 4，不读按 2 还剩 18，两者都大于 0，起跑决定相同：就算那一刻手里有活意图，两臂的起不起跑也不会分开。承接腿自己写了没造到：`research/prompts/m4-close-batch2-code-r2-opus-s2-output.md:138`「没造 H3 改变行为的那一形」。判决第三节写「改法换得到东西，照写进产品代码」（`research/prompts/m4-close-batch2-code-r2-main-verification.md:30`）对 B2 一半成立，对 H3 一半只有「次序对、计数对」。H3 写进产品代码有 kb 字面撑着（上一行），不靠 A/B；若按判据「换不到的撤回」，H3 那一段该有自己的行为臂，现在没有 | 造出一段历史：停着、活意图、预算按读到的数扣完而按未读的数没扣完，两臂起跑结局不同——那才是换得到东西 |
| A/B 结论 `name=m4_close_batch2_r2_ab_verdict …`（任务 12435） | 成立（B2 一半）；口径需分开说 | 日志第 94 行：`without_the_patch_reproduces=yes with_the_patch_holds_at_a_closed_gate=yes with_the_patch_drops_once_the_gate_opens=yes arms_failed=0`。第 34 行与第 89 行的汇总 `first_wakes_publishing_past_a_closed_gate=6` 对 `0`、`drops_while_the_probe_reads_the_gate_closed=6` 对 `0`、`histories_with_the_intent_gone_after_a_wake_held_at_the_gate=0` 对 `6`。有补丁臂在三种几何下每段历史都是「门关着叫起不发、再建 3 个 inode 才发」（`user_changes_until_the_intent_is_gone=3`）。「门开」是探针自己读根环判的（`gate_refuses_by_the_probe`），与产品的门不共用代码。与最终代码对照：补丁里停着那一支的回退门、破例判与 `is_the_rollback_gate_judged` 上挪，与 `crates/singlefs-core/src/mounted_session.rs:4513`「let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })」到 4537 逐行一致（只差注释）。A/B 是主 agent 在副本上复跑的（任务 12435 第 3 次尝试退 0，前两次 memory-cap-hit） | 有补丁臂某段历史门关着时发了删意图；或用别的几何（每区根槽数不是 8）两臂读数不分开 |

### 第三节五条改法有没有落到这一版代码

| 改法 | 判定 | 证据（此刻工作区） |
|---|---|---|
| 1 B2 与 H3 | 落了 | 门：`crates/singlefs-core/src/mounted_session.rs:4524`「if budget != 0 && is_the_rollback_gate_judged && !is_this_drop_the_exception {」；读预算前移：3405「排在停没停那一判之前：停着而有活意图时起跑的那一趟同样吃这一份」；门关着盘上逐字节不变、意图与 R 排他留着：新用例 `crates/singlefs-verification/tests/compaction_stopped_intent_drop_judges_the_rollback_gate.rs:519`「门关着：意图与 R 排他都留着」、`crates/singlefs-verification/tests/compaction_stopped_intent_drop_judges_the_rollback_gate.rs:524`「门关着：盘上逐字节不变」。任务 12477 `test result: ok. 4 passed` |
| 2 H2（触发名与解除停下后的续跑） | 落了（取「不记成没完」那条路） | `crates/singlefs-core/src/mounted_session.rs:7524`「== BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}」（前一行 7522 是 `SpanOfTheBackgroundCall::AtMostOnePublish`、7523 是 `if trigger`）；整趟入口那一支 `crates/singlefs-core/src/mounted_session.rs:3655`「| BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}」本来就不记；文档串 `crates/singlefs-core/src/mounted_session.rs:519`「这一趟不跨调用接着推：放锁的入口排空待发被拒时不记成没完的一趟」与代码一致。用例 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1530`；任务 12479 `test result: ok. 25 passed` |
| 3 补用例：`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 那一拒 | 落了 | 新文件一条用例，任务 12478 `1 passed`；证红任务 12476 日志第 482 行「不带一批的删意图那一核滤掉上一版的记录…抓到」 |
| 4 补小用例：停着删意图、删除队列不空 | 落了（小用例，不是录制流） | `crates/singlefs-core/src/mounted_session.rs:4490`「删意图排在删除续做之前，队列不空时那一版照抄队列，续做等解除之后」；两条用例（显式入口、后台入口）在任务 12479 的 25 条里，证红任务 12476 日志第 483、484 行各抓到 |
| 5 `mounted_read.rs` 注释写实 | 落了，有一处不严 | `crates/singlefs-core/src/mounted_read.rs:1030`「提示不算数时再按中央映射读一遍」。「两遍发出的读都照计」对读的计数成立；但按提示读那一遍里旧槽上坏的那一份，在 `crates/singlefs-core/src/mounted_read.rs:1156`「observation_of_the_read_by_the_hints.data_unit_reads_with_one_copy_failing = 0;」被清零，不照计。同一段注释上一句「已经读到的只坏一份照样并进」与这一处小有出入。不影响计数结论（判决原话 `research/prompts/m4-close-batch2-code-r2-main-verification.md` 第 21 行也这么说） |

第三节五条之外，A/B 承接腿报告「没做的」那三项（`research/prompts/m4-close-batch2-code-r2-opus-s2-output.md:138`「破例那一档（前台写不进）、攒批会话与放锁的后台入口没扫」）：判决写「改法换得到东西，照写进产品代码」时它们没量过；实现员把破例判（`crates/singlefs-core/src/mounted_session.rs:4522`「let is_this_drop_the_exception =」）一并写进了停着那一支。放锁入口那一项下面第三节有一处新现象。

## 三、第三轮攻击面

### 攻击面 1：已定项撑不撑得住（D26 已定项 13 第 4、5 条现行字面对这一版代码）

逐句对。kb 行号是现查的；「对得上」指这一版代码与那一句的动词都对得上。

| 条款句 | 代码 | 判 |
|---|---|---|
| 第 4 条：显式入口与后台入口「起跑一趟、只删意图，之后不写」 | 显式入口走 `crates/singlefs-core/src/mounted_session.rs:4518`「if compaction_stays_stopped(allocator, background_compute) {」那一支，删完当场 `finish`，不进删除续做与循环；后台入口由 `crates/singlefs-core/src/mounted_session.rs:4337`「if !has_a_live_intent || (is_the_budget_spent && !foreground_cannot_write) {」判起不起跑 | 对得上 |
| 第 4 条：「那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 后台入口：预算扣完且前台写得进就 `NotStarted`（4337）；发成一次就计一次 `crates/singlefs-core/src/mounted_session.rs:4547`「compaction.publishes_since_the_last_user_change += 1;」；预算 0 时 `crates/singlefs-core/src/mounted_session.rs:4538`「let dropped_on_the_stop = if budget == 0 {」不发。显式入口：预算是 `publish_budget_of_a_compaction_run`，末尾 `.max(1)`（`crates/singlefs-core/src/compaction_plan.rs:1577`），显式入口的预算永远不为 0，「扣完就等」只在后台入口成立 | 后台入口对得上；显式入口字面对不上（推不出），旧有，不是这一轮 diff 引入，只在字面核时露出；显式入口没有产品调用方（`crates/singlefs-core/src/compaction_plan.rs:452`「不看水位、不过回退门，照常跑」） |
| 第 4 条：「现行那一版没有活意图的，一个写都不发，删除续做也不走」 | 4337 `!has_a_live_intent` 不起跑；显式入口 4538 一支交回 `Stopped`，之后 `finish` 先于删除续做（删除续做在 4594 之后） | 对得上 |
| 第 5 条：「挂载不再给新预算」（`.claude/kb/decisions/26-后台整理与放置回收.md:254`） | 3407 读根环排在 3425 停着那一判之前，停着起跑的那一趟吃同一份 | 对得上 |
| 第 5 条：「这次整理发布不发」与唯一的破例（254） | 停着删意图那一支过门（4524），破例判与循环里同形（4522 对 `crates/singlefs-core/src/mounted_session.rs:4523`「budget_rule == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite;」，循环里同一个判据加 `publishes_of_the_run == 0`） | 对得上，前提是停着删意图算「整理发布」（读法，见上一节 B2 的字面一行） |

门一直不开时意图与 R 排他留多久：

- 门只随根环里不同用户状态数变。停着时整理不发，根环只随用户改动涨。A/B 日志每段切到停着的历史里，门开靠 3 个新用户改动（第 50 至 54 行那一段：`user_changes_after_the_closure=0` 到 `=3`，第 54 行 `states_left_past_the_next_publish_and_the_margin_before=4`、`publishes_of_the_wake=1`）。没有用户改动就一直留着：第 51 行 `wake=again_without_a_change` 仍 `started=true`、`Stopped(CompactionPublishWouldLeaveFewerThanFourRollbackStates`。
- 重新可写挂载会删：`.claude/kb/decisions/26-后台整理与放置回收.md:261`「下一次可写挂载自己的发布不经门」，日志第 49 行 `live_intent_left_by_the_previous_mount=true … live_intent_after_the_mount=false`。
- 与第 4 条「停下都删意图」：第 4 条自己已写「预算扣完就等下一次用户改动之后那一次调用再删」（253），延迟删本来就在条款里。差别是放行条件：预算扣完靠一次用户改动就重给，门要等到根环里不同用户状态够数（这个几何是 3 次）。条款没写这个差别：不冲突，也不是已定，是新增的等待形态。已定项 1 第 3 条的「意图不留在盘上」（`.claude/kb/decisions/26-后台整理与放置回收.md:30`）说的是整理因空间被拒暂停那一支，暂停删意图在循环里、在门之后，不受影响；停着加门关则意图留在盘上，仍满足 I-5.6（`crates/singlefs-checker/invariants/07-分配器.md:11`「活整理意图的区域里没有签发之后分配的记录」），因为 R 排他照留。
- 推翻条件：有一段历史显示门关着停着、没有用户改动，删意图被门之外的别的路径发出；或条款里补一句把停着删意图划出门。

### 攻击面 2：未定项清单完整性

清单照现在的样子（判决「交用户的」只有 B2 一行，实现员写的「走不到的理由」是无），辩方腿补出下面几处。每处写读到的、什么会推翻它。

**F1：停着而门关着的那一趟没有备忘，运行库一拍里最多连叫 64 次**（新冒出的，B2 改法 × 放锁入口的交叉）。
- 读码：不发布的停的备忘只在定时兜底那一支被查：`crates/singlefs-core/src/mounted_session.rs:3511`「.stop_still_holding_at(」；停着那一支在 `crates/singlefs-core/src/mounted_session.rs:3425` 就返回，不查。备忘的文档串本来就只管定时兜底：`crates/singlefs-core/src/compaction_plan.rs:667`「记进备忘，定时兜底下一次不重判」。门关着的这一趟 `finish` 记了备忘，但停着的下一次叫起不读它。
- A/B 读数：日志第 51 行（有补丁臂，`wake=again_without_a_change`，没有任何改动）`started=true trigger=LiveIntentWhileStoppedAfterAComputeError`，结局仍是停在回退门；而没切停着的那条历史（日志第 61 行）第二次是 `NotStarted(UnchangedSinceAStopThatPublishedNothing …)`，备忘起了作用。
- 运行库：`crates/singlefs-mount-runtime/src/lib.rs:61`「pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;」，`crates/singlefs-mount-runtime/src/lib.rs:572`「while pass.compaction_calls < COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST」；`Ran` 带 `Stopped(stop)` 时 `keeps_going` 对除让路三种之外的停都为真（`crates/singlefs-mount-runtime/src/lib.rs:634`「!matches!(」、`crates/singlefs-mount-runtime/src/lib.rs:636`「CompactionRunStop::YieldedToAForegroundFsync」是其中一种），回退门的停不在那三种里。所以停着、门关着、预算还剩时，一拍里连叫到 64 次，每次叫起都在 `stop_of_the_rollback_gate` 里读根环与每条有效根的树表，且每次叫起数一次叫起：`crates/singlefs-core/src/background_compute.rs:1457`「wakes.wakes_while_stopped += 1;」，报警里的 `wakes_while_stopped` 一拍涨 64。改动之前这一支发一次删意图、意图没了，下一次叫起 `NotStarted` 就收手。
- 性质：不写盘，不违反条款文字（条款没说备忘）；是读放大和叫起数膨胀，门一直关着时每个周期都发生。「读码推的」，运行库那一边本腿没造用例。
- 推翻条件：运行库一拍里这一支只调一次（例如 `keeps_going` 对这个停为假），或备忘在停着那一支被查；一个探针让门关着的停着态在一拍里叫起次数为 1。

**F2：停着删意图那一支里「前台写不进」破例分支没有用例也没有读数**。
- 分支在 `crates/singlefs-core/src/mounted_session.rs:4522`「let is_this_drop_the_exception =」，起跑那一边 `crates/singlefs-core/src/mounted_session.rs:4337` 在前台写不进时放过预算扣完。A/B 腿报告写了没扫（`research/prompts/m4-close-batch2-code-r2-opus-s2-output.md:138`），实现员写的新用例都不造它：`grep -rn AtLeastOnePublishWhileTheForegroundCannotWrite crates/*/tests` 零命中（只在 `crates/mutations.tsv` 与 `crates/singlefs-core/src/mounted_session.rs` 里出现）。8 条新变异行里与它相关的第一行是把 `!is_this_drop_the_exception` 判反，反向的变异「破例不免门（总是判门）」没有行；那一种改坏新用例抓不到。
- 判：走得到，没有人钉。不是走不到。
- 推翻条件：补一段「全空段低于底额、前台写不进、停着、有活意图、门关着」的历史，两臂（有无破例判）读数不同，且有变异行钉住。

**F3：H2 取「不记成没完」那条路，别的入口有没有漏**。核了，没漏。
- 守卫只在放锁入口排空被拒那一支：`crates/singlefs-core/src/mounted_session.rs:7524`「== BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}」，只对 `AtMostOnePublish` 起作用；整趟入口（不放锁）排空被拒那一支本来就不记这个触发名：`crates/singlefs-core/src/mounted_session.rs:3655`「| BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}」。显式入口没有「没完的一趟」；可写挂载续做不经起跑那一判。三处都没有需要补的守卫。
- 停着时旧的没完的一趟（别的触发名记下的）留在会话上不被取走：取走在停着那一判之后（`crates/singlefs-core/src/mounted_session.rs:3425` 先于 3437），停着删意图那一趟既不取也不盖它，解除之后接着推，名字对得上它自己。
- 附带变化：排空被拒后 `has_a_background_trip_under_way` 为假，运行库下一拍等压缩周期而不是轮询周期；排空被拒的原因（冻结的发布）由运行库每个轮询周期先重发（`crates/singlefs-mount-runtime/src/lib.rs:668` 一支），影响不大。
- 推翻条件：有一条路径，触发名是停着删意图的那一趟被记成没完、解除停下后还顶着这个名字推普通整理。

**F4：H3 前移让「停着的头一次调用」也消费挂载预算那一次读，读不通会把预算当扣完**（新冒出的，边角）。
- 读不通当扣完是原有口径：`crates/singlefs-core/src/mounted_session.rs:3405`「读不通当那一份已扣完」所在注释，`crates/singlefs-core/src/mounted_session.rs:3415`「publish_budget_of_a_compaction_run(」按已过去 0 次取 `3S−4`？不对：那一处的第二个参数是 0，取的是整份预算 `3S−4`，再经 `.max(publishes_since…)` 并入，所以「读不通」落下来的已过发布数是读不通时回退的 `3S−4−0`。——这一句读法我没读到底，读到的只是第 3414 行起 `.map_or(` 的默认值；读不通到底把预算置零还是置满，本腿推不出，写在没做什么里。只读一次：`crates/singlefs-core/src/compaction_plan.rs:1041`「读没读通都只读这一次」。
- 提示：若读不通确实把预算置为扣完，停着的头一次调用遇到瞬时读错，停着删意图就被压到下一次用户改动；前移之前这一错只影响不停着的路径。
- 推翻条件：读 `map_or` 默认值的方向（`publish_budget_of_a_compaction_run(S, 0)` 是整份预算还是剩余 0），或给读错造一段历史看停着删意图起不起跑。

**F5：doc 与代码的小出入**（不在判决清单里）。`crates/singlefs-core/src/compaction_plan.rs:639`「停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它」没提回退门；后台入口照水位判时这一刻可能停在回退门上、一个写都不发。`compaction_plan.rs` 不在这一轮 diff 里，文档串没跟着改。

### 攻击面 3：报出的数字能不能核

主 agent 在正文第二节报的每个任务号，腿用 `task-server/task status <号>` 与 `task-server/task log <号> --all` 逐个读（读，不交）。

| 任务 | 正文的话 | 核到的 | 判 |
|---|---|---|---|
| 12476 | 点名 8 条：现判 8 条…抓到 8 条 | 日志第 486 行「✓ 点名 8 条：现判 8 条、复用 0 条、拒算 0 条；跑了 8 条，抓到 8 条」；第 477 至 484 行 8 行各一条用例红。跑在实现员的副本里：三份改过的文件与仓根逐字节相同（两份源文件 sha256 对上，三份测试文件 `cmp` 无差别）；副本的 `crates/mutations.tsv` 与仓根差 32 行，都是别的批的行，8 条本批行在两边一致 | 核得到 |
| 12477 | 新目标退 0 | 日志第 145 行 `test result: ok. 4 passed` | 核得到 |
| 12478 | 退 0 | `1 passed` | 核得到 |
| 12479 | 退 0 | `25 passed` | 核得到 |
| 12480 | clippy 退 0 | 末行 `Finished`，日志里 `^warning`、`^error` 行数 0 | 核得到 |
| 12481 | build 退 0 | 退 0 在第 4 次尝试（前三次 memory-cap-hit 退 250）；结局 `succeeded` | 核得到，正文没提前三次 |
| 12482 | 变异行静态核退 0 | 退 0；`UNDEFINED` 行数 0；`IDENTIFIERS_CHECKED 7029 585`。跑在副本里 | 核得到 |
| 12483 | 拒算 12 条、登记表 12 行 | 日志末行「拒算 12 条、登记表 12 行」，列出的全在 `crash_identity.rs`，三个本批测试文件名命中 0 次 | 核得到 |
| 12523 | 命名四格退 0 | 末行「4 格里 4 格绿」 | 核得到；实现员命令清单里另有 `naming-lint.sh` 一条，正文没报它的任务号，推不出 |
| 12435 | A/B 两行 | 日志第 94、95 行与正文引的字句逐字一致 | 核得到；口径见下 |
| 12522、12525 | 在跑 | 此刻 12522 `running`、12525 `sharded`，没有结局 | 推不出 |

「H3 那一行 2 对 16」的口径：对。16 与探针自己数的 `roots_after_the_last_user_change_by_the_probe=16`（日志第 49 行）相等，2 是挂载那一刻会话记的（挂载自己的发布，`publishes_counted_at_the_mount=2`）。两个数在两臂里取的是同一个字段 `publishes_since_the_last_user_change_after_the_first_call`，口径一致。但它是一个内部计数，不是起跑结局，见第二节 H3 的行为一行。

### 勘误（本报告上文三处，以这里为准）

- F4 定稿：读不通时已过发布数取 `publish_budget_of_a_compaction_run(S, 0)`（`crates/singlefs-core/src/mounted_session.rs:3415`「publish_budget_of_a_compaction_run(」，默认值在 3414 的 `.map_or(` 里），即 `3S−4`；剩余预算是 `3S−4` 减已过发布数（`crates/singlefs-core/src/compaction_plan.rs:1590`「.saturating_sub(publishes_since_the_last_user_change)」），所以读不通 = 剩 0 = 扣完，与 3405 注释「读不通当那一份已扣完」一致。前移之后，停着的头一次后台调用遇到一次读不通，会让停着删意图被压到下一次用户改动才有预算（前台写不进的破例除外）。前移之前这一错只影响不停着的路径。读码推的，没造用例。推翻条件：停着的头一次调用在读不通时仍然起跑删意图。上文 F4 里「这一句读法我没读到底」那一句作废。
- F1 里「没切停着的那条历史（日志第 61 行）第二次是 `NotStarted(…)`」应读作：日志第 61 行是没切停着的那条历史门关之后的头一次叫起，已是 `NotStarted(UnchangedSinceAStopThatPublishedNothing …)`，备忘起作用；停着的那条历史（第 50、51 行）连两次叫起都 `started=true`。结论不变。
- F2 里「第一行」指变异行「停着删意图之前不判回退门（后台入口停着那一趟单发的删意图把破例判反：门关着照发）」，它把 `!is_this_drop_the_exception` 判反；没有「破例也要判门」或「去掉破例判」那一方向的行。

## 四、检查单八格

| 格 | 判 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| 走不到的理由二次查实 | 成立（没有走不到的分支），附 F2 | 这一轮 diff 新加的分支：停着那一支的回退门（4524）被 3 条用例走到、1 条用例钉前移；H2 守卫（7524）被 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1530` 走到；停着删除队列不空的两条各走一条。唯一没人走的是破例分支（4522），走得到但没有用例，见 F2 | 有一个新加分支，造不出走它的输入 |
| 类型与名字 | 成立，小处不齐 | `LiveIntentWhileStoppedAfterAComputeError` 说的是「停着时为删活意图起跑」，现在这一趟可能只读一次回退门就停，起跑的原因没变，名字与做的事仍对得上；文档串 519 与代码 7524 一致。小处：`compaction_plan.rs:639` 的停因文档串没提回退门（F5）。新函数名没有：这一轮没新加函数，新局部名 `is_this_drop_the_exception`、`stop_of_the_gate` 说得出内容 | 命名检查（任务 12523）有一格红；或另一个读者读 519 的文档串得出与代码不同的行为 |
| 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r3-diff.md` 零命中（rc=1）；两份新测试文件同一条 grep 零命中；diff 新增行里没有日期、「此前」「原先」「改成了」这类历史句（grep 零命中），注释把原先带日期的两处改成了现状句 | 有一条分支认旧版本、旧布局或为旧行放宽读法 |
| 不新建判定库 | 成立 | diff 只在 `crates/singlefs-core/src/`；新测试文件里 `sqlite`、`rocksdb`、`std::fs`、`File::`、`env::var`、`CARGO_MANIFEST_DIR` 命中 0 次（命中的 `.tsv` 只是文档串里引变异表的名字） | 产品代码里出现写判定的路径 |
| 所有任务多机可用 | 成立 | 新测试都在内存里建池（`crates/singlefs-verification/tests/compaction_stopped_intent_drop_judges_the_rollback_gate.rs` 的 `memory_pool_after_a_sequential_write`），不读本机文件与环境；任务 12483 登记表 12 行里没有本批测试文件；12477 在第二台、12478 在本机，各自退 0 | 一条测试在两台之一读出不同结局 |
| 测试优先多线程 | 成立 | 回退门文件 4 条独立 `#[test]`，每条自建池与盘镜像，没有 `static`、`thread_local`、串行标记；任务 12477 四条 3.46 秒跑完。补用例文件 1 条。第三个文件新加的用例各自独立（`grep -n '^#\[test\]'` 逐条） | 有条用例靠别的用例先跑；或测试线程设 1 才绿 |
| 带 --item 的命令与项 | 推不出 | 任务 12522（变异行点名重放）此刻 `running`，12525（崩溃点重放点名 2 项，缩域 n=5）此刻 `sharded`，都没有结局。两项崩溃流（`compaction-paused-short-of-space-dropping-the-intent`、`compaction-with-a-write-while-the-intent-is-live`）走的是循环里的删意图，不走停着那一支，只能证这一轮没碰坏同一段代码 | 两个任务出结局：变异行有一行没被抓、或崩溃点重放红 |
| 新崩溃流多罩了什么 | 成立（推的）：这一形不需要崩溃枚举 | 停着删意图、删除队列不空那一形只发一次删意图的发布，用的是和暂停流同一个发布函数：`crates/singlefs-core/src/mounted_session.rs:5986`「let published = publish_the_compaction_step_once(」，步种类是 `CompactionIntentStep::Drop`；崩溃前后两态一是「意图活着带着队列」一是「无意图带着队列」，都是别的流已罩的状态（`.claude/gate.d/crash-point-replay-items.tsv:78` 暂停删意图流、`.claude/gate.d/crash-point-replay-items.tsv:39` 删除与整理交错流）。差别只在版本里多抄了队列，不改写盘顺序。这一句是读码推的 | 发现这一形的删意图发布比暂停流多写一步根可见的写，或崩溃后恢复在带队列的无意图态上走出别的结局 |

## 五、要主 agent 跑的命令

无现成可交的命令：本腿没写模型。F1 与 F2 要量行为才能由「读码推的」变成读数，需要新派一条同立场的腿写探针（带「承接：research/prompts/m4-close-batch2-code-r3-sonnet-output.md」），造法：

- F1：沿用 `research/prompts/m4-close-batch2-code-r2-opus-s2-model/probes/attack_ab_stopped_drop_judges_the_rollback_gate.rs` 的停着加门关历史，对同一个会话连叫 N 次后台入口（放锁的那个：`run_compaction_in_the_background_for_at_most_one_publish`），数 `Ran` 的次数与 `wakes_while_stopped`；再套运行库的一拍（`COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST` 是 64）看一拍的叫起次数。对照臂：没切停着的同一历史。命令写成 `task-server/task submit` 的整条由那一腿的报告给。
- F2：同一探针里把前台写不进（全空段低于底额）加进来，两臂：现行码与去掉 `!is_this_drop_the_exception` 的码，看门关着时删意图发不发。

本腿没拿到结局的任务：12522、12525（见攻击面 3 与检查单第七格）；它们的结局出来之后，这两格的「推不出」才能改判。

## 六、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑，报告里的数都是主 agent 已交任务的日志读数。
- 没写模型、没起仓副本与编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r3-sonnet` 里只有两份文本：暂存区那一份旧版 `mounted_session.pre.rs` 与任务 12435 日志的拷贝 `task-12435-log.txt`（`du -sh` 共 508K，交回时保留，主 agent 读完可删）。
- F1、F2、F4 都是读码推的，没有行为读数；F1 的「一拍 64 次」没在运行库上量。
- 没读崩溃枚举的状态数，没核 12522、12525 的结局。
- 没判 kb 该不该补一句「停着删意图受回退门」；那是用户的取舍（判决已把 B2 改法列进交用户表）。
- 没核 `crates/singlefs-core/src/compaction_plan.rs` 以外的文档串是否还有停着删意图不提门的（只看了 639 一处）。
