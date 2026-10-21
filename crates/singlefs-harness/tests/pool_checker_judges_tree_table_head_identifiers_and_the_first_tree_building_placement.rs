//! 池级 checker 的两条（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥，判决 `research/prompts/c673-clauses-r1-main-verification.md`
//! 第二节 K1、K2）：
//! - I-9.27（树表条目的头 ID 取值）照同一张树表里的关系判、不拿字面 65537 比：崩溃恢复丢掉实例 2 的第一个文件版本之后，
//!   实例 3 从根环水位 65549 发树（inode 树 65550）的那段合法历史上一条不红；
//! - I-5.5（从 mkfs 水位建树那一版的 t1 落点）在新池新建文件（默认 768 MiB 环与 128 MiB 环）、只暖机过的实例 2 写第一个文件的那一版上
//!   真被评估过且成立；第一个文件版本离开根环之后只剩覆盖写那几版（诞生 txg 不等于这一版的 txg），一版都不判。
//! - I-5.5 的判据按这一版的分配记录现算（用户 2026-10-04 定，判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md`
//!   第二节 K2、第五节第 1 行）：mkfs 的实例表（U、U + 1）被 COW 走、过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条
//!   把第一个文件的数据单元放在 U——只做过 mkfs 的池只暖机挂载 S + 2 次（S 是根环每区槽数，今天 10 次）之后写第一个文件、同一次挂载里推 22 次零单元发布之后写第一个文件，
//!   两段合法历史上 I-5.5 成立（攻方原型 `research/prompts/m3-formal-code-b17-r1-opus-model/opus_b17_first_data_unit_attack.rs`
//!   的形 A、形 B，写死 U + 4 的判据在这两段上误红）；一块盘上有两个数据单元的那一版，取的是每块盘上最小的那一个。
//!
//! 盘一律是内存稀疏盘（4 GiB × 2），不落镜像文件。

mod common;

use common::{
    crash_state_devices, file_content, geometry, memory_pool_of_sparse_devices, parameters,
    unreadable_root_slot_of, with_unreadable_ranges, without_unreadable_ranges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::{
    InvariantVerdict, FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version,
    publish_without_units, warm_up, FileVersionPlan, FirstFile, InstanceTablePlan, PoolVersion,
    PoolWriter, PublishPlan, TransactionOutput, ZeroUnitPublishPlan, FIRST_INODE_NUMBER,
    NEW_POOL_FILE_CREATION_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    DATA_UNIT_BYTES, NEW_POOL_FILE_CREATION_TXG, ROOT_RING_REGIONS,
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type MemoryDevice = RecordingBlockDevice<SparseBlockDevice>;

/// 根环一圈的槽数（区域 3 × 每区 8 槽）：接着发这么多版之后，这之前那一版的根槽一定被新根盖掉了。
const ROOT_RING_SLOT_COUNT: u64 = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
/// 从 mkfs 水位第一次建树的那一版，每块盘上落点最小的数据单元比单元区起始槽号多几槽（D3（空间分配） 已定项 8 射程，测试这边另写一份）。
const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
/// 非默认环长的那一种：128 MiB 环，单元区起始槽号现算（攻方在这一格上量过 9216，`research/prompts/c673-clauses-r1-opus-model/out/probe.out`）。
const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
/// mkfs 的实例表那一对（U、U + 1）的释放代：实例 2 写行那次发布把它 COW 走。
const RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE: u64 = 3;
/// 只做过 mkfs 的池只暖机挂载这么多次（根环每区槽数 S 加 2），第这一次挂载上写的第一个文件版本落到 U：第 k 次挂载上的第一个文件版本
/// txg 3k − 1，从 k ≥ 2 起成立（第 1 次挂载上是 3、第 2 次 5、第 3 次 8：正推量过，`research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-txg.log`）；
/// 写它之前环里最旧有效根比它早一圈（3S 槽），k = S + 2 时是 5 ≥ 释放代 3，U、U + 1 已回收（k = S + 1 时是 2 < 3，还没有；S ≥ 5 时两个 k 都 ≥ 6，
/// 落在式子成立的范围里）。S 取 mkfs 写进系统配置的那一个（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`，今天 8：第 10 次挂载、txg 29）；
/// mkfs 这条路径上 S 只能取 5–15（`crates/singlefs-core/src/root_ring.rs` 的 `at_make_filesystem_checked_at_compile_time`：4、16 编不过）。
/// 攻方在 S = 8 上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 里 `name=shape_a ring=default k=9`、`k=10` 两行）；正推在 S = 5、6、7、8、12、15
/// 上量过这两条用例照原样绿、放宽一个 txg 的变异红（判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节「K3 H2 前提断言」；
/// 翻转在 S + 2、放宽一个 txg 的翻转在 S + 1 见判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K1-a）。
const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 =
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2;
/// 比上一个少一次（S + 1）：这一次挂载上的第一个文件版本写之前环里最旧有效根 2，比释放代 3 差一个 txg，mkfs 的实例表那一对还没过回收门槛。
/// 回收门槛放松一个 txg（释放代 ≤ 下界 + 1）的坏实现在这一格提前回收、落 U（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`
/// 第二节 K2-1，攻方量过 `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-slack0.txt` 与 `h2-slack1.txt` 里 `mounts=9` 那一行）。
const WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE: u64 =
    WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED - 1;
/// 实例 2 里接连推这么多次零单元发布之后写的第一个文件版本落到 U（攻方量过翻转点 21 → 22，
/// `research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log` 里 `name=shape_b z=21`、`z=22` 两行）。
const ZERO_UNIT_PUBLISHES_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 = 22;

/// 两块空的内存稀疏盘，写录进同一条流。
fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("池级 checker 的清单里有这一条")
        .1
}

/// 判红的那几条与各自第一处违例。
fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两代 → 第一个文件版本），交回那份镜像、单元区起始槽号与第一个文件版本。
fn new_pool_file(
    make_filesystem_parameters: &MakeFilesystemParameters,
) -> (MemoryPool, u64, TransactionOutput) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(make_filesystem_parameters, &mut devices).expect("mkfs");
    let mut allocator =
        allocator_after_make_filesystem(make_filesystem_parameters, &devices, &genesis);
    let first_file = {
        let mut writer = PoolWriter::new(make_filesystem_parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed_up.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    (
        memory_pool_of_sparse_devices(&devices),
        genesis.unit_area_start.slot().0,
        first_file,
    )
}

/// 一个镜像上 I-5.5 真被评估过且成立、别的不变量一条都不红；`history` 是这份镜像怎么来的（写进断言消息）。
/// I-5.5 先判：数据单元落点被挪了的实现上，红的第一条断言就是它。
fn assert_the_first_data_unit_placement_holds_and_nothing_reddens(
    image: &MemoryPool,
    history: &str,
) {
    assert_eq!(
        verdict_of(
            image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::Holds,
        "{history}：I-5.5 要真被评估过且成立"
    );
    assert_eq!(
        violations_of(image),
        Vec::new(),
        "{history}：池级 checker 一条都不红"
    );
    assert_eq!(
        verdict_of(image, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES),
        InvariantVerdict::Holds,
        "{history}：I-9.27 要真被评估过且成立"
    );
}

/// 攻方原型 `k1_recovered_first_file_version_issues_trees_from_the_ring_watermark` 那段历史（`writable_mount_of_a_formatted_pool.rs`
/// 那条用例的内存盘版本）：只做过 mkfs 的池可写挂载两次（实例 1 只暖机，实例 2 写行、暖机），实例 2 发第一个文件版本（txg 5，树 65536..65547），
/// 崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前；可写挂载一次，txg 5 的根槽与它那条记录点名的数据单元盘 0 那一份读不出——择根落到 (2, 4)，
/// 实例 3 的水位按根环与环里记录取 max = 65549，从它发第一个文件版本（inode 树 65549，头 ID 都是 65549）。故障解除之后接着覆盖写，
/// 直到被丢掉的 (2, 5) 那一槽被新根盖掉（不留着它：它引用的数据单元槽被实例 3 复用了，那一形是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到））。
/// 这份镜像合法：池级 checker 一条不红，I-9.27 真被评估过且成立（字面 65537 的读法在这里误红 4 条）；树 ID 不从 65536 起，I-5.5 一版都不判。
#[test]
fn a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let after_mkfs = memory_pool_of_sparse_devices(&devices);
    drop(mount_writable(&parameters(), &mut devices).expect("实例 1：只暖机"));
    let second_instance = mount_writable(&parameters(), &mut devices).expect("实例 2");
    assert_eq!(second_instance.output.instance, InstanceGeneration(2));
    let PoolVersion::WithoutFile(version_of_the_second_instance) = &second_instance.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let mut allocator_of_the_second_instance = second_instance.allocator.clone();
    let dropped_first_file = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_of_the_second_instance,
            &version_of_the_second_instance.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            second_instance.output.instance,
            &version_of_the_second_instance.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    let dropped_txg = dropped_first_file.root.checkpoint_txg;
    assert_eq!(
        (dropped_txg, dropped_first_file.tree_identifiers.inode.0),
        (CheckpointTxg(5), 65537),
        "实例 2 写行 txg 3、暖机 txg 4，第一个文件版本 txg 5，从 mkfs 水位发树"
    );
    let operations = stream.retained_operations();
    let (writes, _segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
    let target = target_for_publish(dropped_txg, geometry().root_ring_slots_per_region);
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = slot_offset(target, geometry().fixed_structure_slot_spacing);
    let root_slot_write = writes
        .iter()
        .position(|write| {
            write.is_force_unit_access
                && write.device == region_device
                && write.offset == root_slot_offset
        })
        .expect("录制流里有 txg 5 的根槽 FUA 写");
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index <= root_slot_write)
        .collect();
    let crashed = crash_state_devices(
        &after_mkfs,
        &writes,
        &persisted,
        &SharedStream::retaining_contents(),
    );
    let named_data_unit = dropped_first_file.data_pointers[0].locations[0];
    let unreadable = SharedUnreadableRanges::new(
        vec![
            unreadable_root_slot_of(dropped_txg, FailingReadsOfARange::Every),
            UnreadableRange {
                device: named_data_unit.device,
                offset_in_bytes: named_data_unit.slot.to_device_offset().0,
                length_in_bytes: DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            },
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut reopened_with_the_fault = with_unreadable_ranges(crashed, &unreadable);
    let third_instance = mount_writable(&parameters(), &mut reopened_with_the_fault)
        .expect("txg 5 的根读不出、那条记录验不过：择根落到 (2, 4)，照常可写挂载");
    let mut reopened = without_unreadable_ranges(reopened_with_the_fault);
    let PoolVersion::WithoutFile(version_of_the_third_instance) = &third_instance.current else {
        panic!("落到实例 2 的暖机根：现行那一版树表 0 条")
    };
    assert_eq!(
        version_of_the_third_instance.root.tree_identifier_watermark, 65549,
        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65549"
    );
    let mut allocator_of_the_third_instance = third_instance.allocator.clone();
    let first_file_of_the_third_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_of_the_third_instance,
            &version_of_the_third_instance.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            third_instance.output.instance,
            &version_of_the_third_instance.record_bytes,
        )
        .expect("水位 65549、树表 0 条：照样发得出第一个文件版本")
    };
    assert_eq!(
        first_file_of_the_third_instance.tree_identifiers.inode.0, 65550,
        "实例 3 从 65549 发树：extent 树 65549、inode 树 65550"
    );
    let mut previous = first_file_of_the_third_instance;
    while previous.root.checkpoint_txg.0 < dropped_txg.0 + ROOT_RING_SLOT_COUNT {
        let content_bytes = 1000 + usize::try_from(previous.root.checkpoint_txg.0).expect("小");
        let next = {
            let publish_parameters = parameters();
            let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut allocator_of_the_third_instance,
                &previous,
                FirstFile {
                    content: &file_content()[..content_bytes],
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
                },
                third_instance.output.instance,
            )
            .expect("覆盖写")
        };
        previous = next;
    }
    let image = memory_pool_of_sparse_devices(&reopened);
    assert_eq!(
        violations_of(&image),
        Vec::new(),
        "树 ID 从根环水位发的合法镜像：池级 checker 一条都不红"
    );
    assert_eq!(
        verdict_of(&image, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES),
        InvariantVerdict::Holds,
        "I-9.27 在头 ID 65550 的树表上真被评估过且成立"
    );
    assert!(
        matches!(
            verdict_of(
                &image,
                FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "根环里每一版的 extent 树都不是 65536：没有哪一版是从 mkfs 水位第一次建树的那一版，I-5.5 一版都不判"
    );
}

/// 新池新建文件（默认 768 MiB 环，U = 50176）：那一版是从 mkfs 水位第一次建树的那一版，数据单元落在 U + 4 = 50180。
#[test]
fn a_new_pool_file_on_the_default_journal_ring_holds_the_first_data_unit_placement() {
    let (image, unit_area_start_slot, _first_file) = new_pool_file(&parameters());
    assert_eq!(unit_area_start_slot, 50176, "默认环长的单元区起始槽号");
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &image,
        "默认环长的新池新建文件",
    );
}

/// 新池新建文件（128 MiB 环）：U 按环长现算（不是 50176），t1 仍是 U + 4。
#[test]
fn a_new_pool_file_on_a_128_mebibyte_journal_ring_holds_the_first_data_unit_placement() {
    let mut make_filesystem_parameters = parameters();
    make_filesystem_parameters.geometry.journal_ring_bytes = JOURNAL_RING_BYTES_OF_128_MEBIBYTES;
    let (image, unit_area_start_slot, first_file) = new_pool_file(&make_filesystem_parameters);
    assert_ne!(
        unit_area_start_slot, 50176,
        "128 MiB 环的单元区起始槽号不是默认环的"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &image,
        "128 MiB 环的新池新建文件",
    );
    assert_eq!(
        first_file.data_pointers[0].locations[0].slot.0,
        unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START,
        "实现把数据单元放在 U + 4"
    );
}

/// 只做过 mkfs 的池可写挂载两次（实例 1 只暖机，实例 2 写行、暖机），实例 2 写第一个文件（txg 5）：
/// 树 ID 仍从 mkfs 水位 65536 起发、树表每条诞生 txg 都是 5 ⇒ 这一版被认出（射程「实例 1、2、3 从 mkfs 水位写的第一个文件」），落点照判、成立。
#[test]
fn the_first_file_of_an_instance_after_a_warm_up_only_mount_is_recognized_and_holds_the_first_data_unit_placement(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    drop(mount_writable(&parameters(), &mut devices).expect("实例 1：只暖机"));
    let second_instance = mount_writable(&parameters(), &mut devices).expect("实例 2");
    let PoolVersion::WithoutFile(version_of_the_second_instance) = &second_instance.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let mut allocator_of_the_second_instance = second_instance.allocator.clone();
    let first_file = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_of_the_second_instance,
            &version_of_the_second_instance.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            second_instance.output.instance,
            &version_of_the_second_instance.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    assert_eq!(
        (
            first_file.root.instance,
            first_file.root.checkpoint_txg,
            first_file.tree_identifiers.highest().0
        ),
        (InstanceGeneration(2), CheckpointTxg(5), 65548),
        "实例 2 的第一个文件版本 txg 5，树从 mkfs 水位发 65536..65548"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &memory_pool_of_sparse_devices(&devices),
        "只暖机过的实例 2 写第一个文件",
    );
}

/// 新池新建文件之后接着覆盖写，直到第一个文件版本（txg 3）与两代暖机根都被新根盖掉：根环里只剩覆盖写那几版，
/// 树表条目的诞生 txg 都是 3、不等于这一版的 txg（认法 ② 不成立），一版都不判——I-5.5 报不适用，池级 checker 一条都不红。
#[test]
fn once_the_first_file_version_leaves_the_root_ring_the_overwrite_versions_are_not_judged_for_the_first_data_unit_placement(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters(), &devices, &genesis);
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let mut previous = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed_up.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed_up.last_record_bytes,
    )
    .expect("新池新建文件");
    let first_file_txg = previous.root.checkpoint_txg;
    while previous.root.checkpoint_txg.0 < first_file_txg.0 + ROOT_RING_SLOT_COUNT {
        let content_bytes = 1000 + usize::try_from(previous.root.checkpoint_txg.0).expect("小");
        previous = publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile {
                content: &file_content()[..content_bytes],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("覆盖写");
    }
    drop(writer);
    let image = memory_pool_of_sparse_devices(&devices);
    assert!(
        matches!(
            verdict_of(
                &image,
                FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "根环里只剩覆盖写那几版（诞生 txg 3 ≠ 这一版的 txg）：I-5.5 一版都不判"
    );
    assert_eq!(
        violations_of(&image),
        Vec::new(),
        "覆盖写一圈之后的镜像：池级 checker 一条都不红"
    );
}

/// 一个文件版本的数据单元在每块盘上的落点槽号，按 (盘, 槽) 升序。
fn data_unit_slots_on_each_device(first_file: &TransactionOutput) -> Vec<(u32, u64)> {
    let mut slots: Vec<(u32, u64)> = first_file
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device.0, location.slot.0))
        .collect();
    slots.sort_unstable();
    slots
}

/// 只做过 mkfs 的池接连可写挂载 `mounts` 次（每次挂上只写行、暖机，不写文件），交回最后那一次挂载。
fn mount_writable_repeatedly(
    devices: &mut Vec<(DeviceIdentity, MemoryDevice)>,
    mounts: u64,
) -> Mounted {
    let mut last_mount = mount_writable(&parameters(), devices).expect("第 1 次可写挂载");
    for _ in 1..mounts {
        last_mount = mount_writable(&parameters(), devices).expect("再一次可写挂载");
    }
    last_mount
}

/// 在一次挂载现行的那一版（树表 0 条）上接着发第一个文件版本，写进 `devices`。
fn publish_the_first_file_on_the_mount(
    devices: &mut [(DeviceIdentity, MemoryDevice)],
    mounted: &Mounted,
    version_root: &singlefs_core::root_record::RootRecord,
    version_record_bytes: &[u8],
) -> TransactionOutput {
    let mut allocator = mounted.allocator.clone();
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices);
    publish_first_file(
        &mut writer,
        &mut allocator,
        version_root,
        FirstFile {
            content: &file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        mounted.output.instance,
        version_record_bytes,
    )
    .expect("树表 0 条的一版上发第一个文件版本")
}

/// 攻方原型形 A（k = S + 2，今天 10）：只做过 mkfs 的池只暖机挂载 S + 2 次，第 S + 2 次挂载上写第一个文件。实例 2 写行时 mkfs 的实例表（U、U + 1）
/// 被 COW 走，到这一版之前已过回收门槛：实现照 D3（空间分配） 已定项 8 第 1 条（起点最小、32768 对齐、两槽都空的那一对）把数据单元放在 U。
/// 这一版仍从 mkfs 水位第一次建树（认法 ①②），I-5.5 按这一版的分配记录现算：U、U + 1 是这一版自己分配的、算空，该在 U，成立；
/// 写死 U + 4 的判据在这里误红。池级 checker 一条都不红。
#[test]
fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement(
) {
    let mounts = WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED;
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let unit_area_start_slot = genesis.unit_area_start.slot().0;
    let mounted = mount_writable_repeatedly(&mut devices, mounts);
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let first_file = publish_the_first_file_on_the_mount(
        &mut devices,
        &mounted,
        &version.root,
        &version.record_bytes,
    );
    let oldest_valid_root_txg_before_the_first_file =
        first_file.root.checkpoint_txg.0 - ROOT_RING_SLOT_COUNT;
    assert!(
        oldest_valid_root_txg_before_the_first_file >= RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE,
        "前提：第 {mounts} 次挂载上的第一个文件版本（{:?}）写之前环里最旧有效根 txg {oldest_valid_root_txg_before_the_first_file} 不早于 mkfs 实例表那一对的释放代 {RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE}",
        first_file.root.checkpoint_txg
    );
    assert_eq!(
        first_file.tree_identifiers.inode.0, 65537,
        "第 {mounts} 次挂载上的第一个文件版本，树仍从 mkfs 水位发"
    );
    assert_eq!(
        data_unit_slots_on_each_device(&first_file),
        vec![(0, unit_area_start_slot), (1, unit_area_start_slot)],
        "mkfs 的实例表那一对已回收：实现把数据单元放在单元区起始槽号 U"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &memory_pool_of_sparse_devices(&devices),
        &format!("只暖机挂载 {mounts} 次之后写第一个文件"),
    );
}

/// 回收门槛差一个 txg 的那一格（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K2-1）：只做过 mkfs 的池只暖机挂载 S + 1 次
/// （今天 9），这一次挂载上写第一个文件（今天 txg 26）。mkfs 的实例表那一对释放代 3，写这一版之前环里最旧有效根是 txg 2，D3（空间分配） 已定项 7 的回收谓词
/// （已释放 ∧ 释放代 ≤ max(F, 环里最旧有效根)）不成立，U、U + 1 还占着：实现把数据单元放在 U + 4。直接钉实现的落点——I-5.5 按这一版的分配记录现算，
/// 放松一个 txg 的坏实现落 U 也判成立（C689），这里不靠它；池级 checker 照样一条都不红。
#[test]
fn the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim(
) {
    let mounts = WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE;
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let unit_area_start_slot = genesis.unit_area_start.slot().0;
    let mounted = mount_writable_repeatedly(&mut devices, mounts);
    let PoolVersion::WithoutFile(version) = &mounted.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let first_file = publish_the_first_file_on_the_mount(
        &mut devices,
        &mounted,
        &version.root,
        &version.record_bytes,
    );
    let oldest_valid_root_txg_before_the_first_file =
        first_file.root.checkpoint_txg.0 - ROOT_RING_SLOT_COUNT;
    assert_eq!(
        oldest_valid_root_txg_before_the_first_file + 1,
        RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE,
        "前提：第 {mounts} 次挂载上的第一个文件版本（{:?}）写之前环里最旧有效根比 mkfs 实例表那一对的释放代早恰好一个 txg",
        first_file.root.checkpoint_txg
    );
    assert_eq!(
        first_file.tree_identifiers.inode.0, 65537,
        "第 {mounts} 次挂载上的第一个文件版本，树仍从 mkfs 水位发"
    );
    assert_eq!(
        data_unit_slots_on_each_device(&first_file),
        vec![
            (
                0,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
            ),
            (
                1,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
            ),
        ],
        "mkfs 的实例表那一对差一个 txg 才过回收门槛：实现把数据单元放在 U + 4，不提前回收 U"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &memory_pool_of_sparse_devices(&devices),
        &format!("只暖机挂载 {mounts} 次之后写第一个文件"),
    );
}

/// 攻方原型形 B（z = 22）：mkfs 之后可写挂载两次（实例 2），在实例 2 里接连推 22 次零单元发布，再写第一个文件——
/// mkfs 的实例表那一对已过回收门槛，数据单元落在 U；I-5.5 按这一版的分配记录现算，成立，池级 checker 一条都不红。
#[test]
fn the_first_file_after_twenty_two_zero_unit_publishes_in_one_mount_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let unit_area_start_slot = genesis.unit_area_start.slot().0;
    let mounted = mount_writable_repeatedly(&mut devices, 2);
    let PoolVersion::WithoutFile(version_of_the_second_instance) = &mounted.current else {
        panic!("只暖机过的池上现行那一版树表 0 条")
    };
    let mut current = version_of_the_second_instance.clone();
    for _ in 0..ZERO_UNIT_PUBLISHES_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        current = publish_without_units(
            &mut writer,
            &current.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                instance: mounted.output.instance,
                back_chain: back_chain_of(&current.record_bytes),
                rollback_floor: current.root.rollback_floor,
                tree_identifier_watermark: current.root.tree_identifier_watermark,
            },
        )
        .expect("零单元发布");
    }
    let first_file = publish_the_first_file_on_the_mount(
        &mut devices,
        &mounted,
        &current.root,
        &current.record_bytes,
    );
    assert_eq!(
        first_file.tree_identifiers.inode.0, 65537,
        "第一个文件版本的树仍从 mkfs 水位发"
    );
    assert_eq!(
        data_unit_slots_on_each_device(&first_file),
        vec![(0, unit_area_start_slot), (1, unit_area_start_slot)],
        "mkfs 的实例表那一对已回收：实现把数据单元放在单元区起始槽号 U"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &memory_pool_of_sparse_devices(&devices),
        "实例 2 里推 22 次零单元发布之后写第一个文件",
    );
}

/// mkfs 同一个进程里那条流，第一个文件版本的内容比一个数据单元多一个字节：切成两个数据单元，每块盘上两个（U + 4 与 U + 6）。
/// I-5.5 取的是每块盘上落点最小的那一个（U + 4），等于按分配记录现算的那一对，成立；取最大的那一个就误红。
#[test]
fn a_new_pool_file_of_two_data_units_holds_the_first_data_unit_placement_on_the_smallest_slot_of_each_device(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream);
    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let unit_area_start_slot = genesis.unit_area_start.slot().0;
    let mut allocator = allocator_after_make_filesystem(&parameters(), &devices, &genesis);
    let content: Vec<u8> = file_content()
        .into_iter()
        .cycle()
        .take(data_unit_payload_capacity() + 1)
        .collect();
    let first_file = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let warmed_up_root = warmed_up.roots.last().expect("暖机两代根");
        allocator.record_zero_unit_roots_leading_to(warmed_up_root.checkpoint_txg);
        let txg = CheckpointTxg(NEW_POOL_FILE_CREATION_TXG);
        publish_version(
            &mut writer,
            &mut allocator,
            PublishPlan {
                txg,
                counter: warmed_up.records.last().expect("暖机两条记录").counter + 1,
                transaction: NEW_POOL_FILE_CREATION_NUMBER,
                highest_transaction_number_before_this_publish: 0,
                instance,
                back_chain: back_chain_of(&warmed_up.last_record_bytes),
                file: Some(FileVersionPlan {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                    inode_object_birth: txg,
                    change_count: txg.0,
                }),
                new_inode_records: &[],
                instance_table: InstanceTablePlan::Carry(warmed_up_root.instance_table),
                tree_birth_txg: txg,
                tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
                rollback_floor: warmed_up_root.rollback_floor,
            },
            None,
        )
        .expect("两个数据单元的新池新建文件")
    };
    assert_eq!(
        data_unit_slots_on_each_device(&first_file),
        vec![
            (
                0,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
            ),
            (
                0,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START + 2
            ),
            (
                1,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
            ),
            (
                1,
                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START + 2
            ),
        ],
        "两个数据单元落在 U + 4 与 U + 6，每块盘上各两个"
    );
    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
        &memory_pool_of_sparse_devices(&devices),
        "两个数据单元的新池新建文件",
    );
}
