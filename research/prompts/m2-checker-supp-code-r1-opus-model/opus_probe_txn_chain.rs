//! 云端攻方腿 m2-checker-supp-code-r1（K1 / K2）的探针，只在副本上跑，不入库。
//! 两格：① 发布在根槽那一步失败之后，同一个写入口接着发布拿到同一个事务号（链只活在成功值里）；
//! ② 从盘上重建的那一版带着「这条记录上的事务号」，拿它接着发布就重号。

use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::recovery::{rebuild_version, recover, JournalPolicy, RebuiltVersion};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError, TransactionOutput,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_core::unit::parse_data_unit;
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::SparseBlockDevice;

const IMAGE_BYTES: u64 = 4 << 30;
const FILE_BYTES: usize = 3000;
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 写到某一个落点就报错的包装：探针要的是「单元写与记录写都成了、根槽那一步报错」。
struct FailingAtOffset {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    armed: Rc<Cell<Option<(DeviceIdentity, u64)>>>,
}

impl BlockDevice for FailingAtOffset {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.armed.get() == Some((self.identity, offset.0)) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "探针注入：这一次写报错",
            )));
        }
        self.inner.write_at(offset, bytes, durability)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

struct Probe {
    devices: Vec<(DeviceIdentity, FailingAtOffset)>,
    allocator: PoolAllocator,
    output: TransactionOutput,
    armed: Rc<Cell<Option<(DeviceIdentity, u64)>>>,
}

fn build() -> Probe {
    let parameters = parameters();
    let armed: Rc<Cell<Option<(DeviceIdentity, u64)>>> = Rc::new(Cell::new(None));
    let mut devices: Vec<(DeviceIdentity, FailingAtOffset)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (
                identity,
                FailingAtOffset {
                    identity,
                    inner: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    armed: armed.clone(),
                },
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content: Vec<u8> = (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务")
    };
    Probe { devices, allocator, output, armed }
}

fn overwrite_in(
    probe: &mut Probe,
    seed: usize,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let previous = probe.output.clone();
    let content = content_of(4100, seed);
    let mut pool = PoolWriter::new(&parameters, &mut probe.devices);
    publish_overwrite(
        &mut pool,
        &mut probe.allocator,
        &previous,
        FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
        instance,
    )
}

fn overwrite(probe: &mut Probe, seed: usize) -> Result<TransactionOutput, PublishError> {
    overwrite_in(probe, seed, InstanceGeneration(1))
}

/// 单元区头 `slots` 个槽上解得开的码 1 数据单元：(盘, 槽, 实例, 事务号, 诞生代号)。
fn scan_data_units(probe: &Probe, slots: u64) -> Vec<(u32, u64, u32, u64, u64)> {
    let mut found = Vec::new();
    let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    for (identity, device) in &probe.devices {
        for step in 0..slots {
            let offset = DeviceOffsetInBytes((UNIT_AREA_START_SLOT + step) * SLOT_BYTES);
            let mut bytes = vec![0u8; unit_bytes];
            if device.read_at(offset, &mut bytes).is_err() {
                continue;
            }
            if let Ok(header) = parse_data_unit(&bytes) {
                found.push((
                    identity.0,
                    UNIT_AREA_START_SLOT + step,
                    header.write_order.instance.0,
                    header.write_order.transaction,
                    header.birth_txg.0,
                ));
            }
        }
    }
    found
}

fn read_record(probe: &Probe, counter: u64) -> Option<JournalRecord> {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    let mut bytes = vec![0u8; 4096];
    probe.devices[0].1.read_at(offset, &mut bytes).expect("读记录");
    JournalRecord::parse(&bytes, unit_filesystem_identifier(&E142_FILESYSTEM_IDENTIFIER))
}

/// K1 探针①：发布在根槽那一步报错 ⇒ 那次的记录（事务号 N、提交标记 1）已经持久，而链只活在成功返回的
/// `TransactionOutput` 里；同一个写入口拿同一个 `previous` 接着发布，拿到的还是 N（jsn 也一样）。
/// `crates/` 里没有实例切换（C381（根已落盘之后发布失败，分配器仍退回） 那一行逐字：探针写、只读复核、实例切换、转只读四样都没有）。
#[test]
fn a_publish_that_fails_at_the_root_slot_burns_a_transaction_number_that_the_next_publish_takes_again() {
    let mut probe = build();
    assert_eq!(probe.output.record.transaction, 1, "第一个事务是 1");
    let first = overwrite(&mut probe, 3).expect("A");
    probe.output = first.clone();
    assert_eq!(first.record.transaction, 2);
    let second = overwrite(&mut probe, 5).expect("B");
    probe.output = second.clone();
    assert_eq!(second.record.transaction, 3);

    let next_txg = CheckpointTxg(second.root.checkpoint_txg.0 + 1);
    let target = target_for_publish(next_txg);
    let region = usize::try_from(target.region).expect("区域号");
    let device = parameters().region_devices[region];
    let offset = slot_offset(target, 4096);
    probe.armed.set(Some((device, offset.0)));
    let failed = overwrite(&mut probe, 7).expect_err("根槽那一步报错");
    assert!(
        matches!(failed, PublishError::BlockDevice(_)),
        "报的是块设备错：{failed:?}"
    );
    probe.armed.set(None);

    let burned = read_record(&probe, second.record.counter + 1).expect("失败那次的记录留在环里");
    assert_eq!(burned.transaction, 4, "失败那次已经把 4 写进了环");
    assert!(burned.is_commit, "提交标记 1");
    assert_eq!(burned.instance, InstanceGeneration(1));

    let retried = overwrite(&mut probe, 9).expect("同一个写入口接着发布");
    assert_eq!(
        retried.record.transaction, burned.transaction,
        "同一个实例里第二次拿到同一个事务号"
    );
    assert_eq!(
        retried.record.counter,
        second.record.counter + 1,
        "jsn 计数器也重号"
    );
    assert_eq!(retried.root.instance, InstanceGeneration(1), "实例没换");

    // 盘上还剩几个写序 (1, 4) 的码 1 单元：失败那次的单元被重试那次盖掉了没有。
    let units = scan_data_units(&probe, 64);
    eprintln!("单元区头 64 槽上解得开的码 1 单元：{units:?}");
    let colliding: Vec<_> = units
        .iter()
        .filter(|(_, _, instance, transaction, _)| *instance == 1 && *transaction == 4)
        .collect();
    eprintln!("写序 (1, 4) 的：{colliding:?}");
    eprintln!("重试那次的数据单元槽：{:?}", retried.unit(singlefs_core::transaction::TransactionUnit::Data).slot);

    // 重试之后整池还读不读得回来。
    let report = recover(&probe.devices, JournalPolicy::Consult);
    eprintln!("重试之后 recover：{:?}", report.outcome);
}

/// K1 探针②：`recovery::rebuild_version` 取「这条记录上的事务号」。最后一条记录是空发布时它是 0，
/// 拿这一版接着在同一个实例上发布，事务号退回 1，与这个实例早先用过的 1 重号。
#[test]
fn publishing_on_a_version_rebuilt_from_disk_takes_a_transaction_number_the_instance_already_used() {
    let parameters = parameters();
    let mut probe = build();
    for seed in 0..2 {
        let next = overwrite(&mut probe, seed * 3 + 1).expect("实例 1 的覆盖写");
        probe.output = next;
    }
    assert_eq!(probe.output.record.transaction, 3, "实例 1 用过 1..3");

    let mounted = mount_writable(&parameters, &mut probe.devices).expect("重开");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    probe.allocator = mounted.allocator.clone();
    probe.output = mounted
        .current
        .file_version()
        .expect("重开之后现行那一版带文件")
        .clone();
    assert_eq!(
        probe.output.highest_transaction_number_in_this_instance, 0,
        "新实例从 0 起"
    );

    for seed in 0..3 {
        let next = overwrite_in(&mut probe, seed * 5 + 2, InstanceGeneration(2)).expect("覆盖写");
        probe.output = next;
    }
    assert_eq!(probe.output.record.transaction, 3, "实例 2 用过 1..3");

    let mut current = probe.output.clone();
    let mut raised_to = None;
    for floor in (1..=12u64).rev() {
        let attempt = raise_rollback_floor(
            &parameters,
            &mut probe.devices,
            &mut probe.allocator,
            &mut current,
            CheckpointTxg(floor),
            ShadowLedger::On,
        );
        match attempt {
            Ok(_) => {
                raised_to = Some(floor);
                break;
            }
            Err(error) => eprintln!("抬到 {floor} 不成：{error:?}"),
        }
    }
    let raised_to = raised_to.expect("抬得上去");
    eprintln!("抬到了 F = {raised_to}");
    assert_eq!(current.record.transaction, 0, "抬 F 推的空发布在记录上写 0");
    assert_eq!(
        current.highest_transaction_number_in_this_instance, 3,
        "内存里的链记着 3"
    );

    let rebuilt = match rebuild_version(&probe.devices, &current.root, Some(current.record.clone()))
        .expect("从盘上重建")
    {
        RebuiltVersion::WithFile(output) => output,
        RebuiltVersion::WithoutFile => panic!("这一版有文件"),
    };
    assert_eq!(
        rebuilt.highest_transaction_number_in_this_instance, 0,
        "重建取的是这条记录上的事务号，空发布是 0"
    );

    probe.output = rebuilt;
    let next = overwrite_in(&mut probe, 99, InstanceGeneration(2)).expect("在重建的那一版上接着发布");
    assert_eq!(next.record.transaction, 1, "退回 1：与实例 2 的第一个事务重号");
    assert_eq!(next.root.instance, InstanceGeneration(2));
}

/// 失败点与「失败之前发过几次」放开扫一遍：每一格都看两样——重试那次是不是拿到同一个事务号，
/// 盘 0 上还剩几个码 1 单元、写序去重之后剩几个（两数不等就是盘上真留下了写序相同的两版）。
#[test]
fn sweeping_the_failure_point_and_the_number_of_publishes_before_it() {
    #[derive(Clone, Copy, Debug)]
    enum FailurePoint {
        TheRootSlot,
        TheJournalRecord,
    }

    let mut lines: Vec<String> = Vec::new();
    for publishes_before in 0..4u64 {
        for point in [FailurePoint::TheRootSlot, FailurePoint::TheJournalRecord] {
            let mut probe = build();
            for seed in 0..publishes_before {
                let next = overwrite(&mut probe, usize::try_from(seed * 3 + 1).expect("种子"))
                    .expect("覆盖写");
                probe.output = next;
            }
            let previous = probe.output.clone();
            let expected = previous.highest_transaction_number_in_this_instance + 1;
            let next_txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
            let armed_at = match point {
                FailurePoint::TheRootSlot => {
                    let target = target_for_publish(next_txg);
                    let region = usize::try_from(target.region).expect("区域号");
                    (
                        parameters().region_devices[region],
                        slot_offset(target, 4096).0,
                    )
                }
                FailurePoint::TheJournalRecord => (
                    DeviceIdentity(0),
                    record_offset(previous.record.counter + 1, JOURNAL_RING_DEFAULT_BYTES).0,
                ),
            };
            probe.armed.set(Some(armed_at));
            let failed = overwrite(&mut probe, 7).expect_err("这一次报错");
            probe.armed.set(None);
            let record_after_the_failure = read_record(&probe, previous.record.counter + 1)
                .map(|record| record.transaction);
            let retried = overwrite(&mut probe, 9).expect("接着发布");
            let units = scan_data_units(&probe, 64);
            let mut orders: Vec<(u32, u64)> = units
                .iter()
                .filter(|(device, _, _, _, _)| *device == 0)
                .map(|(_, _, instance, transaction, _)| (*instance, *transaction))
                .collect();
            let total = orders.len();
            orders.sort_unstable();
            let mut deduplicated = orders.clone();
            deduplicated.dedup();
            lines.push(format!(
                "失败前发过 {publishes_before} 次 / 失败点 {point:?}：失败那次报 {}；环里那条记录的事务号 {record_after_the_failure:?}；重试拿到 {}（该拿 {expected}）；盘 0 码 1 单元 {total} 个、写序去重 {} 个",
                match failed {
                    PublishError::BlockDevice(_) => "BlockDevice",
                    _ => "别的",
                },
                retried.record.transaction,
                deduplicated.len()
            ));
        }
    }
    for line in &lines {
        eprintln!("{line}");
    }
}
