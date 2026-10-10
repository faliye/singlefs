//! b29 正推腿 L2 探针（在 L3 探针的枚举上，对抽到的态与每段边界的整段持久镜像跑池级 checker，数新判法的判定分布）
//! 原 L3 探针说明：b29 正推腿 L3 探针（harness 档，只读冻结树，探针放在草稿副本里跑）：
//! 在随机历史与命名比重组的录制流上，把「mkfs 之后」每一段按「前面的段整段持久、本段每个写 落 / 不落（系统配置槽的写另有撕裂一态）」全枚举，
//! 数每个崩溃态里每块盘自证过的系统配置槽有几个；再在抽出的第一次崩溃镜像上起一次可写挂载（录制），对挂载那几段同样枚举（二次崩溃）。
//! 看有没有「某块盘一份自证过的系统配置都没有」的态（G4 的前提：没有读故障的历史留不下它）。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use std::collections::BTreeMap;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn torn(old: &[u8], new: &[u8]) -> Vec<u8> {
    let differs = |i: &usize| old[*i] != new[*i];
    let Some(first) = (0..new.len()).find(differs) else { return new.to_vec() };
    let last = (0..new.len()).rev().find(differs).unwrap();
    let span = last - first + 1;
    let first_old = first + span / 2;
    let mut t = new.to_vec();
    t[first_old..].copy_from_slice(&old[first_old..]);
    t
}

fn valid_slots(pool: &MemoryPool, device: DeviceIdentity, spacing: u64, fsid: &[u8; 16]) -> usize {
    let bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).unwrap();
    (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        .filter(|slot| {
            let b = pool.devices[&device].read(DeviceOffsetInBytes(slot * spacing), bytes);
            SystemConfiguration::parse_slot(&b)
                .map(|c| c.immutable.filesystem_identifier == *fsid)
                .unwrap_or(false)
        })
        .count()
}

struct Tally {
    states: u64,
    zero_slot_states: u64,
    one_slot_states: u64,
    examples: Vec<String>,
}

/// 枚举 `base` 之上 `writes` 按 `segments` 切的每个崩溃态（前面的段整段落、本段每个写 落 / 不落，系统配置槽的写加撕裂一态）。
/// 段内态数超过 `cap` 时按种子抽 `cap` 个。`visit` 每态调一次，交回这一态的镜像、段号、每个写的取值串。
fn enumerate(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    cap: u64,
    seed: u64,
    mut visit: impl FnMut(&MemoryPool, usize, &str),
) {
    let mut prefix = base.clone();
    let mut rng = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    for (segment_index, segment) in segments.iter().enumerate() {
        let radixes: Vec<u64> = segment
            .iter()
            .map(|w| if writes[*w].kind == StepKind::SystemConfigurationSlot { 3 } else { 2 })
            .collect();
        let total: u128 = radixes.iter().map(|r| u128::from(*r)).product();
        let states: Vec<Vec<u64>> = if total <= u128::from(cap) {
            (0..u64::try_from(total).unwrap())
                .map(|mut n| {
                    radixes
                        .iter()
                        .map(|r| {
                            let d = n % r;
                            n /= r;
                            d
                        })
                        .collect()
                })
                .collect()
        } else {
            (0..cap)
                .map(|_| {
                    radixes
                        .iter()
                        .map(|r| {
                            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                            (rng >> 33) % r
                        })
                        .collect()
                })
                .collect()
        };
        for digits in states {
            let mut image = prefix.clone();
            let mut text = String::new();
            for (w, d) in segment.iter().zip(&digits) {
                text.push(char::from(b'0' + u8::try_from(*d).unwrap()));
                let write = &writes[*w];
                match d {
                    0 => {}
                    1 => write.contents.apply_to(image.devices.get_mut(&write.device).unwrap(), write.offset),
                    _ => {
                        let len = usize::try_from(write.length_in_bytes()).unwrap();
                        let dev = image.devices.get_mut(&write.device).unwrap();
                        let old = dev.read(write.offset, len);
                        let mut new = vec![0u8; len];
                        write.contents.copy_range_into(0, &mut new);
                        dev.write(write.offset, &torn(&old, &new));
                    }
                }
            }
            visit(&image, segment_index, &text);
        }
        for w in segment {
            let write = &writes[*w];
            write.contents.apply_to(prefix.devices.get_mut(&write.device).unwrap(), write.offset);
        }
    }
}

fn geometry() -> FixedGeometry {
    let p = SMALL.parameters();
    FixedGeometry {
        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: p.geometry.journal_ring_bytes,
        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
    }
}


fn class_of(v: &InvariantVerdict) -> &'static str {
    match v {
        InvariantVerdict::Holds => "Holds",
        InvariantVerdict::Violated(_) => "Violated",
        InvariantVerdict::NotApplicable(_) => "NotApplicable",
        InvariantVerdict::PartiallyJudged(_) => "PartiallyJudged",
    }
}

const WATCHED: [&str; 5] = ["I-9.16", "I-5.4", "I-2.4", "I-5.5", "I-1.1"];

#[test]
fn b29_l2_checker_sweep() {
    let seeds = env_u64("PROBE_SEEDS", 3);
    let start = env_u64("PROBE_SEED_START", 0);
    let ops = usize::try_from(env_u64("PROBE_OPS", 6)).unwrap();
    let cap = env_u64("PROBE_STATE_CAP", 400);
    let every = env_u64("PROBE_CHECK_EVERY", 200);
    let spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let _ = spacing;
    let mut dist: BTreeMap<(String, &'static str, &'static str), u64> = BTreeMap::new(); // (不变量, 判定类, 来源)
    let mut violated_names: BTreeMap<String, u64> = BTreeMap::new();
    let mut examples: Vec<String> = Vec::new();
    let mut checks = 0u64;
    let mut histories = 0u64;
    let mut kinds: BTreeMap<String, u64> = BTreeMap::new();
    for weights in GenerationWeights::NAMED.iter() {
        for seed in start..start + seeds {
            let history = generate_history_with_weights(HistorySeed(seed), ops, weights);
            let read_fault_history = history.operations.iter().any(|o| matches!(o.kind(),
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
                | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros));
            let stream = SharedStream::retaining_contents();
            let run = execute_history_with(
                &history,
                HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: SMALL, space_admission: SpaceAdmission::JudgedByTheFormula },
                &stream,
                &mut |_| {},
            );
            let retained = stream.retained_operations();
            let mkfs = run.operations_written_by_make_filesystem;
            let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], SMALL.device_bytes());
            base.apply(&retained[..mkfs]);
            let (writes, segments, _idx) = writes_and_segments_with_stream_indexes(&retained[mkfs..], &geometry());
            histories += 1;
            for o in &history.operations { *kinds.entry(format!("{:?}", o.kind())).or_insert(0) += 1; }
            let ops_text: Vec<String> = history.operations.iter().map(|o| format!("{:?}", o.kind())).collect();
            let mut record = |pool: &MemoryPool, source: &'static str, what: String| {
                let verdicts = check_pool_image(pool);
                checks += 1;
                for (name, v) in &verdicts {
                    if WATCHED.contains(name) {
                        *dist.entry(((*name).to_string(), class_of(v), source)).or_insert(0) += 1;
                    }
                    if let InvariantVerdict::Violated(text) = v {
                        *violated_names.entry(format!("{name}@{source}@rf={read_fault_history}")).or_insert(0) += 1;
                        if examples.len() < 12 && WATCHED.contains(name) {
                            examples.push(format!("{name} {source} W={} seed={seed} ops={ops_text:?} {what}: {}", weights.name, text.chars().take(200).collect::<String>()));
                        }
                    }
                    if let InvariantVerdict::PartiallyJudged(text) = v {
                        if *name == "I-5.4" && examples.len() < 12 {
                            examples.push(format!("I-5.4 PartiallyJudged {source} W={} seed={seed} {what}: {}", weights.name, text.chars().take(160).collect::<String>()));
                        }
                    }
                }
            };
            // 每段边界（前面的段整段落）
            let mut prefix = base.clone();
            record(&prefix, "boundary", "after mkfs".to_string());
            for (si, seg) in segments.iter().enumerate() {
                for w in seg {
                    let write = &writes[*w];
                    write.contents.apply_to(prefix.devices.get_mut(&write.device).unwrap(), write.offset);
                }
                if every > 0 && si % 3 == 0 { record(&prefix, "boundary", format!("seg {si} done")); }
            }
            let mut counter = 0u64;
            enumerate(&base, &writes, &segments, cap, seed, |image, seg, text| {
                counter += 1;
                if every > 0 && counter % every == 0 { record(image, "crash_state", format!("seg={seg} digits={text}")); }
            });
        }
    }
    println!("PROBE histories={histories} checks={checks}");
    for (k, n) in &kinds { println!("PROBE_KIND {k} {n}"); }
    for ((name, class, source), n) in &dist { println!("PROBE {name} {class} {source} {n}"); }
    for (k, n) in &violated_names { println!("PROBE_VIOLATED {k} {n}"); }
    for e in &examples { println!("PROBE_EXAMPLE {e}"); }
}
