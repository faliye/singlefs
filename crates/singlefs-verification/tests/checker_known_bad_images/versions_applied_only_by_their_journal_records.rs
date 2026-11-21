//! 只由日志记录应用的版本：走不走查、分配统计与复用、抬过回退下限与根环转过之后、回链对不上、多条记录的发布与重放停在哪里。

use super::*;

/// 残留记录那一版的内容：与 A、B 都不同长、不同字节（具体哪些字节不承重）。
fn content_applied_only_by_its_journal_record() -> Vec<u8> {
    (0..3700usize)
        .map(|index| u8::try_from((index * 7 + 5) % 253).expect("小于 256"))
        .collect()
}

/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后的镜像（增补 2 收口表第 54 行那段历史，
/// 与 `crash_enumeration_fixed_script_stream.rs` 残留记录那条流里实例 2 的根已落盘的那 12 个状态同形）：
/// mkfs → 取号 → 暖机 → A → B，实例 1 再发一版（txg 5）而**只落了它的单元与 journal 记录、根槽与系统配置槽一个都没落**；
/// 可写挂载择 B 的根 (1, 4)、施加那条记录、给实例 1 写行 (1, 5, 3)、写行发布 txg 6 把 (1, 5) 那一版的固定点单元换下放进 defer、
/// 暖机一次 txg 7。交回挂载前后两份镜像、挂载之后接着发布要用的盘与分配器，以及几个单元的槽（两盘同槽）。
pub(super) struct ImageAfterAVersionAppliedOnlyByItsJournalRecord {
    /// 挂载之前：那一版的单元与记录已在盘上，最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行。
    image_before_the_mount: MemoryPool,
    /// 挂载之后（最新的根是暖机 txg 7）。
    pub(super) image: MemoryPool,
    pub(super) devices: Vec<(
        DeviceIdentity,
        singlefs_verification::RecordingBlockDevice<
            singlefs_verification::memory_pool::SparseBlockDevice,
        >,
    )>,
    pub(super) allocator: singlefs_core::allocator::PoolAllocator,
    pub(super) newest: singlefs_core::transaction::TransactionOutput,
    newest_accounting_root_slot: u64,
    /// 只由记录施加出来的那一版 (1, 5) 自己的树表单元：写行发布 txg 6 把它换下放进 defer，根环里没有一条根引用它。
    tree_table_slot_of_the_version_applied_only_by_its_record: u64,
    /// 只由记录施加出来的那一版 (1, 5) 那次发布的输出（它写了哪些单元、重写了哪几样、那条 journal 记录）。
    version_applied_only_by_its_record: TransactionOutput,
    /// 这次可写挂载写的行（钉住的值是 (1, 5, 3)）。
    pub(super) rows_written: Vec<InstanceRow>,
    /// 这次可写挂载写行那次发布：实例 2 的第一个 txg（钉住的值是 6）与它那条记录的计数器。
    pub(super) row_publish_txg: CheckpointTxg,
    pub(super) row_publish_counter: u64,
}

/// [`writable_mount_over_a_version_applied_only_by_its_journal_record`] 再钉住这次挂载写的行 (1, 5, 3) 与写行发布的 txg 6。
pub(super) fn image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
    tag: &str,
) -> ImageAfterAVersionAppliedOnlyByItsJournalRecord {
    let built = writable_mount_over_a_version_applied_only_by_its_journal_record(tag);
    assert_eq!(
        built.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
        }],
        "写行 (1, 5, 3)"
    );
    assert_eq!(built.row_publish_txg, CheckpointTxg(6), "写行发布 txg 6");
    built
}

/// 同一段历史，不钉这次挂载写的行与写行发布的 txg：拿池级 checker 判这两样的用例（I-3.19、I-8.14）先看判定，
/// 写者把这两样写错时红在池级 checker 那一句上，不先红在钉值上。
pub(super) fn writable_mount_over_a_version_applied_only_by_its_journal_record(
    tag: &str,
) -> ImageAfterAVersionAppliedOnlyByItsJournalRecord {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let operations_through_the_second_publish = pool.retained_operations().len();
    let applied_only_by_its_record = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_applied_only_by_its_journal_record(),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("实例 1 再发一版");
    assert_eq!(
        (
            applied_only_by_its_record.record.instance,
            applied_only_by_its_record.record.checkpoint_txg,
            applied_only_by_its_record.record.transaction
        ),
        (InstanceGeneration(1), CheckpointTxg(5), 3),
        "只由记录施加出来的那一版：实例 1、txg 5、事务号 3"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let units_and_record_of_the_version: Vec<RetainedOperation> = operations
        [operations_through_the_second_publish..]
        .iter()
        .filter(|retained| match classifier.classify(&retained.operation) {
            StepKind::UnitWrite | StepKind::JournalRecord => true,
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => false,
        })
        .cloned()
        .collect();
    let mut image_before_the_mount = pool.memory_pool_after_mkfs();
    image_before_the_mount
        .apply(&operations[pool.mkfs_operation_count..operations_through_the_second_publish]);
    image_before_the_mount.apply(&units_and_record_of_the_version);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image_before_the_mount, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (InstanceGeneration(1), CheckpointTxg(4), CheckpointTxg(5), 1),
        "择 B 的根 (1, 4)，施加那条记录，现行版是 (1, 5)"
    );
    let rows_written = mounted.output.rows_written.clone();
    let row_publish_txg = mounted.output.row_publish.root().checkpoint_txg;
    let row_publish_counter = mounted.output.row_publish.record().counter;
    let newest = mounted
        .current
        .into_file_version()
        .expect("施加了那条记录，现行那一版带文件");
    ImageAfterAVersionAppliedOnlyByItsJournalRecord {
        image_before_the_mount,
        image: memory_pool_of_sparse_devices(&devices),
        devices,
        allocator: mounted.allocator,
        newest_accounting_root_slot: newest.unit(TransactionUnit::AccountingTree).slot.0,
        newest,
        tree_table_slot_of_the_version_applied_only_by_its_record: applied_only_by_its_record
            .unit(TransactionUnit::TreeTable)
            .slot
            .0,
        version_applied_only_by_its_record: applied_only_by_its_record,
        rows_written,
        row_publish_txg,
        row_publish_counter,
    }
}

/// 单元区起点（第一个单元落在这个槽上：mkfs 种的实例表）。
const UNIT_AREA_FIRST_SLOT: u64 = 50176;

/// `units_found_on_device_zero` 从单元区起点往后扫多少个槽：这几份镜像上用到的落点都在前几百个槽里。
const SLOTS_SCANNED_FOR_UNITS: u64 = 1024;

/// 单元区里盘 0 上此刻的单元：(起点槽, 字节数)，按头里的类标签认（码 2 一槽，码 1 / 码 3 两槽）。
/// 镜像是现造出来的、没有写死的单元表时，拿它给沿引用链补校验和（`mutate_unit_of`）当单元表：
/// 父单元（树表、中央映射树的根）里引用被改单元的位置条目都要补上，漏一个就是 I-2.1 跟着红。
pub(super) fn units_found_on_device_zero(image: &MemoryPool) -> Vec<(u64, usize)> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mut units = Vec::new();
    let mut slot = UNIT_AREA_FIRST_SLOT;
    while slot < UNIT_AREA_FIRST_SLOT + SLOTS_SCANNED_FOR_UNITS {
        let header = read(image, 0, slot * SLOT, 512);
        let slots_taken = if &header[..4] == b"SFSU" {
            match header[6] {
                2 => {
                    units.push((slot, node_bytes));
                    1
                }
                1 | 3 => {
                    units.push((slot, 2 * node_bytes));
                    2
                }
                _unregistered_unit_class => 1,
            }
        } else {
            1
        };
        slot += slots_taken;
    }
    units
}

/// 增补 2 收口表第 54 行（候选 b）的两半，建在同一份镜像上：
/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）在 defer 里、仍算已分配，
///    根环里没有一条根引用它们；checker 把那一版并进遍历之后 I-3.1（已分配统计对得上） 真被评估过且成立，别的也一条不红。
///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、盘 0 上差 131 072 字节（8 槽；`crates/mutations.tsv` 里 `take(0)` 那条变异下的量）。
/// ② **改完仍会红**：同一份镜像上记账盘 0 再泄漏一槽（「已分配」+16384、「空闲」−16384），I-3.1 照样红
///    （I-3.11（已分配减 defer 等于最新根走读） 跟着红：defer 行没动），别的不红，
///    说明文字里的机理标识报出那一版确实并进了遍历——补上的只是等式右边缺的那一版，没把泄漏一起吞掉。
#[test]
fn version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_the_leak_on_top_still_reddens_it(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record",
    );
    let verdicts_on_the_clean_image = check_pool_image(&built.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "接在只由记录施加出来的那一版之后的合法镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&built.image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
    let mut leaked = built.image.clone();
    mutate_unit_of(
        &mut leaked,
        &units_found_on_device_zero(&built.image),
        built.newest_accounting_root_slot,
        |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
        true,
    );
    let verdicts = check_pool_image(&leaked);
    // defer 行没跟着抬：已分配减 defer 比最新根走读多一槽，I-3.11（已分配减 defer 等于最新根走读） 跟着红（同 Z3-A）。
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.1", "I-3.11"],
        "再泄漏一槽只该红 I-3.1 与 I-3.11：{verdicts:?}"
    );
    // I-3.16 跟着报部分没判（已分配那一行对不上占着的槽那一份并集），不报违例。
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_on_the_clean_image,
            &[
                "I-3.1",
                "I-3.11",
                FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN
            ]
        ),
        [],
        "I-3.1、I-3.11 与 I-3.16 之外每一条的判定与泄漏之前逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&leaked, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains("并进遍历的由记录施加出来的版本 1 个"),
        "机理标识要报出那一版并进了遍历：{detail}"
    );
}

/// 候选 b 那一版的第 ① 条（「它被施加过」要看实例表里有没有这个实例的行）：同一段历史**挂载之前**那一刻，
/// 那一版的单元与记录已在盘上、最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行——施加要等下一次挂载，这一刻它不算一版。
/// 这一刻它的单元是孤儿、不在记账里；把它并进遍历就是遍历多算，I-3.1（已分配统计对得上） 红
/// （`crash_enumeration_fixed_script_stream.rs` 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格）。
#[test]
fn journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds() {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-record-not-applied-yet",
    );
    let verdicts = check_pool_image(&built.image_before_the_mount);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "挂载之前那一刻一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&built.image_before_the_mount, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 只由记录施加出来的那一版在候选集里：它引用的单元照候选根一样判，I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）
/// 按这一版各判一格。把它自己的树表单元（只有它引用，在 defer 里）两份都抹成 0：从它出发的遍历读不到那个单元，
/// 这两条必须红——根环里的根一条都不引用它，按根判的那几格判不出来。
#[test]
fn erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-version-applied-only-by-its-record-erased",
    );
    let mut image = built.image.clone();
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    for device in DEVICES {
        write(
            &mut image,
            device,
            built.tree_table_slot_of_the_version_applied_only_by_its_record * SLOT,
            &vec![0u8; node_bytes],
        );
    }
    let verdicts = check_pool_image(&image);
    let violated = violated_invariants(&verdicts);
    for invariant in ["I-7.4", "I-4.8"] {
        assert!(
            violated.contains(&invariant),
            "那一版的树表被抹掉，{invariant} 要红：{verdicts:?}"
        );
    }
    assert!(
        !violated.contains(&"I-7.2"),
        "最新的根不引用那个单元，I-7.2（最新根能完整走完） 不该跟着红：{verdicts:?}"
    );
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 射程 ③：记录罩住的起点槽上读不出可用的单元头时这一格没有对象。
/// 发布 B 之后的干净镜像上把 B 的数据单元两份头里的诞生代号改成 99、**不重封**头校验和（盘上坏了的样子）：
/// 那条记录不判，I-3.10 照旧由别的记录判成成立；读不出本身由 I-2.1 / I-2.4 说话。
/// 把「头校验和过」那一道从「可用」里拿掉，这里就按坏头里的 99 判红。
#[test]
fn an_allocation_record_whose_start_slot_header_does_not_verify_is_not_judged_by_the_allocation_generation_invariant(
) {
    let mut image = image_after_the_overwrite("known-good-allocation-generation-unusable-header");
    let data_unit_bytes = 2 * usize::try_from(NODE_BYTES).expect("16384");
    for device in DEVICES {
        let mut unit = read(
            &image,
            device,
            DATA_UNIT_AFTER_OVERWRITE * SLOT,
            data_unit_bytes,
        );
        assert_eq!(
            get_u64(&unit, DATA_UNIT_BIRTH_TXG_OFFSET),
            4,
            "B 的数据单元诞生代号本来是 4"
        );
        set_u64(&mut unit, DATA_UNIT_BIRTH_TXG_OFFSET, 99);
        write(&mut image, device, DATA_UNIT_AFTER_OVERWRITE * SLOT, &unit);
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdict(&image, "I-3.10"),
        InvariantVerdict::Holds,
        "起点槽的头校验和不过的那条记录不判，别的未释放记录照判且成立：{verdicts:?}"
    );
    assert!(
        violated_invariants(&verdicts).contains(&"I-2.1"),
        "单元坏了由 I-2.1 说话：{verdicts:?}"
    );
}

/// 抬 F 之前实例 2 接着发几版：发到 txg 10。准入抬 F 的上限（算法见 `crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor`）
/// 被读回全零的 txg 5 那一槽压在 5；正常卸载把 F 抬到 10（卸载抬 F 不另判上限）。
const OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION: u64 = 3;

/// 实例 2 在同一次挂载里接着覆盖写 `rounds` 次，交回最后那一版。
fn overwrite_in_the_second_instance(
    built: &mut ImageAfterAVersionAppliedOnlyByItsJournalRecord,
    rounds: u64,
    write_time_offset_seconds: u64,
) -> singlefs_core::transaction::TransactionOutput {
    let publish_parameters = parameters();
    let mut previous = built.newest.clone();
    for round in 0..rounds {
        let round_index = usize::try_from(round).expect("轮次");
        let content: Vec<u8> = (0..(2000 + round_index * 37))
            .map(|index| u8::try_from((index * 11 + round_index) % 251).expect("小于 256"))
            .collect();
        let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
        previous = publish_overwrite(
            &mut writer,
            &mut built.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset_seconds + round,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 接着覆盖写");
    }
    previous
}

/// 候选 b 那一版的第 ③ 条（「不低于回退下界」）：F 抬过 (1, 5) 之后它就不再并进遍历。
/// 同一段历史在实例 2 里接着发三版（txg 8–10），再正常卸载、把 F 抬到现行那一版的 txg 10：抬 F 生效之后回收释放代 ≤ 10 的落点，
/// (1, 5) 那一版被写行发布 txg 6 换下的单元（释放代 6）就在其中——不再算已分配。把它照旧并进遍历就是遍历多算，I-3.1 红。
/// 环里比它老的有效根还在（txg 1–4；卸载那一串空发布落在环里没写过的槽上），第 ④ 条拦不住这一格，拦它的只有第 ③ 条。
/// 抬 F 走卸载那一串，不走准入：txg 5 那一槽的根从没落盘、读回全零，D16（发布语义） 已定项 1「抬 F 的上限」那一行把读回全零的槽并进准入抬 F 的上限
/// ，这一槽住得下的最大 txg 是 5，准入抬 F 越不过 5（抬到 6 报 `RollbackFloorAboveCeiling { requested: 6, ceiling: 5 }`
/// ）；那一槽到环转一圈才会被写掉，那时 txg 1–4 那几条根也已被盖掉、第 ④ 条跟着拦。
/// 正常卸载把 F 抬到现行那一版的 txg、不另判上限（同一行「卸载抬 F」），合法地越过 5。
#[test]
fn version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-below-the-floor",
    );
    let last_version_before_the_unmount = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        last_version_before_the_unmount.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 2 发到 txg 10"
    );
    let mut current = PoolVersion::WithFile(Arc::new(last_version_before_the_unmount));
    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = unmount(
        &parameters(),
        &mut built.devices,
        &mut built.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载") else {
        panic!("现行那一版带文件：卸载抬 F");
    };
    assert_eq!(
        unmounted.rollback_floor,
        CheckpointTxg(10),
        "卸载把 F 抬到现行那一版的 txg 10，越过 (1, 5)"
    );
    assert!(
        unmounted.reclaimed.iter().any(|placement| placement.slot.0
            == built.tree_table_slot_of_the_version_applied_only_by_its_record),
        "(1, 5) 那一版的树表单元（释放代 6）在抬 F 生效之后被回收：{:?}",
        unmounted.reclaimed
    );
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "F 抬过 (1, 5) 之后一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 由记录施加出来的那一版照它施加在其上那一版的根走实例表（D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」）：
/// 同一段历史在实例 2 里接着发三版（txg 8–10），再把 F 抬到 5——它施加在其上的 B (1, 4) 掉到 F 之下、
/// 不在回退候选集里，(1, 5) 自己还在（txg 5 ≥ F）。实例 1 那张实例表（mkfs 写的第 0 片，槽 50176）只有实例 1 的根与 (1, 5) 这一版引用，
/// 写行发布 txg 6 换下它、释放代 6 > max(F 5, 环里最旧有效根 1)，抬 F 之后仍算已分配。checker 走 (1, 5) 时照 B 的根记录走它的实例表，
/// I-3.1（已分配统计对得上） 对得上、一条都不红。只走记录新根段里的树表与映射根的话，那一片 32768 字节没人数，I-3.1 红
/// （崩溃注入默认批种子 7463871032432355136 第 91 段那两条同形，`research/prompts/m2-investigate-crash-injection-reds-report.md` 第三节）。
#[test]
fn version_applied_only_by_its_journal_record_walks_the_instance_table_of_the_root_it_was_applied_on_even_below_the_floor(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-base-below-the-floor",
    );
    let mut current = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 2 发到 txg 10"
    );
    singlefs_core::mount::raise_rollback_floor(
        &parameters(),
        &mut built.devices,
        &mut built.allocator,
        &mut current,
        CheckpointTxg(5),
        ShadowLedger::On,
    )
    .expect("抬 F 到 5");
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "B 掉到 F 之下、(1, 5) 还在：一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// journal 环里计数器 `counter` 那条记录（两块盘各一份）的反向链字段改掉（最低位翻过来）、重封头校验和：
/// 写者写错了链，两盘写出同一份错的记录。它的载荷（点名项）一个字节不动。
fn break_the_back_chain_of_journal_record(pool: &mut MemoryPool, counter: u64) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(pool, device, offset, 4096);
        record[JOURNAL_RECORD_BACK_CHAIN_OFFSET] ^= 1;
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(pool, device, offset, &record);
    }
}

/// I-8.6（反向链算法）「不等的记录不进重放前缀」，checker 的前缀口径跟着恢复对齐（恢复补反向链检查）：
/// 可写挂载施加过 (1, 5) 那条记录、写了行的那份镜像上，把那条记录两盘各一份的反向链改坏。
/// 恢复照 I-8.6 不施加它，checker 也不再把它当「被恢复施加过的那一版」并进遍历：它换下的固定点单元仍记在已分配里、没人走到；
/// I-8.6 在那条记录上红，别的不红。I-3.1（已分配统计对得上） 照 C697（已分配统计在短环加大文件上误红） 换成三判之后成立：记账多于遍历落在上沿之内
/// （改三判之前它红在记账多的一边，机理标识报并进遍历的版本 0 个，这条用例靠它分辨并没并进）。三判之下并不并进在这张镜像上判得一样，
/// 另把那一版自己的树表两份清零（根环里没有一条根引用它）：没并进就没人去读它，照旧只红 I-8.6；并进了就要从它出发走、读不出，
/// 红 I-2.1、I-4.8、I-7.4（同 [`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`] 那一形）。
/// checker 认版本时若不看链，照旧并进。
#[test]
fn a_journal_record_whose_back_chain_does_not_match_is_not_walked_as_a_version_a_recovery_applied()
{
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-version-applied-only-by-a-record-with-a-broken-back-chain",
    );
    let mut image = built.image.clone();
    let (counter, _, _) = journal_records_on_device(&image, 0)
        .into_iter()
        .find(|(_, instance, transaction)| *instance == 1 && *transaction == 3)
        .expect("环里有实例 1、事务号 3 的那条记录（只由记录施加出来的 (1, 5)）");
    break_the_back_chain_of_journal_record(&mut image, counter);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-8.6"],
        "只该红 I-8.6：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "那一版没并进遍历、记账多于遍历落在三判的上沿之内：I-3.1 真被评估过且成立"
    );
    let mut image_with_the_version_tree_table_zeroed = image.clone();
    zero_both_copies_of_the_unit(
        &mut image_with_the_version_tree_table_zeroed,
        built.tree_table_slot_of_the_version_applied_only_by_its_record,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts_with_the_version_tree_table_zeroed =
        check_pool_image(&image_with_the_version_tree_table_zeroed);
    assert_eq!(
        violated_invariants(&verdicts_with_the_version_tree_table_zeroed),
        ["I-8.6"],
        "那一版没并进遍历、没人读它的树表：清零之后照旧只该红 I-8.6：{verdicts_with_the_version_tree_table_zeroed:?}"
    );
}

/// 同一个前缀口径的另一处（实审 A2a 报告那一节第 3 条）：「下一次挂载会先施加的那一版」。发布 B 的记录已落、根槽没落的崩溃镜像上，
/// B 那棵分配记录树里 B 的数据单元的分配代改成诞生代号 + 1（这一步单红 I-3.10，见上一条用例），再把 B 那条记录的反向链改坏：
/// 下一次挂载的恢复照 I-8.6 不施加它，checker 不再读它那一版的分配记录——只红 I-8.6，I-3.10 由候选集里 A 那一版判成立。
/// checker 取那一版时若不看链，I-3.10 照红。
#[test]
fn a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first() {
    let mut image = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-next-mount-version-with-a-broken-back-chain",
    );
    raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(&mut image);
    let (counter, _, _) = journal_records_on_device(&image, 0)
        .into_iter()
        .find(|(_, instance, transaction)| *instance == 1 && *transaction == 2)
        .expect("环里有 B 那条记录（实例 1、事务号 2）");
    break_the_back_chain_of_journal_record(&mut image, counter);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-8.6"],
        "B 那条记录进不了前缀：只该红 I-8.6：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.10"),
        InvariantVerdict::Holds,
        "I-3.10 由候选集里 A 那一版判成立"
    );
}

/// 环转过 (1, 5) 那一版之前实例 2 在同一次挂载里接着发几版：从 txg 8 起到 txg 29 为止，环里每个槽都被 txg ≥ 6 的根盖过
/// （根环 24 槽：三个区域各 8 槽，槽位按 txg 轮换）。
const OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION: u64 = 22;

/// 候选 b 那一版的第 ④ 条（「它换下的单元还没到回收的时候」）：环转过去之后它就不再并进遍历。
/// 同一段历史在实例 2 里接着发 22 版（txg 8–29），环里最旧的有效根变成比 (1, 5) 新；再重开可写挂载（实例 3），
/// 重建分配器按「释放代 ≤ max(F_生效, 环里最旧有效根)」（D3（空间分配） 已定项 7）把那一版换下的单元（释放代 6）收回去，
/// 再发一版。这时那一版的单元不再算已分配、可能已被复用；把它照旧并进遍历就是拿已经不归它的槽去比。
///
/// 实例 2 在同一次挂载里转过根环时就按谓词回收（C518（一次挂载之内环转过一圈之后不回收） 修好之后），每一版的记账行按回收之后的数写，
/// 这一份镜像上一条都不红：那一版的单元已回收、不再算已分配，第 ④ 条让它不并进遍历，记账与遍历对得上。
/// 把第 ④ 条拿掉，那一版照旧并进遍历，遍历多出它那几个已回收的槽，I-3.1（已分配统计对得上） 红。
#[test]
fn version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_the_mount_reclaimed_its_units(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-after-the-ring-turned",
    );
    let publish_parameters = parameters();
    let previous = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        previous.root.checkpoint_txg,
        CheckpointTxg(29),
        "实例 2 发到 txg 29，环里每个槽都被 txg ≥ 6 的根盖过"
    );
    let remounted = mount_writable(&publish_parameters, &mut built.devices).expect("重开可写挂载");
    let mut allocator = remounted.allocator;
    let remounted_current = remounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut allocator,
        &remounted_current,
        FirstFile {
            content: &content_applied_only_by_its_journal_record(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
        },
        remounted.output.instance,
    )
    .expect("重开之后再发一版");
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "环转过 (1, 5) 之后那一版不再并进遍历，一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 发布 B（txg 4）的记录已落盘、根槽没落盘的崩溃镜像：B 的十二个单元、两块盘上各一份的记录与记录之后那道屏障都在，
/// 崩在根槽 FUA 之前（Y4-a 那三个前缀里的最后一个，`research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y4）。
/// 最新的根还是 A 的 (1, 3)，实例表里没有实例 1 的行；B 那一版只在它那条记录的新根段里，下一次挂载的恢复由记录重建它
/// （D23（journal 的角色与格式） 已定项 15）。第二次写的内容与 `image_after_the_overwrite` 同一份，B 的落点就是 `UNITS_AFTER_OVERWRITE`。
pub(super) fn crash_image_with_the_overwrite_record_but_not_its_root(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let operations_through_the_first_publish = pool.retained_operations().len();
    let previous = pool.output.clone();
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    assert_eq!(
        second.root.checkpoint_txg,
        CheckpointTxg(4),
        "发布 B 是 txg 4"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let kinds_of_the_overwrite: Vec<StepKind> = operations[operations_through_the_first_publish..]
        .iter()
        .map(|retained| classifier.classify(&retained.operation))
        .collect();
    let root_record_step = kinds_of_the_overwrite
        .iter()
        .position(|kind| *kind == StepKind::RootRecordFua)
        .expect("发布 B 有一步根槽 FUA");
    assert_eq!(
        (
            kinds_of_the_overwrite[..root_record_step]
                .iter()
                .filter(|kind| **kind == StepKind::JournalRecord)
                .count(),
            kinds_of_the_overwrite[root_record_step - 1]
        ),
        (2, StepKind::Barrier),
        "根槽 FUA 之前：B 那条记录两块盘各写了一份，紧挨着根槽的是记录之后那道屏障"
    );
    let mut image = pool.memory_pool_after_mkfs();
    image.apply(
        &operations
            [pool.mkfs_operation_count..operations_through_the_first_publish + root_record_step],
    );
    image
}

/// journal 记录头里新根段「树表 0 条那一版的分配记录树根」那 86 字节：新根段从 99 起，树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
/// 实例表指针 86 之后（D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表）。
const JOURNAL_RECORD_NEW_ALLOCATION_RECORD_TREE_ROOT_RANGE: std::ops::Range<usize> = 373..459;

/// C706（记录单落那一版的分配记录树根没有判据），判掉：发布 B 的记录已落、根槽没落的崩溃镜像上，把 B 那条记录
/// （计数器 4）新根段里「树表 0 条那一版的分配记录树根」换成更早一版的一条非空分配记录树根（A 那一版树表里种类 3 那一条的根指针，原样 86 字节），
/// 两盘各一份、重封头校验和。攻方 H4 换的是写行那一版的那一条；这条流上写行与暖机都是空发布、新根段那一项全零，A 的那一棵是镜像上现成的
/// 另一棵合法的分配记录树。B 那一版带树（树表有条目），这一项只许全零：池级 checker 在只在记录里的那一版上判 I-5.4 那一格
/// 「一条根只有一棵分配记录树」红（`walk::judge_allocation_records_disjoint`，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md`
/// 第二节 P2 起由记录施加出来的版本与下一次挂载先施加的那一版也判；这张镜像上第一处违例点名的是后一类，B 就是下一次挂载先施加的那一版）。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：那一格恒判成立时 checker 读不出这一项写错了（攻方 H4 量到的样子），这条红。
#[test]
fn non_zero_allocation_record_tree_root_in_the_record_of_a_version_with_trees_only_its_record_carries_reddens_the_allocation_record_invariant(
) {
    let mut image = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-allocation-record-tree-root-in-the-record-of-the-version-only-its-record-carries",
    );
    let tree_table_of_the_earlier_publish = read(&image, 0, TREE_TABLE * SLOT, 16384);
    let allocation_entries: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
        .map(tree_table_entry_offset)
        .filter(|entry| {
            u16::from_le_bytes([
                tree_table_of_the_earlier_publish[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                tree_table_of_the_earlier_publish[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
            ]) == TREE_KIND_ALLOCATION
        })
        .collect();
    let [allocation_entry] = allocation_entries.as_slice() else {
        panic!("A 的树表里种类 3 的条目恰好一条：{allocation_entries:?}")
    };
    let allocation_record_tree_root_of_the_earlier_publish = tree_table_of_the_earlier_publish
        [allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.start
            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.end]
        .to_vec();
    assert!(
        allocation_record_tree_root_of_the_earlier_publish
            .iter()
            .any(|byte| *byte != 0),
        "A 那一版的分配记录树根非空"
    );
    let ring_bytes = singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    for device in DEVICES {
        let offset = record_offset(LAST_COUNTER_AFTER_THE_OVERWRITE, ring_bytes).0;
        let mut bytes = read(&image, device, offset, 4096);
        assert_eq!(
            (
                get_u64(&bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
                bytes[JOURNAL_RECORD_NEW_ALLOCATION_RECORD_TREE_ROOT_RANGE]
                    .iter()
                    .all(|byte| *byte == 0)
            ),
            (2, true),
            "盘 {device} 计数器 4 那条是 B 的记录，带树的一版这一项全零"
        );
        bytes[JOURNAL_RECORD_NEW_ALLOCATION_RECORD_TREE_ROOT_RANGE]
            .copy_from_slice(&allocation_record_tree_root_of_the_earlier_publish);
        let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
        assert!(
            JournalRecord::parse(
                &bytes,
                get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET)
            )
            .is_some(),
            "盘 {device} 改过的 B 那条记录照旧自证"
        );
        write(&mut image, device, offset, &bytes);
    }
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!(
            "B 那一版只在记录里、新根段带了第二棵分配记录树：I-5.4 要红，实际 {:?}",
            verdict(&image, "I-5.4")
        );
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）的树表有")
            && detail.ends_with("这一版有两棵分配记录树"),
        "红在只在记录里的 B 那一版（下一次挂载先施加的就是它）、两棵分配记录树那一格：{detail}"
    );
}

/// 每个单元（两盘同内容，取盘 0 那一份）此刻的整单元校验和：沿引用链补过哪几个单元，前后一比就知道。
fn unit_checksums(pool: &MemoryPool, units: &[(u64, usize)]) -> Vec<(u64, u32)> {
    units
        .iter()
        .map(|(slot, length)| {
            (
                *slot,
                crc32_castagnoli(&read(pool, 0, slot * SLOT, *length)),
            )
        })
        .collect()
}

/// 几个单元的整单元校验和变了（`changes`：(起点槽, 旧, 新)）：journal 环里引用它们的位置条目——记录新根段里的树表指针与映射根指针、
/// 点名项——逐条补上，补过的记录先重算载荷校验和、再重封记录头校验和（罩整条 4096，字段在 46）。返回补了几条记录（按盘各算一条）。
/// 根槽没落盘的那一版只在记录里，`propagate` 补到根槽为止、够不着它。
fn propagate_into_journal_records(pool: &mut MemoryPool, changes: &[(u64, u32, u32)]) -> usize {
    let mut records_touched = 0;
    for device in DEVICES {
        for (counter, _, _) in journal_records_on_device(pool, device) {
            let offset = singlefs_core::journal::record_offset(
                counter,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            let mut record = read(pool, device, offset, 4096);
            let mut touched = false;
            for (slot, old, new) in changes {
                for location_device in DEVICES {
                    touched |= replace_all(
                        &mut record,
                        &location_pattern(location_device, *slot, *old),
                        &location_pattern(location_device, *slot, *new),
                    );
                }
            }
            if !touched {
                continue;
            }
            let named_count = usize::try_from(u32::from_le_bytes(
                record[JOURNAL_RECORD_NAMED_COUNT_OFFSET..JOURNAL_RECORD_NAMED_COUNT_OFFSET + 4]
                    .try_into()
                    .expect("4 字节"),
            ))
            .expect("点名项数");
            let payload_end = JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * named_count;
            let payload_checksum =
                crc32_castagnoli(&record[JOURNAL_RECORD_HEADER_BYTES..payload_end]);
            record[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET
                ..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
                .copy_from_slice(&payload_checksum.to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
            record[46..78].copy_from_slice(&digest);
            write(pool, device, offset, &record);
            records_touched += 1;
        }
    }
    records_touched
}

/// Y4-a 的坏法：B 那棵分配记录树里 B 的数据单元那两条未释放记录（两盘各一条），分配代从 4 改成诞生代号 + 1 = 5，
/// 单元一个字节不动；新的整单元校验和沿引用链补到 B 的树表、中央映射，再补进 B 那条记录（两块盘各一份）。
/// 盘上留下的就是攻方那条分配器变异（只在发布 B 那一次把新记的分配代写成 txg + 1）崩在根槽之前的样子。
fn raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(
    image: &mut MemoryPool,
) {
    let checksums_before = unit_checksums(image, &UNITS_AFTER_OVERWRITE);
    assert_eq!(
        mutate_allocation_leaves(
            image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| rewrite_unreleased_allocation_generation(
                bytes,
                DATA_UNIT_AFTER_OVERWRITE,
                4,
                5
            ),
        ),
        2,
        "B 的数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 4"
    );
    let checksums_after = unit_checksums(image, &UNITS_AFTER_OVERWRITE);
    let changes: Vec<(u64, u32, u32)> = checksums_before
        .iter()
        .zip(&checksums_after)
        .filter(|((_, before), (_, after))| before != after)
        .map(|((slot, before), (_, after))| (*slot, *before, *after))
        .collect();
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        2,
        "B 那条记录两块盘各一份，新根段里的树表指针都要补"
    );
}

/// 改法 E（`research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y4、第四节第 2 条，被攻过零轮）：
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的读集再加「下一次挂载会先施加的那一版」——与最新根同实例、
/// txg = 最新根 + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树。
///
/// 发布 B 的记录已落、根槽没落的崩溃镜像上，候选集（A 的根及更早）与「由记录施加出来的那几版」（实例表里还没有实例 1 的行）
/// 都读不到 B 那棵分配记录树，只有 E 那一步读得到：
/// ① 干净的崩溃镜像上一条都不红，I-3.10 真被评估过且成立（E 不在合法镜像上误报）；
/// ② B 的数据单元那两条未释放记录的分配代改成诞生代号 + 1，崩溃镜像上**只**红 I-3.10、别的判定与干净镜像逐项相同——
///    E 之前的读法在这张镜像上判成立；
/// ③ 这张坏镜像可写挂载在任何写之前被拒：挂载重建上一版时判分配记录的代与跨度（实审 A3a，`rebuild_version` 调
///    `allocation_record_generations_and_spans_are_judged`），分配代 5 晚于那一版的根，报「E142 走读同款」那一句；
///    两块盘逐字节不变、录制流一步没多。checker 在挂载之前红的，正是下一次挂载会拒掉的那一版。
#[test]
fn an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-allocation-generation-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, "I-3.10"),
        InvariantVerdict::Holds,
        "干净的崩溃镜像上 I-3.10 要真被评估过且成立"
    );

    let mut image = clean.clone();
    raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.10"],
        "B 那一版只在记录里时，它的分配代写错要在崩溃镜像上就红、只红 I-3.10：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-3.10"]),
        [],
        "I-3.10 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.10") else {
        panic!("上面判过 I-3.10 红");
    };
    assert!(
        detail.contains(&format!("槽 {DATA_UNIT_AFTER_OVERWRITE} "))
            && detail.contains("分配代 5")
            && detail.contains("诞生代号是 4"),
        "红在 B 的数据单元那条记录上：分配代 5、单元头里的诞生代号 4：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
    let recorded_operations_before_the_mount = stream.operation_count();
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            })
        ),
        "重建上一版时分配代 5 晚于那一版的根，可写挂载要拒在分配记录的代 / 跨度这一条上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
    assert_eq!(
        stream.operation_count(),
        recorded_operations_before_the_mount,
        "拒在任何写之前：录制流一步没多（一个写、一道屏障都没发）"
    );
}

/// 前后两份整单元校验和（[`unit_checksums`]）里变了的那几个单元：(起点槽, 旧, 新)，交给 [`propagate_into_journal_records`]。
fn units_whose_checksum_changed(
    before: &[(u64, u32)],
    after: &[(u64, u32)],
) -> Vec<(u64, u32, u32)> {
    before
        .iter()
        .zip(after)
        .filter(|((_, old), (_, new))| old != new)
        .map(|((slot, old), (_, new))| (*slot, *old, *new))
        .collect()
}

/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」
/// 那一行，攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 b3 形）：I-5.4 按版本那一层也判下一次挂载先施加的那一版。
/// 发布 B 的记录已落、根槽没落的崩溃镜像上，B 那棵分配记录树里 A 的 inode 树根那个落点（槽 50244）的记录跨度改成 2
/// （与 [`known_bad_images_of_overlapping_allocation_records`] 第一份同一种改法，两盘各一条），新的整单元校验和沿引用链补到 B 的树表，
/// 再补进 B 那条记录（两块盘各一份）：
/// ① 只红 I-5.4、别的判定与干净的崩溃镜像逐项相同，违例点名「下一次挂载先施加的那一版（实例 1、txg 4）」——这一版若整版不进
///    I-5.4，这张镜像就一条都不红；
/// ② 可写挂载重建分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上，两块盘逐字节不变：I-5.4 立条时要拦的
///    「下一次可写挂载重建分配器时两条记录罩住同一个槽」就是这一版。
#[test]
fn two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-overlapping-records-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净的崩溃镜像上 I-5.4 要真被评估过且成立"
    );
    let mut image = clean.clone();
    let checksums_before = unit_checksums(&image, &UNITS_AFTER_OVERWRITE);
    assert_eq!(
        mutate_allocation_leaves(
            &mut image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
        ),
        2,
        "A 的 inode 树根那一个落点，B 那棵账里两盘各一条"
    );
    let changes = units_whose_checksum_changed(
        &checksums_before,
        &unit_checksums(&image, &UNITS_AFTER_OVERWRITE),
    );
    assert_eq!(
        propagate_into_journal_records(&mut image, &changes),
        2,
        "B 那条记录两块盘各一份，新根段里的树表指针都要补"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "B 那一版只在记录里时，它的两条记录罩住同一个槽要在崩溃镜像上就红、只红 I-5.4：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-5.4"]),
        [],
        "I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树")
            && detail.contains(&format!("槽 {INODE_ROOT} 跨 2 的记录")),
        "红在 B 那一版（下一次挂载先施加的那一版）的分配记录树上、槽 {INODE_ROOT} 跨 2 那一条：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "同一块盘上两条分配记录罩住同一个槽",
            })
        ),
        "重建 B 那一版的分配器时两条记录罩住同一个槽，可写挂载要拒在这一条上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
/// 两条罩住同一个槽（I-5.4 那一行的 Z1-d 形态）。返回改了几条（0 或 1）。
fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
    bytes: &mut [u8],
) -> usize {
    if bytes[INDEX_NODE_LEVEL_OFFSET] != 0 || bytes[INDEX_NODE_KEY_WIDTH_OFFSET] != 10 {
        return 0;
    }
    let record_count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let record_at = |leaf_bytes: &[u8], record_index: usize| {
        let start = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * record_index;
        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4 字节"));
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
        (start, device, u64::from_le_bytes(slot_bytes), span_field)
    };
    for record_index in 0..record_count.saturating_sub(1) {
        let (start, device, slot, span_field) = record_at(bytes, record_index);
        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
        if device == next_device && slot + u64::from(span_slots) == next_slot {
            set_u16(
                bytes,
                start + 10,
                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1),
            );
            return 1;
        }
    }
    0
}

/// 改 `version` 这次发布自己重写的第一片能改的分配记录叶（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]），
/// 叶重封、新的整单元校验和沿引用链补上去（叶 → 内部节点 → 根 → 树表），再补进那次发布的记录：`record_copies_of_the_publish` 是
/// 那次发布的记录条数乘两块盘（一条记录的发布是 2，三条记录的是 6），只有它们引用这一版改过的单元。交回改的是哪一片叶。
/// 例：一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版 (1, 5) 之后的镜像上
/// （[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）改 (1, 5) 那一版。
fn overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
    image: &mut MemoryPool,
    version: &TransactionOutput,
    record_copies_of_the_publish: usize,
) -> u64 {
    let units = units_found_on_device_zero(image);
    let checksums_before = unit_checksums(image, &units);
    let leaf = version
        .units
        .iter()
        .filter(|unit| {
            version.rewritten.contains(&unit.identity)
                && matches!(
                    unit.identity,
                    TransactionUnit::AllocationTree
                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                )
        })
        .map(|unit| unit.slot.0)
        .find(|slot| {
            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
                &mut candidate_leaf,
            ) == 1
        })
        .expect("这次发布重写的分配记录叶里有一对同盘、首尾相接的记录");
    mutate_unit_of(
        image,
        &units,
        leaf,
        |bytes| {
            assert_eq!(
                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                1,
                "上面挑叶时这一片改得动"
            );
        },
        true,
    );
    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        record_copies_of_the_publish,
        "只有这次发布的记录（两块盘各一份）引用这一版改过的单元"
    );
    leaf
}

/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2，攻方报告
/// `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a3 形）：I-5.4 按版本那一层也判由记录施加出来的那几版。
/// 挂载之后 (1, 5) 只由记录施加出来（根环里没有它的根），它自己重写的一片分配记录叶里两条记录罩住同一个槽：红 I-5.4，
/// 违例点名「由记录施加出来的那一版（实例 1、txg 5）」。这一版若整版不进 I-5.4，这张镜像一条都不红。
/// 改宽的那条记录未释放、跨 3 槽，罩住的单元那一类不是三槽：同时是 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，
/// C411（分配记录的跨度段与单元类别不符没检查）） 的违例，红的恰是这两条。
#[test]
fn two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-overlapping-records-of-a-version-applied-only-by-its-record",
    );
    let verdicts_on_the_clean_image = check_pool_image(&built.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "挂载之后的干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    let mut image = built.image.clone();
    let leaf = overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
        &mut image,
        &built.version_applied_only_by_its_record,
        2,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [
            "I-5.4",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ],
        "(1, 5) 的叶 {leaf} 里两条记录罩住同一个槽：红 I-5.4 与改宽那条记录自己的 I-3.18：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("由记录施加出来的那一版（实例 1、txg 5）那棵分配记录树"),
        "红在由记录施加出来的那一版 (1, 5) 的分配记录树上：{detail}"
    );
}

/// 改法 RV 读不出的两形（攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a1、a2 形，F1、33-c 的同形）：
/// 挂载之后把只由记录施加出来的那一版 (1, 5) 的一个单元两份清零，判红的照常是那几条（`violated`），I-5.4 不再报成立、
/// 报部分没判并点名「由记录施加出来的那一版（实例 1、txg 5）」（`partially_judged_detail`）。
fn assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
    tag: &str,
    slot_of: impl Fn(&ImageAfterAVersionAppliedOnlyByItsJournalRecord) -> u64,
    violated: &[&str],
    partially_judged_detail: &str,
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(tag);
    assert_eq!(
        verdict(&built.image, "I-5.4"),
        InvariantVerdict::Holds,
        "挂载之后的干净镜像上 I-5.4 成立"
    );
    let mut image = built.image.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        slot_of(&built),
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(partially_judged_detail.to_owned()),
        "(1, 5) 那一版读不出的那一格：别的版本的树走到了叶不替它计成立，I-5.4 报部分没判、点名它是由记录施加出来的那一版"
    );
}

/// a1 形：(1, 5) 的树表两份清零。那一版的单元没走到、遍历少算：I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判，
/// 记账多于遍历落在上沿之内，不红（改三判之前红在这一边）。
#[test]
fn the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-tree-table-of-a-version-applied-only-by-its-record-zeroed",
        |built| built.tree_table_slot_of_the_version_applied_only_by_its_record,
        &["I-2.1", "I-4.8", "I-7.4"],
        "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
    );
}

/// a2 形：(1, 5) 的分配记录树根两份清零。
#[test]
fn the_allocation_record_tree_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-allocation-root-of-a-version-applied-only-by-its-record-zeroed",
        |built| {
            assert!(
                built
                    .version_applied_only_by_its_record
                    .rewritten
                    .contains(&TransactionUnit::AllocationTree),
                "(1, 5) 重写了自己的分配记录树根，不与别的版本共用"
            );
            built
                .version_applied_only_by_its_record
                .unit(TransactionUnit::AllocationTree)
                .slot
                .0
        },
        &["I-2.1", "I-4.8", "I-7.4"],
        "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
    );
}

// ─── 由记录造出来的两类版本取哪条记录的新根段（FL）、树表指针全零（Q1、被抛弃的根）、I-5.5 认出来而没有数据单元（Q4） ───

/// 恰好要 `data_units` 个数据单元的内容（最后一个单元装一半）：顺序写一次发布切成 `data_units` 条记录（一个数据单元一条）。
fn content_needing_data_units(data_units: usize) -> Vec<u8> {
    let payload_capacity = singlefs_core::unit::data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 8) % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件 A (1, 3) 之后顺序写三个数据单元的发布 B (1, 4)：B 切成三条记录（计数器 4、5、6，每条带提交标记、只有 6 带末条标志），
/// 两块盘各一份；崩在 B 的根槽 FUA 之前（攻方报告 `research/prompts/m3-formal-code-b34-r1-opus-output.md` P1-b 那张 crafted 镜像的同形）。
/// 最新的根还是 A 的，实例表里没有实例 1 的行：B 是下一次挂载先施加的那一版。交回镜像与 A、B 两次发布的输出。
struct CrashBeforeTheRootOfAThreeRecordPublish {
    image: MemoryPool,
    previous: TransactionOutput,
    publish: TransactionOutput,
}

fn crash_image_with_a_three_record_publish_but_not_its_root(
    tag: &str,
) -> CrashBeforeTheRootOfAThreeRecordPublish {
    let mut pool = build_pool(tag);
    let operations_through_the_first_publish = pool.retained_operations().len();
    let previous = pool.output.clone();
    let content = content_needing_data_units(3);
    let publish = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("顺序写三个数据单元")
    };
    let counters: Vec<u64> = publish
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([publish.record.counter])
        .collect();
    assert_eq!(
        (counters, publish.root.checkpoint_txg),
        (vec![4, 5, 6], CheckpointTxg(4)),
        "B 是 txg 4、切成计数器 4、5、6 三条记录"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let kinds_of_the_publish: Vec<StepKind> = operations[operations_through_the_first_publish..]
        .iter()
        .map(|retained| classifier.classify(&retained.operation))
        .collect();
    let root_record_step = kinds_of_the_publish
        .iter()
        .position(|kind| *kind == StepKind::RootRecordFua)
        .expect("发布 B 有一步根槽 FUA");
    assert_eq!(
        (
            kinds_of_the_publish[..root_record_step]
                .iter()
                .filter(|kind| **kind == StepKind::JournalRecord)
                .count(),
            kinds_of_the_publish[root_record_step - 1]
        ),
        (6, StepKind::Barrier),
        "根槽 FUA 之前：B 的三条记录两块盘各写了一份，紧挨着根槽的是记录之后那道屏障"
    );
    let mut image = pool.memory_pool_after_mkfs();
    image.apply(
        &operations
            [pool.mkfs_operation_count..operations_through_the_first_publish + root_record_step],
    );
    CrashBeforeTheRootOfAThreeRecordPublish {
        image,
        previous,
        publish,
    }
}

/// 把 B 的第一条记录（计数器 4，两块盘各一份）的新根段七项换成上一版 A 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15
/// 「每条记录都带同一份新根段」的样子），重封、后面两条重链（攻方报告 P1-b 的 s1 改法）。
fn write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(
    image: &mut MemoryPool,
    crash: &CrashBeforeTheRootOfAThreeRecordPublish,
) {
    let first_counter = crash
        .publish
        .earlier_records_of_this_publish
        .first()
        .expect("B 有三条记录")
        .record
        .counter;
    let stale = &crash.previous.root;
    rewrite_journal_record_and_rechain_the_records_after_it(image, first_counter, |record| {
        record.new_tree_table = stale.tree_table;
        record.new_mapping_root = stale.mapping_root;
        record.new_tree_identifier_watermark = stale.tree_identifier_watermark;
        record.new_rollback_floor = stale.rollback_floor;
        record.new_instance_table = stale.instance_table;
        record.new_allocation_record_tree_root = stale.allocation_record_tree_root;
        record.new_unmount_marker = stale.unmount_marker;
        record.new_reclaim_threshold = stale.reclaim_threshold;
        record.new_pinned_ring_slot = stale.pinned_ring_slot;
    });
}

/// 改法 FL（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告 P1-b 的 s3 形）：
/// 下一次挂载先施加的那一版 B 切成三条记录，第一条的新根段写成上一版 A 的，B 自己重写的那片分配记录叶里两条记录罩住同一个槽
/// （新的整单元校验和沿引用链补到 B 的树表，再补进 B 的三条记录）。core 恢复走到带末条标志的那一条才整次施加、施加的是它的新根段，
/// 重建 B 的分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上。checker 认这一版也取末条那一条：I-5.4 红，点名下一次挂载先施加的那一版；
/// 取先到的那一条时走的是 A 的树，I-5.4 不红。第一条记录写旧了本身是 I-8.10（记录新根段与同代根相同） 往非末条扩的那一半的违例
/// （C738（同一次发布几条记录的新根段一致没有不变量判）），所以红的恰是 I-5.4 与 I-8.10 两条。
#[test]
fn a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot(
) {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-bad-stale-first-record-over-overlapping-records-of-the-next-mount-version",
    );
    let verdicts_on_the_clean_image = check_pool_image(&crash.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的三条记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    let mut image = crash.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4", LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH],
        "第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来：红的恰是 I-5.4 与第一条记录自己的 I-8.10：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
    );
    let InvariantVerdict::Violated(new_root_segment_detail) = verdict(
        &image,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    ) else {
        panic!("上面判过 I-8.10 红");
    };
    assert!(
        new_root_segment_detail.contains("同一次发布（实例 1、txg 4）的两条记录新根段不逐字节相同"),
        "I-8.10 点名 B 那次发布（txg 4）的记录：{new_root_segment_detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "同一块盘上两条分配记录罩住同一个槽",
            })
        ),
        "core 施加的是末条那一份新根段（B 的树），重建分配器时拒在两条记录罩住同一个槽上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：B 的第一条记录写成 A 的新根段（只这一处，不叠别的坏法），
/// 可写挂载照常收下（core 施加末条那一份，现行版是 B (1, 4)），写行发布与暖机之后实例 2 再覆盖写一次：B 成了由记录施加出来、
/// 根槽从没落盘的那一版。checker 取末条那一条走 B 自己的树，走读那几条一条都不红；取先到的那一条时走的是 A 的树、B 自己的单元没走到，
/// I-3.1 与 I-3.5 红。第一条记录写旧了本身照 I-8.10 往非末条扩的那一半红（C738（同一次发布几条记录的新根段一致没有不变量判）），
/// 红的只有这一条。
#[test]
fn a_version_applied_only_by_its_records_whose_first_record_carries_the_previous_root_is_walked_from_its_last_record(
) {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-good-stale-first-record-of-a-version-applied-only-by-its-records",
    );
    let mut image = crash.image.clone();
    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(4), 3),
        "择 A 的根 (1, 3)，施加 B 的三条记录，现行版是 (1, 4)"
    );
    let current = mounted
        .current
        .into_file_version()
        .expect("施加了 B，现行那一版带文件");
    let content: Vec<u8> = (0..2900usize)
        .map(|index| u8::try_from((index * 11 + 2) % 251).expect("小于 256"))
        .collect();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 再覆盖写一次");
    }
    let image_after_the_overwrite = memory_pool_of_sparse_devices(&devices);
    let verdicts = check_pool_image(&image_after_the_overwrite);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH],
        "B 只由记录施加出来、第一条记录写旧了：checker 与 core 同取末条，走的是 B 自己的树，只有写旧的那条记录自己红 I-8.10：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image_after_the_overwrite, "I-3.5"),
        InvariantVerdict::Holds,
        "I-3.5 要真被评估过且成立（取的是末条那一份新根段，B 自己的单元走到了）"
    );
    assert_eq!(
        verdict(&image_after_the_overwrite, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立（B 自己的单元并进了遍历）"
    );
}

/// 这次发布写的第一个数据单元，在这次发布重写的分配记录叶里它那两条未释放记录（两块盘各一条，分在两片叶里）的分配代从这次发布的
/// txg 改成 txg + 1，叶重封、新的整单元校验和沿引用链补上去，再补进引用改过的单元的那几条记录；数据单元一个字节不动。
/// 盘上留下的是 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的违例。交回那个数据单元的起点槽。
fn raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
    image: &mut MemoryPool,
    version: &TransactionOutput,
    record_copies_referencing_the_changed_units: usize,
) -> u64 {
    let units = units_found_on_device_zero(image);
    let checksums_before = unit_checksums(image, &units);
    let data_unit_slot = version
        .units
        .iter()
        .find(|unit| {
            matches!(unit.identity, TransactionUnit::Data(_))
                && version.rewritten.contains(&unit.identity)
        })
        .expect("这次发布写了数据单元")
        .slot
        .0;
    let birth_txg = version.root.checkpoint_txg.0;
    let leaves: Vec<u64> = version
        .units
        .iter()
        .filter(|unit| {
            version.rewritten.contains(&unit.identity)
                && matches!(
                    unit.identity,
                    TransactionUnit::AllocationTree
                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                )
        })
        .map(|unit| unit.slot.0)
        .filter(|slot| {
            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
            candidate_leaf[INDEX_NODE_LEVEL_OFFSET] == 0
                && rewrite_unreleased_allocation_generation(
                    &mut candidate_leaf,
                    data_unit_slot,
                    birth_txg,
                    birth_txg + 1,
                ) == 1
        })
        .collect();
    assert_eq!(
        leaves.len(),
        2,
        "数据单元槽 {data_unit_slot} 两块盘各一份落点，两条未释放记录分在两块盘各自那片叶里"
    );
    for leaf in leaves {
        mutate_unit_of(
            image,
            &units,
            leaf,
            |bytes| {
                assert_eq!(
                    rewrite_unreleased_allocation_generation(
                        bytes,
                        data_unit_slot,
                        birth_txg,
                        birth_txg + 1
                    ),
                    1,
                    "上面挑叶时这一片改得动"
                );
            },
            true,
        );
    }
    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        record_copies_referencing_the_changed_units,
        "只有这次发布落了的那几条记录（两块盘各一份）引用这一版改过的单元"
    );
    data_unit_slot
}

/// 改法 R1（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方报告
/// `research/prompts/m3-formal-code-b35-r1-opus-output.md` P1 第 2 条 `cut=records-2..2-of-2` 那一形）：发布 B (1, 4) 切成三条记录，
/// 首条（计数器 4，两块盘各一份）没落、后两条落了，崩在根槽 FUA 之前——断号。core 从 A 的锚点起要计数器 4、读到的是 5，前缀断在这里，
/// B 整次不施加。B 重写的分配记录叶里放两处坏法：两条记录罩住同一个槽（I-5.4 的形），B 的一个数据单元那两条未释放记录分配代改成 txg + 1
/// （I-3.10 的形）；新的整单元校验和补进 B 的三条记录、后两条重链。
/// 阳性对照：三条都落了时 B 是下一次挂载先施加的那一版，I-3.10 与 I-5.4 都红在 B 上。
/// 首条没落时 checker 照 I-4.1 那一份重放认这一版：B 不施加就不认，I-5.4 与 I-3.10 都不读 B 的分配记录树，这张镜像一条都不红
/// （只看带末条标志的那一条、即计数器 6 时认 B，两条照红）；可写挂载照常收下：择 A 的根，一条记录都不施加，现行版仍是 A。
#[test]
fn a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first() {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-good-publish-with-a-gap-before-its-last-record",
    );
    let mut image = crash.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
    raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
        &mut image,
        &crash.publish,
        6,
    );
    let first_counter = crash
        .publish
        .earlier_records_of_this_publish
        .first()
        .expect("B 有三条记录")
        .record
        .counter;
    rewrite_journal_record_and_rechain_the_records_after_it(
        &mut image,
        first_counter,
        |_record_kept_as_is| {},
    );
    let verdicts_with_every_record_landed = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts_with_every_record_landed),
        ["I-3.10", "I-5.4"],
        "阳性对照：B 的三条记录都落了，B 是下一次挂载先施加的那一版，两处坏法各红一条：{verdicts_with_every_record_landed:?}"
    );

    // mkfs 把整个 journal 环写清零（`crates/singlefs-core/src/make_filesystem.rs` 里 `write_zeroes_at(journal_ring_start, …)`），
    // 计数器 4 那一格在 B 之前是全零：写回全零就是首条那两次写没落。
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DEVICES {
        write(
            &mut image,
            device,
            record_offset(first_counter, JOURNAL_RING_DEFAULT_BYTES).0,
            &vec![0u8; record_bytes],
        );
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "B 断了号、core 不施加：B 的分配记录树不在 I-5.4、I-3.10 判的版本里，两处坏法都不红：{verdicts:?}"
    );
    for invariant in ["I-5.4", "I-3.10"] {
        assert_eq!(
            verdict(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 在 A 那一版上真被评估过且成立"
        );
    }

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("B 不施加：可写挂载照 A 那一版收下，B 的分配记录树它不读");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(3), 0),
        "择 A 的根 (1, 3)，前缀断在计数器 5（要的是 4），一条记录都不施加"
    );
}

/// A 的根落了之后实例 1 再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）。交回整段历史写完的镜像（根槽都在）与 B、C 两次发布的输出；
/// 要「B、C 都只在记录里」的，改完单元之后再把两次的根槽改坏（`tear_the_root_slot_written_for_txg`：改单元时沿引用链重封根槽，先改坏会被封回去）。
struct TwoSingleRecordPublishes {
    image: MemoryPool,
    first_publish: TransactionOutput,
    second_publish: TransactionOutput,
}

fn image_with_two_single_record_publishes(tag: &str) -> TwoSingleRecordPublishes {
    let mut pool = build_pool(tag);
    let content_of = |length: usize, seed: usize| -> Vec<u8> {
        (0..length)
            .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
            .collect()
    };
    let first_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of(CONTENT_BYTES_OF_THE_OLDER_OVERWRITE, 3),
    );
    let second_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of(CONTENT_BYTES_OF_THE_NEWER_OVERWRITE, 11),
    );
    assert_eq!(
        (
            (
                first_publish.root.checkpoint_txg,
                first_publish.record.counter
            ),
            (
                second_publish.root.checkpoint_txg,
                second_publish.record.counter
            )
        ),
        ((CheckpointTxg(4), 4), (CheckpointTxg(5), 5)),
        "B、C 各一条记录：txg 4 计数器 4、txg 5 计数器 5"
    );
    TwoSingleRecordPublishes {
        image: pool.memory_pool(),
        first_publish,
        second_publish,
    }
}

/// 改法 R1 的另一半：前缀停在后面那次发布的点名单元上时，停之前整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs`
/// 的 `replay_journal`：前缀停在验不过的那一条之前），第一次那一版仍是下一次挂载先施加的那一版。B、C 都只在记录里，
/// B 重写的一片分配记录叶里两条记录罩住同一个槽（补进 B 那条记录、C 那条按它重链），C 那条记录第一个点名单元在第二份落点那块盘上翻一个字节（不重封）：
/// 重放施加 B、停在 C 之前。I-4.1 报不适用（起点含介质损坏，那一行自己的射程）；I-5.4 照认 B，只红 I-5.4、点名下一次挂载先施加的那一版
/// （实例 1、txg 4）。core 的只读恢复施加 B 一条、验不过 C 一条，落在 (1, 4)。
#[test]
fn a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
) {
    let built = image_with_two_single_record_publishes(
        "known-bad-overlapping-records-of-the-next-mount-version-before-a-named-unit-that-does-not-verify",
    );
    let mut image = built.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
        &mut image,
        &built.first_publish,
        2,
    );
    rewrite_journal_record_and_rechain_the_records_after_it(
        &mut image,
        built.first_publish.record.counter,
        |_record_kept_as_is| {},
    );
    for txg in [4u64, 5] {
        tear_the_root_slot_written_for_txg(&mut image, txg);
    }
    let offset_of_the_second_record = record_offset(
        built.second_publish.record.counter,
        JOURNAL_RING_DEFAULT_BYTES,
    )
    .0;
    let second_record = read(&image, 0, offset_of_the_second_record, 4096);
    assert!(
        get_u32(&second_record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
        "C 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = get_u32(&second_record, second_location);
    let mut named_slot_bytes = [0u8; 8];
    named_slot_bytes[..6]
        .copy_from_slice(&second_record[second_location + 4..second_location + 10]);
    let named_slot = u64::from_le_bytes(named_slot_bytes);
    let mut unit = read(&image, named_device, named_slot * SLOT, 16384);
    unit[1000] ^= 0xff;
    write(&mut image, named_device, named_slot * SLOT, &unit);

    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        (
            report.effective_root,
            report.journal.prefix_applied,
            report.journal.verification_failed
        ),
        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
        "core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "重放停在 C 之前，B 照认：它的两条记录罩住同一个槽只红 I-5.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
    );
    match verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other:?}")
        }
    }
}

/// 计数器为 `counter` 的那条 journal 记录（两块盘各一份）新根段里的树表指针整条写零，经 `JournalRecord::to_bytes` 重封。
/// 不重链后面的记录：这里改的两条记录（由记录施加出来的 (1, 5)、下一次挂载先施加的 B）在环里都是本实例最后一条，
/// 下一条（若有）是别的实例的第一条，它的反向链不接在这一条上（`common::rewrite_journal_record_and_rechain_the_records_after_it`
/// 不分实例照接，会把那一条改坏、I-8.6 红）。
fn write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
    image: &mut MemoryPool,
    counter: u64,
) {
    let filesystem_identifier =
        singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier);
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DEVICES {
        let mut record = JournalRecord::parse(
            &read(image, device, offset, record_bytes),
            filesystem_identifier,
        )
        .expect("改之前这条记录自证过");
        assert_eq!(
            record.counter, counter,
            "环里那一格是计数器 {counter} 的记录"
        );
        record.new_tree_table = singlefs_core::pointer::NodePointer::empty_root();
        let rewritten = record.to_bytes();
        assert!(
            rewritten[JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                ..JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET + 86]
                .iter()
                .all(|byte| *byte == 0),
            "新根段里的树表指针整条全零"
        );
        write(image, device, offset, &rewritten);
    }
}

/// Q1 甲（两类都判损坏），
/// 由记录施加出来的那一版那一支（正推探针 `zz_probe_q1` 的 q1b 形）：挂载之后只由记录施加出来的那一版 (1, 5) 那条记录（两块盘各一份）
/// 新根段里的树表指针整条写零、重封、后面的记录重链。core 施加记录之后拼出的根交给同一个根记录读者，照它读盘 0 槽 0、读不出；
/// 主走读与根环里的根同一读法跟过去：盘 0 槽 0 那两份对不上（I-2.1），从这一版出发的遍历读不出（I-4.8、I-7.4）。这一版自己的单元
/// 没走到、遍历少算：I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判，记账多于遍历落在上沿之内，成立（改三判之前红在这一边）。
/// 不读成「这一版没有树表」（那样只红 I-3.1、I-3.5）。
#[test]
fn an_all_zero_tree_table_pointer_in_the_record_of_a_version_applied_only_by_its_journal_record_is_followed_like_the_core(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-all-zero-tree-table-pointer-of-a-version-applied-only-by-its-record",
    );
    let mut image = built.image.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
        &mut image,
        built.version_applied_only_by_its_record.record.counter,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.4"],
        "(1, 5) 那条记录里树表指针全零：照 core 跟过去读不出，不再读成没有树表：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.1") else {
        panic!("上面判过 I-2.1 红");
    };
    assert!(
        detail.contains(
            "由记录施加出来的那一版（实例 1、txg 5）的树表（记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
        ),
        "I-2.1 写明读不对的是由记录施加出来的那一版的树表、指针从记录新根段里读来：{detail}"
    );
}

/// Q1 甲，下一次挂载先施加的那一版那一支（正推探针 `zz_probe_q1` 的 q1 形，攻方报告 P2-b 的 z6 形）：发布 B 的记录已落、根槽没落的崩溃镜像上，
/// B 那条记录（两块盘各一份）新根段里的树表指针整条写零、重封。两种挂载都拒成「槽 0 读不出」；checker 的重放（I-4.1）施加 B、
/// 施加出来的那一版照 core 跟过去走不完，I-4.1 红，别的判定与干净的崩溃镜像只差 I-5.4 那一句部分没判（认不出带不带树）。
/// 这一版不读成「没有树表」。
#[test]
fn an_all_zero_tree_table_pointer_in_the_record_of_the_version_the_next_mount_applies_first_reddens_the_replayed_version_invariant(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-all-zero-tree-table-pointer-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    let mut image = clean.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
        &mut image,
        COUNTER_OF_THE_OVERWRITE_RECORD,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE],
        "B 那条记录里树表指针全零：重放施加出来的那一版照 core 跟过去走不完：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_on_the_clean_image,
            &[TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE, "I-5.4"]
        ),
        [],
        "I-4.1 与 I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    assert!(
        matches!(
            verdict(&image, "I-5.4"),
            InvariantVerdict::PartiallyJudged(unjudged)
                if unjudged.starts_with("树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树")
                    && unjudged.ends_with("：下一次挂载先施加的那一版（实例 1、txg 4）")
        ),
        "I-5.4 照旧把这一版记成树表指针全零、认不出带不带树的部分没判：{:?}",
        verdict(&image, "I-5.4")
    );
    let InvariantVerdict::Violated(detail) =
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE)
    else {
        panic!("上面判过 I-4.1 红");
    };
    assert!(
        detail.contains("记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目"),
        "I-4.1 写明施加出来的那一版走不完是因为记录新根段里树表指针全零：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::UnitUnreadable {
                slot: singlefs_core::address::SlotNumber(0)
            })
        ),
        "core 施加 B 之后照全零的树表指针读盘 0 槽 0、读不出：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// Q4 甲（另写一句）：新池新建文件那一版（从 mkfs 水位第一次建树、认法 ①② 都满足）
/// 树表里 extent 树那一条的根指针整条写零、重封、沿引用链补到根槽：这一版认得出、却一个数据单元都没有，I-5.5 一版都不判，
/// 报「认出来了而没有数据单元」那一句专门理由，不落到缺省理由、也不落到「一版都没认出来」那一句。
#[test]
fn a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_the_first_data_unit_invariant_not_applicable_with_its_own_reason(
) {
    let mut image =
        build_pool("known-bad-first-tree-building-version-without-a-data-unit").memory_pool();
    let zeroed_entries = std::cell::Cell::new(0usize);
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
                let entry = tree_table_entry_offset(index);
                let kind = u16::from_le_bytes([
                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
                ]);
                if kind == TREE_KIND_EXTENT {
                    bytes[entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.start
                        ..entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.end]
                        .fill(0);
                    zeroed_entries.set(zeroed_entries.get() + 1);
                }
            }
        },
        true,
    );
    assert_eq!(zeroed_entries.get(), 1, "树表里 extent 树那一条恰一条");
    assert_eq!(
        verdict(
            &image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::NotApplicable(
            singlefs_checker::walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT
        ),
        "认出了从 mkfs 水位第一次建树的那一版而它没有数据单元：I-5.5 报这一句专门理由"
    );
}
