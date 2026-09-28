# 调查：门禁 74 号在 B2b 副本上的三条红（随机历史二进制）

调查员，2026-09-27。只读主工作区；复现、加打印、换文件都在副本 `/tmp/claude-1000/investigate-gate74-reds/repo/` 里做。

## 一、结论

1. **第 1、2 条是真违例，不是 checker 误报。** 28 个种子（逐个看过，不是按签名归类）和第 1 条的固定历史走的是同一个机理，而且就是 C554 那一形：
   - 随机历史的「崩溃恢复抛弃根」这一步，是让可写挂载期间最新根的根槽和它那次发布点名的单元读回全 0 造出来的。
   - 这次挂载重建分配器时，影子账只查读得出的根（`crates/singlefs-core/src/mount.rs:985` 调 `readable_roots_with_ring_slots`，`recovery.rs:976`），被抛弃的根不在它眼里；于是那条根引用的单元一个都没隔离。
   - 这些单元在同一个会话里被重新发了出去：可能在这次挂载自己的写行 / 暖机里（`mount.rs:2262` `dry_run_of_the_publishes_after_acquisition`，经 `establish_instance` `mount.rs:2868`），也可能在它之后、下一次挂载之前的覆盖写里（`transaction.rs:1685` `allocate_placement_for_role`）。
   - 判红那一刻，被抛弃的根 28/28 都在根环里，txg 都严格高于 F_生效；它不在回退候选集里，只因为实例表判它被抛弃。
   - I-7.4 条款原文要求这种根引用的块「在离开根环之前」不许重新分配，没有 F 这一道界；checker 被抛弃根那一半（`walk.rs:5388`）也不看 F，与条款一致。
2. **第 3 条来自实审 A4 改的 `admission.rs`（ckpt_cost 按盘分路计）。**
   - 副本里其余文件都不动、只换 `admission.rs`：换成 A4 那一版（sha256 `08c40771…`），红，`left: 2 right: 0`；用稍后的主工作区那一版（A4b 改到一半，`2eca8b86…`），绿。
   - 那两次落点拒绝发生时，每块盘空闲 100 槽，隔离 0、扣住（抬 F 回收、F 还没生效）0。落点取不到，是因为对齐的空闲槽对全在这次挂载开的两个聚簇段 [50304, 50368] 里：段外 0 对，段内 49 / 48 对。用户数据落点不许落进开放聚簇段（`allocator.rs:587`，D3 已定项 10 ②），准入式子只按字节数算、不管这一条。
   - A4 报告推测的「抬 F 回收的槽扣住」在这两次里不成立（扣住 = 0）。
3. B2b 副本上的三条红，在我的副本上复现了第 1、2 条，数一模一样；第 3 条没复现，原因就是上一条说的 `admission.rs` 版本不同。

## 二、取副本时的快照

主工作区 HEAD `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`。副本取于 2026-09-27，用的是 `rsync -a --exclude target --exclude .git`。取之前和取之后各算一次 7 份文件的 sha256，两次相同：

```
2eca8b864f3f6695763ec3cc553c0d5429c91355ee3b3292c4022f38e29334c6  admission.rs
6290e9d04f50e707b2df10cb665472f2ec38807a68cb4dab7ca46a314b3f5fc5  allocator.rs
7aed0dbb6363f764569a4c013e96eb4bf27103bd27b5d5c8b6b433b4c5b26269  mount.rs
5977d23ee28251816ae9f9d4d69a5639ecfcf1fc05a792ae1f4ced1294325f68  transaction.rs
49a1a1abf9f7aaf3297c7753ff425fbe3c687dbeb2e467d5fb12ad51b518a40b  recovery.rs
40467763deb50536dfff2ea65909de791c79a52b293e60db8a65bcf6bae2f711  mounted_read.rs
842f766a907e5e414fa73900fdc026809dc32068af4476bba69527ad6a3fe0f7  make_filesystem.rs
```

- 副本能编过：`cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_random_history --no-run`， `Finished`。
- 副本与 HEAD 的差别：
  - `crates/` 下有 88 份跟踪文件的内容不同，逐份列在 `copy-vs-head-modified.txt`；另有 28 份未跟踪文件，列在 `copy-untracked.txt`。
  - 其中 `singlefs-core/src` 与 `singlefs-checker/src` 里不同的是：checker 的 image.rs、lib.rs、position_addressed.rs、walk.rs；core 的 admission.rs、allocation_record_tree.rs、allocator.rs、code_two_tree.rs、extent_tree.rs、instance_table.rs、journal.rs、lib.rs、make_filesystem.rs、mount.rs、mounted_read.rs、recovery.rs、rollback_witness.rs、root_record.rs、system_configuration.rs、transaction.rs、write_request_split.rs。
  - core 另有一份未跟踪文件 `mounted_session.rs`。
- 之后再看主工作区：`admission.rs` 已变成 `aa0201f8…`（A4b 还在改）；mount、allocator、transaction、recovery 四份的 sha 与快照相同。
- 下文引的 mount.rs、allocator.rs、transaction.rs、recovery.rs、walk.rs、history.rs 行号，都是那一次在主工作区现取的。其中 walk.rs、history.rs、allocator.rs、mounted_session.rs 与副本逐行比过：副本比主工作区只多出我加的诊断行（`diff` 的 `<` 行 0 条）。
- 负载：开跑前 `ps` 看到别的会话在跑一条 `cargo test --offline -p singlefs-harness --test second_transaction_supplement_three_bad_disk_input`，没有性能测量进程。我用的是自己的 target 目录，没等锁。
- 线程上限：开头按 6；Z 收到主 agent 的消息改成 5，之后起的 cargo 都经 `capped.sh 5`。内存一律经 `run-with-memory-cap.sh 12G`。

## 三、原样复现（第 1 次跑，副本未加任何打印）

命令（在副本根目录，`CARGO_TARGET_DIR=/tmp/claude-1000/investigate-gate74-reds/target`）：

```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_random_history
```

退出码 101，日志 `run1.log`（638 行）。原样行：

```
test crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... FAILED
test unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval ... ok
test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 60.87s
thread 'crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43' (1245530) panicked at crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:1127:13:
要以「已知红」第 0 条收尾：NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(3), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的")], panic: None, newest_ring_root_txg: Some(8), root_ring_slot_count: Some(24), harness_judgement: None, model_disagreement: None, raised_floor_lands_only_on_aband
历史 96 段：跑完 67、以已知红收尾 {0: 1}、新发现 28；根环转过一圈的 62 段；最高 txg 44；一版里最多 746 条分配记录
新发现 CheckerViolations { invariants: ["I-7.4"] }：第一个种子 7463871032432355115（同签名的种子 [7463871032432355115, 7463871032432355119, 7463871032432355122, 7463871032432355124, 7463871032432355128, 7463871032432355129, 7463871032432355131, 7463871032432355134, 7463871032432355136, 7463871032432355137, 7463871032432355142, 7463871032432355144, 7463871032432355148, 7463871032432355165, 7463871032432355167, 7463871032432355168, 7463871032432355171, 7463871032432355178, 7463871032432355183, 7463871032432355184, 7463871032432355189, 7463871032432355192, 7463871032432355194, 7463871032432355197, 7463871032432355198, 7463871032432355199, 7463871032432355203, 7463871032432355205]），在 Operation(4)（Some(PublishOverwrite)）之后
  I-7.4：被抛弃的根（实例 1、txg 4）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的
```

- 第 1、2 条复现了，种子、txg、槽号、「跑完 67、已知红 {0: 1}、新发现 28」都与 B2b 报告第六节一样。
- 第 3 条在这一版上是 `ok`，没复现。与 B2b 现场的差别见第六节：`admission.rs` 从 A4 那一版（`08c40771…`）变成了 A4b 改到一半的那一版（`2eca8b86…`）。B2b 副本核心文件的 sha 没有留档，只能拿 A4 报告与 B2b 日志里的数来对。

注：上面 `要以「已知红」…` 那一行在报告里截到 700 字符，全文见 `run1.log` 第 536 行。

## 四、第 1、2 条：机理

### 4.1 第 1 条固定历史逐步看（`red1-diag2.log`）

- 副本里加的诊断打印都只在环境变量下出声，改动全文在 `diag-instrumentation.patch`：
  - checker 的 `judge_blocks_referenced_by_abandoned_roots` 判红时，打出根环里每条根 (区域, 槽, 实例, txg, F)、实例表行、两块盘系统配置的 F；
  - 执行器每一步打出结局，「崩溃恢复抛弃根」那一步另打出被藏起来的槽；
  - 分配器的 `mark_allocated`、`isolate`、`mark_released`、`mark_reclaimed`、`clear_isolation_of_slot` 碰到被盯的槽时打栈。
- 命令：

```
SINGLEFS_DIAG=1 SINGLEFS_DIAG_WATCH_SLOT=50184 RUST_BACKTRACE=1 nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_random_history -- --exact crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 --nocapture
```

原样（去掉栈帧，DIAG-STEP 行截到 200 字符）：

```
DIAG-STEP 0 PublishOverwrite current(inst,txg,F)=Some((1, 4, 0)) outcome=Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 
DIAG-ALLOC mark_allocated dev=0 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=1 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=0 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=1 slot=50184 span=2
DIAG-STEP 1 PublishOverwrite current(inst,txg,F)=Some((1, 5, 0)) outcome=Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 
DIAG-CRASH-RECOVERY abandons newest root (inst 1, txg 5, F 0); hidden (dev, slot, slots) = [(0, 448, 0), (0, 50184, 1), (1, 50184, 1), (0, 50265, 1), (1, 50265, 1), (0, 50266, 1), (1, 50266, 1), (0, 5
DIAG-STEP 2 CrashRecoveryAbandoningTheNewestRoot current(inst,txg,F)=Some((2, 7, 0)) outcome=Applied(Mounted { instance: InstanceGeneration(2), publishes: 2, allocation_records_compared: Compared { pu
DIAG-ALLOC mark_allocated dev=0 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=1 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=0 slot=50184 span=2
DIAG-ALLOC mark_allocated dev=1 slot=50184 span=2
DIAG-STEP 3 PublishOverwrite current(inst,txg,F)=Some((2, 8, 0)) outcome=Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 
DIAG-I74 roots(region,slot,inst,txg,F)=[(0, 0, 0, 0, 0), (0, 1, 1, 3, 0), (0, 2, 2, 6, 0), (1, 0, 1, 1, 0), (1, 1, 1, 4, 0), (1, 2, 2, 7, 0), (2, 0, 1, 2, 0), (2, 1, 1, 5, 0), (2, 2, 2, 8, 0)] newest=
DIAG-I74 RED abandoned root inst 1 txg 5 F 0
```

- 读法：DIAG-STEP 在这一步做完之后才打，所以两行 DIAG-STEP 之间的 DIAG-ALLOC 属于后一步。每一步都是两次 mark_allocated，栈分别在 `settle_the_allocation_record_tree` 和 `publish_admitted`，都在 `prepare_the_version_publish` 之下。
- 槽 50184 第一次被分配是第 1 步：C = (1, 5) 那次覆盖写发的数据单元。
- 第 2 步是崩溃恢复：把 C 的根槽和 50184 等单元藏起来，择根落到 B = (1, 4)，实例 2 写行（txg 6）、暖机（txg 7）。这次挂载里 50184 没有一次 `isolate`。
- 第 3 步 (2, 8) 又把 50184 发出去，checker 当场红。
- 判红那一刻：
  - 根环里 C = (1, 5) 还在（区域 2 槽 1）；
  - 实例表只有一行 (1, T = 4)，5 > 4，所以 C 被判抛弃；
  - 每条根带的 F 都是 0，两块盘系统配置的 F 也是 0，F_生效 = max(0, 0) = 0；
  - C 的 txg 5 **高于** F_生效。C 出候选集只因为实例表判它被抛弃，与 F 无关。

### 4.2 28 个种子逐个看（`seeds28.log`、`seeds28-analysis.txt`、`analyze.py`）

- 做法：用副本里的诊断测试 `diag_gate74::diag_seeds`，对报告里点名的 28 个种子逐个 `generate_history_with_weights(seed, 30, BROAD)` 再 `execute_history`，与快档同一个生成器、同一个执行器。
- 命令：`DIAG_SEEDS=<28 个种子> SINGLEFS_DIAG=1 … cargo test … --test diag_gate74 -- --exact diag_seeds --nocapture`。那次的种子串从日志里抓了两遍，每个种子各跑了两次；两次结果逐项相同，分析时去了重。
- `analyze.py` 对每个种子取这几样：
  - 判红的被抛弃根 (实例, txg) 与槽；
  - 抛弃它的是第几步「崩溃恢复抛弃根」；
  - 那个槽在不在那一步藏起来的范围里；
  - 从那一步到判红那一步之间有没有别的挂载；
  - 判红时的 F。
- 原样：

```
  red root (inst 1, txg 4) slot dev0:50182; abandoned by crash-recovery at step 3; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 11) slot dev0:50372; abandoned by crash-recovery at step 7; slot in hidden set: True; steps between (exclusive..red]: 0 kinds=[]; mounts in between=[]
  red root (inst 3, txg 26) slot dev0:50200; abandoned by crash-recovery at step 22; slot in hidden set: True; steps between (exclusive..red]: 6 kinds=['PublishFirstFile', 'PublishOverwrite', 'RollBackWhileMounted']; mounts in between=[]
  red root (inst 4, txg 22) slot dev0:50192; abandoned by crash-recovery at step 17; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 6) slot dev0:50186; abandoned by crash-recovery at step 9; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 3) slot dev0:50240; abandoned by crash-recovery at step 3; slot in hidden set: True; steps between (exclusive..red]: 0 kinds=[]; mounts in between=[]
  red root (inst 1, txg 17) slot dev0:50202; abandoned by crash-recovery at step 19; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 3, txg 14) slot dev0:50188; abandoned by crash-recovery at step 9; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 4) slot dev0:50182; abandoned by crash-recovery at step 2; slot in hidden set: True; steps between (exclusive..red]: 4 kinds=['PublishFirstFile', 'PublishOverwrite', 'RaiseRollbackFloor']; mounts in between=[]
  red root (inst 2, txg 6) slot dev0:50182; abandoned by crash-recovery at step 11; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 3, txg 26) slot dev0:50176; abandoned by crash-recovery at step 25; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 3) slot dev0:50240; abandoned by crash-recovery at step 2; slot in hidden set: True; steps between (exclusive..red]: 0 kinds=[]; mounts in between=[]
  red root (inst 1, txg 9) slot dev0:50192; abandoned by crash-recovery at step 8; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 18) slot dev0:50202; abandoned by crash-recovery at step 22; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 16) slot dev0:50192; abandoned by crash-recovery at step 11; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 3, txg 15) slot dev0:50194; abandoned by crash-recovery at step 17; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 4) slot dev0:50182; abandoned by crash-recovery at step 2; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 3, txg 18) slot dev0:50190; abandoned by crash-recovery at step 19; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 13) slot dev0:50190; abandoned by crash-recovery at step 8; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite', 'RollBackWhileMounted']; mounts in between=[]
  red root (inst 1, txg 4) slot dev0:50182; abandoned by crash-recovery at step 2; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 19) slot dev0:50202; abandoned by crash-recovery at step 20; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite', 'RaiseRollbackFloor']; mounts in between=[]
  red root (inst 1, txg 6) slot dev0:50186; abandoned by crash-recovery at step 5; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 1, txg 8) slot dev0:50312; abandoned by crash-recovery at step 5; slot in hidden set: True; steps between (exclusive..red]: 0 kinds=[]; mounts in between=[]
  red root (inst 1, txg 4) slot dev0:50182; abandoned by crash-recovery at step 2; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 4, txg 13) slot dev0:50188; abandoned by crash-recovery at step 14; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite', 'RollBackWhileMounted']; mounts in between=[]
  red root (inst 2, txg 21) slot dev0:50190; abandoned by crash-recovery at step 22; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 2, txg 8) slot dev0:50186; abandoned by crash-recovery at step 8; slot in hidden set: True; steps between (exclusive..red]: 1 kinds=['PublishOverwrite']; mounts in between=[]
  red root (inst 3, txg 18) slot dev0:50200; abandoned by crash-recovery at step 22; slot in hidden set: True; steps between (exclusive..red]: 2 kinds=['PublishOverwrite']; mounts in between=[]
```

- 28/28 都是这几样同时成立：
  - 判红的根是某一步「崩溃恢复抛弃根」抛弃的那条最新根；
  - 被复用的槽在那一步藏起来的范围里；
  - 从那一步到判红那一步之间一次挂载都没有，复用发生在那次看不见它的挂载所开的会话里；
  - 其中 4 个（…119、…129、…144、…194）在崩溃恢复那一步当场就红，复用发生在那次挂载自己的写行 / 暖机里。
- 判红时被抛弃根的 txg 与 F_生效比（F_生效 = max(最新根带的 F, 两块盘系统配置里的 F)，数自同一次日志）：

```
7463871032432355115 被抛弃根 txg 4 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355119 被抛弃根 txg 11 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355122 被抛弃根 txg 26 > F_生效 2（最新根 F 2，系统配置 F [2, 2]）
7463871032432355124 被抛弃根 txg 22 > F_生效 3（最新根 F 3，系统配置 F [3, 3]）
7463871032432355128 被抛弃根 txg 6 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355129 被抛弃根 txg 3 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355131 被抛弃根 txg 17 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355134 被抛弃根 txg 14 > F_生效 1（最新根 F 1，系统配置 F [1, 1]）
7463871032432355136 被抛弃根 txg 4 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355137 被抛弃根 txg 6 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355142 被抛弃根 txg 26 > F_生效 10（最新根 F 10，系统配置 F [10, 10]）
7463871032432355144 被抛弃根 txg 3 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355148 被抛弃根 txg 9 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355165 被抛弃根 txg 18 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355167 被抛弃根 txg 16 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355168 被抛弃根 txg 15 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355171 被抛弃根 txg 4 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355178 被抛弃根 txg 18 > F_生效 1（最新根 F 1，系统配置 F [1, 1]）
7463871032432355183 被抛弃根 txg 13 > F_生效 3（最新根 F 3，系统配置 F [3, 3]）
7463871032432355184 被抛弃根 txg 4 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355189 被抛弃根 txg 19 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355192 被抛弃根 txg 6 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355194 被抛弃根 txg 8 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355197 被抛弃根 txg 4 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355198 被抛弃根 txg 13 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355199 被抛弃根 txg 21 > F_生效 6（最新根 F 6，系统配置 F [6, 6]）
7463871032432355203 被抛弃根 txg 8 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
7463871032432355205 被抛弃根 txg 18 > F_生效 0（最新根 F 0，系统配置 F [0, 0]）
高于 28 相等 0 低于 0
```

### 4.3 条款原文与 checker 被抛弃根那一半

`.claude/kb/invariants.md` 第 54 行（整行）：

| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

`crates/singlefs-checker/src/walk.rs` 第 5388–5391 行（主工作区）：

```
        let abandoned = instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        });
        if index == newest_index || !abandoned {
```

- 这一半只拿最新根的实例表判抛弃，再按「在根环里」取对象，不看 F。
- 条款写的是「它们引用的块在离开根环之前同样不许重新分配、不许抹头」，同样没有 F 这一道界。所以不是「条款说要照候选集判、代码没照」。
- 这一问的前提（被抛弃根已落在候选集下界之外）在 28/28 里都不成立，见 4.2 那张表，这一格这一轮没有观测。
- 顺带：D23 已定项 14 原文（`.claude/kb/decisions/23-journal的角色与格式.md` 第 385 行）说影子账「多查环里每一个**可读**根的账」。随机历史造的根在那次挂载里恰好读不出，所以只看这一句，写者照字面做了；但用户 2026-09-26 在 C554 行里定了「暂时读不出的最新根不许当成被抛弃」（`.claude/kb/checks-owed.md` 第 475 行第 5 列）。

### 4.4 写者哪一段放掉了被抛弃根的单元

以下行号都在主工作区，sha 与快照相同：

- `crates/singlefs-core/src/mount.rs:958` `rebuilt_allocator`：
  - 第 985 行取根环用的是 `readable_roots_with_ring_slots`（`crates/singlefs-core/src/recovery.rs:976`），只收读得出、自证过的根；
  - 被藏起来的最新根不在 `roots` 里，第 827 行 `isolate_slots_referenced_only_by_abandoned_roots` 第 855 行那个循环根本看不到它，`abandoned_roots_unreadable` 也不会为它加一。
- 于是 C 引用的单元在所选那一版（B）的账里是空闲槽，照常被发出去：
  - 挂载自己的写行 / 暖机：`mount.rs:2262` `dry_run_of_the_publishes_after_acquisition` ← `mount.rs:2868` `establish_instance`，见 `watch-7463871032432355129.log`、`watch-7463871032432355119.log` 的栈，栈里有 `DeviceReadingZerosOverHiddenRanges`；
  - 之后的覆盖写：`transaction.rs:1685` `allocate_placement_for_role` → `allocator.rs:579` `lowest_user_data_slot`，见 `red1-diag2.log` 的栈。
- 不是抬 F、也不是回收放掉的：
  - 28 个种子里，被复用的槽在复用之前一次 `mark_released` / `mark_reclaimed` 都没有。这一点逐槽盯过的是第 1 条、…129、…119；其余种子只由「复用那一步之前没有挂载」推。
  - 带 RaiseRollbackFloor 的 …136、…189，那几次抬 F 都是 `Refused { member: "MountError::RollbackFloorAboveCeiling" }`，没有做成的抬 F。
- 造这一形的执行器：`crates/singlefs-harness/src/history.rs:3249` `apply_crash_recovery_abandoning_the_newest_root`，第 3300 行起 `DeviceReadingZerosOverHiddenRanges` 在挂载期间把那几段读成 0，第 3311 行挂载，挂完换回原盘。

### 4.5 和 C554 那一形是不是同一个机理：是

- formatted_pool 那条用例：`second_transaction_step_three_formatted_pool.rs` 的 `crash_recovery_abandoning_the_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`。
- 在副本里盯槽 50240 跑它（`formatted-pool-watch-2eca8b.log`，`admission.rs` = `2eca8b86…`）：
  - 50240 在 M3 挂载的 `dry_run_of_the_publishes_after_acquisition` / `establish_instance` 里被 `mark_allocated`，栈里 `dry_run_of_the_publishes_after_acquisition` 出现 14 次；
  - checker 红在 (2, 3)、(2, 4) 两条被抛弃根上，F 都是 0。
- 走的是同一段代码：`rebuilt_allocator` 只看读得出的根，看不见的被抛弃根，它的单元在那次挂载的写行里被发出去。
- 随机历史种子 …129 / …144 被复用的正好也是 50240（第 3 步崩溃恢复、当场红）。
- 差别只在怎么让根读不出：C554 用例把根槽清零，随机历史把读回改成 0。

### 4.6 排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| checker 误报：那个单元其实没被动过 | 第 1 条：`red1-diag2.log` 里 50184 在 txg 5（C）与 txg 8（实例 2）各被 `mark_allocated` 一次，中间一次 `isolate` / `mark_released` / `mark_reclaimed` 都没有。这条观测来自写者分配器，与 checker 读盘是两条不共用代码的路 |
| 被抛弃的根已落在 F_生效 之下，本不该保护 | 28/28 txg 高于 F_生效（4.2 那张表），第 1 条是 5 > 0 |
| 抬 F / 回收放掉了影子账里的单元 | 复用之前一次成功的抬 F、一次 `mark_reclaimed` 都没有（4.4）；插一次普通挂载之后，同一批操作不再红（4.7） |
| 28 条是几种机理混在一起 | 28/28 满足同一组条件（4.2）；抽了 5 个不同种子（…115、…122、…183、…189、…142）逐个做反事实，结局相同（4.7）；另外 2 个（…129、…119）逐槽看了栈 |
| 与 `admission.rs` 版本有关 | 第 1、2 条的全部观测都在 `2eca8b86…` 上做；B2b 的现场是 A4 那一版，同样的种子、槽号、「新发现 28」，数一样 |

### 4.7 反推：假如「挂载看不见被抛弃根」不是原因，就该看到什么

- 预测：不是这个原因的话，在崩溃恢复那一步之后、复用之前插一次普通的 `CloseAndMountWritable`（这时被抛弃的根读得出），I-7.4 照红。
- 做法：副本诊断测试 `diag_counterfactual_remount_before_overwrite`（第 1 条那段固定历史的前 3 步 + 4 次覆盖写，插 / 不插），以及 `diag_seed_counterfactual`（按种子生成，在崩溃恢复那一步之后插一步）。
- 原样（`cf1.log`，截到 300 字符）：

```
DIAG-CF with_remount=false: outcomes=4 ending=NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(3), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 1、txg 5）引用的单元已被重
DIAG-ALLOC isolate dev=0 slot=50184 span=2
DIAG-ALLOC isolate dev=1 slot=50184 span=2
DIAG-CF with_remount=true: outcomes=8 ending=Completed
```

原样（`cf-seeds.log`，截到 300 字符）：

```
DIAG-CF-END seed 7463871032432355115 insert=false outcomes=5: NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(4), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 1、txg 4）引用的
DIAG-CF-END seed 7463871032432355115 insert=true outcomes=18: KnownRed { form: 0, observation: FailureObservation { position: Operation(17), operation_kind: Some(RaiseRollbackFloor), violations: [("I-3.1", "盘 0：记账的已分配 Some(3506176)，遍历全部有效根得到 3342336（其中隔离
DIAG-CF-END seed 7463871032432355122 insert=false outcomes=29: NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(28), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 3、txg 26）引用
DIAG-CF-END seed 7463871032432355122 insert=true outcomes=31: Completed
DIAG-CF-END seed 7463871032432355183 insert=false outcomes=11: NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(10), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 2、txg 13）引用
DIAG-CF-END seed 7463871032432355183 insert=true outcomes=31: Completed
DIAG-CF-END seed 7463871032432355189 insert=false outcomes=23: NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(22), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 2、txg 19）引用
DIAG-CF-END seed 7463871032432355189 insert=true outcomes=31: Completed
DIAG-CF-END seed 7463871032432355142 insert=false outcomes=28: NewFinding { signature: CheckerViolations { invariants: ["I-7.4"] }, observation: FailureObservation { position: Operation(27), operation_kind: Some(PublishOverwrite), violations: [("I-7.4", "被抛弃的根（实例 3、txg 26）引用
DIAG-CF-END seed 7463871032432355142 insert=true outcomes=31: Completed
```

- 插了那次挂载之后：新挂载的影子账在两块盘上都把 50184 `isolate` 了，历史跑完。
- 5 个种子里，4 个跑完；…115 走到第 17 步，以已知红第 0 条（抬 F 那一形）收尾，I-7.4 那条新发现没了。预测的现象没出现，原因成立。
- 这一次同时是「推翻条件在副本里造得出」的那次演示：只要挂载读得到被抛弃的根，影子账就会隔离它的单元、不红。

### 4.8 第 1、2 条的最小复现与推翻条件

- 第 1 条：主工作区现有用例本身就是固定历史，8 步，第 3 步就红。

```
cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --exact crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
```

  - 这条用例钉的是「F 抬进被抛弃根 txg 的已知红第 0 条」（第 7 步）。
  - 在 C554 那一形修好之前，第 3 步的 I-7.4 会先把这段历史截停，第 0 条走不到。
  - 这是用例设计与 checker 补全之间的冲突；怎么处理不归我判。
- 第 2 条：用仓里现有的收缩工具（`#[ignore]`，不在重型清单里）收种子 …115。命令与原样（`shrink-115-2eca8b.log`，截到 160 字符）：

```
SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=7463871032432355115 SINGLEFS_RANDOM_HISTORY_THREADS=5 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --exact shrink_one_failing_seed_from_the_environment --ignored --nocapture
  最短复现：起点 AfterMakeFilesystem，3 步
    0. CloseAndMountWritable → Applied(Mounted { instance: InstanceGeneration(1), publishes: 2, allocation_records_compared: NoPublishWithFile, reuse: RecordR
    1. PublishFirstFile(ContentChoice { length: Empty, fill_seed: 0 }) → Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_c
    2. CrashRecoveryAbandoningTheNewestRoot → Applied(Mounted { instance: InstanceGeneration(2), publishes: 2, allocation_records_compared: NoPublishWithFile,
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.70s
```

  最短复现是 3 步：mkfs 之后挂载、发第一个文件、崩溃恢复抛弃这个第一个文件版本。恢复那次挂载自己的写行 / 暖机就复用了它的单元。
- 推翻条件：
  - 同一段历史里，恢复挂载时被抛弃的根读得出（或在复用之前插一次普通挂载），I-7.4 仍在这条根上红 ⇒ 机理不是「挂载看不见」。在副本里造过一次：不红（4.7）。
  - 被 checker 点名的那个槽在被抛弃根发布之后没有再被 `mark_allocated` ⇒ 是 checker 误报。第 1 条观测到了第二次分配（4.1）。
  - 判红时被抛弃根的 txg < F_生效 ⇒ 要回到「被抛弃根那一半该不该看 F」那一问。28/28 没有这种情况。

## 五、第 3 条：准入式子放行、落点取不到

### 5.1 复现与二分

- B2b 现场（`/tmp/claude-1000/impl-rev-b2b/random-history-copy-1.log` 第 388 行）与 A4 报告（`research/prompts/m2-rev-a4-implementer-report.md` 第 107 行）报的数相同：
  - `NoSpaceAfterRaisingTheFloor(PlacementRefused(NoFreeSlotOnAnyDevice))` 2 次；
  - `NoSpaceAfterRaisingTheFloor(SpaceAdmissionRefused)` 90 次；
  - 最高 txg 367。
- 在我的副本里只换 `admission.rs` 一份，别的不动：

| `admission.rs` | 来源 | 结局 | 日志 |
|---|---|---|---|
| `2eca8b86…` | 稍后的主工作区（A4b 改到一半） | ok；落点拒绝 0；`SpaceAdmissionRefused` 1095；最高 txg 559 | `run1.log` |
| `08c40771…` | `/tmp/claude-1000/impl-rev-a4b/admission-a4-baseline.rs`，A4b 当作 A4 基线存的那份 | 红 | `red3-a4-admission.log` |
| `529da256…` | HEAD | 编不过（core 别处引用 `admission_reading_of_a_writable_mount`、`publish_has_an_ordinary_allocation`，HEAD 这份没有），没跑成 | `red3-head-admission.log` |

`08c40771…` 那一格的原样：

```
历史 32 段：跑完 32、以已知红收尾 {}、新发现 0；根环转过一圈的 32 段；最高 txg 367；一版里最多 506 条分配记录
  Err 成员 UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))：2 次
  Err 成员 UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::SpaceAdmissionRefused)：90 次
thread 'unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval' (1717328) panicked at crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:496:5:
  left: 2
 right: 0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 25 filtered out; finished in 11.94s
```

- 与 A4 报告第 12 行「改后出现 2 次…原先 0 次」合起来看：这一条随 A4 的改动出现，随 A4b 的改动消失，与 B2b、B2 的 checker 改动无关（这个取样点的 checker 判红为 0，「新发现 0」）。
- 「`08c40771…` 就是 B2b 取的副本里那一份」没有直接证据：B2b 的 `md5-at-start.txt` 只记了 checker 文件。只有数与 A4 报告、B2b 日志逐项相同这一条。

### 5.2 那两次落点拒绝时盘上是什么样（`red3-probe2.log`，`admission.rs` = `08c40771…`）

- 加的探针：`mounted_session.rs` 推抬 F 停下（`Stopped`）而最后一次被拒是 `PlacementRefused` 时，打出每块盘的空闲 / 隔离 / 扣住 / 真可发 / defer / 单元区槽数，以及对齐的空闲槽对落在聚簇段外 / 段内各几对，还有这次挂载开过的聚簇段。
- 原样：

```
DIAG-R3 placement refused after admission; stop=PublishesPerAdmissionWouldBeExceeded { publishes_pushed: 6, publishes_of_the_next_raise: 3 }; pushed=6; per device (free, isolated, held, usable, deferred, unit_area)=[(0, (100, 0, 0, 100, 140, 256)), (1, (100, 0, 0, 100, 140, 256))]; refusal=PlacementRefused { unit: Data(DataUnitIndexInFile(0)), refusal: NoFreeSlotOnAnyDevice }; pairs(dev,outside_segments,inside_segments),segments=([(0, 0, 49), (1, 0, 49)], [50304, 50368])
DIAG-R3 placement refused after admission; stop=PublishesPerAdmissionWouldBeExceeded { publishes_pushed: 6, publishes_of_the_next_raise: 3 }; pushed=6; per device (free, isolated, held, usable, deferred, unit_area)=[(0, (100, 0, 0, 100, 140, 256)), (1, (100, 0, 0, 100, 140, 256))]; refusal=PlacementRefused { unit: Data(DataUnitIndexInFile(0)), refusal: NoFreeSlotOnAnyDevice }; pairs(dev,outside_segments,inside_segments),segments=([(0, 0, 48), (1, 0, 48)], [50304, 50368])
  left: 2
 right: 0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 25 filtered out; finished in 12.22s
```

- 两次都是：每块盘 256 槽的单元区里空闲 100 槽，隔离 0、扣住 0。
- 对齐的空闲槽对 49 / 48 对，全在这次挂载开过的两个聚簇段（50304、50368）里，段外 0 对。
- 数据单元的落点规则（`crates/singlefs-core/src/allocator.rs:579` `lowest_user_data_slot`，第 587 行判 `inside_cluster_segment`）不许落进任何开放聚簇段，所以 `NoFreeSlotOnAnyDevice`。
- 准入式子（`08c40771…` 的 `AdmissionReading::of_allocator`）按字节数计容量、已分配、隔离，没有一项装「空闲但在聚簇段里、用户数据用不了」的槽，所以它放行了。
- A4 报告第 107 行推的另一个原因：

  `08c40771…` 第 308 行：    /// ⚠️ 两处例外计数先于记账行变：挂载重建时按回收门槛回收、按影子账隔离之后到写行那次发布之前；抬 F 回收的槽扣住到 F 生效之前
  这两次里扣住 = 0，不成立。
- 为什么 A4b 那一版（`2eca8b86…`）不红：它把 ckpt_cost 改成按最坏情况计，分配记录树每盘两条路径，中央映射树逐层计（`diff` 见 `saved/`），保留池更大，式子更早拒（`SpaceAdmissionRefused` 90 → 1095）。
  - 这是推的，没量：式子没有补上聚簇段那一项，只是更早拒；有没有别的种子在 A4b 那一版上仍走到「放行而落点取不到」，这一轮没查，这个取样点 32 个种子上是 0。
- 所以第 3 条和 A4 有关，关系在 ckpt_cost 的大小，A4b 正在改的正是这一处；不是 checker 的事，也不是 C554。
- 推翻条件：
  - 在 `08c40771…` 上，那两次拒绝时段外还有对齐空闲槽对，或扣住 > 0 ⇒ 机理不是聚簇段；探针已看过，没有。
  - 在 `2eca8b86…` 上同一个取样点仍出现落点拒绝 ⇒「随 A4b 消失」不成立；`run1.log` 里是 0。
- 最小复现：在一份把 `admission.rs` 换成 `/tmp/claude-1000/impl-rev-a4b/admission-a4-baseline.rs`（`08c40771…`）的副本里跑下面这条，32 个种子、12 秒、`left: 2 right: 0`。没收缩到单个种子：执行器的计数表不带种子，这一轮没做。

```
cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --exact unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
```

## 六、没做什么

- 没修、没判该怎么改，也没判第 1 条用例、formatted_pool 用例的期望该怎么钉。
- 没跑重型测试：名字带 layer0 的、54 / 55 / 57 / 59 / 87 号、全量 cargo test、`gate.sh` 都没跑。门禁 74 号整段没跑，只跑了它里面这一个测试二进制，外加 formatted_pool 一条用例、收缩那一条 `#[ignore]` 用例和我自己的诊断测试。
- 「被抛弃根已在 F_生效 之下时这一半该不该看 F」没有观测：28 个种子里一次都没出现。
- 第 3 条：没收缩到单个种子；没核 A4b 那一版是否在别的种子上仍有同一个缺口；B2b 取的副本里 core 七份文件的确切版本无从核对，副本已删。
- 第 2 条 28 个种子里，逐槽盯过分配栈的只有第 1 条固定历史、…129、…119；其余 25 个靠「槽在藏起来的范围里、中间没有挂载、复用那一步判红」推出同一机理，再用 5 个种子的反事实验证。
- 副本里的改动（诊断打印、诊断测试 `tests/diag_gate74.rs`、换过的 `admission.rs`）都没回主工作区；改动全文在 `diag-instrumentation.patch`。第 3 条那两个探针（`mounted_session.rs`、`allocator.rs` 的 `diag_counts` / `diag_pairs`）也在里面，打印不受环境变量控制，只在「推满之后落点被拒」时出声。
- 没有产物进 `research/results/`：这是调查，不是登记过的实验；日志都在草稿目录里，要不要留由主 agent 定。
- 清理：仓副本 `/tmp/claude-1000/investigate-gate74-reds/repo/` 与编译目录 `/tmp/claude-1000/investigate-gate74-reds/target/` 交回前删掉；日志、`saved/`（三版 `admission.rs`）、`diag-instrumentation.patch`、`progress.md` 留着。
