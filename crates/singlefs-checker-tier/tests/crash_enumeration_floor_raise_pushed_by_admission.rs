//! checker 档模块：crash、crash_state_enumeration、crash_amplification
//! 里程碑「覆盖写、释放、回退与复用」收尾实现批次实五：崩在准入推的那一串空发布中间（安全设计轮第三轮攻方 `research/prompts/m2-safety-r3-opus-output.md`
//! 第六节那一形；D16（发布语义） 已定项 1「准入」那一行，C283（准入失败时不先推发布就报 ENOSPC））。取一条历史、只录会话推的那一串抬 F，
//! 按崩溃点重放的枚举域做小枚举（状态数照崩溃点重放的口径另算：原地覆写取三态，每段 3^m · 2^(n−m) − 1 再加全部持久那一个，控制在 10⁶ 以内），
//! 恢复之后与挂载之后两份镜像的池级 checker 都判；挂载那一遍照枚举用的写表叠，撕裂那一态的镜像也交给挂载。
//! 它不是崩溃点重放的全量流，是一条历史上的一小段，平时跑得起。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
// 池经 `common_admission_pool` 搭、不经 `common_admission`：后者引随机历史脚手架（history.rs）与故障注入，引了就进这条登记用例的判法闭包。
#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;
#[cfg(feature = "verdict-store")]
mod common_crash_points;

use common_admission_pool::{
    content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_core::recovery::RecoveryReport;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs";
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_space_admission, push_one_floor_raise_within_the_admission_budget,
    FloorRaisePushedWithinTheAdmissionBudget,
};
use singlefs_core::transaction::{FirstFile, PoolVersion, PoolWriter, PublishError};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 录好的那一串抬 F：推之前的镜像、那一串的写表与段、最后一次根槽写、每一版（含推过之后重发的那一次覆盖写），与仍挂着的池。
struct RecordedFloorRaise {
    pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    last_root_write: usize,
    versions: Vec<PublishedVersion>,
}

/// 会话覆盖写 61 次之后直接发布到准入先拒（写这条用例时拒在第 62 次，准入的式子改过之后往后挪，按实际拒的那一次取）、推一串抬 F、
/// 再把那一次覆盖写重发做成；只录推的那一串。崩溃枚举用例与流水线用例共用这一段。
fn record_the_floor_raise_pushed_by_the_session() -> RecordedFloorRaise {
    let stream = SharedStream::retaining_contents();
    let stream_for_the_devices = stream.clone();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        move |identity, device| {
            RecordingBlockDevice::with_shared_stream(
                identity,
                device,
                stream_for_the_devices.clone(),
            )
        },
    );
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_index in 1..=61 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    // 之后直接调发布路径，一次一次发，直到准入先拒（每块盘段外成对的空槽 0 对，走固定点之前「这次的单元落得下」那一判拒，在任何写之前返回）；
    // 拒之前做成的每一次都当现行那一版记下（版本表要它）。写这条用例时拒在第 62 次；准入的式子改过之后拒的那一次会往后挪，按实际拒的那一次取，
    // 200 次之内拒不了就是场景没了、用例红。拒了再照会话那样推一串抬 F（`push_one_floor_raise_within_the_admission_budget`，会话调的就是它），只录这一串。
    let mut session = pool.session.take().expect("这次挂载的会话");
    let mut refused_at: Option<usize> = None;
    for attempt in 62..=262 {
        pool.write_time_seconds += 1;
        let content = content_of(OVERWRITE_BYTES, pool.write_time_seconds);
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("挂载之后现行那一版带文件");
        };
        let outcome = {
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            singlefs_core::transaction::publish_overwrite(
                &mut writer,
                &mut session.allocator,
                current,
                FirstFile {
                    content: &content,
                    write_time_seconds: pool.write_time_seconds,
                },
                session.instance,
            )
        };
        match outcome {
            Err(PublishError::SpaceAdmissionRefused(_)) => {
                refused_at = Some(attempt);
                break;
            }
            Ok(published) => {
                pool.content_of_the_current_version = content.clone();
                pool.note_root_of_the_current_content(&published.root);
                *current = published;
            }
            Err(other) => panic!("第 {attempt} 次覆盖写不该这样失败：{other:?}"),
        }
    }
    let refused_at =
        refused_at.expect("200 次直接发布之内准入总要先拒一次（数据单元在段外落不下）");
    println!("FLOOR_RAISE_PUSHED_BY_THE_SESSION admission_refused_at_overwrite={refused_at}");
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("挂载之后现行那一版带文件");
    };
    let base = pool.image();
    let operations_before_the_raise = stream.operation_count();
    let FloorRaisePushedWithinTheAdmissionBudget::Raised(raised) =
        push_one_floor_raise_within_the_admission_budget(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            session.shadow_ledger,
            0,
        )
        .expect("推一串抬 F 没报错")
    else {
        panic!("推一串抬 F 做成：这一刻预算够、F 不在上限")
    };
    pool.session = Some(session);
    pool.note_floor_raises(std::slice::from_ref(&raised));
    let recorded = stream.retained_operations()[operations_before_the_raise..].to_vec();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let (writes, segments) = writes_and_segments(&recorded, &geometry);
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_writes.len(),
        raised.publishes.len(),
        "那一串里每次空发布一次根槽写"
    );
    let last_root_write = *root_writes.last().expect("那一串至少一次空发布");
    // 推过之后同一次覆盖写照会话那样重发，做成（这一格的结局与会话里那一次相同）。
    pool.overwrite(OVERWRITE_BYTES)
        .expect("推过抬 F 之后覆盖写做成");
    let versions = pool.published_versions();
    RecordedFloorRaise {
        pool,
        base,
        writes,
        segments,
        last_root_write,
        versions,
    }
}

/// 崩在准入推的那一串空发布中间（攻方「崩在准入推空发布的那一串中间」那一形）：会话里数据单元在段外落不下的那一刻（单元区 384 槽的小盘，
/// 崩了再挂之后会话覆盖写 61 次、再直接发布到准入拒（写这条用例时是第 62 次，准入的式子改过之后往后挪，用例按实际拒的那一次取、打一行
/// `FLOOR_RAISE_PUSHED_BY_THE_SESSION admission_refused_at_overwrite=`），见 `admission_raises_the_floor_before_refusing.rs` 的
/// `an_overwrite_whose_data_unit_has_no_slot_pair_outside_the_cluster_segments_is_refused_by_the_admission_and_published_after_raising_the_floor_in_the_session`；
/// ckpt_cost 按最坏情况计之后第 18、32、47 次是式子先拒、会话推过再发成，第 62 次是「这次的单元落得下」那一判先拒——C545 用户 2026-09-27 定准入先拒，
/// 此前那一次是走固定点时落点被拒），
/// 只录会话推的那一串抬 F（先把新 F 写进每块盘的系统配置、那几次带新 F 的空发布），按崩溃点重放的枚举域（每一段都展开）枚举这条流的每个崩溃状态：
/// 恢复之后 oracle 与池级 checker 0 违例（枚举器自己判）；再可写挂载（空间准入判着），挂载之后的镜像池级 checker 0 违例；
/// 挂载全做成。状态数照崩溃点重放的口径另算（系统配置槽写是原地覆写、取三态，[`crash_state_count_with_torn_in_place_overwrites`]），
/// 落在 10⁶ 以内才枚举；挂载那一遍的镜像照观察者交来的枚举用写表叠（录制流里的写在前，撕裂镜像与重放接在后面），不只叠录制流那几次写。
#[test]
#[ignore = "崩溃枚举：用户要求时由主 agent 经门禁 checker-tier-crash-point-replay --full 按输入哈希跑（release），提交时的快档只核它的全绿标记，平时不跑"]
fn crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green(
) {
    const STATES_AT_MOST: u64 = 1_000_000;
    let RecordedFloorRaise {
        pool,
        base,
        writes: writes_of_the_raise,
        segments: segments_of_the_raise,
        last_root_write: last_root_write_of_the_raise,
        versions,
    } = record_the_floor_raise_pushed_by_the_session();
    let states = crash_state_count_with_torn_in_place_overwrites(
        &base,
        &writes_of_the_raise,
        &segments_of_the_raise,
        &full_expansion,
    );
    assert!(
        states > 0 && states <= STATES_AT_MOST,
        "推的那一串的崩溃状态数 {states} 在 10⁶ 以内"
    );
    // 每个崩溃状态可写挂载一次、挂载之后的镜像跑池级 checker：当交回镜像的观察者挂进崩溃判定库那一路（两段，用户 2026-10-13 定）：
    // 录入段在 CPU 上挂（挂载真跑实现，豁免表的 writable-mount 那一行；按恢复镜像去重），挂不上当场 panic（记成红项、用例红）；
    // 判段在挂完的盘上判池级 checker（配了 GPU 上卡），判干净时 `mounted_and_checker_clean` 加一。观察者交来的 `image.writes` 是枚举用的写表
    // （录制流里的写在前、原地覆写的撕裂镜像与重放接在后面），`image.persisted` 与它逐条对应。
    // gpu-exempt: writable-mount
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   _counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        assert!(
            image.writes.len() > writes_of_the_raise.len(),
            "那一串先把新 F 写进每块盘的系统配置（原地覆写），撕裂镜像接在录制流那 {} 次写后面：枚举用的写表 {} 次",
            writes_of_the_raise.len(),
            image.writes.len()
        );
        let mut materialized = base.clone();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            if *is_persisted {
                write.contents.apply_to(
                    materialized
                        .devices
                        .get_mut(&write.device)
                        .expect("写落在池里的盘上"),
                    write.offset,
                );
            }
        }
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<_> = plain_devices_on(&materialized)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
        match mount_writable_with_space_admission(
            &pool.parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
        ) {
            Ok(_) => {
                let disks_after_the_observer = MemoryPool {
                    devices: devices
                        .iter()
                        .map(|(identity, device)| {
                            (*identity, device.wrapped_device().image.clone())
                        })
                        .collect(),
                    device_size_in_bytes: materialized.device_size_in_bytes,
                };
                let (writes_on_the_crash_image, _segments_of_the_mount) = writes_and_segments(
                    &stream.retained_operations(),
                    &fixed_geometry_of_the_384_slot_small_devices(),
                );
                ImageAfterTheObserver {
                    writes_on_the_crash_image,
                    disks_after_the_observer,
                    invariants_judged:
                        InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                    counter_when_the_pool_checker_is_clean: "mounted_and_checker_clean".to_string(),
                }
            }
            Err(error) => panic!("挂载失败：{error:?}"),
        }
    };
    let Some(tally) = enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: "floor_raise_pushed_by_the_session::full",
            setting_file: THIS_FILE,
        },
        &base,
        &writes_of_the_raise,
        &segments_of_the_raise,
        last_root_write_of_the_raise,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        &full_expansion,
        &[],
        // 观察者另读的输入：挂载参数（录这一串那个池的参数，几何与小盘一致）——它不随状态变，进版本与去重键时只取它的调试写法的摘要
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "writable_mount_after_the_crash",
            extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
                format!("{:?}", pool.parameters).as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveredImageBytes,
        }],
    )
    .judged() else {
        return;
    };
    assert_eq!(
        tally.states, states,
        "每一段都展开，状态数与按崩溃点重放口径另算的相同（原地覆写三态、其余两态）"
    );
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后 oracle 0 违例"
    );
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named("mounted_and_checker_clean"),
        states,
        "每个崩溃状态都挂得上、挂载之后池级 checker 0 违例"
    );
}

/// 抬 F 那一串接进崩溃放量流水线（用户 2026-09-29 定：崩溃注入之外的崩溃枚举用例都进流水线、由 GPU 判）：推之前的镜像是起点，
/// 只录推的那一串，一个点全部展开；判器是流水线的（两遍恢复、oracle、池级 checker、记录核对器）。挂载那一遍不在流水线里，
/// 归上面的崩溃枚举用例。展开上限从环境取（`SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`，没设 6）。
#[cfg(feature = "verdict-store")]
#[test]
fn crash_amplification_of_the_floor_raise_pushed_by_the_session_is_clean() {
    let recorded = record_the_floor_raise_pushed_by_the_session();
    let crash_points = common_crash_points::one_crash_point_over_every_write(
        file!(),
        "raise_rollback_floor_pushed_by_the_session",
        recorded.writes.len(),
    );
    common_crash_points::amplify_prepared_flow(
        "floor-raise-pushed-by-the-session",
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.last_root_write,
        &recorded.versions,
        crash_points,
        6,
    );
    drop(recorded.pool);
}
