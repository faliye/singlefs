# m4-gpu-wait-outside-the-lock-code-r3：攻方腿报告（2026-11-20）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r3-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-snapshot/`（开工时现算：`crates-sha256.txt` 15 份、`kb-sha256.txt` 1 份与主工作区逐份相同）。读码引主工作区今天的行号，每处读时 `grep -nF` 核过。下文「推的」都是读码推的、没量过；探针的数要主 agent 复跑（第五节），复跑之前一律当「推的，待主 agent 跑」。

复跑命令（第五节第 1 条，整条交服务）：
`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus-run /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model`

模型文件（草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model/`）：

| 文件 | sha256 |
|---|---|
| `src/probe_relock_drop_refused_by_a_freeze_and_resent_stopped_drop.rs` | `5aeb88c9550484594018cd5a1317c081854224080ff8b7e160f05cedc3e51fa2` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`） | `c7e342faa41f54e3bdea0daad9df8f553b3df205e91d45a982f64e88a4d83af5` |
| `tools/rerun-from-a-clean-work-directory.sh`（拷自第二轮攻方模型同名文件，只删工作根下以 `-run` 收尾的那一个目录） | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

冻结副本是 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy`（`scripts/hosts/minimal-copy.py --with-test-inputs` 建，`crates-sha256.txt` 15 份对上，只在它的 `crates/mutations.tsv` 末尾追加了三行攻方候选变异，见第三节），`.deliverables` 写成「交接」。

## 各格判定一览

| 格 | 判 | 一句话 | 证据状态 |
|---|---|---|---|
| 改法第 1 条（被门压住的删意图记下、判、删、忘） | 没打中 | 修补本身没改出与条款不符的新历史；同病根另一形（删意图被冻结或空间拒掉而不是被门压住）没罩到，见形乙 | 读码推的 |
| 改法第 2 条（回锁那一刻判破例） | **打中（形甲）** | 破例照发那一次排在冻结那一判之前：锁外期间一次用户改动冻结着时，删意图被冻结拒掉，这一趟以失败收尾，把那一次冻结的用户改动改记成整理那种；重发成了照整理记账，不重给预算、不清上一条完成的意图 | 推的，探针格一待主 agent 跑 |
| 改法第 3 条（停着时一趟开头删意图发成清零） | 没打中 | 修补那一处做对了；同一次删意图冻结、之后重发成了的那一条路不清，见形丙（归第二节第 3 条） | 推的，探针格二待主 agent 跑 |
| 改法第 5 条（签发那一支补测试） | 没打中 | 四条用例各让对应变异红；资格「钉住与隔离的槽」那一项只用隔离一样造，只改坏钉住那一半照样绿（候选变异） | 读码推的；候选证红待主 agent 跑 |
| 第二节第 1 条：待删四步与绕开取输入的路 | 形乙（没罩到的同病根） | 全仓只有两处调 `take_the_input_of_one_compaction_batch`、锁里一口气推完那一路也经它，绕不开；回锁出错那一支删意图被冻结（或空间）拒掉时会话不记，整理没停就在这条放弃的意图下接着搬 | 推的，探针格一待主 agent 跑 |
| 第二节第 2 条：回锁破例逐字、与冻结和写带宽的次序 | 打中（形甲） | 判法逐字对得上；次序上破例先于冻结那一判，冻结着照去删、被拒，引出形甲与形乙 | 同上 |
| 第二节第 3 条：一批 = 一次整理发布之后还有哪处不清 | **打中（形丙）** | 冻结之后经 `resend_the_frozen_publish` 重发成了的整理发布只扣预算、记根槽，不清两个计数（也不记写带宽） | 推的，探针格二待主 agent 跑 |
| 第二节第 4 条：只供测试的开关、用例测的是什么 | 没打中，两条候选变异罩不到 | 开关产品路径上一处不调；新用例测的是条款的量；破例那一判的「会话记着」合取项、资格的「钉住」一半新用例罩不到 | 候选证红待主 agent 跑 |
| 第二节第 5 条：kb 两处改动与代码 | 没打中 | 第 5 条补的那句逐句对得上；第 6 条「下限到了」已删，代码照用户原话做；「取了输入」一词另有一读，第二轮判决第 4 条已定读法 | 读码推的 |
| 检查单九格 | 九格都成立（「类型与名字」附一条） | 见第四节 | 读码核的 |

## 一、形甲：回锁出错那一支删意图被冻结拒掉，冻结的用户改动被改记成整理（改法第 2 条，打中）

历史（每一步指到许可它的那一句）：

1. 后台入口照水位判、在单元区头一段有活意图、根环装满不同用户状态，放锁的入口取输入、交一批到锁外（取输入那一刻门开）。
2. 锁外期间前台推空发布到门关，再有一次用户改动写盘报错、冻结；冻结的种类由用户改动那一路记成用户改动（`crates/singlefs-core/src/mounted_session/floor_raise_for_space.rs:263`「KindOfTheFrozenPublish::UserChange,」）。锁外期间前台写把全空聚簇段用到底额以下（探针用只供测试的底额开关模拟，判的谓词与产品同一个）。
3. 锁外那一批 GPU 实现交回出错（重复键：整理不停；或掉卡：整理停下）。
4. 回锁：出错先于一切，删意图之前先判破例（修补第 2 条，「if foreground_cannot_write_under_the_switches(allocator) {」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:798`）），破例成立 → 不看门、照去删意图（「parameters, devices, allocator, compaction, current, instance, stop,」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:710`））。删意图那一次发布在发布路径第一道就被冻结拒掉（`crates/singlefs-core/src/transaction/persisting_a_publish.rs:130` `refuse_while_a_publish_is_frozen`），交回错。
5. 一批推完之后记账的出错那一支照「这一趟开头冻没冻着」改记冻结的种类：「run.was_frozen_when_the_run_started,」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:664`）、「KindOfTheFrozenPublish::CompactionPublish,」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:665`）；放锁的入口这一项是取输入那一刻记的（假），于是改记：「if was_frozen_before_this_call {」（`crates/singlefs-core/src/mounted_session.rs:462`）不拦，「allocator.freeze_publish(frozen.of_the_kind(kind));」（`crates/singlefs-core/src/mounted_session.rs:466`）把用户改动那一次换成整理那种。
6. 之后重发成了（运行库维护线程或前台 fsync 都经同一个 `resend_the_frozen_publish`），照整理那种记账：「KindOfTheFrozenPublish::CompactionPublish => {」（`crates/singlefs-core/src/mounted_session.rs:1135`）、「self.compaction.publishes_since_the_last_user_change += 1;」（`crates/singlefs-core/src/mounted_session.rs:1136`），而不是用户改动那一支的「self.compaction.note_a_user_change_published();」（`crates/singlefs-core/src/mounted_session.rs:1126`），也不清上一条完成的意图、不记「该跑了」。

与条款：「冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）；冻结的是后台批，照后台批记」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）。

为什么是修补之后才写得出：修补之前回锁那一判不看破例，第 4 步里门关、现行那一版变了就判压住（`Stopped`，没有错交回），第 5 步走不到。同一个病根（第 5 步拿取输入那一刻的冻结与否判「这一次冻没冻」）在门开、现行那一版没变的那一形修补之前就有（探针 `forced_start` 那一面量的就是它）；修补第 2 条多开了一条进它的路。

四句：
- 分不分辨臂：分辨。对照臂（锁外那一批不出错、其余相同）回锁照冻结丢批（「if allocator.frozen_publish().is_some() {」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:757`）），交回不是错，种类不改；出错两臂改。
- 系统当时看不看得到判别它的东西：看得到。回锁那一刻 `allocator.frozen_publish()` 在、带着种类；删意图被拒的原因就是「冻结着」那一种，不是这一次写盘报错。
- 满足判据字面的哪一句：第 257 行「冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）」。
- 改法在打中的格上中不中：第二轮判决改法第 1、2 条都碰不到这一格（它们只管门）。自己提的改法见第八节，只在我的模型上量过（还没量）、被攻过零轮。

探针格一（`relock_drop_refused_by_a_freeze_in_the_window`）推的读数：出错的臂 `kind_before_the_relock=UserChange kind_after_the_relock=CompactionPublish`、`publishes_after_the_resend` 不是 0；对照臂两项都不变、`publishes_after_the_resend=0`；汇总行 `error_relabelled` 等于出错臂的段数、`control_relabelled=0`。推翻条件：复跑出 `error_relabelled=0`，或出错臂 `relock=` 不是 `failed:` 起头。

## 二、形乙：删意图被冻结（或空间）拒掉时会话不记，整理没停就在放弃的意图下接着搬（改法第 1 条没罩到的同病根）

历史：同形甲第 1–3 步，只是回退门那一面放开（门不判的 `forced_start`，或门关而破例成立的 `gate_closed_floor_crossed`），出错取「重复键」（整理不停，`BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned`）。

1. 回锁出错那一支删意图被冻结拒掉（同形甲第 4 步）。会话只在门压住时记下这条意图：「compaction.hold_back_the_drop_of_the_intent_of_an_abandoned_batch(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:702`）在 `is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate` 为真那一支里；删意图交回错这一支什么都不记。删意图那个函数自己的说明写的就是只靠停着那一趟再删：「下一次整理再走到停下时再删；会话记着的那一条照旧记着。」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1449`）。
2. 冻结的那一次重发成了，意图仍在现行那一版里。
3. 下一次任一入口（用户决定的两步放开扫：重发之后建 0 或 1 个 inode、走后台入口或显式入口）：整理没停着，不进停着那一支；取输入那一刻会话没记着 → 照常开这一批（「compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:51`）之后照常往下），在这条放弃的意图下搬单元。

与条款：「正在算的那一批放弃，意图也删掉」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`）；重复键那一种「那一批的意图照第 4 条一批出错放弃的规矩删」（`.claude/kb/decisions/26-后台整理与放置回收.md:259`）。第二轮判决第 1 条判同一件事（「意图被接着搬」）时依据的就是这两句；第 256 行补的那一句只说「被门压住的」，所以这一形不归第 5 条，归第 4、8 条。

不是修补之后才写得出：修补之前门压住与冻结拒掉都走到「接着搬」；修补只把门压住那一形收了。空间拒掉那一形（R 里外都取不到落点，同一处说明）读码同样走到接着搬，没造历史（推的）。掉卡那一臂整理停着，下一趟开头那一支照删（「if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:347`）），不接着搬，是探针里的对照臂之一。

四句：
- 分不分辨臂：分辨。掉卡臂与不出错的对照臂读数 `relocated_under_the_abandoned_intent=false`，重复键臂（至少显式入口、建了 1 个 inode 的后台入口两种）推的是 `true`；重发之后建 0 个 inode 的后台入口那一段推的是不起跑（`not_started:NothingDueAndTheTimerFallbackOff`，这本身是形甲的后果：重发成了的用户改动没记「该跑了」），不搬。
- 系统看不看得到：看得到，删意图交回的就是错。
- 满足字面的哪一句：第 255 行「意图也删掉」与第 259 行「照第 4 条一批出错放弃的规矩删」。
- 改法在这一格上中不中：第二轮判决改法第 1 条（只在门压住时记）不中。

探针格一同时读这一形：汇总行 `same_entry_key_relocated_under_the_abandoned_intent` 推的 ≥ 2（门不判那一面的显式入口两段、建 1 个 inode 的后台入口一段；门关那一面后台入口停在门上不搬，显式入口照搬），`device_lost_relocated_under_the_abandoned_intent=0`、`control_relocated_under_the_intent=0`。推翻条件：复跑出 `same_entry_key_relocated_under_the_abandoned_intent=0`。

## 三、形丙：冻结之后重发成了的整理发布不清让路的两个计数（第二节第 3 条，打中）

历史（与修补第 3 条那条用例 `a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row` 同一个前缀，只多一处故障）：

1. 选 `gpu`、强制起跑、前台在等会话锁，后台入口让一次路（连着让了 1 次）。
2. 显式入口推签发那一批时 GPU 实现交回出错，手里没有活意图，整理停下，计数照旧是 1。
3. 签一条意图；后台入口判出停着而有活意图，一趟开头单发一次只删意图的发布——这一次写盘报错、冻结。交回错那一支照整理那种记冻结的种类（「KindOfTheFrozenPublish::CompactionPublish,」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:388`）），不清计数（清零只在发成那一支：「compaction.clear_the_yields_in_a_row();」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:374`））。
4. 撤写错，重发成了：整理那种只扣预算、记根槽（`crates/singlefs-core/src/mounted_session.rs:1135-1138`），两个计数照旧是 1。
5. 用户决定的一步：请求一次通过校验的切换解除停下，前台一直在等、下限 2 次。对照臂（第 3 步不摆写错）让 2 次之后交一批到锁外；冻结臂让 1 次就交（计数从 1 接着数）。

与条款：「一批发成两个都清零；取了输入而没让路的那一批也把两个计数清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`），第一轮判决第 8 条定「一批 = 一次整理发布」；同一行「冻结的是后台批，照后台批记」。删意图那一次是在重发成了那一刻才发成的，照后台批记就要清零。全仓清零只三处（`crates/singlefs-core/src/mounted_session/compaction_run.rs:374`、`:559`、`:613`，`grep -rn clear_the_yields_in_a_row` 与 `break_the_yields_in_a_row_by_a_batch_not_yielding` 现数），重发那一处不在里面。同一处也不记这一次写的字节（写带宽，第 6 条「GC 占盘写带宽有一个上限」），读码推的。

别的冻结路走不到不清：搬一批与回锁时冻结的那一批，取输入那一判没让路已经清过（`crates/singlefs-core/src/mounted_session/compaction_run.rs:559`），冻结那一刻计数是 0；产品运行库冻着时整理不跑（「冻着时 `publish_when_due` 与整理都不跑」（`crates/singlefs-mount-runtime/src/lib.rs:808`），读码推的），冻结到重发之间不会再让路。只有停着时一趟开头那一次删意图不经那一判，冻结时计数可以不是 0。

四句：
- 分不分辨臂：分辨（对照臂清、冻结臂不清；探针格二两臂只差第 3 步摆不摆写错）。
- 系统看不看得到：看得到，重发那一处知道冻结的是整理那种。
- 满足字面的哪一句：第 257 行「一批发成两个都清零」，经同一行「冻结的是后台批，照后台批记」接到重发成了的那一刻。
- 改法在这一格上中不中：第二轮判决改法第 3 条只改发成那一支，不中。

探针格二（`stopped_drop_frozen_at_the_trip_start_and_the_yields_in_a_row`）推的读数：`control_yields_in_a_row_after_the_drop=0 frozen_yields_in_a_row_after_the_resend=1 control_yields_before_a_take_without_yielding=2 frozen_yields_before_a_take_without_yielding=1`。推翻条件：复跑出 `frozen_yields_in_a_row_after_the_resend=0`。

## 四、正文第二节五条逐条

### 1. 待删：记下、判、删、忘，与绕开取输入的路

- 绕开的路：`take_the_input_of_one_compaction_batch` 全仓两处调用（`crates/singlefs-core/src/mounted_session/compaction_publish.rs:234`、`crates/singlefs-core/src/mounted_session/compaction_run.rs:728`，`grep -rn` 现数）。锁里一口气推完的入口（显式入口、后台入口整趟、可写挂载续做）每一批走 `advance_compaction_by_one_publish`，它第一步就是前一处；放锁的入口是后一处。可写挂载续做在新挂载里，待删只住内存，进不来。停着时一趟开头那一支在 `begin_a_compaction_run` 里自己删（删成同样忘，「compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1479`））。删除续做在整理批之前走，只发队列那几步、不搬整理的单元，不算「在这条意图下接着搬」。没找到绕开的路（读码推的）。
- 记下：只在门压住那一支（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:702`），记的是现行那一版的活意图；锁外期间前台改不了活意图（会话记有一批在锁外时整理的别的入口都拒），所以记的就是票里那一条。被冻结或空间拒掉的不记 → 形乙。
- 判：取输入之前在每批之前那几判里判（预算、推到哪、至多一次发布、让路、写带宽、回退门；破例时跳过让路与门），然后在取输入第一步删（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:34-50`）。与停着那一次删意图的两处差别：一是这一次问让路、受写带宽挡，停着那一次都不问（`crates/singlefs-core/src/mounted_session/compaction_run.rs:347-402`）；二是这一次在门开、前台在等时让路，第 256 行补的那句「门开或破例成立就只发这一次删意图」按字面是照发。第 257 行又写让路「只约束整理批（每一批取输入之前问调用方一次…）」，而第一轮判决第 8 条把一次整理发布算一批：两读都说得通，判推不出，交主 agent。
- 删：照放弃那一批时的停因收尾（`IntentDroppedOnAStop { stop }`，`stop` 是记下时的 `BatchAbandonedAfterABackgroundComputeError { task_after_the_error }`）。预算扣 1、记根槽、记写带宽、清两个计数（`crates/singlefs-core/src/mounted_session/compaction_run.rs:604-613`），这一趟停、不留着（`stop_holds_back_the_trip_without_ending_it` 把这一条判成 false）；用户改动之后那一趟被它截住时「该跑了」不放回去，搬一批要等下一次用户改动或定时兜底——与停着那一次删意图同一个处置，第 256 行「就只发这一次删意图」，对得上。停因里的 `task_after_the_error` 是放弃那一批时的：停下之后经切换解除了，这一趟仍报 `LeftStopped`。条款没定这一次报什么停因，不判打中，记一条观测口径（读码推的）。
- 忘：删成（`:1479`）、取输入那一刻现行那一版里已不是它（`:51`）两处。冻结：删意图那一次写盘冻结，记着的照旧记着，重发成了之后现行那一版里没有意图，下一次取输入走 `:51` 忘掉——对；而重发之后、下一次取输入之前，`has_an_intent_of_an_abandoned_batch_still_to_drop`（pub，`crates/singlefs-core/src/compaction_plan.rs:855`）照旧答真，名字说的是「还没删」。门关：不忘、不删、不搬，对。破例：照删，对。活意图换成别的：只有冻结重发一条路会让它变，走 `:51`。

### 2. 回锁那一刻判破例，与冻结、写带宽的次序

- 逐字：第 256 行「破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发」；判法 `foreground_cannot_write_under_the_switches`（`crates/singlefs-core/src/mounted_session.rs:521`）是「开关照水位判」与「任一块盘全空段低于底额」两样，与停着那一次、每趟第一次发布用的是同一个谓词。回锁那一刻在此刻的分配器上判、排在门那一判之前、不看取输入时定的档：对得上。
- 次序：回锁里出错先于一切，破例照发排在冻结那一判（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:757`）之前；冻结着照去删，被发布路径第一道拒掉 → 形甲、形乙。写带宽回锁那一刻不判（第 257 行「回会话锁里发布那一刻不再问」），对得上。
- 取下一批之前那一次的破例（修补第 1 条那一处，`crates/singlefs-core/src/mounted_session/compaction_run.rs:507-512`）此刻判，跳过让路与门，写带宽照判：与每趟第一次发布的破例同一个处置，对得上。旁注：普通的整理批破例只看起跑那一判定的预算档，起跑之后排空待发把全空段推到底额以下的那一趟照判门；修补只给待删那一次改成此刻判。第 256 行「每趟至少发 1 次」在这一趟上不成立、下一趟开头补上（推的，没造历史，不报打中）。

### 3. 「一批 = 一次整理发布」之后还有哪一处发成而不清

形丙：冻结之后重发成了的整理发布（`resend_the_frozen_publish` 整理那一支）。别的整理发布发成处：一批推完记账（`:613`）、停着那一趟开头（`:374`），都清；回锁发成走同一份记账（`crates/singlefs-core/src/mounted_session/background_compaction_out_of_the_lock.rs:645`）。全仓 `clear_the_yields_in_a_row` 现数三处调用（含 `break_the_yields_in_a_row_by_a_batch_not_yielding` 里那一处）。

### 4. 只供测试的开关；新用例测的是条款还是实现

- 开关 `note_a_cluster_segment_opened_in_this_mount_for_tests` 全仓只有一处调用，在用例里（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:563`）；产品路径一处不调（`grep -rn` 现查）。它是 `pub`、不带 `cfg(test)`，与仓里别的 `_for_tests` 一样。
- 空洞数那条用例找前提点时用的是产品自己的 `holes_outside_the_region_in_segments_holding_user_data`（同一个函数既定前提又被测）：函数本身写错时前提跟着错。用例断言的是「签了」这一结局，不是那个数，变异第 6 行改的是喂给它的记录，照样红；不算罩不到，记一条（读码推的）。
- 要搬数那条断言 `units_to_move == records_in_the_region_now`：条款说「各个量取现行那一版的」，实现取的是未释放记录数这个上界。这一份几何里选中那一段全是数据单元，上界与在现行那一版上重扫出的能搬数相等，所以断言没把实现碰巧的上界钉死成条款（推的）。
- 资格两个合取项：「这次挂载开过的段」那条用例罩得到；「钉住与隔离的槽」那条只用隔离造，钉住一半罩不到（候选变异，第五节）。待删那几条的破例判法里「会话记着」那个合取项，修补新用例罩不到（候选变异）。

### 5. kb 两处改动与代码

- 第 5 条补的那一句逐句：「回锁里出错那一支要删的意图被门压住的，同样留到门开」——记下那一支（`:702`）；「之后不论走哪条路，取下一批的输入之前都先照这道门与破例判它」——判那一处（`crates/singlefs-core/src/mounted_session/compaction_run.rs:507-512`、`:574`）与取输入第一步；「门开或破例成立就只发这一次删意图，门关就这一趟不发、也不在这条意图下接着搬」——对得上（让路那一读见第 1 条）；「「待删」只记在这次挂载里，卸载重挂之后照盘上那条活意图续做」——只住 `CompactionOfThisMount`，新挂载 `None`，可写挂载续做照盘上那一条推。对得上。
- 第 6 条：「下限到了」在 kb 文件里现查 0 处（`grep -nF`），留下「一批发成两个都清零；取了输入而没让路的那一批也把两个计数清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）。代码在问过让路而没让那一刻清（`crates/singlefs-core/src/mounted_session/compaction_run.rs:559`），之后写带宽或门停下、没真取输入的也清——第二轮判决第 4 条定的读法就是这一形，用例 `a_take_of_the_input_without_yielding_breaks_the_yields_in_a_row_even_when_its_batch_publishes_nothing` 断言的也是它。对得上。变更史两条都在 `.claude/kb/decisions-history.md`（第 15485、15488 行一带，`grep -nF` 现查）。

## 五、新用例与变异行（关的第 2 条）

这一批 10 行（开工快照 `mutation-rows-of-the-batch.tsv`，`crates/mutations.tsv` 里各命中 1 次，`grep -cF` 现数），逐行读码判改坏之后点名的用例红在哪一条断言。点名都在测试目标 `compaction_batch_computed_out_of_the_session_lock` 里，下面只写子模块前缀之后的名字。全部读码推的；合入后验证第 2 步主 agent 量到「抓到 10 条」，与下表一致。

| 行 | 改坏哪一处 | 点名的用例 | 红在哪条断言 | 判 |
|---|---|---|---|---|
| 门关时会话不记被压住的删意图 | `compaction_batch.rs:702` 那一调用换成 `let _` | `compute_errors_and_the_rollback_gate_closed_in_the_window::after_the_same_entry_key_twice_in_one_batch_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens` | 「会话记下这条被门压住的删意图」 | 罩得到 |
| 取下一批之前不删被压住的意图 | `:34-36` 加 `.filter(|_| false)` | `…::after_a_compute_error_lifted_by_a_switch_requested_before_it_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens` | 交到锁外那一支的 `panic!`（门开之后不该再交一批） | 罩得到 |
| 回锁删意图之前不判破例 | `:798` 换成 `if false` | `…::a_compute_error_at_the_relock_after_the_floor_was_crossed_in_the_window_drops_the_intent_behind_the_closed_gate` | 「破例成立：门关着照发那一次只删意图的发布」（发了 0 次） | 罩得到 |
| 停着时一趟开头删意图发成不清计数 | `compaction_run.rs:373-374` 删清零 | `yields_to_a_foreground_entry_waiting_for_the_session_lock::a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row` | 「停着时一趟开头的删意图发成：连着让路的次数清零」（照旧 1） | 罩得到 |
| 重判资格不看钉住与隔离的槽 | `compaction_batch.rs:968` 区间换成空 | `issues_at_the_relock::an_issue_at_the_relock_on_a_segment_with_a_slot_quarantined_in_the_window_is_given_up` | 停因不是 `ChosenSegmentNoLongerEligibleAtTheRelock` | 罩得到 |
| 空洞数取票里那一版的记录 | `:1000` 换成票里那一版 | `issues_at_the_relock::an_issue_at_the_relock_counts_the_holes_of_the_segments_holding_user_data_in_the_current_version` | 「现行那一版的空洞够装下 R 里要搬的：回锁签发了」（发了 0 次） | 罩得到 |
| R 的记录变了要搬数仍取锁外扫出的 | `:985` 换成 `if true` | `issues_at_the_relock::an_issue_at_the_relock_after_the_records_of_the_region_changed_in_the_window_takes_the_unreleased_records_of_the_current_version_as_the_units_to_move` | 要搬数是 1：空洞 ≥ 1 时签了、`let … else` 的 `panic!`；空洞是 0 时 `assert_eq!(units_to_move, …)` | 罩得到 |
| 重判资格不看这次挂载开过的段 | `:966-967` 删那一合取项 | `issues_at_the_relock::an_issue_at_the_relock_on_a_segment_opened_by_this_mount_in_the_window_is_given_up` | 停因不是没资格 | 罩得到 |
| 取下一批之前那一次破例只看预算档 | `compaction_run.rs:510-512` 换成 `|| false` | `compute_errors_and_the_rollback_gate_closed_in_the_window::the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor` | 「取下一批之前此刻破例成立：门关着照发…」（停在门上、发 0 次） | 罩得到 |
| 一批发成不清连着让路的计数（替换行） | `compaction_run.rs:613` 换成占位 | `yields_to_a_foreground_entry_waiting_for_the_session_lock::a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row` | 「一批发成，连着让路的次数清零」（显式入口不经取输入那一判，没有别处清） | 罩得到 |

罩不到的，写成候选变异，追加在冻结副本 `copy/crates/mutations.tsv` 末尾（六段、`\n` 表换行，原文在副本里各命中 1 次，`python3` 现数），点名写带模块前缀的全名：

| 候选变异名 | 改坏 | 点名的用例（推的：照样绿） | 为什么罩不到 |
|---|---|---|---|
| 攻方候选：回锁签发重判资格只看隔离的槽、不看钉住的槽 | `crates/singlefs-core/src/mounted_session/compaction_batch.rs:960-964` 那一并集换成只取会话的隔离集 | `issues_at_the_relock::an_issue_at_the_relock_on_a_segment_with_a_slot_quarantined_in_the_window_is_given_up` | 用例只在 R 里造隔离的槽；锁外期间钉住的单元（映射树节点、树表、实例表）落进 R 的那一形没有用例。产品路径上走不走得到（前台的新分配落不落进没排他的 R 的空洞）我没查清，推不出 |
| 攻方候选：取下一批之前此刻全空段低于底额就破例、不看会话记没记着被门压住的删意图 | `crates/singlefs-core/src/mounted_session/compaction_run.rs:510-512` 换成只判底额 | `compute_errors_and_the_rollback_gate_closed_in_the_window::the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor` | 用例里会话记着那一条，两种写法同答；没记着而排空把底额推过去的那一趟（改坏之后这一批也破例、跳过门）没有用例。别的目标有没有罩到它的，推不出 |
| 攻方候选：取输入时会话记着的那一条已不是现行那一版的活意图也不清 | `crates/singlefs-core/src/mounted_session/compaction_batch.rs:51` 删那一行 | `compute_errors_and_the_rollback_gate_closed_in_the_window::after_the_same_entry_key_twice_in_one_batch_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens` | 用例里删成那一处（`:1479`）先忘了；`:51` 只在删意图冻结后重发成了的那一形起作用，行为上只差 pub 的 `has_an_intent_of_an_abandoned_batch_still_to_drop` 读数（近乎等价，价值低） |

证红命令写在第七节第 3 条。三条都推的是「没红」：没红就坐实罩不到；红了就是我推错了，照红的那一条撤回。

## 六、检查单九格

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 合成补丁 `crates.patch` 里 `crates/singlefs-core` 下的加行，`grep -E 'unreachable!|todo!|unimplemented!|panic!|assert|\.expect\(|\.unwrap\(|_ =>'` 0 行（产品代码没新加走不到的断言与 `expect`；新函数 `stop_of_the_abandoned_batch_whose_intent_is_still_to_drop` 用 `?` 与 `then_some`） |
| 类型与名字 | 成立，附一条 | 待删住一个带名字的结构 `IntentOfAnAbandonedBatchStillToDrop { intent: CompactionIntent, stop_of_the_abandoned_batch: CompactionRunStop }`（`crates/singlefs-core/src/compaction_plan.rs:622-625`），没有裸整数、缩写与 `_ =>`；用例里补丁加行 `grep -E 'other =>|_ =>|unwrap\(\)'` 0 行。附：`has_an_intent_of_an_abandoned_batch_still_to_drop` 是 pub、只看记没记着，删意图冻结后重发成了到下一次取输入之前答真，与名字与说明「还没删」不符（第四节第 1 条「忘」） |
| 不兼容旧代码与旧格式 | 成立 | 补丁不改盘上格式、不加版本号；待删只在 `CompactionOfThisMount`，新挂载初始化成 `None`（`crates/singlefs-core/src/compaction_plan.rs:729` 一带） |
| 不新建判定库 | 成立 | 补丁加行 `grep -E 'Library|prune|RocksDb|rocksdb|sqlite'` 0 行；新用例不调池级 checker 的判定库那一套 |
| 所有任务多机可用 | 成立 | 补丁加行 `grep -E 'hostname|/tmp|env::var|std::env'` 0 行；新用例只用 `build_pool` 的内存盘与替身 GPU |
| 测试优先多线程 | 成立 | 新用例各自 `build_pool(<各自的名字>)`，补丁加行 `grep -E 'thread::spawn|#\[ignore\]|serial|static '` 0 行，没有取进程级锁 |
| 带 --item 的命令与项 | 成立 | 10 行第五段都是 `-- <子模块>::<名字> --exact`，第六段是裸名；判法按「全名等于它、或以「::<它>」收尾」认（`scripts/gate/lib_libtest_result_lines.py:182`），合入后验证量到「基线不绿 0 行」 |
| 新崩溃流多罩了什么 | 成立（本轮没有这一类） | 补丁改的 8 份文件里没有 `crash_enumeration_*`、没有新的崩溃点重放流（`crates.patch` 的 `diff --git` 行现数 8 份） |
| 放的位置 | 成立 | 新用例在 `crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/` 的三个子模块里；开关在 `crates/singlefs-core/src/allocator.rs:258`；补丁没有新包、没有新的 `#[path]` 与 `include!` |

## 七、要主 agent 跑的命令

1. 探针复跑（模型格一、格二；不调崩溃枚举，崩溃状态 0 个）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus-run /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `name=rerun_done probe_exit=0`、两行 `SELF … premises_held=20/20` 与 `premises_held=2/2`、两行 `_summary`。编不过或前提不成立时 `probe_exit` 不是 0，交承接的攻方腿改。
   - 估时（推的）：格一 20 段、格二 2 段，每段一个池；门关那一面每段建 30 个 inode、推至多 48 串空发布。比第二轮攻方探针（14 段）多一些，挂钟由验证包的编译主导，推的在 40 分钟以内，没量。
2. 复跑脚本自证：
   - 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model/tools/rerun.sh --selftest`
   - 在哪起：仓根
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`，日志里「✓ 自检通过（查了 6 格）」
3. 三条攻方候选变异证红（推的是三条都「没红」）：
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy singlefs-verification '攻方候选：回锁签发重判资格只看隔离的槽、不看钉住的槽' '攻方候选：取下一批之前此刻全空段低于底额就破例、不看会话记没记着被门压住的删意图' '攻方候选：取输入时会话记着的那一条已不是现行那一版的活意图也不清'`
   - 在哪起：仓根（`--copy` 指冻结副本，副本 `.deliverables` 写成「交接」）
   - 看哪几行：每条判定行是「没红」还是「抓到」，汇总句的「跑了 N 条，抓到 M 条」与「基线」那几行；退出码按 prove-red.sh 文件头（有没红的那一条时不是 0，这一次要的正是那几条的读数，不算失败）。

## 八、自己提的改法（只在我的模型上量过、被攻过零轮；探针还没复跑，改法本身一行都没实现）

| 改法 | 修哪一格 | 形甲 | 形乙 | 形丙 | 状态 |
|---|---|---|---|---|---|
| 改甲：回锁出错那一支，删意图之前此刻有冻结着的发布（`allocator.frozen_publish()` 在），就不去删、照门压住那一支记下待删、交回停（不交回错） | 回锁出错那一支 | 中（不交回错，记账那一支不改记种类） | 中（重发成了之后下一次取输入照待删先删） | 不中 | 推的，没实现没跑 |
| 改乙：一批推完记账、停着那一趟开头两处改记冻结种类时，只在冻结的就是这一次调用发的那一次时改（例如比冻结那一次的 txg 与这一次调用发布之前的现行那一版，或只在被拒原因是这一次写盘报错时改） | 记账改种类那两处（`crates/singlefs-core/src/mounted_session/compaction_run.rs:385-389`、`:662-666`） | 中 | 不中 | 不中 | 推的，没实现没跑 |
| 改丙：回锁出错那一支删意图交回任何错（冻结、空间）都记下待删 | 记下那一处 | 不中（种类照改） | 中 | 不中 | 推的，没实现没跑 |
| 改丁：`resend_the_frozen_publish` 整理那一支照后台批记：清两个计数（与写带宽的字节，要拿到这一次的上限才记得了，推不出怎么拿） | 重发那一处 | 不中 | 不中 | 中 | 推的，没实现没跑 |

改甲与改丁合起来罩三格（推的）。改甲与条款第 5 条「回锁时有冻结的发布就丢掉这一批」的关系：出错那一支「先于一切」，改甲只是让删意图那一步在冻结着时照门压住那一形留着，不改丢批那一判；要不要这样读，交主 agent。

## 没打中的形状

- 绕开取输入的路：两处调用点、锁里一口气推完、显式入口、后台入口整趟与至多一次发布两种、可写挂载续做、停着那一趟开头、删除续做先走那一段，逐条读码，没找到绕开的（第四节第 1 条）。
- 记下的那一条被换成别的意图：锁外期间别的整理入口都拒，现行那一版里的活意图只有冻结重发一条路会变，变了走 `:51` 忘，没造出误删别的意图的历史。
- 待删那一次的预算、根槽、写带宽、清零、停与不停：与停着那一次删意图逐项比，只差让路与写带宽两判（第四节第 1 条，判推不出）。
- 回锁破例的逐字与谓词：对得上；回锁不判写带宽：对得上。
- 冻结拒掉之外的回锁失败：排空待发被拒在出错那一支之后（出错先于一切，走不到）；锁外那一批算完被调用方丢掉（卸载）不照出错办，条款第 4 条「卸载时不等」，没打中。
- 新用例过度钉实现：要搬数那条用的是上界，这一份几何里上界等于重扫出的数，没钉死；空洞数那条前提用产品函数，断言不依赖它。
- 取样范围：格一 5 个前缀（重复键 / 掉卡 / 不出错 × 门不判；重复键 / 不出错 × 门关而破例成立）× 下一次入口 2 种 × 重发之后建 0、1 个 inode，共 20 段；格二 2 段。几何只有 `common::build_pool` 那一份默认几何（S = 8），没扫别的几何。

## 这条腿自己的限度

- 不能编、不能跑：探针一行都没编过（只用 `rustfmt` 过了语法），API 名照仓里用例抄、读码核过签名，编不过要承接的腿改；三形的读数全是推的。
- 探针放进副本验证包的 `tests/` 下跑（与第二轮攻方探针同一个放法），不调崩溃枚举函数，崩溃状态 0 个；第 3b 步那几条（内存稀疏盘、状态数上限）对它不适用。
- 形甲、形乙要锁外期间一次写盘报错一直撑到回锁、同时 GPU 那一批出错：产品上走得到（运行库锁外期间每拍重发冻结那一次，重发一直失败时冻结撑到回锁），出现频率没量；空间拒掉那一形没造历史。
- 「钉住的单元在锁外期间落进没排他的 R」在产品路径上走不走得到，没查清，候选变异那一条的价值跟着推不出。
- 第二节第 1 条「待删那一次问不问让路」两读都说得通，交主 agent 判，不算没打中。

## 没做什么

- 不判正推腿那几格，不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务、没起 cargo、没跑门禁；判决要引的数没复跑（命令在第七节）。
- 没改 `crates/`、kb 与主工作区任何文件；冻结副本只追加了三行候选变异。

## 清理

- 草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/` 里留两样：`copy/`（仓副本 64M，`.deliverables` 写成「交接」：主 agent 在里面跑候选证红、探针以它为冻结副本）、`model/`（探针与复跑脚本，`.deliverables` 写成「报告」）。没有编译目录。主 agent 用完删 `copy/`。
