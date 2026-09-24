//! 里程碑「第二个事务」增补 2 收口表第 5 行：D28（挂载期承诺量） 已定项 1 的准入读数接上真实的挂载态。
//!
//! 步 4 验收「准入读数：可用比回退前少了正好被隔离的那几块」与 C318（影子账隔离的单元没进准入不等式） 的判别力自证：
//! 固定脚本到 C 之后，同一段历史建两个池，一个影子账开着回退到 A、一个关着回退（只供测试的开关 `ShadowLedger`）。
//! 两边的准入读数除「被抛弃根独占量」之外逐项相等——写行那次发布 D 与暖机按同样的形状分配与释放，只是落点不同——
//! 可用正好差被隔离的那几块；把第九项置 0，开着那一边读出来的可用与关着那一边逐盘相等；需求取「不算隔离时正好够」的量，
//! 开着那一边判拒、每块盘恰好短那几块，置 0 之后放行（C318 欠账那一栏逐字「只差那几块的池要从只读翻成可写」）。
//!
//! 挂载期承诺量暖机那一半的 c_max 与 checkpoint 保留池的 ckpt_cost 条款没给（C363（现算保留池时树高从哪读没有条款），
//! 见 `singlefs_core::admission` 的模块文档）：用例各取两档，断言的差与它们取多少无关。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::{
    admit_on_every_device, checkpoint_reserve_pool, instance_switch_reserve_on_one_device,
    AdmissionReading, AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice,
    DemandOnDevice, DeviceAdmissionTerms, DeviceShortOfDemand, MetadataBlocks, PoolWideCommitments,
    ReplicaCount,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceTableRecords, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::SLOT_BYTES;
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    pool.output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。与步 4 验收那一份同一段历史。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(
        &mut pool,
        &content_of(SECOND_FILE_BYTES, 3),
        InstanceGeneration(1),
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(
        &mut pool,
        &content_of(THIRD_FILE_BYTES, 11),
        InstanceGeneration(2),
    );
    pool
}

/// 回退之前的最后一版（C）、回退的结果，与镜像（活到用例结束）。
struct RolledBack {
    third: TransactionOutput,
    mounted: Mounted,
    _pool: BuiltPool,
}

/// C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)，影子账按 `shadow_ledger`。
fn rolled_back_to_the_first_root(tag: &str, shadow_ledger: ShadowLedger) -> RolledBack {
    let mut pool = build_through_third_publish(tag);
    let third = pool.output.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        shadow_ledger,
    )
    .expect("回退");
    pool.devices = Some(devices);
    RolledBack {
        third,
        mounted,
        _pool: pool,
    }
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 条款没给、由用例给的两个量（C363）。
#[derive(Clone, Copy, Debug)]
struct QuantitiesSuppliedByThisTest {
    worst_empty_publish_cost: MetadataBlocks,
    checkpoint_cost: MetadataBlocks,
}

/// 回退之后的准入读数：逐盘各项从回退交回的分配器取；挂载期承诺量按回退写行之后那一版实例表的行数（rows0）现算。
fn admission_reading(
    rolled_back: &RolledBack,
    supplied: QuantitiesSuppliedByThisTest,
) -> AdmissionReading {
    let rollback_publish = rolled_back
        .mounted
        .output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");
    let rows_after_the_row_publish =
        InstanceTableRecords::parse(&rollback_publish.unit(TransactionUnit::InstanceTable).bytes)
            .expect("D 重写的实例表解得开")
            .rows
            .len();
    let replicas = ReplicaCount::of_every_device_in_the_pool(&rolled_back.mounted.allocator);
    AdmissionReading::of_allocator(
        &rolled_back.mounted.allocator,
        instance_switch_reserve_on_one_device(
            u64::try_from(rows_after_the_row_publish).expect("行数装得进 u64"),
            supplied.worst_empty_publish_cost,
        ),
        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
            supplied.checkpoint_cost,
            replicas,
        )),
    )
}

/// 同一份读数，只把第九项「被抛弃根独占量」逐盘置 0（C318 判别力自证要的那一刀）。
fn reading_with_the_abandoned_root_exclusive_term_zeroed(
    reading: &AdmissionReading,
) -> AdmissionReading {
    AdmissionReading::new(
        reading
            .per_device()
            .iter()
            .map(|terms| DeviceAdmissionTerms {
                device: terms.device,
                capacity: terms.capacity,
                allocated: terms.allocated,
                unreclaimable: terms.unreclaimable,
                deferred: terms.deferred,
                mount_time_commitment: terms.mount_time_commitment,
                abandoned_root_exclusive: BytesOnOneDevice::ZERO,
            })
            .collect(),
        reading.pool_wide(),
        reading.replicas(),
    )
}

fn available_bytes_as_demand(available: AvailableBytesOnOneDevice) -> BytesOnOneDevice {
    BytesOnOneDevice(u64::try_from(available.0).expect("4 GiB 的盘上回退之后可用是正的"))
}

#[test]
fn after_the_rollback_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device(
) {
    let with_shadow_ledger =
        rolled_back_to_the_first_root("admission-formula-shadow-ledger-on", ShadowLedger::On);
    let without_shadow_ledger =
        rolled_back_to_the_first_root("admission-formula-shadow-ledger-off", ShadowLedger::Off);
    let rollback_publish = with_shadow_ledger
        .mounted
        .output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");

    // 独立数出来的被抛弃根独占槽：C 的账里占着、D 的账里不占的（B 10、写行 6、暖机 4 + 4、C 10 = 34，步 4 验收第一条同一个数）。
    let abandoned_slots_per_device: Vec<(DeviceIdentity, u64)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|device| {
                let abandoned = slots_of(&with_shadow_ledger.third, device)
                    .difference(&slots_of(rollback_publish, device))
                    .count();
                (device, u64::try_from(abandoned).expect("槽数装得进 u64"))
            })
            .collect();
    assert_eq!(
        abandoned_slots_per_device,
        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
        "C 的账里有、D 的账里没有的槽，逐盘"
    );

    for supplied in [
        QuantitiesSuppliedByThisTest {
            worst_empty_publish_cost: MetadataBlocks(0),
            checkpoint_cost: MetadataBlocks(0),
        },
        QuantitiesSuppliedByThisTest {
            worst_empty_publish_cost: MetadataBlocks(9),
            checkpoint_cost: MetadataBlocks(9),
        },
    ] {
        let reading_with = admission_reading(&with_shadow_ledger, supplied);
        let reading_without = admission_reading(&without_shadow_ledger, supplied);

        for ((terms_with, terms_without), (device, abandoned_slots)) in reading_with
            .per_device()
            .iter()
            .zip(reading_without.per_device())
            .zip(&abandoned_slots_per_device)
        {
            assert_eq!(
                (terms_with.device, terms_without.device),
                (*device, *device)
            );
            assert_eq!(
                terms_with.abandoned_root_exclusive,
                BytesOnOneDevice::of_slots(*abandoned_slots),
                "{supplied:?} 盘 {device:?}：影子账开着，第九项是只被被抛弃根引用的那几个槽"
            );
            assert_eq!(
                terms_without.abandoned_root_exclusive,
                BytesOnOneDevice::ZERO,
                "{supplied:?} 盘 {device:?}：影子账关着，第九项是 0"
            );
            assert_eq!(
                (
                    terms_with.capacity,
                    terms_with.allocated,
                    terms_with.unreclaimable,
                    terms_with.deferred,
                    terms_with.mount_time_commitment
                ),
                (
                    terms_without.capacity,
                    terms_without.allocated,
                    terms_without.unreclaimable,
                    terms_without.deferred,
                    terms_without.mount_time_commitment
                ),
                "{supplied:?} 盘 {device:?}：第九项之外逐项相等（D 与暖机按同样的形状分配、释放，只是落点不同）"
            );
        }

        let available_with = reading_with.available_on_each_device();
        let available_without = reading_without.available_on_each_device();
        for (((device, with), (_, without)), (_, abandoned_slots)) in available_with
            .iter()
            .zip(&available_without)
            .zip(&abandoned_slots_per_device)
        {
            assert_eq!(
                without.0 - with.0,
                i128::from(*abandoned_slots) * i128::from(SLOT_BYTES),
                "{supplied:?} 盘 {device:?}：影子账开着的可用正好少被隔离的那几块"
            );
        }

        let zeroed = reading_with_the_abandoned_root_exclusive_term_zeroed(&reading_with);
        assert_eq!(
            zeroed.available_on_each_device(),
            available_without,
            "{supplied:?}：第九项置 0，影子账开着的读数多报正好那几块，与关着那一边逐盘相等"
        );

        // 需求取「不算隔离时正好够」的量：只差被隔离的那几块。
        let demand_that_fits_only_without_the_isolation: Vec<DemandOnDevice> = available_without
            .iter()
            .map(|(device, available)| DemandOnDevice {
                device: *device,
                bytes: available_bytes_as_demand(*available),
            })
            .collect();
        assert_eq!(
            admit_on_every_device(&reading_with, &demand_that_fits_only_without_the_isolation),
            Err(AdmissionRefusedOnSomeDevices {
                short_devices: available_with
                    .iter()
                    .zip(&demand_that_fits_only_without_the_isolation)
                    .map(|((device, available), demand)| DeviceShortOfDemand {
                        device: *device,
                        available: *available,
                        demand: demand.bytes,
                    })
                    .collect(),
            }),
            "{supplied:?}：影子账开着，每块盘都恰好短被隔离的那几块，判拒"
        );
        assert_eq!(
            admit_on_every_device(&zeroed, &demand_that_fits_only_without_the_isolation),
            Ok(()),
            "{supplied:?}：第九项置 0，同一份需求翻成放行"
        );
    }
}
