//! harness 测试文件 `crates/singlefs-harness/tests/instance_table_second_page_write.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::Placement;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::instance_table::{
    InstanceRow, InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::pointer::{BirthSequence, NodePointer};
use singlefs_core::recovery::{
    allocation_records_of_version_without_file, choose_root, choose_system_configuration,
    instance_table_chain_of_root, instance_table_of_root, recover, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{acquire_instance, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::run_new_pool_file_creation;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一片实例表里数据行的上限：370 条记录减去链指针那一条。
pub const ROWS_PER_PAGE: usize = 369;

pub const SECOND_PAGE: InstanceTablePageIndex = InstanceTablePageIndex(1);

pub type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

pub fn sparse_devices() -> (Devices, SharedStream) {
    let stream = SharedStream::new();
    let devices: Devices = DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    (devices, stream)
}

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 新池新建文件（发布 A，txg 3，实例 1）。
pub fn pool_after_the_new_pool_file_creation() -> Devices {
    let (mut devices, stream) = sparse_devices();
    run_new_pool_file_creation(&parameters(), &mut devices, &stream, |_, _| {})
        .expect("新池新建文件");
    devices
}

/// 两块 4 GiB 内存盘上只做 mkfs（树表 0 条的那一版）。
pub fn formatted_pool() -> Devices {
    let (mut devices, _stream) = sparse_devices();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    devices
}

/// 连着 `count` 次「取号之后崩溃」：每次取一个新号写进两块盘的系统配置，写行那次发布没发出去。
pub fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

/// 新池新建文件之后连着 369 次取号之后崩溃，再可写挂载一次：取号 371、给 [1, 371) 写 370 行——一片装不下，写两片。
pub fn pool_whose_row_publish_opened_the_second_page() -> (Devices, Mounted) {
    let mut devices = pool_after_the_new_pool_file_creation();
    crash_right_after_acquisition(&mut devices, 369);
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("370 行写成两片，可写挂载要成立：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(371));
    assert_eq!(mounted.output.rows_written.len(), ROWS_PER_PAGE + 1);
    (devices, mounted)
}

/// 行 [1, `last_instance`] 按实例代号升序：写行接出来的表就是这个样子（A 那一行 (1, 3, W)，其余 (i, 0, 0)）。
pub fn instances_of(rows: &[InstanceRow]) -> Vec<u32> {
    rows.iter().map(|row| row.instance.0).collect()
}

/// 从盘上沿链读一片：按链上第 `page` 片解出来。
pub fn page_on_disk(
    devices: &Devices,
    pointer: &NodePointer,
    page: InstanceTablePageIndex,
) -> InstanceTablePage {
    let location = pointer.locations[0];
    let bytes = PoolReader::read(
        devices.as_slice(),
        location.device,
        location.slot.to_device_offset(),
        32768,
    )
    .expect("那一片读得到");
    assert_eq!(
        crc32_castagnoli(&bytes),
        location.unit_checksum,
        "指针里的整单元校验和对得上"
    );
    InstanceTablePage::parse(&bytes, page).expect("那一片按第几片解得开")
}

pub fn newest_root(image: &MemoryPool) -> RootRecord {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    choose_root(image, &system_configuration).expect("最新根")
}

/// 第 371 次可写挂载之后（验收那一条的另一种走法见 `instance_table_page_full.rs`）：
/// 冷启动读回真写者写出的全部行（沿链读两片，370 行按实例代号 1..=370），恢复照常读回文件；池级 checker 在两片上全绿。
pub fn cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green(
    images: &mut PoolCheckerImages,
) {
    let (devices, _mounted) = pool_whose_row_publish_opened_the_second_page();
    let image = memory_pool_of_sparse_devices(&devices);
    let newest = newest_root(&image);
    let chain = instance_table_chain_of_root(&image, &newest).expect("两片都读得出");
    assert_eq!(chain.page_pointers.len(), 2, "最新那一版的表是两片");
    assert_eq!(
        instances_of(&chain.records.rows),
        (1..=370).collect::<Vec<u32>>(),
        "冷启动沿链读回真写者写出的全部 370 行"
    );
    assert!(
        matches!(
            recover(&image, JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "冷启动走读沿链读两片，照常读回文件"
    );
    // I-1.1、I-2.1、I-3.1、I-3.8 在两片上判成立
    images.hand_out(
        "冷启动读回两片之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-1.1")
            .and_holds("I-2.1")
            .and_holds("I-3.1")
            .and_holds("I-3.8"),
    );
}

/// 再挂一次（取号 372、这一版 370 行、写 1 行）：写行那次发布 COW 重写整条链（369 + 2），被换下的两片旧链逐片释放——
/// 两片的落点都在这次释放的清单里、两盘的记录都改写成已释放、释放代是这次写行的 txg；池级 checker 全绿，
/// I-3.9（释放代落在停止引用它的那一格区间里）判成立：旧链第 1 片那几条已释放记录也有引用过它的根作见证。
pub fn the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages(
    images: &mut PoolCheckerImages,
) {
    let (mut devices, mounted) = pool_whose_row_publish_opened_the_second_page();
    let old_first_page = mounted.output.row_publish.root().instance_table;
    let old_second_page =
        match page_on_disk(&devices, &old_first_page, InstanceTablePageIndex::FIRST).chain {
            InstanceTableChainRecord::NextPage(pointer) => pointer,
            InstanceTableChainRecord::LastPage => panic!("上一次挂载写的是两片"),
        };
    let remounted = mount_writable(&parameters(), &mut devices).expect("再挂一次");
    assert_eq!(remounted.output.instance, InstanceGeneration(372));
    let row_publish = remounted
        .output
        .row_publish
        .file_version()
        .expect("写行那次发布带文件");
    let txg = row_publish.root.checkpoint_txg;
    for old_page in [old_first_page, old_second_page] {
        let placement = Placement {
            slot: old_page.locations[0].slot,
            span: 2,
        };
        assert!(
            row_publish.released.contains(&placement),
            "旧链那一片 {placement:?} 在这次释放的清单里：{:?}",
            row_publish.released
        );
        for device in DISKS {
            let record = remounted
                .allocator
                .record_for(device, placement.slot)
                .expect("释放只改写记录、不删");
            assert_eq!(
                (record.is_released, record.generation),
                (true, txg),
                "盘 {device:?} 上旧链那一片改写成已释放、释放代是这次写行的 txg"
            );
        }
    }
    let image = memory_pool_of_sparse_devices(&devices);
    let chain = instance_table_chain_of_root(&image, &newest_root(&image)).expect("新链读得出");
    assert_eq!(chain.page_pointers.len(), 2);
    assert_eq!(
        instances_of(&chain.records.rows),
        (1..=371).collect::<Vec<u32>>()
    );
    let new_second_page = page_on_disk(&devices, &chain.page_pointers[1], SECOND_PAGE);
    assert_eq!(
        instances_of(&new_second_page.rows),
        vec![370, 371],
        "第 1 片装 2 行"
    );
    images.hand_out(
        "再挂一次重写整条链之后",
        || image.clone(),
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-3.9"),
    );
}

/// 树表 0 条的一版上写两片（`publish_instance_table_on_version_without_file`）：mkfs 之后连着 370 次取号之后崩溃，
/// 第一次可写挂载取号 371、给 [1, 371) 写 370 行。落点与出生序号照同一个次序——第 1 片、第 0 片、分配记录树那一片；
/// 分配记录树那一片的账里两片都在、已分配。再挂一次（写 1 行）：两片旧链与旧的分配记录树节点一起释放，池级 checker 全绿。
pub fn version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both(
    images: &mut PoolCheckerImages,
) {
    let mut devices = formatted_pool();
    crash_right_after_acquisition(&mut devices, 370);
    let mounted = mount_writable(&parameters(), &mut devices).expect("树表 0 条的一版上写两片");
    assert_eq!(mounted.output.instance, InstanceGeneration(371));
    let PoolVersion::WithoutFile(row_publish) = &mounted.output.row_publish else {
        panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
    };
    let image = memory_pool_of_sparse_devices(&devices);
    let chain = instance_table_chain_of_root(&image, &row_publish.root).expect("两片读得出");
    assert_eq!(chain.page_pointers.len(), 2);
    assert_eq!(
        instances_of(&chain.records.rows),
        (1..=370).collect::<Vec<u32>>()
    );
    let (first_page, second_page) = (chain.page_pointers[0], chain.page_pointers[1]);
    let allocation_record_node = row_publish.root.allocation_record_tree_root;
    assert!(
        second_page.locations[0].slot < first_page.locations[0].slot
            && first_page.locations[0].slot < allocation_record_node.locations[0].slot,
        "落点按 bump 次序：第 1 片、第 0 片、分配记录树的根在最末"
    );
    let tree = allocation_records_of_version_without_file(&image, &row_publish.root)
        .expect("分配记录树读得出")
        .expect("写过行的一版有自己的分配记录树");
    // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，根之下每块盘有罩着单元区起点那一段的
    // 节点；这一版的树整棵都是这次写的，接在实例表两片之后按 bump 次序发号，根最末。
    let allocation_record_tree_nodes = u32::try_from(tree.version.nodes.len()).expect("节点数");
    assert!(
        allocation_record_tree_nodes > 1,
        "根之下至少有罩着记录的那几片"
    );
    assert_eq!(
        (
            second_page.birth_sequence,
            first_page.birth_sequence,
            allocation_record_node.birth_sequence
        ),
        (
            BirthSequence(0),
            BirthSequence(1),
            BirthSequence(1 + allocation_record_tree_nodes)
        ),
        "都归树 0，出生序号按 bump 次序发：两片实例表在前，分配记录树的节点在后、根最末"
    );
    let records = &tree.records;
    for page in [first_page, second_page] {
        for device in DISKS {
            assert!(
                records.iter().any(|record| record.device == device
                    && record.slot == page.locations[0].slot
                    && !record.is_released
                    && record.generation == row_publish.root.checkpoint_txg),
                "盘 {device:?} 上那一片在这一版的账里、已分配"
            );
        }
    }

    let remounted = mount_writable(&parameters(), &mut devices).expect("再挂一次");
    let PoolVersion::WithoutFile(second_row_publish) = &remounted.output.row_publish else {
        panic!("仍是没有文件版本的一版")
    };
    for page in [first_page, second_page, allocation_record_node] {
        for device in DISKS {
            let record = remounted
                .allocator
                .record_for(device, page.locations[0].slot)
                .expect("释放只改写记录、不删");
            assert_eq!(
                (record.is_released, record.generation),
                (true, second_row_publish.root.checkpoint_txg),
                "盘 {device:?} 上被换下的那一片改写成已释放"
            );
        }
    }
    let image_after_the_second_mount = memory_pool_of_sparse_devices(&devices);
    assert_eq!(
        instances_of(
            &instance_table_of_root(
                &image_after_the_second_mount,
                &newest_root(&image_after_the_second_mount)
            )
            .expect("表读得出")
            .rows
        ),
        (1..=371).collect::<Vec<u32>>()
    );
    images.hand_out(
        "再挂一次释放两片旧链之后",
        || image_after_the_second_mount.clone(),
        PoolCheckerExpectation::no_invariant_violated(),
    );
}
