//! 里程碑「第二个事务」增补 2 收口，代码三方第二轮判决（`research/prompts/m2-wave2-code-r1-main-verification.md`）第二节第 2 行（Z1-a）：
//! 每次可写挂载都给上一个实例写一行（D18（块里携带什么信息） 已定项 11），实例表第一版只有一片：370 条记录，链指针记录恒为最后一条、
//! 数据行至多 369。改之前取号之前的准入不算实例表：第 370 次可写挂载取号写完之后在装实例表单元时越界 panic（`bytes.rs` 的
//! `ByteWriter`），每试一次再烧一个实例代号（攻方探针 `research/prompts/m2-wave2-code-r1-opus-model/opus_probe_pristine.rs` 的
//! `pristine_instance_table_rows_overflow_after_acquisition`，日志 `pristine-run.log`）。
//! 改法（最小）：取号之前的准入加一项——这一版的行数 + 这次要写的行数 + 1（链指针）≤ 一片的记录数，不够返回
//! `InstanceTableRowsExceedOnePageSecondPageUnsupported`，一个写都不发；实例表第二片、删行不在这一轮。
//! 回退走同一个 `establish_instance`，行数按 R_old 那一版表与 [max(r_old, 1), 新实例) 自己算。
//!
//! 「取号之后崩溃」（取号写完、写行那次发布之前掉电）是今天就走得到的历史：下一次挂载要给中间那些实例各补一行 (i, 0, 0)，
//! 一次挂载就能写很多行。本文件用它把表快速填到边上（`acquire_instance` 连取 k 次号 = 连着 k 次取号之后崩溃）。

mod common;

use common::{disk_snapshot, memory_pool_of_sparse_devices, parameters, DiskSnapshot, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::mount::{
    mount_rollback, mount_writable, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::verified_superblock_slots;
use singlefs_core::transaction::{acquire_instance, PoolWriter, TransactionUnit};
use singlefs_format::INSTANCE_TABLE_PAGE_RECORDS;
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::run_first_transaction;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 一片实例表里数据行的上限：370 条记录减去链指针那一条。
const ROWS_PER_PAGE: usize = 369;

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个事务（发布 A，txg 3，实例 1）；录制流只记步数，不留内容。
fn pool_after_the_first_transaction() -> (Devices, SharedStream) {
    let stream = SharedStream::new();
    let mut devices: Devices = DISKS
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
    run_first_transaction(&parameters(), &mut devices, &stream, |_, _| {}).expect("第一个事务");
    (devices, stream)
}

/// 连着 `count` 次「取号之后崩溃」：每次取一个新号写进两块盘的超级块，写行那次发布没发出去。
fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

/// 写行那次发布写出的实例表里有几行。
fn rows_in_the_row_publish(mounted: &Mounted) -> usize {
    let row_publish = mounted
        .output
        .row_publish
        .file_version()
        .expect("第一个事务之后的写行发布带文件");
    InstanceTableRecords::parse(&row_publish.unit(TransactionUnit::InstanceTable).bytes)
        .expect("写出的实例表解得开")
        .rows
        .len()
}

/// 两块盘四个超级块槽里自证过的那些槽写着的实例代号，按盘排。
fn superblock_instances(devices: &Devices) -> Vec<(DeviceIdentity, Vec<InstanceGeneration>)> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    DISKS
        .iter()
        .map(|device| {
            let mut instances: Vec<InstanceGeneration> = verified_superblock_slots(
                devices,
                *device,
                spacing,
                &parameters().filesystem_identifier,
            )
            .iter()
            .map(|superblock| superblock.journal_instance)
            .collect();
            instances.sort();
            (*device, instances)
        })
        .collect()
}

fn snapshot(devices: &Devices, stream: &SharedStream) -> DiskSnapshot {
    disk_snapshot(&memory_pool_of_sparse_devices(devices), stream)
}

/// 拒绝的那一次：错误成员与三个数对得上，盘上逐字节不变（超级块槽原样字节、根环里的根、录制流步数）、四个超级块槽的实例代号不变。
fn assert_refused_before_acquisition(
    refused: Result<Mounted, MountError>,
    expected: (InstanceGeneration, usize, usize),
    devices: &Devices,
    stream: &SharedStream,
    before: &DiskSnapshot,
    instances_before: &[(DeviceIdentity, Vec<InstanceGeneration>)],
) {
    match refused {
        Err(MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported {
            instance_to_acquire,
            rows_in_version,
            rows_to_write,
            records_per_page,
        }) => {
            assert_eq!(
                (instance_to_acquire, rows_in_version, rows_to_write),
                expected,
                "要取的号、这一版的行数、这次要写的行数"
            );
            assert_eq!(
                records_per_page,
                usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370")
            );
            assert!(rows_in_version + rows_to_write + 1 > records_per_page);
        }
        other => panic!(
            "要在取号之前按实例表一片装不下拒绝：{:?}",
            other.map(|_| "挂上了")
        ),
    }
    assert_eq!(
        snapshot(devices, stream),
        *before,
        "盘上逐字节不变：超级块槽、根环里的根、录制流步数（一个写、一道屏障都没发）"
    );
    assert_eq!(
        superblock_instances(devices),
        instances_before,
        "两块盘超级块里的实例代号不变：号没烧"
    );
}

/// 验收：一路可写挂载到拒绝为止。攻方那条历史是从第一个事务起连挂 370 次（第 k 次取号 k + 1、表里 k 行），debug 下一次挂载近一秒，
/// 这里先连着 360 次取号之后崩溃把号推到 361（第 1 次挂载一次写 [1, 362) 共 361 行），之后每次挂载写一行：第 9 次取号 370、
/// 写满一片（369 行 + 链指针 = 370 条）照样成立，第 10 次（要取 371、这一版 369 行、要写 1 行）在取号之前拒绝。
/// 改之前第 10 次取号写完才 panic、号烧掉。
#[test]
fn writable_mounts_fill_the_instance_table_page_and_the_next_one_is_refused_before_acquisition() {
    let (mut devices, stream) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut devices, 360);
    for mount_number in 1..=9usize {
        let mounted = mount_writable(&parameters(), &mut devices)
            .unwrap_or_else(|error| panic!("第 {mount_number} 次可写挂载要成立：{error:?}"));
        assert_eq!(
            mounted.output.instance.0,
            u32::try_from(361 + mount_number).expect("号"),
            "第 {mount_number} 次挂载取的号"
        );
        assert_eq!(
            rows_in_the_row_publish(&mounted),
            360 + mount_number,
            "第 {mount_number} 次挂载写出的表里的行数"
        );
    }
    let before = snapshot(&devices, &stream);
    let instances_before = superblock_instances(&devices);
    assert_eq!(
        instances_before,
        vec![
            (DeviceIdentity(0), vec![InstanceGeneration(370); 2]),
            (DeviceIdentity(1), vec![InstanceGeneration(370); 2]),
        ],
        "第 9 次挂载取的号是 370"
    );
    let refused = mount_writable(&parameters(), &mut devices);
    assert_refused_before_acquisition(
        refused,
        (InstanceGeneration(371), ROWS_PER_PAGE, 1),
        &devices,
        &stream,
        &before,
        &instances_before,
    );
}

/// 一次挂载要写很多行：连着 368 次取号之后崩溃，下一次挂载取 370、给 [1, 370) 写 369 行，正好写满一片——放行；
/// 连着 369 次，要写 370 行——在取号之前拒绝。只按「每次挂载写一行」算的准入分不出这两格。
#[test]
fn mount_after_crashes_right_after_acquisition_may_fill_the_page_but_not_overflow_it() {
    let (mut filled_exactly, _) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut filled_exactly, 368);
    let mounted = mount_writable(&parameters(), &mut filled_exactly)
        .unwrap_or_else(|error| panic!("369 行正好写满一片，要放行：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(370));
    assert_eq!(mounted.output.rows_written.len(), ROWS_PER_PAGE);
    assert_eq!(rows_in_the_row_publish(&mounted), ROWS_PER_PAGE);

    let (mut one_row_too_many, stream) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut one_row_too_many, 369);
    let before = snapshot(&one_row_too_many, &stream);
    let instances_before = superblock_instances(&one_row_too_many);
    let refused = mount_writable(&parameters(), &mut one_row_too_many);
    assert_refused_before_acquisition(
        refused,
        (InstanceGeneration(371), 0, ROWS_PER_PAGE + 1),
        &one_row_too_many,
        &stream,
        &before,
        &instances_before,
    );
}

/// 回退按自己那一版算：连着 367 次取号之后崩溃、再可写挂载一次（取 369，表 368 行），之后回退到 A（实例 1、txg 3，它指着的表 0 行）
/// 要写 [1, 370) 共 369 行——0 + 369 + 1 = 370，放行；拿最新那张表的 368 行去算就会误拒。
/// 连着 368 次取号之后崩溃、再挂一次（取 370，表 369 行）之后回退到 A 要写 370 行——在取号之前拒绝。
#[test]
fn rollback_counts_the_rows_of_the_table_it_rolls_back_to() {
    let target = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    };

    let (mut fits, _) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut fits, 367);
    let mounted_one_row_short =
        mount_writable(&parameters(), &mut fits).expect("368 行，可写挂载放行");
    assert_eq!(
        rows_in_the_row_publish(&mounted_one_row_short),
        ROWS_PER_PAGE - 1
    );
    let rolled_back = mount_rollback(&parameters(), &mut fits, target, ShadowLedger::On)
        .unwrap_or_else(|error| panic!("回退到 A 要写 369 行，正好写满一片：{error:?}"));
    assert_eq!(rolled_back.output.instance, InstanceGeneration(370));
    assert_eq!(rolled_back.output.rows_written.len(), ROWS_PER_PAGE);
    assert!(
        rolled_back.output.rows_written[0].is_rollback,
        "第一行是 A 那个实例的回退行"
    );
    assert_eq!(rows_in_the_row_publish(&rolled_back), ROWS_PER_PAGE);

    let (mut overflows, stream) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut overflows, 368);
    let mounted_full_page =
        mount_writable(&parameters(), &mut overflows).expect("369 行，可写挂载放行");
    assert_eq!(rows_in_the_row_publish(&mounted_full_page), ROWS_PER_PAGE);
    let before = snapshot(&overflows, &stream);
    let instances_before = superblock_instances(&overflows);
    let refused = mount_rollback(&parameters(), &mut overflows, target, ShadowLedger::On);
    assert_refused_before_acquisition(
        refused,
        (InstanceGeneration(371), 0, ROWS_PER_PAGE + 1),
        &overflows,
        &stream,
        &before,
        &instances_before,
    );
}
