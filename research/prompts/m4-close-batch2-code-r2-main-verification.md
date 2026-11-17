# m4-close-batch2-code-r2 判决：里程碑四收口第二批，代码轮第二轮（2026-11-10）

<!-- doc-lint:not-numbers R -->

## 一、腿与核查

- 腿：辩方 `research/prompts/m4-close-batch2-code-r2-sonnet-output.md`（复核第一轮判决 `research/prompts/m4-close-batch2-code-r1-main-verification.md`），攻方 `research/prompts/m4-close-batch2-code-r2-opus-output.md`，模型 `research/prompts/m4-close-batch2-code-r2-opus-model/`。各抽一次，「没打中」不支撑结论。
- 开工快照 `sha256sum -c research/prompts/m4-close-batch2-code-r2-snapshot/crates-sha256.txt`：六份 `OK`。
- 引文：两腿各自报的 cite-check 末行，辩方「✓ 核了 54 处引文、32 处裸行号，对上 54 处」，攻方「✓ 核了 19 处引文、0 处裸行号，对上 19 处」。主 agent 没有另跑倒推快照树的 `snapshot-tree.py` 与 cite-check（用户 2026-11-10 暂停交任务服务的活，这两条要经服务起）；判决引的行主 agent 现读过：`crates/singlefs-core/src/mounted_session.rs` 第 3394–3412 行（停着那一判在第 3394 行，挂载那一份预算读根环在第 3409 行之后）、第 4505–4515 行（一趟开头停着删意图）、第 4626–4630 行（回退门只罩后台入口照水位判的那一趟）、第 4728 行（回退门那一判）；`.claude/kb/decisions/26-后台整理与放置回收.md` 第 252 行（已定项 13 第 5 条，原文「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发」）。都与腿引的一致。
- 复跑：攻方探针任务 12314（模型拷到 `~/.local/share/singlefs/work/m4-close-batch2-code-r2-main-rerun/model`），前几次尝试因第二台断连失败，第 4 次退 0：`name=m4_close_batch2_r2_attack_snapshot rc=0 expected=0`、`name=m4_close_batch2_r2_attack_summary hit_reproduced=yes`，读数见第二节 B2 一行。

## 二、判决

| 格 | 判 |
|---|---|
| B2（攻方）＝ H1（辩方）：停着那一趟删意图不过回退门 | 打中，读码，两腿各自独立得出、主 agent 现读坐实：第 4505 行那一支在进循环之前发删意图，第 4728 行的门在循环里；门的罩法（第 4626 行注释「回退门只在后台入口、开关照水位判时判」）没把它划出去，却走不到它。与 D26 已定项 13 第 5 条字面对不上：那一次删意图也是整理发布，根环里不同用户状态会少于 4 个时照样发。量的：任务 12314（第 4 次尝试退 0，前几次第二台断连），门前用户改动 4、6、8 三档全中：`stopped_drop_gate_summary geometries_judged=3 hits=3`；每档切到一张卡都用不上之后叫起 `gate_closed_before=true live_intent_before=true … states_left_past_the_next_publish_and_the_margin_before=3 started=true trigger=LiveIntentWhileStoppedAfterAComputeError … publishes_of_the_wake=1`，对照臂不切换 `started=false … NotStarted(… CompactionPublishWouldLeaveFewerThanFourRollbackStates …)` |
| H3（辩方）：停着那一判排在挂载那一份预算读根环之前 | 打中，读码（第 3394 行在第 3409 行之前）：刚挂载、停着而有活意图时，那一趟预算按「已过去 0 次」算。攻方同一处判「推不出」、没造到产品前缀。与 B2 同一处改 |
| H2（辩方）：攒批会话停着、排空被拒时记下的触发名是 `LiveIntentWhileStoppedAfterAComputeError`，解除停下后照普通整理续跑 | 推不出，算轻：不违反条款，名字与做的事不符。并进下一批 |
| 停着删意图没经 fsync 让路与写带宽两判（攻方） | 不打中：D26 已定项 13 第 6 条约束的是整理的批，辩方同判 |
| A2 修补（删意图那一次放开 R） | 两腿都没打中（各一次抽样）。放开与放回在三条返回路上成对（攻方）；发布出错与冻结两形照样放回（辩方）。附两处：新崩溃流的崩溃镜像不带「R 之外满」，恢复后在同样紧的盘上续做那一支没枚举（攻方，推不出）；`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 那一拒没有用例走到（辩方，`tests/` 下零命中） |
| A3 修补、`mounted_read.rs` | 两腿都没打中（各一次抽样）。`mounted_read.rs` 第 1029 行注释「一次调用里一个单元只读一遍」字面不成立（提示不算数时按映射再读一遍），计数结论不受影响 |
| 复核第一轮判决 | A1、A2、A3 三处打中站得住（辩方）。两处更正采纳：第一轮判决把 D26 已定项 13 第 4 条写成「限在……没有活意图那一刻」，`.claude/kb/decisions/26-后台整理与放置回收.md` 第 251 行原文没有这个限定，撑住 A1 改法的是用户 2026-11-10 定案，条款字面要随写回改；第一轮回看决策表 D26 已定项 13 一行写「不影响」与要改字面自相矛盾，改在这一轮「回看决策」表里 |
| 四·1 至四·7 | 两腿都判成立；攻方另报第 516 行文档串「预算照后台入口别的起跑给」在回退门上推不出，并进 B2 |
| 四·8 | 新流多罩的那一句成立；第一轮判决第三节第 3 条要补的「停着删意图、删除队列不空」那一形仍没有录制流罩着 |

## 三、改法

下一批交实现员（改 `crates/singlefs-core/src/mounted_session.rs` 一处为主）：

1. B2 与 H3：后台入口停着那一趟删意图之前，先读挂载那一份预算（第 3409 行那一段挪到停着那一判之前），再过回退门；门关着就照门不发、意图与 R 排他照留到门开。这一改法是攻方自提的「改法甲」加辩方 H3 的次序，被攻过零轮；照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」里零轮改法那一条，写进产品代码之前先在探针里对它做 A/B（有这一改法与没有两臂、同一批输入），读数由主 agent 经任务服务跑出：A/B 读数任务 12435（攻方承接腿 `research/prompts/m4-close-batch2-code-r2-opus-s2-output.md` 的两臂探针，补丁 `research/prompts/m4-close-batch2-code-r2-opus-s2-model/patches/stopped-drop-reads-the-mount-budget-and-judges-the-gate.patch`），`name=m4_close_batch2_r2_ab_verdict without_the_patch_reproduces=yes with_the_patch_holds_at_a_closed_gate=yes with_the_patch_drops_once_the_gate_opens=yes arms_failed=0`；H3 `name=m4_close_batch2_r2_ab_mount_budget counted_at_the_mount_without_the_patch=2 after_the_first_call_without_the_patch=2 counted_at_the_mount_with_the_patch=2 after_the_first_call_with_the_patch=16`。改法换得到东西，照写进产品代码。
2. H2：触发名与解除停下后的续跑对上。
3. 补用例：`RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 那一拒（不带一批的删意图那一核放过上一版就在 R 里的记录那一方向）。
4. 补录制流或小用例：停着删意图、删除队列不空那一形。
5. `mounted_read.rs` 第 1029 行注释写实。

## 交用户的

| 项 | 状态 |
|---|---|
| B2 改法（停着删意图先读预算、过回退门，门关就不删） | 零轮；条款字面已写明那一次照样受第 5 条约束，主 agent 判照条款改，A/B 读数出来之后派实现员 |

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：A3 改法的文档串（「走它的读」列表补两处）
- crates/singlefs-core/src/compaction_plan.rs：A1 文档串、A2 放开与放回 R 排他的两个方法
- crates/singlefs-core/src/mount.rs：A3 改 `caller_inputs_agreeing_with_the_disk` 及调用方经计数层读者、锚点涉及的那几段
- crates/singlefs-core/src/mounted_read.rs：相对 HEAD 的全部改动（类型包装、只读挂载、`read_at` 的读与计数）
- crates/singlefs-core/src/mounted_session.rs：A1 后台入口停着删意图、A2 删意图取落点、A3 读文件入口经计数层读者（B2、H3 打在这里）
- crates/singlefs-core/src/transaction.rs：A2 不带一批的删意图那一核与取落点放开 R

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 推翻 | 2026-11-10 改了：第 4 条字面要随用户 2026-11-10 定案改（后台入口停着删意图、吃预算），写回随这一批 kb 写回 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-11-10 不受影响：删意图那一次放开 R 的写回随 kb 写回批，条款义不改 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-10 不受影响：不判名单的射程写回随 kb 写回批 |
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-10 不受影响：A3 是实现没接全，条款不改 |
