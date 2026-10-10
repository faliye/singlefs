//! m3-c694-r2 辩方腿的 K4 探针（harness 档，不碰 checker 档的测试二进制）：
//! 在冻结副本的补丁版上，拿随机历史（与崩溃注入快档同一个种子基、同一组比重 BROAD、同样每段 24 步）摆崩溃状态，
//! 每个崩溃状态上数四件事：只读恢复落没落到「走读失败」、可写挂载走没走到影子账的拒那一支与账读不出那一支、
//! C 的挂钩（水位之上没施加的记录点名的落点）有没有东西可隔离，以及四臂（今天 / C / 废根放行 W1 / 读不出一律放行 W2）
//! 在这个状态上的挂载结局与挂载之后池级 checker 的红名是否相同。
//! 环境变量：K4_SEEDS（几段历史，缺省 24）、K4_OPS（每段几步，缺省 24）、K4_WIDTH（4g | 384 | 256，缺省 4g）、K4_ARMS（缺省 none,c,w1,w2）。
//! 崩溃状态取每个段的每个前缀（段内前缀，read_faulted 那条用例同一种摆法），不是快档的 4 个抽样点。

use std::sync::atomic::Ordering;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    candidate_switches, mount_writable_with_space_admission, probe_counters, MountSpaceAdmission,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::SharedStream;

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn env_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn set_arm(arm: &str) {
    candidate_switches::W1.store(arm == "w1", Ordering::SeqCst);
    candidate_switches::W2.store(arm == "w2", Ordering::SeqCst);
    candidate_switches::W4.store(arm == "c", Ordering::SeqCst);
}

/// 一个状态上一臂的结局：可写挂载的结局（做成时带隔离槽数、实例、有效根 txg）+ 挂载之后池级 checker 的红名。
fn signature(image: &MemoryPool, width: HistoryDeviceWidth, arm: &str) -> (String, [u64; 9]) {
    set_arm(arm);
    let parameters = width.parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    let before = probe_counters::snapshot();
    let result = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let after = probe_counters::snapshot();
    let mut delta = [0u64; 9];
    for index in 0..9 {
        delta[index] = after[index] - before[index];
    }
    let text = match result {
        Ok(mounted) => {
            let isolated: u64 = mounted
                .output
                .isolated_slots_per_device
                .iter()
                .map(|(_, slots)| *slots)
                .sum();
            let after_image = MemoryPool {
                devices: devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.image.clone()))
                    .collect(),
                device_size_in_bytes: image.device_size_in_bytes,
            };
            let reds: Vec<&str> = if env_number("K4_CHECK", 1) == 0 {
                Vec::new()
            } else {
                check_pool_image(&after_image)
                    .iter()
                    .filter_map(|(name, verdict)| match verdict {
                        InvariantVerdict::Violated(_) => Some(*name),
                        _ => None,
                    })
                    .collect()
            };
            let admission = match &mounted.output.space_admission {
                MountSpaceAdmission::AdmittedBeforeAcquisition => "before".to_string(),
                MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => {
                    format!("after_raises{}", floor_raises.len())
                }
                MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
                    format!("still_short_raises{}", floor_raises.len())
                }
                MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => "not_judged".to_string(),
            };
            format!(
                "Ok eff=({},{}) isolated={isolated} unreadable={} admission={admission} warmups={} reds={reds:?}",
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
                mounted.output.abandoned_roots_unreadable,
                mounted.output.warm_up_publishes.len(),
            )
        }
        Err(error) => {
            let text = format!("{error:?}");
            let kind: String = text
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            format!("Err {kind}")
        }
    };
    set_arm("none");
    (text, delta)
}

#[test]
#[ignore = "辩方腿的探针：按 rerun.sh 起"]
fn k4_random_histories_reach() {
    let seeds = env_number("K4_SEEDS", 24);
    let operations = usize::try_from(env_number("K4_OPS", 24)).expect("步数");
    let width = match std::env::var("K4_WIDTH").unwrap_or_default().as_str() {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    };
    let arms_text = std::env::var("K4_ARMS").unwrap_or_else(|_| "none,c,w1,w2".to_string());
    let arms: Vec<&str> = arms_text.split(',').collect();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut totals = vec![[0u64; 9]; arms.len()];
    let mut states_total = 0u64;
    let mut recover_failed_total = 0u64;
    let mut recover_no_root_total = 0u64;
    let mut differing_total = vec![0u64; arms.len()];
    let mut err_total = vec![0u64; arms.len()];
    let mut red_total = vec![0u64; arms.len()];
    let first = env_number("K4_FIRST", 0);
    let mut kinds: Vec<std::collections::BTreeMap<String, u64>> =
        vec![std::collections::BTreeMap::new(); arms.len()];
    for seed_offset in first..first + seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset;
        let started = std::time::Instant::now();
        let history =
            generate_history_with_weights(HistorySeed(seed), operations, &GenerationWeights::BROAD);
        let stream = SharedStream::retaining_contents();
        execute_history_with(&history, execution, &stream, &mut |_| {});
        let (writes, segments, _indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &width.fixed_geometry(),
        );
        let mut base = MemoryPool::with_devices(&DEVICES, width.device_bytes());
        let mut applied = 0usize;
        let mut states = 0u64;
        let mut recover_failed = 0u64;
        let mut recover_no_root = 0u64;
        let mut history_counters = vec![[0u64; 9]; arms.len()];
        let mut differing = vec![0u64; arms.len()];
        let last_segment = segments.len().saturating_sub(1);
        for (segment_index, segment) in segments.iter().enumerate() {
            let first_write = segment[0];
            base.apply_writes(&writes[applied..first_write]);
            applied = first_write;
            let upper = if segment_index == last_segment {
                segment.len() + 1
            } else {
                segment.len()
            };
            for persisted_writes in 0..upper {
                let mut image = base.clone();
                image.apply_writes(&writes[first_write..first_write + persisted_writes]);
                states += 1;
                match recover(&image, JournalPolicy::Consult).outcome {
                    RecoveryOutcome::Failed { root: Some(_), .. } => recover_failed += 1,
                    RecoveryOutcome::Failed { root: None, .. } => recover_no_root += 1,
                    _ => {}
                }
                let mut first_text = String::new();
                for (arm_index, arm) in arms.iter().enumerate() {
                    let (text, delta) = signature(&image, width, arm);
                    for index in 0..9 {
                        history_counters[arm_index][index] += delta[index];
                    }
                    if arm_index == 0 {
                        first_text = text.clone();
                    } else if text != first_text {
                        differing[arm_index] += 1;
                        println!(
                            "K4DIFF seed_offset={seed_offset} segment={segment_index} persisted_writes={persisted_writes} arm={arm} base_arm={} base={first_text} | arm_result={text}",
                            arms[0]
                        );
                    }
                    if text.starts_with("Err") {
                        err_total[arm_index] += 1;
                        *kinds[arm_index].entry(text.clone()).or_insert(0) += 1;
                    }
                    if text.contains("reds=[") && !text.contains("reds=[]") {
                        red_total[arm_index] += 1;
                    }
                }
            }
        }
        states_total += states;
        recover_failed_total += recover_failed;
        recover_no_root_total += recover_no_root;
        for arm_index in 0..arms.len() {
            for index in 0..9 {
                totals[arm_index][index] += history_counters[arm_index][index];
            }
            differing_total[arm_index] += differing[arm_index];
        }
        println!(
            "K4HIST width={width:?} seed_offset={seed_offset} segments={} writes={} states={states} recover_failed_on_a_chosen_root={recover_failed} recover_no_root_chosen={recover_no_root} seconds={:.1}",
            segments.len(),
            writes.len(),
            started.elapsed().as_secs_f64()
        );
    }
    println!("K4TOTAL width={width:?} seeds={seeds} ops={operations} states={states_total} recover_failed_on_a_chosen_root={recover_failed_total} recover_no_root_chosen={recover_no_root_total}");
    for (arm_index, arm) in arms.iter().enumerate() {
        for (kind, count) in &kinds[arm_index] {
            println!("K4ERR arm={arm} {kind} count={count}");
        }
        let counters: Vec<String> = probe_counters::NAMES
            .iter()
            .zip(totals[arm_index].iter())
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        println!(
            "K4ARM arm={arm} mount_err={} mount_ok_with_red_checker={} states_differing_from_{}={} {}",
            err_total[arm_index],
            red_total[arm_index],
            arms[0],
            differing_total[arm_index],
            counters.join(" ")
        );
    }
}
