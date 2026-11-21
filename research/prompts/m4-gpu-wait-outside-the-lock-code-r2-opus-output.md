# m4-gpu-wait-outside-the-lock-code-r2：攻方腿（Opus）报告（2026-11-20）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r2-body.md`；上一轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md`。开工时核过开工快照 `crates-sha256.txt` 的 23 份文件与主工作区逐份相同（`sha256sum -c` 没有一行不是 OK）。下文的数凡没写「量的」都是读码推的；探针没在任何地方跑过，读数都待主 agent 跑。

## 复跑命令与文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-model/`，冻结副本 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy`（`scripts/hosts/minimal-copy.py --with-test-inputs` 从主工作区建，另在副本的 `crates/mutations.tsv` 末尾追加了四条候选变异行，原文副本在草稿目录 `candidates/candidate-mutation-rows.tsv`）。

复跑（主 agent 交，见「要主 agent 跑的命令」第 1 条）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-model
```

| 文件 | sha256 |
|---|---|
| `src/probe_held_back_drop_and_stopped_drop_yields.rs` | ac2c46d960f573a03450895485021be90e0b3fe1a14545396583cde004a61cd5 |
| `tools/rerun.sh` | 265320abf2f677b335c149805e66817cfb0b3420145959ccdd57297122e4bcd4 |
| `tools/rerun-from-a-clean-work-directory.sh` | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |
| 草稿目录 `candidates/candidate-mutation-rows.tsv` | f9f470f0503dadf7f199a802a7789db5db8711bb1f4ccf295b1dcbd928e1e09e |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`；`rerun-from-a-clean-work-directory.sh` 从 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/` 原样拷。两份过了 `bash -n`。探针没编过（子 agent 不起 cargo）：编不过时 `probe_exit` 非 0，交回承接的同立场腿改。

探针不调崩溃枚举函数，崩溃状态数 0；三格一共 8 + 2 + 4 = 14 段历史，每段一个 `common::build_pool` 池（同第一轮攻方探针的取法，那一份 36 段历史主 agent 复跑成过）。挂钟与内存没量，推的：与第一轮探针同量级（编验证包一个测试目标加 14 个小池），没有超过 40 分钟的理由，所以没先跑一小段估时。

## 各格判定一览

| 格 | 判定 | 证据在哪（详见各节） |
|---|---|---|
| 修补第 1 条（回锁出错那一支过回退门） | **打中**（两形） | 甲：出错之后整理不停（只放弃这一批的出错、出错之前已请求的切换）或停着而补跑时，门关压住的删意图门开之后不再补发，这条意图被接着搬；乙：窗口里破例条件变成立、门关，删意图照样压住。探针格一、格二，推的，待主 agent 跑 |
| 修补第 2 条（回锁有冻结就丢批） | 没打中 | 第三节 |
| 修补第 3 条（建缓冲那一等读停标记） | 没打中 | 第三节 |
| 修补第 4 条（fsync 先 `try_lock`） | 没打中（不是修补改出的） | 第三节：试锁失败到登记之间、放锁到减计数之间两个窗口，与别的前台入口同一写法 |
| 修补第 5 条（补测试与变异行） | 部分没补 | 第二节：空洞数、要搬数、资格的两个合取项四处仍没有会红的用例 |
| 修补第 6 条（不切片理由、注释口径） | 没打中 | 12/12、10/10，量的（grep 计数） |
| 修补第 7 条（取了输入没让也清零） | **打中**（判据字面） | 清零落在写带宽与回退门两判之前：没取输入、下限也没到就清；新用例断言的正是这一形。病根多半在 kb 写回的措辞，交主 agent 判 |
| 三处之一：停着时开头单发的删意图清不清让路计数 | **坐实：不清** | 一趟开头那一支发成之后只记预算、根槽、写带宽，不调清零；探针格三 |
| 三处之二：签发那一支空洞数、要搬数取现行那一版 | 坐实：仍罩不到 | 补的用例只断言 `c0`、`segs0`；空洞数、要搬数没有用例；候选变异两条交主 agent 证红（预期没红） |
| 三处之三：资格的「开过的段」「钉住与隔离」 | 坐实：罩不到 | 两条候选变异交主 agent 证红（预期没红） |
| 合入之后修的两行变异 | 锚点成立；红不红待跑 | 原文在今天的源码各命中 1 次、替换文 0 次，量的；证红命令在「要主 agent 跑的命令」 |
| 测试测的是条款还是实现 | 两条测的是实现 | 让路「取输入没让路」那条断言的是没取输入、没到下限也清零；出错压住那条只钉「门关不发」，不钉「门开补发」 |
| 检查单九格 | 九格成立（「带 --item」那一格只核了点名的测试在拆后的树里各找得到 1 处，`--list-items` 没跑） | 第五节 |

## 一、修补第 1 条：回锁出错那一支过回退门（打中两形）

条款（`.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13）：「正在算的那一批放弃，意图也删掉（单发一次只删意图的发布，已定项 1 第 3 条）」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`）；「停着时那一次只删意图的发布（第 4 条）同样过这道门：门关就不发，意图与 `R` 排他留到门开；破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发。」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）。第一轮判决第 1 条把出错那一支单发的删意图读成这一次（判决第二节表第 1 行）。

### 甲：压住的删意图只在「停着」那一条路上补发，别的出口永远不发

修补的写法：门关就交回 `Stopped`、不发（「let step = if is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:666`）），它的文档只写了一条补发的路：「/// （[`is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate`]：门关就不删，意图与 R 排他留到门开，停着时那一趟开头那一判再删）；」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:625`）。补发只在一趟开头判出停着那一支：「if compaction_stays_stopped(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:345`）。会话上没有记下「有一次删意图被门压住」，之后取输入看到的是一条普通的活意图。

出错之后不进那一支的出口有四个，每个都由条款许可：

| 出口 | 代码 | 条款许可那一步的原文 |
|---|---|---|
| 出错之前已请求、还没生效的切换 | 「return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;」（`crates/singlefs-core/src/background_compute/compute_of_this_mount.rs:111`） | 「出错之前已经收到、还没生效的切换请求（只指请求了、还没交给实现的那一次），在停下那一刻也算数，照它解除停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`） |
| 只放弃这一批的出错（重复键） | 「ReachOfTheComputeError::AbandonsOnlyTheBatch => {」（`crates/singlefs-core/src/background_compute/compute_of_this_mount.rs:105`） | 「那一批的意图照第 4 条一批出错放弃的规矩删（有活意图就单发一次只删意图的发布）」（`.claude/kb/decisions/26-后台整理与放置回收.md:259`） |
| 停着而占用超过阈值、在 CPU 上补跑 | 「WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(_) => false,」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:66`） | 「这一批在 CPU 上补跑」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`） |
| 新卡开卡期间出错停下、门开之前新卡开成 | 同上表第 1 行那一份状态，停下由换上解除 | 「实现答新卡开成（换上了）那一刻解除停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`） |

造出来的历史（切换那一形，探针格一 `switch_requested_before_the_error` 臂）：

1. 前缀：挂载选 `gpu`、开关照水位判（越线用只供测试的开关）、在单元区头一段签着一条意图、根环装满不同用户状态；取输入那一刻门开（「if run.is_the_rollback_gate_judged && !is_this_publish_the_exception {」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:565`）判过）。
2. 故障：锁外那一批 GPU 实现交回设备丢失。
3. 用户在窗口里请求切到 `cpu`（用户动作）；前台推抬 F 的空发布推到门关（与 `accept` 被空间拒时当场推的那一串同一条路）。
4. 回锁：出错先于一切，`stop_the_task_after_a_compute_error` 交回解除停下；现行那一版变了、门关，删意图压住（「current.root().checkpoint_txg != version_of_the_ticket.root.checkpoint_txg」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:765`））。整理没有停着。
5. 用户改动到门重开，再多改 `extra` 次（用户动作，探针扫 0、3）。
6. 下一次调用走后台入口或显式入口（用户动作，两种都扫）：没停着，不进第 345 行那一支；取输入看到活意图，照活意图那一支在 CPU 上接着搬 R。那一次被压住的删意图永远不发。

对照臂（`stays_stopped`）：同一前缀与故障、不请求切换，回锁之后整理停着，门开之后那一次调用在一趟开头单发删意图。两臂回锁那一刻都门关、都压住，只差停没停。修补之前回锁那一刻照发删意图，这段历史写不出来。

打中之后的四句：

1. 分辨臂：分辨（对照臂补发，这一臂不发）。
2. 系统做决定那一刻看不看得到判别它的东西：现在看不到，修补没在会话上留下「有一次删意图被压住」的记号；条款要的事系统完全记得下，不是判据要了看不到的东西。
3. 满足的分句：第 256 行「门关就不发，意图与 `R` 排他留到门开」的「留到门开」，连同第 255 行「意图也删掉」（重复键那一形是第 259 行「那一批的意图照第 4 条一批出错放弃的规矩删」）。
4. 改法在这几格上中不中：见下面乙之后的改法表。

### 乙：回锁那一刻破例条件已成立，删意图照样被门压住

破例按取输入那一刻定：「if run.is_the_rollback_gate_judged && !run.is_the_batch_the_exception {」（`crates/singlefs-core/src/mounted_session/compaction_publish.rs:288`），`is_the_batch_the_exception` 是一趟开头的预算档在取输入之前那一判里算的（「== CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:502`））。回锁里出错那一支不再看此刻前台写不写得进。

历史（探针格二 `floor_crossed_in_the_window` 臂）：前缀与故障同甲；取输入那一刻全空段不低于底额、门开；窗口里前台写把全空聚簇段用到底额之下（前台写不进，`accept` 被空间拒时推抬 F 串，正是它把门推关）；回锁：出错、门关、第 288 行交回判门，删意图压住。条款第 256 行「破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发」此刻成立。整理停下的那一形下一次调用（`start_while_the_compaction_stays_stopped` 按此刻前台写不进给破例档）会不过门删掉，晚一次叫起；前台写不进时没有发布触发，那一次叫起多半是定时兜底（默认 30 秒，推的）；甲那几种不停的出口则永远不发。探针用只供测试的底额开关（`EmptyClusterSegmentsFloor::ForcedForTests(u64::MAX)`）在窗口里把底额抬到全空段数之上来模拟「前台写满」，判的谓词是同一个 `foreground_writes_are_below_the_empty_cluster_segments_floor`；对照臂在取输入之前就抬（这一趟走破例档），回锁那一刻两臂的门与底额相同。

四句：分辨（对照臂照发，这一臂不发）；看得到（谓词在回锁那一刻读得到分配器）；满足第 256 行「破例的条件…成立时，这一次也照发」；改法见下表。回锁里发普通一批那一支同样按取输入那一刻定破例，那是修补之前就有的写法、不是这一轮的新错，只记在这里。

### 改法（只在我的模型上推过、没实现没跑，被攻过零轮）

| 改法 | 甲（切换、重复键、补跑、开卡换上） | 乙（窗口里破例变成立） |
|---|---|---|
| 一、会话上记「出错那一支删意图被门压住」，之后每一次调用在判过回退门之后、取输入之前先单发那一次删意图（过门；破例照发），发成清记号 | 推的：中（四个出口都要经取输入之前那一判） | 推的：只把延迟缩到下一次调用，不中 |
| 二、出错那一支判门之前先看此刻 `foreground_cannot_write_under_the_switches`，成立就当破例照发 | 推的：不中 | 推的：中 |
| 三、撤回修补第 1 条（门关照发） | 推的：中 | 推的：中；但回到第一轮判决第 1 条采的那一格（门关照发违背第 256 行），不可取 |

一与二合起来罩甲乙两形。

## 二、第一轮只抽查过的三处

### 停着时开头单发的删意图发成之后，清不清让路的两个计数：坐实，不清

一趟开头判出停着那一支发成删意图之后只记了预算、根槽与写带宽，就把这一版推进这一趟：「let dropped_on_the_stop = if budget == 0 {」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:362`）之后那一段的 `Published` 分支在「run.versions_published.push(*version);」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:377`）收住，没有调 `clear_the_yields_in_a_row`。同一份文件里一批推完之后记账的那一处是清的：「的一批是一次整理发布：签发、搬一批、删意图都清零。」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:603`）、「compaction.clear_the_yields_in_a_row();」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:604`）。取输入那一刻判出停着的删意图（「step: drop_the_live_intent_on_a_stop(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:37`））与回锁出错那一支的删意图都经那一处记账，所以清；只有一趟开头这一支不清。第一轮判决第 8 条定的口径是「没限定搬没搬单元，签发也是一次整理发布」（`research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md:38`），条款是「连着让路 N 次或让路累计 T 毫秒，哪一个先到下一批就不让，一批发成两个都清零；下限到了、取了输入而没让路的那一批也把两个计数清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）。

计数此刻大于 0 才看得出：停着之后的调用不问让路（一趟开头那一支在问调用方之前），一批出错停下之前那一次取输入已经因没让路清过零；所以要靠「先让过路、再在两批之间进停着」。历史（探针格三 `drop_at_the_trip_start_while_stopped` 臂）：

1. 前缀：选 `gpu`（两张卡都用得上的假实现）、照水位判、有活意图、根环装满；下限 2 次。
2. 前台在等，取输入让一次：连着让路 1 次。
3. 用户挂着切到一组一张卡都用不上的设置（第 4 条「挂着切换到参与的卡一张都不剩的设置」那一种，用户动作）。
4. 下一次调用（前台等不等都扫）：判起跑之前换上这一组、判出停着，一趟开头过门（门开）单发删意图、发成。连着让路的次数还是 1。
5. 用户切回两张卡都点名的那一组（解除停下，用户动作）、建一个 inode；前台一直在等：推的读数是再让 1 次就不让（照条款清零之后应再让 2 次）。

对照臂（`publish_of_the_explicit_entry`）：第 3、4 步换成显式入口发成一次整理发布，经第 604 行清零。四句：分辨（对照臂清零、这一臂不清）；系统看得到（就在发成的那一刻、同一个会话上）；满足第 257 行「一批发成两个都清零」，前提是第一轮判决第 8 条那个口径；改法：第 377 行之前补一句 `compaction.clear_the_yields_in_a_row();`（推的，被攻过零轮），只修这一格。

### 签发那一支的空洞数、要搬数取现行那一版：坐实，补的用例罩不到

`an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version` 只断言两样：「"c0 取签发那次发布接着的那一版（现行那一版）的 txg"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:88`）与「"segs0 取现行那一版按记录数出的全空段数"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs:96`），窗口里只建一个 inode。第一轮那四条候选的去向：

| 候选 | 今天谁罩着 | 依据 |
|---|---|---|
| `segs0` 取票里那一版 | 另一条新用例（窗口里顺序写 40 个数据单元），变异行「放锁的整理：回锁签发的 segs0 取票里那一版的记录」 | 开工快照 `mutation-rows-of-the-batch.tsv` 第 12 行；红不红是主 agent 合入后验证量的，我没复核 |
| 取输入时就冻着那一批 | 另一条新用例，变异行「放锁的整理：回锁不判冻结（取输入之前就冻着的那一次照发这一批）」 | 同一份第 2 行；同上 |
| 空洞数取票里那一版的记录 | 没有 | 「let holes = holes_outside_the_region_in_segments_holding_user_data(」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:961`）下一行取现行那一版；修补的实现员报告第三节写了不收（转述：读码造不出敏感的取样点） |
| 要搬数在 R 的记录变了时仍取锁外扫出的能搬数 | 没有 | 「let units_to_move = if records_of_the_region_at_the_input == records_of_the_region_now {」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:947`）；实现员报告同一节写了不补（转述：要让空洞数正好落在两个要搬数之间） |

所以这条补的用例一条候选都不让红（`c0` 那一行之外），四条里两条另有新用例接住，两条仍罩不到。候选变异 1、2 写在副本的 `crates/mutations.tsv` 末尾，证红命令在「要主 agent 跑的命令」第 2 条，预期「没红」（推的）。要罩住，需要一段窗口历史让「R 外空洞数」落在锁外扫出的能搬数与现行那一版 R 里未释放记录数之间（要搬数那一格），或让某一段在两版记录口径上空与非空不同而空闲槽数跨过要搬数（空洞数那一格）；这两段历史我也没造出来，写进限度。

### 签发重判资格的「开过的段」「钉住与隔离」两个合取项：坐实，罩不到

资格四个合取项在「!allocator.cluster_segments().contains(&region.start_slot())」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:928`）到第 936 行。扫描拒过的段（第 933–935 行）与 R 里还有记录（第 936 行）各有一行变异与一条用例；开过的段（第 928 行）与钉住与隔离的槽（「.range(region.start_slot()..region.end_slot())」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:930`）那一项）没有。运行库那条用例 `after_the_chosen_segment_is_no_longer_eligible_at_the_relock_the_pass_calls_again`（`crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs:1363`）靠覆盖写把那一段的记录全放掉，走的是第 936 行，单删第 928 行或第 930 行那一项它照样绿（推的）。第一轮判决第 5 条说「资格重判那一处现有运行库用例罩得到」量的是整处不判（`if false &&`），不是单项。候选变异 3、4 在副本里，证红命令同在第 2 条，预期「没红」。要钉「钉住与隔离」，可照 `a_unit_quarantined_in_the_window_with_only_its_quarantine_mark_changed_is_pruned_at_the_relock_on_the_cpu_judge` 那一形在签发那一支上造：窗口里让 R 里一个槽进隔离（回锁第 823 行那一句把它并进常驻隔离集之后才重判资格，推的）。

## 三、修补第 2、3、4、6、7 条

### 第 7 条（取了输入没让也清零）：打中判据字面，病根多半在写回的措辞

清零落在问过调用方之后、写带宽与回退门两判之前：「compaction.break_the_yields_in_a_row_by_a_batch_not_yielding();」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:550`），注释直说「// （D26（后台整理与放置回收） 已定项 13 第 6 条），后面写带宽或回退门停下、回锁没发成都不改这一条。」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:549`），之后才是「if let Err(stop) = compaction.admit_the_next_batch_under_the_write_bandwidth_limit(」（`crates/singlefs-core/src/mounted_session/compaction_run.rs:551`）与第 565 行的回退门；走到第 550 行的三种情形是前台没有入口在等、这一批是破例、下限到了。真正取输入（读设置、拷指针）在这一判之后的 `take_the_input_of_one_compaction_batch` 里。

条款：「连着让路 N 次或让路累计 T 毫秒，哪一个先到下一批就不让，一批发成两个都清零；下限到了、取了输入而没让路的那一批也把两个计数清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）；取输入的定义是「会话锁里取输入（问让路、排空待发、读一次后台计算设置、现行那一版只拷指针）」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`）。

历史就是修补新写的那条用例 `a_take_of_the_input_without_yielding_breaks_the_yields_in_a_row_even_when_its_batch_publishes_nothing`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs:210`）：产品下限（8 次、5 秒），钟 0 毫秒前台在等、让一次；钟 1000 毫秒前台不等，这一次停在回退门上（「"前提：前台不等，这一批停在回退门上：{:?}",」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs:278`）），用例断言清零（「"这一次取输入没让路：连着让路断了，计数清零"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs:290`））；钟 5000 毫秒前台又在等，照样让。这一次调用下限没到（让过 1 次、离头一次 1 秒），也没读设置、没拷指针，第 257 行那一分句的两个限定都不满足：照字面计数不清，钟 5000 毫秒时离头一次已满 5 秒、这一批不让。用写带宽上限代替门关是同一形（推的）。

四句：

1. 分辨：代码与第 257 行字面在钟 5000 毫秒那一判上给出相反的让与不让。
2. 系统看得到：取没取输入、下限到没到，第 550 行那一刻都知道。
3. 满足的分句：第 257 行「下限到了、取了输入而没让路的那一批也把两个计数清零」——这段历史两个限定都不满足，代码照样清。
4. 这一格是不是判据自己写错：第一轮判决把交用户的那一问的乙写成「取了输入而没让路的那一批也清零」，用户原话是「取了输入没让也清零 (Recommended)」（`research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md:63`）；判决第 7 条要补的用例是「中间那一批停在回退门上没发，后来前台再等照样让」（`research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md:54`），第一轮攻方探针的中间那一次正是「否则先推空发布到回退门关、这一批停在门上不发」（`research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/src/probe_relock_gate_yield_floor_and_frozen_input.rs:446`），而且前台不等、下限没到。用户拍板时看的就是这一形；写回 kb 时多出的「下限到了、」，与把停在门上、没读设置的那一次叫作「取了输入」，都比用户定的那一形窄。所以多半是判据（kb 写回）写错了，代码与用例照的是用户定的那一形。这件事要主 agent 定：改 kb 第 257 行的措辞（例如「问过调用方而没让路的那一次也清零，不论之后写带宽、回退门停没停、回锁发没发成」），还是照字面改代码（把清零挪到取输入之后，并且只在下限到了时清）。我倾向前者，理由只有上面这三处原文，没量过。

显式入口与可写挂载续做不问调用方，只在一批发成时清零。会不会永远不清：后台入口每一次走到第 532 行之后，要么让（加一）、要么走到第 550 行清，计数不会卡住（推的）；显式入口取了输入、没发成（例如没有够格的段）的那一批算不算「取了输入而没让路」，条款没写，推不出。

### 第 2 条（回锁有冻结就丢批）：没打中

现在只看回锁那一刻有没有冻结（「if allocator.frozen_publish().is_some() {」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:724`）），条款第 256 行也只写回锁那一刻，没要求取输入那一刻就拒。运行库一拍先重发冻结的那一次、重发了就返回（「if self.resend_the_frozen_publish_if_any()? {」（`crates/singlefs-mount-runtime/src/lib.rs:839`）），冻着时不会每一拍都白算一批；同一拍里 `publish_when_due` 写错冻结之后接着取输入的那一批白算一次、回锁丢掉、记单独的停因，与条款一致。出错又冻着时出错先于一切、单发删意图被冻结拒、记成整理失败，这一格修补之前就是这样，不是修补改出的。

### 第 3 条（建缓冲等错误范围那一等读停标记）：没打中

那一等取「&budget.deadline_of_a_wait_of_the_batch_starting_now(&clock),」（`crates/singlefs-background-gpu/src/background_gpu/buffers_of_a_chunk_in_one_out_of_memory_scope.rs:73`）；期限的上限与停标记都取这张卡账上那一份，同一个函数也给交一段之后那几等用：「/// （交一段之后等它、冲掉待写那一等、交活与重交之前等前一次还在跑的那次都从这里取期限）。」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:127`），读的标记是「self.caller_stopping_seen_by_the_waits_of_the_batches」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:136`）。那一等放弃之后冲掉待写的那一等也从同一处取期限（「.deadline_of_a_wait_of_the_batch_starting_now(&clock);」（`crates/singlefs-background-gpu/src/background_gpu/gpu_in_use.rs:311`）），标记已置、同样当场放弃。没找到期限或标记取错的形状。

### 第 4 条（fsync 先 `try_lock`）：没打中（两个窗口都不是修补改出的）

- 试锁失败到登记之间：「let fsync_registered_before_waiting = match self.locked.try_lock() {」（`crates/singlefs-mount-runtime/src/lib.rs:683`）失败之后到「self.register_a_foreground_fsync_waiting_for_the_session_lock()」（`crates/singlefs-mount-runtime/src/lib.rs:689`）之间，维护线程可以放锁、再取锁、问调用方（第 503 行读 `fsync_waiters`），看到 0、发下一批。那一刻系统看不到这个 fsync（判别子观测不到），别的前台入口是同一写法（「match self.locked.try_lock() {」（`crates/singlefs-mount-runtime/src/lib.rs:648`）），设计第三轮改法第 9 条定的就是「`try_lock` 失败才登记」；不分辨 fsync 与别的入口，不归这一条修补。
- 放锁到减计数之间：`publish_pending_now` 先放锁、函数返回时才减（「let published = session.publish_pending_now(devices, now);」（`crates/singlefs-mount-runtime/src/lib.rs:1545`）的下一行放锁），次序是注释定的：「/// 取不到才登记在等、再等；交回登记（没等过是 `None`）与锁守卫。登记先于锁守卫声明：调用方照这个次序收，放锁在前、减计数在后。」（`crates/singlefs-mount-runtime/src/lib.rs:630`）。在锁上排着的维护线程拿到锁、减计数之前问调用方，会为一个已经做完的 fsync 让一次路、记进连着让路的次数（推的，没量过；窗口是放锁到函数返回的几条指令）。这是第一轮判决第 4 条那一类「没人等也让」，别的入口同样有；注释没写为什么非放锁在前不可，我没看出理由。提的改法：先减计数再放锁（推的，被攻过零轮）。

### 第 6 条（不切片理由、注释口径）：没打中

两份运行库测试 `#[test]` 与「// 不切片并行」注释行数各是 12/12、10/10（量的：`grep -c '^#\[test\]'` 与 `grep -c '^// 不切片并行'`）。清零处的口径改成「一批是一次整理发布」（第 603 行）；文件头第 10 行「一批 = 搬了单元的那一次发布」管的是「至多一批」的批数，第 606–607 行把两个口径分开写了，不矛盾。

## 四、合入之后修的两行变异；测试测的是条款还是实现

### 两行变异

开工快照 `mutation-rows-of-the-batch.tsv` 第 22、23 行，原文与替换文在今天主工作区的源码里的命中次数（量的：python 把 `\n` 换成换行后对被点名的文件 `str.count`，23 行全数都核了，原文都是 1 次）：

| 变异名 | 原文在今天的源码 | 替换文在今天的源码 | 点名的测试在拆后的树里 |
|---|---|---|---|
| 七·3 设置交给实现：切换生效的那一批开头交给实现的是换上之前的那一组 | 1 次：「let handed_to_the_implementation = settings.hand_the_settings_to_the_implementation();」（`crates/singlefs-core/src/background_compute/compute_of_this_mount.rs:357`） | 0 次 | `the_switch_requested_in_the_middle_of_a_batch_reaches_the_kernels_only_at_the_start_of_the_next_batch`，`crates/singlefs-background-gpu/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs` 里 1 处 |
| 七·3 GPU 代码轮一：一批三段怎么交的不记进那一批的记账 | 1 次（两行，第二行「implementation.take_the_submissions_of_the_batch();」（`crates/singlefs-core/src/mounted_session/compaction_batch.rs:312`）） | 0 次 | `the_submissions_on_the_gpus_of_each_batch_are_recorded_in_the_accounting_of_the_batch`，同一份文件里 1 处 |

替换文编不编得过：第 22 行把 `settings` 换成 `self.in_effect`（同一类型的字段）；第 23 行把赋值换成 `let _not_taken = &mut implementation;`，`gpu_work` 那一支照旧绑定而不用，只是告警（核心包只有 `#![deny(unsafe_code)]`，没有 `deny(warnings)`，量的：grep）。改回去会不会红：正文说锚点修补那 3 条在拆分之前的树上抓到，拆后的树上没见到这两行单独再证；证红命令在「要主 agent 跑的命令」第 3 条，推的：会红。

### 测试测的是条款还是实现

| 用例 | 断言的 | 判 |
|---|---|---|
| `a_take_of_the_input_without_yielding_breaks_the_yields_in_a_row_even_when_its_batch_publishes_nothing` | 停在回退门上、下限没到、前台不等的那一次也清零（第 290 行） | 测的是这一版实现（第三节第 7 条）：与 kb 第 257 行字面相反，与用户拍板那一形相同 |
| `a_compute_error_at_the_relock_after_the_rollback_gate_closed_in_the_window_keeps_the_live_intent` | 门关不发、意图留着：「"活意图与 R 排他留到门开"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs:273`） | 只钉了「门关就不发」半句；「留到门开」（门开之后补发）没有一条用例钉，第一节甲就落在这个空里 |
| `a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row` | 显式入口发成之后连着让路的次数清零（「"一批发成，连着让路的次数清零"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs:190`）） | 条款，但只看两个计数里的次数；累计时长那一个（「self.reading_when_the_yields_in_a_row_began = None;」（`crates/singlefs-core/src/compaction_plan.rs:761`））靠上一条用例的钟 5000 毫秒那一判间接钉着（两处共用同一个清零函数，推的） |
| `after_yielding_as_many_times_in_a_row_as_the_floor_allows_the_next_batch_does_not_yield` | 下限到了、取了输入（交到锁外）之后清零：「"下限到了、取了输入而没让路：连着让路断了，计数清零"」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs:113`） | 条款字面那一形 |
| `a_batch_taken_while_a_publish_is_already_frozen_is_dropped_at_the_relock_without_failing_the_compaction` | 取输入之前就冻着、回锁丢批、单独的停因、这一趟还在途 | 条款第 256 行 |
| `an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version` | `c0`、`segs0` | 条款第 256 行「各个量取现行那一版的」的两样，空洞数与要搬数没测（第二节） |
| `the_wait_for_the_out_of_memory_scope_gives_up_within_the_wait_limit_once_the_caller_is_stopping` | 标记经把手置上之后建缓冲那一等放弃、不等满 | 条款第 255 行「卸载时不等还在跑的提交」 |
| `an_fsync_taking_a_free_session_lock_is_not_registered_as_waiting` | 锁空着时 fsync 不登记 | 设计第三轮改法第 9 条 |

## 五、检查单九格

| 格 | 答 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 修补的 `crates.patch` 里以 `+` 起头、带 `unreachable!`、`todo!`、`assert`、`expect(`、`panic!` 或那三种错误成员收尾的行，产品源码里一行没有；命中的都在测试文件与 `mod tests` 里（第 199、205 行那两处 `expect` 在 `background_gpu.rs` 的 `mod tests` 那一段 hunk 里），量的：grep。`background_compaction_out_of_the_lock.rs` 第 670 行的 `unreachable!` 修补之前就在 |
| 类型与名字 | 成立 | 补丁里新加的行没有 `_ =>`（量的：grep 0 行）；`is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate` 与回锁再判门那一处都逐个写出 `RollbackGateAtTheRelock` 的两个成员（`compaction_batch.rs` 第 763–769、805–822 行）；新计数各是单独命名的 `u64`，没有共用一个裸整数；产品代码没新加 `expect` |
| 不兼容旧代码与旧格式 | 成立 | `txg_of_the_frozen_publish_at_the_input` 在 `crates/` 的产品源码里 0 处；剩下的两处是新用例里的局部变量（「let txg_of_the_frozen_publish_at_the_input = pending」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/publishes_frozen_and_drains_refused_at_the_relock.rs:221`）），核的是冻着的一直是同一次，不是旧判据的分支 |
| 不新建判定库 | 成立 | 池级 checker 那几判照目标头一段写的用一次性库：「池级 checker 那几判：录制流施加到内存镜像上」（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs:8`）；修补没加新的存放处 |
| 所有任务多机可用 | 成立 | 新用例与两份运行库测试里 `localhost`、`127.0.0.1`、`/tmp/`、`/home/`、`hostname` 都是 0 处（量的：grep） |
| 测试优先多线程 | 成立 | 12/12、10/10（第三节第 6 条）；验证包新用例各自一个池 |
| 带 --item 的命令与项 | 成立（只核了名字） | 这一批 23 行点名的测试名在 `crates/` 里各找得到 1 处函数定义（量的：grep）；`--test compaction_batch_computed_out_of_the_session_lock` 的目录里有 `main.rs`，目标名不变；`merge-and-verify.sh --list-items` 没跑 |
| 新崩溃流多罩了什么 | 成立（没有） | `crates.patch` 里 `+++` 行没有一份 `crash_enumeration` 文件（量的：grep 0） |
| 放的位置 | 成立 | 新用例在被测包自己的 `tests/`；`#[path]` 只有 `main.rs` 第 14、16 行指同一个包 `tests/` 下的 `common` 与 `common_background_gpu_compute`；GPU 包的替身在 `src/background_gpu/tests/` 下的 `#[cfg(test)]` 模块里 |

## 要主 agent 跑的命令

一行一条，串行交（三条都用同一棵副本，同一棵副本上并行交的证红会被副本上的证红锁拒）。副本 `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy` 在草稿目录的 `.deliverables` 里写成「交接」。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus`（运行目录 `…-r2-opus-run` 由包的那一层先删再建，只删工作根下以 `-run` 收尾的那一个）
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里三行 `SELF name=… failed=0`；三行 `RESULT name=…_summary`。我推的读数（判决要引的由主 agent 这一趟定）：`held_back_drop_after_the_gate_opens_summary histories=8 control_held_back=4 control_dropped=4 switched_held_back=4 switched_carried_on=4 switched_dropped=0`；`held_back_drop_while_the_exception_holds_at_the_relock_summary histories=2 same_state_at_the_relock=2 control_published=1 window_published=0`；`stopped_drop_at_the_trip_start_and_the_yields_in_a_row_summary histories=4 premises_held=4 control_not_cleared=0 drop_not_cleared=2`。推翻条件：`switched_carried_on=0` 且 `switched_dropped=4`（甲不成立）；`window_published=1`（乙不成立）；`drop_not_cleared=0`（第二节第一处不成立）。`exit` 非 0 而 `SELF` 行没出现的是编不过，日志交承接的同立场腿改
2. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy singlefs-verification '攻方候选：回锁签发前判的空洞数取票里那一版的记录' '攻方候选：回锁签发前判的要搬数记录变了也取锁外扫出的能搬数' '攻方候选：回锁签发重判资格不看这次挂载开过的段' '攻方候选：回锁签发重判资格不看钉住与隔离的槽'`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看哪几行：每条变异名那一行是「抓到」还是「没红」，汇总句「现判 N 条」。预期四条都「没红」、`exit=1`（坐实第二节两处缺口）；哪一条「抓到」，第二节那一处就改成「罩得到」并写点名的那条用例。基线红（退 2、「基线」那一行点名）不算判过
3. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy singlefs-background-gpu '七·3 设置交给实现：切换生效的那一批开头交给实现的是换上之前的那一组' '七·3 GPU 代码轮一：一批三段怎么交的不记进那一批的记账'`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看哪几行：`exit=0`，两行都「抓到」、基线绿（第四节「两行变异」改回去点名的测试红）

## 没打中的形状

- 第 1 条：门关压住之后同一趟里还会不会别处偷发删意图——回锁那一次停在 `BatchAbandonedAfterABackgroundComputeError`，`stop_holds_back_the_trip_without_ending_it` 判它不留这一趟，不再发（读码）；现行那一版没变时不再判门（取输入那一刻同一份根环判过门开）——没找到根环变而 txg 不变的路（冻结重发是原位同字节，推的）。
- 第 2 条：冻着时每一拍白算一批（运行库重发了就返回，第三节）；出错又冻着（修补之前就是这样）。
- 第 3 条：期限的上限取错卡、标记取错份、放弃之后冲掉待写那一等不放弃（第三节）。
- 第 4 条：两个窗口（第三节），都不分辨 fsync 与别的入口。
- 第 7 条：计数在某条路上永远不清——后台入口每次问过调用方要么加一要么清，造不出（推的）。
- 合入之后修的两行：锚点命中 0 或多次、替换文已在源码里（量的：都不是）。
- 用户决定的动作放开扫的范围：格一扫了切不切换（2）、下一次走哪个入口（2）、门开之后再多改几次（0、3），每段都建 inode 直到门开（至多 60 次）；格二只扫了底额在取输入前后两个时刻；格三扫了删意图那一次前台等不等（2）。门关推的空发布串数没扫，一律推到门关为止（至多 48 串）。

## 这条腿自己的限度

- 探针一次没跑过，所有读数是推的；编不过、或前提断言在某一格不成立（例如格三那一组设置换上之后不判出停着），由承接的腿改。
- 格二用只供测试的底额开关模拟「前台写满」，没造真的前台写满；判的谓词与产品同一个，造真历史要用一个填满全空段的池，没做。
- 第二节空洞数、要搬数两格，我同实现员一样没造出让候选变异红的窗口历史；资格的「开过的段」一格同样没造。
- 第 7 条判成「多半是判据写错」只凭三处原文的措辞对比，没有可跑的量；要不要改 kb 是主 agent 与用户的事。
- 第 4 条两个窗口没有并发模型，窗口宽度是推的。
- 只读了修补碰到的那几份文件与它们直接调的函数；拆分本身不在射程里。

## 没做什么

- 不判辩方那几格，不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，没起 cargo，没跑门禁；判决要引的数没复跑（命令在上一节）。
- 草稿目录里的仓副本 `copy`（64M，量的：`du -sh`）留给主 agent 跑上面三条，`.deliverables` 写成交接；用完由主 agent 删。
