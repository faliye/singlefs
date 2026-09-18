//! m2-wave2-code-r1 云端攻方腿（Opus）Z2 探针：失败账。未插桩副本上跑（crates/ 与开工快照逐字节相同）。
//! 设备栈同 `second_transaction_supplement_one_write_accounting.rs`：录制器 → 按开关报错的一层 → 稀疏内存盘；报错那次写不进录制流、不进按种类的账。
//! z2a：发布 C 在最后一步（盘 1 的超级块槽写）失败——根槽 FUA 已经落盘——之后不重试 C，同一个写入口、同一个分配器改发一次空发布 E（换一条路）。
//!      账：C 的失败账 + E 的账 == 录制器；再让 E 在 journal 记录那一步失败（E 的单元已落两盘、E 的根没写）然后重开。
//! z2b：同一个写入口连着失败两次再成功；加一次在落盘之前就失败的（内容装不下一个数据单元），看失败账的份数。

use std::cell::Cell;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_version, warm_up, FirstFile,
    InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput,
};
use singlefs_core::write_accounting::{WriteCallsAndBytes, WrittenStructureKind};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::{RecordedOperation, RecordedOperationKind, RecordingBlockDevice, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
type Remaining = Rc<Cell<Option<u64>>>;

struct WriteFailingDevice {
    inner: SparseBlockDevice,
    remaining: Remaining,
}

impl BlockDevice for WriteFailingDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        match self.remaining.get() {
            None => {}
            Some(0) => return Err(BlockDeviceError::InputOutput(std::io::Error::other("注入的设备写错"))),
            Some(left) => self.remaining.set(Some(left - 1)),
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

type Device = RecordingBlockDevice<WriteFailingDevice>;

struct Built {
    devices: Vec<(DeviceIdentity, Device)>,
    faults: Vec<Remaining>,
    stream: SharedStream,
    allocator: PoolAllocator,
    b: TransactionOutput,
}

fn build() -> Built {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let faults: Vec<Remaining> = (0..2).map(|_| Rc::new(Cell::new(None))).collect();
    let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    WriteFailingDevice {
                        inner: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        remaining: faults[usize::try_from(number).expect("盘号")].clone(),
                    },
                    stream.clone(),
                ),
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
    let b = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &first_file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个事务");
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &first,
            FirstFile { content: &[7u8; 4100], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            instance,
        )
        .expect("发布 B")
    };
    Built { devices, faults, stream, allocator, b }
}

fn recorded(operations: &[RecordedOperation]) -> WriteCallsAndBytes {
    operations.iter().fold(WriteCallsAndBytes::NONE, |sum, operation| match operation.kind {
        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => sum.plus(WriteCallsAndBytes { write_calls: 1, written_bytes: operation.length }),
        RecordedOperationKind::Barrier => sum,
    })
}

fn empty_plan(previous: &TransactionOutput) -> PublishPlan<'static> {
    PublishPlan {
        txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
        counter: previous.record.counter + 1,
        transaction: 0,
        instance: InstanceGeneration(1),
        back_chain: back_chain_of(&previous.record_bytes),
        file: None,
        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: previous.root.tree_identifier_watermark,
        rollback_floor: previous.root.rollback_floor,
    }
}

fn image(devices: &[(DeviceIdentity, Device)]) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.inner().inner.image.clone())).collect::<BTreeMap<_, _>>(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn violated(devices: &[(DeviceIdentity, Device)]) -> Vec<String> {
    check_pool_image(&image(devices))
        .iter()
        .filter(|(_, verdict)| format!("{verdict:?}").starts_with("Violated"))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

#[test]
fn z2a_failure_after_the_root_is_durable_then_a_different_publish() {
    let mut built = build();
    let parameters = e142_parameters(512, 512);
    let window_start = built.stream.operations().len();
    let b = built.b.clone();
    println!("B: txg={} units={:?}", b.root.checkpoint_txg.0, b.units.iter().map(|unit| (unit.identity, unit.slot.0)).collect::<Vec<_>>());
    // C 的 txg 5 落区域 2 = 盘 0；盘 1 上 C 的写：8 个单元 + 1 条记录 = 9 次，第 10 次（盘 1 的超级块槽）报错。
    built.faults[1].set(Some(9));
    let mut writer = PoolWriter::new(&parameters, built.devices.as_mut_slice());
    let c = publish_overwrite(
        &mut writer,
        &mut built.allocator,
        &b,
        FirstFile { content: &[9u8; 3700], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 },
        InstanceGeneration(1),
    );
    println!("C result: {:?}", c.as_ref().map(|_| "ok").map_err(|error| format!("{error:?}")));
    let failed_c = writer.writes_of_failed_publishes().to_vec();
    println!(
        "C failure account: total={:?} root_slot={:?} superblock={:?} journal={:?}",
        failed_c[0].total(),
        failed_c[0].of(WrittenStructureKind::RootSlot),
        failed_c[0].of(WrittenStructureKind::SuperblockSlot),
        failed_c[0].of(WrittenStructureKind::JournalRecord)
    );
    built.faults[1].set(None);
    // 换一条路：不重试 C，改发空发布 E（同一个上一版 B ⇒ 同一个 txg 5）；让 E 在盘 0 的 journal 记录那一步失败——
    // 单元按「每个单元两盘各一次」写，盘 0 放行 4 次写 = E 的四个固定点单元，第 5 次（盘 0 的记录）报错：E 的单元已落两盘、E 的根没写。
    built.faults[0].set(Some(4));
    let e = publish_version(&mut writer, &mut built.allocator, empty_plan(&b), Some(&b));
    println!("E result: {:?}", e.as_ref().map(|_| "ok").map_err(|error| format!("{error:?}")));
    let failed_all = writer.writes_of_failed_publishes().to_vec();
    drop(writer);
    built.faults[0].set(None);
    let sum = failed_all.iter().fold(WriteCallsAndBytes::NONE, |sum, account| sum.plus(account.total()));
    let device_level = recorded(&built.stream.operations()[window_start..]);
    println!("failure accounts: {} entries, sum={sum:?}; device level in window={device_level:?}; equal={}", failed_all.len(), sum == device_level);
    println!("checker on the image now: {:?}", violated(&built.devices));
    // 断电重开（故障都清了）。
    let result = catch_unwind(AssertUnwindSafe(|| mount_writable(&parameters, &mut built.devices)));
    match result {
        Ok(Ok(mounted)) => println!("remount: mounted, chosen root txg={} instance={}", mounted.output.chosen_root.checkpoint_txg.0, mounted.output.instance.0),
        Ok(Err(error)) => println!("remount: REFUSED {error:?}"),
        Err(payload) => println!("remount: PANICKED {:?}", payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))),
    }
}

/// 对照：同一段历史，但 C 失败之后原样重试 C（同内容），E 不发。
#[test]
fn z2a_control_retry_the_same_publish() {
    let mut built = build();
    let parameters = e142_parameters(512, 512);
    let window_start = built.stream.operations().len();
    let b = built.b.clone();
    built.faults[1].set(Some(9));
    let mut writer = PoolWriter::new(&parameters, built.devices.as_mut_slice());
    let first_try = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &[9u8; 3700], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 }, InstanceGeneration(1));
    built.faults[1].set(None);
    let retry = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &[9u8; 3700], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 }, InstanceGeneration(1)).expect("重试");
    let failed = writer.writes_of_failed_publishes().to_vec();
    drop(writer);
    let sum = failed.iter().fold(retry.writes.total(), |sum, account| sum.plus(account.total()));
    println!("control: first try err={} failure entries={} sum={sum:?} device={:?}", first_try.is_err(), failed.len(), recorded(&built.stream.operations()[window_start..]));
    let result = catch_unwind(AssertUnwindSafe(|| mount_writable(&parameters, &mut built.devices)));
    match result {
        Ok(Ok(mounted)) => println!("control remount: mounted, chosen root txg={}", mounted.output.chosen_root.checkpoint_txg.0),
        Ok(Err(error)) => println!("control remount: REFUSED {error:?}"),
        Err(_) => println!("control remount: PANICKED"),
    }
}

#[test]
fn z2b_two_failures_then_success_and_a_failure_before_any_write() {
    let mut built = build();
    let parameters = e142_parameters(512, 512);
    let window_start = built.stream.operations().len();
    let b = built.b.clone();
    let mut writer = PoolWriter::new(&parameters, built.devices.as_mut_slice());
    let content = [5u8; 3000];
    built.faults[1].set(Some(2));
    let first = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1 }, InstanceGeneration(1));
    built.faults[1].set(None);
    built.faults[0].set(Some(8));
    let second = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1 }, InstanceGeneration(1));
    built.faults[0].set(None);
    let too_big = vec![1u8; 40_000];
    let before_any_write = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &too_big, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1 }, InstanceGeneration(1));
    let success = publish_overwrite(&mut writer, &mut built.allocator, &b, FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1 }, InstanceGeneration(1)).expect("第四次成功");
    let failed = writer.writes_of_failed_publishes().to_vec();
    drop(writer);
    let sum = failed.iter().fold(success.writes.total(), |sum, account| sum.plus(account.total()));
    println!(
        "results: first={:?} second={:?} before_any_write={:?}",
        first.as_ref().err().map(|error| format!("{error:?}").chars().take(40).collect::<String>()),
        second.as_ref().err().map(|error| format!("{error:?}").chars().take(40).collect::<String>()),
        before_any_write.as_ref().err().map(|error| format!("{error:?}").chars().take(60).collect::<String>())
    );
    println!("failed publishes: 3; failure account entries: {}; per entry totals: {:?}", failed.len(), failed.iter().map(|account| account.total()).collect::<Vec<_>>());
    println!("sum(failure accounts)+success={sum:?}; device level={:?}", recorded(&built.stream.operations()[window_start..]));
    let _ = PublishError::NoSpaceFor { unit: singlefs_core::transaction::TransactionUnit::Data };
}

/// 调用方不取时：可写挂载里第二次暖机的第一个单元写在盘 1 上报错——取号、写行、第一次暖机都已成功落盘，
/// `MountError` 交回之后这几次成功发布的账与失败那一份都拿不到。
#[test]
fn z2c_mount_that_fails_at_the_second_warm_up_hands_back_no_account() {
    let mut built = build();
    let parameters = e142_parameters(512, 512);
    let window_start = built.stream.operations().len();
    // 盘 1：取号 1 + 写行（5 个单元 + 1 条记录 + 1 个超级块槽）7 + 第一次暖机（4 + 1 + 1）6 = 14 次，第 15 次（第二次暖机的第一个单元）报错。
    built.faults[1].set(Some(14));
    let result = mount_writable(&parameters, &mut built.devices);
    built.faults[1].set(None);
    let device_level = recorded(&built.stream.operations()[window_start..]);
    match result {
        Ok(_) => println!("z2c: mounted (fault point wrong)"),
        Err(error) => println!("z2c: mount error = {error:?}; device level writes in the mount window = {device_level:?}"),
    }
}
