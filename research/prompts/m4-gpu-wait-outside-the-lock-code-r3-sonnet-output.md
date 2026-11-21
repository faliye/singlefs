# m4-gpu-wait-outside-the-lock-code-r3：云端正推腿（Sonnet）报告（2026-11-20）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r3-body.md`。这一腿只读码、不起 cargo、不跑门禁，没写模型（没有 `rerun.sh`）；草稿目录里只有两份只读核对脚本与一份候选变异表，sha256 见「要主 agent 跑的命令」一节。读到的树是主工作区今天的树：合成补丁里 4 份核心文件的「计划改后 sha256」与主工作区此刻逐份相同（`sha256sum` 现算：compaction_batch.rs 为 `b5afe3de…8925`、compaction_run.rs 为 `e8778c4d…4bba`、compaction_plan.rs 为 `76a562df…4dc`、allocator.rs 为 `39e47fe7…eae`，与补丁 `apply-outcome.tsv` 的回读行一致）。凡写「推的，没量过」的都是读码推出来的、没有读数。

## 一、第二轮判决「三、改法」第 1、2、3、5、6 条（第 4 条一并）逐条核

先把今天的条款原文钉死（`.claude/kb/decisions/26-后台整理与放置回收.md`，行号现查）：

- 第 256 行（已定项 13 第 5 条）补的那一句：「之后不论走哪条路，取下一批的输入之前都先照这道门与破例判它，门开或破例成立就只发这一次删意图，门关就这一趟不发、也不在这条意图下接着搬」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`），后接「只记在这次挂载里，卸载重挂之后照盘上那条活意图续做」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）；前一句「停着时那一次只删意图的发布（第 4 条）同样过这道门：门关就不发，意图与 R 排他留到门开；破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发」同在这一行（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）。「待删」那一句的另一半在同一行：「回锁里出错那一支要删的意图被门压住的，同样留到门开」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）。
- 第 257 行（第 6 条）清零那一句：「一批发成两个都清零；取了输入而没让路的那一批也把两个计数清零，N 与 T 是挂载参数」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`），「下限到了、」已不在（`grep -cF '下限到了、'` 对该 kb 文件交回 0）。

### 五条各一行

| 改法 | 判 | 依据（文件:行） |
|---|---|---|
| 第 1 条 被门压住的删意图：记下、判、删、忘 | **成立** | 记：回锁出错那一支门关时 `compaction.hold_back_the_drop_of_the_intent_of_an_abandoned_batch(`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:702`），只在现行那一版里有活意图时记（同一分支 697-701）；状态只住会话：`intent_of_an_abandoned_batch_still_to_drop: Option<IntentOfAnAbandonedBatchStillToDrop>,`（`crates/singlefs-core/src/compaction_plan.rs:615`），新挂载 `intent_of_an_abandoned_batch_still_to_drop: None,`（`crates/singlefs-core/src/compaction_plan.rs:729`）。判：取输入之前 `stop_of_the_abandoned_batch_whose_intent_is_still_to_drop(compaction, current)`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:35`），门与破例在每批之前那几判里（`crates/singlefs-core/src/mounted_session/compaction_run.rs:574`、507-512）。删：`take_the_input_of_one_compaction_batch` 开头发 `drop_the_live_intent_on_a_stop`（compaction_batch.rs:34-50）。忘：删成 `compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1479`），现行那一版里已不是它时取输入开头再清 `compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:51`）。 |
| 第 2 条 回锁那一刻判破例 | **成立** | `if foreground_cannot_write_under_the_switches(allocator) {`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:798`）放在 `match rollback_gate_at_the_relock` 之前，成立交回假（照发），与取输入时定的 `rollback_gate_at_the_relock` 无关；判法 `fn foreground_cannot_write_under_the_switches(allocator: &PoolAllocator) -> bool {`（`crates/singlefs-core/src/mounted_session.rs:521`），与停着时、取输入时是同一个函数（compaction_run.rs:85、512 同样调它）。 |
| 第 3 条 停着时开头删意图发成后清让路计数 | **成立** | `Ok(CompactionStep::Published { version, kind: _ }) => {`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:372`）里紧接 `compaction.clear_the_yields_in_a_row();`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:374`），与第 6 条「一批发成两个都清零」对得上（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）。 |
| 第 5 条 签发那一支补测试 | **成立**（有一格没单独罩到，见二.4） | 空洞数：`&current_version.allocation_records,`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1000`）由 `an_issue_at_the_relock_counts_the_holes_of_the_segments_holding_user_data_in_the_current_version` 罩（用例里「`if holes_on_the_records_of_the_ticket < records_in_the_region_now`」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:513`）与现行记录口径下够才跳出循环）；要搬数：`let units_to_move = if records_of_the_region_at_the_input == records_of_the_region_now {`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:985`）由 `…takes_the_unreleased_records_of_the_current_version_as_the_units_to_move` 罩（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:749`）；资格两项：「这次挂载开过的段」`!allocator.cluster_segments().contains(&region.start_slot())`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:966`）经只供测试的开关 `.note_a_cluster_segment_opened_in_this_mount_for_tests(region);`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:563`）罩，「钉住与隔离的槽」`.union(compaction.quarantined_slots())`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:962`）只有隔离那一半有用例。10 行变异的核对见三、第 7 格。 |
| 第 6 条 新加注释去判决号与节号 | **成立** | 两处点名的已去：`这一趟与观测数的批数（batches_published）只数搬了单元的那一次，与上面清零的口径不同：没搬单元的发布不记进批数`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:615`）、`意图里的量取现行那一版的：c0 是签发那次发布接着的那一版的 txg`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:929`）；补丁新加（`+`）行里没有「判决」「第三节」「此前」「原先」「后来」「改成了」「换成了」「修过」（对合成补丁 `crates.patch` 的 `+` 行 `grep`，零命中）。拆分之前就有的同类注释（compaction_batch.rs:1274、1283、1312、1315、1369 等，如「判决 m4-segment-code-r1 第三节 F2」）不在这一题改，第二轮判决第 8 条已划出。 |
| 第 4 条 kb 改回原话（一并） | **成立** | 第 257 行清零一句照用户 2026-11-19 的原话（见上，「下限到了、」零命中）；变更史有这一条：「已定项 13 第 6 条清零那一句去掉」（`.claude/kb/decisions-history.md:15487`）。 |

### kb 逐句对代码（第 5 条补的那一句、第 6 条清零那一句）

| kb 子句 | 代码今天做的 | 判 |
|---|---|---|
| 回锁里出错那一支要删的意图被门压住的，同样留到门开 | 门关时 `Ok(CompactionStep::Stopped(stop))` 不发、意图与 R 排他留着、会话记下（compaction_batch.rs:696-707） | 对上 |
| 之后不论走哪条路 | 唯二的两个调用点 `crates/singlefs-core/src/mounted_session/compaction_publish.rs:234` 与 `crates/singlefs-core/src/mounted_session/compaction_run.rs:728` 都走 `take_the_input_of_one_compaction_batch`；锁里推完（compaction_run.rs:218-224）、放锁入口（728）、可写挂载续做与删除续做都在这两处之下或不搬单元（见二.1） | 对上 |
| 取下一批的输入之前先照这道门与破例判它 | 门：`if run.is_the_rollback_gate_judged && !is_this_publish_the_exception {`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:574`）；破例：`let is_this_publish_the_exception = (run.budget_rule`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:507`）的第二支「有待删且此刻前台写不进」 | 对上；门只在后台入口、开关照水位判时判，与别的整理发布同口径（compaction_run.rs:283-288），条款没另说 |
| 门开或破例成立就只发这一次删意图 | 取输入开头删完即返回 `AdvancedInTheLock`（compaction_batch.rs:34-50），不开这一批、不搬单元 | 对上 |
| 门关就这一趟不发、也不在这条意图下接着搬 | 门关 `stop_of_the_rollback_gate` 在 `judge_before_the_next_compaction_batch` 里先停（compaction_run.rs:574-581），走不到取输入 | 对上（没有用例单独钉，见二.4） |
| 只记在这次挂载里，卸载重挂之后照盘上那条活意图续做 | 字段在 `CompactionOfThisMount`；续做用 `let mut compaction_state_of_this_mount = CompactionOfThisMount::of_a_new_mount();`（`crates/singlefs-core/src/mount/continuation_at_the_mount.rs:40`），新状态里是 `None` | 对上 |
| 第 6 条：取了输入而没让路的那一批也把两个计数清零 | `compaction.break_the_yields_in_a_row_by_a_batch_not_yielding();`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:559`）在问过调用方、没让路之后，**先于** 写带宽（560）与回退门（574）两判 | 对上口径；字面有一格差：停在写带宽或回退门上的那一次没真取到输入也清。第二轮判决第 4 条与变更史记的用户动机（「整理一直发不成（例如回退门关着）时，让满一次之后……不再让路」（`.claude/kb/decisions-history.md:15496`））正是要这一格，用例 `…even_when_its_batch_publishes_nothing` 钉的就是它。我不当新打中，只标字面可读成不一致 |

## 二、要攻的地方五条

### 二.1 改法第 1 条：记下、判、删、忘四步，哪条路绕得过 —— 没打中

不经取输入就能接着在这条意图下搬的路，我逐条找了，没有。整理里「搬单元」只有一处入口：`take_the_input_of_one_compaction_batch`，它只被两处调：锁里一口气推完的 `let input = match take_the_input_of_one_compaction_batch(`（`crates/singlefs-core/src/mounted_session/compaction_publish.rs:234`）与放锁入口的 `match take_the_input_of_one_compaction_batch(`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:728`）。待删检查是它的第一件事（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:35`），先于 `begin_a_batch`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:52`）。

| 路 | 去向（文件:行） |
|---|---|
| 锁里一口气推完的入口 | 每一轮 `judge_before_the_next_compaction_batch` 再 `advance_compaction_by_one_publish`（compaction_run.rs:218-224），后者调 compaction_publish.rs:234 那处，先删 |
| 放锁的三段入口 | `take_the_next_compaction_batch_or_finish_the_run`（compaction_run.rs:693）里调 728 那处，先删；删完 `note_the_compaction_batch_advanced` 带着 `IntentDroppedOnAStop` 收尾，这一趟停（compaction_run.rs:636-653），迭代上界 2 的注释同步改了（compaction_run.rs:687-688） |
| 可写挂载续做 | 用的是 `CompactionOfThisMount::of_a_new_mount()` 新状态（`crates/singlefs-core/src/mount/continuation_at_the_mount.rs:40`），本来没有待删；它推的那一趟也走 218-224 那条循环，没有绕道 |
| 删除续做 | 只发删除步、不搬单元（compaction_run.rs:404-449）；它发的版照抄 logged_ops，活意图与会话记着的都不动，之后第一次取输入照样先删 |
| 停着时一趟开头那一支 | `if compaction_stays_stopped(allocator, background_compute) {`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:347`）：门关 `return Err(run.finish(` 带门的停因（353-362），会话记着的不动；删成走 `drop_the_live_intent_on_a_stop`，里面一并忘（compaction_batch.rs:1479）；预算 0 时不删（364-366），记着的留到有预算那一趟 |

忘得对不对（各情形，推自 `crates/singlefs-core/src/mounted_session/compaction_batch.rs:1494-1499` 与 1466-1485，没量过）：

- 删成：`compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1479`）排在发成之后，对。
- 冻结：删意图那次发布写失败，`?` 在 1476 就返回，忘的那行走不到，记着的照旧；之后前台或维护线程重发成了（`crates/singlefs-core/src/mounted_session.rs:1099-1124`，现行那一版换成删意图那一版），下一次取输入 `(live_intent_now == Some(held_back.intent)).then_some(held_back.stop_of_the_abandoned_batch)`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1499`）交回 `None`，51 行再清。对，但「记着」在这段时间里是过期的（`has_an_intent_of_an_abandoned_batch_still_to_drop` 此刻读真，`crates/singlefs-core/src/compaction_plan.rs:855`）。
- 门关：不动，对。破例：门不判、直接删，对（测过，见三.7 第 9 行）。
- 现行那一版里的活意图已换成别的：等式不成立，不删它、51 行清掉，对。这一格没有用例到（见二.4 乙）。

删它那一次用放弃那一批时的停因，这一趟停与不停、预算、让路计数（读码）：`stop_of_the_abandoned_batch` 存在 `crates/singlefs-core/src/compaction_plan.rs:622` 的结构里，删意图发成走 `note_the_compaction_batch_advanced`：`publishes_of_the_run` 与 `publishes_since_the_last_user_change` 各加一、记根槽与写带宽账、`compaction.clear_the_yields_in_a_row();`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:613`），与停着时那一次（compaction_run.rs:374-380）同口径。这一趟结束（`stop_holds_back_the_trip_without_ending_it` 对该停因交回假，compaction_run.rs:782）；运行库那一拍判是否接着调时，这个停因落在返回真的那一组里（`crates/singlefs-mount-runtime/src/lib.rs:1149`），下一次调用重判起跑、从头签发，与条款「下一次整理从头签发」的措辞一致（`crates/singlefs-core/src/compaction_plan.rs:245`）。

两处不当打中、记下的观察：

1. 停因里的 `task_after_the_error` 是放弃那一批当时的值，补发那一趟读它时任务可能已被切换解除（读码，全仓没有读它去分支的地方，只进 `last_stop` 观测：`compaction.observations.last_stop = Some(*stop);`（`crates/singlefs-core/src/mounted_session.rs:1181`））。推的，没量过。
2. 回锁出错那一支的删意图没发成而又不是被门压住（取不到落点被拒，不冻结）：不记（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1466` 一带的文档写「下一次整理再走到停下时再删」），整理没停着的任务（切换已解除、只放弃这一批）下一批会在这条意图下接着取输入。条款只管「被门压住的」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`），这一支条款没说，我标「规则没说」；它被空间拒的那一支下一批同样会被空间拒、走 `pause_by_dropping_the_live_intent_or_hand_up` 删（compaction_batch.rs:1413-1434），自己收口。推的，没量过。

推翻「没打中」的现象：任一入口在会话记着待删、现行那一版里仍是那条活意图、门开（或破例成立）而没先发那次删意图就交出一批输入（即 `BatchOutOfTheLock` 或 `Input`）。

### 二.2 改法第 2 条：回锁那一刻的破例与条款逐字、次序 —— 没打中

逐字：条款括号列两个条件（全空聚簇段数低于底额、前台已写不进），代码里 `foreground_cannot_write_under_the_switches` 的定义是开关照水位判且任一块盘全空段低于底额（`crates/singlefs-core/src/mounted_session.rs:521`、`crates/singlefs-core/src/compaction_plan.rs:1459`），文档把「前台写入类发布已写不进」就写成这同一个量（`全空段低于底额、前台写入类发布已写不进（合取 7，N₀ 是开关给的那一档）：任一块盘低于就算。`，`crates/singlefs-core/src/compaction_plan.rs:1457`），同一个函数还被起跑（compaction_run.rs:85）、取输入（512）、入口（`crates/singlefs-core/src/mounted_session/compaction_entry_points.rs:535`、766）用。两个条件在这个仓里是同一个判，与条款括号一致。

次序（读码）：

| 站 | 回锁出错那一支 | 取输入那一次（门关且有待删） |
|---|---|---|
| 1 | 出错先于一切，在冻结判之前（compaction_batch.rs:678-724 先于 757） | 预算、至多一批、至多一次发布三个停（compaction_run.rs:478-503） |
| 2 | 破例：`if foreground_cannot_write_under_the_switches(allocator) {`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:798`），成立就交回假、照发 | 破例标志（507-512，第二支：有待删且此刻前台写不进） |
| 3 | 回退门：只在破例不成立、这一批判门、现行那一版与票里不同时判（801-806） | 问调用方；破例时跳过让路（541）；没让路就清两个计数（559）；写带宽（560）；门（574，破例时跳过） |
| 4 | 删意图那次发布：冻结时被 `refuse_while_a_publish_is_frozen` 当场拒（`crates/singlefs-core/src/transaction/persisting_a_publish.rs:130`），走 `Failed` 一路、不记待删 | 取输入开头删意图（35-50） |
| 5 | 写带宽：回锁不判，发成后记账（compaction_run.rs:608-611） | 删成后记账同上 |

所以「破例照发」之后，写带宽那一判在取输入那一次仍在（560 先于门 574，破例只绕让路与门），回锁那一次不判；条款第 5 条只说破例排在让路之前（`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「破例排在第 6 条的让路之前，前台有入口在等也照发（前台已写不进，让路换不来什么）」），没说写带宽；每一批破例同样如此，不是这次补丁引进的。「规则没说」。另一处不对称：停着时那一次的破例取起跑那一刻定的预算档（`let is_this_drop_the_exception =`，`crates/singlefs-core/src/mounted_session/compaction_run.rs:351`），取输入与回锁取此刻的；条款只写「成立时」，没写取哪一刻，「规则没说」。

推翻「没打中」的现象：回锁那一刻 `foreground_cannot_write_under_the_switches` 为真而出错那一支仍被门压住（即删意图没发、会话记下了它）；第二轮探针格二的读数 `window_published=0`（第二轮判决复跑过）在补丁后应变成 1，这一点我推的、没量过，用例 `a_compute_error_at_the_relock_after_the_floor_was_crossed_in_the_window_drops_the_intent_behind_the_closed_gate` 断言的就是它（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs:568`）。

### 二.3 改法第 3 条与第 6 条清零口径合起来：还有哪一处整理发布发成之后不清 —— 打中一处（读法有岔，推的，没量过）

整理发布发成的每一处与清零的对照（`grep -n 'clear_the_yields_in_a_row\|break_the_yields_in_a_row_by_a_batch_not_yielding'` 全 `crates/` 非测试代码只命中 `crates/singlefs-core/src/compaction_plan.rs` 的 773、781，compaction_run.rs:374、559、613）：

| 发成的整理发布 | 清不清 | 文件:行 |
|---|---|---|
| 取输入一路里的签发、搬一批、删意图（含回锁发的）| 清 | `compaction.clear_the_yields_in_a_row();`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:613`） |
| 停着时一趟开头单发的删意图 | 清（这一轮补的） | compaction_run.rs:374 |
| 删除续做的步 | 不清 | 条款：整理开头先走的那一遍删除不是整理批（`.claude/kb/decisions/26-后台整理与放置回收.md:257`），按条款对 |
| 冻结之后重发成了的整理发布 | **不清** | `KindOfTheFrozenPublish::CompactionPublish => {`（`crates/singlefs-core/src/mounted_session.rs:1135`）只 `self.compaction.publishes_since_the_last_user_change += 1;`（1136）与根槽数，没有清零 |

打中的是最后一行。条款相关两句都在第 257 行：「一批发成两个都清零」，以及「重发成了照冻结那一次的种类记账：冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）；冻结的是后台批，照后台批记」（都是 `.claude/kb/decisions/26-后台整理与放置回收.md:257`）。

历史（构造得出，没量过；不是这一轮补丁引进的，是补丁补了停着时那一支之后剩下的同类一处）：
1. 前台在等，后台入口问调用方后让路一次，`yields_in_a_row` 为 1（让路记数在 `judge_a_yield_under_the_floor`，调用处 `crates/singlefs-core/src/mounted_session/compaction_run.rs:542`）。
2. 一批出错、任务停下，现行那一版里有活意图（compaction_batch.rs:681-684）。
3. 下一次后台入口判出停着而有活意图，起跑那一趟开头单发删意图（compaction_run.rs:347、364-370）；这一支不问调用方，所以 559 行那一清零走不到，计数仍是 1。
4. 删意图那次发布写盘失败：`Err(refused) => {`（`crates/singlefs-core/src/mounted_session/compaction_run.rs:384`）里 `note_the_kind_of_a_publish_frozen_by_this_call(` 把冻结的种类记成整理发布（385-389）。
5. 冻着时维护线程不跑整理（`冻着时 publish_when_due 与整理都不跑`，`crates/singlefs-mount-runtime/src/lib.rs:808`），计数不动；前台 fsync 或下一拍重发成了，走 mounted_session.rs:1135 那一支：预算与根槽数记了，让路的两个计数仍是 1。之后前台再在等时按「已连着让了 1 次」往下数，下限比条款早到。
没有用例同时碰这两样（`grep -rl 'resend_the_frozen_publish' crates/*/tests` 与 `grep -rl 'yields_in_a_row' crates/*/tests` 两份清单不相交）。

读法有岔：把「照后台批记」读成含「一批发成两个都清零」就打中；读成只指预算与根槽（今天代码就是这样记的）就不中。条款没写，交主 agent 判。若采：改的只是 mounted_session.rs:1135 那一支多一行 `self.compaction.clear_the_yields_in_a_row();`，再补一条「停着时开头删意图冻结、重发成了、计数清零」的用例与变异行；写进产品代码之前照三方规则走。推翻它的现象：重发成了之后 `yields_in_a_row()` 读 0（读码推出为 1）。

### 二.4 改法第 5 条：新用例测的是条款说的量，还是实现碰巧的计数；改坏对应那一判会不会照样绿 —— 打中测试缺口三处、断言钉实现取值两处（都是推的，没量过；产品代码本身没打中）

先说条款说的量与用例的对应（成立的部分）：

- 空洞数取现行那一版：用例循环到「`if holes_on_the_records_of_the_ticket < records_in_the_region_now`」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:513`）为止，即票里记录口径与现行记录口径给出分得开的两个值，再看回锁签没签；换成票里记录就红（变异行第 6 行）。量取的是条款的量（已定项 1 第 2 条「R 外非全空段的空洞 ≥ R 里要搬的单元数」），不是实现的某个计数。算左边的 oracle 用的是实现同一个函数（issues_at_the_relock.rs:453），测的是「取哪一版」，不是这个函数本身，对。
- 要搬数：用例把锁外扫出的能搬数压成 1（`fn computed_with_only_the_first_unit_of_the_region_readable(`，`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:609`），断言 `units_to_move, records_in_the_region_now,`（issues_at_the_relock.rs:749）。实现取的是「现行那一版 R 里未释放的记录数」这个上界（`let units_to_move = if records_of_the_region_at_the_input == records_of_the_region_now {`，compaction_batch.rs:985），条款只说「各个量取现行那一版的」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）。用例场景里现行那一版真能搬的数就等于未释放记录数（锁外只是人为让读不出），所以断言不依赖「上界」这个实现选择：实现改成在现行那一版上重扫能搬数，这条用例仍绿。
- 资格「这次挂载开过的段」：只供测试的开关造出这一格，产品路径上的走法在开关注释里写了（`crates/singlefs-core/src/allocator.rs:255-259`）：选中那一段的记录先全放掉、再被提交内生块开成新段，此时新段的记录是 R 里未释放的记录，`!records_of_the_region_now.is_empty()` 仍为真，所以这一格在产品路径上走得到，开关不是造出一个走不到的格。

缺口与碰巧的（打中）：

甲、**「门关就这一趟不发」那一半没有用例**。四条新用例里，切换解除、重复键两条在门重开之后才取下一次输入（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs:427-442` 的循环建 inode 到门开才调取输入），「排空越线」那条先把底额抬到越线再取输入（694-735），「回锁越线」那条根本没有下一次取输入，没有一条在「会话记着待删、门仍关、底额没越线」时再取一次输入，断言「停在回退门上、一个写都不发、会话仍记着」。改坏：把破例那一支的 `&& foreground_cannot_write_under_the_switches(allocator)` 去掉（`crates/singlefs-core/src/mounted_session/compaction_run.rs:512`），记着待删就当破例、绕门。四条用例都仍绿（推：门重开之后或底额越线之后绕不绕门结果相同），第 9 行变异只把整支改成 `false`，这个改法不在里面。候选变异 C2 见「要主 agent 跑的命令」。
乙、**「记着的不再是现行那一版里的活意图」那一判没有用例**。`(live_intent_now == Some(held_back.intent)).then_some(held_back.stop_of_the_abandoned_batch)`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1499`）改成不核等式，全部待删用例仍绿（只有这三份文件引用待删状态：`grep -rln 'still_to_drop\|hold_back_the_drop' crates --include=*.rs` 在测试里只命中 compute_errors…rs 一份）。改坏之后的后果不止是多删：记着的已过时、现行那一版又没有活意图时，`drop_the_live_intent_on_a_stop` 走「没有活意图」的早退（`return Ok(CompactionStep::Stopped(stop));`，`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1466`），早退不忘，之后每次取输入都在 35 行返回，永远取不到下一批（活性坏；推的）。过时的记着在产品路径上走得到：二.1 的冻结→重发成了那一支。候选变异 C1。
丙、**「钉住」那一半没有用例**。变异行「不看钉住与隔离的槽」把整个 `range` 改空（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:967` 一带），被隔离用例抓；而 `.pinned_slots` 那半（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:961`）单独去掉，隔离用例仍绿，整个验证包里该目标目录没有一条钉住的槽落在选中那一段里的用例（`grep -rni 'pinned' crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/` 零命中）。候选变异 C3。
丁、**「忘」那一步没有变异行**：1479 行那一忘（`compaction.forget_the_intent_of_an_abandoned_batch_still_to_drop();`）现在只靠 `has_an_intent_of_an_abandoned_batch_still_to_drop` 的断言（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs:486`、612、767）盯，门禁的变异那一道不判它。我推去掉它那条用例会红（候选 C4，预期红，是用来核「这一步确有人盯」的）。
戊、**断言钉了条款没写的停因**。条款没规定补发那一趟停在哪一条，用例却断言 `stop_of(&after_the_gate_reopened)` 等于放弃那一批时的停因（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs:469-473`、757-761）。把补发那一趟的停因换成别的、条款照样满足，这两条用例会红：碰巧的是实现取值，不是条款的量。

另一处小的：新用例里的 `computed_with_only_the_first_unit_of_the_region_readable` 名字带位置代称 first（`.claude/agent-common.md` 写的「不写 `first_stream`」一类），见三.2。

推翻我「甲、乙、丙没被罩」的现象：候选变异 C1、C2、C3 点名的现有用例在副本上被判「红」；推翻丁的反面：C4 判「没红」。

### 二.5 kb 两处改动与代码今天做的逐句对得上吗 —— 成立（一格字面有岔，已在一节标出）

逐句表在一节末尾：第 5 条补的那一句对上（唯一要说明的是门只在后台入口、开关照水位判时判，与别的整理发布同口径，条款没另说）；第 6 条清零那一句对上口径，字面「取了输入」与代码「问过调用方没让路」之间有一格差，第二轮判决第 4 条已按用户原话认下，不当新打中。

## 三、检查单九格

### 三.1 走不到的理由二次查实 —— 成立

三个实现员写的「没新加 `todo!`、`unreachable!`、`assert!` 或条款没定的错误成员」在今天的树里成立。`sed -n 1,327p` 取合成补丁 `crates.patch` 的非测试部分（8 份文件里 5 份产品文件），对 `+` 行 `grep -n 'assert\|expect(\|unreachable\|todo!\|panic!\|unwrap\|_ =>\|as u\|as i\|#\[allow'` 零输出。补丁碰的几处里原有的断言与 `expect` 没动：取输入里 `assert_eq!`（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:83`）要求「分配器这一侧的活意图就是现行那一版 logged_ops 里那一条」，待删这一支都在它之前返回（35-50），之后的路上这条不变量没被新状态破坏：删成撤分配器那一侧（`allocator.compaction_mut().withdraw_the_live_intent();`，`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1477`），冻结重发成了按现行那一版重新对齐（mounted_session.rs:1117-1124）。新加的 `?` 只有 Option 上那一个（`let held_back = compaction.intent_of_an_abandoned_batch_still_to_drop()?;`，`crates/singlefs-core/src/mounted_session/compaction_batch.rs:1494`），`None` 是正常分支。唯一一个「条款没定的走法」是过时的记着（二.4 乙），代码是静默清掉，不是断言。推翻：新加行里出现上面任一词。

### 三.2 类型与名字 —— 成立，一处名字有位置代称

- 类型：`pub(crate) struct IntentOfAnAbandonedBatchStillToDrop {`（`crates/singlefs-core/src/compaction_plan.rs:622`）两个字段是 `CompactionIntent` 与 `CompactionRunStop`，没有裸整数；放弃那一批时的停因没有另起一个整数或布尔去表示。产品新加行里的分支形态是 `if let Some(...)`（compaction_batch.rs:34、697）与 `.then_some`（1499），补丁新加行里没有 `_ =>`、没有 `other =>`；新用例里的 `let ... else`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:739-747`）有 panic 说明依据。
- `expect`：产品新加行里没有；用例里的 `expect("…")` 写的是依赖的前提（如「没有一批在锁外、盘表是挂载的那一份」，compute_errors…rs:450）。
- 名字：五个新名字都说内容，没有缩写与序数。一处例外：`computed_with_only_the_first_unit_of_the_region_readable`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:609`）带 `first`，是「读出来的那一个单元」的位置代称，`.claude/agent-common.md:38` 的口径（不写 `first_stream` 一类）更严；主 agent 观测命名检查已过，所以我不判红，只标「不成立（规则文字）/成立（命名检查）」。`position_of_the_first_occurrence` 等是核心层已有的成员名，非新增。
- `has_an_intent_of_an_abandoned_batch_still_to_drop`（`crates/singlefs-core/src/compaction_plan.rs:855`）是 `pub fn`，同块别的新方法是 `pub(crate)`；它只被用例用，与同文件 `has_a_batch_out_of_the_lock` 同写法，不判。

### 三.3 不兼容旧代码与旧格式 —— 成立

补丁新加行对 `legacy|compat|兼容|旧版|旧库|迁移|version_number|FORMAT_VERSION|rocksdb|prune` 零命中。待删只在内存：字段在会话状态里，新挂载是 `None`（`crates/singlefs-core/src/compaction_plan.rs:729`），挂载续做用全新状态（`crates/singlefs-core/src/mount/continuation_at_the_mount.rs:40`），不进盘上格式、不加版本号。没有为旧路径留的分支。推翻：补丁里出现读旧盘或补旧行的分支。

### 三.4 不新建判定库 —— 成立

补丁的文件清单（`grep '^diff --git' crates.patch`）：`allocator.rs`、`compaction_plan.rs`、`mounted_session/` 下三份、验证包 `tests/compaction_batch_computed_out_of_the_session_lock/` 下三份用例，外加 `crates/mutations.tsv` 的变异行；没有崩溃枚举用例，没有判定存放处。`+` 行对 `crash|崩溃|rocksdb|prune` 零命中。

### 三.5 所有任务多机可用 —— 成立

补丁新加行对 `hostname|gethostname|/home/|HOME|127\.0|192\.168|std::env|nproc|available_parallelism` 零命中。新用例的盘镜像走仓里已有的装置：`image_path` 用 `std::env::temp_dir()` 加进程号与自增序号命名（`crates/singlefs-verification/tests/common/mod.rs:55-61`），GPU 实现是照 CPU 写法算的替身（`let shared = Arc::new(Mutex::new(`，`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs:218` 一带），不开卡、不读多机配置。装置本身不是这一批新写的，我没再深查 `TMPDIR` 在两台上的取值：推的。

### 三.6 测试优先多线程 —— 成立，没找到该并行而没并行的

九条新用例各自 `build_pool("…")` 一份池（标签各不相同，`image_path` 另有进程内自增序号），一个场景一条 `#[test]`，住同一个测试目标 `compaction_batch_computed_out_of_the_session_lock`，由 libtest 并行跑。没有 `serial`、静态锁；唯一的环境锁 `one_test_changes_the_environment_at_a_time` 只在 GPU 判器取卡那几个函数里（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs` 的 413、455、499），新用例不调。两条共用一个起法的（切换解除、重复键）是两条 `#[test]` 各调同一个辅助函数，不是在一条里串着跑。用例里的几十圈建 inode 是状态相依的，拆不开。

### 三.7 带 --item 的命令与项 —— 点名与点全成立；命令带不带范围声明推不出

我对「合成补丁追加 9 行、替换 1 行」逐行读了：原文在今天的树里各恰好命中 1 次；第 5 段点名的测试都是带模块前缀的全名加 `--exact`，第 6 段是短名；点名的 `fn` 在对应模块文件里都找得到（`python3 -I …/tools/check_rows.py` 现跑，十行都是 `anchor_hits=1 … file_has_fn=True last_col=True`）。

| 行 | 改坏的地方 | 点名的测试（模块::函数，短写） |
|---|---|---|
| 1 | 回锁出错门关时不记待删（compaction_batch.rs:702） | compute_errors…::after_the_same_entry_key_twice_in_one_batch_…_once_the_gate_reopens |
| 2 | 取输入开头不删待删（35） | compute_errors…::after_a_compute_error_lifted_by_a_switch_requested_before_it_…_once_the_gate_reopens |
| 3 | 回锁出错那一支不判破例（798） | compute_errors…::a_compute_error_at_the_relock_after_the_floor_was_crossed_in_the_window_drops_the_intent_behind_the_closed_gate |
| 4 | 停着时开头删意图发成不清计数（compaction_run.rs:374） | yields_to_a_foreground_entry_waiting_for_the_session_lock::a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row |
| 5 | 重判资格不看钉住与隔离的槽（967 一带） | issues_at_the_relock::an_issue_at_the_relock_on_a_segment_with_a_slot_quarantined_in_the_window_is_given_up |
| 6 | 空洞数取票里记录（1000） | issues_at_the_relock::an_issue_at_the_relock_counts_the_holes_of_the_segments_holding_user_data_in_the_current_version |
| 7 | 要搬数仍取锁外扫出的（985） | issues_at_the_relock::an_issue_at_the_relock_after_the_records_of_the_region_changed_…_as_the_units_to_move |
| 8 | 重判资格不看这次挂载开过的段（966） | issues_at_the_relock::an_issue_at_the_relock_on_a_segment_opened_by_this_mount_in_the_window_is_given_up |
| 9 | 取下一批前破例只看预算档（compaction_run.rs:512） | compute_errors…::the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor |
| 10（替换） | 一批发成不清计数（compaction_run.rs:613） | yields_to_…::a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row |

九条新用例与前九行一一对应；第十行点名的是第二轮已有的用例。合入后验证第 2 步的命令原文不在我读的材料里，范围声明（`SINGLEFS_RUN_SCOPE`）带没带我推不出。变异行覆盖面的缺口见二.4 甲到丁（候选变异 C1 至 C4 不在这十行里）。

### 三.8 新崩溃流多罩了什么 —— 成立

补丁没有新加崩溃点重放流或崩溃枚举用例：文件清单见三.4，`+` 行对 `crash|崩溃` 零命中；变异行点名的目标 `compaction_batch_computed_out_of_the_session_lock` 不在崩溃枚举登记里（读它的 `main.rs` 里的 `mod` 声明与用例，没有读 `SINGLEFS_CRASH_*` 的代码），我没再去 `.claude/gate.d/stage-inputs.tsv` 核登记行：推的。

### 三.9 放的位置 —— 成立，一处位置与近邻不同

用例都在验证包 `tests/compaction_batch_computed_out_of_the_session_lock/` 下，子模块声明在 `mod issues_at_the_relock;` 一带（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs:87`，85-90 六行）。只供测试的开关 `pub fn note_a_cluster_segment_opened_in_this_mount_for_tests(`（`crates/singlefs-core/src/allocator.rs:258`）在被测的产品包里，写法照仓里已有的 `_for_tests`（无 `cfg`，`pub fn`，文档里写「只供测试」与对应的规则条款）。近邻不同：同类的 `pub fn try_allocate_at_the_lowest_unblocked_slot_for_tests(`（`crates/singlefs-core/src/allocator/pool_allocator.rs:288`）在 `PoolAllocator` 自己的文件里，新开关追加在模块根文件 `allocator.rs` 末尾的另一个 `impl PoolAllocator` 块里，靠字段 `pub(super)` 可见（pool_allocator.rs:51）；不违反哪条规则，标一笔。没有新开包，补丁新加行里没有 `#[path` 与 `include!`（零命中）；`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs` 的 14、16 行的 `#[path = "../common/mod.rs"]` 是本包内已有的，不是跨包。

## 要主 agent 跑的命令

没有写模型，没有 `rerun.sh`，没有探针，所以没有内存与时间读数。草稿目录里三份文件的 sha256（`sha256sum` 现算）：

- `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/candidate-mutations.tsv`：`668408192c0e84da3df32cdebeefb37333b77dbff54a6d9b7572c6ab3ff15d69`
- `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/tools/check_rows.py`：`4118fa7973a605f19f66bc81c03934e3e249534787da1d2a016e9759a5a85090`（只读，核十行变异行的原文命中数与点名的 `fn`，我已现跑，结果在三.7）
- `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/tools/make_candidates.py`：`d6ea66433943dc8b60eed473227c9ce38216f38503a35761c0b4d1891720eb1c`（只读主工作区，核四条候选的原文各命中 1 次后写出候选表，我已现跑，四行都是 `anchor_hits=1`）

候选表四行（六段，`\n` 表换行；预期结局写在每行后）：

- C1 `正推腿候选：会话记着的待删意图不核现行那一版里的活意图还是不是它`：`compaction_batch.rs` 的 `(live_intent_now == Some(held_back.intent)).then_some(held_back.stop_of_the_abandoned_batch)`（1499）改成不核等式；点名用例借用切换解除那一条。**预期：没红**（二.4 乙）；红了就推翻我「过时记着没有用例」。
- C2 `正推腿候选：取下一批之前删被门压住的意图那一次破例不看全空段是否低于底额`：`compaction_run.rs` 的 `.is_some()\n && foreground_cannot_write_under_the_switches(allocator));`（511-512）去掉第二支；点名用例借用「排空越线」那一条。**预期：没红**（二.4 甲）。
- C3 `正推腿候选：回锁签发重判资格不看钉住的槽`：`compaction_batch.rs` 的 `.pinned_slots` 那四行（960-964）换成只取隔离集；点名用例是隔离那一条。**预期：没红**（二.4 丙）；若编不过（字段未读告警成错误）判无效，不算没红。
- C4 `正推腿候选：删意图发成之后会话不忘掉被压住的那一条`：`compaction_batch.rs` 去掉 1479 那一行；点名用例借用重复键那一条（它断言 `has_an_intent_of_an_abandoned_batch_still_to_drop` 为假，compute_errors…rs:486）。**预期：红**（二.4 丁，反过来核「这一步有人盯」）。

1. 建副本（直接跑，不经服务；以主工作区此刻为基线）
   - 命令：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/copy --with-test-inputs`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：脚本不拒绝（退 0）；建前先 `du -sh` 列草稿目录已有的副本。我没建，副本在 `.deliverables` 里没有登记，建好用完请删。
2. 把候选表追加进副本的变异表（直接跑，不经服务）
   - 命令：`cat /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/candidate-mutations.tsv >> /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/copy/crates/mutations.tsv`
   - 在哪起：仓根；先确认副本里该表末尾以换行结束。
   - 看哪几行：`tail -n 4` 读到四行候选名。
3. 证红（交服务；待主 agent 跑）
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-sonnet/copy singlefs-verification '正推腿候选：会话记着的待删意图不核现行那一版里的活意图还是不是它' '正推腿候选：取下一批之前删被门压住的意图那一次破例不看全空段是否低于底额' '正推腿候选：回锁签发重判资格不看钉住的槽' '正推腿候选：删意图发成之后会话不忘掉被压住的那一条'`
   - 在哪起：仓根（`--cwd /home/fy5090/code/singlefs`，与第二轮判决里同类命令一致）；范围声明若被 `run-scope-guard` 拒，照拒绝说明补，不改命令。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；日志里每条候选的判定行：前三条为「没红」、第四条为「抓到」即与我的推断一致；前三条任一为「抓到」就推翻对应一条缺口；第四条「没红」就推翻丁。点名的测试用的是带模块前缀的全名加 `--exact`，基线应各判出「点名的测试通过」。
4. 二.3 冻结重发不清让路计数：我没写用例，所以没有命令。要量需先让实现员写「停着时开头删意图写盘失败冻结 → 重发成了 → 读 `yields_in_a_row()`」一条用例，写进产品代码的改法照三方规则走。**待量**，现为「推的，没量过」。

我自己现跑过的只读命令（输出已转进正文，都是 `grep -nF` 对原文与 `python3 -I` 读两份目录，没起 cargo、没交任务）：

- 引文核：每处引文写进报告前在被引文件里 `grep -nF` 一次，命中 1 次取行号。
- `python3 -I …/tools/check_rows.py`：十行变异行 `anchor_hits=1`，点名的 `fn` 都在，第 6 段与 `fn` 名相同。
- `python3 -I …/tools/make_candidates.py`：四条候选原文各命中 1 次。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 改法第 1 条 待删记下、判、删、忘 | 成立 | 唯二的取输入调用点都先删，状态只住会话；忘有两处，过时记着靠第 51 行清 |
| 改法第 2 条 回锁判破例 | 成立 | compaction_batch.rs:798 与取输入、停着时是同一个判法函数，次序在门之前 |
| 改法第 3 条 停着时清让路计数 | 成立 | compaction_run.rs:374 在 `Published` 一支里清 |
| 改法第 5 条 签发那一支补测试 | 成立 | 空洞数、要搬数、开过的段各有用例；钉住那一半没单独罩（二.4 丙） |
| 改法第 6 条 注释去判决号 | 成立 | 两处点名的已去，补丁新加行里无来历词 |
| 改法第 4 条 kb 改回原话 | 成立 | kb 第 257 行无「下限到了、」，变更史有记 |
| kb 第 5 条补的那一句 | 对上 | 逐子句对照在一节末；门只在后台入口、开关照水位判时判，与别的整理发布同口径 |
| kb 第 6 条清零那一句 | 对上口径，字面一格差 | 停在写带宽或门上没真取到输入也清；用户动机就是这一格，第二轮已认 |
| 二.1 待删哪条路绕得过 | 没打中 | 搬单元只有取输入一处入口；记下一支不记（没发成非冻结）与停因里的 task_after_the_error 过时两处标「规则没说」 |
| 二.2 回锁破例与条款、次序 | 没打中 | 两个条件在仓里是同一个判；写带宽、取预算档时点两处「规则没说」 |
| 二.3 还有哪处整理发布发成不清 | 打中一处（读法有岔，推的） | 冻结后重发成了的整理发布只记预算与根槽，不清让路计数：mounted_session.rs:1135 |
| 二.4 新用例测什么、改坏会不会绿 | 打中测试缺口三处、钉实现取值一处 | 门关不发那一半、过时记着、钉住那一半没用例；补发停因钉了条款没写的值 |
| 二.5 kb 两处与代码 | 成立 | 同一节末逐句表 |
| 三.1 走不到的理由 | 成立 | 补丁新加行零个断言、expect、unreachable、todo |
| 三.2 类型与名字 | 成立（一名带位置代称） | `computed_with_only_the_first_unit_of_the_region_readable` 带 first |
| 三.3 不兼容旧 | 成立 | 待删只住内存，无版本号与迁移 |
| 三.4 不新建判定库 | 成立 | 补丁文件清单无崩溃用例与判定存放 |
| 三.5 多机可用 | 成立 | 新加行无主机名与路径；装置用 `temp_dir` 加进程号 |
| 三.6 测试多线程 | 成立 | 九条各一池一场景，无串行锁 |
| 三.7 带 --item 的命令与项 | 点名与点全成立；范围声明推不出 | 十行原文各命中 1 次，点名带模块前缀；验证命令原文不在材料里 |
| 三.8 新崩溃流 | 成立 | 补丁无崩溃流；登记行没去核，推的 |
| 三.9 放的位置 | 成立（开关位置与近邻不同） | 用例在验证包目标下；开关在 allocator.rs 末尾，近邻在 pool_allocator.rs |

## 没做什么

- 不判别的腿的格（攻方腿的第二节五条各自怎么打），不替主 agent 采纳或出判决；没往统一任务服务交任务，没起 cargo、没跑门禁；判决要引的数（候选变异的红与没红、二.3 的读数）没复跑，命令在上一节。
- 没写模型，也就没有 `rerun.sh`、探针内存与时间；没建仓副本。读的只是主工作区今天的树与合成补丁目录；对 `crates/singlefs-mount-runtime/tests/` 下的用例只核了有没有碰「重发」与「让路计数」，没逐条读。
- 第一、二轮已攻过而没打中的角度（锁外与维护线程发布交错、CPU 路径卸载等整批、锁里旧入口仍是 pub、取输入先排空后问让路、回锁有冻结就丢批、GPU 包建缓冲那一等读停标记、fsync 试锁失败到登记与放锁到减计数两个窗口、注释口径与不切片理由）没重复。
- 没读禁读清单里的文件（同一轮攻方腿的报告、模型与草稿目录）。
