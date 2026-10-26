//! checker 档模块：crash、crash_point_replay_progress、crash_given_states、crash_amplification
//! 回退改形态的第二批（SysPre 与 B1）落到 `crates/` 上：
//! - F 的生效值 = max(根上带的 F, 系统配置里读得出的 F)（D16（发布语义） 已定项 1「生效」）；
//! - 抬 F 那一串（准入与卸载共用）第一次发布之前先把新 F 写进每块盘的系统配置、过一道屏障，先写那一步任何一块盘失败就一条根都不发、
//!   已写进的新 F 留着（「抬 F 那一串」那一行，已定项 7）；
//! - 非抬 F 的系统配置写（取号、发布末尾的轮换、取号失败的回卷）带整池的 F 生效值；
//! - 正常卸载把 F 抬到现行那一版的 txg、不判上限，那一串空发布的根全带卸载记号（「正常卸载」那一行，D22（单元原子性怎么合成） 已定项 7）；
//! - 录制流核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查））；
//! - 崩溃点重放的记录核对器把系统配置里的 F 算进回收谓词的界（C556（checker 与崩溃点重放不读系统配置里的 F） 的崩溃点重放那一半）。
//!
//! 条款没写的一格按「停在那一处」做成错误成员、在任何写之前拒：往下「抬」F（新 F 低于盘上的 F 生效值）。
//! 树表 0 条的一版上卸载：一个字节都不写、卸载照常做成。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, disk_snapshot, file_content, format_pool, geometry, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker_tier::crash::{evaluate_state_for_versions, CrashPointReplayTally};
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{PoolAllocator, ReuseWindow};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, MountError,
    RollbackCandidateExclusion, RollbackError, RollbackTarget, ShadowLedger, UnmountRaisedTheFloor,
    Unmounted,
};
use singlefs_core::recovery::{
    effective_rollback_floor, readable_roots, verified_system_configuration_slots, PoolReader,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker, ROOT_CHECKSUM_OFFSET};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::write_accounting::WriteCallsAndBytes;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::{
    unmount_markers_outside_the_unmount_entry, writes_and_segments,
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, SparseBlockDevice,
    UnmountMarkerOutsideTheUnmountEntry, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{
    RecordedOperationKind, RecordedPublishEntry, RecordingBlockDevice, SharedStream,
};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
        .collect()
}

/// 新池新建文件（txg 3）之后在同一个进程里覆盖写 `times` 次（txg 4 起），现行版本跟着往前推。
fn overwrite_times(pool: &mut BuiltPool, times: usize) {
    for seed in 0..times {
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            pool,
            &previous,
            &content_of(2000 + 97 * seed, seed),
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
    }
}

/// 抬 F 到 `new_floor`（准入入口，影子账开着），现行版本换成这一串最后落盘的那一次。
fn raise_floor(
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
) -> Result<singlefs_core::mount::RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

/// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557）；现行版本换成这一串最后落盘的那一次。
/// 带文件的现行那一版上正常卸载：交回抬 F 那一串（带文件的一版上卸载必抬 F，另一个成员在这里是错）。
fn unmount_recorded(pool: &mut BuiltPool) -> Result<UnmountRaisedTheFloor, MountError> {
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream.record_entry(RecordedPublishEntry::Unmount, || {
        unmount(
            &parameters(),
            devices,
            allocator,
            &mut current,
            ShadowLedger::On,
        )
    });
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    unmounted.map(|unmounted| match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => {
            panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
        }
    })
}

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 一块盘两槽里自证过的系统配置带的 F，按槽序。
fn system_configuration_floors_on<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
) -> Vec<CheckpointTxg> {
    verified_system_configuration_slots(
        reader,
        device,
        u64::from(parameters().geometry.fixed_structure_slot_spacing),
        &parameters().filesystem_identifier,
    )
    .iter()
    .map(|system_configuration| system_configuration.quantities.rollback_floor)
    .collect()
}

fn effective_floor_of<Reader: PoolReader + ?Sized>(reader: &Reader) -> CheckpointTxg {
    effective_rollback_floor(
        reader,
        &parameters().region_devices,
        &parameters().geometry,
        &parameters().filesystem_identifier,
    )
}

fn readable_roots_of<Reader: PoolReader + ?Sized>(reader: &Reader) -> Vec<RootRecord> {
    readable_roots(
        reader,
        &parameters().region_devices,
        &parameters().geometry,
        &parameters().filesystem_identifier,
    )
}

/// 录制流第 `from` 步起每一步的种类与盘：切段用的分法（`FixedGeometry::classify`），屏障记成 `StepKind::Barrier`。
fn step_kinds_since(stream: &SharedStream, from: usize) -> Vec<(StepKind, DeviceIdentity)> {
    stream.operations()[from..]
        .iter()
        .map(|operation| (geometry().classify(operation), operation.device))
        .collect()
}

/// 把 `device` 那块盘上两个系统配置槽都改坏一个字节（整槽校验和罩整槽）：两槽都自证不过。
fn make_both_system_configuration_slots_unreadable(pool: &mut BuiltPool, device: DeviceIdentity) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, target) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("有这块盘");
    for offset in [0, spacing] {
        let mut bytes = vec![0u8; slot_bytes()];
        target
            .read_at(DeviceOffsetInBytes(offset), &mut bytes)
            .expect("读系统配置槽");
        bytes[4000] ^= 0xff;
        target
            .write_at(DeviceOffsetInBytes(offset), &bytes, WriteDurability::Plain)
            .expect("改坏系统配置槽");
    }
}

/// 把 checkpoint_txg 为 `txg` 的那条根所在的根槽改坏一个字节（自证校验和罩整槽）：那条根读不出。
fn make_the_root_unreadable(pool: &mut BuiltPool, txg: CheckpointTxg) {
    let target = target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("那块盘");
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let mut bytes = vec![0u8; root_slot_bytes];
    recorded.read_at(offset, &mut bytes).expect("读根槽");
    bytes[100] ^= 0xff;
    recorded
        .write_at(offset, &bytes, WriteDurability::Plain)
        .expect("改坏根槽");
}

/// 一张盘上的池级 checker 走两段（用户 2026-10-13 定「能走两段的都走两段」）：那张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）
/// 交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`），录入段写进给定状态表那一类的判定库，判段从库里读回判
/// （配了 GPU 走 GPU 判器，没配由 CPU 判器判），判过的复用。只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
///
/// # Panics
/// 引擎起不了、判不了这张表（不退回别处判）；GPU 判不了这张盘（第三轮正文 H6 判红）。
fn pool_checker_conclusion_through_the_given_states_engine(
    table_name: &str,
    image: &MemoryPool,
) -> Option<GivenStateConclusion> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "rollback_floor_written_into_the_system_configuration_first::fixed"
                    .to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的盘照第三轮正文 H6 判红"
    );
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
    }
    let [conclusion] = <[GivenStateConclusion; 1]>::try_from(judged.states)
        .unwrap_or_else(|states| panic!("一个状态一份结论，交回 {} 份", states.len()));
    Some(conclusion)
}

/// 池级 checker 在这个结论里把 `invariant` 真判过（成立或违例，不是不适用、部分没判）且没判违例。
fn invariant_is_judged_and_holds(conclusion: &GivenStateConclusion, invariant: &str) -> bool {
    let position = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|implemented| *implemented == invariant)
        .unwrap_or_else(|| panic!("{invariant} 在池级 checker 的清单里"));
    let bit = 1u128 << position;
    conclusion.words.pool_checker_judged_bits & bit != 0
        && conclusion.words.pool_checker_violated_bits & bit == 0
}

/// SysPre（D16（发布语义） 已定项 1「抬 F 那一串」、已定项 7）：新池新建文件之后覆盖写三次（txg 4–6），抬 F 到上限 3——
/// txg 6 末尾轮换写的那一槽先持久（代码审阅第 19 条）：C577 之后由 txg 6 那次发布返回之前那道池屏障兑现，它是抬 F 之前录制流的最后两步；
/// 这一串自己开头那道屏障前面没有写，录制器把它与前一道并成一道（设备上照收 FLUSH），录制流里这一串从系统配置写起。
/// 接着两块盘各一次系统配置槽写、带新 F 3（tail 与实例代号照现行那一版末次轮换写的），再一道池屏障，第一次根槽 FUA 写在它之后；
/// 录制流按设备记屏障（实审 B3a-2 第 1 条），每道池屏障是两块盘各一步；
/// 那两次写的账（两次、8192 字节）单列在 `system_configuration_writes_before_the_first_publish`，不在任何一次发布里。
/// 做完之后每块盘两槽里读得出的最大 F 是 3，F 生效值是 3。
#[test]
fn raising_the_floor_writes_the_new_floor_into_every_system_configuration_behind_a_barrier_before_the_first_root(
) {
    let mut pool = build_pool("rbf2-syspre-order");
    overwrite_times(&mut pool, 3);
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(6));
    let tail_before_the_raise = pool.output.record.counter;
    let operations_before_the_raise = pool.stream.operation_count();
    let raised = raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到上限 3");
    assert_eq!(
        step_kinds_since(&pool.stream, operations_before_the_raise - 2)[..2],
        [
            (StepKind::Barrier, DeviceIdentity(0)),
            (StepKind::Barrier, DeviceIdentity(1)),
        ],
        "抬 F 之前录制流的最后两步是 txg 6 那次发布返回之前那道池屏障（C577）：txg 6 末尾轮换的那一槽先持久（代码审阅第 19 条）"
    );
    let steps = step_kinds_since(&pool.stream, operations_before_the_raise);
    assert_eq!(
        steps[..4],
        [
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
            (StepKind::Barrier, DeviceIdentity(0)),
            (StepKind::Barrier, DeviceIdentity(1)),
        ],
        "抬 F 那一串的头四步：两块盘各写一次系统配置槽、再一道池屏障（两块盘各一步）；开头那道屏障与 txg 6 返回之前那道之间没有写，录制流里并成一道"
    );
    let first_root_step = steps
        .iter()
        .position(|(kind, _)| *kind == StepKind::RootRecordFua)
        .expect("这一串写了根");
    assert!(
        first_root_step > 3,
        "第一条带新 F 的根在写完系统配置之后那道池屏障之后：{first_root_step}"
    );
    let retained = pool.stream.retained_operations();
    // 头四步里第 0、1 步是两块盘的系统配置槽写（第 2、3 步是那道池屏障）。
    for step in 0..2 {
        let written = SystemConfiguration::parse_slot(
            retained[operations_before_the_raise + step]
                .contents
                .as_deref()
                .expect("录制流留着内容"),
        )
        .expect("先写的那一槽自证得过");
        assert_eq!(
            (
                written.quantities.rollback_floor,
                written.quantities.journal_tail,
                written.quantities.journal_instance
            ),
            (
                CheckpointTxg(3),
                tail_before_the_raise,
                InstanceGeneration(1)
            ),
            "第 {step} 步：带新 F，tail 与实例代号照现行那一版"
        );
    }
    assert_eq!(
        raised
            .system_configuration_writes_before_the_first_publish
            .total(),
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 2 * 4096,
        },
        "先写系统配置那一步的账单列：每块盘一次整槽写"
    );
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (publish.root.checkpoint_txg, publish.root.rollback_floor))
            .collect::<Vec<_>>(),
        vec![
            (CheckpointTxg(7), CheckpointTxg(3)),
            (CheckpointTxg(8), CheckpointTxg(3))
        ],
        "两次空发布带新 F，落到两块盘"
    );
    let devices = pool.devices.as_ref().expect("镜像还开着");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            system_configuration_floors_on(devices.as_slice(), device)
                .into_iter()
                .max(),
            Some(CheckpointTxg(3)),
            "盘 {}：两槽里读得出的最大 F 是新 F",
            device.0
        );
    }
    assert_eq!(effective_floor_of(devices.as_slice()), CheckpointTxg(3));
}

/// 生效取 SysPre：抬到 3 之后把两条带新 F 的根（txg 7、8）都改坏，根环里读得出的根带的 F 全是 0，而两块盘系统配置里的 F 还是 3——
/// F 生效值读出 3（改之前按「各盘所带 F 最大值的最小值」只读根，是 0）；重开可写挂载，新实例写行那次发布的根带的就是它。
#[test]
fn the_floor_takes_effect_from_the_system_configuration_when_every_root_carrying_it_is_unreadable()
{
    let mut pool = build_pool("rbf2-effective-from-system-configuration");
    overwrite_times(&mut pool, 3);
    raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到上限 3");
    make_the_root_unreadable(&mut pool, CheckpointTxg(7));
    make_the_root_unreadable(&mut pool, CheckpointTxg(8));
    let mut devices = pool.reopen_recorded();
    assert_eq!(
        readable_roots_of(devices.as_slice())
            .iter()
            .map(|root| root.rollback_floor)
            .max(),
        Some(CheckpointTxg(0)),
        "带新 F 的两条根都读不出了：根环里读得出的根带的 F 全是 0"
    );
    assert_eq!(
        effective_floor_of(devices.as_slice()),
        CheckpointTxg(3),
        "F 生效值取系统配置里的 3"
    );
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    assert_eq!(
        mounted.output.row_publish.root().rollback_floor,
        CheckpointTxg(3),
        "新实例写行那次发布的根带 F 生效值"
    );
}

/// 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「回退候选集」「生效」），不是最新根自己带的 F、也不是目标根自己带的 F。
/// 头几步照下面那条「先写系统配置那一步失败」：覆盖写三次到 txg 6，抬 F 到 3，第二次写（盘 1 的系统配置槽）报块设备错——
/// 只有盘 0 的系统配置带新 F 3，一条带新 F 的根都没发；最新读得出的根仍是 txg 6（带 F 0），F 生效值是 3。
/// 同一个进程里回退到暖机那条 txg 2 的根（实例 1，根槽 (1, 2)，带 F 0），在任何写之前拒成 `BelowEffectiveFloor`：录制流不多一步、
/// 分配器与现行版本不动。按最新根或目标根自己的 F（都是 0）判，它过得了下界这一道，报的是「树表 0 条」（`VersionWithoutFile`）。
///
/// 为什么不用原来的造法（抬到 3 做成、再把 txg 7、8 那两条带新 F 的根改坏）：31-1 之后 `build_pool` 的分配器装了根环表，那两槽是
/// 这个进程写过的、「这个进程知道住着根的槽」，判候选与算 F_生效 读根环时重读一次仍坏，就先拒成 `CandidateJudgementStillUnreadableAfterOneReread`、
/// 到不了下界这一判（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）；那一形由 `reuse_after_raising_the_floor.rs` 的
/// `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write` 钉成先拒。
/// 这一造法一个根槽都不碰，下界只能从盘 0 的系统配置里来。
#[test]
fn rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write() {
    let mut pool = three_overwrites_on_fault_injected_devices();
    let roots_before_the_raise = readable_roots_of(pool.devices.as_slice());
    pool.fault_plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            2,
        ));
    let mut current_for_the_raise = pool.output.clone();
    let refused_raise = raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut current_for_the_raise,
        CheckpointTxg(3),
        ShadowLedger::On,
    );
    pool.fault_plan.disarm();
    assert!(
        matches!(
            &refused_raise,
            Err(MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed))
                if failed.devices_carrying_the_raised_floor == vec![DeviceIdentity(0)]
        ),
        "盘 1 的系统配置写报错：先写系统配置那一步失败、只有盘 0 带新 F：{:?}",
        refused_raise.as_ref().err()
    );
    let roots_after_the_raise = readable_roots_of(pool.devices.as_slice());
    assert_eq!(
        roots_after_the_raise, roots_before_the_raise,
        "抬 F 这一串一条根都没发：根环逐条不变"
    );
    let newest_readable_root = roots_after_the_raise
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("根环里有根");
    assert_eq!(
        (
            newest_readable_root.checkpoint_txg,
            newest_readable_root.rollback_floor
        ),
        (CheckpointTxg(6), CheckpointTxg(0)),
        "最新读得出的根是 txg 6、带 F 0"
    );
    let target_root = roots_after_the_raise
        .iter()
        .find(|root| {
            root.instance == InstanceGeneration(1) && root.checkpoint_txg == CheckpointTxg(2)
        })
        .expect("暖机那条 txg 2 的根读得出");
    assert_eq!(
        target_root.rollback_floor,
        CheckpointTxg(0),
        "目标根自己带 F 0"
    );
    assert_eq!(
        (
            system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(0))
                .into_iter()
                .max(),
            system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(1))
                .into_iter()
                .max()
        ),
        (Some(CheckpointTxg(3)), Some(CheckpointTxg(0))),
        "只有盘 0 的系统配置带新 F 3，盘 1 没写进"
    );
    // F 生效值是 3 这里不另断言：同一盘面在下面那条「先写系统配置那一步失败」里断言过；F_生效 不取系统配置的变异要红在回退判候选这一判上，
    // 不在前面的搭建核对上。
    let operations_before_the_rollback = pool.stream.operation_count();
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refused = roll_back_by_a_forward_publish(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
    );
    assert!(
        matches!(
            &refused,
            Err(RollbackError::TargetNotACandidate {
                target: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(2),
                },
                exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
            })
        ),
        "txg 2 低于 F 生效值 3：不是候选：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operation_count(),
        operations_before_the_rollback,
        "拒在任何写之前：录制流不多一步"
    );
    assert_eq!(pool.output, current_before, "现行版本不动");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
}

/// 非抬 F 的系统配置写带整池的 F 生效值（主 agent 2026-09-26 定）：抬到 3 之后把盘 1 两个系统配置槽都改坏，再覆盖写一次——
/// 那次发布末尾轮换写到盘 1 的那一槽（两槽都读不出，世代号从 1 起、落槽 1）带的 F 是整池的 3，不是这块盘自己读得出的（没有，写 0）。
#[test]
fn system_configuration_write_after_both_slots_of_one_device_became_unreadable_carries_the_pool_effective_floor(
) {
    let mut pool = build_pool("rbf2-pool-effective-floor");
    overwrite_times(&mut pool, 3);
    raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到上限 3");
    make_both_system_configuration_slots_unreadable(&mut pool, DeviceIdentity(1));
    {
        let devices = pool.devices.as_ref().expect("镜像还开着");
        assert_eq!(
            system_configuration_floors_on(devices.as_slice(), DeviceIdentity(1)),
            Vec::<CheckpointTxg>::new(),
            "盘 1 两槽都自证不过"
        );
    }
    overwrite_times(&mut pool, 1);
    let devices = pool.devices.as_ref().expect("镜像还开着");
    let written_on_device_one =
        system_configuration_floors_on(devices.as_slice(), DeviceIdentity(1));
    assert_eq!(
        written_on_device_one,
        vec![CheckpointTxg(3)],
        "盘 1 上那次轮换写出的一槽带整池的 F 生效值 3"
    );
}

/// 正常卸载（D16（发布语义） 已定项 1「正常卸载」，B1）：覆盖写三次到 txg 6（除以 3 余 0）⇒ 推 2 次空发布（txg 7 落盘 1、8 落盘 0）；
/// 覆盖写四次到 txg 7（余 1）⇒ 推 3 次（txg 8、9 落盘 0，10 落盘 1）。F 抬到卸载开始时现行那一版的 txg、不判上限（这段历史上准入上限是 3）；
/// 这一串写的根全带卸载记号，写出去的根槽读回来也带；卸载之前那条根不带；头几步照样先写系统配置（带新 F）、过一道池屏障。
/// 卸载之后的镜像 checker 一条都不红，I-7.9（回退下界 F 不高于抬 F 的上限） 按带记号那一支真被评估过且成立（按准入上限 3 判就红了），
/// I-7.12（系统配置 F 不低于同盘根上的 F） 成立；再可写挂载，择到带记号的最后一条根，新实例的根带 F 生效值、不带记号。
// 一条用例一个场景（`.claude/rules/verification.md`「harness 档里再分快档用例与慢档用例」）：现行 txg 除以 3 余 0 与余 1 两格各建一个池，
// 拆成两条用例，各调这一份。
fn normal_unmount_after_overwrites_raises_the_floor_with_marked_empty_publishes(
    overwrites: usize,
    expected_txgs: Vec<CheckpointTxg>,
) {
    let mut pool = build_pool(&format!("rbf2-unmount-{overwrites}"));
    overwrite_times(&mut pool, overwrites);
    let current_txg = pool.output.root.checkpoint_txg;
    assert_eq!(
        pool.output.root.unmount_marker,
        UnmountMarker::NotWrittenByTheUnmountSequence
    );
    let operations_before_the_unmount = pool.stream.operation_count();
    let unmounted = unmount_recorded(&mut pool).expect("正常卸载");
    assert_eq!(
        unmounted.rollback_floor, current_txg,
        "F 抬到现行那一版的 txg"
    );
    assert_eq!(
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        expected_txgs,
        "现行 txg {} 除以 3 余 {}：推 {} 次",
        current_txg.0,
        current_txg.0 % 3,
        expected_txgs.len()
    );
    for publish in &unmounted.publishes {
        assert_eq!(
            (publish.root.rollback_floor, publish.root.unmount_marker),
            (current_txg, UnmountMarker::WrittenByTheUnmountSequence),
            "txg {}：带新 F、带卸载记号",
            publish.root.checkpoint_txg.0
        );
    }
    assert_eq!(
        step_kinds_since(&pool.stream, operations_before_the_unmount - 2)[..6],
        [
            (StepKind::Barrier, DeviceIdentity(0)),
            (StepKind::Barrier, DeviceIdentity(1)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(0)),
            (StepKind::SystemConfigurationSlot, DeviceIdentity(1)),
            (StepKind::Barrier, DeviceIdentity(0)),
            (StepKind::Barrier, DeviceIdentity(1)),
        ],
        "卸载那一串照抬 F 那一串：上一次发布返回之前那道池屏障（C577；卸载开头那道与它之间没有写，录制流里并成一道）、\
         先写系统配置、过池屏障（录制流按设备记屏障，每道两块盘各一步）"
    );
    let image = pool.memory_pool();
    let roots_on_disk = readable_roots_of(&image);
    for root in &roots_on_disk {
        let expected_marker = if expected_txgs.contains(&root.checkpoint_txg) {
            UnmountMarker::WrittenByTheUnmountSequence
        } else {
            UnmountMarker::NotWrittenByTheUnmountSequence
        };
        assert_eq!(
            root.unmount_marker, expected_marker,
            "盘上读回的 txg {} 那条根",
            root.checkpoint_txg.0
        );
    }
    let Some(conclusion) = pool_checker_conclusion_through_the_given_states_engine(
        &format!("after-the-unmount-of-{overwrites}-overwrites"),
        &image,
    ) else {
        return;
    };
    let violations = conclusion.pool_checker_violations();
    assert_eq!(
        violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<&str>>(),
        Vec::<&str>::new(),
        "卸载之后的镜像一条都不红：{violations:?}"
    );
    for invariant in ["I-7.9", "I-7.12"] {
        assert!(
            invariant_is_judged_and_holds(&conclusion, invariant),
            "{invariant} 真被评估过且成立"
        );
    }
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("卸载之后再可写挂载");
    pool.devices = Some(devices);
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.chosen_root.unmount_marker
        ),
        (
            *expected_txgs.last().expect("至少一次"),
            UnmountMarker::WrittenByTheUnmountSequence
        ),
        "择到卸载那一串的最后一条根"
    );
    assert_eq!(
        (
            mounted.output.row_publish.root().rollback_floor,
            mounted.output.row_publish.root().unmount_marker
        ),
        (current_txg, UnmountMarker::NotWrittenByTheUnmountSequence),
        "新实例写行那次发布的根带 F 生效值、不带记号"
    );
}

/// 覆盖写三次到 txg 6（除以 3 余 0）：推 2 次空发布（txg 7 落盘 1、8 落盘 0）。
#[test]
fn normal_unmount_raises_the_floor_to_the_current_txg_with_marked_empty_publishes_on_every_device_after_writing_the_system_configuration(
) {
    normal_unmount_after_overwrites_raises_the_floor_with_marked_empty_publishes(
        3,
        vec![CheckpointTxg(7), CheckpointTxg(8)],
    );
}

/// 覆盖写四次到 txg 7（除以 3 余 1）：推 3 次空发布（txg 8、9 落盘 0，10 落盘 1）。
#[test]
fn normal_unmount_raises_the_floor_with_three_marked_empty_publishes_when_the_current_txg_leaves_remainder_one_by_three(
) {
    normal_unmount_after_overwrites_raises_the_floor_with_marked_empty_publishes(
        4,
        vec![CheckpointTxg(8), CheckpointTxg(9), CheckpointTxg(10)],
    );
}

/// 树表 0 条的一版（还没写过文件）上正常卸载：一个字节都不写，卸载照常做成（主 agent 2026-09-26 定，实二交回第一问：回退候选要带文件，
/// 这一版的时间线上还没有带文件的版本，抬 F 买不到东西）。只做过 mkfs 的池可写挂载之后现行那一版就是树表 0 条的；卸载交回
/// `NothingWrittenOnAVersionWithoutFile`、报出现行那一版，盘上逐字节不变（系统配置槽、根环、录制流一步不多），分配器与现行版本不动。
#[test]
fn unmount_of_a_version_without_file_writes_nothing_and_succeeds() {
    let mut pool = format_pool("rbf2-unmount-without-file");
    let mut reopened = pool.reopen_recorded();
    let mut mounted =
        mount_writable(&parameters(), &mut reopened).expect("只做过 mkfs 的池可写挂载");
    pool.devices = Some(reopened);
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_root = *mounted.current.root();
    let records_before = mounted.allocator.records().to_vec();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let unmounted = unmount(
        &parameters(),
        devices,
        &mut mounted.allocator,
        &mut mounted.current,
        ShadowLedger::On,
    )
    .expect("树表 0 条的一版上卸载照常做成");
    match unmounted {
        Unmounted::NothingWrittenOnAVersionWithoutFile { current } => assert_eq!(
            (current.instance, current.checkpoint_txg),
            (current_root.instance, current_root.checkpoint_txg),
            "报出现行那一版"
        ),
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => panic!(
            "树表 0 条的一版上卸载不抬 F，却推了 {} 次发布",
            raised.publishes.len()
        ),
    }
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "一个字节都不写：录制流一步不多"
    );
    assert_eq!(*mounted.current.root(), current_root, "现行版本不动");
    assert_eq!(
        mounted.allocator.records(),
        records_before.as_slice(),
        "分配器不动"
    );
}

/// 往下「抬」F 条款没写（`RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`）：抬到 3 之后再要抬到 2，
/// 在任何写之前拒，报出要的 2 与盘上的生效值 3，盘上逐字节不变；再抬到 3（等于生效值）照常做。
#[test]
fn raising_below_the_effective_floor_is_refused_before_any_write() {
    let mut pool = build_pool("rbf2-raise-below");
    overwrite_times(&mut pool, 3);
    raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到上限 3");
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = raise_floor(&mut pool, CheckpointTxg(2));
    assert!(
        matches!(
            &refused,
            Err(
                MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                    requested: CheckpointTxg(2),
                    effective: CheckpointTxg(3),
                }
            )
        ),
        "往下抬拒成条款未定那一条：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "拒在任何写之前"
    );
    raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到等于生效值照常做");
}

/// 内存盘外面包录制器，录制器外面包故障注入（注入层在录制器外面：报错的那次写不进录制流）。
type FaultInjectedDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

struct PoolOnFaultInjectedDevices {
    devices: Vec<(DeviceIdentity, FaultInjectedDevice)>,
    stream: SharedStream,
    fault_plan: SharedFaultPlan,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

/// 两块 4 GiB 内存盘、注入计划先不开，mkfs 同一个进程里第一个文件（txg 3）之后覆盖写三次（txg 4–6）。
fn three_overwrites_on_fault_injected_devices() -> PoolOnFaultInjectedDevices {
    let parameters = parameters();
    let stream = SharedStream::new();
    let fault_plan = SharedFaultPlan::unarmed(geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectedDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                            stream.clone(),
                        ),
                        fault_plan.clone(),
                    ),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let output = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let mut output = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        for seed in 0..3 {
            output = publish_overwrite(
                &mut writer,
                &mut allocator,
                &output,
                FirstFile {
                    content: &content_of(2000 + 97 * seed, seed),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                instance,
            )
            .expect("覆盖写");
        }
        output
    };
    assert_eq!(output.root.checkpoint_txg, CheckpointTxg(6));
    PoolOnFaultInjectedDevices {
        devices,
        stream,
        fault_plan,
        allocator,
        output,
    }
}

/// 先写系统配置那一步失败（「抬 F 那一串」那一行，主 agent 2026-09-26 定）：抬 F 到 3，第二次写（盘 1 的系统配置槽）报块设备错——
/// 报 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`、点名只有盘 0 已带新 F、这一步记下的写是盘 0 那一次；
/// 这一串的根一条都没发（根环逐条不变、录制流只多盘 0 那一次写——写之前那道池屏障与 txg 6 返回之前那道（C577）之间没有写、录制流里并成一道——、
/// 报错之后没有屏障），调用方的现行版本没动，分配器与抬 F 之前逐项相同；
/// 盘 0 系统配置里的新 F 留着、不回卷，F 生效值已经是 3（「只增不减」）。之后同一个进程里要抬到 2（低于生效值）在任何写之前拒；
/// 再抬到 3 做成，两块盘都带上新 F。
#[test]
fn failing_to_write_the_new_floor_into_the_second_system_configuration_publishes_no_root_and_keeps_the_first_device_carrying_it(
) {
    let mut pool = three_overwrites_on_fault_injected_devices();
    let roots_before = readable_roots_of(pool.devices.as_slice());
    let allocator_before = format!("{:?}", pool.allocator);
    let operations_before = pool.stream.operation_count();
    pool.fault_plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            2,
        ));
    let mut current = pool.output.clone();
    let refused = raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    );
    pool.fault_plan.disarm();
    let Err(MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed)) = &refused
    else {
        panic!(
            "盘 1 的系统配置写报错，报的应是先写系统配置那一步：{:?}",
            refused.as_ref().err()
        );
    };
    assert_eq!(
        failed.devices_carrying_the_raised_floor,
        vec![DeviceIdentity(0)],
        "报错之前只有盘 0 写进了新 F"
    );
    assert_eq!(
        failed.writes.total(),
        WriteCallsAndBytes {
            write_calls: 1,
            written_bytes: 4096,
        },
        "这一步记下的写：盘 0 那一次整槽写"
    );
    let recorded: Vec<(RecordedOperationKind, DeviceIdentity)> = pool.stream.operations()
        [operations_before..]
        .iter()
        .map(|operation| (operation.kind, operation.device))
        .collect();
    assert_eq!(
        recorded,
        vec![(RecordedOperationKind::Write, DeviceIdentity(0))],
        "录制流只多盘 0 那一次写：写之前那道池屏障（代码审阅第 19 条）与 txg 6 返回之前那道（C577）之间没有写、录制流里并成一道；\
         报错之后一个写、一道屏障都没发"
    );
    assert_eq!(
        pool.stream.operations()[operations_before - 2..operations_before]
            .iter()
            .map(|operation| (operation.kind, operation.device))
            .collect::<Vec<_>>(),
        vec![
            (RecordedOperationKind::Barrier, DeviceIdentity(0)),
            (RecordedOperationKind::Barrier, DeviceIdentity(1)),
        ],
        "盘 0 那一次写之前录制流的最后两步是 txg 6 返回之前那道池屏障：覆写旧槽时新槽已持久"
    );
    assert_eq!(
        readable_roots_of(pool.devices.as_slice()),
        roots_before,
        "这一串的根一条都没发"
    );
    assert_eq!(current, pool.output, "调用方的现行版本没动");
    assert!(
        format!("{:?}", pool.allocator) == allocator_before,
        "分配器与抬 F 之前逐项相同（扣住的槽放开、回收的回到 defer、补的隔离撤掉）"
    );
    assert_eq!(
        system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(0))
            .into_iter()
            .max(),
        Some(CheckpointTxg(3)),
        "盘 0 系统配置里的新 F 留着、不回卷"
    );
    assert_eq!(
        system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(1))
            .into_iter()
            .max(),
        Some(CheckpointTxg(0)),
        "盘 1 没写进"
    );
    assert_eq!(
        effective_floor_of(pool.devices.as_slice()),
        CheckpointTxg(3),
        "F 生效值只增不减：已经是 3"
    );
    let operations_before_the_refusal = pool.stream.operation_count();
    let mut current_for_the_lower_raise = pool.output.clone();
    let below = raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut current_for_the_lower_raise,
        CheckpointTxg(2),
        ShadowLedger::On,
    );
    assert!(
        matches!(
            below,
            Err(
                MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                    requested: CheckpointTxg(2),
                    effective: CheckpointTxg(3),
                }
            )
        ),
        "同一个进程里现行版本的根还带 F 0，要抬到 2 仍低于盘上的生效值 3：{:?}",
        below.as_ref().err()
    );
    assert_eq!(
        pool.stream.operation_count(),
        operations_before_the_refusal,
        "往下抬拒在任何写之前"
    );
    let mut current_for_the_retry = pool.output.clone();
    let raised = raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut current_for_the_retry,
        CheckpointTxg(3),
        ShadowLedger::On,
    )
    .expect("再抬到 3 做成");
    assert_eq!(raised.publishes.len(), 2);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            system_configuration_floors_on(pool.devices.as_slice(), device)
                .into_iter()
                .max(),
            Some(CheckpointTxg(3)),
            "盘 {}：带上新 F",
            device.0
        );
    }
}

/// C557（卸载记号只由卸载入口打没有检查）：录制流上逐条核「带卸载记号的根只出现在卸载入口发的那一串里」。
/// ① 卸载那一串记成卸载入口发的一段：流里带记号的根槽写（txg 7、8）都在那一段里，一条都不列；
/// ② 同一条流不带那一段去核（等于说这两次根槽写不是卸载入口发的）：两条都列出来，按流的次序；
/// ③ 准入入口（抬 F）写的根被人打上记号（改流里第一条抬 F 根槽写的 flags、重封自证校验和）：那一条列出来。
#[test]
fn unmount_markers_are_judged_against_the_recorded_unmount_entry() {
    let mut pool = build_pool("rbf2-c557-unmount");
    overwrite_times(&mut pool, 3);
    unmount_recorded(&mut pool).expect("正常卸载");
    let operations = pool.retained_operations();
    let entry_spans = pool.stream.entry_spans();
    assert_eq!(entry_spans.len(), 1, "只记了卸载那一段");
    assert_eq!(
        unmount_markers_outside_the_unmount_entry(&operations, &entry_spans, &geometry()),
        Vec::<UnmountMarkerOutsideTheUnmountEntry>::new(),
        "带记号的根槽写都在卸载入口那一段里"
    );
    let outside_without_the_entry =
        unmount_markers_outside_the_unmount_entry(&operations, &[], &geometry());
    assert_eq!(
        outside_without_the_entry
            .iter()
            .map(|stray| (stray.instance, stray.checkpoint_txg))
            .collect::<Vec<_>>(),
        vec![
            (InstanceGeneration(1), CheckpointTxg(7)),
            (InstanceGeneration(1), CheckpointTxg(8)),
        ],
        "不带卸载入口那一段去核：两条带记号的根槽写都在入口之外"
    );
    for stray in &outside_without_the_entry {
        assert!(
            entry_spans[0].operations.contains(&stray.stream_index),
            "列出来的下标就是卸载入口那一段里的那两次根槽写：{stray:?}"
        );
    }

    let mut raised_pool = build_pool("rbf2-c557-admission");
    overwrite_times(&mut raised_pool, 3);
    let operations_before_the_raise = raised_pool.stream.operation_count();
    raise_floor(&mut raised_pool, CheckpointTxg(3)).expect("抬到上限 3");
    let mut tampered = raised_pool.retained_operations();
    let first_raise_root = (operations_before_the_raise..tampered.len())
        .find(|index| geometry().classify(&tampered[*index].operation) == StepKind::RootRecordFua)
        .expect("抬 F 那一串写了根");
    let bytes = tampered[first_raise_root]
        .contents
        .as_mut()
        .expect("录制流留着内容");
    bytes[20] |= 0x01;
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        bytes,
        bytes.len(),
        ROOT_CHECKSUM_OFFSET,
    );
    bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    assert_eq!(
        unmount_markers_outside_the_unmount_entry(
            &raised_pool.retained_operations(),
            &raised_pool.stream.entry_spans(),
            &geometry()
        ),
        Vec::<UnmountMarkerOutsideTheUnmountEntry>::new(),
        "没改之前准入入口写的根都不带记号"
    );
    assert_eq!(
        unmount_markers_outside_the_unmount_entry(
            &tampered,
            &raised_pool.stream.entry_spans(),
            &geometry()
        ),
        vec![UnmountMarkerOutsideTheUnmountEntry {
            stream_index: first_raise_root,
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(7),
        }],
        "准入入口写的根被打上卸载记号：列出来"
    );
}

/// C557 那一道接在崩溃点重放切写表的入口上（`crash::writes_and_segments_with_stream_indexes`）：没记入口的流里一条带卸载记号的根槽写都不许有。
/// 卸载那一串的流不带入口段交给它，当场停，停的理由点名 C557；卸载之前那一段流（没有带记号的根槽写）照常切。
#[test]
fn cutting_a_stream_with_an_unmount_marker_outside_any_recorded_unmount_entry_panics() {
    let mut pool = build_pool("rbf2-c557-writes-and-segments");
    overwrite_times(&mut pool, 3);
    let operations_before_the_unmount = pool.stream.operation_count();
    unmount_recorded(&mut pool).expect("正常卸载");
    let operations = pool.retained_operations();
    let (writes_before_the_unmount, _segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &operations[pool.mkfs_operation_count..operations_before_the_unmount],
            &geometry(),
        );
    assert!(
        !writes_before_the_unmount.is_empty(),
        "卸载之前那一段流照常切"
    );
    let cut = std::panic::catch_unwind(|| {
        writes_and_segments_with_stream_indexes(
            &operations[pool.mkfs_operation_count..],
            &geometry(),
        )
    });
    let Err(panic_payload) = cut else {
        panic!("带卸载记号的根槽写不在任何卸载入口段里，切写表必须当场停");
    };
    let message = panic_payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic_payload.downcast_ref::<&str>().copied())
        .expect("assert! 的停机消息是字符串");
    assert!(message.contains("C557"), "停的理由点名 C557：{message}");
}

/// C556（checker 与崩溃点重放不读系统配置里的 F） 的崩溃点重放那一半：记录核对器的复用豁免判「那次复用过不过得了回收谓词」时，
/// F 生效值的界把系统配置槽写带的 F 也算进去（生效值取根上带的与系统配置里的最大值）。
/// 造法同 `record_checker_reuse_exemption_legality.rs`：复用窗口置 0，txg 5 的数据单元落回 txg 4 的那一对槽 50178，
/// 崩在 txg 5 的单元写全部落盘、记录与根槽都没落盘那一刻——只按根上的 F（0）界，这次复用过不了回收谓词、记录核对器判红；
/// 把写表里 txg 5 单元写之前那两次系统配置槽写换成带 F 5 的（整槽重封），界抬到 5，那次复用过得了、记录核对器开脱。
#[test]
fn the_record_checker_bounds_the_effective_floor_by_the_floor_written_into_the_system_configuration(
) {
    let mut pool = build_pool("rbf2-record-checker-system-configuration-floor");
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);
    overwrite_times(&mut pool, 2);
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(5));
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, _segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let (fourth_root_index, fifth_root_index) = match root_indexes.as_slice() {
        [.., fourth_root_index, fifth_root_index] => (*fourth_root_index, *fifth_root_index),
        too_few => panic!("写表里至少有暖机、A、txg 4、txg 5 的根槽写：{too_few:?}"),
    };
    let reused_slot = SlotNumber(50178);
    let copies_at_the_reused_slot = writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite && write.offset == reused_slot.to_device_offset()
        })
        .count();
    assert_eq!(
        copies_at_the_reused_slot, 4,
        "50178 上 txg 4 与复用它的 txg 5 各两盘一份（复用窗口置 0）"
    );
    let first_record_of_the_fifth_publish = (fourth_root_index + 1..fifth_root_index)
        .find(|index| writes[*index].kind == StepKind::JournalRecord)
        .expect("txg 5 写了 journal 记录");
    let persisted: Vec<bool> = (0..writes.len())
        .map(|index| index < first_record_of_the_fifth_publish)
        .collect();
    let versions = [
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: content_of(2000, 0),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
            content: content_of(2097, 1),
        },
    ];
    let record_checker_red_states =
        |write_table: &[singlefs_harness::memory_pool::RetainedWrite]| {
            let mut tally = CrashPointReplayTally::default();
            let _report = evaluate_state_for_versions(
                &base,
                write_table,
                persisted.clone(),
                fifth_root_index,
                &versions,
                &mut tally,
            );
            tally.record_claimed_state_missing_unit
        };
    assert_eq!(
        record_checker_red_states(&writes),
        1,
        "只按根上的 F（0）界：复用过不了回收谓词，记录核对器判红"
    );
    let mut with_the_floor_in_the_system_configuration = writes.clone();
    let rotations_before_the_fifth_units: Vec<usize> = (fourth_root_index + 1..fifth_root_index)
        .filter(|index| writes[*index].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotations_before_the_fifth_units.len(),
        2,
        "txg 4 末尾两块盘各一次轮换"
    );
    for index in rotations_before_the_fifth_units {
        let write = &mut with_the_floor_in_the_system_configuration[index];
        let mut system_configuration =
            SystemConfiguration::parse_slot(write.bytes().expect("系统配置槽写带着字节"))
                .expect("轮换写的槽自证得过");
        system_configuration.quantities.rollback_floor = CheckpointTxg(5);
        write.contents = WrittenContents::Bytes(system_configuration.to_slot());
    }
    assert_eq!(
        record_checker_red_states(&with_the_floor_in_the_system_configuration),
        0,
        "系统配置槽写带 F 5：界抬到 5，那次复用过得了回收谓词，记录核对器开脱"
    );
}
