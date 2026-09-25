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

use common::{build_pool, disk_snapshot, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::admission::{
    admit_on_every_device, checkpoint_reserve_pool, instance_switch_reserve_on_one_device,
    AdmissionReading, AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice,
    DemandOnDevice, DeviceAdmissionTerms, DeviceShortOfDemand, MetadataBlocks, PoolWideCommitments,
    ReplicaCount,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_rollback, mount_writable, mount_writable_with_space_admission, InstanceTableRecords,
    MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepOutcome,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
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

/// 同一段历史回退到 A，影子账开着与关着各走一遍：第九项正好是被隔离的那几块；第九项置 0 之后可用正好多出那几块，
/// 「不算隔离时正好够」的需求在开着那一边每块盘都恰好短那几块、置 0 之后翻成放行。分配记录树按位置寻址之后
/// （D8（核心索引结构） 已定项 14）两边写的分配记录树节点数随落点不同（开着那边多两片叶 62），「已分配」差的正是这几个节点，
/// 两边读数之差是被隔离的那几块加这几个节点，不再只是被隔离的那几块。
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

    // 独立数出来的被抛弃根独占槽：C 的账里占着、D 的账里不占的（B 14、写行 10、暖机 8 + 8、C 14 = 54，步 4 验收第一条同一个数；
    // 分配记录树按位置寻址之后每版的固定点是三个角色加分配记录树五个节点 8 槽，带文件的一版再加文件四个单元 6 槽，D8（核心索引结构） 已定项 14）。
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
        vec![(DeviceIdentity(0), 54), (DeviceIdentity(1), 54)],
        "C 的账里有、D 的账里没有的槽，逐盘"
    );

    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）之后两边写的节点数不一样：影子账开着时 D 与暖机的落点被推到隔离的那几块之后
    // （越过 50344，叶 62），分配记录树两块盘各多一片叶 62；关着时落点复用被抛弃的槽、都在叶 61 里。两边的「已分配」差的正是这几个节点，
    // 从两边这一次挂载写出的单元现数（每个角色的跨度加起来），不从读数反推。
    let slots_written_by_the_rollback_mount = |rolled_back: &RolledBack| -> u64 {
        std::iter::once(&rolled_back.mounted.output.row_publish)
            .chain(&rolled_back.mounted.output.warm_up_publishes)
            .map(|version| {
                version
                    .file_version()
                    .expect("回退到带文件的 A：D 与暖机都带文件")
                    .rewritten
                    .iter()
                    .map(|role| role.span_slots())
                    .sum::<u64>()
            })
            .sum()
    };
    let extra_slots_written_with_the_shadow_ledger =
        slots_written_by_the_rollback_mount(&with_shadow_ledger)
            - slots_written_by_the_rollback_mount(&without_shadow_ledger);
    assert_eq!(
        extra_slots_written_with_the_shadow_ledger, 2,
        "影子账开着那一边多写两块盘各一片叶 62"
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
                    BytesOnOneDevice(
                        terms_without.allocated.0
                            + BytesOnOneDevice::of_slots(extra_slots_written_with_the_shadow_ledger)
                                .0
                    ),
                    terms_without.unreclaimable,
                    terms_without.deferred,
                    terms_without.mount_time_commitment
                ),
                "{supplied:?} 盘 {device:?}：第九项之外逐项相等，只有「已分配」多那两片叶 62（D 与暖机按同样的角色分配、释放，\
                 落点不同，分配记录树按位置寻址时落点决定写几片叶）"
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
                i128::from(*abandoned_slots + extra_slots_written_with_the_shadow_ledger)
                    * i128::from(SLOT_BYTES),
                "{supplied:?} 盘 {device:?}：影子账开着的可用少被隔离的那几块，加多写的那两片叶"
            );
        }

        let zeroed = reading_with_the_abandoned_root_exclusive_term_zeroed(&reading_with);
        for (((device, with), (_, zeroed_available)), (_, abandoned_slots)) in available_with
            .iter()
            .zip(&zeroed.available_on_each_device())
            .zip(&abandoned_slots_per_device)
        {
            assert_eq!(
                zeroed_available.0 - with.0,
                i128::from(*abandoned_slots) * i128::from(SLOT_BYTES),
                "{supplied:?} 盘 {device:?}：第九项置 0，影子账开着的读数多报正好被隔离的那几块"
            );
        }

        // 需求取「不算隔离时正好够」的量（第九项置 0 那一份读数的可用）：影子账开着时只差被隔离的那几块。
        let demand_that_fits_only_without_the_isolation: Vec<DemandOnDevice> = zeroed
            .available_on_each_device()
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

/// 造小盘盘面的那几次覆盖写（随机历史的覆盖写操作）：长度选择子与填充种子都固定，同一段历史逐字节复现；选择子 2999 ⇒ 3000 字节。
const SMALL_POOL_OVERWRITE_LENGTH_SELECTOR: u64 = 2999;
const SMALL_POOL_OVERWRITE_FILL_SEED: u64 = 1;

/// 盘面造好之后用例自己发的那一次覆盖写的内容长度：与造盘面那几次一样长，一个数据单元装得下。
const DIRECT_OVERWRITE_CONTENT_BYTES: usize = 3000;

/// 一块小盘上的盘面：mkfs 同一个进程里发完第一个文件，再可写挂载一次、覆盖写 `overwrites` 次（内容见上面两个常量），
/// 空间准入关掉（只供测试的开关）——这几次都照写，造出的是一份合法的、占得很满的盘面；准入判不判，由下面的用例在这份盘面上各自判。
/// 交回最后一步之后的整份镜像与这段历史的盘宽。
fn small_pool_image_after_overwrites(
    device_width: HistoryDeviceWidth,
    overwrites: usize,
) -> MemoryPool {
    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
        length: ContentLength::InsideOneDataUnit {
            selector: SMALL_POOL_OVERWRITE_LENGTH_SELECTOR,
        },
        fill_seed: SMALL_POOL_OVERWRITE_FILL_SEED,
    });
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: std::iter::once(HistoryOperation::CloseAndMountWritable)
            .chain(std::iter::repeat_n(overwrite, overwrites))
            .collect(),
    };
    let mut last_image = None;
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Run,
            device_width,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        &SharedStream::new(),
        &mut |observation| last_image = Some(observation.image.clone()),
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
    assert!(
        run.outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Applied(_))),
        "挂载与 {overwrites} 次覆盖写都做成（每一步之后池级 checker 判绿）：{:?}",
        run.outcomes
    );
    last_image.expect("每一步之后都有一份镜像")
}

/// 一份镜像交给两块录着的内存盘（录进 `stream`）。
fn devices_on_the_image(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 两块内存盘此刻的整份镜像。
fn image_of_the_devices(
    devices: &[(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner().image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// C363 (b) 判决第四节第 2 条的验收（可写挂载那一处）：两块单元区 384 槽的小盘，第一个文件之后可写挂载、覆盖写 10 次的盘面上，
/// 再可写挂载——D28（挂载期承诺量） 已定项 1 的式子扣掉这次挂载的实例切换预留（已定项 3，rows0 按实例表现算）、checkpoint 保留池
/// （已定项 4）与已分配、defer 待释放之后，两块盘的可用(d) 都是 −3 槽（−49152 字节）< 0：实例切换的预留拿不到（D2（RAID 条带策略）
/// 已定项 13），在**取号之前**返回 `SpaceAdmissionRefusedBeforeAcquisition`，录制流一步没多、两块盘逐字节不变、`DiskSnapshot` 不变
/// （两块盘系统配置里的实例代号还是 2）。判别力：同一份盘面上只供测试的开关关掉准入，同一次挂载做成（取号 3、写行、暖机）——
/// 拒的是式子，不是落点。覆盖写 8 次的盘面上式子放行（草稿副本上扫过 0–30 次，10 次是第一个被拒的点）。
#[test]
fn a_writable_mount_whose_instance_switch_reserve_does_not_fit_is_refused_by_the_space_admission_before_acquisition_with_the_disk_unchanged(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let image = small_pool_image_after_overwrites(device_width, 10);

    let stream = SharedStream::new();
    let before = disk_snapshot(&image, &stream);
    let mut devices = devices_on_the_image(&image, &stream);
    let refused = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
        instance_to_acquire,
        refusal,
    }) = refused
    else {
        panic!("式子判拒、在取号之前返回：{:?}", refused.as_ref().err());
    };
    assert_eq!(instance_to_acquire, InstanceGeneration(3));
    assert_eq!(
        refusal,
        AdmissionRefusedOnSomeDevices {
            short_devices: [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|device| DeviceShortOfDemand {
                    device,
                    available: AvailableBytesOnOneDevice(-3 * i128::from(SLOT_BYTES)),
                    demand: BytesOnOneDevice::ZERO,
                })
                .collect(),
        },
        "两块盘都短：可用 −3 槽，挂载这一刻不另要需求"
    );
    let after = image_of_the_devices(&devices, image.device_size_in_bytes);
    assert!(stream.operations().is_empty(), "一个写、一道屏障都没发");
    assert!(after == image, "两块盘逐字节不变");
    assert_eq!(disk_snapshot(&after, &stream), before, "DiskSnapshot 不变");

    let mounted = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices_on_the_image(&image, &SharedStream::new()),
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("同一份盘面上关掉准入，同一次挂载做成：拒的是式子，不是落点");
    assert_eq!(mounted.output.instance, InstanceGeneration(3));
}

/// C363 (b) 判决第四节第 2 条的验收（发布路径那一处）：两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 4 次的盘面上，
/// 进程重开、可写挂载（式子放行：挂载这一刻不另要需求），在这次挂载里再覆盖写一次：这次的普通分配是数据单元 2 槽、extent 树根 1 槽、
/// inode 树叶容器 2 槽、inode 树根 1 槽，每块盘 6 槽（98304 字节；固定点与实例表链各有自己那一项保留，不算需求，
/// `admission::space_budget_of_role`），而两块盘的可用(d) 都只剩 5 槽（81920 字节）——少一槽，在读盘核与动分配器之前返回
/// `PublishError::SpaceAdmissionRefused`：录制流一步没多、两块盘逐字节不变、`DiskSnapshot` 不变、分配器与发布之前逐项相同。
/// 判别力：同一次挂载里把只供测试的开关装成关掉准入，同一次覆盖写做成——拒的是式子，不是落点。
#[test]
fn an_overwrite_whose_ordinary_allocations_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let image = small_pool_image_after_overwrites(device_width, 4);
    let content = content_of(DIRECT_OVERWRITE_CONTENT_BYTES, 1);
    let publish_parameters = device_width.parameters();

    let stream = SharedStream::new();
    let mut devices = devices_on_the_image(&image, &stream);
    let Mounted {
        output: mount_output,
        mut allocator,
        current,
    } = mount_writable_with_space_admission(
        &publish_parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("挂载这一刻式子放行");
    let current = current
        .into_file_version()
        .expect("可写挂载之后现行那一版带文件");
    let image_after_the_mount = image_of_the_devices(&devices, image.device_size_in_bytes);
    let before = disk_snapshot(&image_after_the_mount, &stream);
    let allocator_before_the_publish = format!("{allocator:?}");
    let refused = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
            },
            mount_output.instance,
        )
    };
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = refused else {
        panic!("式子判拒：{:?}", refused.as_ref().err());
    };
    let ordinary_slots_of_one_overwrite =
        TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile::FIRST).span_slots()
            + TransactionUnit::ExtentRoot.span_slots()
            + TransactionUnit::InodeLeafContainer(
                singlefs_core::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST,
            )
            .span_slots()
            + TransactionUnit::InodeRoot.span_slots();
    assert_eq!(ordinary_slots_of_one_overwrite, 6);
    assert_eq!(
        refusal,
        AdmissionRefusedOnSomeDevices {
            short_devices: [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|device| DeviceShortOfDemand {
                    device,
                    available: AvailableBytesOnOneDevice(5 * i128::from(SLOT_BYTES)),
                    demand: BytesOnOneDevice::of_slots(ordinary_slots_of_one_overwrite),
                })
                .collect(),
        },
        "两块盘都短一槽：可用 5 槽 < 需求 6 槽"
    );
    let after = image_of_the_devices(&devices, image.device_size_in_bytes);
    assert!(after == image_after_the_mount, "两块盘逐字节不变");
    assert_eq!(disk_snapshot(&after, &stream), before, "DiskSnapshot 不变");
    assert!(
        format!("{allocator:?}") == allocator_before_the_publish,
        "分配器与发布之前逐项相同"
    );

    allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut allocator,
        &current,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
        },
        mount_output.instance,
    )
    .expect("同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点");
}
