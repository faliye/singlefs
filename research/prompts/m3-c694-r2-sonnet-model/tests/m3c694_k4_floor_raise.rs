//! m3-c694-r2 辩方腿的 K4 探针之二（harness 档）：登记的崩溃枚举用例 floor-raise-pushed-by-the-session
//! （`crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs`）在 harness 档的缩小复刻。
//! 同一条历史（384 槽小盘、第一个文件之后崩了再挂、会话覆盖写 61 次、直接发布到准入先拒、推一串抬 F、只录推的那一串），
//! 崩溃状态取每个段的每个前缀（不枚举段内子集，不枚举系统配置槽的撕裂态），每个状态上可写挂载一次（空间准入判着），
//! 四臂（今天 / C / W1 / W2）各挂一次，比挂载结局、隔离槽数、空间准入的推抬 F 串数与挂载之后池级 checker 的红名。
//! 环境变量：K4_ARMS（缺省 none,c,w1,w2）。

#[allow(dead_code)]
mod common_admission_pool;

use std::sync::atomic::Ordering;

use common_admission_pool::{
    checker_violations_on, content_of, device_bytes_of_the_384_slot_small_devices,
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    candidate_switches, mount_writable_with_space_admission,
    push_one_floor_raise_within_the_admission_budget, probe_counters,
    FloorRaisePushedWithinTheAdmissionBudget, MountSpaceAdmission,
};
use singlefs_core::transaction::{FirstFile, PoolVersion, PoolWriter, PublishError};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn set_arm(arm: &str) {
    candidate_switches::W1.store(arm == "w1", Ordering::SeqCst);
    candidate_switches::W2.store(arm == "w2", Ordering::SeqCst);
    candidate_switches::W4.store(arm == "c", Ordering::SeqCst);
}

#[test]
#[ignore = "辩方腿的探针：按 rerun.sh 起"]
fn k4_the_floor_raise_pushed_by_the_session_remounted_under_each_arm() {
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
    let refused_at = refused_at.expect("200 次直接发布之内准入总要先拒一次");
    println!("K4FR admission_refused_at_overwrite={refused_at}");
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
    let recorded = stream.retained_operations()[operations_before_the_raise..].to_vec();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let (writes, segments) = writes_and_segments(&recorded, &geometry);
    println!(
        "K4FR raise_publishes={} writes={} segments={} record_writes={}",
        raised.publishes.len(),
        writes.len(),
        segments.len(),
        writes
            .iter()
            .filter(|write| write.kind == StepKind::JournalRecord)
            .count()
    );
    let arms_text = std::env::var("K4_ARMS").unwrap_or_else(|_| "none,c,w1,w2".to_string());
    let arms: Vec<&str> = arms_text.split(',').collect();
    let parameters = parameters_of_the_384_slot_small_devices();
    let mut running_base = base.clone();
    let mut applied = 0usize;
    let mut states = 0u64;
    let mut differing = vec![0u64; arms.len()];
    let mut errs = vec![0u64; arms.len()];
    let mut reds = vec![0u64; arms.len()];
    let mut hook_with_records = 0u64;
    let mut hook_placements = 0u64;
    let last_segment = segments.len() - 1;
    for (segment_index, segment) in segments.iter().enumerate() {
        let first_write = segment[0];
        running_base.apply_writes(&writes[applied..first_write]);
        applied = first_write;
        let upper = if segment_index == last_segment {
            segment.len() + 1
        } else {
            segment.len()
        };
        for persisted_writes in 0..upper {
            let mut image: MemoryPool = running_base.clone();
            let landed: Vec<RetainedWrite> = writes[first_write..first_write + persisted_writes].to_vec();
            image.apply_writes(&landed);
            states += 1;
            let mut first_text = String::new();
            for (arm_index, arm) in arms.iter().enumerate() {
                set_arm(arm);
                let mut devices = plain_devices_on(&image);
                let before = probe_counters::snapshot();
                let result = mount_writable_with_space_admission(
                    &parameters,
                    &mut devices,
                    SpaceAdmission::JudgedByTheFormula,
                );
                let after = probe_counters::snapshot();
                if arm_index == 0 {
                    // 挂钩计数在每一臂都数（挂钩总是算、只有 C 臂才隔离），取第一臂的
                    hook_with_records += after[6] - before[6];
                    hook_placements += after[8] - before[8];
                }
                let text = match result {
                    Ok(mounted) => {
                        let isolated: u64 = mounted
                            .output
                            .isolated_slots_per_device
                            .iter()
                            .map(|(_, slots)| *slots)
                            .sum();
                        let admission = match &mounted.output.space_admission {
                            MountSpaceAdmission::AdmittedBeforeAcquisition => "before".to_string(),
                            MountSpaceAdmission::AdmittedAfterTheFloorRaises {
                                floor_raises, ..
                            } => format!("after_raises{}", floor_raises.len()),
                            MountSpaceAdmission::StillShortAfterTheFloorRaises {
                                floor_raises, ..
                            } => format!("still_short_raises{}", floor_raises.len()),
                            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
                                "not_judged".to_string()
                            }
                        };
                        let mounted_image = MemoryPool {
                            devices: devices
                                .iter()
                                .map(|(identity, device)| (*identity, device.image.clone()))
                                .collect(),
                            device_size_in_bytes: image.device_size_in_bytes,
                        };
                        let violations = checker_violations_on(&mounted_image);
                        if !violations.is_empty() {
                            reds[arm_index] += 1;
                        }
                        format!(
                            "Ok eff=({},{}) isolated={isolated} admission={admission} reds={violations:?}",
                            mounted.output.effective_root.instance.0,
                            mounted.output.effective_root.checkpoint_txg.0
                        )
                    }
                    Err(error) => {
                        errs[arm_index] += 1;
                        let text = format!("{error:?}");
                        let kind: String = text
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        format!("Err {kind}")
                    }
                };
                if arm_index == 0 {
                    first_text = text;
                } else if text != first_text {
                    differing[arm_index] += 1;
                    println!(
                        "K4FRDIFF segment={segment_index} persisted_writes={persisted_writes} arm={arm} base={first_text} | arm_result={text}"
                    );
                }
            }
        }
    }
    set_arm("none");
    println!("K4FRTOTAL states={states} hook_mounts_with_unapplied_records_under_{}={hook_with_records} hook_placements={hook_placements}", arms[0]);
    for (arm_index, arm) in arms.iter().enumerate() {
        println!(
            "K4FRARM arm={arm} mount_err={} mount_ok_with_red_checker={} states_differing_from_{}={}",
            errs[arm_index], reds[arm_index], arms[0], differing[arm_index]
        );
    }
}
