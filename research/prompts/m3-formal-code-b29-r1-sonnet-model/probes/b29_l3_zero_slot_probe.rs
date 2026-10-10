//! b29 正推腿 L3 探针（harness 档，只读冻结树，探针放在草稿副本里跑）：
//! 在随机历史与命名比重组的录制流上，把「mkfs 之后」每一段按「前面的段整段持久、本段每个写 落 / 不落（系统配置槽的写另有撕裂一态）」全枚举，
//! 数每个崩溃态里每块盘自证过的系统配置槽有几个；再在抽出的第一次崩溃镜像上起一次可写挂载（录制），对挂载那几段同样枚举（二次崩溃）。
//! 看有没有「某块盘一份自证过的系统配置都没有」的态（G4 的前提：没有读故障的历史留不下它）。
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

#[test]
fn b29_l3_zero_slot_probe() {
    let seeds = env_u64("PROBE_SEEDS", 6);
    let ops = usize::try_from(env_u64("PROBE_OPS", 4)).unwrap();
    let cap = env_u64("PROBE_STATE_CAP", 3000);
    let second_level_per_history = env_u64("PROBE_SECOND_LEVEL", 6);
    let named = std::env::var("PROBE_WEIGHTS").ok();
    let spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let mut first = [Tally { states: 0, zero_slot_states: 0, one_slot_states: 0, examples: vec![] },
                     Tally { states: 0, zero_slot_states: 0, one_slot_states: 0, examples: vec![] }];
    let mut second = [Tally { states: 0, zero_slot_states: 0, one_slot_states: 0, examples: vec![] },
                      Tally { states: 0, zero_slot_states: 0, one_slot_states: 0, examples: vec![] }];
    let mut histories = 0u64;
    let mut refused_second_mounts = 0u64;
    for weights in GenerationWeights::NAMED.iter() {
        if let Some(n) = &named { if !weights.name.contains(n.as_str()) { continue; } }
        for seed in env_u64("PROBE_SEED_START", 0)..env_u64("PROBE_SEED_START", 0) + seeds {
            let history = generate_history_with_weights(HistorySeed(seed), ops, weights);
            let read_fault_history = history.operations.iter().any(|o| matches!(o.kind(),
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
                | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros));
            let class = usize::from(read_fault_history);
            let stream = SharedStream::retaining_contents();
            let run = execute_history_with(
                &history,
                HistoryExecution {
                    per_step_checker: PerStepChecker::Skipped,
                    device_width: SMALL,
                    space_admission: SpaceAdmission::JudgedByTheFormula,
                },
                &stream,
                &mut |_| {},
            );
            let retained = stream.retained_operations();
            let mkfs = run.operations_written_by_make_filesystem;
            let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], SMALL.device_bytes());
            base.apply(&retained[..mkfs]);
            let fsid = {
                let b = base.devices[&DeviceIdentity(0)].read(DeviceOffsetInBytes(0), 4096);
                SystemConfiguration::parse_slot(&b).expect("mkfs 之后盘 0 槽 0 自证过").immutable.filesystem_identifier
            };
            let (writes, segments, _idx) = writes_and_segments_with_stream_indexes(&retained[mkfs..], &geometry());
            histories += 1;
            let mut sample: Vec<MemoryPool> = Vec::new();
            let mut counter = 0u64;
            enumerate(&base, &writes, &segments, cap, seed, |image, seg, text| {
                let t = &mut first[class];
                t.states += 1;
                let counts: Vec<usize> = [DeviceIdentity(0), DeviceIdentity(1)].iter().map(|d| valid_slots(image, *d, spacing, &fsid)).collect();
                let min = *counts.iter().min().unwrap();
                if min == 0 {
                    t.zero_slot_states += 1;
                    if t.examples.len() < 5 { t.examples.push(format!("W={} seed={seed} ops={:?} seg={seg} digits={text} counts={counts:?}", weights.name, history.operations.iter().map(|o| format!("{:?}", o.kind())).collect::<Vec<_>>())); }
                } else if min == 1 { t.one_slot_states += 1; }
                counter += 1;
                if second_level_per_history > 0 && counter % 37 == u64::from(seed) % 37 && u64::try_from(sample.len()).unwrap() < second_level_per_history {
                    sample.push(image.clone());
                }
            });
            // 第二层：抽出的第一次崩溃镜像上起一次可写挂载（录制），对挂载那几段枚举
            for crash_image in sample {
                let p = SMALL.parameters();
                let mstream = SharedStream::retaining_contents();
                let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = crash_image
                    .devices
                    .iter()
                    .map(|(id, sectors)| {
                        let mut d = SparseBlockDevice::new(crash_image.device_size_in_bytes, PhysicalBlockSizeInBytes(p.geometry.physical_block_size));
                        d.image = sectors.clone();
                        (*id, RecordingBlockDevice::with_shared_stream(*id, d, mstream.clone()))
                    })
                    .collect();
                let result = mount_writable_with_space_admission(&p, &mut devices, SpaceAdmission::JudgedByTheFormula);
                if result.is_err() { refused_second_mounts += 1; continue; }
                drop(result);
                let ops: Vec<RetainedOperation> = mstream.retained_operations();
                let (mwrites, msegments, _) = writes_and_segments_with_stream_indexes(&ops, &geometry());
                enumerate(&crash_image, &mwrites, &msegments, cap, seed + 77, |image, seg, text| {
                    let t = &mut second[class];
                    t.states += 1;
                    let counts: Vec<usize> = [DeviceIdentity(0), DeviceIdentity(1)].iter().map(|d| valid_slots(image, *d, spacing, &fsid)).collect();
                    let min = *counts.iter().min().unwrap();
                    if min == 0 {
                        t.zero_slot_states += 1;
                        if t.examples.len() < 5 { t.examples.push(format!("W={} seed={seed} seg={seg} digits={text} counts={counts:?}", weights.name)); }
                    } else if min == 1 { t.one_slot_states += 1; }
                });
            }
        }
    }
    println!("PROBE histories={histories} second_level_mounts_refused={refused_second_mounts}");
    for (name, tallies) in [("first_cut", &first), ("second_cut", &second)] {
        for (class, label) in [(0usize, "no_read_fault_history"), (1usize, "read_fault_history")] {
            let t = &tallies[class];
            println!("PROBE {name} {label} states={} disks_with_zero_valid_slots_states={} min_one_valid_slot_states={}", t.states, t.zero_slot_states, t.one_slot_states);
            for e in &t.examples { println!("PROBE_EXAMPLE {name} {label} {e}"); }
        }
    }
}
