//! harness 测试文件 `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::extent_tree::{ExtentLowerNodePosition, ExtentUpperNodePosition};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, scan_journal, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, PositionAddressedTreeHeights,
    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{JOURNAL_NAMED_ENTRIES_PER_RECORD, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};

/// 一份长度为 `length_in_bytes` 的可复现内容，字节随下标与种子走（两次写的内容不同才看得出读回的是哪一版）。
pub fn content_of(length_in_bytes: usize, seed: usize) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半，声明长度那一格才有东西可判。
pub fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    content_of(
        (data_units - 1) * payload_capacity + payload_capacity / 2,
        seed,
    )
}

/// 在同一个进程、同一个实例里顺序写一次整份内容。
pub fn sequential_write(
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

/// 冷恢复读回的内容与实际走的根；走不下去就报错。
pub fn recovered(reader: &dyn PoolReader) -> (Vec<u8>, (InstanceGeneration, CheckpointTxg)) {
    let report = recover(reader, JournalPolicy::Consult);
    let effective_root = report.effective_root.expect("择得到根");
    let RecoveryOutcome::FileRead { content, .. } = report.outcome else {
        panic!("要读回一个文件，实际 {:?}", report.outcome);
    };
    (content, effective_root)
}

/// 交一张镜像给池级 checker：一条都不红（`what` 是这一刻的名字）。
pub fn hand_out_an_image_the_pool_checker_finds_no_violation_on(
    images: &mut PoolCheckerImages,
    image: &MemoryPool,
    what: &str,
) {
    images.hand_out(
        what,
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}

/// 盘上环里这次发布（txg）的记录条数：从冷镜像全环扫描数，不信写者自己交回的。
pub fn records_on_disk_with_txg(reader: &dyn PoolReader, txg: CheckpointTxg) -> usize {
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
pub fn cold_sequential_read_matches(
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

/// 验收第 1、2 条：65 个单元（跨过一条记录 64 个点名项那个门槛）再 144 个单元（一片 extent 叶装满），
/// 每次冷启动读回逐字节相同，一次发布的记录条数 = 数据单元数，extent 节点读取次数 ≤ 树高 + 叶数，池级 checker 没有违例。
pub fn units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
    images: &mut PoolCheckerImages,
) {
    // 写路径不读这个上限，单元数写成字面量；先钉住上限今天的值，上限一变这里先红，单元数才会跟着挪，不会贴着旧上限照样绿下去。
    assert_eq!(
        JOURNAL_NAMED_ENTRIES_PER_RECORD + 1,
        65,
        "第一次写的单元数是一条 journal 记录的点名项上限 ⌊(4096 − 487) ÷ 56⌋ = 64 加一；上限变了要跟着挪单元数"
    );
    let mut pool = build_pool("parallel-line-one-large-file");
    let sixty_five_units = content_needing(65, 1);
    let first = sequential_write(&mut pool, &sixty_five_units, InstanceGeneration(1));
    assert_eq!(first.data_pointers.len(), 65);
    let image_after_the_first = pool.memory_pool();
    assert_eq!(
        records_on_disk_with_txg(&image_after_the_first, first.root.checkpoint_txg),
        65,
        "一次发布的记录条数 = 数据单元数 65（C310 用户定案的口径），64 那个点名项上限够不着"
    );
    // 144 个单元之内（D8（核心索引结构） 已定项 14）：上段一片根兼叶，下段一片根兼叶。
    cold_sequential_read_matches(&image_after_the_first, &sixty_five_units, 65, (2, 2));
    let (content, effective_root) = recovered(&image_after_the_first);
    assert!(
        content == sixty_five_units,
        "冷走读（recover）读回 65 个单元"
    );
    assert_eq!(
        effective_root,
        (first.root.instance, first.root.checkpoint_txg)
    );
    hand_out_an_image_the_pool_checker_finds_no_violation_on(
        images,
        &image_after_the_first,
        "65 个单元之后",
    );

    let one_hundred_forty_four_units = content_needing(144, 2);
    let second = sequential_write(
        &mut pool,
        &one_hundred_forty_four_units,
        InstanceGeneration(1),
    );
    assert_eq!(second.data_pointers.len(), 144);
    for position in 0..65u64 {
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
    hand_out_an_image_the_pool_checker_finds_no_violation_on(
        images,
        &image_after_the_second,
        "144 个单元之后",
    );
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
/// 从新池新建文件那一版（一个单元、内联）起四次顺序写，走遍下段的几种变形：
/// 145 个单元（多于一片叶装得下的 144 个）⇒ 下段长成两层：单元 0..=143 那片叶、单元 144 那片叶、上面第 1 层第 0 个根；
/// 再写 144 个单元 ⇒ 下段缩回一片根兼叶；再写一个单元 ⇒ 没有下段，那个数据指针内联在上段叶条目里；再写三个单元 ⇒ 下段又长出来。
/// 上段恒只有 inode 1 所在那片根兼叶。每一次上一版下段的节点都在这次发布里释放；每一版冷启动顺序读回逐字节相同、
/// 打开文件时走过的高与叶数（上段加下段）对得上、池级 checker 没有违例。
pub fn the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline(
    images: &mut PoolCheckerImages,
) {
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
        hand_out_an_image_the_pool_checker_finds_no_violation_on(
            images,
            &image,
            &format!("{data_units} 个单元之后"),
        );
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
pub fn shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has(
    images: &mut PoolCheckerImages,
) {
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
    hand_out_an_image_the_pool_checker_finds_no_violation_on(images, &image, "文件变短之后");
}

/// 进程重开：可写挂载从盘上重建上一版（extent 根兼叶里的每条记录、每个数据单元都读回来），写行与暖机照抄三个数据单元；
/// 之后再顺序写两个单元，上一版的三个数据单元按重建出来的映射 key 经映射释放。冷启动读回两个单元的那一版。
pub fn reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping(
    images: &mut PoolCheckerImages,
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
    hand_out_an_image_the_pool_checker_finds_no_violation_on(images, &image, "重开之后再顺序写");
    let reopened_devices = pool.reopen_cold();
    let (content_after_reopen, _) = recovered(&reopened_devices);
    assert!(
        content_after_reopen == two_units,
        "冷启动读回两个单元的那一版"
    );
}
