# 第二轮改法 B + 合入后验证一没做完的那一半：实现员报告（上下文到线停下，没做完）

写于 2026-09-27 JST 22:5x（UTC 13:5x）。规格 `/tmp/claude-1000/impl-r2-fixes-b/spec.md`；途中收到主 agent 四条消息（fsync_drop / c366 并进来、C579 定案、第 7 件只改钉 35、singlefs-checker-tier 一律只静态改、线程上限改 10、上下文到线叫停），都在下文对应处。

## 一、结论

- 补丁 `patch/crates.patch`（25 个文件，+1891 / −670）对主工作区 `git apply --check` 退出码 0；在「主工作区此刻的副本 + 补丁」上 `cargo build --all-targets`、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加七条纪律 lint 全过（第六节原样）。
- 第 1、2、3 件的 src 与新测试做完、跑绿；第 4 件与四份挪走文件只静态改（checker 档）；第 5 件的小盘钉值改完、**编过没跑**；双故障两条、step_three :661、random_history 两条**没改**；第 6 件证红**一行都没做**；第 7 件主工作区已是 35，跑绿。
- 推翻条件：补丁打到主工作区后编不过；第三节「跑绿」一列里任一目标在打完补丁的树上红；`patch/mutations-append.tsv` / `-replacements.tsv` 任一行原文在打完补丁的树上不是恰好命中一次（我在最终副本上核过 34 行 + 删 1 行，0 处不对）。

## 二、补丁与文件

- `patch/crates.patch`、`patch/mutations-append.tsv`（23 行）、`patch/mutations-replacements.tsv`（10 行）、`patch/mutations-delete.txt`（1 行）、`patch/report.md`（本报告拷贝）。sha256 见交回。
- 我的改动写成可重放的脚本：`edits/apply-all.sh <仓副本>`（`01_y4.py` … `16_crash_injection_imports.py`、`new/` 下的新测试文件、`05_moved.sh` 打挪走文件），别的会话还在挪文件，补丁打不上时在主工作区新拷一份副本上重放它再出补丁。
- 写范围外的两份（规格单里没列，是加字段 / 加成员的连带）：`crates/singlefs-core/src/make_filesystem.rs`（`SystemImmutableConfiguration` 的结构体字面量加两个字段）、`crates/singlefs-harness/tests/common_admission/mod.rs`（`UserChangeRefused` 两个新成员的穷举）。改法 A 不碰这两份。
- 补丁里的文件（`git apply --check --stat` 原样末三行）：
```
 .../system_configuration_mutability_classes.rs     |    5 
 ...ing_and_unreadable_reads_are_not_passed_over.rs |  383 ++++++++++++
 25 files changed, 1891 insertions(+), 670 deletions(-)
```
  逐个：core 六份（mount、mounted_session、recovery、system_configuration、transaction、make_filesystem）；harness src 两份（history、model_comparison）；checker-tier 两个 bin（first_transaction_on_device、e158_root_choice_repair）、四份挪走测试（findings_log、record_checker、parallel_line_one_layer0、crash_injection）；harness 测试十一份（新文件 entries_after_mount_refuse_swapped_or_behind_devices、unit_area_start…、system_configuration_mutability_classes、common_admission/mod.rs、step_four_rollback、step_five_reuse、fsync_drop…、warm_up_counter、a_floor_raise…、admission_refuses…、admission_formula）。

## 三、逐件：做了什么、验到哪一步

「跑绿」= 在副本上经 `run-with-memory-cap.sh 8G` + `capped.sh 4` 跑过整个测试二进制、全过；「编过」= 只过了 build / clippy。

| # | 做了什么 | 验到 |
|---|---|---|
| 1 | `mount.rs`：`caller_inputs_agreeing_with_the_disk` 加参数 `current`，「可见」之后核落后支（新 `devices_behind_the_current_version`，与可写挂载落后支共用新抽出的 `units_of_the_version_missing_on_a_device_behind_it`）；本盘设备号那一判抽成 `own_device_numbers_on_disk_differing_from_the_identities_handed_in`（`device_table_disagreeing_with` 改调它）；新成员 `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits`、`CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnits`（回退经 `RollbackError::CallerInputsDisagreeWithTheDisk` 交出）、类型 `DeviceBehindTheCurrentVersion`、`OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn`。`mounted_session.rs`：「可见」之后再核本盘设备号与落后支，新成员 `UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`、`::DevicesBehindTheCurrentVersionAndMissingItsUnits`。四个入口（会话、卸载、抬 F、回退）都在任何写之前拒。新测试 `entries_after_mount_refuse_swapped_or_behind_devices.rs` 10 条：Y2-a 四入口各一条、Y2-b 四入口各一条、Y2-c 一条（四入口不误拒）、攻方 16 格放开扫一条（按线程切片并行） | 跑绿（10 passed，117 s）；run1 里又跑一次 0 |
| 2 | `system_configuration.rs`：`SystemImmutableConfiguration` 加 `journal_ring_start_slot`、`root_ring_base_slot`（`SlotNumber`），`parse_slot` 按偏移 325、371 读，`to_slot` 写内存里那一份（断言位置）；常量 `JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION`、`ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION`。`recovery.rs`：`system_configuration_values_this_reader_accepts` 判槽距上界改用同一槽自述的根环起点（`largest_fixed_structure_slot_spacing_under_the_root_ring_base`，逐档找槽 1 那一处仍用常量），之后判根环起点、journal 环起点等于第一版常量，新成员 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`。字面量：transaction.rs、make_filesystem.rs、mutability_classes 两处。用例进 `unit_area_start…`：攻方 A–E 五格只读与可写都拒成同一个值、盘上不变（E 格报槽距越界，与 checker I-7.13 同一格）；择到的系统配置两字段是常量 | 跑绿（unit_area_start 23 passed；mutability_classes 在 run1 前段之外，没单跑——见第五节） |
| 3 | D16 第 37 行全句：`readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 对知道住着根的槽「读不出或自证不过」都重读一次、仍坏报错；错误类型改名 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`，带 `first_reading` / `reread`，`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot` 同样带上。新用例：自证不过两遍 ⇒ 抬 F 拒、自证不过一遍 ⇒ 重读后照做（测试包装盘加「读得出、字节被改」那一种坏法）。`step_four_rollback.rs:807` 与 `step_five_reuse.rs:641`：两条都是「这个进程写过的根被改坏之后回退」⇒ 改钉「回退拒成 `CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{自证不过, 自证不过})`、盘上不变、现行版本与分配器不动」并改名；step_four 那条槽改回之后回退照做、水位照钉。「水位取 max(内存里的, 环里读得出的)」那一判挪进 `unit_area_start…` 新用例（判候选两遍读得出、算水位那一遍瞬时读不出），实三「水位」那一行变异换靶。为什么不挪到挂载之前：挂载之后写行与暖机的根都带 F 与新水位，原来那一判的区分力就没了（推的，没造） | 新用例跑绿；step_four / step_five 两条**编过没跑**（run1 在跑到它们之前被叫停） |
| 4 | `first_transaction_on_device.rs` 两条（`raise_rollback_floor_mode_…` 与 `the_recorded_stream_projects_…`）：断言改成「录制流投出的 FLUSH + 录制器并掉的那几道 = 设备一层」，并掉的数照合入后验证一跑出来的差（盘 0：12/13、16/17）写 1；盘 1 同一机制推的 1（没跑）；注释写明「录制器并掉相邻屏障、设备收两次，归第五步 55 号六档重录时定」 | 编过（checker 档，不跑） |
| 5a | 四份挪走文件：findings_log、record_checker、parallel_line_one_layer0 照 `crates-moved.patch` 打上（fuzz）；crash_injection 那份 use 行那一块手改（`16_crash_injection_imports.py`） | 编过（checker 档，不跑） |
| 5b | fsync_drop 两条（主 agent 定）：「盘 0 是空盘」那一格与「盘 1 每次读都报错」那条改钉 C554 乙成员（`PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`，见证 5 > 4 / 4 > 3，两遍读数相同）+ 拒在任何写之前；后一条改名 `…_is_refused_by_the_witness_before_any_write`；「盘 1 是空盘」那一格照钉逐盘核 | 编过没跑 |
| 5c | c366（C579 定案）：改钉可写挂载拒成乙（见证 4、判不出）、录制流一步没多、盘上不变、只读挂载择 (1, 4)、jsn 1–3、一条不施加；原来的写行 / 暖机 / 重开那一段删掉，文件头第 6–7 行照主 agent 的原句改；助手函数与用不上的 use 删掉；改名 | 编过没跑 |
| 5d | 小盘几份（`15_small_disks.py`，数全部取自草稿探针在新几何上现跑的输出 `probe/out.txt`、`probe/out2.txt`）：admission_formula 第一条 −47 → −39（其余照旧）；admission_formula 第二条换成 256 槽、覆盖写 4 次、挂载之后先直接覆盖写三次再第四次被拒（可用 11 < 需求 14，扫三档 1–12 次只有这一格落在 [6, 需求) 里）；admission_refuses 第一、四条换成 200 次空发布、写到 5 个、第 6 个（段外 0 对段内 85 对、式子可用 60 ≥ 上界 31、报 12 槽）；a_floor_raise 第一条 19 次之后第 20 次（那一串三次里第 1 次）、第二条 18 次（两次里第 1 次） | **编过没跑** |
| 5e | 没做：random_history 两条（`writable_mount_whose_own_publishes…` 探针结果：只有 18 次那一档成立，直接挂载报 `publish_index 0, warm_up_publishes_planned 1, unit AllocationTreeNodeBelowTheRoot{level 0, device 1, index 2}`；`seed_4000000204` 探针在 4000000000–399 上找到「publish 2 of 3」的有 4000000001 第 30 步等，见 `probe/out.txt` 的 PROBE6 行，但探针跑的是不带 checker 的，没核带 checker 能跑完）；双故障两条（C583，见第四节）；step_three :661（`torn_anchor…` 撕 jsn 5 那一格被乙拒、见证 6、`AgainstTheRecordAtTheWitnessedCounter{(1,6)}`，改法：那一格改钉乙成员 + `disk_snapshot` 不变，另一格照旧；没写）；sharded 金样 | — |
| 6 | 13 行证红：没做 | — |
| 7 | 主工作区已是 35（合入后验证一打上的）、只点名六份里的四份、不枚举目录，所以按主 agent 的消息什么都不用改 | run1 跑绿 |

## 四、交主 agent 定的

1. **双故障两条（C583）没改，给改法**：`random_history.rs` 的 `crash_recovery_abandoning_a_newest_root_…_known_red_form_of_closeout_row_43` 与 `step_five_reuse.rs` 的 `raising_the_floor_into_the_txg_of_the_root_abandoned_…_known_red_form_of_closeout_row_43` 都红在抬 F 之前的「一条违例都没有」（I-7.4）。清单第 0 条（`history.rs` 的 `KNOWN_RED_FORMS`）要求「只有 I-3.1 红」，池子抬 F 之前已经 I-7.4 红，抬 F 之后那一段归不进第 0 条——所以改钉只能停在「抬 F 之前恰好 I-7.4 红、说明文字点名实例 1、txg 5 的被抛弃根，已知（C583，双故障，留里程碑三）」，抬 F 与归类那一段删掉。连带：`crates/mutations.tsv` 第 755、756 行（实四乙，点名 step_five 那一条）失去取样点——756 可以试换靶到 `history.rs` 里第 0 条归类的库单测（第 4935 行起那几条），755 改的是 checker 的 I-3.1 说明文字，库单测用的是合成文字、罩不到，要另找或删。
2. **c366 改钉之后 `crates/mutations.tsv` 第 351、352、353 行（C366 挂载层三行）没有取样点了**：C579 之后可写挂载这一层分得开 txg 与 jsn 的盘面不可达。353（发布轮换写 tail）可以试换靶到同文件第一条用例（函数层，jsn 从 40 起）；351、352 在 `mount.rs` 的写行 / 暖机计数，函数层用例走不到，要删（`mutations-delete.txt`）还是留着红，交主 agent。我没动这三行。
3. `step_three_second_instance.rs` 第 36 行那条变异（链首没锚点）点名 `torn_anchor_record…`：那条改钉之后撕 jsn 5 那一格被乙拒、走不到链首判定，撕 jsn 4 那一格还走得到——改钉时要确认 36 行仍抓得到。
4. `second_transaction_step_five_reuse.rs:641` 那条原来守的是「F_生效 取 max(根上带的, 系统配置里的)」（SysPre）；改钉拒之后这一判在挂着的回退路径上没有用例了（没有变异行点名它）。要不要另造「已知根读得出、带 F 的根不在环里」的盘面补一条，交主 agent。
5. `crash_enumeration_sharded_across_processes.rs` 金样四个值不重录（主 agent 定）。推的：46 个状态下打印行 = 第一趟 1 + ⌈46/16⌉ 3 + 3 = 7 行、第二趟 1 + 1 + 46 + 3 = 51 行，共 58 行（今天钉 67）；进度文件 47 行（文件头 + 46 行片行）；两个 SHA-256 与进度文件名里的计划哈希推不出来，待提交时 crash-verifier 跑出来再钉。
6. D18 已定项 11 的落后支我照攻方原型取「落后**且**缺现行那一版的单元」（与可写挂载落后支同一判），不是 kb 字面的「不落后」：只剩一槽自证、那一槽停在上一次轮换的盘按字面会被误拒，规格要它不误拒。代价：现行那一版是零单元发布、单元与旧快照逐字节相同时，旧快照不被拒（与可写挂载落后支同一个取舍）。

## 五、没做完的、现场在哪

- **没跑的目标**（编过）：step_four_rollback、step_five_reuse、fsync_drop、warm_up_counter、a_floor_raise、admission_refuses、admission_formula、step_three（没改）、random_history（没改）、system_configuration_mutability_classes，以及 run1 在 `publish_order_matches_litmus` 之后按字母序还没轮到的全部 harness 目标（`run1/exit-codes.tsv` 34 行，最后一行 publish_order_matches_litmus 0）。run1 是改完第 1–3 件与 11–14 号脚本、没打 15 号小盘脚本时的树；红的两个（admission_refuses、a_floor_raise）就是 15 号脚本要改的那两份。
- **证红一条都没做**：第 6 件的 13 行，加上这一轮新测试的变异行（`mutations-append.tsv` 23 行、`mutations-replacements.tsv` 10 行），都没过 `prove-red.sh`。checker-tier 目标上的 711、1366–1368 按口径只能静态、留给提交时的 59 号。
- **没跑**：登记给我的门禁阶段（stage-owners 里 implementation-writer 那几道，含 74 号）；带 `SINGLEFS_…` 的重型一律不跑。
- **现场**：`edits/`（全部改动的重放脚本，`apply-all.sh <副本>`；`mutation_rows.py` 出三份 tsv）、`run1/`（每个目标一份日志与 `exit-codes.tsv`）、`probe/`（两份草稿探针源码 `zz_probe_small_disks.rs`、`zz_probe_two.rs` 与输出 `out.txt`、`out2.txt`，不交付）、`moved/`（四份挪走文件从 `crates-moved.patch` 抽出来的补丁）、`build-final.log`、`clippy-final.log`、`t-unit-area.log`、`t-entries-new.log`、`progress.md`。

## 六、原样输出

最终副本（主工作区 UTC 13:4x 的拷贝 + 重放全部脚本）上：
```
$ cargo clippy --offline --all-targets --all-features -- -D warnings <七条纪律 lint>   # capped.sh 10
    Checking singlefs-checker-tier v0.1.0 (/tmp/claude-1000/impl-r2-fixes-b/work/crates/singlefs-checker-tier)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.36s
exit=0
$ cargo fmt --all -- --check | grep -c "^Diff in"
0
$ cargo build --offline --all-targets   # capped.sh 10
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
$ git apply --check patch/crates.patch   # 在主工作区
apply_check=0
```

新用例跑过的两份（经 run-with-memory-cap.sh 8G、capped.sh 4）：
```
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.51s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 117.28s
```

run1（`run1/exit-codes.tsv` 原样，被叫停时停在第 35 个目标）：
```
lib-singlefs-format	0
lib-singlefs-core	0
lib-singlefs-harness	0
a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is	0
acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics	0
acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish	0
admission_checkpoint_cost_per_device_paths	0
admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments	101
a_floor_raise_refused_for_space_counts_as_short_of_space	101
a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable	0
a_publish_returns_only_after_the_system_configuration_rotation_is_durable	0
a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side	0
checker_cross_links_malformed_nodes_and_mapping_entries	0
checker_judges_reserved_bytes_and_widths_it_used_to_skip	0
checker_known_bad_images	0
checker_narrow_invariants_and_abandoned_roots	0
core_review_geometry_back_chain_and_empty_inode	0
core_review_tree_table_duplicates_and_slot_one_search	0
core_review_unit_area_start_and_publish_limits	0
corrupt_on_disk_content_is_refused_instead_of_panicking	0
crash_image_journal_hint_matches_the_full_ring_scan	0
entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration	0
entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write	0
entries_after_mount_refuse_swapped_or_behind_devices	0
first_transaction_region_bytes	0
first_transaction_step_five_publish	0
first_transaction_step_one_mkfs	0
first_transaction_step_six_recovery	0
first_transaction_step_two_data_unit	0
index_node_header_width_computed_three_ways_agrees_for_every_key_width	0
instance_acquisition	0
model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file	0
parallel_line_one_sequential_write	0
publish_order_matches_litmus	0
```

## 七、变异表

全部没证红（第五节）。在最终副本上逐行核过原文恰好命中一次、追加的名字表里没有、替换与删的名字表里恰好一行：0 处不对。

追加（`mutations-append.tsv`，23 行，名字）：
- 代码三方第二轮 Y2-a：会话发布不核落后支（旧快照的盘照常收到本池的写）
- 代码三方第二轮 Y2-a：会话发布不核落后支（放开扫的 16 格）
- 代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（正常卸载）
- 代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（准入抬 F）
- 代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（管理员回退）
- 代码三方第二轮 Y2-b：会话发布不比本盘设备号（盘体对调照常发布）
- 代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（正常卸载）
- 代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（准入抬 F）
- 代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（管理员回退）
- 代码三方第二轮 Y2-c：落后支只看落后、不读单元（只剩一槽自证的盘被误拒）
- 代码三方第二轮 Y4-a：读者不判根环起点等不等第一版常量（写 65 照挂）
- 代码三方第二轮 Y4-a：读者不判 journal 环起点等不等第一版常量（写 1023 照挂）
- 代码三方第二轮 Y4-a：判槽距上界退回编译期常量根环起点（根环起点写 0 那一格报成根环起点不等，不报槽距越界）
- 代码三方第二轮 Y4-a：解槽读根环起点读错了位置（偏移 371 往前挪 8）
- 代码三方第二轮 Y4-a：解槽读 journal 环起点读错了位置（偏移 325 往前挪 8）
- 代码三方第二轮 Y4-a：写者写 journal 环起点写的不是内存里那一份（写成常量 + 1）
- 代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读、当没有根（只管读不出）
- 代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读（管理员回退，step_four 那一形）
- 代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读（管理员回退，step_five 那一形）
- 代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽读坏不重读、第一遍坏就拒
- 调查三条红（主 agent 定）：C554 乙关掉，读路径那一格（盘 1 每次读都报错）走到逐盘核、不再被乙拒
- 调查三条红（主 agent 定）：C554 乙关掉，盘 0 是空盘那一格走到逐盘核、不再被乙拒
- C579（用户 2026-09-27 定：乙判不出按真）：乙把「判不出」按假处置，所选根自己那条记录读不出时可写挂载照做

整行替换（`mutations-replacements.tsv`，10 行）：前 9 行是锚点被这一轮改掉的原有行（实二七三行、实审 A1b 四行、A3a 第 38 条、A3b Q6 根槽那一行），第 10 行是实三「水位」换靶到 `unit_area_start…` 的新用例。

删（`mutations-delete.txt`）：实二八「读路径那一格……盘 1 不被点名」——那条用例改钉乙之后它的变异抓不到，改成追加里「C554 乙关掉，读路径那一格走到逐盘核」那一行。

## 八、没做什么

- 没走三方对抗；checker 档（54 号快档与全量、QEMU、herd7、crates 变异整表、singlefs-checker-tier 包的一切测试）归 crash-verifier；没提交。
- 第 6 件证红、第 5 件 random_history 两条 / 双故障两条 / step_three :661、第四节交主 agent 的几件，见上。
- 登记给我的门禁阶段一道都没跑。
- 这一轮没改 checker（`crates/singlefs-checker/src/`），不需要「受影响的层 0 流与崩溃枚举用例」一节；core 的改动（入口多核两样、读 325/371、D16 全句）会不会改层 0 流的钉值：入口核只读盘、不改写流；读 325/371 在合法镜像上与常量相同；D16 全句只在知道住着根的槽读坏时才走得到——推的，没跑层 0。

## 九、删了的副本与编译目录

- `/tmp/claude-1000/impl-r2-fixes-b/work`（改动副本，含 target）19G，删了；`/tmp/claude-1000/impl-r2-fixes-b/base`（主工作区原样拷贝）305M，删了。没有别的仓副本或编译目录。
- 留着的都不是副本：`edits/`、`patch/`、`run1/`、`probe/`（两份探针源码与输出）、`moved/`、各份日志、`progress.md`、`spec.md`。
