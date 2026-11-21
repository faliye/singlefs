# m4-gpu-wait-outside-the-lock-code-r3：判决（2026-11-20）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r3-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-snapshot/`。这一题代码轮第三轮（最后一轮），攻第二轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-main-verification.md` 的修补；第一轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-sonnet-output.md` | 改法 1、2、3、5、6 都成立；第二节第 3 条打中一处（冻结后重发成的整理发布不清让路计数），读法交主 agent；测试缺口三处、钉实现取值一处；九格成立 |
| 攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-opus-output.md` | 改法第 2 条打中形甲（锁外那一段里冻结的用户改动被回锁改记成整理）；同一病根另一形乙（删意图被拒不记待删）；第二节第 3 条打中形丙（同正推腿那一处）；三条候选变异推的罩不到；九格成立 |
| 攻方承接（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-opus-s2-output.md` | 搭改乙、改丙、改丁三样合起来的 A/B，读数推的 |

核查：
- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpu-wait-outside-the-lock-code-r3 --out ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main/m4-gpu-wait-outside-the-lock-code-r3-tree --repository /home/fy5090/code/singlefs`，`type=succeeded exit=0`：「清单点名 19 份，倒推出 19 份，sha256 对上 19 份」，19 份都取工作区此刻那一份。
- `python3 scripts/agents/cite-check.py`（三份腿报告，`--root` 快照树，`--background` 正文）：「核了 50 处引文、156 处裸行号，对上 45 处，没判 161 处」，没判的是快照之外的文件；`--root` 换成仓根：「核了 50 处引文、156 处裸行号，对上 50 处，没判 156 处（其中裸行号没有引文可比 156 处）」。
- 攻方探针（模型拷进 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main/model/`，三份文件 sha256 与腿报告的表相同）在它的冻结副本上复跑，`type=succeeded exit=0`，汇总两行原样：
  - `RESULT name=relock_drop_refused_by_a_freeze_in_the_window_summary histories=20 premises_held=20 error_relabelled=12 control_relabelled=0 error_accounted_as_a_compaction=12 control_accounted_as_a_compaction=0 same_entry_key_relocated_under_the_abandoned_intent=5 device_lost_relocated_under_the_abandoned_intent=0 control_relocated_under_the_intent=6`
  - `RESULT name=stopped_drop_frozen_at_the_trip_start_and_the_yields_in_a_row_summary histories=2 premises_held=2 control_yields_in_a_row_after_the_drop=0 frozen_yields_in_a_row_after_the_resend=1 control_yields_before_a_take_without_yielding=2 frozen_yields_before_a_take_without_yielding=1`
  腿推的 `control_relocated_under_the_intent=0` 对不上（量到 6）：对照臂没出错，在意图下接着搬本来就对，这一项分不出臂，形乙改用读码判（判决第 2 条）。
- 候选变异证红：正推腿四条在主 agent 建的副本 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main/copy` 上，`type=command-failed exit=1`「跑了 4 条，抓到 1 条」：「会话记着的待删意图不核现行那一版里的活意图还是不是它」「取下一批之前删被门压住的意图那一次破例不看全空段是否低于底额」「回锁签发重判资格不看钉住的槽」没红，「删意图发成之后会话不忘掉被压住的那一条」抓到。攻方腿三条在它的冻结副本上 `type=command-failed exit=1`「跑了 3 条，抓到 0 条」。
- A/B：承接腿的装置拷进 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main/model-ab/` 头一趟复跑两臂读数逐字相同；有改法那一臂的输出里没有 `Compiling singlefs-core`，两臂共用一个编译目录，有改法那一臂跑的是没改法那一臂的产物，那一趟不作数。主 agent 在拷贝的 `tools/rerun.sh` 里给有改法那一臂单给编译目录，并加一道核（输出里没有重编 `singlefs-core` 退 5），重跑 `type=succeeded exit=0`，有改法那一臂重编了，读数在「三、改法」那一行。没改法那一臂与前一趟复跑 26 行逐行相同，对照 17 行两臂逐行相同。同一副本上点名 17 条已有用例（回锁出错、让路、回锁时冻结三个子模块）「17 passed; 0 failed」，那一趟重编了 `singlefs-core`。

## 二、判决

条款原文在 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 4、5、6 条（第 255–257 行）。

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 形甲：锁外那一段里前台一次用户改动冻结，回锁出错那一支删意图被冻结拒，记账照取输入那一刻「当时没冻」把冻结的那一次改记成整理，重发成了照整理记账 | **采，量过** | 第 257 行「冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）」。`crates/singlefs-core/src/mounted_session/compaction_run.rs:662-666` 传的是 `run.was_frozen_when_the_run_started`，`crates/singlefs-core/src/mounted_session.rs:457` 据它改记种类。探针 `error_relabelled=12 control_relabelled=0` |
| 2 | 形乙：回锁出错那一支删意图被冻结或空间拒时不记待删，整理只放弃这一批、没停时之后在这条意图下接着搬 | **采，读码坐实** | 第 255 行「意图也删掉」、第 259 行重复键「那一批的意图照第 4 条一批出错放弃的规矩删」。`crates/singlefs-core/src/mounted_session/compaction_batch.rs:705-708` `drop_the_live_intent_on_a_stop` 交回错时什么都不记。修补之前就写得出 |
| 3 | 形丙：冻结之后重发成的整理发布不清让路的两个计数（两条腿都报） | **采，量过** | 第 257 行「一批发成两个都清零」「冻结的是后台批，照后台批记」。`crates/singlefs-core/src/mounted_session.rs:1135-1138` 只记预算与根槽。探针 `frozen_yields_in_a_row_after_the_resend=1`、对照 0 |
| 4 | 测试缺口：钉住的槽单独去掉、补发那一次破例判法里的两个合取项、过时的待删那一判，改坏照样绿 | **采，量过** | 候选证红：正推腿三条没红、攻方腿三条没红（「一、腿与核查」）；`crates/singlefs-core/src/mounted_session/compaction_batch.rs:961`、`crates/singlefs-core/src/mounted_session/compaction_run.rs:512`、`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1499` |
| 5 | 正推腿：补发停因的断言钉了条款没写的值 | **不采成缺陷** | 用例照这一版实现断言停因，条款没写停因，改动停因时用例跟着改；不挡这一题 |
| 6 | 攻方腿：删意图被冻结拒之后这一趟以失败收尾，要不要照改甲改成停 | **不采** | 第 255 行没写那一次删意图被拒算不算整理失败；改丙之后待删记着、下一次照删，结局与条款相符，不改丢批那一判 |
| 7 | 两条腿：待删那一次问不问让路、写带宽对破例批照判、重发那一次记不记写带宽 | **推不出，不改** | 条款第 5、6 条没写；改丁只清两个计数（攻方腿「写带宽要拿到这一次的上限才记得了，推不出怎么拿」） |
| 8 | 攻方腿：pub 的 `has_an_intent_of_an_abandoned_batch_still_to_drop` 在冻结重发成之后、下一次取输入之前仍答「还没删」 | **不采成缺陷** | 它答的是会话记没记着，下一次取输入照现行那一版核（`compaction_batch.rs:1499`）；不影响写 |
| 9 | 改法第 1、3、5、6 条与 kb 两处改动 | **没打中**（两条腿都判） | 腿报告各自第一、二节 |

九格检查单：两条腿九格都答成立；正推腿「带 --item 的命令与项」答推不出（验证命令原文不在它读的材料里），主 agent 核：合入后验证第 2 步点这一批 10 行「基线不绿 0 行」。

## 三、改法

都在 `crates/` 改，合成一批交实现员，以主工作区此刻为基线。第 1 到 3 条是这一轮（最后一轮）定的修补，被攻过零轮，三样合起来在探针里做过 A/B：有改法那一臂 `error_relabelled=0`（没改法 12）、`error_drop_refused_noted_at_the_relock=12`（没改法 0）、`error_drop_refused_then_relocated_under_the_abandoned_intent=0`（没改法 5）、`frozen_yields_in_a_row_after_the_resend=0`（没改法 1），对照 17 行两臂相同。改法以 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main/model-ab/fixes.patch` 为准。
A/B 读数：任务 24796

1. 改乙（判决第 1 条）：回锁那一次调用进门时重读一次此刻冻没冻着，一批推完记账、停着那一趟开头两处改记冻结种类只在冻结的就是这一次调用发的那一次时改。补用例（锁外那一段里前台冻结一次用户改动、回锁出错、重发成了照用户改动记账）与变异行。
2. 改丙（判决第 2 条）：回锁出错那一支删意图交回任何错都照门压住那一支记下待删，错照旧交回。补用例（重复键、删意图被冻结拒、重发成了之后下一次调用把意图删掉、不在它下面接着搬）与变异行。
3. 改丁（判决第 3 条）：`resend_the_frozen_publish` 后台批那一支清让路的两个计数。补用例（停着时开头删意图写盘冻结、重发成了、计数是 0）与变异行。
4. 补测试（判决第 4 条）：钉住的槽单独一项、补发那一次破例判法的两个合取项、过时的待删那一判各一条会让对应候选变异红的用例，候选收进 `crates/mutations.tsv`，点名测试写带模块前缀的全名。

## 交用户的

无。判决第 6、7 条条款没写的几处照现状不改，登进 `records/2026-11-19-未做完的任务登记.md`。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：判了只供测试用的开关 `note_a_cluster_segment_opened_in_this_mount_for_tests` 产品路径一处不调，两条腿都判成立
- crates/singlefs-core/src/compaction_plan.rs：判了待删那一条记下、取、忘三个方法与只住内存（判决第 8 条不采）
- crates/singlefs-core/src/mounted_session.rs：判了冻结种类改记那一处与冻结重发后台批那一支不清让路计数（判决第 1、3 条打中）
- crates/singlefs-core/src/mounted_session/background_compaction_out_of_the_lock.rs：判了回锁发布入口沿用取输入那一刻的冻结状态，并进判决第 1 条
- crates/singlefs-core/src/mounted_session/compaction_batch.rs：判了出错那一支删意图被拒不记待删（判决第 2 条打中）、取下一批输入之前先删待删、过时的待删那一判缺用例（判决第 4 条）
- crates/singlefs-core/src/mounted_session/compaction_run.rs：判了一批推完记账改记冻结种类（判决第 1 条）、补发那一次的破例判法缺用例（判决第 4 条）
- crates/singlefs-core/src/mounted_session/compaction_publish.rs：判了取输入的调用点，没有绕开先删待删的路（两条腿都判没打中）
- crates/singlefs-core/src/mounted_session/compaction_entry_points.rs：判了锁里一口气推完的入口经同一处取输入，没有绕开的路
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs：判了补发那一次的用例只罩门开与破例一半、停因断言钉实现（判决第 4、5 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/entries_while_a_batch_is_out_of_the_lock.rs：判了锁外期间再取输入被拒的用例照旧成立
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs：判了签发那一支四条新用例测的是条款的量，钉住的槽单独一项罩不到（判决第 4 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs：判了子模块声明与九条新用例一一对应变异行
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/publishes_frozen_and_drains_refused_at_the_relock.rs：判了回锁时冻结那几条用例没罩到锁外期间冻结的用户改动被改记（判决第 1 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/relock_after_a_foreground_change_in_the_window.rs：判了窗口里前台改动之后回锁剔单元的用例照旧成立
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs：判了让路清零用例没罩到冻结重发那一形（判决第 3 条）
- research/prompts/_m4-gpu-wait-outside-the-lock-code-r3-body.md：判了题面列的修补落点与九格答案，两条腿逐格核过
- research/prompts/m4-gpu-wait-outside-the-lock-code-r2-main-verification.md：判了第二轮改法五条做的是判决说的，两条腿都判成立
- research/prompts/m4-gpu-wait-outside-the-lock-code-r3-snapshot/mutation-rows-of-the-batch.tsv：判了 10 行原文各命中 1 次、点名带模块前缀的全名，三处罩不到（判决第 4 条）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-20 不受影响：形甲、形乙、形丙都是实现没照第 4、5、6 条做，条款不改；没写的几处（判决第 6、7 条）照现状不改 |
