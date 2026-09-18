//! 攻方腿探针（仓副本，不入库）：Y1 回落与挡位 / Y2 拒绝与失败路径 / Y3 写量计数。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{PoolAllocator, ReclaimedReuse};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::superblock::Superblock;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, UNIT_AREA_START_SLOT};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 251).expect("小于 256"))
        .collect()
}

fn try_overwrite(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )?;
    pool.output = output.clone();
    Ok(output)
}

fn slot_of(output: &TransactionOutput, identity: TransactionUnit) -> u64 {
    output
        .units
        .iter()
        .find(|unit| unit.identity == identity)
        .expect("角色在这一版里")
        .slot
        .0
}

/// 每块盘上给每个全空段占掉段首一槽：之后一个全空段都没有（与 supplement_two_commit_generated_fallback.rs 同一个手法）。
fn kill_every_empty_segment(allocator: &mut PoolAllocator) {
    for device_map in &mut allocator.devices {
        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        for segment_index in 0..full_segments {
            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                device_map.mark_allocated(
                    SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
                    1,
                );
            }
        }
    }
}

/// 每块盘上把 `[UNIT_AREA_START_SLOT, upper)` 里还空着的槽全占上。
fn fill_below(allocator: &mut PoolAllocator, upper: u64) {
    for device_map in &mut allocator.devices {
        for slot in UNIT_AREA_START_SLOT..upper {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// Y1：回落把开放段置空之后，下一次发布的用户数据落进刚才那一段——那一段里装着这个池的提交内生块。
#[test]
fn y1_user_data_lands_inside_the_cluster_segment_that_the_fallback_closed() {
    let mut pool = build_pool("opus-y1");
    let instance = InstanceGeneration(1);
    let segment = pool
        .allocator
        .open_segment()
        .expect("第一个事务之后开放段开着")
        .0;
    println!("PROBE y1 open_segment_after_A={segment}");

    // ① 池里一个全空段都不剩：段耗尽之后只剩回落这一条路。
    kill_every_empty_segment(&mut pool.allocator);

    // ② 一直覆盖写，直到某一次发布走了回落（回落把开放段置空）。
    let mut publishes = 0u32;
    while pool.allocator.open_segment().is_some() {
        publishes += 1;
        assert!(publishes < 40, "40 次覆盖写还没把开放段用完");
        try_overwrite(&mut pool, &content_of(3000, publishes as usize), instance)
            .unwrap_or_else(|failure| panic!("第 {publishes} 次覆盖写：{failure:?}"));
    }
    println!("PROBE y1 fallback_at_publish={publishes} open_segment_now=None");

    // ③ 抬 F 之后的回收：释放代 ≤ 现在这个 txg 的已释放落点回到空闲（段里被 COW 换下的提交内生块就在其中）。
    let reclaimed =
        pool.allocator
            .reclaim_released_up_to(pool.output.root.checkpoint_txg, ReclaimedReuse::Immediately);
    println!("PROBE y1 reclaimed_placements={}", reclaimed.len());

    // ④ 段下方（[50176, 段起点)）没有空位了：真实老化里这一段迟早被用户数据与回落的提交内生块占满。
    fill_below(&mut pool.allocator, segment);
    let free_in_segment: Vec<u64> = (segment..segment + CLUSTER_SEGMENT_SLOTS)
        .filter(|slot| pool.allocator.devices[0].is_free(SlotNumber(*slot)))
        .collect();
    println!("PROBE y1 free_slots_inside_that_segment={free_in_segment:?}");

    // ⑤ 再发一版：数据单元先取落点，开放段已经是 None ⇒ 候选集不再排除那一段。
    let next = try_overwrite(&mut pool, &content_of(3000, 900), instance).expect("回落之后再发一版");
    let data_slot = slot_of(&next, TransactionUnit::Data);
    let commit_generated_in_segment: Vec<(String, u64)> = next
        .units
        .iter()
        .filter(|unit| {
            unit.identity != TransactionUnit::Data
                && unit.slot.0 >= segment
                && unit.slot.0 < segment + CLUSTER_SEGMENT_SLOTS
        })
        .map(|unit| (format!("{:?}", unit.identity), unit.slot.0))
        .collect();
    println!(
        "PROBE y1 data_slot={data_slot} commit_generated_of_the_same_publish_in_that_segment={commit_generated_in_segment:?}"
    );
    assert!(
        data_slot >= segment && data_slot < segment + CLUSTER_SEGMENT_SLOTS,
        "用户数据落进了刚才那个聚簇段：slot={data_slot}，段 [{segment}, {})",
        segment + CLUSTER_SEGMENT_SLOTS
    );
}

/// Y2：发布层的拒绝发生在取号之后——拒绝时盘上不是逐字节不变，实例代号已经推高、两条超级块槽已经落盘，而且不回卷。
#[test]
fn y2_a_refused_row_publish_leaves_the_bumped_instance_generation_on_disk() {
    let mut pool = build_pool("opus-y2");
    let instance = InstanceGeneration(1);
    let mut publishes = 0u32;
    let refusal = loop {
        publishes += 1;
        assert!(publishes < 80, "80 次覆盖写还没撞上分配记录树装不下");
        match try_overwrite(&mut pool, &content_of(3000, publishes as usize), instance) {
            Ok(_) => {}
            Err(failure) => break failure,
        }
    };
    println!(
        "PROBE y2 refused_at_publish={publishes} failure={refusal:?} records={}",
        pool.allocator.records().len()
    );

    let before: Vec<u32> = {
        let devices = pool.devices.as_ref().expect("镜像还开着");
        devices
            .iter()
            .map(|(_, device)| read_superblock_instance(device))
            .collect()
    };
    let operations_before = pool.stream.operations().len();

    let mut devices = pool.reopen_recorded();
    let mount = mount_writable(&parameters(), &mut devices);
    let after: Vec<u32> = devices
        .iter()
        .map(|(_, device)| read_superblock_instance(device))
        .collect();
    let operations_after = pool.stream.operations().len();
    println!(
        "PROBE y2 mount={:?} instance_before={before:?} instance_after={after:?} operations_added={}",
        mount.as_ref().err(),
        operations_after - operations_before
    );
    assert!(mount.is_err(), "可写挂载被写行那次发布挡住");
    assert_ne!(before, after, "取号那两条超级块槽写已经落盘、实例代号推高了");
}

/// 一块盘上两个超级块槽里自证过的最大实例代号。
fn read_superblock_instance<Device: BlockDevice>(device: &Device) -> u32 {
    (0..2u64)
        .filter_map(|slot| {
            let mut bytes = vec![0u8; 4096];
            device
                .read_at(DeviceOffsetInBytes(slot * 4096), &mut bytes)
                .ok()?;
            Superblock::parse_slot(&bytes).map(|superblock| superblock.journal_instance.0)
        })
        .max()
        .unwrap_or(u32::MAX)
}

/// 外层包一层：数交给设备的写（尝试数与真落盘数），并让第 `fail_at_attempt` 次写报错。
struct FailingDevice<Inner: BlockDevice> {
    inner: Inner,
    attempted_writes: u64,
    attempted_bytes: u64,
    landed_writes: u64,
    landed_bytes: u64,
    fail_at_attempt: Option<u64>,
}

impl<Inner: BlockDevice> FailingDevice<Inner> {
    fn new(inner: Inner) -> Self {
        Self {
            inner,
            attempted_writes: 0,
            attempted_bytes: 0,
            landed_writes: 0,
            landed_bytes: 0,
            fail_at_attempt: None,
        }
    }
}

impl<Inner: BlockDevice> BlockDevice for FailingDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.attempted_writes += 1;
        self.attempted_bytes += u64::try_from(bytes.len()).expect("长度");
        if self.fail_at_attempt == Some(self.attempted_writes) {
            return Err(BlockDeviceError::OutOfRange {
                offset,
                length: u64::try_from(bytes.len()).expect("长度"),
                device_size: self.inner.size_in_bytes(),
            });
        }
        self.inner.write_at(offset, bytes, durability)?;
        self.landed_writes += 1;
        self.landed_bytes += u64::try_from(bytes.len()).expect("长度");
        Ok(())
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

/// Y3：一次发布写到一半被设备拒掉，之后拿同一个上一版重试——失败那次已经落盘的写不属于任何一次发布的账，
/// 这一段窗口里「按种类的合计」与设备一层的数对不上。
#[test]
fn y3_a_failed_publish_and_its_retry_break_the_per_publish_equality_with_the_device_layer() {
    let mut pool = build_pool("opus-y3");
    let instance = InstanceGeneration(1);
    let publish_parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, FailingDevice<common::Recorded>)> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| (identity, FailingDevice::new(device)))
        .collect();

    // 第 6 次写（16 个单元写里的一次）报错：写到一半的发布。
    for (_, device) in devices.iter_mut() {
        device.fail_at_attempt = Some(3);
    }
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let failed = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content_of(3000, 1),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    );
    drop(writer);
    println!("PROBE y3 first_attempt={:?}", failed.as_ref().err());
    assert!(failed.is_err(), "第一次发布被设备拒掉");

    // 拿同一个上一版重试（`publish_version` 把分配器退回进来时的样子，就是为了这一步）。
    for (_, device) in devices.iter_mut() {
        device.fail_at_attempt = None;
    }
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let retried = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content_of(3000, 1),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("重试成功");
    drop(writer);

    let account = retried.writes.total();
    let attempted: u64 = devices
        .iter()
        .map(|(_, device)| device.attempted_writes)
        .sum();
    let attempted_bytes: u64 = devices
        .iter()
        .map(|(_, device)| device.attempted_bytes)
        .sum();
    let landed: u64 = devices.iter().map(|(_, device)| device.landed_writes).sum();
    let landed_bytes: u64 = devices.iter().map(|(_, device)| device.landed_bytes).sum();
    println!(
        "PROBE y3 by_kind_write_calls={} by_kind_written_bytes={} device_attempted_writes={attempted} device_attempted_bytes={attempted_bytes} device_landed_writes={landed} device_landed_bytes={landed_bytes}",
        account.write_calls, account.written_bytes
    );
    assert_ne!(
        account.write_calls, landed,
        "失败那次落盘的写不属于任何一次发布的账"
    );
}

/// Y1 之二：记账行「全空聚簇段数」只看已分配位，影子账隔离与抬 F 扣住的段照样算进去；
/// 而 `lowest_empty_segment` 会跳过它们 ⇒ 记账报出来的全空段数里有分配器永远开不了的段。
#[test]
fn y1b_the_accounting_row_counts_segments_the_allocator_will_never_open() {
    let mut pool = build_pool("opus-y1b");
    let segment_one = UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS; // 50240 是开放段，50304 是下一段
    let target = segment_one + CLUSTER_SEGMENT_SLOTS;
    let before_rows = pool.allocator.devices[0].empty_segments();
    let before_open = pool.allocator.open_segment();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot in target..target + CLUSTER_SEGMENT_SLOTS {
            pool.allocator.isolate_abandoned(device, SlotNumber(slot), 1);
        }
    }
    let after_rows = pool.allocator.devices[0].empty_segments();
    let openable = pool.allocator.devices[0].lowest_empty_segment().map(|s| s.0);
    println!(
        "PROBE y1b isolated_segment={target} empty_segments_row_before={before_rows} empty_segments_row_after={after_rows} lowest_empty_segment_now={openable:?} open_segment_before={before_open:?}"
    );
    assert_eq!(
        before_rows, after_rows,
        "整段 64 槽被影子账隔离之后，记账行「全空聚簇段数」一个都没减"
    );
    assert_ne!(
        openable,
        Some(target),
        "分配器不会再开这一段"
    );
}
