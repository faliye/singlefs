# m4-gpu-wait-outside-the-lock-code-r3：攻方腿承接报告，改乙、改丙、改丁三样合起来的探针 A/B（2026-11-20）

承接 `research/prompts/m4-gpu-wait-outside-the-lock-code-r3-opus-output.md` 第八节改法表。只做主 agent 定的三样合起来（改乙、改丙、改丁，不做改甲）的 A/B，不另攻别的面。全部读数都是推的，待主 agent 跑（第六节）；改法只在我的探针上推过、被攻过零轮，一次都没编过、没跑过。

复跑命令（第六节第 1 条，整条交服务）：
`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model-with-and-without-the-fixes/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus-s2-run /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model-with-and-without-the-fixes`

注意 `--frozen`（第二个参数）给的是草稿目录根，不是 `copy`：复跑脚本从它底下取 `copy/`（没有改法那一臂）与 `copy-with-the-fixes/`（有改法那一臂）。

模型文件（草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model-with-and-without-the-fixes/`，原来那份 `model/` 一字没动）：

| 文件 | sha256 |
|---|---|
| `src/probe_relock_drop_refused_by_a_freeze_and_resent_stopped_drop.rs`（从 `model/` 同名文件拷来，只加行、一行没删：`diff` 现数删 0 行，加的见第二节） | `0777dc188673e1baada749294e5d2ef0623428a44bbcf2ece69b5e4e342927e9` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps` 那一段与它上面的说明） | `4ccb1dddb62a74f4d88829f46179cf167fe63ffc90b0cb163238616a43429743` |
| `tools/rerun-from-a-clean-work-directory.sh`（拷自 `model/` 同名文件，逐字相同） | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `fixes.patch`（与草稿目录根的 `fixes.patch` 逐字相同） | `0918b39e4f49765cca66622c07c3118f14c37028a11e70144cd3e8aa8ec920d1` |
| `expected/without-the-fixes-lines-of-the-main-agent-rerun.out`（拷自主 agent 复跑的 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-main-run/out/result-lines.out`，26 行） | `6e07b8fc65bc6703e9f7875e24a19a47092a6e9dd2af5bb7f34a8b860c0d699c` |

两棵副本：`copy/` 是前一条攻方腿的冻结副本，这一轮一个字没改；`copy-with-the-fixes/` 是今天用 `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <草稿目录>/copy-with-the-fixes --with-test-inputs` 新建的，建完 `diff -rq copy copy-with-the-fixes -x target -x .git -x prove-red-logs` 只差 `crates/mutations.tsv`（`copy` 里多前一条腿追加的三行候选变异），之后只在它的 `crates/singlefs-core/` 里照三样改。`fixes.patch` 是 `git diff --no-index copy/crates/singlefs-core copy-with-the-fixes/crates/singlefs-core` 把路径前缀改成 `a/crates/`、`b/crates/`；在 `copy/crates/singlefs-core` 的拷贝上 `git apply` 之后与 `copy-with-the-fixes/crates/singlefs-core` `diff -r` 退 0（我直接跑的，只核补丁能复原那一棵）。改完的五份文件过了 `rustfmt --check --edition 2021`；没编过（不起 cargo）。

## 各格判定一览

| 格 | 判 | 一句话 | 证据状态 |
|---|---|---|---|
| 改乙（形甲：冻结的用户改动被改记成整理） | 推的：有改法那一臂修好 | 回锁那一次调用进门时重读冻没冻着，冻着时删意图被拒不改种类；推的 `error_relabelled=0`、`error_accounted_as_a_compaction=0` | 推的，待主 agent 跑 |
| 改丙（形乙：删意图被冻结拒掉不记、整理接着搬） | 推的：有改法那一臂修好 | 删意图交回任何错都照门压住那一支记下待删；推的 12 段出错且删被拒的段全记下，其中 10 段重发之后下一次调用把意图删掉、一个单元不搬，余下 2 段停在关着的回退门上照留，`relocated=0` | 推的，待主 agent 跑 |
| 改丁（形丙：重发成了的整理发布不清让路计数） | 推的：有改法那一臂修好 | 重发整理那一支清两个计数；推的 `frozen_yields_in_a_row_after_the_resend=0`、`frozen_yields_before_a_take_without_yielding=2` | 推的，待主 agent 跑 |
| 没有改法那一臂与主 agent 复跑逐字相同 | 推的：相同 | 探针原有的行一字没动、`copy` 一字没动；复跑脚本把这一臂的原有行与主 agent 复跑出的 26 行排序逐行比 | 推的，待主 agent 跑 |
| 对照臂（不出错、不冻结的段）两臂读数相同 | 推的：相同 | 三样改动只碰出错那一支、冻结那一支与重发整理那一支，对照臂走不到；复跑脚本逐行比两臂对照的 10 行 | 推的，待主 agent 跑 |
| 写带宽 | 没做 | 改丁照主 agent 定的只清两个计数，重发那一刻不记写带宽的字节（前一条腿第八节已写推不出怎么拿上限），形丙里写带宽那一半照旧不中 | 读码推的 |

## 一、三样改法落在哪（补丁 `fixes.patch`，只含 `crates/singlefs-core` 下五份文件，加 57 行、删 20 行，`git apply --stat fixes.patch` 现数）

下面引的行号都是主工作区今天的（与 `copy` 逐字相同，`diff -rq` 只差 `crates/mutations.tsv`），读时 `grep -nF` 核过；改后的样子看补丁。

### 改乙：只在冻结的就是这一次调用发的那一次时改种类

- 病根：一批推完记账出错那一支照「这一趟开头冻没冻着」判，「run.was_frozen_when_the_run_started,」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:664`）；这一项在开一趟那一刻记，「let was_frozen_when_the_run_started = allocator.frozen_publish().is_some();」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:269`）。放锁的入口开一趟在取输入那一次调用里，回锁是另一次调用，中间前台的发布可以冻结（形甲）。改种类那个函数只认传进来的这一项：「if was_frozen_before_this_call {」（`crates/singlefs-core/src/mounted_session.rs:462`）。
- 改法：字段改名 `was_frozen_when_this_call_came_in`（说明写清：锁里推完的入口是开一趟那一刻，放锁的入口回锁那一次是回锁那一刻），`CompactionRunUnderWay` 加一个方法 `note_whether_a_publish_is_frozen_as_the_relock_comes_in` 重读此刻冻没冻着；`publish_the_batch_computed_out_of_the_lock` 在认完票、核完盘表、`take_back_the_batch_out_of_the_lock` 之后、回锁头几判之前调它（今天那一行「.take_back_the_batch_out_of_the_lock(batch);」（`crates/singlefs-core/src/mounted_session/background_compaction_out_of_the_lock.rs:555`）之后）。两处改种类的调用（`compaction_run.rs:387` 停着那一趟开头、`:664` 一批推完记账）只换字段名。
- 为什么这样就等于「冻的就是这一次调用发的那一次」（读码推的）：会话锁里只有这一次调用在发；进门时冻着，发布路径第一道就拒（`refuse_while_a_publish_is_frozen`），这一次调用发不出任何一次，冻着的那一次不是它的；进门时没冻着、之后冻着了，冻的只能是这一次调用里发的那一次。锁里推完的入口开一趟与发布在同一次调用里，进门那一刻就是开一趟那一刻，行为不变。
- 比前一条腿第八节写的两种做法（比 txg、看被拒原因）选这一种的理由：比 txg 分不开（前台冻结的那一次与整理将要发的那一次 checkpoint_txg 相同，探针里两者都是 `CheckpointTxg(8)` 一类，推的）；看被拒原因要剥 `CompactionRunFailure::Batch(RelocationBatchRefused::Publish(PublishError::…))` 几层，层数一变就判错。重读进门那一刻只看一个布尔，与改种类那个函数已有的口径相同。
- 顺带修的一格（推的，探针没造）：取输入那一刻冻着、锁外期间重发成了、回锁那一次发布自己写盘报错冻结的，修之前「这一趟开头冻着」→ 不改种类，冻着的那一次留在发布路径刚冻结时的 `NeitherOfTheCountedKinds`，重发成了两样都不记；修之后照整理那种记。

### 改丙：回锁出错那一支删意图交回任何错都记下待删

- 病根：今天只在门压住那一支记，「compaction.hold_back_the_drop_of_the_intent_of_an_abandoned_batch(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:702`）；另一支「drop_the_live_intent_on_a_stop(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:709`）交回错时什么都不记（形乙）。
- 改法：把门压住那一支里「现行那一版有活意图就记下」那几行抽成 `note_the_live_intent_of_the_abandoned_batch_still_to_drop(compaction, current, stop)`（`compaction_batch.rs` 里按名字搜过没有同名），门压住那一支照调；删意图那一支 `let dropped = drop_the_live_intent_on_a_stop(…)`，`dropped.is_err()` 时同样调它，再交回 `dropped`（错照旧交回，这一趟照旧以失败收尾，不改成停：那是改甲，主 agent 定了不做）。顺改两处说明：`hold_back_the_drop_of_the_intent_of_an_abandoned_batch` 的说明加「或没发成」；`drop_the_live_intent_on_a_stop` 的 `# Errors` 末尾补「回锁里出错那一支没删成的，调用方把它记在会话上，取下一批的输入之前先删」。
- 三种错各怎样（读码推的）：冻结着被第一道拒、R 里外取不到落点——盘上不变、意图还在，记下的就是它，之后取下一批的输入之前先删（`take_the_input_of_one_compaction_batch` 第一步，门与破例照已有那一判）；删意图那一次自己写盘报错冻结——记下的那条在冻结那一次重发成了之后已不在现行那一版，下一次取输入走「现行那一版里已不是它，就清掉那一条照常取」那一句（`take_the_input_of_one_compaction_batch` 的说明），不误删别的意图。
- 与条款：「正在算的那一批放弃，意图也删掉」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`），第 5 条补的那句「门开或破例成立就只发这一次删意图，门关就这一趟不发、也不在这条意图下接着搬」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）原本只说被门压住的；改丙让被冻结、空间拒掉的走同一条路。这一读法是不是条款要的，归主 agent 判（前一条腿第二节已写这一形归第 4、8 条）。

### 改丁：重发整理那一支清让路的两个计数

- 病根：「KindOfTheFrozenPublish::CompactionPublish => {」（`crates/singlefs-core/src/mounted_session.rs:1135`）那一支只扣预算、记根槽，到「self.compaction.observations.root_ring_slots_used += 1;」（`crates/singlefs-core/src/mounted_session.rs:1137`）为止，不清计数（形丙）。
- 改法：那一支末尾加 `self.compaction.clear_the_yields_in_a_row();`（它清的就是两个计数：连着让路的次数与连着让路开始那一刻的读数），注释写「冻结的是后台批，照后台批记：这一次整理发布在重发成了这一刻发成，一批发成，让路的两个计数都清零」。
- 与条款：「冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）；冻结的是后台批，照后台批记」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`），同一行「一批发成两个都清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）。写带宽那一半不做（见一览末行）。

## 二、探针与复跑脚本改了什么

- 探针：原有的每一行一字不改（`diff model/src/… model-with-and-without-the-fixes/src/…` 现数删 0 行），两臂跑同一份。格一每段在原来那一行之后另打一行 `RESULT name=relock_drop_refused_by_a_freeze_in_the_window_next_take`：`relock_drop_refused_by_the_freeze`（回锁交回的是 `failed:` 起头且带 `PublishFrozenAfterAWriteFailureIsNotResentYet`）、`noted_at_the_relock`（回锁之后会话记没记着待删，即原来那一行的 `held_back_noted`）、`intent_before_the_entry`、`next`、`units_relocated`、`intent_after_the_next_call`（下一次调用之后现行那一版的活意图：`none` / `the_abandoned_one` / `another_one`）、`noted_after_the_next_call`、`abandoned_intent_dropped_without_moving`（之前是那条放弃的意图、之后没有活意图、搬 0 个）、`abandoned_intent_kept_without_moving`（之前之后都是它、搬 0 个）。格一末尾另打一行汇总 `relock_drop_refused_by_a_freeze_in_the_window_next_take_summary`。格二不加行。
- 复跑脚本 `rerun_steps`：
  1. 核两臂只差补丁：`copy/crates` 拷一份打上 `$MODEL/fixes.patch`，与 `copy-with-the-fixes/crates` `diff -r`（不比 `mutations.tsv`），两棵 `crates/` 之外 `diff -r`（不比 `prove-red-logs/`，那是主 agent 在 `copy` 里跑证红留下的）；打不上或有差退 3，打 `name=arms_differ_only_by_the_fixes_patch`。
  2. 同一份探针拷进两臂验证包的 `tests/`，先没有改法那一臂、再有改法那一臂，各跑 `cargo test --offline -p singlefs-verification --test probe_relock_drop_refused_by_a_freeze_and_resent_stopped_drop -- relock_drop_refused_by_a_freeze_in_the_window stopped_drop_frozen_at_the_trip_start_and_the_yields_in_a_row --exact --nocapture`；`RESULT`、`SELF` 行行尾贴 ` arm=without_the_fixes` / ` arm=with_the_fixes`（格二的段行本来就有一个 `arm=`，指历史那一臂；行尾那一个是 A/B 的臂）。
  3. 没有改法那一臂：探针原有的行（名字不带 `_next_take`）去掉行尾臂名、排序，与 `expected/` 里主 agent 复跑出的 26 行排序逐行比，打 `name=without_the_fixes_matches_the_main_agent_rerun`；对不上退 4。
  4. 对照：两臂里 `failure=no_error_control` 与 `arm=published_control` 的行（格一对照 8 段：门不判与门关两面各 4 段，原有行 8、新加行 8；格二对照臂 1 行；推的共 17 行）去掉行尾臂名、排序逐行比，打 `name=control_lines_equal_across_the_arms`（只打读数，不改退出码：对照臂两臂不同是改法打坏了别的，是这一次要的读数，不是装置坏了）。
  5. 打 `name=rerun_done probe_exit_without_the_fixes=… probe_exit_with_the_fixes=…`，再把两臂的行全打出来；探针退非 0 照它的码退。
- 自证：模板的 `--selftest` 第一格要 `rerun_steps` 在只有 `crates/marker.txt` 的假冻结副本上退 0，这一份 `rerun_steps` 在那里找不到两棵副本、退 2，那一格会红（前一条腿那一份同样：它的 `rerun_steps` 在自证里拷 `$MODEL/src/…` 拷不到）。所以第六节不列自证命令；要不要为复跑脚本补一份能过自证的写法，交主 agent 定。

## 三、两臂推的读数与推翻条件

没有改法那一臂两行汇总推的与主 agent 复跑的逐字相同：

- `RESULT name=relock_drop_refused_by_a_freeze_in_the_window_summary histories=20 premises_held=20 error_relabelled=12 control_relabelled=0 error_accounted_as_a_compaction=12 control_accounted_as_a_compaction=0 same_entry_key_relocated_under_the_abandoned_intent=5 device_lost_relocated_under_the_abandoned_intent=0 control_relocated_under_the_intent=6 arm=without_the_fixes`
- `RESULT name=stopped_drop_frozen_at_the_trip_start_and_the_yields_in_a_row_summary histories=2 premises_held=2 control_yields_in_a_row_after_the_drop=0 frozen_yields_in_a_row_after_the_resend=1 control_yields_before_a_take_without_yielding=2 frozen_yields_before_a_take_without_yielding=1 arm=without_the_fixes`

（这两行是复跑脚本贴了臂名之后的样子，去掉行尾 ` arm=without_the_fixes` 就是派发提示里那两行。）

逐格推的读数（「无」是没有改法那一臂，「有」是有改法那一臂；全部推的，待主 agent 跑）：

| 汇总行的项 | 无 | 有 | 哪一样改法管它 |
|---|---|---|---|
| 格一 `premises_held` | 20 | 20 | 三样都不碰前提 |
| 格一 `error_relabelled` | 12 | 0 | 改乙 |
| 格一 `error_accounted_as_a_compaction` | 12 | 0 | 改乙（种类留在用户改动，重发成了走「self.compaction.note_a_user_change_published();」（`crates/singlefs-core/src/mounted_session.rs:1126`），自上一次用户改动以来的发布数归 0） |
| 格一 `control_relabelled` / `control_accounted_as_a_compaction` | 0 / 0 | 0 / 0 | 对照 |
| 格一 `same_entry_key_relocated_under_the_abandoned_intent` | 5 | 0 | 改丙（改乙让重发之后建 0 个 inode 的后台入口那一段也起跑，起跑了走取输入第一步先删） |
| 格一 `device_lost_relocated_under_the_abandoned_intent` | 0 | 0 | 掉卡臂停着，下一趟开头那一支照删 |
| 格一 `control_relocated_under_the_intent` | 6 | 6 | 对照（这条意图没有放弃，照搬是对的） |
| 新汇总 `error_drop_refused_at_the_relock` | 12 | 12 | 改甲才会改它；不做改甲，两臂都是 12 |
| 新汇总 `error_drop_refused_noted_at_the_relock` | 0 | 12 | 改丙 |
| 新汇总 `error_drop_refused_then_dropped_without_moving_at_the_next_call` | 4 | 10 | 改丙；无那一臂的 4 是掉卡臂（停着那一支删的），有那一臂多出门不判那一面重复键 4 段与门关那一面显式入口 2 段 |
| 新汇总 `error_drop_refused_then_kept_without_moving_at_the_next_call` | 3 | 2 | 有那一臂剩门关那一面后台入口两段：停在回退门上、意图留着、会话照记着，第 256 行「门关就这一趟不发、也不在这条意图下接着搬」 |
| 新汇总 `error_drop_refused_then_relocated_under_the_abandoned_intent` | 5 | 0 | 改丙 |
| 新汇总 `control_noted_at_the_relock` / `control_relocated_under_the_intent` | 0 / 6 | 0 / 6 | 对照 |
| 格二 `premises_held` | 2 | 2 | 冻结臂那一次删意图是这一趟开头这一次调用自己发的、自己冻的，改乙照旧改记成整理那种 |
| 格二 `control_yields_in_a_row_after_the_drop` | 0 | 0 | 对照 |
| 格二 `frozen_yields_in_a_row_after_the_resend` | 1 | 0 | 改丁 |
| 格二 `control_yields_before_a_take_without_yielding` | 2 | 2 | 对照 |
| 格二 `frozen_yields_before_a_take_without_yielding` | 1 | 2 | 改丁（计数从 0 起，让满下限 2 次才交） |

有改法那一臂的两行汇总推的是：

- `RESULT name=relock_drop_refused_by_a_freeze_in_the_window_summary histories=20 premises_held=20 error_relabelled=0 control_relabelled=0 error_accounted_as_a_compaction=0 control_accounted_as_a_compaction=0 same_entry_key_relocated_under_the_abandoned_intent=0 device_lost_relocated_under_the_abandoned_intent=0 control_relocated_under_the_intent=6 arm=with_the_fixes`
- `RESULT name=stopped_drop_frozen_at_the_trip_start_and_the_yields_in_a_row_summary histories=2 premises_held=2 control_yields_in_a_row_after_the_drop=0 frozen_yields_in_a_row_after_the_resend=0 control_yields_before_a_take_without_yielding=2 frozen_yields_before_a_take_without_yielding=2 arm=with_the_fixes`

推翻条件（跑出下面任一样，对应的那一条推断就是错的，照它撤）：

- `name=arms_differ_only_by_the_fixes_patch` 不是 `patch_applied=true crates_after_the_patch_differ=0 rest_of_the_trees_differ=0`：两臂不只差补丁，这一次 A/B 不作数（复跑退 3）。
- `name=without_the_fixes_matches_the_main_agent_rerun` 的 `diff_exit` 不是 0：没有改法那一臂与主 agent 复跑的对不上，探针加行时碰到了原有读数，或副本变了（复跑退 4）。
- 有改法那一臂格一汇总 `error_relabelled` 不是 0，或 `error_accounted_as_a_compaction` 不是 0：改乙没修住形甲。
- 有改法那一臂新汇总 `error_drop_refused_noted_at_the_relock` 小于 12，或 `error_drop_refused_then_relocated_under_the_abandoned_intent` 不是 0：改丙没修住形乙。`dropped_without_moving` 是 9、`kept_without_moving` 是 3 推翻的是我对门不判那一面重复键、后台入口、重发之后建 0 个 inode 那一段的推断（我推它起跑：重发用户改动那一支在水位判起跑时记「该跑了」，强制起跑时水位判是「return WatermarkVerdict::ForcedToStartByTheTestOnlySwitch」（`crates/singlefs-core/src/compaction_plan/watermarks.rs:249`），`starts()` 对它答真），不推翻改丙：没起跑就没搬，同样不是接着搬；那时把那一段的段行贴出来交主 agent 判。
- 有改法那一臂格二 `frozen_yields_in_a_row_after_the_resend` 不是 0：改丁没修住形丙。
- `name=control_lines_equal_across_the_arms` 的 `diff_exit` 不是 0：三样改法打坏了对照臂，看 `control-lines-across-the-arms.diff` 哪几段变了。
- 有改法那一臂两条测试有一条不是 `ok`（前提断言或格二「对照臂：删意图发成清零」那一条红）、或编不过：改法本身写错，交承接的腿改。

## 四、四句（三样改法各对它修的那一格）

- 分不分辨臂：分辨。三格都是前一条腿第一到第三节打中的那几形，出错（或冻结）臂两臂推的读数不同，对照臂推的两臂相同。
- 被判的系统当时看不看得到判别它的东西：看得到。改乙判的是回锁进门那一刻 `allocator.frozen_publish()` 在不在；改丙判的是删意图交回的 `Result`；改丁判的是冻结着的那一次的种类。三样都是做决定那一刻会话手里就有的。
- 满足的是判据字面的哪一个分句：改乙对第 257 行「冻结的是用户改动，照一次用户改动发成办（重给预算、清掉上一条完成的意图）」；改丙对第 255 行「正在算的那一批放弃，意图也删掉」、第 256 行补的「也不在这条意图下接着搬」；改丁对第 257 行「冻结的是后台批，照后台批记」接「一批发成两个都清零」。
- 跑前条款给的每个改法在打中的那几格上还中不中（前一条腿第八节那张表，这一次三样合起来）：形甲改乙中、改丙不中、改丁不中；形乙改丙中、改乙不中（单看）、改丁不中，合起来之后改乙还让重发之后建 0 个 inode 的后台入口那一段起跑、走到取输入第一步先删（推的）；形丙改丁中、改乙改丙不中。改甲没做：它在形甲、形乙上与改乙、改丙重叠（前一条腿第八节），不做它，回锁那一次照旧以失败收尾、交回删意图被冻结拒掉的错（新汇总 `error_drop_refused_at_the_relock` 两臂都是 12）——这一次删意图失败本身还算不算一次「整理失败」、要不要照改甲改成停，交主 agent 定。

## 五、几个改法各修哪一格（只在我的探针上推过、被攻过零轮）

| 改法 | 形甲（冻结的用户改动被改记成整理） | 形乙（删意图被拒不记、接着搬） | 形丙（重发成了的整理发布不清计数） | 对照臂 |
|---|---|---|---|---|
| 改乙 | 中（推的，待主 agent 跑） | 单看不中；合起来让重发之后建 0 个 inode 的后台入口那一段起跑（推的，待主 agent 跑） | 不中（推的） | 不变（推的，待主 agent 跑） |
| 改丙 | 不中（推的） | 中（推的，待主 agent 跑） | 不中（推的） | 不变（推的，待主 agent 跑） |
| 改丁 | 不中（推的） | 不中（推的） | 中，写带宽那一半不中（推的，待主 agent 跑） | 不变（推的，待主 agent 跑） |
| 三样合起来 | 推的 `error_relabelled=0` | 推的 `error_drop_refused_then_relocated_under_the_abandoned_intent=0`、`noted=12` | 推的 `frozen_yields_in_a_row_after_the_resend=0` | 推的 17 行逐行相同 |

没有一格标「量过」：这一条腿没有主 agent 给的号可读，三样改法一行都没编过。

## 六、要主 agent 跑的命令

1. 改法 A/B 复跑（两臂各编一次、各跑一遍探针；不调崩溃枚举，崩溃状态 0 个）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model-with-and-without-the-fixes/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus-s2-run /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/model-with-and-without-the-fixes`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus`（`copy` 与 `copy-with-the-fixes` 都在 `.deliverables` 写成「交接」）
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `name=arms_differ_only_by_the_fixes_patch patch_applied=true crates_after_the_patch_differ=0 rest_of_the_trees_differ=0`、`name=without_the_fixes_matches_the_main_agent_rerun … diff_exit=0 differing_diff_lines=0`、`name=control_lines_equal_across_the_arms … diff_exit=0`、`name=rerun_done probe_exit_without_the_fixes=0 probe_exit_with_the_fixes=0`，再读两臂各两行 `_summary`、一行 `_next_take_summary` 与第三节表对。退 3：两臂不只差补丁；退 4：没有改法那一臂对不上；别的非 0：探针那一臂编不过或前提断言红，看运行目录 `out/probe-*.out`。
   - 估时与内存（推的，没量）：两棵树各编一次验证包的这个测试目标（服务给编译目录，两臂共用时第三方依赖只编一次），探针每臂两条测试，主 agent 上一次复跑同一份探针（不含新加的行）日志里 `finished in 115.50s`；推的挂钟在 40 分钟以内、内存由编译主导，没量。
2. 有改法那一臂上核三样改法没打坏已有用例（点名 17 条：回锁出错与回退门、让路、回锁时冻结三个子模块里的全部用例）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_batch_computed_out_of_the_session_lock 里的 17 条测试; 范围=17 个测试; 为什么=改法 A/B 有改法那一臂的副本上核改乙改丙改丁没打坏回锁出错、让路与回锁时冻结三个子模块已有的用例' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy-with-the-fixes -- cargo test -p singlefs-verification --test compaction_batch_computed_out_of_the_session_lock -- compute_errors_and_the_rollback_gate_closed_in_the_window::a_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does compute_errors_and_the_rollback_gate_closed_in_the_window::a_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_on_the_cpu_judge compute_errors_and_the_rollback_gate_closed_in_the_window::a_relock_after_the_rollback_gate_closed_in_the_window_publishes_nothing compute_errors_and_the_rollback_gate_closed_in_the_window::a_compute_error_at_the_relock_after_the_rollback_gate_closed_in_the_window_keeps_the_live_intent compute_errors_and_the_rollback_gate_closed_in_the_window::after_a_compute_error_lifted_by_a_switch_requested_before_it_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens compute_errors_and_the_rollback_gate_closed_in_the_window::after_the_same_entry_key_twice_in_one_batch_the_drop_held_back_by_the_gate_is_published_once_the_gate_reopens compute_errors_and_the_rollback_gate_closed_in_the_window::a_compute_error_at_the_relock_after_the_floor_was_crossed_in_the_window_drops_the_intent_behind_the_closed_gate compute_errors_and_the_rollback_gate_closed_in_the_window::the_drop_held_back_by_the_gate_is_published_behind_the_closed_gate_when_the_drain_before_the_next_batch_crosses_the_floor yields_to_a_foreground_entry_waiting_for_the_session_lock::a_foreground_entry_waiting_for_the_session_lock_holds_back_the_next_batch_and_keeps_the_trip_under_way yields_to_a_foreground_entry_waiting_for_the_session_lock::after_yielding_as_many_times_in_a_row_as_the_floor_allows_the_next_batch_does_not_yield yields_to_a_foreground_entry_waiting_for_the_session_lock::after_yielding_for_as_long_as_the_floor_allows_the_next_batch_does_not_yield yields_to_a_foreground_entry_waiting_for_the_session_lock::a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row yields_to_a_foreground_entry_waiting_for_the_session_lock::a_take_of_the_input_without_yielding_breaks_the_yields_in_a_row_even_when_its_batch_publishes_nothing yields_to_a_foreground_entry_waiting_for_the_session_lock::a_drop_published_at_the_start_of_a_trip_while_the_compaction_stays_stopped_clears_the_yields_in_a_row publishes_frozen_and_drains_refused_at_the_relock::a_batch_relocked_while_a_publish_frozen_in_the_window_is_dropped_without_failing_the_compaction publishes_frozen_and_drains_refused_at_the_relock::a_batch_taken_while_a_publish_is_already_frozen_is_dropped_at_the_relock_without_failing_the_compaction publishes_frozen_and_drains_refused_at_the_relock::a_relock_whose_drain_of_the_pending_changes_is_refused_hands_back_the_refusal_and_drops_the_batch --exact`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/copy-with-the-fixes`（`.deliverables` 写成「交接」）
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志里 `test result: ok. 17 passed; 0 failed`。推的是 17 条全绿（三样改法碰的那几处，这 17 条里只走删意图发成、门压住、回锁时冻结丢批三支；断言会话记着待删的几条都在删意图发成或门压住那一支，改丙只加在删意图出错那一支）。范围声明闸若判这个目标会走崩溃枚举、要缩域或全域理由，照拒绝说明改。名字是按 `#[test]` 下一行的 `fn` 读的，`--exact` 点不中就是我读漏了属性行，看日志里 `running N tests` 的 N。

## 没打中的形状

这一条腿只做 A/B，不另攻别的面，没有新的打中。造历史与取样沿用前一条腿的探针：格一 5 个前缀（重复键 / 掉卡 / 不出错 × 门不判；重复键 / 不出错 × 门关而破例成立）× 重发之后的入口 2 种 × 重发之后建 0、1 个 inode，共 20 段；格二 2 段；几何只有 `common::build_pool` 那一份默认几何（S = 8）。由用户决定的两步（入口、建几个 inode）照旧放开扫，前缀与故障固定。

读码时顺带核过、没写成打中的：

- 改乙改名的那一项全仓只在 `compaction_run.rs` 四处与 `mounted_session.rs` 字段一处（`grep -rn was_frozen_when_the_run_started crates/singlefs-core/src` 现数 5 行），放锁的入口取输入那一次调用里推完的（取输入那一刻删待删、停着那一支）与开一趟同一次调用，进门那一刻就是开一趟那一刻，不用重读。
- 改丙记下的那一条在写盘报错冻结、之后重发成了的形里，下一次取输入照已有那一句清掉，不误删别的意图；被删意图那一次冻结着时它照旧以失败收尾（不做改甲）。
- 改丁那一支里冻结的整理发布只有两种来路（一批推完记账与停着那一趟开头），都是整理发布，清计数对两种都成立；用户改动那一支不碰。

## 这条腿自己的限度

- 三样改法与探针新加的行一行都没编过（`cargo fmt` 被工具层的权限判定拒了、派发提示也写了不起 cargo，改用 `rustfmt --check --edition 2021` 只核了格式；探针在临时目录里配空的 `common` 两个模块核的格式），编不过要承接的腿改。读数全是推的。
- 改乙选了「回锁进门那一刻重读」这一种实现（理由见第一节），前一条腿第八节另写的两种（比 txg、看被拒原因）没实现、没比。
- 改丙只改回锁出错那一支；停着那一趟开头那一次删意图被拒、取输入第一步删待删被拒，照旧留给下一次（前者下一趟开头再删，后者会话照旧记着），没造历史。
- 写带宽那一半没做（主 agent 只定了清两个计数）；形丙里重发成了的整理发布照旧不记写带宽的字节。
- `has_an_intent_of_an_abandoned_batch_still_to_drop` 在删意图那一次自己冻结、重发成了到下一次取输入之前照旧答真（前一条腿检查单「类型与名字」那一条），改丙让这一形多了一条来路（写盘报错冻结时也记下），没改。
- 复跑脚本的 `--selftest` 第一格在这一份 `rerun_steps` 上会红（第二节末条），没修。
- 有改法那一臂只核了 17 条已有用例（第六节第 2 条），这一批合入后验证点过的 54 条与别的目标没点；采纳改法时由实现员在快照上照常跑。

## 没做什么

- 不判正推腿那几格，不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务、没起 cargo、没跑门禁；判决要引的数没复跑（命令在第六节）。
- 没改主工作区任何文件（只写了这一份报告）；`copy` 与前一条腿的 `model/` 一个字没改。
- 没做改甲。

## 清理

- 草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r3-opus/` 里这一条腿新建的：`copy-with-the-fixes/`（仓副本 64M，`.deliverables` 写成「交接」：主 agent 在里面跑第六节第 2 条，第 1 条以草稿目录根为冻结副本读它）、`model-with-and-without-the-fixes/`（探针、复跑脚本、补丁与期望行，「报告」）、`fixes.patch`（「补丁」）、`minimal-copy-with-the-fixes.txt`（建副本的输出）。删了自己的替换小工具与改动片段 `fix-tools/`（136K）、`probe-diff.txt`（8.0K）。没有编译目录。主 agent 用完删 `copy/` 与 `copy-with-the-fixes/`。
