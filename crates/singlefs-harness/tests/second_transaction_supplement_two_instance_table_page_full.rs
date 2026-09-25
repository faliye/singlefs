//! 里程碑「第二个事务」增补 2 收口，代码三方第二轮判决（`research/prompts/m2-wave2-code-r1-main-verification.md`）第二节第 2 行（Z1-a）：
//! 每次可写挂载都给上一个实例写一行（D18（块里携带什么信息） 已定项 11），一片实例表 370 条记录，链指针记录恒为最后一条、
//! 数据行至多 369。改之前取号之前的准入不算实例表：第 370 次可写挂载取号写完之后在装实例表单元时越界 panic（`bytes.rs` 的
//! `ByteWriter`），每试一次再烧一个实例代号（攻方探针 `research/prompts/m2-wave2-code-r1-opus-model/opus_probe_pristine.rs` 的
//! `pristine_instance_table_rows_overflow_after_acquisition`，日志 `pristine-run.log`）。
//! 用户 2026-09-19 定第二片进里程碑二（增补 2 收口表第 38 行），2026-09-24 定两条写法：行一片写满 369 行再开下一片、
//! 多于一片时在 bump 次序里尾片先。所以一片写满之后的那一次挂载不再拒，写第二片（真写者写出的两片怎么排、checker 怎么判见
//! `second_transaction_supplement_two_instance_table_second_page_write.rs`）；回退走同一个 `establish_instance`，
//! 行数按 R_old 那一版表与 [max(r_old, 1), 新实例) 自己算。
//!
//! 「取号之后崩溃」（取号写完、写行那次发布之前掉电）是今天就走得到的历史：下一次挂载要给中间那些实例各补一行 (i, 0, 0)，
//! 一次挂载就能写很多行。本文件用它把表快速填到边上（`acquire_instance` 连取 k 次号 = 连着 k 次取号之后崩溃）。

mod common;

use common::{memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::{InstanceTablePage, InstanceTablePageIndex};
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{instance_table_chain_of_root, PoolReader};
use singlefs_core::transaction::{acquire_instance, PoolWriter};
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

/// 连着 `count` 次「取号之后崩溃」：每次取一个新号写进两块盘的系统配置，写行那次发布没发出去。
fn crash_right_after_acquisition(devices: &mut Devices, count: u32) {
    let publish_parameters = parameters();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    for _ in 0..count {
        acquire_instance(&mut writer).expect("取号");
    }
}

/// 写行那次发布写出的实例表每一片的行数，按链上的次序：从盘上沿链读（根记录指着第 0 片）。
fn rows_of_each_page_of_the_row_publish(devices: &Devices, mounted: &Mounted) -> Vec<usize> {
    let chain = instance_table_chain_of_root(devices, mounted.output.row_publish.root())
        .expect("写出的实例表沿链读得出、解得开");
    chain
        .page_pointers
        .iter()
        .enumerate()
        .map(|(position, pointer)| {
            let location = pointer.locations[0];
            let bytes = PoolReader::read(
                devices.as_slice(),
                location.device,
                location.slot.to_device_offset(),
                32768,
            )
            .expect("那一片读得到");
            InstanceTablePage::parse(
                &bytes,
                InstanceTablePageIndex(u64::try_from(position).expect("片序号")),
            )
            .expect("那一片按第几片解得开")
            .rows
            .len()
        })
        .collect()
}

/// 验收：一路可写挂载过一片。先连着 360 次取号之后崩溃把号推到 361（第 1 次挂载一次写 [1, 362) 共 361 行），之后每次挂载写一行：
/// 第 9 次取号 370、写满一片（369 行 + 链指针 = 370 条）；**第 10 次取号 371**（这一版 369 行、要写 1 行）在改之前于取号之前拒绝，
/// 现在写第二片：第 0 片 369 行、第 1 片 1 行，池级 checker 全绿。
#[test]
fn writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page(
) {
    let (mut devices, _stream) = pool_after_the_first_transaction();
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
            rows_of_each_page_of_the_row_publish(&devices, &mounted),
            vec![360 + mount_number],
            "第 {mount_number} 次挂载写出的表：一片"
        );
    }
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("第 10 次可写挂载（取号 371）要成立：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(371));
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&devices, &mounted),
        vec![ROWS_PER_PAGE, 1],
        "第 0 片写满 369 行、第 1 片装剩下的 1 行"
    );
    let verdicts = check_pool_image(&memory_pool_of_sparse_devices(&devices));
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(violated, Vec::<&str>::new(), "{verdicts:?}");
}

/// 一次挂载要写很多行：连着 368 次取号之后崩溃，下一次挂载取 370、给 [1, 370) 写 369 行，正好写满一片；
/// 连着 369 次，要写 370 行——一片写满 369 行、第二片装 1 行。
#[test]
fn mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page(
) {
    let (mut filled_exactly, _) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut filled_exactly, 368);
    let mounted = mount_writable(&parameters(), &mut filled_exactly)
        .unwrap_or_else(|error| panic!("369 行正好写满一片，要放行：{error:?}"));
    assert_eq!(mounted.output.instance, InstanceGeneration(370));
    assert_eq!(mounted.output.rows_written.len(), ROWS_PER_PAGE);
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&filled_exactly, &mounted),
        vec![ROWS_PER_PAGE]
    );

    let (mut one_row_more, _) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut one_row_more, 369);
    let mounted_past_one_page = mount_writable(&parameters(), &mut one_row_more)
        .unwrap_or_else(|error| panic!("370 行写两片：{error:?}"));
    assert_eq!(
        mounted_past_one_page.output.instance,
        InstanceGeneration(371)
    );
    assert_eq!(
        mounted_past_one_page.output.rows_written.len(),
        ROWS_PER_PAGE + 1
    );
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&one_row_more, &mounted_past_one_page),
        vec![ROWS_PER_PAGE, 1]
    );
}

/// 回退按自己那一版算：连着 367 次取号之后崩溃、再可写挂载一次（取 369，表 368 行），之后回退到 A（实例 1、txg 3，它指着的表 0 行）
/// 要写 [1, 370) 共 369 行——正好写满一片；拿最新那张表的 368 行去算就会多算出一片。
/// 连着 368 次取号之后崩溃、再挂一次（取 370，表 369 行）之后回退到 A 要写 370 行——A 那张表是一片，这次写成两片（369 + 1），
/// 第一行是 A 那个实例的回退行。
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
        rows_of_each_page_of_the_row_publish(&fits, &mounted_one_row_short),
        vec![ROWS_PER_PAGE - 1]
    );
    let rolled_back = mount_rollback(&parameters(), &mut fits, target, ShadowLedger::On)
        .unwrap_or_else(|error| panic!("回退到 A 要写 369 行，正好写满一片：{error:?}"));
    assert_eq!(rolled_back.output.instance, InstanceGeneration(370));
    assert_eq!(rolled_back.output.rows_written.len(), ROWS_PER_PAGE);
    assert!(
        rolled_back.output.rows_written[0].is_rollback,
        "第一行是 A 那个实例的回退行"
    );
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&fits, &rolled_back),
        vec![ROWS_PER_PAGE]
    );

    let (mut past_one_page, _) = pool_after_the_first_transaction();
    crash_right_after_acquisition(&mut past_one_page, 368);
    let mounted_full_page =
        mount_writable(&parameters(), &mut past_one_page).expect("369 行，可写挂载放行");
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&past_one_page, &mounted_full_page),
        vec![ROWS_PER_PAGE]
    );
    let rolled_back_past_one_page =
        mount_rollback(&parameters(), &mut past_one_page, target, ShadowLedger::On)
            .unwrap_or_else(|error| panic!("回退到 A 要写 370 行，写成两片：{error:?}"));
    assert_eq!(
        rolled_back_past_one_page.output.instance,
        InstanceGeneration(371)
    );
    assert_eq!(
        rolled_back_past_one_page.output.rows_written.len(),
        ROWS_PER_PAGE + 1
    );
    assert!(rolled_back_past_one_page.output.rows_written[0].is_rollback);
    assert_eq!(
        rows_of_each_page_of_the_row_publish(&past_one_page, &rolled_back_past_one_page),
        vec![ROWS_PER_PAGE, 1]
    );
    let verdicts = check_pool_image(&memory_pool_of_sparse_devices(&past_one_page));
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated,
        Vec::<&str>::new(),
        "回退写出的两片上 checker 全绿：{verdicts:?}"
    );
}
