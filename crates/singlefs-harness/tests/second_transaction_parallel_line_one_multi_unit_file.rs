//! 里程碑「第二个事务」并行线一（一个文件跨多个单元：大文件顺序写、顺序读）的验收。
//!
//! 压着它的条款：D8（核心索引结构） 已定项 3（extent key 的 offset 段是文件字节偏移）；D23（journal 的角色与格式） 已定项 17
//! （一次发布切成 N 条记录时共享内生块只在最后一条点名，只有真正的最后一条带「本次发布末条」标志）、已定项 14 注 1
//! （链首锚在所选根覆盖的最后一条 = 那次发布带末条标志的那条，读法乙）与第六条（发布边界按记录标志位 0 认，末条没到的发布整体不施加）；
//! C310（事务切分纪律与记录数口径打架）
//! 2026-09-16 用户定案（N 个数据单元 = N 个事务 = N 条记录）；D4（校验和位置） 已定项 5（净荷 32634，文件偏移到单元做除法）。
//!
//! 七条用例：
//! 1. 写 68 个单元（跨过 67）再写 144 个单元（一片 extent 叶装满），冷启动顺序读回逐字节相同；一次发布的记录条数 = 数据单元数；
//!    extent 树节点只在打开文件时按需读、读取次数 ≤ 树高 + 叶数（顺序读期间块层一次 16 KiB 节点读都没有）；池级 checker 一条违例都没有。
//!    多于 144 个单元时 extent 树下段长成两层（D8（核心索引结构） 已定项 14），下段几种变形走一遍的是
//!    `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`。
//! 2. 文件变短：上一版多出来的数据单元在同一次发布里释放；释放之前同样按映射条目的位置项读盘核校验和
//!    （D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），核出对不上的那一份隔离。
//! 3. 进程重开之后可写挂载（重建上一版、写行、暖机都照抄多个数据单元），再顺序写一次：经映射释放重建出来的那几个单元。
//! 4. P6 / C365：链首锚在所选根那次发布的末条——两条记录的发布 B 是所选根，C 的记录落了、根没落，恢复要施加 C。
//! 5. C376：两条记录的发布 B 崩在第二条记录之前 ⇒ 整次发布不施加、文件是旧长度（I-4.3（提交原子））；两条都落了、根没落 ⇒ 施加。
//! 6. P6 后一半（D23（journal 的角色与格式） 已定项 14 注 1 / 已定项 4）：所选根那条记录读不出时，链首只接本次发布内序号为 1 的那条——
//!    下一次发布的第一条也读不出、第二条读得出 ⇒ 一条都不施加（三方第一轮 K4-b）；第一条读得出 ⇒ 整次施加。

mod common;

use common::{
    build_pool, file_content, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::check_journal_record;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes,
    InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::Placement;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::extent_tree::{ExtentLowerNodePosition, ExtentUpperNodePosition};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, scan_journal, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_sequential_write, CopyQuarantinedAfterReleaseChecksumMismatch, FirstFile, PoolWriter,
    PositionAddressedTreeHeights, QuarantinedCopyReading, TransactionOutput, TransactionUnit,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};
use singlefs_harness::{RecordedOperationKind, RetainedOperation};

/// 一份长度为 `length_in_bytes` 的可复现内容，字节随下标与种子走（两次写的内容不同才看得出读回的是哪一版）。
fn content_of(length_in_bytes: usize, seed: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半，声明长度那一格才有东西可判。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    content_of(
        (data_units - 1) * payload_capacity + payload_capacity / 2,
        seed,
    )
}

/// 在同一个进程、同一个实例里顺序写一次整份内容。
fn sequential_write(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

/// 录制流前 `operation_count` 步施加到两块空内存盘上：崩在那一步之前的镜像。
fn memory_pool_of_the_first(pool: &BuiltPool, operation_count: usize) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&pool.retained_operations()[..operation_count]);
    image
}

/// 冷恢复读回的内容与实际走的根；走不下去就报错。
fn recovered(reader: &dyn PoolReader) -> (Vec<u8>, (InstanceGeneration, CheckpointTxg)) {
    let report = recover(reader, JournalPolicy::Consult);
    let effective_root = report.effective_root.expect("择得到根");
    let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
        panic!("要读回一个文件，实际 {:?}", report.outcome);
    };
    (content, effective_root)
}

fn assert_the_pool_checker_finds_no_violation(image: &MemoryPool, what: &str) {
    let violated: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert!(
        violated.is_empty(),
        "{what}：池级 checker 判红 {violated:?}"
    );
}

/// 盘上环里这次发布（txg）的记录条数：从冷镜像全环扫描数，不信写者自己交回的。
fn records_on_disk_with_txg(reader: &dyn PoolReader, txg: CheckpointTxg) -> usize {
    let system_configuration = choose_system_configuration(reader).expect("系统配置");
    scan_journal(reader, &system_configuration)
        .values()
        .filter(|record| record.checkpoint_txg == txg)
        .count()
}

/// 冷启动顺序读：只读挂载（择根、扫环、施加前缀、打开挂载态），打开文件之后按单元一个一个往后读到文件末尾，
/// 读回的字节拼起来与 `expected` 逐字节相同；extent 树节点只在打开文件时按需读（D8（核心索引结构） 已定项 14，K4），
/// 读取次数 ≤ 树高 + 叶数，树高与叶数是 `extent_tree_height_and_leaves`（上段加下段）；
/// 顺序读期间块层一次 16 KiB（码 2 节点）的读都没有——不是每个单元从 inode 重走一遍。
fn cold_sequential_read_matches(
    reader: &dyn PoolReader,
    expected: &[u8],
    data_units: u64,
    extent_tree_height_and_leaves: (u64, u64),
) {
    let counting = ReadCountingPoolReader::new(
        reader,
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
    );
    let mounted = mount_read_only(&counting).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开文件");
    let extent_tree_reads = file.extent_tree_reads_at_open();
    assert!(
        extent_tree_reads.node_reads <= extent_tree_reads.height + extent_tree_reads.leaves,
        "extent 树节点的读取次数 ≤ 树高 + 叶数：{extent_tree_reads:?}"
    );
    assert_eq!(
        (extent_tree_reads.height, extent_tree_reads.leaves),
        extent_tree_height_and_leaves,
        "extent 树打开时走过的高与叶数（上段加下段）"
    );
    assert_eq!(file.data_unit_count(), data_units);
    counting.reset_tally();
    let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("32634");
    let file_size = u64::try_from(expected.len()).expect("文件大小");
    let mut read_back = Vec::with_capacity(expected.len());
    for unit_number in 0..data_units {
        let first_byte = DataUnitIndexInFile(unit_number).first_file_byte(payload_capacity);
        let length = payload_capacity.min(file_size - first_byte.0);
        let output = file
            .read_at(&counting, FileOffsetInBytes(first_byte.0), length)
            .expect("读一个单元");
        assert_eq!(output.observation.data_units_read, 1, "一次读一个单元");
        read_back.extend_from_slice(&output.bytes);
    }
    assert!(
        read_back == expected,
        "冷启动顺序读回的字节与写入逐字节相同"
    );
    let tally = counting.tally();
    assert_eq!(
        tally.reads_of_a_whole_index_node, 0,
        "顺序读期间一个码 2 节点都不读：extent 树在打开时读过一次"
    );
    assert_eq!(
        tally.reads_inside_the_journal_ring, 0,
        "顺序读期间一次都不扫 journal 环"
    );
    assert_eq!(
        mounted.mounted.observation_since_open().data_units_read,
        data_units,
        "读路径报出这次顺序读读了几个单元"
    );
}

/// 验收第 1、2 条：68 个单元（跨过一条记录 67 个点名项那个门槛）再 144 个单元（一片 extent 叶装满），
/// 每次冷启动读回逐字节相同，一次发布的记录条数 = 数据单元数，extent 节点读取次数 ≤ 树高 + 叶数，池级 checker 没有违例。
#[test]
fn units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
) {
    let mut pool = build_pool("parallel-line-one-large-file");
    let sixty_eight_units = content_needing(68, 1);
    let first = sequential_write(&mut pool, &sixty_eight_units, InstanceGeneration(1));
    assert_eq!(first.data_pointers.len(), 68);
    let image_after_the_first = pool.memory_pool();
    assert_eq!(
        records_on_disk_with_txg(&image_after_the_first, first.root.checkpoint_txg),
        68,
        "一次发布的记录条数 = 数据单元数 68（C310 用户定案的口径），67 那个点名项上限够不着"
    );
    // 144 个单元之内（D8（核心索引结构） 已定项 14）：上段一片根兼叶，下段一片根兼叶。
    cold_sequential_read_matches(&image_after_the_first, &sixty_eight_units, 68, (2, 2));
    let (content, effective_root) = recovered(&image_after_the_first);
    assert!(
        content == sixty_eight_units,
        "冷走读（recover）读回 68 个单元"
    );
    assert_eq!(
        effective_root,
        (first.root.instance, first.root.checkpoint_txg)
    );
    assert_the_pool_checker_finds_no_violation(&image_after_the_first, "68 个单元之后");

    let one_hundred_forty_four_units = content_needing(144, 2);
    let second = sequential_write(
        &mut pool,
        &one_hundred_forty_four_units,
        InstanceGeneration(1),
    );
    assert_eq!(second.data_pointers.len(), 144);
    for position in 0..68u64 {
        let replaced_slot = first
            .unit(TransactionUnit::Data(DataUnitIndexInFile(position)))
            .slot;
        assert!(
            second
                .released
                .iter()
                .any(|placement| placement.slot == replaced_slot),
            "上一版第 {position} 个数据单元在这次发布里经映射释放"
        );
    }
    let image_after_the_second = pool.memory_pool();
    assert_eq!(
        records_on_disk_with_txg(&image_after_the_second, second.root.checkpoint_txg),
        144
    );
    assert_the_pool_checker_finds_no_violation(&image_after_the_second, "144 个单元之后");
    let reopened_devices = pool.reopen_cold();
    cold_sequential_read_matches(
        &reopened_devices,
        &one_hundred_forty_four_units,
        144,
        (2, 2),
    );
    let (content_after_reopen, _) = recovered(&reopened_devices);
    assert!(
        content_after_reopen == one_hundred_forty_four_units,
        "进程退出、按路径重开镜像之后冷走读读回 144 个单元"
    );
}

/// extent 树下段按单元号按位置寻址（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K2），一个文件下段的形状跟着单元数走。
/// 从第一个事务那一版（一个单元、内联）起四次顺序写，走遍下段的几种变形：
/// 145 个单元（多于一片叶装得下的 144 个）⇒ 下段长成两层：单元 0..=143 那片叶、单元 144 那片叶、上面第 1 层第 0 个根；
/// 再写 144 个单元 ⇒ 下段缩回一片根兼叶；再写一个单元 ⇒ 没有下段，那个数据指针内联在上段叶条目里；再写三个单元 ⇒ 下段又长出来。
/// 上段恒只有 inode 1 所在那片根兼叶。每一次上一版下段的节点都在这次发布里释放；每一版冷启动顺序读回逐字节相同、
/// 打开文件时走过的高与叶数（上段加下段）对得上、池级 checker 没有违例。
#[test]
fn the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline() {
    let mut pool = build_pool("parallel-line-one-extent-shapes");
    let lower_leaf = |index| ExtentLowerNodePosition { level: 0, index };
    let lower_root_over_two_leaves = ExtentLowerNodePosition { level: 1, index: 0 };
    let steps = [
        (
            145_usize,
            vec![lower_leaf(0), lower_leaf(1), lower_root_over_two_leaves],
            (3, 3),
        ),
        (144, vec![lower_leaf(0)], (2, 2)),
        (1, Vec::new(), (1, 1)),
        (3, vec![lower_leaf(0)], (2, 2)),
    ];
    for (seed, (data_units, lower_shape, height_and_leaves)) in steps.into_iter().enumerate() {
        let content = content_needing(data_units, 10 + seed);
        let previous = pool.output.clone();
        let output = sequential_write(&mut pool, &content, InstanceGeneration(1));
        assert_eq!(
            output
                .extent_tree
                .lower_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect::<Vec<_>>(),
            lower_shape,
            "{data_units} 个单元的下段"
        );
        assert_eq!(
            output
                .extent_tree
                .upper_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect::<Vec<_>>(),
            vec![ExtentUpperNodePosition { level: 0, index: 0 }],
            "上段只有 inode 1 所在那片根兼叶"
        );
        // 树高从根节点头现读（D8（核心索引结构） 已定项 14 末句照已定项 11「根节点头层级 + 1」；D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读它）：
        // 4 GiB 两块盘上分配记录树根在第 2 层；extent 树上段只有根兼叶；下段的高就是读回来的高里减去上段那一层。
        assert_eq!(
            output.position_addressed_tree_heights_read_from_the_root_node_headers(),
            PositionAddressedTreeHeights {
                allocation_record_tree: 3,
                extent_tree_upper_segment: 1,
                extent_tree_lower_segment_of_the_file: height_and_leaves.0 - 1,
            },
            "{data_units} 个单元：从根节点头现读的树高"
        );
        for (position, pointer) in &previous.extent_tree.lower_nodes {
            assert!(
                output
                    .released
                    .iter()
                    .any(|placement| placement.slot == pointer.locations[0].slot),
                "上一版下段的 {position:?} 在写 {data_units} 个单元那次发布里释放"
            );
        }
        let image = pool.memory_pool();
        assert_the_pool_checker_finds_no_violation(&image, &format!("{data_units} 个单元之后"));
        cold_sequential_read_matches(
            &image,
            &content,
            u64::try_from(data_units).expect("单元数"),
            height_and_leaves,
        );
    }
}

/// 文件变短：三个单元的文件再顺序写成一个单元，上一版第 1、2 个数据单元没有接替它们的新角色，同样在这次发布里释放
/// （每块盘的分配记录都改写成已释放），读回的是短的那一版，池级 checker 没有违例（I-3.1（已分配统计对得上）罩得到漏释放）。
#[test]
fn shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has() {
    let mut pool = build_pool("parallel-line-one-shrink");
    let three_units = content_needing(3, 3);
    let longer = sequential_write(&mut pool, &three_units, InstanceGeneration(1));
    let short_content = content_of(1000, 4);
    let shorter = sequential_write(&mut pool, &short_content, InstanceGeneration(1));
    assert_eq!(shorter.data_pointers.len(), 1);
    for position in 0..3u64 {
        let replaced_slot = longer
            .unit(TransactionUnit::Data(DataUnitIndexInFile(position)))
            .slot;
        assert!(
            shorter
                .released
                .iter()
                .any(|placement| placement.slot == replaced_slot),
            "上一版第 {position} 个数据单元在这次发布里释放"
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let record = pool
                .allocator
                .record_for(device, replaced_slot)
                .expect("释放只改写记录、不删");
            assert!(
                record.is_released,
                "盘 {device:?} 上第 {position} 个数据单元那一槽的记录改写成已释放"
            );
        }
    }
    let image = pool.memory_pool();
    let (content, _) = recovered(&image);
    assert!(content == short_content, "读回的是一个单元的那一版");
    assert_the_pool_checker_finds_no_violation(&image, "文件变短之后");
}

/// 改坏一块盘上某个槽起的那一份单元：翻它第 4096 字节起那个扇区的第一个字节，位置项里的校验和从此对不上。
/// 写经录制器落盘，与别的写一样进录制流（与 `second_transaction_supplement_two_release_checksum_quarantine.rs` 里同名那一个同一个做法）。
fn corrupt_the_copy_on(
    devices: &mut [(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    slot: SlotNumber,
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let sector_offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device
        .read_at(sector_offset, &mut sector)
        .expect("读那一个扇区");
    sector[0] ^= 0xff;
    block_device
        .write_at(sector_offset, &sector, WriteDurability::Plain)
        .expect("写回改坏的扇区");
}

/// 文件变短换下的尾巴也是经映射释放的，释放之前同样按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5
/// 硬规则 1：经映射释放的每一个都核）：三个单元的文件，第 2 个数据单元两盘那一份都被改坏，再顺序写成一个单元——
/// 第 2 个单元这次没有接替它的新角色，照样逻辑上释放，核出两份都对不上 ⇒ 两份都隔离（两盘那条记录留在已分配）。
/// 只改坏一块盘那一份时两块盘那一份一起隔离（D19（块指针的结构与宽度预算） 已定项 5，`second_transaction_supplement_two_release_checksum_quarantine.rs` 用例 2）。
#[test]
fn shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it(
) {
    let mut pool = build_pool("parallel-line-one-shrink-checksum");
    let three_units = content_needing(3, 5);
    let longer = sequential_write(&mut pool, &three_units, InstanceGeneration(1));
    let tail = TransactionUnit::Data(DataUnitIndexInFile(2));
    let tail_slot = longer.unit(tail).slot;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        corrupt_the_copy_on(
            pool.devices.as_mut().expect("镜像还开着"),
            device,
            tail_slot,
        );
    }
    let shorter = sequential_write(&mut pool, &content_of(1000, 6), InstanceGeneration(1));
    let tail_placement = Placement {
        slot: tail_slot,
        span: tail.span_slots(),
    };
    assert_eq!(
        shorter.quarantined_after_release_checksum_mismatch,
        [DeviceIdentity(0), DeviceIdentity(1)]
            .map(|device| CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: tail,
                device,
                placement: tail_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            })
            .to_vec(),
        "文件变短换下的第 2 个数据单元：两盘那一份都核出对不上，都隔离"
    );
    assert!(
        shorter
            .released
            .iter()
            .any(|placement| placement.slot == tail_slot),
        "逻辑上照样释放"
    );
}

/// 进程重开：可写挂载从盘上重建上一版（extent 根兼叶里的每条记录、每个数据单元都读回来），写行与暖机照抄三个数据单元；
/// 之后再顺序写两个单元，上一版的三个数据单元按重建出来的映射 key 经映射释放。冷启动读回两个单元的那一版。
#[test]
fn a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping(
) {
    let mut pool = build_pool("parallel-line-one-reopen");
    let three_units = content_needing(3, 5);
    let before_reopen = sequential_write(&mut pool, &three_units, InstanceGeneration(1));

    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("三个单元的文件之后重开，现行那一版带文件");
    assert_eq!(
        pool.output.data_pointers, before_reopen.data_pointers,
        "写行与暖机照抄三个数据单元的指针"
    );
    let instance_after_reopen = pool.output.root.instance;
    let image_after_reopen = pool.memory_pool();
    let (content, _) = recovered(&image_after_reopen);
    assert!(content == three_units, "重开之后读回的仍是三个单元那一版");

    let two_units = content_needing(2, 6);
    let after_reopen = sequential_write(&mut pool, &two_units, instance_after_reopen);
    for position in 0..3u64 {
        let replaced_slot = before_reopen
            .unit(TransactionUnit::Data(DataUnitIndexInFile(position)))
            .slot;
        assert!(
            after_reopen
                .released
                .iter()
                .any(|placement| placement.slot == replaced_slot),
            "重建出来的上一版第 {position} 个数据单元经映射释放"
        );
    }
    let image = pool.memory_pool();
    assert_the_pool_checker_finds_no_violation(&image, "重开之后再顺序写");
    let reopened_devices = pool.reopen_cold();
    let (content_after_reopen, _) = recovered(&reopened_devices);
    assert!(
        content_after_reopen == two_units,
        "冷启动读回两个单元的那一版"
    );
}

/// 录制流里最后一次根槽 FUA 写的下标：这一刻之前的写都落了、这次发布的根没落。
fn index_of_the_last_root_slot_write(operations: &[RetainedOperation]) -> usize {
    operations
        .iter()
        .rposition(|retained| {
            retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("至少一次发布写过根槽")
}

/// P6 / C365（D23（journal 的角色与格式） 已定项 14 注 1）：所选根是两条记录的发布 B 的根，C 的记录落了、C 的根没落。
/// 链首要锚在 B 的**末条**（带「本次发布末条」标志的那条，D23（journal 的角色与格式） 已定项 17）之后，下一条就是 C 的记录 ⇒ C 施加、读回 C。
/// 锚在 B 的第一条时，期待的下一条是 B 自己的第二条（它不在水位之上），C 的记录对不上号、一条都不施加（三方第一轮 K3）。
#[test]
fn the_chain_head_anchors_on_the_last_record_of_the_chosen_roots_publish() {
    let mut pool = build_pool("parallel-line-one-chain-head");
    let second = sequential_write(&mut pool, &content_needing(2, 7), InstanceGeneration(1));
    assert_eq!(
        second.earlier_records_of_this_publish.len(),
        1,
        "B 是两条记录的发布"
    );
    let third_content = content_of(2500, 8);
    let third = sequential_write(&mut pool, &third_content, InstanceGeneration(1));
    assert!(
        third.earlier_records_of_this_publish.is_empty(),
        "C 一个单元一条记录"
    );

    let operations = pool.retained_operations();
    let crash_before_the_third_root =
        memory_pool_of_the_first(&pool, index_of_the_last_root_slot_write(&operations));
    let (content, effective_root) = recovered(&crash_before_the_third_root);
    assert_eq!(
        effective_root,
        (third.root.instance, third.root.checkpoint_txg),
        "所选根是 B 的，C 的记录接在 B 的末条之后，施加之后走的是 C 的根"
    );
    assert!(content == third_content, "读回 C 的内容");
}

/// C376（D23（journal 的角色与格式） 已定项 14 第六条）与并行线一验收第 3 条：两条记录的发布 B 崩在第二条记录之前——
/// 第一条两份都落了，第二条一份都没落、根也没落 ⇒ 前五条判出来的前缀里 B 的末条没到，B 整体不施加，文件是 A 的旧长度（I-4.3（提交原子））。
/// 同一个 B 两条都落了、根没落 ⇒ 末条到了，整次施加、读回 B（这一格证明它不是一律不施加）。
#[test]
fn a_crash_before_the_second_record_of_a_two_record_publish_keeps_the_old_file() {
    let mut pool = build_pool("parallel-line-one-publish-boundary");
    let first_version = pool.output.clone();
    let two_units = content_needing(2, 9);
    let second = sequential_write(&mut pool, &two_units, InstanceGeneration(1));
    let last_record_offset = record_offset(second.record.counter, JOURNAL_RING_DEFAULT_BYTES);

    let operations = pool.retained_operations();
    let first_write_of_the_last_record = operations
        .iter()
        .position(|retained| {
            retained.operation.kind == RecordedOperationKind::Write
                && retained.operation.offset == last_record_offset
        })
        .expect("B 的末条写过");
    let earlier_record_offset = record_offset(
        second.earlier_records_of_this_publish[0].record.counter,
        JOURNAL_RING_DEFAULT_BYTES,
    );
    assert_eq!(
        operations[..first_write_of_the_last_record]
            .iter()
            .filter(|retained| retained.operation.offset == earlier_record_offset)
            .count(),
        2,
        "崩溃点之前 B 的第一条两盘各落了一份"
    );

    let crash_before_the_second_record =
        memory_pool_of_the_first(&pool, first_write_of_the_last_record);
    let (content, effective_root) = recovered(&crash_before_the_second_record);
    assert_eq!(
        effective_root,
        (
            first_version.root.instance,
            first_version.root.checkpoint_txg
        ),
        "B 的末条没到：B 整体不施加，走的还是 A 的根"
    );
    assert!(
        content == file_content(),
        "文件是 A 的旧长度、旧内容（{} 字节）",
        content.len()
    );

    let crash_before_the_second_root =
        memory_pool_of_the_first(&pool, index_of_the_last_root_slot_write(&operations));
    let (content_with_both_records, effective_root_with_both_records) =
        recovered(&crash_before_the_second_root);
    assert_eq!(
        effective_root_with_both_records,
        (second.root.instance, second.root.checkpoint_txg),
        "两条都落了、根没落：末条到了，B 整次施加"
    );
    assert!(content_with_both_records == two_units, "读回 B 的两个单元");
}

/// P6 后一半（D23（journal 的角色与格式） 已定项 14 注 1 / 已定项 4）：所选根那条记录读不出时，链首接水位之上第一条可读记录，
/// 前提是它的 checkpoint_txg = 所选根的 txg + 1、本次发布内序号为 1。
///
/// 历史：A（第一个文件，一条记录）之后顺序写三个数据单元的 B（三条记录，序号 1、2、3），B 的记录全落了、根没落 ⇒ 所选根是 A 的；
/// 再把 A 那条记录两份都撕掉（没有锚点）。
/// - 再撕 B 的第一条（两份）：水位之上第一条可读的是 B 的第二条，txg 正是 A + 1，序号却是 2 ⇒ B 的开头缺了，断号即止，
///   一条都不施加、走的还是 A 的根、读回 A 的内容。只看 txg 时链首接在 B 的第二条上，缺了第一条的 B 整体施加（三方第一轮 K4-b：多接）。
/// - B 的第一条读得出：序号 1、txg = A + 1 ⇒ 接上，B 三条整次施加、读回 B——这一格证明它不是「没锚点就一条都不接」。
#[test]
fn without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish() {
    let mut pool = build_pool("parallel-line-one-chain-head-without-anchor");
    let first_version = pool.output.clone();
    let three_units = content_needing(3, 11);
    let second = sequential_write(&mut pool, &three_units, InstanceGeneration(1));
    let anchor_counter = first_version.record.counter;
    let counters_of_the_second_publish: Vec<u64> = second
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([second.record.counter])
        .collect();
    assert_eq!(
        counters_of_the_second_publish,
        vec![anchor_counter + 1, anchor_counter + 2, anchor_counter + 3],
        "B 是三条记录的发布，紧接在 A 那条之后"
    );
    assert_eq!(
        second.root.checkpoint_txg.0,
        first_version.root.checkpoint_txg.0 + 1,
        "B 的 txg = A + 1"
    );

    let operations = pool.retained_operations();
    let crash_before_the_second_root = index_of_the_last_root_slot_write(&operations);
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    // 序号从盘上读、用 checker 的独立解析：B 的三条依次是 1、2、3（写者那一半）。
    let image_with_every_record = memory_pool_of_the_first(&pool, crash_before_the_second_root);
    let ordinals_on_disk: Vec<u32> = counters_of_the_second_publish
        .iter()
        .map(|counter| {
            let bytes = image_with_every_record
                .read(
                    DeviceIdentity(0),
                    record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES),
                    record_bytes,
                )
                .expect("B 的记录落了");
            check_journal_record(&bytes, filesystem_identifier)
                .expect("B 的记录自证过")
                .ordinal_within_publish
        })
        .collect();
    assert_eq!(
        ordinals_on_disk,
        vec![1, 2, 3],
        "B 三条记录的本次发布内序号"
    );

    let first_record_of_the_second_publish = counters_of_the_second_publish[0];
    for (torn_counters, expected_effective_root, expected_content, expected_applied, what) in [
        (
            vec![anchor_counter, first_record_of_the_second_publish],
            (
                first_version.root.instance,
                first_version.root.checkpoint_txg,
            ),
            file_content(),
            0,
            "A 那条与 B 的第一条都读不出：B 的第二条序号是 2，不许当链首",
        ),
        (
            vec![anchor_counter],
            (second.root.instance, second.root.checkpoint_txg),
            three_units.clone(),
            3,
            "只有 A 那条读不出：B 的第一条序号 1、txg = A + 1，接上、B 整次施加",
        ),
    ] {
        let mut image = memory_pool_of_the_first(&pool, crash_before_the_second_root);
        for counter in &torn_counters {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                image.flip_byte(
                    device,
                    record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES),
                    300,
                );
            }
        }
        let report = recover(&image, JournalPolicy::Consult);
        assert_eq!(
            report.effective_root,
            Some(expected_effective_root),
            "{what}：施加之后走的根（{:?}）",
            report.journal
        );
        assert_eq!(report.journal.prefix_applied, expected_applied, "{what}");
        let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
            panic!("{what}：要读回一个文件，实际 {:?}", report.outcome);
        };
        assert!(
            content == expected_content,
            "{what}：读回的内容（{} 字节）",
            content.len()
        );
    }
}
