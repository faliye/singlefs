//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 5 行：D28（挂载期承诺量） 已定项 1 的准入读数接上真实的挂载态。
//!
//! 步 4 验收「准入读数：可用比回退前少了正好被隔离的那几块」与 C318（影子账隔离的单元没进准入不等式） 的判别力自证：
//! 固定脚本到 C 之后，崩溃恢复抛弃 C（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：
//! C 的根槽与数据单元暂时读不出，恢复落到 (2, 7)，实例 3 写行与暖机，再把 C 写回），同一段历史建两个池，
//! 一个影子账开着重开可写挂载、一个关着（只供测试的开关 `ShadowLedger`）。管理员回退改成挂着时的向前发布之后不抛弃任何根，
//! 被抛弃的根只由崩溃恢复造出（D23（journal 的角色与格式） 已定项 14）。
//! 两边的准入读数除「被抛弃根独占量」之外逐项相等——重开那次写行与暖机按同样的形状分配与释放，只是落点不同——
//! 可用正好差被隔离的那几块；把第九项置 0，开着那一边读出来的可用与关着那一边逐盘相等；需求取「不算隔离时正好够」的量，
//! 开着那一边判拒、每块盘恰好短那几块，置 0 之后放行（C318 欠账那一栏逐字「只差那几块的池要从只读翻成可写」）。
//!
//! 挂载期承诺量暖机那一半的 c_max 与 checkpoint 保留池的 ckpt_cost 条款没给（C363（现算保留池时树高从哪读没有条款），
//! 见 `singlefs_core::admission` 的模块文档）：用例各取两档，断言的差与它们取多少无关。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
    parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::admission::{
    admission_reading_of_a_writable_mount, admit_on_every_device, checkpoint_reserve_pool,
    instance_switch_reserve_on_one_device, space_budget_of_role, AdmissionReading,
    AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice, DemandOnDevice,
    DeviceAdmissionTerms, DeviceShortOfDemand, MetadataBlocks, PoolWideCommitments, ReplicaCount,
    SpaceBudgetOfARole,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
    InstanceTableRecords, MountSpaceAdmission, Mounted, ShadowLedger,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, StepOutcome,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
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

/// 被抛弃的那一版（C）、崩溃恢复抛弃它之后重开那次可写挂载的结果，与镜像（活到用例结束）。
struct RemountedAfterTheRecovery {
    third: TransactionOutput,
    mounted: Mounted,
    _pool: BuiltPool,
}

/// C 之后崩溃恢复抛弃 C（实例 3），再进程退出、重开可写挂载（实例 4），影子账按 `shadow_ledger`。
fn remounted_after_a_recovery_abandoned_the_third_version(
    tag: &str,
    shadow_ledger: ShadowLedger,
) -> RemountedAfterTheRecovery {
    let mut pool = build_through_third_publish(tag);
    let third = pool.output.clone();
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_test_only_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        shadow_ledger,
    )
    .expect("崩溃恢复抛弃 C 之后重开可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(4));
    pool.devices = Some(devices);
    RemountedAfterTheRecovery {
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

/// 重开之后的准入读数：逐盘各项从挂载交回的分配器取；挂载期承诺量按重开写行之后那一版实例表的行数（rows0）现算。
fn admission_reading(
    remounted: &RemountedAfterTheRecovery,
    supplied: QuantitiesSuppliedByThisTest,
) -> AdmissionReading {
    let row_publish = remounted
        .mounted
        .output
        .row_publish
        .file_version()
        .expect("落到带文件的一版上：写行那次带文件");
    let rows_after_the_row_publish =
        InstanceTableRecords::parse(&row_publish.unit(TransactionUnit::InstanceTable).bytes)
            .expect("写行那次重写的实例表解得开")
            .rows
            .len();
    let replicas = ReplicaCount::of_every_device_in_the_pool(&remounted.mounted.allocator);
    AdmissionReading::of_allocator(
        &remounted.mounted.allocator,
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
                mount_time_commitment: terms.mount_time_commitment,
                abandoned_root_exclusive: BytesOnOneDevice::ZERO,
            })
            .collect(),
        reading.pool_wide(),
        reading.replicas(),
    )
}

fn available_bytes_as_demand(available: AvailableBytesOnOneDevice) -> BytesOnOneDevice {
    BytesOnOneDevice(u64::try_from(available.0).expect("4 GiB 的盘上重开之后可用是正的"))
}

/// 同一段历史崩溃恢复抛弃 C 之后重开，影子账开着与关着各走一遍：第九项正好是被隔离的那几块；第九项置 0 之后可用正好多出那几块，
/// 「不算隔离时正好够」的需求在开着那一边每块盘都恰好短那几块、置 0 之后翻成放行。分配记录树按位置寻址之后
/// （D8（核心索引结构） 已定项 14）两边写的分配记录树节点数随落点不同，「已分配」差的正是这几个节点，
/// 两边读数之差是被隔离的那几块加这几个节点，不再只是被隔离的那几块。
#[test]
fn after_a_recovery_abandoned_a_root_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device(
) {
    let with_shadow_ledger = remounted_after_a_recovery_abandoned_the_third_version(
        "admission-formula-shadow-ledger-on",
        ShadowLedger::On,
    );
    let without_shadow_ledger = remounted_after_a_recovery_abandoned_the_third_version(
        "admission-formula-shadow-ledger-off",
        ShadowLedger::Off,
    );
    let row_publish = with_shadow_ledger
        .mounted
        .output
        .row_publish
        .file_version()
        .expect("落到带文件的一版上：写行那次带文件");

    // 独立数出来的被抛弃根独占槽：C 的账里占着、重开写行那一版的账里不占的（C 那一次发布写的 14 个，步 4 那几条影子账用例同一个数）。
    let abandoned_slots_per_device: Vec<(DeviceIdentity, u64)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|device| {
                let abandoned = slots_of(&with_shadow_ledger.third, device)
                    .difference(&slots_of(row_publish, device))
                    .count();
                (device, u64::try_from(abandoned).expect("槽数装得进 u64"))
            })
            .collect();
    assert_eq!(
        abandoned_slots_per_device,
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "C 的账里有、重开写行那一版的账里没有的槽，逐盘"
    );

    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）之后两边写的节点数可以不一样：影子账开着时写行与暖机的落点被推到隔离的那几块之后，
    // 关着时落点复用被抛弃的槽。两边的「已分配」差的正是这几个节点，从两边这一次挂载写出的单元现数（每个角色的跨度加起来），不从读数反推。
    let slots_written_by_the_remount = |remounted: &RemountedAfterTheRecovery| -> u64 {
        std::iter::once(&remounted.mounted.output.row_publish)
            .chain(&remounted.mounted.output.warm_up_publishes)
            .map(|version| {
                version
                    .file_version()
                    .expect("落到带文件的一版上：写行与暖机都带文件")
                    .rewritten
                    .iter()
                    .map(|role| role.span_slots())
                    .sum::<u64>()
            })
            .sum()
    };
    let extra_slots_written_with_the_shadow_ledger =
        slots_written_by_the_remount(&with_shadow_ledger)
            - slots_written_by_the_remount(&without_shadow_ledger);
    assert_eq!(
        extra_slots_written_with_the_shadow_ledger, 0,
        "两边写的节点数（实三草稿里量的）"
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
                    terms_without.mount_time_commitment
                ),
                "{supplied:?} 盘 {device:?}：第九项之外逐项相等，「已分配」只差两边多写的节点（写行与暖机按同样的角色分配、释放，\
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
                "{supplied:?} 盘 {device:?}：影子账开着的可用少被隔离的那几块，加多写的节点"
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
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 可写挂载那一处（C363 (b) 判决第四节第 2 条接进来；D16（发布语义） 已定项 1「准入」那一行定了不够时怎么办，用户 2026-09-26 定）：
/// 两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 12 次的盘面上再可写挂载——取号之前按这次挂载的 rows0 算
/// （D28（挂载期承诺量） 已定项 3），扣掉实例切换预留、checkpoint 保留池（已定项 4，按最坏情况计，用户 2026-09-27 定）与已分配之后，
/// 两块盘的可用(d) 都是 −39 槽 < 0（A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是 −47 槽）：
/// 实例切换的预留拿不到。写行那次发布照走切换预留、写完行与暖机之后推抬 F 的空发布、再判：这里推一串（F 抬到上限 14，3 次空发布），
/// 再判就够了，挂载做成（取号 3），交回 `MountSpaceAdmission::AdmittedAfterTheFloorRaises`，带着取号之前那一判的拒绝原样。
/// 推的那一串的根都带新 F = 14、接在暖机后面，现行那一版就是最后一次；挂载之后的镜像池级 checker 0 违例；按可写挂载的读数再判，每块盘可用 ≥ 0。
/// 判别力：同一份盘面上只供测试的开关关掉准入，同一次挂载一次都不推（`NotJudgedByTheTestOnlySwitch`）——推是式子判出来的，不是落点逼的。
#[test]
fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let image = small_pool_image_after_overwrites(device_width, 12);

    let stream = SharedStream::new();
    let mut devices = devices_on_the_image(&image, &stream);
    let mounted = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("取号之前不够的挂载写行之后推抬 F、再判够了，挂载做成");
    assert_eq!(mounted.output.instance, InstanceGeneration(3));
    let MountSpaceAdmission::AdmittedAfterTheFloorRaises {
        refusal_before_acquisition,
        floor_raises,
    } = &mounted.output.space_admission
    else {
        panic!(
            "取号之前不够、写行之后推了再判够了：{:?}",
            mounted.output.space_admission
        );
    };
    assert_eq!(
        refusal_before_acquisition,
        &AdmissionRefusedOnSomeDevices {
            short_devices: [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|device| DeviceShortOfDemand {
                    device,
                    available: AvailableBytesOnOneDevice(-39 * i128::from(SLOT_BYTES)),
                    demand: BytesOnOneDevice::ZERO,
                })
                .collect(),
        },
        "取号之前两块盘都短：可用 −39 槽，挂载这一刻不另要需求"
    );
    assert_eq!(floor_raises.len(), 1, "推一串就够");
    let raised = &floor_raises[0];
    assert_eq!(raised.ceiling, CheckpointTxg(14), "F 抬到这一刻的上限");
    let last_warm_up_txg = mounted
        .output
        .warm_up_publishes
        .last()
        .expect("写行之后暖机到本实例的根覆盖每块盘")
        .root()
        .checkpoint_txg;
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg, publish.root.rollback_floor))
            .collect::<Vec<_>>(),
        (1..=3)
            .map(|offset| (
                CheckpointTxg(last_warm_up_txg.0 + offset),
                CheckpointTxg(14)
            ))
            .collect::<Vec<_>>(),
        "三次空发布接在暖机后面、根都带新 F"
    );
    assert_eq!(
        mounted.current.root().checkpoint_txg,
        CheckpointTxg(last_warm_up_txg.0 + 3),
        "现行那一版是推的最后一次"
    );
    for (device, available) in
        admission_reading_of_a_writable_mount(&mounted.allocator, mounted.current.file_version())
            .available_on_each_device()
    {
        assert!(
            available.0 >= 0,
            "盘 {device:?}：推过之后按可写挂载的读数再判够了：{available:?}"
        );
    }
    let after = image_of_the_devices(&devices, image.device_size_in_bytes);
    let violations: Vec<String> = check_pool_image(&after)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert!(
        violations.is_empty(),
        "挂载之后的镜像池级 checker 0 违例：{violations:?}"
    );

    let not_judged = mount_writable_with_space_admission(
        &device_width.parameters(),
        &mut devices_on_the_image(&image, &SharedStream::new()),
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("同一份盘面上关掉准入，同一次挂载做成");
    assert!(
        matches!(
            not_judged.output.space_admission,
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
        ),
        "关掉准入一次都不推：{:?}",
        not_judged.output.space_admission
    );
}

/// 发布路径那一处（C363 (b) 判决第四节第 2 条）：两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 4 次的盘面上，进程重开、
/// 可写挂载（取号之前就够），在这次挂载里直接调发布路径接连覆盖写三次（都放行），第四次：需求 = 这次发布新写的全部槽（D28（挂载期承诺量） 已定项 1 接线，
/// 普通分配加固定点、不加换下的，用户 2026-09-25 定），每块盘 14 槽（普通分配 6、固定点 8），而两块盘的可用(d) 都只剩 11 槽
/// （实例切换的预留按下一次挂载的 rows0，已定项 3；checkpoint 保留池与 c_max 按最坏情况计，已定项 4，用户 2026-09-27 定）——
/// 在读盘核与动分配器之前返回 `PublishError::SpaceAdmissionRefused`：录制流一步没多、两块盘逐字节不变、`DiskSnapshot` 不变、
/// 分配器与发布之前逐项相同。这一格只有把固定点算进需求才拒：普通分配只有 6 槽 ≤ 11。需求那 14 槽从同一次覆盖写真发出去的角色现数
/// （关掉准入的那一次），不从读数反推。直接调发布路径不推抬 F（那是挂着的会话的事，`mounted_session`）。
/// 覆盖写 9 次那一份盘面（按树高、按每块盘一条叶路径计时这一格用它）在按最坏情况计之下挂载那一刻就要推抬 F、推完可用 44 槽，
/// 分不出「普通分配装得下、固定点装不下」，换成 5 次；A3b 把小盘改成 6 MiB 环、单元区从 1408 起之后（分配记录树按绝对槽号按位置寻址，
/// 每次发布重写的节点跟着变），240 槽 5 次那一格第一次直接覆盖写就放行了。草稿探针在三档、覆盖写 1–12 次、挂载之后接连直接覆盖写
/// 1–4 次上扫：可用落在 [普通分配 6, 需求) 之间的只有 256 槽、覆盖写 4 次、第 4 次直接覆盖写那一格，取它。
/// 判别力：同一次挂载里把只供测试的开关装成关掉准入，同一次覆盖写做成——拒的是式子，不是落点。
#[test]
fn an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write(
) {
    let device_width = HistoryDeviceWidth::UnitAreaOf256Slots;
    let image = small_pool_image_after_overwrites(device_width, 4);
    let content = content_of(DIRECT_OVERWRITE_CONTENT_BYTES, 4);
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
    assert!(
        matches!(
            mount_output.space_admission,
            MountSpaceAdmission::AdmittedBeforeAcquisition
        ),
        "取号之前就够，一次都没推：{:?}",
        mount_output.space_admission
    );
    let mut current = current
        .into_file_version()
        .expect("可写挂载之后现行那一版带文件");
    for seed in 1..=3 {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of(DIRECT_OVERWRITE_CONTENT_BYTES, seed),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + 600
                    + u64::try_from(seed).expect("小"),
            },
            mount_output.instance,
        )
        .unwrap_or_else(|refusal| panic!("挂载之后第 {seed} 次直接覆盖写放行：{refusal:?}"));
    }
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
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 604,
            },
            mount_output.instance,
        )
    };
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = refused else {
        panic!("式子判拒：{:?}", refused.as_ref().err());
    };
    let after = image_of_the_devices(&devices, image.device_size_in_bytes);
    assert!(after == image_after_the_mount, "两块盘逐字节不变");
    assert_eq!(disk_snapshot(&after, &stream), before, "DiskSnapshot 不变");
    assert!(
        format!("{allocator:?}") == allocator_before_the_publish,
        "分配器与发布之前逐项相同"
    );

    allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let published = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 604,
            },
            mount_output.instance,
        )
        .expect("同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点")
    };
    let slots_written_by_the_overwrite: u64 = published
        .rewritten
        .iter()
        .map(|role| role.span_slots())
        .sum();
    let ordinary_slots_written_by_the_overwrite: u64 = published
        .rewritten
        .iter()
        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .map(|role| role.span_slots())
        .sum();
    assert_eq!(
        (slots_written_by_the_overwrite, ordinary_slots_written_by_the_overwrite),
        (14, 6),
        "一次覆盖写新写 14 槽，其中普通分配 6 槽（数据单元 2、extent 根 1、inode 叶容器 2、inode 根 1），固定点 8 槽"
    );
    assert_eq!(
        refusal,
        AdmissionRefusedOnSomeDevices {
            short_devices: [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|device| DeviceShortOfDemand {
                    device,
                    available: AvailableBytesOnOneDevice(11 * i128::from(SLOT_BYTES)),
                    demand: BytesOnOneDevice::of_slots(slots_written_by_the_overwrite),
                })
                .collect(),
        },
        "两块盘都短：可用 11 槽 < 需求 14 槽（普通分配的 6 槽装得下，固定点那 8 槽装不下）"
    );
}
