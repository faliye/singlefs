//! 可写挂载自己写行之后才判成被抛弃的根 R：R 的账读不出时照 C 那一圈在 R 那一槽记着的落点抬 F，R 与所选那一版、与中间那次发布共用的单元留到它那一槽写掉。

use super::*;

/// H2 那块盘之前覆盖写几次（攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K2-2 原样那一格 k = 5）。
pub const OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH: u64 = 5;

/// H2（用 C 那一圈记着的落点）：`tests/common_unwitnessed_publish/mod.rs` 那块盘（覆盖写 5 次之后录一次覆盖写 R，
/// 崩在 R 的根槽落盘之后、轮换之前），可写挂载那一刻 R 的账（树表、映射根、分配记录树根）之外写出的单元读回全零：R 走读不成、择根退到 P，
/// R 的根槽与账读得出，按最新那条根（R 自己）的表
/// 判有效、根环表记成有效根、不带落点；写行之后 R 按现行那一版的表被抛弃。挂上之后 R 的账（树表、映射根、分配记录树根）读报错：
/// 正常卸载那一串照 C 那一圈在 R 那一槽记着的落点算（那一槽记的是有效根、txg 与 R 相同），不读 R 的账、不拒（计数 0），那些落点一槽都不空闲，
/// 池级 checker 不红 I-7.4 与 I-4.8。表里没记着 R 就读盘的话，账与映射都读不出、现行账里没有凭据，卸载拒
/// （`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，两腿都量到的白拒）。
pub fn run_the_scenario_where_unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let root_of_the_recorded_publish = recorded.root();
    let account_locations_of_the_recorded_publish: Vec<LocationEntry> = [
        root_of_the_recorded_publish.tree_table,
        root_of_the_recorded_publish.mapping_root,
        root_of_the_recorded_publish.allocation_record_tree_root,
    ]
    .into_iter()
    .filter(|pointer| *pointer != NodePointer::empty_root())
    .flat_map(|pointer| pointer.locations)
    .collect();
    let unit_writes_of_the_recorded_publish_outside_its_account: Vec<
        &singlefs_verification::memory_pool::RetainedWrite,
    > = recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            !account_locations_of_the_recorded_publish
                .iter()
                .any(|location| {
                    location.device == write.device
                        && location.slot.to_device_offset() == write.offset
                })
        })
        .collect();
    assert!(
        !unit_writes_of_the_recorded_publish_outside_its_account.is_empty(),
        "R 那次覆盖写写出了账之外的单元（数据单元、两棵树的节点）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&unit_writes_of_the_recorded_publish_outside_its_account);
    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut mount_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账之外写出的单元读回全零：可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P"
    );
    let root_ring_table = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert_eq!(
        root_ring_table.placements_recorded_for_the_abandoned_root(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg
        ),
        None,
        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点"
    );
    assert!(
        root_ring_table
            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                root_of_the_recorded_publish.instance,
                root_of_the_recorded_publish.checkpoint_txg,
            )
            .is_some_and(|placements| !placements.is_empty()),
        "C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）"
    );
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&root_of_the_recorded_publish);
    let devices = devices_with_the_read_fault_on(&image_after_the_mount, &fault);
    let mut pool = pool_under_test_after_the_mount(
        &recorded.pool.parameters,
        recorded.pool.device_bytes,
        devices,
        mounted,
        recorded.pool.write_time_seconds + 10_000,
    );
    fault.is_active.set(true);
    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &root_of_the_recorded_publish);
    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "R 的账读不出");
    assert_eq!(
        abandoned_roots_unreadable, 0,
        "R 的落点借 C 那一圈在 R 那一槽记着的那一份，这一遍不读它的账"
    );
    // 期望从录下来的 R 那次发布的写独立算（正推 K1-2 末段：取自被测方法时，借来的那一份被削成一半照绿）。
    assert_none_of_the_placements_is_free(
        &allocator_after_the_unmount,
        &placements_written_by(&recorded),
        "R 自己写出的单元",
    );
    images.hand_out(
        "卸载之后 R 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 丙那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
pub const OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_THE_RECORDED_PUBLISH: u64 = 3;

/// 丙那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；攻方原样那一格 `Wss` 的两个 s）。
pub const SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 2;

/// 照 `record_one_publish_after_overwrites` 的录法录 R（崩在 R 的根槽落盘之后、系统配置轮换之前那一份写表），R 换成建一个 inode：
/// 文件的数据单元不动，R 与它之前那一版 P 共用它们（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/tests/opus10zd_h2_borrowed.rs`
/// 的 `record_r` 那一种 `n`）。
pub fn record_one_inode_creation_after_overwrites(overwrites_before: u64) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_the_recorded_publish = stream.operation_count();
    pool.create_one_inode().expect("录的那次建 inode 落得下");
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_recorded_publish..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        1,
        "录的那一次建 inode 只写一条根（准入够，没推抬 F）"
    );
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

/// R 的账（树表、中央映射树根、分配记录树根；空指针不算）那几个单元的写，每块盘一份。
pub fn account_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    let root_of_the_recorded_publish = recorded.root();
    [
        root_of_the_recorded_publish.tree_table,
        root_of_the_recorded_publish.mapping_root,
        root_of_the_recorded_publish.allocation_record_tree_root,
    ]
    .into_iter()
    .filter(|pointer| *pointer != NodePointer::empty_root())
    .flat_map(|pointer| recorded.unit_writes_at(&pointer))
    .collect()
}

/// R 引用、R 自己没写的文件数据单元的落点（R 从所选那一版 P 继承的）：录下来的会话里 R 那一版的文件数据指针，
/// 每条位置条目里 R 一份都没写过的那几个。从录下来的会话与写表算，不经被测的方法。
pub fn data_unit_placements_the_recorded_publish_shares_with_the_previous_version(
    recorded: &RecordedPublish,
) -> Vec<PlacementOnDevice> {
    let written_by_the_recorded_publish = placements_written_by(recorded);
    recorded
        .pool
        .current_file_version()
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| PlacementOnDevice {
            device: location.device,
            slot: location.slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .filter(|placement| {
            !written_by_the_recorded_publish
                .iter()
                .any(|written| written.device == placement.device && written.slot == placement.slot)
        })
        .collect()
}

/// 丙那一形挂上之后，R 的账那几段还读不读回全零。
#[derive(Clone, Copy, Debug)]
pub enum AccountOfTheRecordedPublishAfterTheMount {
    /// 挂上之后撤掉读回全零（攻方 a2、a3 那几组）。
    ReadBackAllZeroLifted,
    /// 挂上之后照旧读回全零，写到哪一段那一段才照读（攻方 a4「故障不撤」）。
    StillReadingBackAllZero,
}

/// 丙那一形（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节，原样那一格 k = 3、R 建一个 inode、故障 `acct`、动作 `Wss`）：
/// 第一个文件之后覆盖写 3 次得 P，录一次建 inode 得 R（R 与 P 共用文件的数据单元），崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻
/// R 的账（树表、中央映射树根、分配记录树根）读回全零：R 走读不成、择根退到 P，R 的记录齐，C 那两份账都不加，R 的根槽读得出、记成有效根，
/// 写行之后 R 按现行那一版的表被抛弃。挂上之后照 `after_the_mount` 撤或不撤故障；会话覆盖写到准入推出第一串抬 F（覆盖写释放了 R 与 P 共用的
/// 数据单元，推的那一刻 R 还在根环里），再顺序写两次。每一步之后池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8，R 写的单元一份都没被写掉；
/// 推过抬 F 那一刻 R 共用的数据单元与 R 写的单元覆盖的槽一槽都不空闲（期望从录下来的会话与写表独立算）；最后核挂载那一刻 C 那一槽借得到
/// R 共用的那几个数据单元（丙并上的那一截）。只有 H2、没有丙时抬 F 只借到 R 的记录点名的落点，共用的数据单元回收、被顺序写发出去，I-7.4 红。
pub fn the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(
    images: &mut PoolCheckerImages,
    after_the_mount: AccountOfTheRecordedPublishAfterTheMount,
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_THE_RECORDED_PUBLISH,
    );
    let root_of_the_recorded_publish = recorded.root();
    let shared_with_the_previous_version =
        data_unit_placements_the_recorded_publish_shares_with_the_previous_version(&recorded);
    assert!(
        !shared_with_the_previous_version.is_empty(),
        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P"
    );
    let root_ring_table = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert_eq!(
        root_ring_table.placements_recorded_for_the_abandoned_root(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg
        ),
        None,
        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点（抬 F 那一串借 C 那一圈，H2）"
    );
    let borrowed_at_the_mount = root_ring_table
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着 R 的记录点名的落点");
    match after_the_mount {
        AccountOfTheRecordedPublishAfterTheMount::ReadBackAllZeroLifted => lift.lift(),
        AccountOfTheRecordedPublishAfterTheMount::StillReadingBackAllZero => {}
    }
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_the_recorded_publish_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            let image = pool_after_the_step.image();
            if root_is_in_the_ring(&image, recorded.key) {
                images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}（{after_the_mount:?}），R 的根还在根环里：R 引用的单元一个都没复用"),
                || image.clone(),
                PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
                assert_eq!(
                units_of_the_recorded_publish_overwritten_on(&recorded, &image),
                Vec::<(DeviceIdentity, u64)>::new(),
                "{after_what}（{after_the_mount:?}），R 的根还在根环里：R 写的单元一份都没被写掉"
            );
            }
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_the_recorded_publish_was_in_the_ring =
            root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写（{after_the_mount:?}）被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_the_recorded_publish_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    // 推过抬 F 那一刻的空闲槽先记下、顺序写之后再判：池级 checker 那一道先判（回收出来的槽被顺序写发出去才红得出），这一道接着判。
    let free_among_the_shared_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &shared_with_the_previous_version);
    let free_among_the_written_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &placements_written_by(&recorded));
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!(
                    "推过抬 F 之后第 {sequential_write_number} 次顺序写（{after_the_mount:?}）被拒：{refusal:?}"
                )
            });
        hand_out_expecting_nothing_of_the_recorded_publish_reused(&pool, "推过抬 F 之后顺序写");
    }
    assert!(
        free_among_the_shared_after_the_raise.is_empty(),
        "推过抬 F 那一刻（{after_the_mount:?}）R 与 P 共用的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_shared_after_the_raise:?}"
    );
    assert!(
        free_among_the_written_after_the_raise.is_empty(),
        "推过抬 F 那一刻（{after_the_mount:?}）R 自己写出的单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_written_after_the_raise:?}"
    );
    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !borrowed_at_the_mount
                .iter()
                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_borrowed.is_empty(),
        "挂载那一刻 C 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在借得到的那一份里，缺 {shared_not_borrowed:?}"
    );
}

/// 丙：挂上之后撤掉 R 的账那几段的读回全零（a2、a3 那几组的取法），流程与断言见
/// [`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]。
/// 判别力：撤掉丙（变异「丙 较新可读根账读不出而记录齐时 C 那一槽不并现行账里仍分配着的落点」）时 I-7.4 红。
pub fn run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
    images: &mut PoolCheckerImages,
) {
    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(images,
        AccountOfTheRecordedPublishAfterTheMount::ReadBackAllZeroLifted,
    );
}

/// 丙：挂上之后 R 的账那几段照旧读回全零（攻方 a4「故障不撤」），流程与断言同上。
/// 判别力：同上那一条变异时 I-7.4 红。
pub fn run_the_scenario_where_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
    images: &mut PoolCheckerImages,
) {
    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(images,
        AccountOfTheRecordedPublishAfterTheMount::StillReadingBackAllZero,
    );
}

/// 丙并中间那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
pub const OVERWRITES_BEFORE_THE_INTERMEDIATE_AND_THE_RECORDED_PUBLISH: u64 = 3;

/// 丙并中间那一形挂上之后，会话至多覆盖写几次去等写掉 R 那一槽（384 槽小盘一圈根环 24 次发布；攻方原样那一格第 20 步写掉）。
pub const OVERWRITES_UNTIL_THE_RING_SLOT_OF_THE_RECORDED_PUBLISH_IS_WRITTEN_AT_MOST: usize = 40;

/// 录下来的 Q、R 两次发布：Q 之前那一刻（所选那一版 P）的盘面、Q 与 R 两次发布的全部写（按落盘次序）、两条根槽写在写表里的下标、
/// P、Q、R 的 (实例, txg)，加起池那一份（会话停在 R 那一版）。
pub struct RecordedIntermediateAndLaterPublishes {
    pub pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub root_index_of_the_intermediate_publish: usize,
    pub root_index_of_the_recorded_publish: usize,
    pub key_of_the_previous_version: (InstanceGeneration, CheckpointTxg),
    pub key_of_the_intermediate_publish: (InstanceGeneration, CheckpointTxg),
    pub key_of_the_recorded_publish: (InstanceGeneration, CheckpointTxg),
}

/// 攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-model/src/opus10zghi-probe/tests/opus_k21_chain.rs` 的 `record_chain`（q = `s`、r = `n`）：
/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，再录 Q（顺序写三个数据单元，三条记录）与 R（建一个 inode，
/// 文件的数据单元与 Q 共用）。
pub fn record_a_sequential_write_as_the_intermediate_publish_and_an_inode_creation_as_the_recorded_publish_after_overwrites(
    overwrites_before: u64,
) -> RecordedIntermediateAndLaterPublishes {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let key_of = |pool_being_recorded: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>| {
        let root = pool_being_recorded.session().current.root();
        (root.instance, root.checkpoint_txg)
    };
    let key_of_the_previous_version = key_of(&pool);
    let base = pool.image();
    let operations_before_the_intermediate_publish = stream.operation_count();
    pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000)
        .expect("录的那次顺序写（Q）落得下");
    let key_of_the_intermediate_publish = key_of(&pool);
    pool.create_one_inode()
        .expect("录的那次建 inode（R）落得下");
    let key_of_the_recorded_publish = key_of(&pool);
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_intermediate_publish..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        2,
        "录的 Q、R 两次发布各写一条根（准入够，没推抬 F）"
    );
    RecordedIntermediateAndLaterPublishes {
        pool,
        base,
        writes,
        root_index_of_the_intermediate_publish: root_indices[0],
        root_index_of_the_recorded_publish: root_indices[1],
        key_of_the_previous_version,
        key_of_the_intermediate_publish,
        key_of_the_recorded_publish,
    }
}

impl RecordedIntermediateAndLaterPublishes {
    /// 崩在 R 的根槽落盘之后、R 那次系统配置轮换之前（Q 那次轮换已持久）。
    pub fn crash_image_with_the_root_of_the_recorded_publish_persisted_and_its_rotation_lost(
        &self,
    ) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply_writes(&self.writes[..=self.root_index_of_the_recorded_publish]);
        image
    }

    pub fn root_of_the_recorded_publish(&self) -> RootRecord {
        RootRecord::parse_slot(
            self.writes[self.root_index_of_the_recorded_publish]
                .bytes()
                .expect("根槽写带字节"),
            &self.pool.parameters.filesystem_identifier,
        )
        .expect("R 的根自证")
    }

    /// Q 写出的单元写（每块盘一份）。
    pub fn unit_writes_of_the_intermediate_publish(&self) -> Vec<&RetainedWrite> {
        self.writes[..self.root_index_of_the_intermediate_publish]
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .collect()
    }

    /// 可写挂载那一刻读回全零的几段（攻方故障组 `qroot,qlast,racct`）：Q 的根槽、Q 带末条标志的那条记录、R 写的树表 / 中央映射树根 / 分配记录树根那几份。
    /// Q 那次轮换写的系统配置槽不在里面：要它读不回 Q 的见证，盘面用 [`image_where_both_devices_lost_their_newest_rotation_write`] 造（两槽都自证过，乙入判不出不拒）。
    pub fn ranges_reading_back_all_zero_at_the_mount(
        &self,
        crash_image: &MemoryPool,
    ) -> RangesReadingBackAllZero {
        let root_of_the_recorded_publish = self.root_of_the_recorded_publish();
        let account_pointers_of_the_recorded_publish = [
            root_of_the_recorded_publish.tree_table,
            root_of_the_recorded_publish.mapping_root,
            root_of_the_recorded_publish.allocation_record_tree_root,
        ];
        let writes_in_between_the_two_roots = &self.writes
            [self.root_index_of_the_intermediate_publish..self.root_index_of_the_recorded_publish];
        let rotation_of_the_intermediate_publish: Vec<&RetainedWrite> =
            writes_in_between_the_two_roots
                .iter()
                .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
                .collect();
        assert!(
            !rotation_of_the_intermediate_publish.is_empty(),
            "Q 那次轮换写了系统配置槽"
        );
        let account_of_the_recorded_publish: Vec<&RetainedWrite> = writes_in_between_the_two_roots
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter(|write| {
                account_pointers_of_the_recorded_publish
                    .iter()
                    .any(|pointer| {
                        *pointer != NodePointer::empty_root()
                            && pointer.locations.iter().any(|location| {
                                location.device == write.device
                                    && location.slot.to_device_offset() == write.offset
                            })
                    })
            })
            .collect();
        assert!(
            !account_of_the_recorded_publish.is_empty(),
            "R 自己写了它的账那几个单元"
        );
        RangesReadingBackAllZero::default()
            .with_the_writes(&[&self.writes[self.root_index_of_the_intermediate_publish]])
            .with_the_last_record_of_the_publish(crash_image, self.key_of_the_intermediate_publish)
            .with_the_writes(&account_of_the_recorded_publish)
    }
}

/// `placements` 里有没有一个落点罩住这次单元写的起点（同一块盘、起点槽到跨度之内）。
pub fn some_placement_covers(placements: &[PlacementOnDevice], write: &RetainedWrite) -> bool {
    placements.iter().any(|placement| {
        let start_in_bytes = placement.slot.to_device_offset().0;
        placement.device == write.device
            && start_in_bytes <= write.offset.0
            && write.offset.0 < start_in_bytes + placement.span * singlefs_format::SLOT_BYTES
    })
}

/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节，原样那一格 k = 3、Q 顺序写、R 建 inode、
/// 故障 `qroot,qlast,racct` 加 Q 那次轮换没写成、动作全是覆盖写；判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形）：
/// 第一个文件之后覆盖写 3 次得 P，录 Q（顺序写）与 R（建 inode，数据单元与 Q 共用），崩在 R 的根槽落盘之后、R 那次轮换之前；
/// 每块盘 Q 那次轮换写的那一槽没写成（两槽都自证过），可写挂载那一刻 Q 的根槽、Q 的末条记录、R 的账读回全零：择根退到 P，R 的记录齐、账读不出，走丙。
/// C 里 R 那一槽并上 (P, R) 之间 Q 那一槽记着的落点，Q 写的单元全在里面。挂上之后撤故障，会话覆盖写到写掉 R 那一槽：
/// 每一步之后、每一步最后一条根槽写之前那个段边界状态（单元与记录都落了、这一步的根没落：攻方红的那一态）池级 checker 都不红
/// I-7.4（近 K 代块未被复用）与 I-4.8；其间有一步 Q 已离开根环而 R 还在（这一形的窗口确实走到了）。
/// 判别力：撤掉丙并中间（变异「丙并中间 走丙那一支的根那一槽不并所选那一版与它之间那几槽」）时，写掉 Q 那一槽之后、
/// 写掉 R 那一槽那一步根落之前那一态 I-7.4 红，违例的根是 R；挂载那一刻 C 那一槽缺 Q 的单元那一句先红。
pub fn run_the_scenario_where_both_devices_lost_their_newest_rotation_write_and_units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_a_sequential_write_as_the_intermediate_publish_and_an_inode_creation_as_the_recorded_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_INTERMEDIATE_AND_THE_RECORDED_PUBLISH,
    );
    let crash_image = image_where_both_devices_lost_their_newest_rotation_write(
        &recorded
            .crash_image_with_the_root_of_the_recorded_publish_persisted_and_its_rotation_lost(),
    );
    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut faulted_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.key_of_the_previous_version,
        "R 走读不成、Q 的根读不出：择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_for_the_recorded_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            recorded.key_of_the_recorded_publish.0,
            recorded.key_of_the_recorded_publish.1,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let units_of_the_intermediate_publish_not_in_the_ring_slot_of_the_recorded_publish: Vec<
        &RetainedWrite,
    > = recorded
        .unit_writes_of_the_intermediate_publish()
        .into_iter()
        .filter(|write| !some_placement_covers(&borrowed_for_the_recorded_publish, write))
        .collect();
    assert!(
        units_of_the_intermediate_publish_not_in_the_ring_slot_of_the_recorded_publish.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了 Q 那一槽记着的落点（丙并中间）：Q 写的单元都在里面，缺 {} 份：{units_of_the_intermediate_publish_not_in_the_ring_slot_of_the_recorded_publish:?}",
        units_of_the_intermediate_publish_not_in_the_ring_slot_of_the_recorded_publish.len()
    );
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&recorded.key_of_the_previous_version)
        .cloned()
        .expect("P 是录的池写过的一版");
    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert(
            (root.instance, root.checkpoint_txg),
            content_of_the_selected_version.clone(),
        );
    }
    let session_stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(&image_after_the_mount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        device,
                        session_stream.clone(),
                    ),
                )
            })
            .collect(),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root,
    };
    let mut intermediate_publish_left_the_ring_while_the_recorded_publish_was_in_it = false;
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_RING_SLOT_OF_THE_RECORDED_PUBLISH_IS_WRITTEN_AT_MOST；跨轮带着「Q 先离开根环、R 还在」那一个判断；
    // 出口：R 离开根环的那一步判完就 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites < OVERWRITES_UNTIL_THE_RING_SLOT_OF_THE_RECORDED_PUBLISH_IS_WRITTEN_AT_MOST,
            "{OVERWRITES_UNTIL_THE_RING_SLOT_OF_THE_RECORDED_PUBLISH_IS_WRITTEN_AT_MOST} 次覆盖写之内写掉 R 那一槽（384 槽小盘一圈根环 24 次发布）"
        );
        let image_before_the_step = pool.image();
        let operations_before_the_step = session_stream.operation_count();
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("挂上之后第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1)
        });
        overwrites += 1;
        let (writes_of_the_step, _segments, _stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &session_stream.retained_operations()[operations_before_the_step..],
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
        let last_root_of_the_step = writes_of_the_step
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("一次覆盖写至少写一条根");
        let mut state_before_the_last_root_of_the_step = image_before_the_step;
        state_before_the_last_root_of_the_step
            .apply_writes(&writes_of_the_step[..last_root_of_the_step]);
        images.hand_out(
            &format!(
                "第 {overwrites} 次覆盖写最后一条根槽写之前那一态：被抛弃根引用的单元一个都没复用"
            ),
            || state_before_the_last_root_of_the_step.clone(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
        let image_after_the_step = pool.image();
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后：被抛弃根引用的单元一个都没复用"),
            || image_after_the_step.clone(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
        let recorded_publish_is_in_the_ring =
            root_is_in_the_ring(&image_after_the_step, recorded.key_of_the_recorded_publish);
        if recorded_publish_is_in_the_ring
            && !root_is_in_the_ring(
                &image_after_the_step,
                recorded.key_of_the_intermediate_publish,
            )
        {
            intermediate_publish_left_the_ring_while_the_recorded_publish_was_in_it = true;
        }
        if !recorded_publish_is_in_the_ring {
            break;
        }
    }
    assert!(
        intermediate_publish_left_the_ring_while_the_recorded_publish_was_in_it,
        "有一步之后 Q 已离开根环而 R 还在（这一形的窗口走到了：Q 那一槽先写掉、解开，R 那一槽晚一个 txg）"
    );
}
