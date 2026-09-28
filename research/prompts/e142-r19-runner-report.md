# E142 第十九次跑 执行员报告（停在步 ② 冻结之后；步 ③④ 没做）

日期：2026-09-27 开工，同日交回。

## 结论

- 步 ⓪ 八条开跑条件都过（命令与输出在「步 ⓪」一节）；R19E-1 把 S19-clause 基线换成 kb 第八批之后的现值 `e1bda65b…`，写进登记第十二节。
- 步 ① 做完：四个 sha256 等于期望（模型 `0676ed9b…` 按 R19C-3(a)）；快照 A、臂 N18 产物落盘（准入放行）；Q142.56 零差异（臂 N18 的模型那一侧与第十八次 `-main-2` 逐行相同，V19a 没触发）；D_crates 归一化 diff 写成文件（28 行），没读内容、没归因。
- 步 ② 做完并冻结（R19E-2）：模型照 N19C 的 ①–⑪ 改完，单测 94 过 0 败 1 忽略；变异表 180 条，第二次整张跑 抓到 180 / 无效 0 / 没红 0，内存撞顶 0、超时 0，「已还原，基线仍全绿」，退出码 0。
- **步 ③ 没开，照主 agent 的指示交回。** 快照 A之后别的会话把 `crates/` 拆出新 crate `singlefs-checker-tier`（导出 bin 在内），`crates/singlefs-core/src/{mount,recovery,system_configuration,transaction}.rs` 与 `crates/singlefs-format/src/lib.rs` 的 sha256 都变了，`research/scripts/replay.sh` 的 `driver_e142` 被改成 `-p singlefs-checker-tier`，我的模型文件里也被改了两行注释。照原文跑步 ③，快照 A 与 B 一定不同，会触发 **V19c**。
- 问题单三行都**没够判**（见「岔路表」）。

什么现象会推翻「步 ② 冻结有效」：冻结之后模型或变异表的 sha256 又变了（别的会话还有 4 处注释写着 `singlefs-harness`，会话的路径迁移可能还会改这份文件）。

## 步 ⓪（开跑条件）

- 第 1 条：派发提示给了八份报告与另六份，逐个 `test -f` 都在（14 份）。
- 第 2、3 条：逐件在「结论」「写过的文件」两节点名的 `crates/` 文件里各 grep 一个标识符（C577 只读节标题与「这一轮写过的文件」）。原样输出（`/tmp/claude-1000/e142-r19-runner/step0/cond2.txt`）：
```
[C554yi] crates/singlefs-core/src/mount.rs :: NewerStateStillUnreadableAfterOneReread -> 8
[C554yi] test -f crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs -> ok
[C554yi] crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs :: crash_recovery_abandoning_the_unwitnessed_row_publish_ -> 1
[C554yi] crates/singlefs-harness/tests/second_transaction_supplement_two_unreadable_abandoned_root_slot.rs :: an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable -> 1
[C554yi] crates/singlefs-harness/src/history.rs :: NewerStateStillUnreadableAfterOneReread -> 4
[C554yi] crates/singlefs-harness/src/bin/first_transaction_on_device.rs :: NewerStateStillUnreadableAfterOneReread -> 2
[C554yi] crates/singlefs-harness/src/model_comparison.rs :: NewerStateStillUnreadableAfterOneReread -> 6
[C554yi] crates/singlefs-harness/tests/common/mod.rs :: SharedUnreadableRanges -> 5
[C554yi-carry] crates/singlefs-core/src/transaction.rs :: JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION -> 3
[C554yi-carry] crates/singlefs-core/src/transaction.rs :: roll_back_acquisition -> 3
[C554yi-carry] test -f crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs -> ok
[Z3Ayi] crates/singlefs-core/src/mount.rs :: self_verified_system_configuration_slots_of_the_pool -> 5
[Z3Ayi] crates/singlefs-core/src/mounted_session.rs :: DevicesWithoutASelfVerifiedSystemConfiguration -> 2
[Z3Ayi] test -f crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs -> ok
[A3a] crates/singlefs-core/src/pointer.rs :: PointerHeadFieldOutsideTheFirstVersion -> 13
[A3a] crates/singlefs-core/src/unit.rs :: EncryptionReservedBytesNotZero -> 2
[A3a] crates/singlefs-core/src/code_two_tree.rs :: InternalEntryRefusal -> 6
[A3a] crates/singlefs-core/src/journal.rs :: record_offset -> 7
[A3a] crates/singlefs-core/src/allocator.rs :: DeviceEndsBeforeTheUnitAreaStart -> 5
[A3a] crates/singlefs-core/src/recovery.rs :: SystemConfigurationValueRefused -> 3
[A3a] crates/singlefs-core/src/mount.rs :: format_time_allocator -> 5
[A3a] test -f crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs -> ok
[A3c] crates/singlefs-core/src/transaction.rs :: InstanceAcquisitionFailed -> 19
[A3c] crates/singlefs-core/src/mount.rs :: InstanceAcquisitionFailed -> 5
[A3c] crates/singlefs-core/src/inode_tree.rs :: a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit -> 2
[A3c] test -f crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs -> ok
[A3c] crates/singlefs-harness/src/history.rs :: InstanceAcquisitionFailed -> 0
[A3d] crates/singlefs-harness/src/bad_disk_input.rs :: EntriesKeptWhenNarrowing -> 14
[A3d] crates/singlefs-harness/src/fault_injection.rs :: CountedDevice -> 1
[A3d] crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs :: observation_of -> 5
[A3d] crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split.rs :: a_central_mapping_root_whose_separator_hides_a_key_is_refused_by_the_rebuild_before_the_instance_generation_is_acquired -> 1
[A3b] crates/singlefs-core/src/system_configuration.rs :: UNIT_AREA_START_SLOT_OFFSET -> 3
[A3b] crates/singlefs-core/src/make_filesystem.rs :: UnitAreaStartOffTheClusterSegmentBoundaryUnsupported -> 4
[A3b] crates/singlefs-core/src/recovery.rs :: unit_area_start_of_the_chosen_system_configuration -> 5
[A3b] crates/singlefs-core/src/mount.rs :: with_unit_area_start -> 2
[A3b] crates/singlefs-harness/src/crash.rs :: unit_area_start_slot_recorded_in_the_slot -> 2
[A3b] crates/singlefs-harness/src/model.rs :: after_make_filesystem_on_a_journal_ring_of -> 3
[A3b] crates/singlefs-harness/src/history.rs :: InstanceTableOfTheNewestRootStillUnreadableAfterOneReread -> 2
[A3b] test -f crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs -> ok
[C577] crates/singlefs-core/src/transaction.rs :: CommitStep::Barrier -> 10
[C577] test -f crates/singlefs-harness/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs -> ok
[A3c] crates/singlefs-harness/src/history.rs :: publish_without_units -> 7（第 2985 行 publish_error_member 臂）
```
- 新第 3 条原样：`recovery.rs:741` 一行 `pub(crate) fn unit_area_start_of_the_chosen_system_configuration(`；`grep -c with_unit_area_start mount.rs` = 2；C577 报告在；`transaction.rs:1193` 一行；函数体里 `CommitStep::Barrier` 计数 1。
- 第 4 条：`grep -c research/prompts/e142-r19-prereg.md .claude/gate.d/stage-inputs.tsv` = 1。
- 第 5 条：`research/results/e142-r19-crates-status-2026-09-27.txt`（37 行，那一刻 16 份文件有未提交改动）。
- 第 6 条（`ps`）：别的会话在跑三条 `cargo test -p singlefs-harness`（8G / 24G 包装）；没有 qemu、vm-bench、e152、fio。臂 N18 那次导出排内存队 88 秒。
- 第 7 条：派发提示有那一句（step_five 与 crash_segments 正由「合入后验证一」改钉值，没合入）。
- 第 8 条：重抄七段 189 行，sha256 `e1bda65b8b3c333efc4cba03cf88dfbce7ca43d7a22ae837e7a49d5ba8cf49ce`，与基线差三类（D13 出处行号挪一；`layout/01` 第八节五处用例路径 `crates/singlefs-harness/tests/` → `crates/singlefs-checker/tests/`；D16 已定项 7 依据「实现待派」→「实现已合入（…）」）。第二类派发提示没点名，判它只是路径、不碰段序列与字数，照 R19C-3(b) 同一种收严换基线，**交主 agent 认**（R19E-1）。步 ② 开工前再抄一次，sha256 相同。

## 步 ①

- (1) `sha256sum` 原样：模型 `0676ed9bc6f21fd9a6627eb55543021c63e02d0c5c74d9452c0305d0cd3f7ba2`、独立 bin `f4f2638681bf548228ea9a71971a269e2eb72cc9581ac9c90e0dcd75bd12feeb`、模型变异表 `dfc51c2093fa7dd3761884af0f3b20bf8f68a7bd9e6ba049ed6d9154e29f0016`、独立 bin 变异表 `8978e80a8a70a454f244a77f650eff04620016b7794042ed8388547933708a8e`，与期望相同。
- (2) 快照 A：`research/results/e142-r19-crates-sha256-a-2026-09-27.txt`，171 行。
- (3) 臂 N18 产物：`research/results/e142-first-txn-dry-run-2026-09-27-r19-arm-n18.out`，754 行，sha256 `2bbacde1ec2f45408537bf263a2dfe7959258f1a4751f3dffc5f752a0895d7d8`。照 `driver_e142` 当时的两条命令（`-p singlefs-harness`）跑，模型 stdout 在前、导出在后。准入 stderr 原样：`✓ 放行 E142：输入自 research/results/e142-first-txn-dry-run-2026-09-26-r18-main.out 以来变了（那一份 f62e18e1695d，今天 0c63a9f36181）`；头一行 `E7INPUT name=input_fingerprint key=E142 sha256=0c63a9f36181c2845b648ae1e77ac2dfa7a17c0873a61e079299baf60eed0655 files=192`；`forced_rerun` 0 行。完成标记：`2` 行 `name=done`。
- (4) R36：`cmp e142-first-txn-dry-run-2026-09-26-r18-main.out …-main-2.out` 退 1，只差第 1 行（准入头：`f62e18e1…` 对 `d75bf4db…`），第 2 行起 `diff | wc -l` = 0。
- D_crates（第十八次 `-main-2` → 臂 N18，归一化）：`research/results/e142-r19-diff-r18-to-arm-n18-2026-09-27.txt`，28 行（删 10、加 14）。按登记只打行数，内容没读、没归因（归因在步 ④）。
- Q142.56：两份各取第一行 `name=done` 之前、剔掉读导出的六种行（归一化脚本 `--model-side-only`），各 681 行，`diff` 退 0、0 行 ⇒ 「N18 = 第十八次的模型」，V19a 没触发。归一化脚本 `/tmp/claude-1000/e142-r19-runner/diff/normalize.py`。

## 步 ②（模型改动、单测、变异、冻结）

改的都在 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`，照登记条款原文写，没读 `crates/`：
- ① `RecordedOperation::Barrier(DeviceIdentity)`；`RecordingPool::barrier` 按设备号升序每块盘记一步，不合并（R40）。
- ② 新 `SegmentClosingRule`（「没放行的盘」集合），`split_into_segments` 与 `segment_step_kinds` 共用；`window_segment_sizes_from_dump` 没动。
- ③ `is_in_place_overwrite`（不是单元写、长于 512、R39 那一版镜像上罩住的字节里有非零）、`in_place_overwrite_flags`、`closed_form_three_state_count`；五条路径各一行 `name=segments_three_state`。
- ④ 种类串规范序 `zero_fill` 最前（`canonical_rank_in_kinds_text`）。⑤⑨ `first_transaction_write_list_matches`：写 29、屏障步 3 × 盘数、FUA 1。
- ⑥ G10a / G10b 四行 `name=segments_sensitivity`（`swallow_one_device_barrier_step_before_the_root`）。⑧ `TrailingPublishBarrier`：暖机两次与第一个事务在系统配置槽写之后各一道池屏障（只在 `BarrierPolicy::Settled`）。⑩ G11 五行、G12a / G12b 四行（`trailing_barrier=off`）。⑪ `layer0_main_arm_verdict` 门槛 67_108_885 → 16_777_240；单测里 67108885 / 134217754 的钉值照 C5、C13 改，`#[ignore]` 那条改名 `layer0_state_count_is_16777240_with_zero_violations`。
- ⑦ `name=segments ` 五行的字段与次序没动。

单测：`cargo test --release -p e7-index-bench --bin e142-first-txn-dry-run`（经 8G 包装、`capped.sh 4`）末行原样 `test result: ok. 94 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.85s`。新加 8 条（屏障每盘一步、三态五行、G11 五行、吞屏障八行、窗口计数与 write_list_ok、原地覆写三条件的合成点、放行规则的合成点），改 7 条的钉值。锚点全取登记 7.1 / 7.2 / 7.4。
第一次跑红两条（`transaction_issues_21_…` 与 `publish_group_boundaries_…` 还钉旧的 2 道屏障与 `26`），照 7.4 C5 / C11 改了钉值与名字，不是改模型。

变异：表 158 → 180 行。重锚 3 条（M2、M13、M154，写法见登记 R19E-2）；加 M180–M201 共 22 条。整张第二次跑（`MUTATE_JOBS=2 capped.sh 2 bash scripts/mutate.sh e142-first-txn-dry-run …`，日志 `/tmp/claude-1000/e142-r19-runner/mutate-2.log`）按行首符号数：✅ 180、⏭ 0、❌ 0、💥 0、⚠️ 0、⏱ 0、🧱 0；收尾原样「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」，退出码 0。**抓到 180 / 无效 0 / 没红 0**，没有要分类的。
第一次整张跑同为 180 条抓到，但退 5（别的会话改了模型两行注释），不算数。

冻结（写进登记 R19E-2）：模型 9319 行 `41ef20b50ffc32adece04054d6ffdffce1d0f6ca8d127ad8680953b37e4b9dd1`；变异表 180 行 `15a4fe6da352e15236f267b38e6735295d7d0c7659900de6dedd80867d2e9c77`。
`naming-lint.sh` 退 1，红在它自己的 awk 读不到 `crates/singlefs-harness/src/bad_disk_input.rs`（被别的会话挪走了），不是这份源码；没法用它核新名字，交主 agent。

## 快照 A 之后 crates/ 的改动（V19c 的事实）

`find crates -type f -not -path '*/target/*' | sort | xargs sha256sum` 再与快照 A 比：`diff` 282 行（删 114、加 118），存 `/tmp/claude-1000/e142-r19-runner/snapshotA-vs-later.diff`。非测试文件：
- 新 crate `crates/singlefs-checker-tier/`（`Cargo.toml` 未跟踪、`src/lib.rs` 新），从 `crates/singlefs-harness/src/` 挪进去：`bad_disk_input.rs`、`crash.rs`、`crash_injection.rs`、`device_log.rs`、`layer0_progress.rs`、`on_device_modes.rs` 与 bin `e142_first_transaction_write_dump.rs`、`e142_first_transaction_write_dump_one_device.rs`、`e156_…`、`e158_…`、`e161_…`、`first_transaction_device_log_check.rs`、`first_transaction_on_device.rs`、`first_transaction_region_bytes.rs`；十几份测试也挪了。
- 内容变了：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`system_configuration.rs`、`transaction.rs`，`crates/singlefs-format/src/lib.rs`，`crates/singlefs-checker/Cargo.toml`、`crates/singlefs-harness/Cargo.toml`，`crates/mutations.tsv`。内容我没读（步 ② 禁读 core / format）。
- `research/scripts/replay.sh` 的 `driver_e142` 已被改成 `cargo run -q -p singlefs-checker-tier --bin e142_first_transaction_write_dump`；臂 N18 是按 `-p singlefs-harness` 跑的。
- 模型里还有 4 处注释写着 `singlefs-harness`，迁移那边可能还会改这份文件（冻结的 sha256 随之失效）。

## 判决行点名（第 4c 步；这一次的产物只有臂 N18 一份）

`grep -n 'name=verdict' research/results/e142-first-txn-dry-run-2026-09-27-r19-arm-n18.out` 共 1 条，第 714 行。表示「没过」的字段：
- `layer0_states_ok=not_run`、`layer0_violations=not_run`、`journal_differing_states=not_run`：模型写死不跑层 0 主臂枚举（第 6557 行起那段，登记「够判停机」），登记预期内。
- `control_outcome_matrix_ok=false`：同一份产物第 640 行 `E7RESULT name=layer0_control_outcome_matrix_summary off_diagonal=2`，门槛是 `off_diagonal == 0`。与第十八次 `-main-2` 同值（Q142.56 零差异）。登记没给它预期值，是不是预期内**拿不准**，交主 agent。
- `g7_states_ok=false`、`g7_violations_zero=false`：第 642 行 `E7RESULT name=layer0_sensitivity point=G7 barriers=none fua_is_boundary=true segments=12+1 states=4097 closed_form=4097 violations=2046 …`，门槛写死 2050 与 0。登记第四节第 4 条预言这一格不改门槛时判 false，**预期内**。与第十八次同值。
这三格都是臂 N18（改动前的模型）的值；N19C 那一列没有产物。

## 门禁（登记给执行员的阶段；这一次不写实验页，14 道都在最后跑）

    33-mutation-tables exit=0 |   ✓ 150 个实验二进制都有成形的变异表，1732 条变异的原文各命中源码一次；crates/mutations.tsv 1363 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 7 
    52-segment-registry exit=1 |        再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。
    80-absolute-assertions exit=0 |       crates/singlefs-checker-tier/src/bin/first_transaction_region_bytes.rs
    96-experiment-source-discipline exit=0 |         e92_reuse_requirement.rs
    27-format-constants exit=0 |   ✓ 格式常量同步（51 个已登记，51 个在源码里被钉住）
    34-experiment-index-sync exit=0 |   ✓ 实验索引行与正文标题一致（索引 162 行、正文 162 份）
    40-results-cited exit=1 |        E163：e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out e163-gpu-multicard-crc32c-2026-09-27-r1.out
    69-evidence-in-repo exit=1 |                原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。
    75-decision-experiment-links exit=0 |   ✓ 决策与实验双向登记对得上、回看不过期（查了实验页 162 个、决策 28 条；实验页 162 个（待回填 0）、决策 28 条（待回填 0）、已瘦身决策的已定项 346 个（其中写「无�
    84-verdict-false-named exit=0 |   ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 15 个）
    85-repro-command exit=0 |     写明「原始输出未留存」不判的 0 页：（没有）
    86-experiment-orphans exit=0 |     扫了 research crates/singlefs-checker-tier/src/bin（target/ 不扫）；不在的目录 0 个：（没有）
    88-quoted-result-lines exit=77 |   ! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判
    99-multipath-registry exit=0 |   ✓ 登记了路径与共用项的实验页判过了（5 份、21 条路径）；还没写登记节的 9 份在 multipath-registry-lag.tsv 里、每一份都现存且确实还没写，与基准 61b40787cb9fbe2de2eba582af4176abb0
    
    [exited with code 0]

红的三道与这一次的关系：
- 52 号红两处：mkfs 那一行种类串 kb 与钉它的用例不一致、第二条流的数组在挪到 `crates/singlefs-checker-tier/tests/` 的用例里找不到。两处都不指 E142 产物；登记本来就说改登记行之后 52 号预期会红，这一次没改登记行。
- 40 号红：这一次落盘的四份文件（臂 N18 产物、快照 A、`crates/` 状态、D_crates diff）还没被实验页点名；另有别的实验（E142 旧产物、E152、E163…）的旧项。这一次按指示不写实验页，交主 agent。
- 69 号红：模型（别的会话改的那一次）与变异表比最新带指纹的产物（臂 N18）新——要主产物才能转绿；另有一处 `research/prompts/m2-impl-r2-fixes-a-evidence/spec.md:25` 引 /tmp，不是这一次的。
- 86 号出路里「删 research 下文件」那一句没照做（它这次是绿的）。

## 岔路表（问题单三行）

| # | 状态 | 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 段数、写数、两态与三态闭式 | 还差 | 步 ③ 全部：主产物（冻结的 N19C 在定下来的 `crates/` 上跑 `driver_e142`）、Q142.46–Q142.50、Q142.57、Q142.60、Q142.61、Q142.48 新格、PC-a、PC-d 与判别力自证 ①–④、R41 行数。模型一侧的数目前只被单测钉在 7.4 的锚点上（`three_state_rows_match_the_registered_anchors` 等），没有产物，不算量过 | 能：与 step_five / crash_segments 钉值比、与导出现采比（Q142.49）都还没做；R40 分叉（R19C-10 第 4 条）预计在这一格显出 |
| 2 六个判决字段 | 还差 | Q142.51 的 N19C 那一列（要主产物）。第十八次与臂 N18 两列已有：臂 N18 第 714 行与第十八次逐字相同（Q142.56 零差异） | 能：N19C 那一列没量 |
| 3 与第十八次逐行比、各因哪一批 | 还差 | D_crates 文件已落盘（28 行，没读、没归因）；D_model、D_total、Q142.52–Q142.55、PC-b、PC-c 都要主产物。另外 D_crates 是在快照 A 那一版 `crates/` 上量的，`crates/` 之后又变了（上一节），续做时要重取快照 A 并重跑臂 N18，否则 V19c | 能 |
| 第二段（R19C-8 层 0 主臂） | 未跑 | 重型，等主 agent 问用户 | — |

续做要备的：臂 N18 的源码是登记冻结的 `0676ed9b…`，工作区里的模型已经是 N19C。重跑臂 N18 要另编那一份（`git show` 能取到的那一版加别的会话改的注释），编在哪、产物算不算，由主 agent 定。草稿目录里已备好比对脚本，没跑过真产物：`/tmp/claude-1000/e142-r19-runner/judge/anchors.py`（7.1 / 7.2 / 7.4 锚点逐格抄）、`judge.py`（Q142.47 / 48 / 57 / 60 / 61、R41、判别力自证 ①–④）、`pc_b.sh`、`pc_c.sh`；归一化脚本 `diff/normalize.py`。PC-a 的抽取脚本没写（要步 ③-8 才能读用例）。

## 登记修订

- R19E-1（步 ⓪，任何产物之前）：S19-clause 基线换成 kb 第八批之后的现值；依据是派发提示第 8 条原句；多出来的 layout/01 用例路径一类交主 agent 认。
- R19E-2（步 ② 冻结，主产物之前）：单测、变异三个数、M2 / M13 / M154 重锚、冻结的两个 sha256、停在步 ③ 之前的理由。
- 臂与判据一格没改。

## 没做什么

- 步 ③④ 没做（主产物、快照 B / C、准入核指纹、独立 bin、D_model / D_total、读用例钉值、控制、归因）；`replay.sh` 第 171 行没改；实验页、实验索引、`experiments-history.md`、`layout/01` 第八节都没写（layout/01 不在我的写范围，是准入输入）；doc-lint 没跑（没写 kb）。
- 没判这个实验的结论能不能推翻或确立决策；没跑门禁全量；没提交。第二段（层 0 主臂枚举）是重型，没跑。
- 没读 C577 报告的「结论」「第 1 条盘点」「钉值怎么变」；没读 `crates/singlefs-core/**` 等步 ② 禁读的文件；没读 D_crates diff 的内容。
- 没删：`/tmp/singlefs-mutate-target` 与 `-w2`（`mutate.sh` 跨轮复用的编译目录，9 月 22 / 25 日就有，不是这一次建的）。草稿目录 `/tmp/claude-1000/e142-r19-runner/`（4.3M）里没有编译目录与仓副本，报告、脚本与日志留给主 agent 核。
