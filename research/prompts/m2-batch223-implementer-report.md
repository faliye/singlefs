# 实二二三报告（实二二 + 实二三并做：a–h，加主 agent 追加的 i 与 h 的 Z4-1 历史）

时刻一律 UTC。底：主工作区 17:02:26 的 `crates/`、`litmus/`、`Cargo.*`（rsync 进 `base/`，16:50 打进的实二十、实十六接续、实十九都在里面）。
改在副本 `repo/`，交付的是 `repo/` 对 `base/` 的补丁 `impl-m2-batch223.patch`（只含 `crates/` 下的源码与测试；`crates/mutations.tsv` 不在补丁里，按变异名另交三件）。

## 一、结论

| 件 | 做到哪 | 验收 |
|---|---|---|
| a 发布失败原样重发 | 做完 | 注入一次写失败（B 的第二条记录在盘 1 上写失败）：B 冻结在分配器上，再发 C 在任何写之前被拒；原样重发时失败那一遍落盘的每一步是重发这一遍的前缀、逐字节相同；之后 C 照常；冷启动读回 C，checker 一条不红（I-8.9 成立）。改之前同一段历史恢复撞 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`（第三节末 basecheck） |
| b 读者规则两格 | 做完 | C539、C540 各一份坏镜像：恢复断在那里、那次发布不施加；checker 的 I-8.9 分别判「不从 1 起」「末条之后还有记录」 |
| c 只一份坏两盘一起留 | 做完，`ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported` 删了 | 一份坏：两盘记录都留、覆盖写照成、冷启动读回、下一次覆盖写不被拒 |
| d 位置项池外 / 两条同盘 | 做完，成员改名去掉「未定」 | 两种各一条：在任何写之前拒、盘上逐字节不变（快照比两盘四个系统配置槽、根环、录制流步数）、分配器一条记录没改写 |
| e checker 的 I-3.1 / I-3.11 豁免 | 做完，按主 agent 定的**按单元**读法（18:1x UTC 收到） | 两份都坏：豁免、不红；一份坏一份好：两条记录都豁免、不红；改回两份都好：照红 |
| f 树表 0 条写行跨记录 | 做完 | 只供测试的开关把一条记录压到一项：写行跨两条记录（事务号 0、序号 1、2、提交标记与末条标志只在末条）；恢复锚在末条、施加下一次发布；checker 一条不红；再挂载照常 |
| g 回退见证 | 做完（格式、写序、择根、重放、候选集、删除规则、表满、checker 两条新判定）；层 0 三条流只 build 过 | C332 那一格：回退实例的根在两块盘上都读不出，恢复择 R_old、读回 A；改之前落到被抛弃的 C（第三节末 basecheck） |
| h 写行释放核验挪到取号之前 | 做完，另加主 agent 追加的 Z4-1 历史 | 坏映射条目上写行的释放核验报错：取号之前拒、号不变、盘上逐字节不变；树表 0 条那一臂 66 片、分配记录装不下：取号之前拒、号不涨，再试一次同样 |
| i 事务号 0 跨多条记录的提交标记 | 做完（主 agent 追加） | 提交标记 2 的坏镜像：读者断链、那次发布不施加；checker 的 I-8.8 判「提交标记字节」。`mount.rs` 那句「jsn 最大」改成按末条标志认 |

**还红着的（交主 agent，第六节 Q1）**：故障注入二进制的 `fault_injection_fast_tier_returns_errors_instead_of_panicking` 与 `one_fixed_history_injects_twelve_faults_and_none_of_them_panics` 两条红，新发现一条：种子 7463871032432355113 在第 7 步覆盖写的释放读盘核上**瞬时读回坏字节**（`read_returns_corrupted_bytes`），按 c 两盘记录一起留，而盘上那两份其实都完好，按 e（包括按单元读法）读得出且对得上就不豁免 ⇒ I-3.11 红。按单元豁免只修掉「一份真坏、一份好」那一格，这一格修不掉。基底 `base/` 上这两条是绿的（那时这一格在写之前被 `…AsymmetricRecordsUnsupported` 拒掉、算「返回了错误」）。

推翻条件：主工作区打上补丁之后第五节那些二进制里有一条红（除上面两条）；第三节证过的变异在主表里有一条不红；层 0 三条新流全量跑出违例；统一跑的 `cargo test --all` 里有别的二进制因这份补丁红（第八节列了最可能受牵连的几个）。

### 主 agent 18:5x 要写清的四件

1. **还红的测试**：`second_transaction_supplement_three_fault_injection` 二进制里两条——
   - `fault_injection_fast_tier_returns_errors_instead_of_panicking`：红在 `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs:112`「注入之后「已知红」清单外的失败」；
   - `one_fixed_history_injects_twelve_faults_and_none_of_them_panics`：红在同一文件第 319 行「「已知红」清单外的失败」。
   两条都是同一个新发现：种子 7463871032432355113、`read_returns_corrupted_bytes`、整池第 787018 次读、第 7 步 `PublishOverwrite`，`CheckerViolations { invariants: ["I-3.11"] }`，说明「盘 0：记账的已分配 Some(933888) 减 defer 待释放 Some(704512)，不等于从最新根（txg 10）走读到的 196608（其中隔离豁免 0）」（`logs-touched-fault-injection-2.log`）。
   原因就是主 agent 18:5x 定的那一格：释放读盘核那一读报成功、读回的字节翻了一位，核出对不上就隔离（两盘一起留），盘上两份其实都完好，checker 读得出且对得上不豁免。修法（运行时核出对不上也先重读一次、两次都坏才隔离；每次读都给坏字节的那一格登记成已知红）交下一个实现员，这份补丁里没做。
2. **c × e 按单元豁免已落**：checker `quarantined_slots_exempted_per_device` 按单元（起点槽、跨度）归组，一组里任一份 `check_unit` 不过整组豁免（第二节 e；用例 10、11；第三节 e 那三行变异）。它修掉了「一份真坏、一份好」那一格（探针 `probe-one-copy-checker.log`：按份豁免时好盘那条 I-3.11 红），修不掉上面那一格。
3. **I-7.11 回退到 (0, 0) 那一格**：按单元豁免之后第一次重跑故障注入，快档里还有一条新发现——种子 7463871032432355114、`barrier_fails`、第 7 步 `CloseAndMountRollback`，I-7.11「所选根（实例 1、txg 2）的实例表罩不住见证条目 (新实例 1, 目标 (0, 0))」。回退到 mkfs 的第 0 代根时实例 0 不写行（D18 已定项 11），我写的 I-7.11 却要目标实例那一行。修法：目标实例是 0 时不要那一行（实例 0 只有 txg 0 那一条根，没有越过目标的根要抛弃）；配用例 `a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero` 与一行变异（第二轮证红）。修完重跑，快档与 one_fixed_history 只剩第 1 条那一个新发现。
4. **证红第二轮作废重来**：第一轮在新拷的副本上做，23 条都红在点名的那条（`mutation/proof-summary-round1.txt`）。第二轮（e 的新锚点、I-7.11 的新变异）之前我用 `rsync -a` 把 `repo/` 拷回已有构建的 `mutant/`：`rsync -a` 保留了 `repo/` 里较旧的 mtime，比第一轮还原时 `touch` 过的时间早，cargo 认为源码没变、用了第一轮最后一条变异（`to_slot` 不写见证表）编出来的旧产物，I-7.11 那条变异没红在点名的测试上、别的 8 条见证用例反而红了。正是定义第 3 步点名的那个坑。处置：那一轮作废（`mutation/proof-summary-round1-and-stale-round2.txt` 留着），删掉 `mutant/` 重拷、先跑两个二进制的基线（都绿）再证，4 条都红在点名的那条（第三节表里 e 三行与 I-7.11 那一行是这一遍的）。

## 二、做了什么（按条款）

**a（D23 已定项 14「这一版的失败处置」）**
- 三条发布路径（带文件、零单元、树表 0 条上写行）先把要落盘的字节装成 `PublishWrites`（单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换），由同一个 `persist_publish_writes` 落盘（`transaction.rs`）。
- 落盘中途失败的那一次冻结成 `FrozenPublish`（装好的字节、那一版、发布成立之后的分配器），住在 `PoolAllocator::frozen_publish`：每一次发布都要交分配器进来，冻结着时 `publish_version` / `publish_first_file` / `publish_instance_table_on_version_without_file` 第一道就拒（`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），`raise_rollback_floor` 在动分配器之前拒（包在 `RaiseFloorSequencePublishFailed` 里）。`resend_the_frozen_publish` 逐字节重发，成功就把分配器换成发布成立之后那一份、交回那一版。
- 系统配置槽不在冻结的字节里：世代号按写那一刻盘上自证过的槽现算（D22 已定项 16 逐盘计），重发时照常轮换。
- 零单元发布（`publish_without_units`）不冻结：它只在暖机与可写挂载取号之后那一串里用，失败即整个挂载返回错误，下一次发布在下一次挂载里、换了实例代号。
**b（D23 已定项 4 读者规则，C539、C540）**：`replay_journal` 加两格——任一次发布的首条（锚点读得出时接上的那一次与之后每一次）序号不是 1 断在这一条；带末条标志的那一条之后同一 (实例, txg) 里还有读得出的记录（`a_readable_record_of_the_same_publish_follows`）断在带标志的那一条。checker 的 I-8.9 早就判这两格，没改。
**c（D19 已定项 5）**：`copies_failing_the_release_checksum_check` 里一个单元有一份对不上（或重读一次之后仍读不出），两块盘上那一份都进隔离、记录都留在已分配；隔离项加字段 `reading`（`QuarantinedCopyReading`：对不上 / 重读之后读不出 / 这一份好而另一份坏），计数报出时分得开真坏的份数。
**d（同上）**：`refuse_mapping_entries_that_do_not_name_two_pool_devices`：位置项指池外的盘 ⇒ `MappingEntryLocationOnADeviceOutsideThePool`（原 `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided` 改名），两条指同一块盘 ⇒ `MappingEntryLocationsOnTheSameDevice`（新）。在读盘核任何一份之前判；取号之前的预演里也判（h）。
**e（invariants.md I-3.1、I-3.11 末句；按单元是主 agent 定的读法）**：checker `quarantined_slots_exempted_per_device`：最新根那棵分配记录树里未释放、没有引用的记录按单元（起点槽、跨度）归组，一组里有一份 `check_unit` 不过（读不出、头校验和或载荷 CRC 不对）整组豁免；I-3.1 对全部走过的版本的引用、I-3.11 对最新根那一遍的引用各算一份。违例说明里带「其中隔离豁免 N」。
**f（D23 已定项 17；实十六接续 Q7）**：树表 0 条的一版上写行改走 `roles_named_by_each_record_of_the_publish` / `transaction_offset_of_each_record_of_the_publish`（与带文件那一路同一个切法）；`VersionWithoutFilePublishOutput` 加 `earlier_records_of_this_publish`；`mount` 的 `placements_taken_by` 按全部记录的点名项接。只供测试的开关 `JournalRecordNamedEntryCapacity`（`PoolWriter::set_journal_record_named_entry_capacity`，产品路径恒 `FromTheRecordFormat`，`branch_name` 报得出走了哪一臂）把一条记录装的项数压小。
**g（D23 已定项 14「回退见证」；D22 已定项 9）**
- 格式：`singlefs-format` 加 `ROLLBACK_WITNESS_*`——槽内偏移 481 起（紧接字段表）、条数 1 字节 + 47 个 16 字节条目位 = 753 字节定宽（[481, 1234)，越过 512），条目 = 新实例代号 4 + R_old 实例代号 4 + R_old txg 8，小端；一个池的条数上限 R × S − 1（第一版 S = 8 是 23）。条目按 (N, r_old, T_old) 升序、不重复，没用上的位全 0。
- `singlefs-core::rollback_witness`（新模块）：表的装、解、抛弃判法。`SystemConfiguration` 加 `rollback_witness`，罩在整槽校验和里；读者解不开（条数越上限、不升序、r_old ≥ N、尾部不是 0）就是这一槽 `NotSelfDescribing`。空表时槽字节与改之前逐字节相同（`system_configuration_mutability_classes` 两个 sha256 照旧）。
- 写：写入口 `rollback_witness_to_write`，没装时每一次系统配置写照抄这块盘上择到的那一槽里的表；可写挂载与回退在取号之前算好两张（删除规则删过的旧表；再加上这一次回退那一条的新表），取号那两次写带旧表，从写行那次发布的轮换起带新表（写序 post）。
- 读：`recovery::rollback_witness_of_the_pool` = 各盘择到的那一槽（自证过、世代号最大）里的表取并集。`choose_root` 先跳过被见证抛弃的根；`replay_journal` 遇到被见证抛弃的记录即停；`rebuilt_allocator` 的「被抛弃」把见证并进来（影子账、根环占用表）；`mount_rollback` 的候选集把被见证抛弃的目标判 `OnAbandonedTimeline`。
- 删除规则（我定的，第六节 Q2）与表满（第六节 Q3）。
- checker：自己按 `singlefs-format` 的偏移解表（`lib.rs` `rollback_witness_of_system_configuration_slot`、`image.rs` 两个函数）；择根与候选集跳过被见证抛弃的根；新判定 I-7.10、I-7.11（第七节草稿），`IMPLEMENTED_INVARIANTS` 44 → 46。
- 层 0 三条流：`second_transaction_supplement_two_rollback_witness_layer0.rs`（第三节末）。
**h（D18 已定项 11「可写挂载的顺序」第五个合取）**
- 预演（`placements_of_the_publishes_after_acquisition_on_a_copy`）里释放核验报错不再交给发布路径：写行那次报错 ⇒ `RowPublishAdmissionRefusedBeforeAcquisition`，暖机那几次 ⇒ `WarmUpAdmissionRefusedBeforeAcquisition`（原 `ReleaseCheckFailedBeforeTheFirstPlacement` 那一臂拿掉）。预演里写行那次经映射换下的角色同样判 d 那两格。
- 树表 0 条那一臂：`release_check_and_admission_of_a_row_publish_on_a_version_without_file`（旧链与上一版分配记录树节点的释放核 + 条数准入）发布路径与取号之前那道准入共用，取号之前按它拒（Z4-1）。这是容量墙：只做到不烧号，池照样挂不上可写，归实二一的分配记录树结构。
- 报错复用已有的两个 `MountError` 成员：`first_transaction_on_device.rs`（实二四在改、我不碰）对 `MountError` 穷举，加成员它就编不过。
**i（主 agent 追加）**：`JournalRecord::parse` 读到提交标记 2..=255 交 `None`（当损坏、断链即止）；`mount_writable` 取上一版的记录按末条标志认、注释改成读法乙（代码也跟着改了，见第六节 Q10）；`recovery.rs` `rebuild_version` 文档注释同改。

## 三、新测试与「改坏哪一行 → 哪条断言红」

证红在副本 `mutant/`（`rsync -a --exclude target --exclude .git`，自己的 target）里做：先跑一遍不改动的基线（`mutation/baseline-summary.txt`），每条变异按交付的变异行改坏一处、跑那条测试所在的**整个**测试二进制，再从 `repo/` 拷回原件并 `touch`。基线红集：故障注入二进制的 `one_fixed_history…`、`fault_injection_fast_tier…`（第一节那一格）；`singlefs-core --lib` 的 `system_configuration::tests::the_rollback_witness_rides…`（第一轮基线时那条用例的断言写错了，之后改对）。下表只认基线红集之外的。
⚠️ 第二轮我先用 `rsync -a` 把 `repo/` 拷回了已有构建的 `mutant/`，mtime 倒退、cargo 用了旧产物，那一轮作废（`proof-summary-round1-and-stale-round2.txt` 留着）；删掉 `mutant/` 重拷、重跑基线之后再证，下表是重做那一遍。

| 新测试（文件） | 改坏哪一处（变异名见 `mutation/`） | 红在哪条断言 | 同时红的 |
|---|---|---|---|
| `a_publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish`（新文件 `…_publish_failure_resent_unchanged.rs`） | `refuse_while_a_publish_is_frozen` 的 `match allocator.frozen_publish()` 换成恒 `None`（冻结着不拒） | 第 207 行「B 没重发之前不许建下一次发布：Ok(…)」 | — |
| 改写的 `a_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`（故障注入） | 落盘失败时不 `freeze_publish` | 第 662 行「报的应当是「B 冻结着、还没重发」：BlockDevice(…)」 | （基线红集那两条） |
| 改写的 `publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up` | 落盘失败时不记失败账（替原第 99 行那条） | 第 538 行「中途失败过一次，交出一份账 left: 0 right: 1」 | — |
| `with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens`（`…last_record_flag.rs`） | `replay_journal` 那一格 `if records_of_the_open_publish.is_empty() && …FIRST` 前加 `false &&` | 第 194 行「C 的首条序号是 2：…施加之后走的根与施加的记录条数」 | — |
| `a_last_record_flag_followed_by_a_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens` | `a_readable_record_of_the_same_publish_follows` 那一判前加 `false &&` | 第 194 行「C 的首条带末条标志…」 | — |
| `a_checksum_failure_on_only_one_copy_keeps_both_records_allocated_and_the_pool_stays_writable_and_readable`（quarantine 用例 2，替原用例 2） | 隔离只留对不上的那一份（替原第 551 行那条） | 第 275 行「盘 0 那一份对不上，盘 1 那一份对得上也一起隔离」 | — |
| `a_mapping_location_on_a_device_outside_the_pool_is_refused_as_a_damaged_mapping_entry_before_anything_is_written`（用例 8，改名） | 池外那一判前加 `false &&`（替原第 552 行那条） | 第 795 行「位置项指的盘不在池里 ⇒ 当映射条目损坏拒绝：Ok(…)」 | — |
| `two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_entry_before_anything_is_written`（用例 9） | 同盘那一判前加 `false &&` | 第 822 行「两条位置项指同一块盘 ⇒ …：Ok(…)」 | — |
| `a_quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read`（用例 10） | checker `if some_copy_fails` 前加 `false &&`（一律不豁免） | 用例 10「I-3.1：…⇒ 豁免 left: Violated(…)」 | 用例 11 |
| 同上 | `copy_self_verifies` 恒 false（不读、一律豁免） | 用例 10「…改回对得上…照红」 | 用例 11 |
| `a_unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not`（用例 11，按单元） | 改回按份豁免 | 用例 11「单元有一份坏 ⇒ 两块盘上的两条记录都豁免」 | — |
| `a_row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it`（新文件 `…multi_record_transaction_zero_publishes.rs`） | 树表 0 条写行不按写入口装的项数切 | 第 176 行「写行那次发布跨两条记录：…」 | — |
| `a_commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens` | `JournalRecord::parse` 把 2..=255 读成「不带」 | 第 332 行「第一条的提交标记是 2：当它损坏、断链即止…left: (Some((1, 4)), 2)」 | — |
| `with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline`（新文件 `…rollback_witness.rs`） | `choose_root` 不跳过被见证抛弃的根 | 第 150 行「择到 R_old…left: ((1, 5), Some((1, 5)), 0)」 | `a_writable_mount_with_the_rollback_instances_roots_unreadable_builds_on_the_rollback_target` |
| `the_witness_rides_from_the_first_rotation_after_the_row_publish_and_every_later_system_configuration_write` | 写行那次起不换成新表 | 第 205 行「取号两次空表；…left: [[], [], [], [], [], []]」 | 另 5 条见证用例（没带见证，择根、候选、删除都跟着红） |
| `an_entry_is_dropped_only_when_every_ring_slot_is_readable_and_no_root_lies_between_the_target_and_the_new_instance` | 根环有槽读不出时照删 | 第 321 行「有一个根槽读不出：条目照留 left: []」 | 表满、回退实例读不出挂载、写序那三条 |
| `a_rollback_target_abandoned_by_the_witness_is_not_a_candidate` | 候选集不看见证 | 第 363 行「被见证表抛弃的 C 不是候选：Ok(…)」 | — |
| `a_rollback_whose_witness_table_would_exceed_its_capacity_is_refused_before_any_write` | 上限按 47 不按这个池的 S | 第 435 行「删不掉的 23 条加这一次那一条是 24 条：Ok(…)」 | — |
| `the_pool_checker_holds_both_witness_invariants_after_a_rollback` | checker I-7.11 目标那一行要求 T 严格小于 T_old | 第 571 行 `left: ["I-7.11"] right: []` | — |
| `contradicting_or_unparseable_witness_tables_redden_the_witness_consistency_invariant` | I-7.10 两种说法那一判恒真 | 第 601 行「同一个新实例两种回退目标：Holds」 | — |
| `a_witness_entry_the_chosen_roots_instance_table_does_not_cover_reddens_the_witness_table_invariant` | I-7.11 罩得住那一判恒真 | 第 669 行 `left: ["I-3.1"] right: ["I-3.1", "I-7.11"]` | — |
| `a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero`（故障注入快档第一次跑出来的新发现修了之后加的） | 回退到 (0, 0) 也要实例 0 那一行 | 这条用例（I-7.11 红） | — |
| `rollback_witness::tests::an_entry_abandons_exactly_the_roots_after_the_rollback_target_and_before_the_new_instance`（`singlefs-core --lib`） | 抛弃判法 `<` 改 `<=` | `(1, 3) left: true right: false` | — |
| `a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition`（新文件 `…row_publish_checks_before_acquisition.rs`） | 预演里不查映射条目的位置项 | 第 127 行「预演里写行那次的释放核验报错 ⇒ 取号之前拒：Err(Publish(PublishSequenceFailed { cause: MappingEntryLocationOnADeviceOutsideThePool…」 | — |
| `a_row_publish_on_a_version_without_file_that_outgrows_the_allocation_node_is_refused_before_acquisition`（Z4-1） | 树表 0 条那一臂的释放核与条数准入不在取号之前判 | 第 206 行「第 6 次挂载：分配记录装不下，取号之前拒：Publish(PublishSequenceFailed { cause: AllocationRecordsExceedOneNode { records: 942, capacity: 812 }…」 | — |

另外两处改之前的代码上跑（`basecheck/`，`base/` 的拷贝加一份只用改之前的 API 的用例，不进补丁）：
- C332：`with_the_rollback_instances_roots_unreadable…` 在改之前的代码上红——`left: ((InstanceGeneration(1), CheckpointTxg(5)), Some((InstanceGeneration(1), CheckpointTxg(5))), 0)`，落到被抛弃的 C（`basecheck-zz_c332_on_the_code_before_the_change.log`）。
- a：B 落盘中途失败之后在 A 上另建 C，C 的根落盘之后恢复：`Failed { root: Some((InstanceGeneration(1), CheckpointTxg(4))), failure: RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4), counters: [4, 5] } }`（`basecheck-zz_resend_on_the_code_before_the_change.log`）。

层 0 三条流（`second_transaction_supplement_two_rollback_witness_layer0.rs`，名字含 layer0，照规矩没跑、没证红，只 build 过）：回退（紧接被抛弃实例之后的回退，写行那次的轮换是第一次带非空表的系统配置写）、第二次回退（表两条、第二条落在 [498, 514) 越过 512）、跨挂载（回退挂载崩在写行那次的轮换刚写完 → 回退实例写行那条根一直读不出的可写挂载 → 撤故障再挂一次）。每条一个快档（10 写以上的段不展开）与一个 `#[ignore]` 全量；段序列与状态数没写死（第一次全量跑出来再登记）。比已有流多罩的：流 1 回退前没有普通挂载、没有中间实例；流 2 旧表整张带着再加一条的那几次系统配置写与 D 这条被第二次回退抛弃的根；流 3 一次在读故障下写出来的挂载整段——已有流都没有。快档断言 checker 每条都 0 违例、I-7.10 / I-7.11 至少评估过一次。

## 四、变异表（按变异名，不按行号；三件在 `mutation/`）

- `mutations-replacements.tsv`（sha256 db7c5ba162a594bc316a6f81c9c0aa7fc420b8d1969224ab4cfd545ca8e1a747）：18 行整行替代，第一段是主表里现在那一行的变异名、其后六段是替代它的整行（锚点随改动腐化的；其中 4 行连名字一起换了）。
- `mutations-append.tsv`（sha256 99583fe6ac5677d243696e46969c2894dc9aa2e7cddc21cdf44ae5fa09bb6c96）：27 行追加到末尾。
- `mutations-delete.txt`：空（没有要删的行）。
- 核过：主工作区 18:49 的现状打上补丁、主表按这三件改完，total=648 bad=0（每一行的原文在目标文件里恰好命中一次；`tools/anchor_check.py`）。
- 证过的：第三节表里那 26 条（替代行里 4 条、追加行里 22 条；第一轮按旧锚点证的 e 那一条作废、第二轮按新锚点重证）；其余 14 条替代行只换锚点、点名的测试没变，其余 5 条追加行没单独证，整表复跑留给门禁 59 号。

替代的（旧名 → 新名，同名的只写一次）：

```
失败的发布不退回分配器
步 3：零单元发布在记录与根之间少一道屏障 → 步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）
增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）
增补 3 第 4 件（被测的性质）：发布落盘阶段的块设备错改成 panic，不再把错返回上来
增补 3 第 4 件（增补 2 收口表第 40 行 / C381 的判别力）：发布失败时分配器不再退回（C381 的候选改法之一），下一次发布就不会盖掉那条根指着的单元 → 增补 3 第 4 件 × 这一版的失败处置（D23 已定项 14，实二二三）：落盘失败的发布不冻结——同一个进程拿同一个上一版另建一次发布，把那条已落盘的根指着的单元盖掉（C381 那一格回来）
C511（2026-09-23 用户定案，I-9.14 射程收窄的那一条）：候选集不再把被回退行判出局的根剔掉（被抛弃时间线的根又进了遍历与并集）
C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）
C394（释放判定不核映射条目位置项里的单元校验和）：释放之前读盘核校验和那一核拿掉（位置项的校验和比不比都当对得上），被改坏的那一份照常回到空闲池
C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）
并行线一：C394：释放之前读盘核校验和只走这次重写的角色（文件变短换下的尾巴不核，被改坏的那一份照常回到空闲池）
P6 后一半：多条记录的发布每条都写本次发布内序号 1
P6 后一半：树表 0 条那一版上写行那次发布的本次发布内序号写成 2
实二十：树表 0 条那一版上写行那次发布的记录不带末条标志
C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉）
C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试）
C394 N3（2026-09-24 故障注入快档打中的那一格）：只一部分副本对不上时照做、不在写之前停（两块盘的分配记录从此不对称，冷启动走读判整池失败） → C394 × D19 已定项 5（用户 2026-09-25 定：任一份对不上两块盘一起留）：只留对不上的那一份、对得上的那一份照常释放（两块盘的账不对称：冷启动走读判整池失败）
C394：位置项指的盘不在池里时不在写之前交回点名条款没写的成员（当成读不出、按对不上处置，却没有那块盘的记录可留） → D19 已定项 5（用户 2026-09-25 定）：映射条目的位置项指池外的盘时不当映射条目损坏拒（接着读盘核那一份，没有那块盘的记录可留）
实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见）（树分裂换锚点：第 0 次释放那一段按两棵多层树的形状重排过）
```

追加的：

```
实二二三 a（D23 已定项 14「这一版的失败处置」）：冻结着一次没重发的发布时发布路径不拒（另建下一次发布，同一 (实例, txg) 留下两条末条）
实二二三 a：原样重发成功之后分配器不换成那次发布成立之后的一份（下一次发布的落点落在重发的那次发布的单元上）
实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接
实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界
实二二三 d（D19 已定项 5，用户 2026-09-25 定）：映射条目两条位置项指同一块盘时不拒（另一块盘那一份没核就释放）
实二二三 e（I-3.1 / I-3.11 的隔离读法，用户 2026-09-25 定）：已分配而没有根引用的记录一律不豁免
实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）
实二二三 e（按单元判，主 agent 2026-09-25 定）：只豁免读不出或对不上的那一份的记录（对得上那块盘上的那一条照红）
实二二三 f（D23 已定项 17，实十六接续 Q7）：树表 0 条的一版上写行不按写入口装的「一条记录装几项」切（点名项装不下一条也只写一条）
实二二三 f / i：树表 0 条的一版上写行跨多条记录时每条都带提交标记（事务号 0 的事务被切开）
实二二三 i（主 agent 2026-09-24 定）：读者把提交标记 2..=255 读成「不带」（与 checker 的 I-8.8 说的不是一件事）
实二二三 g（C332，D23 已定项 14「回退见证」）：择根不跳过被见证表抛弃的根（回退实例的根都读不出时落到被抛弃的 C）
实二二三 g（C332）：重放不在被见证表抛弃的记录上停（落到 R_old 之后把被抛弃的 B、C 的记录施加回来）
实二二三 g：见证表不落进系统配置槽（to_slot 不写那 753 字节）
实二二三 g（写序 post）：回退那一次挂载从写行那次的轮换起不带这一次回退那一条（一直写删过的旧表）
实二二三 g（删除规则）：根环有槽读不出时照删条目
实二二三 g（删除规则）：环里没有落在 [r_old, N) 的根时条目也不删
实二二三 g：回退目标被见证表抛弃时照当候选（回退回被抛弃的时间线）
实二二三 g（条数上限 R × S − 1）：见证表上限按定宽的 47 算、不按这个池的 S（表满不拒，写出这个池自己读不回的表）
实二二三 g（checker I-7.10）：同一个新实例两种回退目标不判红
实二二三 g（checker I-7.10）：校验和过而见证表解不开的槽不判红
实二二三 g（checker I-7.11）：所选根的实例表罩不住见证条目不判红
实二二三 g（checker I-7.11）：回退目标那一行的 T 要严格小于目标 txg 才算罩得住（回退行 (r_old, T_old) 自己罩不住，健康镜像上红）
实二二三 g（回退见证的判法，D23 已定项 14）：(r_old, T_old) ≤ (i, T) 也算抛弃（R_old 自己被抛弃）
实二二三 g（checker I-7.11）：回退到 mkfs 的第 0 代根时也要实例 0 那一行（实例 0 不写行，健康镜像上红）
实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）
实二二三 h（Z4-1）：树表 0 条的一版上写行的释放核与条数准入不在取号之前判（装不下时取号之后才拒，每试一次烧一个号）
```

## 五、验证（副本 `repo/` 上，线程上限 5；末尾原样）

动到的测试二进制各跑整个（`touched-summary.txt`，18:3x 那一遍；之后改过的 walk.rs 与两份测试文件，quarantine、rollback_witness 两个二进制在第二轮证红的基线里重跑过、故障注入重跑过）：

```
--lib exit=0 | test result: ok. 110 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s | failed=[]
--lib exit=0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | failed=[]
--lib exit=0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | failed=[]
second_transaction_supplement_two_publish_failure_resent_unchanged exit=0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s | failed=[]
second_transaction_parallel_line_one_last_record_flag exit=0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.17s | failed=[]
second_transaction_supplement_two_release_checksum_quarantine exit=0 | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.62s | failed=[]
second_transaction_parallel_line_one_multi_unit_file exit=0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.42s | failed=[]
second_transaction_supplement_one_write_accounting exit=0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s | failed=[]
second_transaction_supplement_three_fault_injection exit=101 | test result: FAILED. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 69.39s | failed=[one_fixed_history_injects_tw
second_transaction_supplement_two_multi_record_transaction_zero_publishes exit=0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s | failed=[]
second_transaction_supplement_two_rollback_witness exit=0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.45s | failed=[]
second_transaction_supplement_two_row_publish_checks_before_acquisition exit=0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.68s | failed=[]
system_configuration_mutability_classes exit=0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s | failed=[]
round2 baseline second_transaction_supplement_two_release_checksum_quarantine exit=0 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.65s failed=[]
round2 baseline second_transaction_supplement_two_rollback_witness exit=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.64s failed=[]
fault_injection 重跑：test result: FAILED. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 70.79s
```

故障注入那两条红的只有一条新发现（第一节、第六节 Q1）：种子 7463871032432355113 read_returns_corrupted_bytes：整池第 787018 次read，落在第 7 步（PublishOverwrite）：CheckerViolations { invariants: ["I-3.11"] }

fmt / clippy / build（终版）：

```
$ cargo fmt --all -- --check
Diff in /tmp/claude-1000/impl-m2-batch223/repo/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
fmt exit=1
$ cargo clippy --offline --all-targets --all-features -- -D warnings -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.70s
clippy exit=0
$ cargo build --offline --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.30s
build exit=0
```

fmt 唯一的 Diff 在 `e158_root_choice_repair.rs`：E158 执行员在改的文件，`base/` 与主工作区同样没排版，我没碰它（补丁里没有它）。

对主工作区现状（18:48:44Z）：

```
$ git apply --check impl-m2-batch223.patch
apply-check exit=0
$ sha256sum impl-m2-batch223.patch
d4ca110a9e1d101739705c08dfb4b9a34ac5a92ced55ed8ab4633a38f21e95f3  impl-m2-batch223.patch
```

主工作区现状打上补丁的副本 `applycheck/`：`cargo build --offline --all-targets` exit=0；变异表按第四节三件改完 648 行锚点全命中；登记给我的门禁阶段对它跑（`.claude`、`research` 软链到主工作区）：

```
== 33-mutation-tables.sh exit=77 |   ! 找不到 research/e7-index-bench/src/bin 或 research/mutations，本阶段跳过
== 53-format-const-placeholders.sh exit=0 |   ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写
== 92-layout-checker-sync.sh exit=0 |       第一条纯 SSD 布局线：.claude/kb/layout/02-second-txn.md
== 94-checker-implementation-disjoint.sh exit=0 |     这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半
== 93-feature-bits.sh exit=0 |   ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 1 位、登记表分出去 1 位；扫了 50 个 .rs，
== 89-closeout-row27-preconditions.sh exit=77 |       alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点
== 74-model-differential.sh exit=0 |   ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 5 段）：
```

## 六、停下交主 agent 的设计问题

**Q1（c × e，还红着）瞬时读回坏字节落在释放读盘核上。** 种子 7463871032432355113、整池第 787018 次读、第 7 步覆盖写：读盘核那一读报成功、读回的字节翻了一位（`read_returns_corrupted_bytes`），CRC 对不上 ⇒ 按 D19 已定项 5「任一份核出对不上，两块盘一起留」两条记录留在已分配；而盘上那两份都完好，checker 读出来对得上 ⇒ 按 I-3.1 / I-3.11 末句（按单元读法也一样）不豁免 ⇒ I-3.11 红（`盘 0：记账的已分配 Some(933888) 减 defer 待释放 Some(704512)，不等于从最新根（txg 10）走读到的 196608（其中隔离豁免 0）`，`logs-touched-fault-injection-2.log`）。
条款的缝：D19「读盘本身失败先重读一次」只管读失败，读成功而对不上不重读；I-3.1 / I-3.11 的豁免要 checker 自己也读出坏。主 agent 18:5x 定了：运行时核出对不上也先重读一次、两次都坏才隔离（与 N1 读盘失败那一次重读对称），故障注入若每次读都给坏字节、那一格登记成已知红的形态——交下一个实现员，这份补丁里没做，这两条用例照红交回。
**Q2（g）见证条目的删除规则（派发提示说由我定）**：条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、都自证过（是本池的根），且其中没有一条根的实例代号落在 [r_old, N) 里。理由：条目抛弃的是实例落在 (r_old, N) 的根与 r_old 里越过 T_old 的根，还护着「所选根是 r_old、不超过 T_old」时的重放；环里没有实例落在 [r_old, N) 的根，这几样都没有对象，之后新写的根实例都 > N，删了永远安全。读不出、自证不过的槽按「可能住着这样的根」算（与 D18 已定项 11 行回收的根环条件同一个读法），代价是一个槽持续读不出时条目永远删不掉。删只在挂载时算（`mount::rollback_witness_entries_still_needed`），同一次挂载的取号写就写删过的表。被攻过零轮，交代码三方。
**Q3（g）表满**：条款说「表写不满」（上限只由根环几何定），那是按根环每个槽都读得出推的；按 Q2 的删除规则，有槽持续读不出时条目删不掉，回退够多次就满。那时怎么办条款没写 ⇒ 在取号之前返回 `RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided { entries, capacity }`（包在 `MountError::Recovery` 里；`MountError` 加不了成员，理由同 h），用例钉住「返回这个成员、盘上逐字节不变」，不钉之后。走得到：用例用塞满 23 条的表造。
**Q4（g）前缀第五条怎么读见证**：我取「重放遇到被见证抛弃的记录即停」（与择根同一个判法，攻方原型也是这么补的）；C340 仍取 P2（新实例接在环里最大 jsn 之后），`rollback_high_water_of_root` 的回退行截断没动。D23 写的是「随实现交代码三方」。
**Q5（g）见证读哪几槽**：各盘择到的那一槽（自证过、世代号最大）取并集，不并旧槽——挂载删掉的条目留在旧槽里，并旧槽就删不掉；择到的那一槽读不出时退回旧槽，那一槽里多出来的条目按 Q2 已抛弃不了环里任何根。
**Q6（g）落点**：偏移 481（紧接字段表，不给 C331 乙那 8 字节留位）、条数 1 字节、按 S 上界 47 条定宽 753 字节、读者对尾部非 0 与不升序也当读不出。这几样是我取的，写进 D22 已定项 9 字段表与 `layout/02-second-txn.md` 归书记员；C331 乙真要那 8 字节时见证表要挪。
**Q7（a）抬 F 失败之后**：冻结的是抬 F 那一串里失败的那一次；重发成功之后分配器换成那次发布成立之后的一份，但 `raise_rollback_floor` 扣住的回收（`release_reclaim_holds`）只在整串做完时放开，失败的那一串永远不放，扣住的槽要到下一次挂载重建分配器才回来。没改。C541（原样重发把一次失败放大成整条发布流阻塞）照旧欠着：持续失败时调用方会一直收到块设备错或「冻结着」。
**Q8（h）报错成员**：预演里写行 / 暖机的释放核验报错复用 `RowPublishAdmissionRefusedBeforeAcquisition` / `WarmUpAdmissionRefusedBeforeAcquisition`（原因在 `cause` 里）。更贴切的是另开一个成员，但 `first_transaction_on_device.rs`（实二四在改）对 `MountError` 穷举，加成员它编不过。C542（预演不读盘核）照旧。
**Q9（f）** 写行那次跨多条记录时事务号都是 0、提交标记只在末条，是照带文件那一路的切法推的（「事务号 0 的发布跨多条时第一条提交标记写 0」是攻方 Z1 附记记下、主 agent 认的那一格）。
**Q10（i）** 主 agent 要的是改注释；我把 `mount_writable` 取「上一版的记录」的代码也改成按末条标志挑（注释说按标志、代码按 jsn 最大就又说反话）。这条记录只有 jsn 会被用到、jsn 另算，结果不变；推翻条件：有调用方读它的别的字段。`mount_rollback` 里取 R_old 那条记录的同一写法没改。

### 断言、`expect` 与 `unreachable!`：为什么走不到
- `transaction.rs` 树表 0 条写行那一路 `named_unit_of` 的 `unreachable!`：`named_roles` 由 `instance_table_page_roles_in_bump_order` 的实例表各片加 `AllocationTree` 拼成，只有这两样。
- `PoolAllocator::freeze_publish` 的 `assert!`（已经冻结着一次）：每条经分配器的发布路径第一道是 `refuse_while_a_publish_is_frozen`，冻结着时走不到落盘；`resend_the_frozen_publish` 先 `take` 再重发、失败才放回。
- `PoolWriter::set_journal_record_named_entry_capacity` 的 `assert!`：只供测试的开关给错了（0 或 > 67）。
- 两处 `.expect("交给落盘的是…一版，交回的就是它")`：`PoolVersion::with_writes` 不换成员。
- `mapping_locations_of_a_released_unit` 的 `panic!`：原 `copies_failing_the_release_checksum_check` 里那条不变量搬出来，调用方先走 `placements_to_release_via_mapping`、查不到时它已经报错返回。
- checker `.expect("非空")`：`roots.is_empty()` 之前已经返回。

## 七、不变量条文草稿（交书记员；编号 I-7.10 / I-7.11 是我取的空号，书记员可改）

| 编号 | 名字 | 判据 | 状态 |
|---|---|---|---|
| I-7.10 | 回退见证表各槽自洽、各盘一致 | 每块盘两槽里自证过（magic、整槽校验和、incompat 位）的系统配置槽，槽内偏移 481 起的回退见证表解得开：条数不超过这个池的 R × S − 1（R、S 读自同一槽）、条目按 (新实例代号, 目标实例代号, 目标 txg) 严格升序、每一条目标实例代号小于新实例代号、条数之后的条目位全 0；同一个新实例代号在各盘各槽里记的回退目标相同。各盘、各槽的条目集合可以不同（轮换写到一半崩了一新一旧、挂载按删除规则删过的条目旧槽里还在），比的只是同一个新实例代号有没有两种说法（D23（journal 的角色与格式） 已定项 14「回退见证」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_witness_tables`，解析在 `lib.rs` 的 `rollback_witness_of_system_configuration_slot`；阳性对照与两份坏镜像在 `second_transaction_supplement_two_rollback_witness.rs`） |
| I-7.11 | 所选根不被回退见证表抛弃、见证与所选根的实例表对得上 | 所选根 = 根环里自证过、不被回退见证表（各盘择到的那一槽里的表取并集）抛弃的根里 (txg, 实例代号) 最大的那一条——根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N。判两样：① 根环里至少一条自证过的根不被见证表抛弃（择根择得出来）；② 见证表里新实例代号不大于所选根实例代号的每一条 (N, r_old, T_old)，所选根指着的实例表罩得住它：r_old 那一行的 T 不大于 T_old，(r_old, N) 之间每个实例都有一行、T 是 0。新实例代号大于所选根实例代号的条目不判（所选根不在那次回退之后的时间线上，那张表里本来没有它的行） | 已实现（`walk.rs` 的 `judge_rollback_witness_against_the_chosen_roots_table`；checker 的择根与候选集同时跳过被见证表抛弃的根；坏镜像在 `second_transaction_supplement_two_rollback_witness.rs`） |

I-3.1、I-3.11 两行末尾那句（「只在 checker 自己读它罩住的那一份也读不出、或校验和对不上时豁免」）按主 agent 2026-09-25 定的按单元读法改：已分配而没有根引用的记录，它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例。

`IMPLEMENTED_INVARIANTS` 44 → 46（`crates/singlefs-checker/src/image.rs`）：invariants.md 第 12 行「判 44 条」与 `<!-- invariant-count -->` 那一句随书记员改，门禁 36 号在书记员写进之前会红。

## 八、这一轮写过的文件

补丁里的 26 个（`crates/` 下；新建 6 个：`crates/singlefs-core/src/rollback_witness.rs` 与 5 份测试 `…_multi_record_transaction_zero_publishes.rs`、`…_publish_failure_resent_unchanged.rs`、`…_rollback_witness.rs`、`…_rollback_witness_layer0.rs`、`…_row_publish_checks_before_acquisition.rs`）。`crates/mutations.tsv` 没改（第四节三件）。别的会话在改的四份（`on_device_modes.rs`、`first_transaction_on_device.rs`、`first_transaction_device_log_check.rs`、`e158_root_choice_repair.rs`）一份都没碰：`e158` 被我一次 `cargo fmt --all` 改过，当场从 `base/` 拷回、不在补丁里。

补丁对主工作区的 `git apply --stat`（18:48:44Z）：

```
 crates/singlefs-checker/src/image.rs               |   77 +
 crates/singlefs-checker/src/lib.rs                 |   67 +
 crates/singlefs-checker/src/walk.rs                |  269 +++++
 crates/singlefs-core/src/allocator.rs              |   29 +
 crates/singlefs-core/src/journal.rs                |   12 
 crates/singlefs-core/src/lib.rs                    |    1 
 crates/singlefs-core/src/make_filesystem.rs        |    2 
 crates/singlefs-core/src/mount.rs                  |  266 ++++-
 crates/singlefs-core/src/recovery.rs               |  125 ++
 crates/singlefs-core/src/rollback_witness.rs       |  314 ++++++
 crates/singlefs-core/src/system_configuration.rs   |   76 +
 crates/singlefs-core/src/transaction.rs            | 1050 +++++++++++++++-----
 crates/singlefs-format/src/lib.rs                  |   45 +
 crates/singlefs-harness/src/history.rs             |   18 
 crates/singlefs-harness/src/model_comparison.rs    |   12 
 ...ansaction_parallel_line_one_last_record_flag.rs |   71 +
 ...ransaction_parallel_line_one_multi_unit_file.rs |    5 
 ..._transaction_supplement_one_write_accounting.rs |   23 
 ...transaction_supplement_three_fault_injection.rs |   88 +-
 ..._two_multi_record_transaction_zero_publishes.rs |  372 +++++++
 ...plement_two_publish_failure_resent_unchanged.rs |  295 ++++++
 ...n_supplement_two_release_checksum_quarantine.rs |  326 +++++-
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 +++++++++
 ..._transaction_supplement_two_rollback_witness.rs |  716 ++++++++++++++
 ...nt_two_row_publish_checks_before_acquisition.rs |  246 +++++
 .../system_configuration_mutability_classes.rs     |    5 
 26 files changed, 4512 insertions(+), 455 deletions(-)
```

主工作区 `git diff --stat -- crates litmus`（18:52，只看得到别的会话的改动：我的改动都在副本里）：

```
 crates/mutations.tsv                               |  189 +-
 crates/singlefs-checker/src/image.rs               |    8 +-
 crates/singlefs-checker/src/lib.rs                 |   32 +-
 crates/singlefs-checker/src/walk.rs                | 1024 +++++++-
 crates/singlefs-core/src/allocator.rs              |   98 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  131 +-
 crates/singlefs-core/src/lib.rs                    |    1 +
 crates/singlefs-core/src/mount.rs                  |  602 ++++-
 crates/singlefs-core/src/mounted_read.rs           |   87 +-
 crates/singlefs-core/src/recovery.rs               |  624 +++--
 crates/singlefs-core/src/transaction.rs            | 2574 ++++++++++++++++----
 crates/singlefs-core/src/write_accounting.rs       |   12 +-
 crates/singlefs-format/src/lib.rs                  |   16 +
 .../src/bin/e158_root_choice_repair.rs             | 2103 +++++++++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  253 +-
 .../src/bin/first_transaction_on_device.rs         |  847 ++++++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |   55 +-
 crates/singlefs-harness/src/model.rs               |  107 +-
 crates/singlefs-harness/src/model_comparison.rs    |   50 +-
 crates/singlefs-harness/src/on_device_modes.rs     |  131 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              |  727 +++++-
 .../tests/first_transaction_step_five_publish.rs   |   42 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../second_transaction_mapping_node_admission.rs   |  257 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |   45 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  223 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  225 +-
 .../tests/second_transaction_step_four_rollback.rs |    8 +-
 .../tests/second_transaction_step_one_overwrite.rs |    1 -
 ...second_transaction_step_three_formatted_pool.rs |   45 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |   10 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ...transaction_supplement_three_fault_injection.rs |   35 +
 ..._transaction_supplement_three_random_history.rs |  182 +-
 ...nsaction_supplement_two_accounting_node_full.rs |  131 +-
 ...two_c533_row_publish_record_without_its_root.rs |    3 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   73 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   45 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +--
 ...n_supplement_two_release_checksum_quarantine.rs |  551 +++--
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  416 +++-
 47 files changed, 10346 insertions(+), 2092 deletions(-)
```

### 加了的非分支项（逐项）
- `FrozenPublish` 的三个访问器（`checkpoint_txg`、`instance`、`writes`）；`PoolAllocator::frozen_publish` / `freeze_publish` / `take_frozen_publish`；`PoolVersion::into_version_without_file` / `writes` / `with_writes`（私有）。
- `JournalRecordNamedEntryCapacity::named_entries_per_record` / `branch_name`；`PoolWriter::set_journal_record_named_entry_capacity` / `journal_record_named_entry_capacity` / `write_rollback_witness_from_now_on`（crate 内）。
- `RollbackWitnessTable` 的 `EMPTY`、`of_entries`、`entries`、`abandons`、`to_bytes`、`parse`；`RollbackWitness` 的 `of_tables`、`entries`、`abandons`；`rollback_witness_capacity`；checker 的 `RollbackWitnessEntryView::abandons`、`RollbackWitnessOfASlot`。
- 新类型上的 derive：`PublishWrites`、`JournalRecordWrite`（Clone、Debug、PartialEq、Eq）；`FrozenPublish`（Clone、Debug）；`QuarantinedCopyReading`、`JournalRecordNamedEntryCapacity`、`RollbackWitnessTable`、`RollbackWitnessTableFull`（Clone、Copy、Debug、PartialEq、Eq）；`RollbackWitnessEntry`、`RollbackWitnessEntryView`（另加 PartialOrd、Ord、Hash：表按它排序去重）；`RollbackWitness`（Clone、Debug、Default、PartialEq、Eq）。没有新的 trait 实现。

## 九、没做什么

- 没走三方对抗；层 0（三条新流的快档与全量都没跑、没证红，只 build 过）、QEMU、herd7 与 crates 变异整表归 crash-verifier / 门禁 59；没提交；kb、`research/` 一个字没动（不变量条文草稿在第七节，D22 已定项 9 字段表与 `layout/02-second-txn.md` 的见证表落点、D23 已定项 14 删除规则与第五条读法、D19 已定项 5 两个成员改名、I-3.1 / I-3.11 末句，都归书记员）。
- 没跑的二进制（按规矩只跑动到的）：`checker_known_bad_images`（它的阳性对照按 `IMPLEMENTED_INVARIANTS` 逐条比，I-7.10 / I-7.11 在没回退过的镜像上判成立，推它照绿）、`second_transaction_step_four_rollback`、`second_transaction_step_three_formatted_pool` 与 `…_second_instance`、`second_transaction_supplement_three_random_history`（门禁 74 号那五段在 `applycheck/` 上绿；整个二进制别的几条没跑）、`…crash_injection`、`…bad_disk_input`，以及全部 layer0 二进制——这几个最可能被这份补丁牵动（见证读进了择根、候选集与影子账；预演拒绝的时机变了；系统配置槽多了字段后的 753 字节）。
- 第四节 14 条替代行、5 条追加行没单独证红，只核了锚点与「改坏之后编得过」（`mutation/compile-unproven.txt`）；整表复跑留给门禁 59 号。
- 门禁：登记给我的 33、89 号报 77（本次未跑：33 找不到 `research/e7-index-bench/src/bin`，89 没有前置）；92、93、94、53、74 在 `applycheck/` 上绿。门禁 36 号（不变量条数跨文件）在书记员改 invariants.md 之前会因 44 → 46 红。
- 第六节 Q1 那一格没修（故障注入快档两条照红），Q2–Q10 是我替条款没写的地方做的选择或留下的缺口，都交主 agent。

## 草稿目录里的东西（`/tmp/claude-1000/impl-m2-batch223/`）

`base/`（底）、`repo/`（改过的）、`mutant/`（证红副本）、`basecheck/`（改之前代码上的两条证据用例）、`applycheck/`（主工作区现状 + 补丁）；`impl-m2-batch223.patch`；`mutation/`（三件变异表、`definitions.json`、`proofs*.tsv`、`proof-summary*.txt`、`baseline-summary.txt`、`compile-unproven.txt`、`logs/`）；`logs-touched-*.log`、`touched-summary.txt`、`clippy-*.log`、`build-*.log`、`fmt-final.log`、`gates.log`、`gate74.log`、`basecheck-*.log`、`tools/`（`anchor_check.py`、`definitions.py`、`mutation_rows.py`、`prove.py`、`make_patch.sh` 等）、`progress.md`、本报告。
