# 实五交回报告（准入改形态 P3 + P3pl + P3r、挂载处推、445）

时刻按本机 UTC 记；报告写于 2026-09-26。

## 一、交主 agent 定的设计（条款没写、我取的那一种，都已实现、被攻过零轮）

1. **C565（挂载处推满仍不够怎么收尾）**：取「挂载照样做成」。实例已取、行已写、暖机与推的那几串都已落盘，`mount_writable*` 交回 `Ok(Mounted)`，`MountOutput::space_admission = MountSpaceAdmission::StillShortAfterTheFloorRaises { refusal_before_acquisition, floor_raises, last_refusal, stop }`（`crates/singlefs-core/src/mount.rs:370` 起的枚举）。之后的发布照发布路径的准入判（会话里推满仍不够报 `UserChangeRefused::NoSpaceAfterRaisingTheFloor`，即 ENOSPC）；正常卸载照常可走（它抬 F 到现行那一版、不判上限），卸载之后再挂取号之前就够（用例 `mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount` 钉住）。
   - 理由：另一种（推满仍不够报可写挂载被拒）没有会话可卸载，只崩了再挂那一路每试一次再烧一个号、写一行、F 已在上限推不动，是吸收态；这一种留着正常卸载这条出路。代价：这次挂载里实例切换的预留没拿到（D2 已定项 13 字面是「任一不成立即只读挂载」），挂载内切换在 `crates/` 里没有实现（推的，没核全仓）。
   - 推翻它：主 agent 定另一种时，把 `push_floor_raises_after_the_row_publish` 推满那一臂改成返回错误；变异表第 766 行就是这一改，点名的用例会红（已证）。
2. **挂着的会话那一层怎么放**：新模块 `crates/singlefs-core/src/mounted_session.rs`。`MountedSession { allocator, current, instance, shadow_ledger }` 只持内存态、不持设备表，每次 `publish_user_change(parameters, devices, change)` 由调用方交参数与设备表（用例在两次调用之间照样拷镜像、注入故障；发布路径与抬 F 各自开写入口的做法不变）。`MountedSession::of_the_mount(Mounted)` 从可写挂载接过来。改动类型 `UserChange::{Overwrite, SequentialWrite, NewInodes}`，只接在带文件的一版后面；现行那一版树表 0 条时在任何读写之前报 `UserChangeRefused::NoFileVersionToChange`（第一个文件版本不经这一层）。`MountOutput` 加了 `shadow_ledger` 字段给会话用。
3. **一次准入最多 B = 8 次发布怎么数**：数「这次准入推的空发布 + 这次准入为之判的那一次发布」（发布路径是那次用户发布，挂载那一处是写行那次），与 `df` 保留池按 B − 1 次空发布留同一个口径（`PUBLISHES_PER_ADMISSION_AT_MOST`，`mount.rs:1402`）。推之前按抬 F 那一串的纯算函数数下一串要几次，超了就不推。挂载那一处暖机那 2–3 次不算进 B。
4. **挂载处推只对上一版带文件的挂载**：上一版树表 0 条（只做过 mkfs）准入不够照旧在取号之前拒（`SpaceAdmissionRefusedBeforeAcquisition`，盘上逐字节不变）：抬 F 要带文件的现行版本（`raise_rollback_floor` 的入参），这一版上没有可退的带文件版本、推不出空间（与卸载在这一版上一个字节都不写同一条理由）。
5. **挂载处推的次序**：写行 → 暖机（照旧，取号之前的整串预演照旧只罩这两样）→ 按可写挂载的读数（这次的 rows0）再判 → 不够就推一串抬 F 到上限 → 再判……推的根接在暖机之后，`Mounted::current` 是最后一次。推的那几串不进 `warm_up_publishes`，在 `space_admission.floor_raises()` 里。
6. **什么算「空间不够、要推」**：`PublishError::SpaceAdmissionRefused` 与 `PublishError::PlacementRefused`（任何 `refusal` 原因都算，含第一版不支持的池形状那几种）；别的错原样交回、不推（`mounted_session::refusal_is_short_of_space`）。抬 F 自己报的任何错都当「不再推」的原因交回（`FloorRaiseStop::FloorRaiseFailed`），挂载那一处也一样——包括真发途中写错（那时分配器可能冻结着一次发布，挂载照样交回，冻结要调用方重发）。这一格要不要让挂载报错，交主 agent。
7. **445 在没装根环表的分配器上**：`ring_slots_known_to_hold_a_root_by` 对没装 `RootRingOccupancy` 的分配器给空集，读坏的槽一律当没有根（今天的做法）。产品路径两处（可写挂载、`make_filesystem::allocator_after_make_filesystem`）都装；没装的只有用例自己拼的分配器（`tests/common/mod.rs` 的 `build_pool` 用 `PoolAllocator::new`）。没选「没表就报错」：没写过的槽全 0、自证不过，没表时分不出「从没写过」与「这一次读坏」，报错会让 `build_pool` 上的每一次抬 F 都被拒。
8. **445 的「这个进程知道住着根的槽」取的是分配器根环表的键集**（`RootRingOccupancy::ring_slots_known_to_hold_a_root`，`allocator.rs:738`）：挂载那一刻读得出、自证过的槽（有效与被抛弃的都算）加这个进程之后记过的根（发布在落盘之前或落盘途中失败时分配器整个换回，记着的只剩落盘做成的）。
9. **随机历史执行器**：可写挂载写行之后推的那几串，执行器照模型的抬 F 逐串比（新 F 取那一串根带的、上限取实现报的），分配代也逐次比（`history.rs` 的 `settle_mount` 与 `mount_publish_allocation_judgement`）；执行器里的用户发布仍直接调发布路径，没改成经会话（C283 的推在随机历史里只在挂载那一处跑到）。要不要让随机历史的发布也经会话，交主 agent。

## 二、五件怎么落的

| 件 | 落点 |
|---|---|
| 1 D28 已定项 1 式子 | `admission.rs`：`DeviceAdmissionTerms` 去掉 `deferred` 字段与那一项扣减（`:219` 起）；需求 = 新写的全部角色的 `span_slots`（`demand_of_the_roles_on_each_device`，`:564`）；判不判看 `publish_has_an_ordinary_allocation`（`:549`，有普通分配才判）；`SpaceBudgetOfARole::Demand` 改名 `OrdinaryAllocation`。`transaction.rs` 的 `prepare_the_version_publish` 改用这两个函数。 |
| 2 P3r | `admission.rs`：`InstanceRowsOfTheSwitchReserve { OfThisMount, OfTheNextWritableMount }`（`:581`）；`admission_reading_before_a_publish`（`:631`）按 rows0 + 1，`admission_reading_of_a_writable_mount`（`:645`）按 rows0，`mount.rs` 取号之前那一判与写行之后再判都用后者。 |
| 3 C283 + P3pl | `mounted_session.rs`（新）：被拒是准入拒或落点被拒就推一串抬 F 到上限再重发，一次准入至多 8 次发布；`mount.rs` 新加 `raise_rollback_floor_to_the_admission_ceiling`（`:1423`，上限不高于现行 F 就不推）、`push_one_floor_raise_within_the_admission_budget`（`:1483`，先按预算判再抬）、`FloorRaiseStop`（`:1460`）。 |
| 4 挂载处推 | `mount.rs` `establish_instance`：取号之前不够且上一版带文件时记下拒绝、照走（`:2443` 起），暖机之后 `push_floor_raises_after_the_row_publish`（`:2744`）推、再判；结局进 `MountOutput::space_admission`。 |
| 5 445 | `recovery.rs`：`read_root_ring_slot`（`:755`，读一个槽、分「读不出 / 自证不过」）、`every_root_ring_slot`（`:724`），`visit_valid_roots_with_ring_slots` 改走它（行为不变）。`mount.rs`：`roots_of_the_ring_for_the_ceiling`（`:1038`）读坏的槽不在根环表里就跳过，在就重读一次、仍坏报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, first_reading, reread }`（`:149`）；`rollback_floor_ceiling`（`:1096`）多一个入参（知道住着根的槽），`ring_slots_known_to_hold_a_root_by`（`:1392`）从分配器取。harness 里 `history.rs`、`model_comparison.rs`（新成员归「模型没有的理由」）、`on_device_modes.rs`、两个 bin 的穷举 `match` 与调用点跟着改。 |

## 三、用例与证红

新测试二进制两个：`crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs`（10 条）、`crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（1 条，层 0 那一格的小枚举）；共用搭建在 `crates/singlefs-harness/tests/common_admission/mod.rs`。改写两条：`second_transaction_supplement_two_admission_formula.rs` 的挂载与发布那两条（条款变了，旧期望不再成立）。单测新加两条（`admission.rs`）。

用例格对照（派发要覆盖的六格）：

| 格 | 用例 |
|---|---|
| 会话里准入放行、落点取不到 → 推之后做成 | `an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session`（384 槽，第 51 次覆盖写被落点拒、推一串之后做成；60 次全做成、checker 0 违例） |
| 删掉之后同样大小的写 3 次用户发布内做成 | `after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes`（256 槽，先覆盖写 20 次、长到 6 个单元、截断，写回被拒两次、各建一个 inode，第 3 次做成） |
| 实例表行数逼近 369 的整数倍，崩了再挂不被拒 | `at_366_instance_rows_every_admitted_overwrite_leaves_the_next_mounts_row_so_a_crash_remount_is_admitted_before_acquisition`（384 槽，崩了再挂 366 次到 rows0 = 366，之后每次放行的覆盖写在拷贝上崩了再挂，取号之前就够） |
| 240 槽只崩了再挂，第 7–9 次不再挂死 | `crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck`（15 次全做成：第 10、11 次推满仍不够、第 12 次推了就够）；C565 那一格 `mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount` |
| 崩在准入推的那一串中间，两份镜像 checker 0 违例 | `crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green`（384 槽第 51 次那一刻，只录推的那一串：20493 个状态，恢复后 oracle 与 checker 0 违例，每个状态再可写挂载、挂载后 checker 0 违例） |
| 445 三个方向 | `known_root_slot_that_reads_bad_once_is_reread_and_the_ceiling_is_unchanged`、`known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write`、`root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise`，实四丙的构造 `the_seeded_history_whose_ceiling_read_of_the_genesis_root_slot_fails_once_refuses_the_raise_above_the_ceiling` |

另一条：`writable_mount_on_a_version_without_file_short_of_its_reserve_is_refused_before_acquisition_with_the_disk_unchanged`（第一节第 4 条那一格，盘上逐字节不变、录制流一步没多）。

与攻方量到的数对不上的：攻方 240 槽只崩了再挂是第 9 次被拒，我的起点第 10 次取号之前才不够（起点少一次崩了再挂：我的 mkfs 进程里没先挂一次）；rows0 攻方记 366 用 365 次、我数的是 366 次（mkfs 进程的实例 1 不写行）。

### 证红（在仓副本里，每份副本自己的 target；改完从原件拷回并 touch）

先跑不改动的副本：基线红集为空（`lib` 123 过、admission_formula 3 过、新二进制 10 过、层 0 那条 1 过、unreadable_abandoned_root_slot 3 过、fsync_drop 17 过）。以下每一格：改坏的是 `crates/mutations.tsv` 那一行的原文 → 替换文，跑整个点名的测试二进制（debug），红的用例原样（名字是改名之前的，`a_` 起头那五条之后改了名，见第五节）：

| 变异表行 | 改坏什么 | 跑的二进制 | 红的用例 → 红在哪条断言（消息原样摘头） |
|---|---|---|---|
| 760、761（同一处） | 会话只在准入拒时推，落点被拒原样交回 | 新二进制 | `an_admitted_overwrite…`「第 51 次覆盖写做成：Publish { cause: PlacementRefused … }」；`after_a_file_is_truncated…`「第 20 次覆盖写：Publish { cause: PlacementRefused …」；同时红 `at_366…` |
| 762 | 会话被拒一律不推 | 新二进制 | `a_mount_still_short…`「这次挂载的会话里写：推满仍不够，报空间不够：Err(Publish { … SpaceAdmissionRefused …」；同时红 `after_a_file…`、`an_admitted…`、`at_366…` |
| 763 | 一次准入 8 次发布改 10 次 | 新二进制 | `a_mount_still_short…`「写行与暖机之后推两串：F 抬到 8、再到 11」；同时红 `crash_only_remounts…` |
| 764（765 同一处） | 带文件的一版照旧在取号之前拒、写行之后不推 | 新二进制 | `crash_only_remounts…`「第 10 次崩了再挂做成：SpaceAdmissionRefusedBeforeAcquisition …」；`a_mount_still_short…` 同形。765 点名随机历史那条，没在副本里跑（那个二进制约 10 分钟），留给门禁 59 号 |
| 766 | C565 换成另一种收尾（推满仍不够报拒） | 新二进制 | `a_mount_still_short…`「第 10 次：推满仍不够，挂载照样做成: SpaceAdmissionRefusedBeforeAcquisition …」；同时红 `crash_only_remounts…` |
| 767 | 树表 0 条那一版准入不够也不拒 | 新二进制 | `a_writable_mount_on_a_version_without_file…`「树表 0 条的一版上准入不够在取号之前拒：None」 |
| 768、769（同一处） | 发布路径照旧按这次的 rows0 | 新二进制、lib | `at_366…`「第 41 次覆盖写做成之后崩了再挂，取号之前就够：Ok(AdmittedAfterTheFloorRaises …」；单测 `publish_reading_reserves…`「发布路径按下一次挂载的 367 行」 |
| 770 | defer 扣两次 | lib | `reading_of_an_allocator_takes_each_devices_own_capacity_allocated_and_isolated_slots`「已分配含 defer 里的 2 槽（读法甲）；隔离只在盘 0 上」 |
| 771（与改写后的 640 同一处） | 需求退回只算普通分配 | lib、admission_formula | 单测 `demand_counts…`「每块盘的需求是这次写出的全部 10 槽，固定点的 4 槽也在内」；`an_overwrite_whose_new_slots…`「式子判拒：None」 |
| 772 | 只写固定点的发布也判 | lib | `demand_counts…`「空发布只写固定点：不判」 |
| 773、774、775（同一处） | 读坏的槽一律当没有根、不重读 | 新二进制 | `a_known_root_slot_that_reads_bad_once…`「重读读出第 0 代根，上限还是 0」；`a_known_root_slot_still_bad…`「重读仍坏就拒这次抬 F：None」；`the_seeded_history…`「历史跑完、没有新发现」 |
| 776 | 挂载时就读不出的槽也重读、仍坏就拒 | 新二进制 | `a_root_slot_already_unreadable_at_mount…`「挂载时就坏的槽不挡抬 F：Err(RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { … region: 0, slot: 0 …」；同时红 `a_known_root_slot_that_reads_bad_once…`（不注入时区域 0 槽 2 从没写过、也被当成要重读）、`the_seeded_history…`、`after_a_file…` |
| 777 | 抬 F 那一串不先写系统配置 | 层 0 那条 | `crash_states_inside…`「恢复之后池级 checker 0 违例：{"I-7.12": "盘 1：两槽里自证过的系统配置带的 F 最大 42，低于这块盘上实例 2 txg 61 那条根带的 F 57"}」（8192 个状态判红） |
| 638（改名） | 发布路径不判准入 | admission_formula | `an_overwrite_whose_new_slots…`「式子判拒：None」 |
| 639（锚点改） | 取号之前不判准入 | admission_formula | `writable_mount_short_of_its_instance_switch_reserve…`「取号之前不够、写行之后推了再判够了：NotJudgedByTheTestOnlySwitch」 |
| 641、642（改名 / 锚点改） | ckpt_cost 漏树表 / 不扣切换预留 | admission_formula | `writable_mount_short_of…`「…：AdmittedBeforeAcquisition」 |
| 644（替换文改） | 开关关不掉发布路径准入 | admission_formula | `an_overwrite_whose_new_slots…`「同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点: SpaceAdmissionRefused …」 |
| 432（锚点改） | 根环表全记成区域第 0 槽 | unreadable_abandoned_root_slot | `the_isolation_bits_only…`「重开那一刻影子账两块盘各罩住 14 个槽」 |
| 751（锚点改） | 择根读不到一槽就当整环读完 | fsync_drop… | `read_only_mount_with_only_device_zero…`「盘 0 上最新的根是 txg 5（根环区域 2 归盘 0），只读挂载沿它打开」 |

全部在 debug 下跑；被测代码里的 `debug_assert` 没有一条先红（红的都是上表里用例自己的断言）。每条新测试都有一行证过；765 那一行没证、留给提交时的门禁 59 号。证红日志与汇总：`/tmp/claude-1000/impl-rbf-5/proof2/`（`summary.txt` 一格一行），没入库（草稿，主 agent 要留再拷）。

## 四、crates/mutations.tsv

追加 18 行（第 760–777 行），变异名：
- 760 实五 P3pl（D16 已定项 1「准入」那一行：准入放行而落点取不到也推）：会话只在准入拒时推抬 F，落点被拒原样交回
- 761 实五 P3pl（D3 已定项 9 第 2 条的界）：会话只在准入拒时推抬 F，删掉之后同样大小的写回落点被拒原样交回
- 762 实五 C283（D16 已定项 1「准入」那一行）：会话被拒一律不推抬 F（准入拒、落点被拒都原样交回）
- 763 实五 C283：一次准入最多 8 次发布改成 10 次（推满的判据松了，推满仍不够那一格多推一串）
- 764 实五 挂载处推（D16 已定项 1「准入」那一行）：上一版带文件的可写挂载准入不够照旧在取号之前拒，写行之后不推（只崩了再挂的窄池挂死）
- 765 实五 挂载处推：上一版带文件的可写挂载准入不够照旧在取号之前拒（取样点上可写挂载一次都没推过）
- 766 实五 C565（挂载处推满仍不够怎么收尾）：换成另一种收尾——推满仍不够就报可写挂载被拒
- 767 实五 挂载处推：树表 0 条那一版准入不够也不在取号之前拒（照带文件那一格往下走，取号、写盘）
- 768 实五 P3r（D28 已定项 3「发布路径用 rows0 + 1」）：发布路径的切换预留照旧按这次挂载的 rows0
- 769 实五 P3r：发布路径的切换预留照旧按这次挂载的 rows0（读数那一处）
- 770 实五 ND（D28 已定项 1：可用不再扣 defer 待释放）：defer 里的槽扣两次（已分配里一次、另加一次）
- 771 实五 A1n（D28 已定项 1 接线：需求 = 新写的全部槽）：需求退回只算普通分配（读数那一处的单测）
- 772 实五 A1n（D28 已定项 1 接线：判不判照旧）：只写固定点的空发布也判空间准入
- 773 实五 445（D16 已定项 1「根槽这一次读坏」那一行）：算上限时读坏的槽一律当没有根、不重读（改之前的做法，重读读得出那一格上限被抬高）
- 774 实五 445：算上限时读坏的槽一律当没有根、不重读（重读仍坏那一格照抬、F 抬过条款上限）
- 775 实五 445：算上限时读坏的槽一律当没有根、不重读（实四丙的构造：种子基 + 339 第 1 步，F 抬过条款上限、I-7.9 红）
- 776 实五 445：挂载那一刻就读不出的槽也重读、仍坏就拒（挡住了条款说当没有根的那一格）
- 777 实五 崩在准入推的那一串空发布中间：抬 F 那一串不先写系统配置（带新 F 的根落盘而系统配置里还是旧 F，I-7.12 判红）

**改了别人的行（这一批的改动让它们的锚点或点名的用例不在了；这一轮没有别的会话在改 crates/，交主 agent 过目）**：
- 375、382：点名的单测改名（去掉 defer 那一项：`…_allocated_and_isolated_slots`、`each_of_the_eight_terms_…`）。
- 432、751：`recovery.rs` 读根环改成按槽读的结局分支，锚点换成新的那一臂，替换文语义照旧（全记成区域第 0 槽 / 读不到就当整环读完）。
- 638、641、642、644：点名的用例改名（`an_overwrite_whose_new_slots_…`、`writable_mount_short_of_its_instance_switch_reserve_…`）；642 锚点换成读数按哪个 rows0 算那一处；644 的替换文里原来的 `BytesOnOneDevice` 已不在 `transaction.rs`，换成判不判的谓词（语义照旧：开关关不掉准入）。
- 639：锚点换成 `let admission_before_acquisition = match start.space_admission {`，点名用例改名。
- **640 语义反过来了**：原来那一行变异是「需求把固定点也算进去」，而 D28 已定项 1 接线（用户 2026-09-25 定）正是这样；改成「需求退回只算普通分配」、名字改成「实五 准入（D28 已定项 1 接线，用户 2026-09-25 定 A1n）：…」。
- 每一行改之前的原样在 `/tmp/claude-1000/impl-rbf-5/mutations.tsv.before-fix`（草稿）。门禁 33 号判锚点全在（见第六节）。

## 五、这一轮写过的文件（我自己列；`git diff --stat` 里还有前几批没提交的改动，分不出谁改的）

- 新建：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-harness/tests/common_admission/mod.rs`、`crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs`、`crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`。
- 改：`crates/singlefs-core/src/{admission.rs, allocator.rs（根环表键集的访问器与一处文档）, lib.rs（登记新模块）, mount.rs, recovery.rs, transaction.rs}`；`crates/singlefs-harness/src/{history.rs（挂载那一处推的串照模型逐串比、分配代逐次比、计数 `mounts_by_space_admission` 与 `MountSpaceAdmissionOutcome`、新成员的名字）, model_comparison.rs, on_device_modes.rs, bin/e158_root_choice_repair.rs, bin/first_transaction_on_device.rs}`；`crates/singlefs-harness/tests/{second_transaction_supplement_two_admission_formula.rs, second_transaction_supplement_three_random_history.rs}`；`crates/mutations.tsv`。
- 用例改名：新二进制里五条原以 `a_` 起头的（命名检查判单字母）去掉了 `a_`；第三节证红表里的名字是改名之前的。
- 层 0 那条用例照主 agent 转来的用户决定：证红与复跑做完之后加了 `#[ignore = "崩溃枚举：提交时由崩溃验证员按输入哈希跑（release），平时不跑"]`；接进崩溃验证员与门禁那一半没做。

`git diff --stat -- crates litmus` 末行原样（含前几批未提交的改动；四个新文件未跟踪、不在其内）：
 80 files changed, 8201 insertions(+), 6680 deletions(-)

完整的 `git diff --stat -- crates litmus` 原样在 `/tmp/claude-1000/impl-rbf-5/diffstat.txt`（80 行）。

## 六、交回前的验证（末行原样）

跑之前 `ps` 看过：没有性能测量在跑，只有我自己的 cargo。命令都经 `research/scripts/run-with-memory-cap.sh 8G`（取 `replay.sh` 的默认值）与 `capped.sh 16`（复跑那一批用 8）。

- `cargo fmt --all -- --check`：退出码 0，无输出。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 check.sh 那七条 lint：退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.08s`
- `cargo build --offline --all-targets`：退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4.26s`
- 加 `#[ignore]` 之后的最后一遍：`singlefs-core --lib` `test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`；新二进制 `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 87.41s`；层 0 那个二进制 `test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s`。
- 加 `#[ignore]` 之前、终版代码上（改名之前）跑过的：层 0 那条 `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2201.79s`（20493 个状态，8 线程）；`second_transaction_supplement_three_random_history` `test result: ok. 21 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 576.05s`；`second_transaction_supplement_two_admission_formula` `test result: ok. 3 passed; …; finished in 5.84s`；`…_unreadable_abandoned_root_slot` `ok. 3 passed`；`…_fsync_drop_and_devices_without_the_selected_version` `ok. 17 passed`；`second_transaction_step_five_reuse` `ok. 14 passed`；`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount` `ok. 11 passed`。
- 较早一版代码上跑过、之后没再跑的（受 core 改动影响、这一批没改它们的文件）：`second_transaction_step_four_rollback`、`…_root_ring_turn_in_one_mount`、`…_release_checksum_quarantine`、`…_commit_generated_fallback`、`checker_known_bad_images`、`…_three_crash_injection`、`…_three_fault_injection` 全绿；`…_three_bad_disk_input` 红一条 `every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking`（期望的槽 50368、实际 50304）——实四丙报告 `/tmp/claude-1000/impl-rbf-4c/report.md` 第三节记着这条在实四乙之后的工作区上就红、HEAD 上绿，归实七，不是这一批带进来的（我没在去掉这一批改动的副本上复核）。
- 登记给我的门禁阶段：
  - 33 号 退出码 0：`  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 772 条的原文各命中源码一次（…）`
  - 53 号 退出码 0；92 号 退出码 0；94 号 退出码 0；93 号 退出码 0；89 号 退出码 77（本次未跑，不是通过）。
  - 74 号 退出码 0：`  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：`（小盘准入判着那一段：模型对拍 4832 步、单元区墙按区间放行 2723 次）。
- 命名检查（`.claude/scripts/naming-lint.sh`，不归我）：改名之后单字母那一类 124 条，与这一批之前相同，这一批不新增。

## 七、证红之外的探针观测（草稿副本上量的，不是入库装置上的数）

- 384 / 256 / 240 槽会话里一直覆盖写 150 次、只崩了再挂 14 次、删了再写三种起点：全做成、checker 0 违例（探针日志 `/tmp/claude-1000/impl-rbf-5/probe/p1.log`–`p8.log`，草稿、没入库）。
- P3r 的判别：rows0 = 366 那一格，发布路径按这次 rows0 算时，200 次覆盖写里有 5 次之后崩了再挂取号之前不够（写行之后推了才够）；按 rows0 + 1 算时 0 次（`p7.log` / `p8.log`）。

## 八、没做什么

- 没走三方对抗；层 0 全量、QEMU、herd7、crates 变异整表归 `crash-verifier`；没提交。
- 变异表第 765 行（点名随机历史那条）没在副本里证红，留给门禁 59 号。
- 层 0 那条小枚举接进崩溃验证员与门禁（按输入哈希、release 跑）没做，按主 agent 的话另派。
- 随机历史执行器里的用户发布没改成经挂着的会话（第一节第 9 条）。
- C563（读根环的别的调用方读坏一槽）没动：条款没判。C546 真发途中写错那一格扣住位怎么放没动（条款没写）。
- kb 没写（写范围外）：D28 已定项 1 的接线描述、C283 / C562 / C565 的状态与 checks-owed 要跟着改，交书记员。

## 草稿与副本

- 删了：`/tmp/claude-1000/impl-rbf-5/probe/repo`（仓副本，158M）、`/tmp/claude-1000/impl-rbf-5/probe/target`（它的编译目录，2.2G）。
- 留着（报告与复核材料）：`/tmp/claude-1000/impl-rbf-5/proof2/`（证红日志）、`runs1/`–`runs3/`（复跑日志）、`probe/p*.log`、`scripts/`、`mutations.tsv.before-fix`、`progress.md`。
