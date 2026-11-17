# m4-close-batch2-code-r2 辩方腿（Sonnet）：复核第一轮判决与第二轮正文第五节（2026-11-10）

<!-- doc-lint:not-numbers R -->

## 〇、范围、做法与限制

- 立场：辩方。先复核 `research/prompts/m4-close-batch2-code-r1-main-verification.md` 第二节每一格的判法够不够得着、是不是同样打中替代方案，
  再逐格回正文 `research/prompts/_m4-close-batch2-code-r2-body.md` 第五节三格。
- 开工快照核：`sha256sum -c research/prompts/m4-close-batch2-code-r2-snapshot/crates-sha256.txt` 六份都 OK，今天工作区的六份产品代码就是被判的那一版，下面的行号按今天的文件。
- 读过的：判决与正文、diff、D26 / D4 / D28 / I-5.6 的条款原文（行号去 kb 文件里现查）、六份产品代码的相关函数、验证包里点名的测试、`crates/mutations.tsv`、
  任务 12031、12045、12030 的输出（只读）。没读两份实现员报告（`~/.local/share/singlefs/work/m4-batch2-r1-fixes-writer/result.md`），所以「走不到的理由：无」那一句我没有独立核到它们。
- 没跑任何测试、没写模型、没交任务。凡写「读码推的」的，是我读源码推的，主 agent 复跑之前一律当「推的」。
- 判语：第一部分用「判决成立 / 判决不成立 / 推不出」，第二部分用「成立 / 不成立 / 推不出」；推不出算打中。
- 引 kb 条款写 kb 文件名加那份文件自己的行号。

## 要主 agent 跑的命令

以下是复跑已有用例、让本报告里「读码推的」落成读数的命令；子 agent 不交任务，整条由主 agent 交。后三条（H1、H2、H3）目前没有现成用例，要量就得新派一条写探针的腿，这里不编命令。

1. `task-server/task submit -- cargo test -p singlefs-verification --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure -- the_background_entry_left_stopped --nocapture`
   （A1 三条：删意图、预算扣完、与显式入口逐步相同；读 `ok` 条数与名字）
2. `task-server/task submit -- cargo test -p singlefs-verification --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure -- the_drop_on_a_pause_takes_its_placements_in_the_region_once_only_the_region_has_free_slots --exact`
   （A2 不带崩溃的一面）
3. `task-server/task submit -- cargo test -p singlefs-verification --test crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream -- the_deletion_without_a_batch_takes_its_placements_in_the_region_once_only_the_region_has_free_slots --exact`
   （不带一批的删意图那一核）
4. `task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail`
   （A3：读 `13 passed` 是否复现）
5. H1（待写）：后台入口停着、有活意图、根环里只剩 4 个不同用户状态时，一趟开头的删意图发不发（第三部分 H1）。
6. H2（待写）：攒批会话、停着、有活意图、排空待发被拒，之后解除停下，留下的那一趟被谁当成什么跑（第三部分 H2）。
7. H3（待写）：刚挂载的会话第一次后台调用就是停着而有活意图、根环里已过去的发布数已到预算时，删意图发不发（第三部分 H3）。

## 一、复核第一轮判决第二节（逐格）

| 格 | 判 | 证据（文件:行与原文） | 什么现象会推翻 |
|---|---|---|---|
| A1 后台入口停着时不删意图 | 判决成立（打中站得住）；判决里附的一句对条款的读法不成立 | 修补前的臂打中：任务 12031 输出第 9、10 行「background_entry_while_stopped not_started_while_stopped=[true, true, true]」对「explicit_entry_while_stopped outcome=Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError) versions_published=1」。修补后的臂不中：`crates/singlefs-core/src/mounted_session.rs:3394` 「if compaction_stays_stopped(allocator, background_compute) {」、`:3395` 「return Ok(start_while_the_compaction_stays_stopped(」，用例 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1350` 「fn the_background_entry_left_stopped_drops_the_intent_a_compensating_run_issued_and_then_writes_nothing(」。同形的另一臂（显式入口）本来就删，所以这一格不是「所有替代方案都中」，只中后台入口原先那一臂。判决写「D26 已定项 13 第 4 条「在任何写之前就报停着」限在切到一张卡都不剩、手里没有活意图那一刻」，kb 原文是 `.claude/kb/decisions/26-后台整理与放置回收.md:251`「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」，同一行还写「正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2）」，没有「没有活意图」这个限定；那句限定是判决的读法，能撑住 A1 的是用户 2026-11-10 定案（`records/2026-10-13-里程碑四预开发.md:1591`「整理暂停与出错停下都删意图、删意图那一次不判式子，用户 2026-11-10 定案」），不是条款字面 | 用户撤回「后台入口也删」；或有人造出后台入口停着、有活意图而不起跑的历史且不属于预算扣完、前台写得进那一支 |
| A2 暂停删意图那一次的落点受 R 排他 | 判决成立；对所有「R 照旧排他」的替代方案同样中 | 任务 12045 输出「drop_refused_setup … free_slots_in_the_region=[34, 34] … free_slots_outside_the_region=[0, 0]」，对照臂「drop_contrast_with_the_region_open … published=true」。根因是结构的：暂停就是因为 R 之外取不到落点，删意图那一次也要写固定点，落点那一判照判（`.claude/kb/decisions/28-挂载期承诺量.md:38` 「落点那一判照判」），所以只要 R 照旧排他，就和被拒的那一批吃同一个短缺。「等前台腾出来」也出不去：同一份输出的 `drop_contrast_user_change published=false`，前台的用户改动在 R 排他时同样被拒。唯一能绕开的是把 R 放开，条款上「在意图期内不可作分配目的地」（`.claude/kb/decisions/26-后台整理与放置回收.md:30`）与 I-5.6「活整理意图的区域里没有签发之后分配的记录」（`crates/singlefs-checker/invariants/07-分配器.md:11`）都只管意图活着的那几版，删意图落盘的那一版没有意图 | 造出暂停之后 R 之外总有够固定点用的空槽的历史（暂停的成因反过来说它不会出现） |
| A3 落后支两处经裸读者调 | 判决成立（读码） | 修补前的两处裸调，今天都改成 `crates/singlefs-core/src/mounted_session.rs:2633` 「.reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {」与 `crates/singlefs-core/src/mount.rs:7741` 「devices_behind_the_current_version(reader, &parameters_of_the_pool, current)」（外面套同一个计数层）。裸读者默认什么都不记：`crates/singlefs-core/src/recovery.rs:193` 「fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}」。同形没有第三处：全仓 `devices_behind_the_current_version(` 的调用点只有这两处加会话改动入口那一处，三处今天都在计数层里。与 D4 对得上：`.claude/kb/decisions/04-校验和位置.md:73` 「可写挂载里的读都计数，包括部分写与截断的读回」。判别力行 `crates/mutations.tsv:7601`、`:7602` | 找到第四个不经计数层读落后盘上单元的入口 |
| 正推 1.3 删意图免式子的射程比定案字面宽 | 判决成立 | 采纳的结构判据（只删整理意图、不搬单元）在代码里就是 `crates/singlefs-core/src/transaction.rs:10968` 「let deletes_only_the_intent = planned.is_empty()」；落在它里面的发布种类恰是四种：暂停与出错停下的 `IntentDroppedOnAStop`、`IntentDropped`、`IntentDroppedWithNothingMovable`、不带一批的 `IntentCompletedWithoutABatch`（`crates/singlefs-core/src/mounted_session.rs:5763`、`:5798` 一带的计划函数）。D28 不判名单今天的字面是 `.claude/kb/decisions/28-挂载期承诺量.md:38`「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」，「意图」一词能罩住这四种；射程写进 D28 已定项 1 时写明四种这一条是对的 | 有第五种只删意图而不搬单元的发布路径不经 `planned.is_empty()` |
| 正推 1.4 抬 F 第一次空发布被拒时那一读不进账 | 判决成立（不改这一批、登欠账站得住）；「差的是一次计数」仍是推的 | 条款字面 `.claude/kb/decisions/04-校验和位置.md:73` 「写入口每次发布写系统配置槽之前读最新那条根的实例表，读到的记在写入口自己的账上，发布落盘之后转进分配器」：条款写的转交点是发布落盘之后，被拒在写之前的那一次没有转交点，不算违字面。判决自己标了「推的」，我没有读数把它改成量的 | 量到被拒的那次之后，同一张实例表的同一片再没有任何一次读进账（那就不是差一次计数，是丢了这一组） |
| 四·1 `budget == 0` 走不到 | 判决成立（A1 修补之后仍成立） | 停着那一支的护栏 `crates/singlefs-core/src/mounted_session.rs:4507` 「let dropped_on_the_stop = if budget == 0 {」。三个入口给不出 0：显式入口与挂载续做都走 `SinceTheLastUserChange`（`:4487`）而 `crates/singlefs-core/src/compaction_plan.rs:1577` 「        .max(1)」把预算垫到至少 1；后台入口新加的起跑函数在预算扣完而前台写得进时直接 `NotStarted`（`:4331` 「if !has_a_live_intent \|\| (is_the_budget_spent && !foreground_cannot_write) {」），前台写不进时走 `AtLeastOnePublishWhileTheForegroundCannotWrite`，`:4502` 「            .max(1)」。会话是 `&mut self`，判起跑与起跑之间收不到切换请求，停没停不会在两次判之间翻面（读码推的） | 找到一条调用路径让「停着」是在起跑决定之后才成立 |
| 四·2 类型与名字 | 判决成立 | 判决点的 `expect` 今天在 `crates/singlefs-core/src/mount.rs:7166`（同文另在 `:6998`、`:7220`）「usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");」，行号随别的会话的补丁移了，依赖没变。本轮 diff 的新增行里没有 `expect(`、`unwrap(`、`panic!`、`assert!`、`unreachable!`（对 `research/prompts/_m4-close-batch2-code-r2-diff.md` 的 diff 段 grep 零命中） | 新增行里出现一个 |
| 四·3、四·4、四·5 | 判决成立 | 按本轮同名三格重核：`grep -nE 'legacy\|old_format\|migrat'` 对 diff 零命中；diff 只碰 `crates/singlefs-core/src/` 六份文件（`git diff` 头六行）；新增行里没有 RocksDB、sqlite、tsv、prune-server；点名的四份测试源码对 `std::env`、`hostname`、`/home/`、`$HOME` 零命中 | 出现一个 |
| 四·6 测试优先多线程 | 判决成立（已改） | `crates/singlefs-verification/tests/writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs:1292` 「let threads = singlefs_verification::heavy_threads::default_heavy_threads()」。判决说「三处改成」，我在这一批点名的测试里只核到这一处；另一处在任务 12030 点名的另一份文件（`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs`），没核 | 门禁 heavy-thread-defaults 格再点名这一批的文件 |
| 四·7 带 --item 的命令与项 | 判决成立（读登记行；`--list-items` 没跑） | `.claude/gate.d/stage-inputs.tsv:162` 「crash-case:compaction-paused-with-only-the-region-free-dropping-the-intent-in-the-region-stream」；`.claude/gate.d/crash-point-replay-items.tsv:78` 「compaction-paused-short-of-space-dropping-the-intent」那一项的第二列带这一条 | `--list-items` 点不到 |
| 四·8 新崩溃流 | 判决成立（当时缺）；它交实现员补的第 3 条补得不全，推不出 | 判决第三节第 3 条的验收是「钉住与暂停流逐步相同」。今天补到的 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1542` 「fn the_background_entry_left_stopped_writes_exactly_what_the_explicit_entry_left_stopped_writes() {」只钉后台入口与显式入口那一次调用录下的写逐步相同；两者与暂停流相同靠的是都经同一个函数 `crates/singlefs-core/src/mounted_session.rs:5943` 「fn drop_the_live_intent_on_a_stop<Device: BlockDevice>(」，没有一条用例把「停着开头那一次」的录制流与暂停流比。出错停下那一条与暂停流相同的用例早就在（`crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs:167`「fn the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes() {」）。差别只在被删的那一版，不在函数，风险低 | 造出两处入口经同一函数却录下不同的写 |
| 三·2 同实例点名窗口 | 判决成立（它只记「两腿都没打中（一次抽样）」，没据此下结论）；但回看决策表里据它写的「不影响」推不出 | `research/prompts/m4-close-batch2-code-r1-main-verification.md:7`「各抽一次，「没打中」不支撑结论」，而同一份判决 `:64` 的 D18 已定项 11 一行写「不影响 \| 2026-11-10 不受影响：窗口两腿都没打中」。规则 `.claude/rules/three-way-inference.md:158` 「拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次」，正推与攻方是两种立场，不是同一份提示的两次样本。「不影响」那一格要么等第二轮攻方的 s2 样本、要么改写成「没有读到影响，未稳」 | 同一份提示的第二次样本打中 |
| 三·4 交叉 | 判决成立 | 交叉的那一形（落后支读到坏的一份放行而不记）就是 A3 的症状，A3 一并修了，见 A3 一格 | 交叉另有一处不属于 A3 的读 |
| 回看决策：D26 已定项 13「不影响」 | 推不出 | 判决 `:61` 写「不影响 \| 2026-11-10 不受影响：后台入口停着删意图由用户定案，写回随 kb 写回批」。`.claude/kb/decisions/26-后台整理与放置回收.md:251` 的字面是「正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2）」与「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」，代码与定案都要改这两句；「要写回」与「不影响」互相打架，关系栏该写「要改」 | 条款字面与代码、定案一致 |

## 第一部分小结

- A1、A2、A3 三处打中都站得住，A2 是结构性的（对所有「R 照旧排他」的替代方案同样中），A1 只中后台入口原先那一臂，A3 没有第四处同形。
- 判决里两处措辞推不出：D26 已定项 13 第 4 条被写成「限在……没有活意图那一刻」（kb 原文没有这个限定）；回看决策表里 D18 已定项 11、D26 已定项 13 两行的「不影响」。不动判决的采纳结论。
- 四·8 的补法钉住的是入口间相同，不是与暂停流相同；低风险。

## 二、正文第五节第 1 格：代码做的是不是条款与用户 2026-11-10 定案说的

正文第二节的定案原文在 `research/prompts/_m4-close-batch2-code-r2-body.md:14`「用户 2026-11-10 定案：A1 后台入口判出停着、现行那一版有活意图时起跑一趟只删意图，之后照旧不写」。

| 定案 | 判 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| A1：停着且有活意图时后台入口起跑一趟只删意图 | 成立 | `crates/singlefs-core/src/mounted_session.rs:3394` 「if compaction_stays_stopped(allocator, background_compute) {」之后 `:3395` 返回 `start_while_the_compaction_stays_stopped`，它给出 `:4337` 「trigger: BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError,」；起跑后的那一趟在 `:4505` 「if compaction_stays_stopped(allocator, background_compute) {」那一支只调一次 `drop_the_live_intent_on_a_stop`，一批都不开、删除续做也不走（用例 `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1350` 一条断言 `run.batches_published == 0 && run.deletion_steps_published == 0`） | 停着而有活意图、预算还剩、前台写得进，后台入口却 `NotStarted` |
| A1：之后照旧不写 | 成立 | 同一条用例末尾 `:1424` 「删完之后照旧停着、不起跑」并比对盘上快照逐字节不变；没有活意图时 `:4331` 那一判直接 `NotStarted` | 删完之后再调一次，盘上多出一个写 |
| A1：那一趟吃发布预算，扣完等下一次用户改动之后那一次调用再删 | 成立；边上两处用户没说到的取舍推不出（第三部分 H1、H3） | `:4331` 「if !has_a_live_intent \|\| (is_the_budget_spent && !foreground_cannot_write) {」；用例 `:1437` 「fn the_background_entry_left_stopped_with_the_budget_spent_drops_the_intent_after_the_next_user_change(」连叫三次不起跑、`:1466` 「预算扣完时一个写都不发」、用户改动之后一趟删意图。前台写不进时按 D26 已定项 13 第 5 条破例（`.claude/kb/decisions/26-后台整理与放置回收.md:252` 「每趟至少发 1 次」）仍发一次：用户的话没提这一支，代码取的是后台入口别的起跑的通则（我没读实现员报告，判决第三节与正文第二节也没另说） | 用户要的是「破例也不删」 |
| A2：只删意图、不搬单元的那一次取落点不排除 R | 成立 | `crates/singlefs-core/src/transaction.rs:10968` 「let deletes_only_the_intent = planned.is_empty()」，`CompactionIntentStep::Drop` 与 `DeleteOnceTheRegionIsEmptyByTheRecords` 两个成员在其中为真；`:10979` 「.open_the_region_to_the_publish_deleting_the_intent()」；`:10994` 「.put_back_the_region_exclusion(exclusion_before_the_publish);」。放开的是 `crates/singlefs-core/src/compaction_plan.rs:856` 「std::mem::replace(&mut live.is_the_region_excluded_from_placement, false)」 | 删意图那一次在 R 之外有空槽也被拒，或放开后落点仍不进 R |
| A2：撤分配器侧意图的时机不变 | 成立 | `crates/singlefs-core/src/mounted_session.rs:5970` 「allocator.compaction_mut().withdraw_the_live_intent();」仍排在 `publish_the_compaction_step_once(...)?` 之后；发布出错时 `?` 先返回，意图与（已放回的）R 排他都留着 | 撤的动作移到发布返回之前 |
| A2：带一批的最后一批照旧排除 R | 成立（读码）；没有变异行单独钉 `planned.is_empty()` 那一半 | `transaction.rs:10968` 的 `planned.is_empty() &&` 让带一批的发布 `exclusion_before_the_publish` 为 `None`，不放开。即使有人去掉这半句，带一批的删意图那一核仍按 `CountedAsRecordsInTheRegion` 数记录（`transaction.rs:11205` 一带的分支），落回 R 的目的地会让核不过、照这一批不删意图再发（`mounted_session.rs` 的 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 退路），不会违 I-5.6；`crates/mutations.tsv` 里 `planned.is_empty()` 只出现在无关的两行（`:6532`、`:7293`） | 去掉 `planned.is_empty() &&` 而没有任何用例红 |
| A3：两处落后支改经计数层的读者 | 成立 | 见第一部分 A3 一格；`crates/singlefs-core/src/mounted_session.rs:2633` 与 `crates/singlefs-core/src/mount.rs:7741`；`caller_inputs_agreeing_with_the_disk` 五个调用方都把分配器交进来（`crates/singlefs-core/src/mount.rs:4783`、`:4997`、`:5347`、`:10883`，加 `parameters_of_the_pool_agreeing_with_the_caller_inputs` 的 `mounted_session.rs:3994` 一处） | 有一处调用方交进来的是别的分配器（不共用那份账） |

## 三、正文第五节第 2 格：第三节四条攻击面

### 攻击面 1：A2 的修补本身

| 问 | 判 | 证据 |
|---|---|---|
| 发布途中崩溃：恢复后 I-5.6 与续做 | 成立（读码推的；枚举流登记在册，我没跑） | 发布途中（单元、journal 记录、根槽之前）恢复择到的仍是删意图之前那一版，意图活着，它的账里没有这次发布的记录，I-5.6 逐版看账，不受影响。盘上已有这次写了一半的字节落在 R 的空槽里，但那些槽在那一版的账里是空槽。流头注释写明这一点：`crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs` 里「R 里那几个槽在那一版里是空槽，盘上却已有删意图那一次写了一半的字节」。续做：挂载按 logged_ops 里的意图装回 R 排他（`crates/singlefs-core/src/mount.rs:10063` 「allocator.compaction_mut().issue(intent);」）；journal 点名、水位之上没施加的落点预隔离（`.claude/kb/decisions/28-挂载期承诺量.md:29` 「水位之上没施加的 journal 记录点名的落点」），R 里于是有隔离槽，续做走 `crates/singlefs-core/src/mounted_session.rs:5581` 「if region_holds_a_quarantined_slot(region, &quarantined_slots) {」那一支丢意图，丢意图的发布此刻放开 R，发得成 |
| 落盘前一刻、落盘后一刻 | 成立（读码推的） | 根槽已写：版本是删意图那一版，没有意图，I-5.6 对它「不适用」；系统配置轮换没到位而被判成被抛弃根时，恢复回到删意图之前那一版，同上一行，那一版里 R 的落点按被抛弃根预隔离。每个枚举状态都跑池级 checker，I-5.6 在其列：`crates/singlefs-checker/src/image.rs:274` 「COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,」在 `IMPLEMENTED_INVARIANTS` 里，测试文件 `:405` 「for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {」逐条断言零违例 |
| 放开 R 那一窗口里别的分配能不能落进 R | 成立（读码） | 窗口是 `transaction.rs:10979` 到 `:10994` 之间那一次 `publish_compaction_step_after_its_check` 调用，期间分配器只被这一次发布用（`&mut PoolAllocator` 独占），同一次发布里的别的单元（记账树、分配记录树、映射树、树表这些固定点）本来就属于这一版，分配代都是这次的 txg，落进 R 也在没有意图的那一版里。另发那一批、前台写都是别的调用，排在放回之后。冻结这一形也对：`transaction.rs:13170` 的 `allocator.clone()` 取在放开之后（拷贝里排他为假），落盘失败 `:1663` 把分配器换回拷贝、冻结，但 `:10994` 的放回作用在换回之后的那份分配器上；冻结的那一次重发成功时 `mounted_session.rs:3328` 「self.allocator.compaction_mut().withdraw_the_live_intent();」按那一版的 logged_ops 重装（`:3322` 起），删意图那一版里没有意图，排他于是随意图一起撤 |
| 发布出错返回时排他放不放回 | 成立 | 所有出口都在 `let published = publish_compaction_step_after_its_check(`（`transaction.rs:10983`）里，放回在其后无条件执行；放开之前的 `return Err` 与 `?`（`:10964` 等）都排在 `:10979` 之前，那几处没有放开过，不需要放回。只有 panic 会跳过放回，而 panic 之后会话不再用（运行库把会话标坏） |
| 不带一批的删意图那一核不把自己的新落点算成 R 不空，会不会放过 R 里真有别的记录 | 成立（读码）；反向没有用例钉 | `transaction.rs:11213` 「record.generation != txg_of_this_publish」只把「分配代是这次发布的、未释放的」记录挪出去，上一版就在 R 里的记录（分配代必小于这次的 txg）全留着，照 `crates/singlefs-core/src/compaction_plan.rs:1439` 「record.device == region.device() && !record.is_released && region.contains_slot(record.slot)」数；判法不会放过上一版的记录。变异行 `crates/mutations.tsv:7603` 只钉「多数」的方向（把自己的落点算成 R 不空），用例是 `crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs:262`。「少数」方向（上一版有记录却放过）验证包里没有一条用例走到这一核的拒绝：对 `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted` 在 `crates/singlefs-verification/tests/` 下 grep 零命中，只在 `src/history.rs` 里有一处报告文字 |

### 攻击面 2：A1 的修补本身

| 问 | 判 | 证据 |
|---|---|---|
| 新触发与预算、节奏、让路的次序 | 推不出（H1、H3） | 第三部分 |
| 攒批会话里先排空待发（一次用户改动）再删意图 | 主流程成立；排空被拒时留下的那一趟推不出（H2）；没有用例 | 主流程：`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction_in_the_background_over_a_span` 对所有 `Starts` 一视同仁先排空再跑，新触发在 `:7506` 一带的 `match trigger` 里与 `TimerFallback`、`DeletionQueueNotEmpty` 同列不改「该跑了」；排空发成一次用户改动，把已过去的发布数归零（`compaction_plan.rs` 的 `note_a_user_change_published`），预算在 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 里按规则重算，所以排空之后的预算只会更宽。用例只有两条（`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 里 `LiveIntentWhileStoppedAfterAComputeError` 出现在 `:1372`、`:1495` 两处），都是不攒批的 `MountedSession`，攒批的会话停着删意图没有用例 |
| 切到一张卡都不剩那一刻有活意图 | 成立（按状态机同一性）；条款字面要等写回 | `compaction_stays_stopped` 只问「停没停」，不问为什么停（`:4298` 起，`WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => true`），切到没有可用卡的设置与一批出错是同一个停着，同走 `:3394`。仓里切到没有可用卡的用例（`compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs`）里的「intent」是删除意图，没有整理意图的一例。条款字面 `.claude/kb/decisions/26-后台整理与放置回收.md:251` 「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」这一刻与代码相左，用户 2026-11-10 定案在先，kb 写回批要改这一句 |

### 攻击面 3：A3 的修补本身

| 问 | 判 | 证据 |
|---|---|---|
| `caller_inputs_agreeing_with_the_disk` 与五个调用方，账并进分配器 | 成立 | 调用点今天五处都把分配器交进去：`crates/singlefs-core/src/mount.rs:4783`（准入抬 F 到上限）、`:4997`（一次准入里再推一串）、`:5347`（抬 F，卸载走同一个 `raise_the_floor_through`）、`:10883`（管理员回退），加 `parameters_of_the_pool_agreeing_with_the_caller_inputs` 的 `crates/singlefs-core/src/mounted_session.rs:3994`（推环）。账是 `crates/singlefs-core/src/allocator.rs:1945` 「other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,」，`PoolAllocator` 派生 `Clone`，各份拷贝（预演、落盘失败换回的那份、冻结的那份）共用同一份账，`allocator.rs:1994`「账各份分配器拷贝共用（读是真发到盘上的），所以 `&self` 够了。」 |
| 读文件入口 | 成立 | `crates/singlefs-core/src/mounted_session.rs:2867` 把 `&self.allocator` 交进 `refuse_a_read_on_a_device_table_other_than_the_one_of_the_mount`，落后支在 `:2633` 经计数层；该入口核不过就不读文件，账照留（计数层在读完之后记账，不看读的结局：`allocator.rs:1986`「读完（读成与否）把数到的那几组记进账」） |
| 拒了随错丢掉 | 成立 | 同上：账不在被拒的那份分配器拷贝里，在共用的 `Arc` 里，调用方把分配器换回进来时那一份，账照留。用例 `crates/singlefs-verification/tests/writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs:1076` 「fn a_device_behind_at_a_rollback_refused_after_the_check_of_the_caller_inputs_counts_the_copy() {」断言被拒在任何写之前、两块盘逐字节不变、账里仍有一份 |
| 新取样点用例（回退目标取根环里没有的 txg）是不是只靠那一核记账 | 成立（读码推的） | 目标点名 `u64::MAX`，核过之后回退在 `rollback_candidate` 就以 `NotInRing` 被拒（`mount.rs` 的 `roll_back_by_a_forward_publish`，核之后只读实例表链与根环），不读那个坏了一份的数据单元，账里那一份只能是落后支记的。变异行 `crates/mutations.tsv:7602` 点名的用例就是这一条。我没跑它；任务 12132 的「抓到 15 条」是主 agent 的读数 |
| 只覆盖了五个调用方里的哪几个 | 成立，但覆盖面窄 | 用例只经回退（`:1038`、`:1076`）、会话改动入口（`:850`）和读文件入口（`:996`）进来，抬 F、准入抬 F、推环、卸载四个调用方没有各自的用例，靠的是它们共用 `crates/singlefs-core/src/mount.rs:7741` 那一处，变异「换回裸读者」改的恰是那一处，所以会红；某个调用方交进一个不共用账的分配器，用例看不见（读码上没有这种调用） |

### 攻击面 4：`mounted_read.rs` 相对 HEAD 的全部改动

| 块 | 判 | 证据 |
|---|---|---|
| import 与 `InodeNumber(record.inode)`、`SeparatorKeyOfAnInodeLeaf(separator_key)`、`ContainerNumberOfAnInodeLeaf(...)` 三个包装 | 成立 | 只是把三个 u64 各包成类型，实参次序与旧调用相同；被调函数在 `crates/singlefs-core/src/recovery.rs:6228` 「pub(crate) fn inode_record_is_not_below_the_separator_key_and_the_container(」，判的是 `record_inode.0 < separator_key.0 \|\| record_inode.0 < container.0`，包装不改语义。调用点 `crates/singlefs-core/src/mounted_read.rs:576` 「InodeNumber(record.inode),」 |
| `mount_read_only_of_the_pool` 与共用的 `mount_read_only_from_the_system_configuration_choice` | 成立 | `mounted_read.rs:665` 「pub fn mount_read_only_of_the_pool(」只换了择系统配置的那一步（`crates/singlefs-core/src/recovery.rs:1823` 「pub fn choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot(」，与可写挂载同一个筛法），其后与 `mount_read_only` 共用同一段；用例 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:2032` 「let read_only = mount_read_only_of_the_pool(image, &FILESYSTEM_IDENTIFIER)」与 `:2076` 的「不是本池 fsid 就报没有有效系统配置」覆盖两头 |
| `open_pool_for_read` 文档：只读语义、只计数不搬 | 成立 | `mounted_read.rs:372` 「这是只读语义的那一条路：读者读到只坏一份只计数」对着 D4 `.claude/kb/decisions/04-校验和位置.md:73` 「冷恢复读与只读挂载只计数、不搬，只读挂载读到只坏一份不修」 |
| `read_at` 注释「这个累计不去重：一次调用里一个单元只读一遍，跨调用相加」 | 成立 | `mounted_read.rs:1060` 「for unit_number in span.first.0..=span.last_inclusive.0 {」一个单元一轮；单元内按提示读到只坏一份而落点不在映射位置时 `:1155` 把那一次的计数清零，所以一次解引用至多记一次；累计在 `:1031` 「self.mount.add_to_observation_since_open(observation);」先于 `:1033` 「bytes: read?,」，读到一半报错也并进（注释前半句成立） |
| 范围缺口 | 推不出 | `mounted_read.rs` 的改动依赖 `recovery.rs` 里今天工作区才有的三样（`choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot` 与两个包装类型，`git show HEAD:crates/singlefs-core/src/recovery.rs` 对这些名字零命中，`git diff HEAD --stat` 里 `recovery.rs` 是 250 行改动），而这一轮被判的文件清单里没有 `recovery.rs`。我只读了被 `mounted_read.rs` 调到的那三个符号，没有判 `recovery.rs` 的其余改动 |

## 四、正文第五节第 3 格：第四节八格逐格

| 格 | 判 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 发布出错返回时排他一样放回，逐处追过：`transaction.rs:10964` 之前的 `return Err`/`?` 都在放开之前；放开之后唯一的出口变量是 `:10983` 的 `published`，`:10994` 无条件放回；落盘失败的冻结形见攻击面 1 第三行。两份实现员报告写「走不到的理由：无」，我没读它们，不能独立核「无」 |
| 类型与名字 | 成立 | 新名字都按功能读得出：`LiveIntentWhileStoppedAfterAComputeError`、`open_the_region_to_the_publish_deleting_the_intent`、`put_back_the_region_exclusion`、`PlacementsOfThisPublishInTheRegion`、`CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion`（`crates/singlefs-verification/tests/common_compaction_prefix/mod.rs:174`）；`expect` 依赖：diff 新增行零个。命名检查四格的退 0 是任务 12168，我没重跑 |
| 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy\|old_format\|migrat'` 对 diff 零命中 |
| 不新建判定库 | 成立 | diff 只碰 `crates/singlefs-core/src/` 六份；新增行零个存判定的东西 |
| 所有任务多机可用 | 成立 | 点名的四份测试源码零命中本机状态（`std::env`、`hostname`、`/home/`、`$HOME`）；拒算登记任务 12169 的读数是主 agent 的 |
| 测试优先多线程 | 成立 | 新用例都是单场景，没有扫描；有扫描的 `writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs:1292` 取登记线程数；崩溃枚举用例走库里的 `crash_point_replay_worker_threads`（`crates/singlefs-verification/src/crash_state_enumeration.rs:250`） |
| 带 --item 的命令与项 | 成立 | 登记行见第一部分四·7；`.claude/gate.d/stage-inputs.tsv:162` 的项挂在 `.claude/gate.d/crash-point-replay-items.tsv:78` 下，任务 12167 点到 2 条是主 agent 的读数 |
| 新崩溃流多罩了什么 | 成立（头一句）；「终点够不够」推不出但风险低 | 头一句：新流 `crates/singlefs-verification/tests/common_compaction_prefix/mod.rs:174` 的枚举成员与 `:761` 的 `isolate_every_free_slot_outside_the_region_of_the_crash_flows`（只动分配器的隔离位、不写盘、不进录制流），暂停流的成员与函数没动（`the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes` 仍在，`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs:167`）。终点：流停在删意图那一版，之后重新签发在 R 上（R 里已有这次发布的落点）不在流里。R 在这几条流里本来就是装着提交内生块的那一段（流头注释「R 与批大小同整理搬迁流（第一个文件那次发布的提交内生块所在的段，一批两个）」），所以「R 里有提交内生块时再被扫」已有同形先例；崩溃状态的覆盖（途中、之后）够 I-5.6 与续做，但「删意图之后在装着自己落点的 R 上重新签发」没有枚举 |

## 五、第三部分：读码推出、没有读数的三处（推不出，算打中）

### H1：停着开头那一次删意图不过回退门

- 现象：同一件事（停着时单发一次只删意图的发布）在两处走法不同。一趟已经起跑、循环里碰到停着（补跑腾出空间、批出错、暂停）时，这次发布排在回退门之后：`crates/singlefs-core/src/mounted_session.rs:4728` 「if is_the_rollback_gate_judged && !is_this_publish_the_exception {」；一趟开头就判出停着时，`:4505` 那一支在进循环之前返回，不过门。显式入口的门本来就不判（`:4626`「回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方」），所以修补之前不算漏；A1 把「开头判出停着的删意图」接到后台入口这个产品调用方上，这条缺口才对产品路径开口。
- 条款：`.claude/kb/decisions/26-后台整理与放置回收.md:252` 「这次整理发布不发」的前半句是「一次整理发布要写的根槽连同它之后的余量里的根一走、根环里不同用户状态就少于 4 个时」，字面管「整理发布」；只有 `:252` 「每趟至少发 1 次」那一支（全空聚簇段数低于底额、前台写不进）破例，且破例只在循环里实现（`is_this_publish_the_exception`）。让路和写带宽不是问题：`:253` 「它只约束整理批（每一批之前问调用方一次）」，删意图不是批。
- 影响面：一次删意图占一个根槽，最坏让根环里可退到的不同用户状态少一个；余量取 S 槽（D26 已定项 13 第 5 条）本来就留了这个量。严重度低，但是「用户定案没说」与「条款字面」两头都没明确。
- 推翻：用户明确「停着时的删意图不受门管」；或读出 `stop_of_the_rollback_gate` 在这一支之前已被调过。
- 要量就写探针：根环里只剩 4 个不同用户状态时，后台入口停着开头那一次发不发。

### H2：攒批会话排空被拒时留下的趟，解除停下之后被当成普通一批续跑

- 现象：`mounted_session.rs:7495` 「self.session.compaction.keep_the_background_trip_under_way(」在排空被拒时把这一趟（触发为 `LiveIntentWhileStoppedAfterAComputeError`、跨度为至多一次发布时）记下；停着时每次调用都在 `:3394` 先返回，不取这一趟（取在 `:3432` 「.take_the_background_trip_under_way()」，排在停着判之后）；解除停下后下一次调用取到它，按 `:3451` 起的 `Starts { trigger: trip.trigger, ... }` 起跑，`reach` 是记下的 `AtMostOneBatchAfterAUserChange`（`:4338`），于是这一趟不再是「只删意图」，而是一趟至多一批的普通整理，触发名还是 `LiveIntentWhileStoppedAfterAComputeError`。
- 影响面：多跑至多一批；意图那时还活着，这一批正是续做它，不违反任何条款；报出的触发名与做的事不符。读码推的，没有用例。
- 推翻：解除停下时清掉这一趟，或 `:3433` 的 `.filter(\|trip\| {` 排除这个触发。

### H3：刚挂载第一次后台调用就停着时，预算没按根环读数收

- 现象：`mounted_session.rs:3394` 的停着支在 `:3409` 「&& compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()」之前返回，而 `crates/singlefs-core/src/compaction_plan.rs:991` 「publishes_since_the_last_user_change: 0,」加 `:1008` 「publishes_of_the_mount_are_still_to_be_read_from_the_ring: true,」意味着这一次挂载里还没读过根环、已过去的发布数是 0。停着支把 0 当真：预算看起来是满的，D26 已定项 13 第 5 条「挂载不再给新预算」（`:252`）在这一支不起作用。
- 可达性窄：挂载续做已经先做了一次（`crates/singlefs-core/src/mount.rs:8307` 「Some(run_compaction_on(」，停着时它走同一个停着支删意图），只有删除队列没走空、挂载续做没推的那一形（`crates/singlefs-core/src/mount.rs:8304` 「DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => None,」）或挂载续做那次删意图失败，意图才会留到第一次后台调用。
- 影响面：多发至多一次删意图。严重度低。
- 推翻：第一次后台调用之前一定有别的路径读过根环预算。

## 六、总结

- 第一部分：A1、A2、A3 打中都站得住（A2 对所有「R 照旧排他」的替代方案同样中，A1 只中原先那一臂，A3 没有第四处同形）；判决里两处措辞推不出（D26 已定项 13 第 4 条的读法、回看决策表里「不影响」两行），一处补得不全（四·8）。
- 第二部分：用户三条定案代码都做到了；A2 修补本身读码上成立，没有找到能落进 R 的第二条分配路径；A3 成立；`mounted_read.rs` 改动成立但依赖一份没被判的 `recovery.rs` 改动。
- 推不出算打中的：H1（门）、H2（留下的趟）、H3（挂载第一次的预算读数），外加 D26 已定项 13 在回看决策表里的「不影响」、D18 已定项 11 那一行据一次抽样写的「不影响」，以及攻击面 1 里「少数方向」没有用例。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没写模型、没跑任何测试；凡写「读码推的」的都是推的。
- 没读两份实现员报告，没独立核「走不到的理由：无」。
- 没判 `recovery.rs` 里与 `mounted_read.rs` 无关的改动。
- 没核任务 12093、12094、12095、12096、12132、12098、12168、12167、12169、12170 的读数，它们是主 agent 的。
