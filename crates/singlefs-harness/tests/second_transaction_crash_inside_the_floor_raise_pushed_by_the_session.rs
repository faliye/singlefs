//! 里程碑「第二个事务」收尾实现批次实五：崩在准入推的那一串空发布中间（安全设计轮第三轮攻方 `research/prompts/m2-safety-r3-opus-output.md`
//! 第六节那一形；D16（发布语义） 已定项 1「准入」那一行，C283（准入失败时不先推发布就报 ENOSPC））。取一条历史、只录会话推的那一串抬 F，
//! 按层 0 的枚举域做小枚举（状态数照层 0 的口径另算：原地覆写取三态，每段 3^m · 2^(n−m) − 1 再加全部持久那一个，控制在 10⁶ 以内），
//! 恢复之后与挂载之后两份镜像的池级 checker 都判；挂载那一遍照枚举用的写表叠，撕裂那一态的镜像也交给挂载。
//! 名字里不带 layer0：它不是层 0 的全量流，是一条历史上的一小段，平时跑得起。

mod common_admission;

use common_admission::{
    checker_violations_on, content_of, plain_devices_on, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_space_admission, push_one_floor_raise_within_the_admission_budget,
    FloorRaisePushedWithinTheAdmissionBudget,
};
use singlefs_core::transaction::{FirstFile, PoolVersion, PoolWriter, PublishError};
use singlefs_harness::crash::{
    enumerate_layer0_in_state_slices, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, writes_and_segments, Layer0Parallelism,
    MemoryPool, RetainedWrite,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::layer0_progress::Layer0Resume;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 崩在准入推的那一串空发布中间（攻方「崩在准入推空发布的那一串中间」那一形）：会话里数据单元落点取不到的那一刻（单元区 384 槽的小盘，
/// 崩了再挂之后第 62 次覆盖写，见 `second_transaction_admission_raises_the_floor_before_refusing.rs` 的 `an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session`；
/// ckpt_cost 按最坏情况计之后第 18、32、47 次是式子先拒、会话推过再发成，落点那一道第一次走到是第 62 次），
/// 只录会话推的那一串抬 F（先把新 F 写进每块盘的系统配置、那几次带新 F 的空发布），按层 0 的枚举域（每一段都展开）枚举这条流的每个崩溃状态：
/// 恢复之后 oracle 与池级 checker 0 违例（枚举器自己判）；再可写挂载（空间准入判着），挂载之后的镜像池级 checker 0 违例；
/// 挂载全做成。状态数照层 0 的口径另算（系统配置槽写是原地覆写、取三态，[`layer0_state_count_with_torn_in_place_overwrites`]），
/// 落在 10⁶ 以内才枚举；挂载那一遍的镜像照观察者交来的枚举用写表叠（录制流里的写在前，撕裂镜像与重放接在后面），不只叠录制流那几次写。
#[test]
#[ignore = "崩溃枚举：提交时由崩溃验证员按输入哈希跑（release），平时不跑"]
fn crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green(
) {
    const STATES_AT_MOST: u64 = 1_000_000;
    let stream = SharedStream::retaining_contents();
    let stream_for_the_devices = stream.clone();
    let mut pool = PoolUnderTest::start_after_the_first_file(
        HistoryDeviceWidth::UnitAreaOf384Slots,
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
    // 第 62 次：直接调发布路径，数据单元落点被拒（发布路径算定形状时在分配器的拷贝上就取不到，在准入与任何写之前返回）；再照会话被拒之后那一步推一串抬 F
    // （`push_one_floor_raise_within_the_admission_budget`，会话调的就是它），只录这一串。
    let mut session = pool.session.take().expect("这次挂载的会话");
    pool.write_time_seconds += 1;
    let content = content_of(OVERWRITE_BYTES, pool.write_time_seconds);
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("挂载之后现行那一版带文件");
    };
    let refused = {
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
    assert!(
        matches!(refused, Err(PublishError::PlacementRefused { .. })),
        "第 62 次：数据单元落点被拒：{:?}",
        refused.as_ref().map(|version| version.root.checkpoint_txg)
    );
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
    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
    let (writes_of_the_raise, segments_of_the_raise) = writes_and_segments(&recorded, &geometry);
    let root_writes: Vec<usize> = writes_of_the_raise
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
    let last_root_write_of_the_raise = *root_writes.last().expect("那一串至少一次空发布");
    let states = layer0_state_count_with_torn_in_place_overwrites(
        &base,
        &writes_of_the_raise,
        &segments_of_the_raise,
        &full_expansion,
    );
    assert!(
        states > 0 && states <= STATES_AT_MOST,
        "推的那一串的崩溃状态数 {states} 在 10⁶ 以内"
    );
    // 推过之后同一次覆盖写照会话那样重发，做成（这一格的结局与会话里那一次相同）。
    pool.overwrite(OVERWRITE_BYTES)
        .expect("推过抬 F 之后覆盖写做成");
    let versions = pool.published_versions();
    // 线程数与层 0 同一个来源（环境变量 `SINGLEFS_LAYER0_THREADS`，`research/scripts/capped.sh` 设它），挂载那一遍也按它分片。
    let parallelism = Layer0Parallelism::from_environment();
    // 观察者交来的 `image.writes` 是枚举用的写表（`CrashImage` 的文档：录制流里的写在前，原地覆写的撕裂镜像与重放接在后面），
    // `image.persisted` 与它逐条对应、比录制流长。整趟枚举只有这一张表：头一次收下，之后每个状态核它一样长。
    let mut enumerated_writes: Option<Vec<RetainedWrite>> = None;
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    let mut collect =
        |image: &singlefs_harness::crash::CrashImage<'_>,
         _report: &singlefs_core::recovery::RecoveryReport,
         _counts: &mut singlefs_harness::crash::Layer0ObserverCounts| {
            let table = enumerated_writes.get_or_insert_with(|| image.writes.to_vec());
            assert_eq!(
                table.len(),
                image.writes.len(),
                "每个状态交来的都是这一趟枚举用的那一张写表"
            );
            persisted_sets.push(image.persisted.clone());
        };
    // 不留进度文件：观察者把持久集合收进自己的变量，续跑接不上。
    let tally = enumerate_layer0_in_state_slices(
        &base,
        &writes_of_the_raise,
        &segments_of_the_raise,
        last_root_write_of_the_raise,
        &versions,
        &full_expansion,
        parallelism,
        Some(&mut collect),
        &Layer0Resume::NoProgressFile,
    );
    assert_eq!(
        tally.states, states,
        "每一段都展开，状态数与按层 0 口径另算的相同（原地覆写三态、其余两态）"
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
    assert_eq!(u64::try_from(persisted_sets.len()).expect("状态数"), states);
    let enumerated_writes = enumerated_writes.expect("观察者至少看过一个状态");
    assert!(
        enumerated_writes.len() > writes_of_the_raise.len(),
        "那一串先把新 F 写进每块盘的系统配置（原地覆写），撕裂镜像接在录制流那 {} 次写后面：枚举用的写表 {} 次",
        writes_of_the_raise.len(),
        enumerated_writes.len()
    );
    // 每个崩溃状态可写挂载一次、挂载之后的镜像跑池级 checker：按线程分片并行（观察者在调用线程上，逐个挂载太慢）。
    let worker_threads = parallelism.worker_threads.get();
    let chunk_length = persisted_sets.len().div_ceil(worker_threads);
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = persisted_sets
            .chunks(chunk_length.max(1))
            .map(|chunk| {
                let base = &base;
                let writes = &enumerated_writes;
                let parameters = &pool.parameters;
                scope.spawn(move || {
                    let mut failures = Vec::new();
                    for persisted in chunk {
                        let mut image = base.clone();
                        for (write, is_persisted) in writes.iter().zip(persisted) {
                            if *is_persisted {
                                write.contents.apply_to(
                                    image
                                        .devices
                                        .get_mut(&write.device)
                                        .expect("写落在池里的盘上"),
                                    write.offset,
                                );
                            }
                        }
                        let mut devices = plain_devices_on(&image);
                        match mount_writable_with_space_admission(
                            parameters,
                            &mut devices,
                            SpaceAdmission::JudgedByTheFormula,
                        ) {
                            Ok(_) => {
                                let mounted_image = MemoryPool {
                                    devices: devices
                                        .iter()
                                        .map(|(identity, device)| (*identity, device.image.clone()))
                                        .collect(),
                                    device_size_in_bytes: image.device_size_in_bytes,
                                };
                                let violations = checker_violations_on(&mounted_image);
                                if !violations.is_empty() {
                                    failures.push(format!("挂载之后 checker 判红：{violations:?}"));
                                }
                            }
                            Err(error) => failures.push(format!("挂载失败：{error:?}")),
                        }
                    }
                    failures
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("挂载那一片的线程不 panic"))
            .collect()
    });
    assert_eq!(
        failures.first(),
        None,
        "每个崩溃状态都挂得上、挂载之后池级 checker 0 违例（{} 个状态里 {} 个不成）",
        states,
        failures.len()
    );
}
