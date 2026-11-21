# m4-gpu-wait-outside-the-lock-code-r1：判决（2026-11-19）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r1-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/`。这一题代码轮第一轮；它攻的实现照设计三轮的判决写成（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md`、`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md`、`research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md`）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-sonnet-output.md` | 「十四条都有实现，没有一条的主体与条款冲突」；钉得不全：签发那一支资格重判、空洞数、segs0 没有变异行，冻结丢批「与取输入时比」那一半没有变异行；fsync 取锁前无条件登记；建缓冲收显存不足错误域的那一等不读停标记；没资格换段的计数无人读；锁内一口气算完的入口仍是 pub |
| 攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-output.md` | 打中「回锁时出错那一支单发的删意图发布不过回退门」「让路下限的「连着」「累计」实现得比条款宽」「回锁丢批只认锁外期间新冻结的」；交判 CPU 路径卸载等锁外整批、「一批」两种口径；8 处新行为没有测试或变异行 |

同一题上一次客户端重启留下的旧进程派过一对腿，它的攻方腿没写报告就交回，读码报的四处线索记在 `records/2026-11-18-等GPU拆到锁外与前台等锁让路.md` 第 110 行，与这一轮攻方、正推两腿报的重合，一并在下面核。

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpu-wait-outside-the-lock-code-r1 --out ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-main/m4-gpu-wait-outside-the-lock-code-r1-tree --repository /home/fy5090/code/singlefs`，`type=succeeded exit=0`：快照里 32 份文件 32 份「✓」、0 份「✗」（工作区此刻那一份 sha256 与快照相同）。
- `python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-code-r1-sonnet-output.md research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-main/m4-gpu-wait-outside-the-lock-code-r1-tree --background research/prompts/_m4-gpu-wait-outside-the-lock-code-r1-background.md`：「✓ 核了 68 处引文、104 处裸行号，对上 67 处，没判 105 处（其中裸行号没有引文可比 100 处）」；没判的 5 处是快照树里没有的验证包 `src/` 文件（`history.rs`、`crash_library.rs`、`crash_state_enumeration.rs`、`allocator.rs`），下面用到的那几处在主工作区逐行现读。
- 攻方腿探针由主 agent 复跑：`task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh …`，`type=succeeded exit=0`，日志三行 `SELF … failed=0`，读数原样：
  - `RESULT name=relock_error_ignores_the_gate_summary histories=24 control_closed=2 control_closed_but_published=0 error_closed=2 error_closed_and_published=2`
  - `RESULT name=yield_floor_after_a_stop_summary histories=10 control_not_yielded=0 stopped_arm_not_yielded=2`
  - `RESULT name=frozen_before_the_input arm=frozen_before_the_input frozen_as_planned=1 batch_out_of_the_lock=1 frozen_at_the_relock=1 published=0 batches_dropped_while_frozen=0 trip_under_way=0 outcome=failed:Batch(Publish(PublishFrozenAfterAWriteFailureIsNotResentYet{…`（窗口里冻结那一臂 `batches_dropped_while_frozen=1 … outcome=stopped:BatchDroppedWhileAPublishIsFrozenAtTheRelock`）
- 正推腿五条命令主 agent 交了：变异表列项、钉第 5 到 9 条的 5 条验证包用例、2 条运行库让路用例、命名检查两条，都 `type=succeeded exit=0`；它提的 5 条候选变异证红在副本上照点名测试所在的包拆成两条交（结局见「三、改法」第 5 条）。
- 主 agent 读码现查：`crates/singlefs-core/src/mounted_session.rs` 第 6330–6345 行（出错那一支直接调 `drop_the_live_intent_on_a_stop`，之前没有判门）；第 6392–6394 行（`txg_of_the_frozen_publish_now.is_some() && txg_of_the_frozen_publish_now != txg_of_the_frozen_publish_at_the_input`）；`crates/singlefs-mount-runtime/src/lib.rs` 第 1474 行 `publish_pending_now` 取锁之前 `let _waiting = self.foreground_fsync_is_waiting();`；`crates/singlefs-background-gpu/src/background_gpu.rs` 第 895 行 `out_of_memory_captured_by` 调的 `wait_for_the_wgpu_future` 只收期限与时钟（第 4322 行签名）。

## 二、判决

条款原文在 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 4、5、6 条（第 255–257 行）。

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 回锁时出错那一支单发的删意图发布不过回退门 | **采，坐实** | 条款第 5 条「停着时那一次只删意图的发布（第 4 条）同样过这道门：门关就不发，意图与 `R` 排他留到门开」；出错那一支单发的正是第 4 条那一次删意图的发布。复跑读数 `error_closed=2 error_closed_and_published=2`、对照臂 `control_closed_but_published=0`。条款里「回锁里出错先于一切」管的是先停、先报，不免掉这道门，两句不冲突 |
| 2 | 回锁丢批只认锁外期间新冻结的 | **采，坐实** | 条款第 5 条「回锁时有冻结的发布就丢掉这一批、记单独的停因」，没有「锁外期间新冻结」的限定；复跑读数取输入前就冻着那一臂 `batches_dropped_while_frozen=0`、记成 `failed:…PublishFrozenAfterAWriteFailureIsNotResentYet`（整理失败），不是单独的停因 |
| 3 | GPU 包建缓冲收显存不足那一等不读停标记 | **采，读码坐实** | 条款第 4 条「卸载时不等还在跑的提交」与设计第三轮改法第 11、13 条；`background_gpu.rs` 第 895 行那一等只收期限与时钟，卸载时正卡在这里要等满 `background_gpu_wait_limit` |
| 4 | 前台 fsync 取锁之前无条件登记 | **采，读码坐实** | 设计第三轮改法第 9 条「计数在锁外、`try_lock` 失败才登记」；`lib.rs` 第 1474 行 fsync 先登记后取锁，会话锁空着时后台也让一次 |
| 5 | 新行为缺测试或缺变异行 | **采** | 攻方腿第四节表 8 行与正推腿报的签发那一支、冻结比对；逐行补，候选变异证红结局见「三、改法」第 5 条 |
| 6 | 运行库两份测试 7 条没写不切片理由 | **采** | `.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」：不能并行的在用例注释里写明为什么 |
| 7 | 让路下限只在一批发成时清零：整理一直发不成时让满之后不再让路 | **不采成代码缺陷，交用户** | 代码与条款第 6 条字面一致（「一批发成两个都清零」），也与设计第三轮改法第 9 条「N 自己计、不被别的事件清零，一批发成才清零」一致。复跑读数 `stopped_arm_not_yielded=2`、`control_not_yielded=0`：中间那一批停在回退门上没发的两段历史里，后来前台再等也不让路。这是条款的读法要不要改，不是代码偏离条款 |
| 8 | 「一批发成」两种口径（清零用任何整理发布，`mounted_session.rs` 第 5110 行注释说一批是搬了单元的那一次） | **不采成缺陷，注释改一致** | 条款第 6 条「一批发成」没限定搬没搬单元，签发也是一次整理发布；第 5110 行注释的说法与清零处不一致，改注释（改法第 6 条） |
| 9 | CPU 路径卸载会等锁外整批算完 | **不采** | 条款第 4 条「卸载时不等还在跑的提交」管的是 GPU 上的提交；CPU 路径锁外算的是本线程上的计算，没有提交，第三轮改法第 11 条候选乙也只写在 GPU 包等的那一处查停标记 |
| 10 | 锁内一口气算完的旧入口仍是 pub、验证包与实验二进制在用 | **不采** | 它是锁里现算的参照：新崩溃流用它比块序列（`the_in_lock_reference_with_an_overwrite_writes_the_blocks_of_the_registered_flow_with_a_write_while_the_intent_is_live`），不是兼容旧格式或旧版本的分支 |
| 11 | 取输入先排空待发、后问让路 | **不采** | 排空发的是用户改动不是整理批，条款第 6 条的让路只约束整理批 |

九格检查单：两条腿在「走不到的理由二次查实」「不新建判定库」「所有任务多机可用」「新崩溃流多罩了什么」「放的位置」五格都答成立；「测试优先多线程」照第 6 条补注释；「不兼容旧代码与旧格式」照第 10 条判成立；「类型与名字」命名检查两条都 `exit=0`；「带 --item 的命令与项」变异表列项 `exit=0`、合入后验证已按项点到。

## 三、改法

都在 `crates/` 改，合成一批交实现员，以主工作区此刻为基线：

1. 回锁时出错那一支单发删意图之前判回退门，照条款第 5 条「停着时那一次只删意图的发布」同一判：门关就不发，意图与 `R` 排他留到门开；破例条件成立时照发。补用例（照攻方腿探针 `relock_error_ignores_the_gate` 那一形：取输入时门开、窗口里抬 F 把门推关、锁外出错，回锁不发）与变异行。**量过**（复跑读数在「一、腿与核查」）。
2. 回锁时只要有冻结的发布就丢掉这一批、记 `BatchDroppedWhileAPublishIsFrozenAtTheRelock` 停因，删掉「与取输入时不同」那一半；补取输入前就冻着那一形的用例与变异行。**量过**。
3. GPU 包建缓冲收显存不足那一等（`background_gpu.rs` 第 895 行）读停标记，调用方在停时交 `GaveUpWaitingForTheCallerIsStopping`；补用例与变异行。读码定位，改法照同一包里等提交结果那一处的写法，不是新设计。
4. 前台 fsync 改成先 `try_lock`、取不到才登记，与别的前台入口同一写法；补用例（会话锁空着时 fsync 不登记）与变异行。
5. 补测试与变异行：攻方腿第四节表里的 8 行（回锁排空被拒、没资格换段计数、签发那一支「要搬的」取现行那一版、签发重判资格的几个合取项、三种回锁停因算这一趟没完、锁外读盘盘表不是挂载那一份、出错那一支与回退门、取输入前就冻着那一批），与正推腿报的签发那一支空洞数与 segs0。候选变异证红（主 agent 在副本上交 `bash scripts/mutation/prove-red.sh --copy <副本> singlefs-verification <4 个候选名>`，`type=command-failed exit=1`）：「✗ 点名 4 条：现判 4 条……跑了 4 条，抓到 0 条」——空洞数、segs0、要搬数取票里那一版的三条点的 `an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version` 都没红，取输入时就冻着那一条点的 `a_batch_relocked_while_a_publish_frozen_in_the_window_is_dropped_without_failing_the_compaction` 没红：这四处覆盖缺口坐实，补的用例要让这四行红。运行库那一条候选（签发那一支不重判资格）另交 `prove-red.sh --copy <副本> singlefs-mount-runtime …`，`type=succeeded exit=0`，「候选：回锁签发那一支不重判资格（版本变了也照签）	抓到」：资格重判那一处现有运行库用例罩得到，收进变异表即可。
7. 让路下限：一批发成清零之外，下限到了、取了输入而没让路的那一批也把 N 与 T 两个计数清零（用户 2026-11-19 定，见「交用户的」）；补用例（照攻方腿探针 `yield_floor_after_a_stop` 那一形：中间那一批停在回退门上没发，后来前台再等照样让）与变异行。
6. 运行库两份测试里 7 条用例补不切片的理由注释；`mounted_session.rs` 第 5110 行注释改成与清零处同一口径（一批 = 一次整理发布）。

## 交用户的

| 问题 | 依据 | 候选 |
|---|---|---|
| 让路下限：整理一直发不成（例如回退门关着）时，让满一次之后这次挂载里不再让路，要不要改 | 判决第 7 条；复跑读数 `stopped_arm_not_yielded=2`、`control_not_yielded=0`；条款第 6 条「一批发成两个都清零」 | 甲：照现在的条款，只有一批发成才清零；乙：改条款，取了输入而没让路的那一批也清零（「连着」被它打断） |

用户 2026-11-19 弹窗定（原话）：「取了输入没让也清零 (Recommended)」。改法第 7 条照这一句写，D26（后台整理与放置回收） 已定项 13 第 6 条照这一句写回。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/src/background_gpu_compute.rs：判了停标记与 `GaveUpWaitingForTheCallerIsStopping` 这一支（条款第 4 条），成立
- crates/singlefs-background-gpu/src/background_gpu.rs：判了建缓冲收显存不足那一等不读停标记（判决第 3 条，采）
- crates/singlefs-background-gpu/src/gpus_of_this_host.rs：判了切换校验读卡清单快照、不取 GPU 实现那把锁（第三轮改法第 11 条），成立
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：判了切片等卡那几处读停标记，成立
- crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs：判了量代价那一等读停标记，成立
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了等提交结果那一等读停标记，成立
- crates/singlefs-core/src/background_compute.rs：判了锁外算那一段先放 GPU 实现那把锁、中毒交回错误，成立
- crates/singlefs-core/src/compaction_plan.rs：判了至多一批在锁外的断言与让路下限计数（判决第 7 条交用户）
- crates/singlefs-core/src/mounted_session.rs：判了三段、回锁重排剔单元、出错那一支不过门（判决第 1 条，采）、冻结丢批的条件（判决第 2 条，采）、第 5110 行注释口径（判决第 8 条）
- crates/singlefs-core/src/region_scan.rs：判了锁外读盘回锁里读那一段，成立
- crates/singlefs-mount-runtime/src/lib.rs：判了前台入口让路的登记（判决第 4 条 fsync，采）与锁外那一段期间维护线程照发
- crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs：判了它测的是不是条款，缺不切片理由的几条（判决第 6 条）
- crates/singlefs-mount-runtime/tests/mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock.rs：判了让路用例覆盖的入口，缺 fsync 空锁那一形（判决第 4 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs：判了回锁、签发、冻结、让路下限的用例覆盖（判决第 5 条补）
- crates/singlefs-verification/tests/crash_enumeration_compaction_batch_computed_before_a_foreground_change_and_pruned_at_the_relock_stream.rs：判了新流比已登记流多罩的序列与前缀相同的钉法，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_deadlist_reclamation_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_switching_backends_between_batches.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- crates/singlefs-verification/tests/crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs：判了 CPU 两段版与 GPU 版录同一条流，成立
- research/prompts/_m4-gpu-wait-outside-the-lock-code-r1-body.md：判了检查单九格的答案，两腿与主 agent 复核
- research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md：判了改法十四条与实现逐条对得上，第 9 条的让路清零读法交用户
- research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/mutation-rows-appended-by-the-batch.tsv：判了这一批追加的变异行锚点与点名测试，签发那一支与冻结比对缺行（判决第 5 条）
- research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/mutation-rows-deleted-after-the-merge.txt：判了删掉的那一行是三样全同的重复行，被删行点的用例另有行点着
- research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/mutation-rows-replaced-after-the-merge.tsv：判了第 5750 行改后的替换文编得过、证红抓到
- research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/mutation-rows-replaced-by-the-batch.tsv：判了换掉的行锚点在今天的源码上恰好命中一次

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-19 不受影响：判决第 1 到 6 条是代码照条款改；第 7 条的让路清零读法交用户，用户定了再回看这一条 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 支撑 | 2026-11-19 不受影响：GPU 包等卡读停标记照射程实现，判决第 3 条补一处漏读的等 |
