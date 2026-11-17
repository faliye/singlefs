# m4-close-batch2-code-r3：攻方腿报告（Opus，2026-11-10）

<!-- doc-lint:not-numbers R -->

立场：攻方，最终轮。分工：已定项撑不撑得住、未定项清单的完整性、报出的数字能不能核。只攻这一轮 diff（B2、H3、H2 的修补）与它们和前两轮站住的形态的交叉；前两轮攻方三份报告与两份判决列过的形（B2 越门本身、H3 次序本身、A1/A2/A3、节奏两判、开放段与活 R、`mounted_read.rs` 那句注释、门关时每次叫起重读根环）不再攻。方案按 `crates/` 今天的实现谈：开工时 `sha256sum -c research/prompts/m4-close-batch2-code-r3-snapshot/crates-sha256.txt` 两份 `OK`（量的）。

我什么都没编、没跑项目代码（子 agent 不交任务）。读了主 agent 点名的任务号的状态与日志（`task-server/task status <号>`、`task-server/task log <号> --all`，落在草稿目录 `log-<号>.txt`）。下面没标「量的」的都是推的，判决要引的数等主 agent 复跑模型。本腿原型跑的崩溃状态数：0（没写调枚举函数的原型）。

## 复跑

模型目录 `research/prompts/m4-close-batch2-code-r3-opus-model/`。复跑命令（整条交服务，见末节「要主 agent 跑的命令」）：
`bash research/prompts/m4-close-batch2-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录（工作根下、以 -run 收尾）> <模型目录>`。
外层与第二轮那一份逐字节相同（sha256 同），先删那一个运行目录，再起同目录的 `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps`）。`rerun_steps`：经 `minimal-copy.py` 拷冻结副本、核两份被判文件与开工快照的 sha256；把探针放进副本验证包当新测试目标先跑一遍（产品臂）；再在同一份副本上把停着删意图那一判的破例改成恒判门（变异臂，`&& !is_this_drop_the_exception` 换成 `&& (true || !is_this_drop_the_exception)`），跑同一条探针与 `compaction_stopped_intent_drop_judges_the_rollback_gate`、`compaction_on_the_cpu_while_stopped_after_a_gpu_failure` 两个已有测试目标。观测行以 `m4-close-batch2-r3-attack:` 起头，末行 `name=m4_close_batch2_r3_attack_summary …`。退出码只说跑没跑成，读数看末行。

```
2e4628d7e9b1ae786e79c652b75e8ccb0b323f2e12cad9a3b1e32a7bfeb0b9b2  probes/attack_stopped_drop_exception_and_the_explicit_entry.rs
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
bc8d41a01e076f8b984d54463e211717e93834e81cf8beb64746ffaab2975d67  tools/rerun.sh
```

探针只过了 `rustfmt --edition 2021 --check`（草稿目录里配两份空占位模块，量的：退 0）；`rerun.sh` 与外层过 `bash -n`；变异的锚点那一行在主工作区 `mounted_session.rs` 里恰好命中 1 次（`grep -cxF`，量的）；末行几个字段的正则拿造出来的四行 `test … ok` 试过（量的）。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| 三·1 第 4、5 条现行字面与代码逐句对 | **打中（推不出，T1）** | 「门关就不删、意图与 R 排他留到门开」不在 D26 现行字面里（`回退门` 在 D26 零命中）；第 4 条只说「吃发布预算、扣完就等」，已定项 3 第 2 条把这一次归删除类，而删除类的发布不经门、不占预算。停着删意图过门这一条只靠第二轮判决的读法撑着 | grep（量的）＋读条款 |
| 三·1 显式入口「扣完就等」 | **打中（轻，T2）** | 第 4 条点名「显式入口与后台入口」都吃预算、扣完就等；显式入口的预算取至少 1、不判门，停着时预算扣完、门关着照发删意图 | 读码＋探针第二格（待主 agent 跑） |
| 三·1 门一直不开时留多久 | 推不出（窄，T3） | 门靠用户改动打开；前台写不进的破例按「全空段数 < 底额」判，活意图的 R 按记录全空时它自己算一个全空段、却不许落点：R 是唯一空段时破例不触发、门也等不来用户状态。能不能走到取决于抬 F 能不能从 R 之外放出 COW 换下的槽，没造 | 读码（推的） |
| 三·1 与「停下都删意图」冲不冲突 | 没打中（一次抽样） | 门关时留着与「预算扣完就等」同形，都由下一次用户改动解开；条款字面没写门这一档（并进 T1） | 读码 |
| 三·2 H2 漏没漏别的入口 | 没打中（一次抽样） | 显式入口与可写挂载续做不记「没完的一趟」，没有触发名可顶 | 读码 |
| 三·2 H2 文档串比代码宽 | 推不出（轻，T4） | 新文档串说「这一趟不跨调用接着推」，代码只在排空被拒那一支不记；排空发成而把最满的盘推过补跑阈值时，这一趟不再停着、照普通整理走，停在 `OnePublishOfTheTripDone` 就照样以这个触发名记成没完 | 读码（推的） |
| 三·2 B2 × 前台写不进的破例 | **打中（测试缺口，T5）** | 停着那一判的破例分支（`is_this_drop_the_exception` 为真时不判门）走得到、没有一条测试走到；变异表里那一行把破例判反，抓的是「正常那一档不判门」，「破例照样判门」那一方向没有行 | grep（量的）＋探针第一格与变异臂（待主 agent 跑） |
| 三·2 B2 × 攒批会话、放锁入口 | 没打中（一次抽样） | 攒批：判起跑在排空之前、判门在排空之后，排空发成的那一次用户改动先进门的读数；放锁：停在门上不是 `OnePublishOfTheTripDone`，不记成没完 | 读码 |
| 三·3 任务号与引的行 | 核得到 11 个，1 处核不到 | 12435、12476–12483、12522、12523 日志里都找得到正文引的那几行（量的）；命名检查第二条（`naming-lint.sh`）正文没给任务号；12525 在跑 | `task-server/task log`（量的） |
| 三·3 A/B 口径（H3 那一行 2 对 16） | 口径对，作证范围窄（T6） | 2 与 16 都是「会话记的已过发布数」，16 等于探针自己数的根数；但两臂那一行都是 `live_intent_after_the_mount=false`、`started=false`，H3 改变起跑的那一形没量到，这一行只证「读到了」 | 日志原样（量的） |
| 四·1 走不到的理由 | 成立；附 T5 | 新加的分支都走得到；破例那一支走得到而没测 | 读码＋grep |
| 四·2 类型与名字 | 推不出（轻） | T4；另：枚举文档串与 `start_while_the_compaction_stays_stopped` 文档串说「过了回退门才单发」，破例那一档不过门 | 读码 |
| 四·3 不兼容旧代码 | 成立 | diff 上 `grep -nE 'legacy|old_format|migrat'` 退 1（量的） | grep |
| 四·4 不新建判定库 | 成立 | diff 上 prune／rocksdb／sqlite／`.tsv` 零命中（量的） | grep |
| 四·5 多机可用 | 成立 | 两份新测试文件零命中 `std::env`、`hostname`、`/home/`、`CARGO_MANIFEST_DIR`、读文件（量的）；12483 拒算 12 条、登记表 12 行 | grep＋日志 |
| 四·6 测试优先多线程 | 成立（推的） | 回退门那份按门前改动数拆成三条、H3 一条，四条各自一段历史、由 libtest 并行；另一份只有一条 | 读码 |
| 四·7 带 --item 的命令与项 | 变异那一半成立；崩溃点重放那一半待结局 | 12522「抓到 9 条、没红 0 条」（量的）；12525 我读状态时仍在跑（3 片里 2 片 running） | `task-server/task status`（量的） |
| 四·8 新崩溃流多罩了什么 | 推不出（轻） | 停着删意图、删除队列不空那一形只钉了写出的那一版；删意图那一版落盘前后崩、恢复先走删除续做（意图活着时 R 排他照在）那一段没有枚举 | 读码 |

哪一条最脆弱：T1（停着删意图过门这一条没有条款字面），见末节。

## 攻击面一：已定项撑不撑得住

### T1：「门关就不删」不在条款字面里（打中，推不出）

正文把这一条当现行字面：「门关就不删、意图与 R 排他留到门开」（`research/prompts/_m4-close-batch2-code-r3-body.md:21`）；第二轮判决交用户表写「条款字面已写明那一次照样受第 5 条约束」（`research/prompts/m4-close-batch2-code-r2-main-verification.md:40`）。现查（量的）：`grep -nF '回退门' .claude/kb/decisions/26-后台整理与放置回收.md` 零命中；`grep -nF '门开'`、`'门关'` 同样零命中。

条款里管这一格的三句：

- 第 4 条只说预算：「停着时现行那一版有活意图的，显式入口与后台入口起跑一趟、只删意图，之后不写；那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。用户定案的原话也只到预算：「照预算，扣完就等（推荐）」（`records/2026-10-13-里程碑四预开发.md:1597`）。
- 第 5 条的门管「整理发布」：「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时，这次整理发布不发」（`.claude/kb/decisions/26-后台整理与放置回收.md:254`）。
- 而这一次发布在已定项 3 里另有归类：「例外是只删整理意图、不搬单元的那一次：它归删除类，不判式子」（`.claude/kb/decisions/26-后台整理与放置回收.md:104`）。删除类的发布在第 5、6 条里都不经门、不占预算：「删除队列不空也算起跑：这一趟只走队列、不发整理，不看水位，也不被发布预算挡」（`.claude/kb/decisions/26-后台整理与放置回收.md:254`）、「那一遍删除发的版不占整理一趟的发布预算」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`）。

所以字面上两种读法都站得住：按已定项 3 归删除类，就不该判门、也不该吃预算；按第 4 条「吃发布预算」当整理发布，就该判门。第 4 条已经取了后一种的一半（吃预算），用户定的也只有这一半；判门这一半是第二轮判决主 agent 的读法（判决第二节 B2 一行「那一次删意图也是整理发布」），代码文档串把它写成条款的意思：「那一次删意图是整理发布，开关照水位判时发之前过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）」（`crates/singlefs-core/src/mounted_session.rs:517`）。改法照样站得住（A/B 12435 量过、门关不越），站不住的是「已定」这个标签：这一格是零轮读法、条款字面没写，第 4 条也没随写回补一句。

四句：
- 分不分辨臂：分辨。「删意图是删除类」那一臂停着时门关照发（就是第二轮的 B2）；「是整理发布」那一臂不发。两臂在同一段历史上读数不同（12435 两臂各 6 段历史，甲臂 `first_wakes_publishing_past_a_closed_gate=6`，乙臂 `=0`，量的）。
- 系统当时看不看得到：看得到，门要的四样那一刻都在手里（第二轮已核）。
- 满足的是哪一个分句：第 4 条「那一趟吃发布预算」与已定项 3 第 2 条「它归删除类」两句对同一次发布给了两种归类；门那一分句只认「整理发布」。
- 改法在这一格上中不中：「补字面」与「改回不判门」都修得到这一格；只改代码不补字面的，这一格照中。

推翻条件：D26 现行字面里另有一句（我 grep 的三个词之外的说法）明写停着删意图过门；或用户另有定案原话管门这一档（records 里我只核了第 1597 行那三条）。

### T2：显式入口停着、预算扣完照删（打中，轻）

第 4 条那一句点名两个入口都「吃发布预算，预算扣完就等」（同上 `.claude/kb/decisions/26-后台整理与放置回收.md:253`）。显式入口给的预算规则是 `SinceTheLastUserChange`：「CompactionRunBudget::SinceTheLastUserChange,」（`crates/singlefs-core/src/mounted_session.rs:3215`），算出来恒至少 1：「.max(1)」（`crates/singlefs-core/src/compaction_plan.rs:1577`）。停着那一支只在 `budget == 0` 时不删，显式入口走不到 0；判门又只罩后台入口：「回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）」（`crates/singlefs-core/src/mounted_session.rs:4512`）。所以显式入口在「停着、有活意图、预算扣完、门关」那一刻照发一次删意图。

条款射程里免门的只有挂载自己的发布：「第 5 条的余量撑的是有限次连崩：下一次可写挂载自己的发布不经门」（`.claude/kb/decisions/26-后台整理与放置回收.md:261`）；显式入口免门、免「扣完就等」都只在代码注释里。轻：显式入口在 `crates/` 产品代码里没有调用方（`grep -rn '\.run_compaction(' crates --include=*.rs` 去掉 `/tests/` 只命中验证包的 `history.rs` 与一个实验 bin，量的）。

探针第二格 `attack_the_explicit_entry_left_stopped_drops_the_intent_with_the_budget_spent_and_the_gate_closed`：前缀同回退门那份用例（门前 4 次用户改动、叫起到门关、切到零卡），已过发布数写成 `3S − 4`（造「扣完」，不写盘）；对照臂后台入口同一刻期望 `NotStarted`、意图留着；显式入口期望发 1 版、意图没了、停在 `BackgroundTaskStoppedAfterAnEarlierComputeError`。断言写在打中那一边。推翻条件：观测行 `explicit_entry_with_the_budget_spent` 里 `explicit_publishes=0`（显式入口另有一判挡住）。改法：条款射程把显式入口写出去（推的，被攻过零轮），或显式入口停着时也按后台入口的那一份算（推的，被攻过零轮）。

### T3：门一直不开时意图与 R 排他留多久（推不出，窄）

停着、门关、有活意图：后台入口每次叫起起跑一趟、停在门上、不写；解开靠两样——用户改动给根环添新状态（S = 8 时 12435 乙臂 6 段历史全在门开那一次删掉：`histories_with_the_intent_gone_after_a_wake_held_at_the_gate=6 histories_left_with_the_intent=0`，量的），或前台写不进的破例（停着那一判跳过门，见 T5）。

窄的那一形（推的，没造）：破例按「有一块盘的全空段数低于底额」判：「.any(|device_map| device_map.empty_segments() < floor)」（`crates/singlefs-core/src/compaction_plan.rs:1918`），全空段数按记录数：「self.full_segments_without_an_unreleased_record」（`crates/singlefs-core/src/allocator.rs:910`）。活意图的 R 按记录全空时（停着期间前台删光了 R 里的文件）它算一个全空段，产品底额 1 ⇒ 破例不触发；可分配器取落点排除 R，R 之外又满时前台写不进、加不出新状态，门等不来。能不能真卡住取决于前台被拒时推的抬 F 能不能放出 R 之外 COW 换下的槽：放得出就写得进、门迟早开；放不出（换下的都在 R 里）就一直留着，直到重挂或手动解除停下（解除之后普通整理的达成删意图同样过门）。这一形没停着时在循环里本来就有（不是这一轮的 diff），B2 改法把它带进了停着那一支：改之前停着删意图不判门，这一形不卡。

推翻条件：造出「R 按记录全空、是唯一全空段、R 之外满、门关」的盘，前台下一次改动经抬 F 写成（门迟早开），这一格就不成立；或 `empty_segments` 实际不把活意图的 R 计进去（我只读了计数字段，没读它在 R 排他下的增量维护）。

## 攻击面二：未定项清单的完整性

### H2 的「解除时重判」漏没漏别的入口（没打中，一次抽样）

「没完的一趟」只住会话、只由放锁的后台入口记（`keep_the_background_trip_under_way` 在 `crates/singlefs-core/src/` 下只有两处调用，`mounted_session.rs` 第 3634、7526 行，量的）。显式入口与可写挂载续做都走 `PacingOfTheRun::NotPaced`、整趟推、不记也不取，没有触发名可顶，不需要 H2 那一支。

### T4：H2 的新文档串比代码宽（推不出，轻）

新文档串：「这一趟不跨调用接着推：放锁的入口排空待发被拒时不记成没完的一趟」（`crates/singlefs-core/src/mounted_session.rs:519`）。代码只在排空被拒那一支按触发名跳过（`crates/singlefs-core/src/mounted_session.rs:7522`–7524）。排空发成之后那一趟交 `run_the_background_compaction_that_starts`，那里不看触发名：放锁、停在 `OnePublishOfTheTripDone` 就记（「|| is_done_with_this_call_only =>」（`crates/singlefs-core/src/mounted_session.rs:3632`），记的是 `trigger` 原样）。

走到的历史（推的，没造）：
1. 整理停着、最满的盘不超过补跑阈值、有活意图、预算还剩；攒批会话里有待发（`accept` 收下的写）。
2. 放锁的后台入口：判起跑走停着那一支、起跑 `LiveIntentWhileStoppedAfterAComputeError`（判在排空之前）。
3. 排空发成，这几笔写把某一块盘推过阈值（阈值 0 到 100 可调，第 4 条许可）。
4. 一趟开头 `compaction_stays_stopped` 判出补跑（不再停着），照普通整理推。第一次发布不是一批（R 已按记录全空的达成删意图、R 里有隔离槽的丢意图）、或删除队列不空先走一步时，下一轮停在 `OnePublishOfTheTripDone`（`crates/singlefs-core/src/mounted_session.rs:4641`、4693 行），这一趟以 `LiveIntentWhileStoppedAfterAComputeError` 记成没完。
5. 下一次调用照记下的那一趟接着推（签新意图、搬一批），报的触发仍是它。

对不上的是文档串那一句与枚举文档串「这一趟只单发一次删意图的发布」（第 515 行）。不改盘上状态，只是观测名。第二轮攻方报告提过「排空把最满的盘推过阈值时触发名不准」（整趟入口、单次调用），这里多的是跨调用记成没完那一半。改法（推的，被攻过零轮）：`run_the_background_compaction_that_starts` 记没完那一趟时，触发是这个名字就不记；或把文档串收窄到排空被拒那一支。

### T5：B2 改法 × 前台写不进的破例（打中，测试缺口）

停着那一判：「let is_this_drop_the_exception =」之后「budget_rule == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite;」（`crates/singlefs-core/src/mounted_session.rs:4523`），「if budget != 0 && is_the_rollback_gate_judged && !is_this_drop_the_exception {」（`crates/singlefs-core/src/mounted_session.rs:4524`）。破例那一档在产品上走得到：`start_while_the_compaction_stays_stopped` 判出前台写不进就给这一档的预算（第 4345–4349 行）。

谁钉着它（量的）：
- 变异表里管这一判的三行（`crates/mutations.tsv` 第 7610–7612 行）：一行把破例判反（「停着删意图之前不判回退门」（`crates/mutations.tsv:7610`），换成 `&& is_this_drop_the_exception`）、一行把入口条件判反（`!is_the_rollback_gate_judged`）、一行把预算条件判反（`budget == 0`）。三行在正常那一档都让门不判，所以都被门关着照发的那三条抓到（12476、12522）。「破例那一档照样判门」那一方向（去掉 `&& !is_this_drop_the_exception`）没有行。
- 测试：验证包 `tests/` 下抬底额的用例（`EmptyClusterSegmentsFloor::ForcedForTests` 5 处）所在的文件，没有一份同时出现停着的停因或停着删意图的触发名（`grep -rln … | xargs grep -ln …` 退 123、零输出，量的）；回退门那份新用例底额取产品值。

所以「前台写不进时停着删意图不判门」这一支只有读码，没有一条测试会在它被拿掉时红（推的，变异臂量它）。

探针第一格（`attack_the_stopped_drop_after_{four,six,eight}_user_changes_passes_the_closed_gate_while_the_foreground_cannot_write`）：同一段前缀推到门关、切到零卡；先照产品底额叫起一次（对照：停在门上、一版不发、意图留着）；再用只供测试的开关把底额抬到 `1 << 20`（每块盘都低于它，造「前台写不进」，不写盘）叫起一次，期望破例那一档发 1 版删意图、意图与 R 排他都撤了。断言写产品该有的样子：产品臂绿 = 破例照条款走；变异臂（恒判门）这三条应红、两个已有测试目标应绿 = 已有测试抓不到。末行 `mutant_caught_by_the_probe=yes mutant_survives_the_existing_targets=yes` 是这一格打中。

四句：
- 分不分辨臂：分辨（产品与变异两臂在同一段历史、同一刻，期望一发一不发）。
- 系统当时看不看得到：看得到，破例那一档在判起跑时就算好、随预算规则进来。
- 满足的是哪一个分句：第 5 条「只有全空聚簇段数低于底额（已定项 3）、前台已写不进时破例，每趟至少发 1 次」（`.claude/kb/decisions/26-后台整理与放置回收.md:254`）；打中的不是行为，是「没有会红的形式」。
- 改法在这一格上中不中：补一行变异（去掉破例那个合取）＋把探针第一格收成用例，修得到；只补用例不补变异行的，门禁不判这一方向。

推翻条件：变异臂里两个已有测试目标有一个红（`mutant_survives_the_existing_targets=no`），说明已有测试另有一条走到了破例；或产品臂第一格红在「前提」（`contrast_below_the_floor=true` 或对照没停在门上），造法要改，不算没打中。

### B2 × 攒批会话、放锁入口（没打中，一次抽样）

- 攒批：判起跑（停着那一支只看有没有活意图与预算，不看门）在排空之前，判门在那一趟开头、排空之后；排空发成是一次用户改动（预算归零重给、根环添一个状态），门读的是排空之后的根环。排空被拒走 H2 那一支、不跑这一趟。预算扣完的那一刻判起跑交 `NotStarted`、不排空：待发留着，等前台发成或下一次判起跑，同「扣完就等」。
- 放锁：停在门上是 `CompactionPublishWouldLeaveFewerThanFourRollbackStates`，不是让路也不是 `OnePublishOfTheTripDone`，不记成没完；下一次调用照停没停重判。门关期间每次叫起重读根环第二轮承接腿已报，不重复。

## 攻击面三：报出的数字能不能核

逐个读了正文第二节点名的任务（`task-server/task status <号>`、`task-server/task log <号> --all`，量的；日志拷在草稿目录 `log-<号>.txt`）：

| 正文说的 | 日志里找到的（原样，私有值没抄） | 判 |
|---|---|---|
| 证红 12476「点名 8 条：现判 8 条…抓到 8 条」 | `✓ 点名 8 条：现判 8 条、复用 0 条、拒算 0 条；跑了 8 条，抓到 8 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了` | 对上 |
| 12477 退 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.46s`；状态「结局 succeeded、退 0」 | 对上 |
| 12478 退 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s` | 对上 |
| 12479 退 0 | `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.55s` | 对上 |
| clippy 12480、build 12481 退 0 | 两份状态都是「结局 succeeded、退 0」；12481 前三次撞内存上限（退 250）、第 4 次退 0 | 对上 |
| 变异行静态核 12482 退 0 | 状态「结局 succeeded、退 0」，日志 `CHECKED	7614	785	12` | 对上（读数各列的义我没核） |
| 拒算登记 12483「拒算 12 条、登记表 12 行」 | `  · 拒算 12 条、登记表 12 行；未登记的由主 agent 逐条二次确认该不该拒、确认了才登记（list 不判红，退 0）` | 对上 |
| 命名四格 12523 退 0 | `✓ code-source-discipline.sh：4 格里 4 格绿（vague-names test-file-names ordinal-names data-names），0 格本次未跑（无）` | 对上 |
| 命名检查第二条 `naming-lint.sh` | 正文没给任务号；会话状态列表里只剩 12525 一个在跑或没人取的（量的） | **核不到** |
| 变异 12522「点名 9 行」（正文写在跑） | 已完：`计数：抓到 9 条、没红 0 条、无效 0 条、锚点失效 0 行（共 9 条）` | 对上 |
| 崩溃点重放 12525（正文写在跑） | 我读状态时：sharded，3 片里 2 片 running，计划子任务退 0 | 待结局 |
| A/B 12435 那一行 | `name=m4_close_batch2_r2_ab_verdict without_the_patch_reproduces=yes with_the_patch_holds_at_a_closed_gate=yes with_the_patch_drops_once_the_gate_opens=yes arms_failed=0` | 对上 |

### T6：H3 那一行「2 对 16」的口径对，作证的只是「读到了」

12435 末行：`name=m4_close_batch2_r2_ab_mount_budget counted_at_the_mount_without_the_patch=2 after_the_first_call_without_the_patch=2 counted_at_the_mount_with_the_patch=2 after_the_first_call_with_the_patch=16`。两臂各自那一行（节录字段，原样在 `log-12435.txt` 第 9、49 行）：`publishes_counted_by_the_previous_mount=12 live_intent_after_the_mount=false publishes_counted_at_the_mount=2 roots_after_the_last_user_change_by_the_probe=16 started=false trigger=- outcome=NotStarted(BackgroundTaskStoppedAfterAComputeError) publishes_of_the_wake=0`。

- 口径：2 与 16 是同一个量（会话的 `publishes_since_the_last_user_change`）在头一次停着的后台调用前后的值；乙臂取的 16 与探针自己数的「最后一次用户改动之后根环里读得出、不低于 F 的根数」相等，比上一次挂载记的 12 加续做的 2 多 2（推的：挂载自己的写行、暖机那几条，读法照代码注释「写行、暖机、上一次挂载的整理都在里面」）。这一句「读法对」我认。
- 作证范围：两臂都是 `live_intent_after_the_mount=false`、`started=false`，挂载续做已把意图推完，头一次调用在两臂上都不起跑。H3 要修的那一形（停着、有活意图、头一次调用拿没读根环的数判「没扣完」而起跑删意图）没走到；两臂的行为相同，差只在会话记的那个数。新用例 `the_first_background_call_left_stopped_after_a_mount_reads_the_publishes_of_the_mount_from_the_root_ring` 同样断言 `NotStarted` 加那个数（用例第 662–676 行，读码）。所以「H3 换得到东西」只到「读到了」；「换到了起不起跑」没量。第二轮承接腿自己写了这一形没造（它的报告「H3 改变行为的那一形……这里没造」那一段）。
- 推翻条件：造出「重挂之后头一次调用前停着、现行那一版仍有活意图」的前缀（例如续做的删意图被空间拒、或续做批小、预算在续做里用完），甲臂起跑删意图而乙臂 `NotStarted(BackgroundTaskStoppedAfterAComputeError)`，这一格就成了换得到行为。

## 第四节八格

1. **走不到的理由二次查实**：成立。这一轮 diff 新加的分支逐个追：停着那一判里判门（预算不为 0、后台入口照水位判、不是破例）——回退门那份用例三条走到；门关那一支 `return finish(… Stopped(stop_of_the_gate))`——同上；破例那一支（`is_this_drop_the_exception` 为真、不判门）——产品上走得到（前台写不进时 `start_while_the_compaction_stays_stopped` 给这一档预算），没有测试走到（T5）；`budget == 0` 那一支照第一轮判的仍走不到（后台入口扣完而前台写得进时不起跑，写不进时预算取至少 1）；H2 那一支——`the_stopped_drop_refused_at_the_drain_is_not_continued_under_its_trigger_once_a_switch_lifts_the_stop` 走到（12479 日志里 `ok`）；H3 挪过的那一段——新用例走到。没有走不到的；有一支走得到而没测。
2. **类型与名字**：推不出（轻），算打中。`LiveIntentWhileStoppedAfterAComputeError` 在停着那一支起跑时名实相符；不符的两处：T4（排空发成、推过阈值之后以这个名字记成没完、跨调用推普通整理）；枚举文档串「开关照水位判时发之前过回退门」（`crates/singlefs-core/src/mounted_session.rs:517`）与「那一趟开头判出同一个停着，过了回退门才单发一次删意图」（`crates/singlefs-core/src/mounted_session.rs:4320`）都没写破例那一档不过门（行内注释第 4520–4521 行写了）。新变量名 `is_this_drop_the_exception` 说得出内容。
3. **不兼容旧代码与旧格式**：成立。`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r3-diff.md` 退 1、零命中（量的）。
4. **不新建判定库**：成立。diff 上 `grep -niE 'prune|rocksdb|sqlite|\.tsv'` 退 1（量的）。
5. **所有任务多机可用**：成立。两份新测试文件零命中 `std::env`、`env::var`、`hostname`、`/home/`、`CARGO_MANIFEST_DIR`、`fs::read`、`File::open`（量的）；12483 末行「拒算 12 条、登记表 12 行」。
6. **测试优先多线程**：成立（推的）。`compaction_stopped_intent_drop_judges_the_rollback_gate.rs` 四个 `#[test]`（第 597、606、615、627 行），按门前改动 4、6、8 与 H3 拆开，各自一段历史、互不共享状态，libtest 默认并行；文件里零命中 `available_parallelism`、`thread::`（量的）。另一份一个 `#[test]`（第 72 行），量小。
7. **带 --item 的命令与项**：变异那一半成立（12522 已完，9 条全抓到，量的）；崩溃点重放 12525 我读状态时 3 片里 2 片在跑，结局待主 agent 读。另：`--crash-expand-up-to 5` 写在 12525 的命令里（状态行原样），是不是那两项的全量深度我没核。
8. **新崩溃流多罩了什么**：推不出（轻）。正文说停着删意图、删除队列不空那一形只补两条小用例；两条用例钉的是删意图那一版照抄队列、不走续做（读用例名与 12479 里 `ok`）。没罩的：那一版落盘前后崩，恢复后的可写挂载先走删除续做（落盘前崩的那几个状态里意图还活着、R 排他照在）再推或删意图——删除续做在 R 排他下取落点、续做与续推意图的先后，都没有枚举。照「崩溃点测试准确率优先」，推的是要补一条录制流（从停着删意图、队列不空那一版起）；我没写调枚举函数的原型，状态数没算。

## 哪一条最脆弱

**T1：停着删意图过回退门这一条，条款字面里没有。** 依据：D26 里 `回退门` 零命中；第 4 条只写预算、已定项 3 第 2 条又把这一次归删除类，删除类在第 5、6 条里不经门、不占预算；用户定案原话只到「照预算，扣完就等」。代码与判决都把它叫「条款的意思」，可它是第二轮判决主 agent 的读法、被攻过零轮（A/B 量的是「改了能不能做到」，不是「该不该这样做」）。一旦有人照已定项 3 的归类回读，停着删意图就回到「不判门」，B2 那一形重开。第二脆的是 T5：破例那一档没有会红的形式。推翻 T1 的现象：D26 或 records 里找到明写停着删意图过门的原话。

## 没打中的形状

- 攻击面一：「停下都删意图」与门关时留着冲不冲突（同「扣完就等」那一形）；门开得了开不了（S = 8 时 12435 乙臂 6 段全删掉，量的）；挂载续做免门（条款射程第 261 行写了）。
- 攻击面二：H2 对显式入口与可写挂载续做（不记没完的一趟）；攒批会话判起跑与判门的先后；放锁入口门上的停不记没完；`budget == 0` 在新改法下仍走不到；H3 挪动让停着的头一次调用读根环、记进分配器那份「别的读」账（读到只坏一份才记，不是写盘，不违反「一个写都不发」）。
- 攻击面三：12 个任务号逐个读日志。
取样范围：除探针两格之外全是读码，一次抽样；按规则「没打中」一次不算。

## 这一条腿自己的限度

- 探针没编过：API 照 `compaction_stopped_intent_drop_judges_the_rollback_gate.rs`（12477 编成、跑过）的写法抄，新用到的只有 `EmptyClusterSegmentsFloor::ForcedForTests`、`foreground_writes_are_below_the_empty_cluster_segments_floor`、`publish_budget_of_a_compaction_run`、`publishes_left_of_the_budget_given_by_the_last_user_change`、`MountedSession::run_compaction` 与结构体更新写法 `..CompactionSwitches::PRODUCT`（类型是 `Copy`、字段都是 `pub`，读码）。`.expect` 与 `{:?}` 用到的四个类型（`CompactionRunRefused`、`BackgroundCompactionTrigger`、`BackgroundCompactionNotStarted`、`CompactionRunOutcome`）都带 `Debug`（grep 量的）。最可能编不过的是结构体更新写法与未用到的引入（后者只是警告）。编不过时 `^error` 行会打出来，交承接的攻方腿改。
- 第一格的「前台写不进」靠只供测试的底额开关造，不是真写满；第二格的「扣完」直接写会话的已过发布数。两处都是造前提，不进被判那一判。
- T3、T4、四·8 只读码；T3 那一形没造盘面。
- records 里只核了第 1596–1598 行；D26 只 grep 了 `回退门`、`门开`、`门关` 三个词与第 104、253–255、261 行。
- 内存与时间（推的，没量）：一份副本、编验证包三个测试目标两轮（产品臂一轮、变异臂改一行之后增量重编一轮）。12477 一个目标第二台 87.873 秒、峰值 6.26 GiB，12479 117.252 秒、5.72 GiB（`task-server/task status`，量的）；推的这一条十分钟上下、峰值与 12479 相当。远低于 40 分钟，没缩取样；子 agent 不起项目代码，没先跑一小段估时。

## 要主 agent 跑的命令

1. 探针与变异臂（判决要引 T2、T5 的数靠它；在草稿目录起，冻结副本给仓根——两份被判文件此刻与快照逐份相同，脚本里先核一遍）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_stopped_drop_exception_and_the_explicit_entry（攻方探针，副本里新加，产品臂与变异臂各一次）与变异副本上的 compaction_stopped_intent_drop_judges_the_rollback_gate、compaction_on_the_cpu_while_stopped_after_a_gpu_failure; 范围=3 个测试目标; 为什么=攻方腿核停着删意图的破例那一档有没有会红的形式、显式入口停着而预算扣完照不照删' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r3-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r3-opus/attack-run /home/fy5090/code/singlefs/research/prompts/m4-close-batch2-code-r3-opus-model`
   - 在哪起：`--cwd /home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r3-opus`（运行目录 `…/attack-run` 是新目录，外层先删它再起；判决复跑照规则把模型目录拷到主 agent 自己的草稿目录、末参数换成那一份）。被范围声明闸按「范围」那一段拒的，照拒绝说明改那一段再交。
   - 看什么：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`（0 = 快照核过、产品臂编成跑完、变异打上、变异臂三个目标都编成跑完；1 = 有一步没跑成，看下面哪一行；别的码是脚本那一步）。日志里依次：`name=m4_close_batch2_r3_attack_snapshot rc=0`、`name=m4_close_batch2_r3_attack_probe arm=product rc=`（期望 0）、`name=m4_close_batch2_r3_attack_mutation rc=0`、`name=m4_close_batch2_r3_attack_probe arm=exception-judged-at-the-gate rc=`（期望非 0）、两行 `name=m4_close_batch2_r3_attack_existing arm=exception-judged-at-the-gate target=… rc=`（期望都是 0）；观测行 `arm=<臂> m4-close-batch2-r3-attack: stopped_drop_exception …`（每臂 3 行）与 `… explicit_entry_with_the_budget_spent …`（每臂 1 行）；末行 `name=m4_close_batch2_r3_attack_summary product_probe_all_ok=… exception_drops_through_the_closed_gate=… explicit_entry_drops_with_the_budget_spent=… mutant_caught_by_the_probe=… mutant_survives_the_existing_targets=…`。T5 打中 = 后两个字段都是 `yes`；T2 打中 = `explicit_entry_drops_with_the_budget_spent=yes`；`exception_drops_through_the_closed_gate=no` 时先看产品臂第一格红在哪条断言（红在「前提」是造法的事）。产物落在运行目录（`observations.txt`、`probe-product.log`、`probe-mutant.log`、`existing-*.log`、`mutation.txt`、`snapshot-check.txt`）；副本在 `attack-run.attempts/attempt.*/copy`，脚本跑完自己删，编译目录归服务（`task-server/task clean <号>`）。
   - 接着做：编不过时 `^error` 行会打出来，新派同立场的腿（「承接：」这一份，指到任务号与 `attack-run/observations.txt`）改探针。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，上面那一条与判决要引的数都没跑，报告里没标「量的」的读数都是推的。副本上量出的数（跑出来之后）不算入库装置上的数。
- T3（R 按记录全空而是唯一全空段时门等不来）、T4（排空推过阈值之后以停着删意图的名字记成没完）、四·8 的恢复那一段只读码，没造盘面。
- 没写调崩溃枚举函数的原型，崩溃状态数 0；这一腿与前面的腿合起来原型跑的崩溃状态数仍是 0。
- 12525 的结局、命名检查第二条的任务号没核到。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-batch2-code-r3-opus` 里只有文本：任务日志拷贝 `log-<号>.txt` 与 `status*.txt`、`syntax/`（探针拷贝与两份空占位，给 rustfmt 解析用）、`rerun_steps.sh`、`rerun_steps_literal.sh`、`rerun.sh`（拼脚本的草稿）；没建仓副本、没建编译目录，没有要删的。
