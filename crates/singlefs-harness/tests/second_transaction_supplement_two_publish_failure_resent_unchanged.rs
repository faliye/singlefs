//! 这一版的失败处置（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」，用户 2026-09-24 定）：发布不接受失败。
//! 一次发布落盘中途失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、
//! 单元的位置与字节都不变），重发成功才建下一次发布——于是同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录。
//!
//! 历史：A（第一个文件，txg 3，jsn 3）→ 顺序写两个数据单元的 B（txg 4，jsn 4、5，两条记录）：盘 1 写 jsn 5 那一槽时报块设备错，
//! 盘 0 上 jsn 5 已经落了。改之前接着在 A 上发一次一个数据单元的覆盖写 C：它也是 txg 4、jsn 4，只写一条记录，盘 0 上 B 的 jsn 5
//! 留在环里、与 C 共享 (实例 1, txg 4)、各带一个末条标志——C 的根落盘之后，恢复停在
//! `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` 上（实二十报告第六节 1）。

mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, resend_the_frozen_publish, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::fault_injection::injected_block_device_error;

/// 一块盘上某一个偏移的第一次写报块设备错（什么都不写），之后照转；别的写、读、屏障都照转。
struct FirstWriteAtOneOffsetFails<Inner: BlockDevice> {
    inner: Inner,
    failing_offset: Option<DeviceOffsetInBytes>,
    writes_refused: u64,
}

impl<Inner: BlockDevice> BlockDevice for FirstWriteAtOneOffsetFails<Inner> {
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
        if self.failing_offset == Some(offset) && self.writes_refused == 0 {
            self.writes_refused += 1;
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
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

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 1) % 241).expect("小于 256"))
        .collect()
}

/// 在 A 上顺序写 B（两个数据单元、两条记录），盘 1 上 jsn 5 那一槽的第一次写报块设备错。交回发布的结局与这次挂着的写数到了几次拒写。
fn sequential_write_with_the_second_record_refused_on_disk_one(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
    content: &[u8],
) -> (Result<TransactionOutput, PublishError>, u64) {
    let second_record_offset =
        record_offset(previous.record.counter + 2, JOURNAL_RING_DEFAULT_BYTES);
    let mut wrapped: Vec<(DeviceIdentity, FirstWriteAtOneOffsetFails<Recorded>)> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstWriteAtOneOffsetFails {
                    inner: device,
                    failing_offset: (identity == DeviceIdentity(1)).then_some(second_record_offset),
                    writes_refused: 0,
                },
            )
        })
        .collect();
    let result = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            previous,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
    };
    let writes_refused = wrapped
        .iter()
        .map(|(_, device)| device.writes_refused)
        .sum();
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    (result, writes_refused)
}

fn overwrite(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
    content: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        InstanceGeneration(1),
    )
}

/// 验收：注入一次写失败（B 的第二条记录在盘 1 上写失败）。
/// 1. 失败之后 B 冻结在分配器上：在 A 上再发 C 被拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），一个写、一道屏障都没发；
/// 2. 原样重发：失败那一遍真落到盘上的每一步（设备、种类、偏移、字节）是重发这一遍的前缀，逐字节相同；重发交回的 B 仍是 txg 4、jsn 4 与 5；
/// 3. 之后的发布照常：C 接在重发的 B 上，txg 5；
/// 4. 冷启动恢复读回 C，不撞「所选根那次发布带两个末条标志」；池级 checker 的 I-8.9 成立。
#[test]
fn a_publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish() {
    let mut pool = build_pool("publish-failure-resent-unchanged");
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let operations_before_the_failed_attempt = pool.retained_operations().len();

    let (failed, writes_refused) = sequential_write_with_the_second_record_refused_on_disk_one(
        &mut pool,
        &first,
        &second_content,
    );
    assert_eq!(writes_refused, 1, "盘 1 上 jsn 5 那一槽的写被拒了一次");
    assert!(
        matches!(failed, Err(PublishError::BlockDevice(_))),
        "B 落盘中途报块设备错：{failed:?}"
    );
    let frozen = pool
        .allocator
        .frozen_publish()
        .expect("落盘中途失败的 B 冻结在分配器上")
        .clone();
    assert_eq!(
        (
            frozen.checkpoint_txg(),
            frozen.instance(),
            frozen
                .writes()
                .records
                .iter()
                .map(|record| record.counter)
                .collect::<Vec<u64>>()
        ),
        (CheckpointTxg(4), InstanceGeneration(1), vec![4, 5]),
        "冻结的是 B：txg 4、实例 1、两条记录 jsn 4 与 5"
    );
    let failed_attempt: Vec<_> =
        pool.retained_operations()[operations_before_the_failed_attempt..].to_vec();
    assert!(
        !failed_attempt.is_empty(),
        "失败之前 B 已经有写落到盘上（盘 0 的 jsn 5 在内）"
    );

    // 1. 冻结着的时候在 A 上再发 C：第一道就拒，录制流一步都没多。
    let operations_before_the_refused_publish = pool.retained_operations().len();
    let third_content = content_needing(1, 2);
    let refused = overwrite(&mut pool, &first, &third_content);
    assert!(
        matches!(
            refused,
            Err(
                PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: CheckpointTxg(4),
                    instance: InstanceGeneration(1),
                }
            )
        ),
        "B 没重发之前不许建下一次发布：{refused:?}"
    );
    assert_eq!(
        pool.retained_operations().len(),
        operations_before_the_refused_publish,
        "被拒的那一次一个写、一道屏障都没发"
    );

    // 2. 原样重发。
    let operations_before_the_resend = pool.retained_operations().len();
    let resent = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut pool.allocator).expect("重发这一遍不再报错")
    };
    let second = match resent {
        Some(PoolVersion::WithFile(second)) => second,
        other => panic!("重发交回带文件的 B：{other:?}"),
    };
    let resend: Vec<_> = pool.retained_operations()[operations_before_the_resend..].to_vec();
    assert!(
        resend.len() > failed_attempt.len(),
        "重发这一遍走完了整次发布（{} 步），比失败那一遍（{} 步）多",
        resend.len(),
        failed_attempt.len()
    );
    assert!(
        resend[..failed_attempt.len()] == failed_attempt[..],
        "失败那一遍落到盘上的每一步（设备、种类、偏移、字节）都是重发这一遍的前缀、逐字节相同"
    );
    assert_eq!(
        (
            second.root.checkpoint_txg,
            second
                .earlier_records_of_this_publish
                .iter()
                .map(|written| written.record.counter)
                .chain([second.record.counter])
                .collect::<Vec<u64>>()
        ),
        (CheckpointTxg(4), vec![4, 5]),
        "重发交回的 B：txg 4、jsn 4 与 5"
    );
    assert!(
        pool.allocator.frozen_publish().is_none(),
        "重发成功之后分配器上不再冻结着什么"
    );

    // 3. 之后的发布照常：C 接在重发的 B 上。
    let third = overwrite(&mut pool, &second, &third_content).expect("重发之后 C 照常发布");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));

    // 4. 冷启动恢复读回 C；池级 checker 的 I-8.9 成立（同一 (实例, txg) 只有一条末条）。
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { root, content } = &report.outcome else {
        panic!("冷启动要读回 C：{:?}", report.outcome);
    };
    assert_eq!(*root, (InstanceGeneration(1), CheckpointTxg(5)));
    assert!(content == &third_content, "读回的是 C 的内容");
    let verdicts = check_pool_image(&image);
    let publish_ordinals = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-8.9")
        .map(|(_, verdict)| verdict.clone());
    assert_eq!(
        publish_ordinals,
        Some(InvariantVerdict::Holds),
        "同一 (实例, txg) 只有一条末条标志"
    );
    // 重发成功之后分配器换成 B 成立之后的那一份：C 的落点不落在 B 的单元上，B 的根（仍在候选集里）指着的单元照旧对得上。
    let violated: Vec<&'static str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert!(violated.is_empty(), "池级 checker 一条都不红：{violated:?}");
}
