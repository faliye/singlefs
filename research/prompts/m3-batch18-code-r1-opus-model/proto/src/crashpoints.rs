//! 手摆的崩溃点（不是层 0 的枚举域）：一段删除历史的录制流从意图那次发布的第一步起，每个操作前缀各当一个崩溃镜像（前缀里的写全持久、
//! 之后的一个都没落），先对崩溃镜像跑池级 checker，再可写挂载（续做走空），对挂载之后的镜像再跑一遍。
//! 层 0 的枚举域（段内任意整写子集）这几段历史每段 ≥ 2^20，按原型规则不跑，见报告。
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

use crate::common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use crate::judge::judge;
use crate::pool::{content_of, SparsePool};
use crate::sweep::{devices_of, image_of, switches};

#[derive(Clone, Debug)]
struct CrashHistory {
    sequential_write_bytes: Option<usize>,
    new_inodes_before: u64,
    targets: Vec<u64>,
    batch_units: u64,
    /// 意图之后正常卸载、可写挂载续做（续做那几次发布也在录制流里，崩溃点落在续做中途）。
    continuation_in_a_mount: bool,
}

fn crash_histories() -> Vec<CrashHistory> {
    let payload = data_unit_payload_capacity();
    let mut all = Vec::new();
    for shape in [None, Some(0), Some(payload), Some(2 * payload + payload / 2), Some(3 * payload), Some(5 * payload + 1)] {
        for (new_inodes_before, targets) in [(0u64, vec![1u64]), (2, vec![2, 1])] {
            for batch_units in [1u64, 2, 144] {
                for continuation_in_a_mount in [false, true] {
                    all.push(CrashHistory { sequential_write_bytes: shape, new_inodes_before, targets: targets.clone(), batch_units, continuation_in_a_mount });
                }
            }
        }
    }
    all
}

/// 跑历史，交回 (mkfs 之后的基线, 意图之前录制流的长度, 整条录制流)。
fn record(history: &CrashHistory) -> (MemoryPool, usize, Vec<singlefs_harness::RetainedOperation>) {
    let mut pool = SparsePool::build(4 << 30, &file_content());
    if let Some(bytes) = history.sequential_write_bytes {
        pool.sequential_write(&content_of(bytes), FIXED_WRITE_TIME_SECONDS + 60);
    }
    if history.new_inodes_before > 0 {
        pool.new_inodes(history.new_inodes_before, FIXED_WRITE_TIME_SECONDS + 120);
    }
    let before_intent = pool.stream.operation_count();
    if history.continuation_in_a_mount {
        let first = history.targets[0];
        let time = FIXED_WRITE_TIME_SECONDS + 200;
        pool.intent(first, time).expect("意图");
        let mut current = singlefs_core::transaction::PoolVersion::of_a_version_of_trees(pool.output.clone());
        singlefs_core::mount::unmount(&parameters(), &mut pool.devices, &mut pool.allocator, &mut current, ShadowLedger::On).expect("卸载");
        let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(), &mut pool.devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches(history.batch_units),
        )
        .expect("挂载续做");
        let (_output, mut session) = singlefs_core::mounted_session::MountedSession::of_the_mount(mounted);
        for target in &history.targets[1..] {
            session
                .publish_user_change(&mut pool.devices, singlefs_core::mounted_session::UserChange::Unlink { inode: singlefs_core::address::InodeNumber(*target), write_time_seconds: time })
                .expect("会话 unlink");
        }
    } else {
        for target in &history.targets {
            pool.intent(*target, FIXED_WRITE_TIME_SECONDS + 200).expect("意图");
            pool.steps_until_empty(history.batch_units);
        }
    }
    let operations = pool.stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
    base.apply(&operations[..pool.mkfs_operation_count]);
    (base, before_intent, operations)
}

/// 把一段历史的前缀按序号模 `parts` 分成几份分批跑（每批挂钟压在一次前台命令以内），`M3B18_PART=第几份/几份`，没设是整份。
fn part_of_the_prefixes() -> (usize, usize) {
    std::env::var("M3B18_PART")
        .ok()
        .and_then(|value| {
            let (part, parts) = value.split_once('/')?;
            Some((part.parse().ok()?, parts.parse().ok()?))
        })
        .unwrap_or((0, 1))
}

pub fn run(threads: usize, from: usize, to: usize) {
    let all = crash_histories();
    let histories: Vec<CrashHistory> = all[from.min(all.len())..to.min(all.len())].to_vec();
    println!("CRASH_BATCH from={from} to={} of={} part={:?}", to.min(all.len()), all.len(), part_of_the_prefixes());
    let started = std::time::Instant::now();
    let next = AtomicUsize::new(0);
    let lines: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::SeqCst);
                let Some(history) = histories.get(index) else { break };
                let (base, before_intent, operations) = record(history);
                let mkfs = {
                    let mut probe = SparsePool::build(4 << 30, &file_content());
                    let count = probe.mkfs_operation_count;
                    let _ = &mut probe;
                    count
                };
                let mut prefixes = 0u64;
                let mut crash_red = Vec::new();
                let mut mount_refused = 0u64;
                let mut after_mount_red = Vec::new();
                let mut crash_partial = 0u64;
                let (part, parts) = part_of_the_prefixes();
                for prefix in (before_intent..=operations.len()).filter(|prefix| prefix % parts == part) {
                    if prefix > before_intent && matches!(operations[prefix - 1].operation.kind, singlefs_harness::RecordedOperationKind::Barrier) {
                        continue;
                    }
                    prefixes += 1;
                    let mut image = base.clone();
                    image.apply(&operations[mkfs..prefix]);
                    let judged = judge(&image);
                    for watched in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
                        if judged.verdict_of(watched) == 'N' && judged.detail_of(watched).contains("读不") {
                            crash_partial += 1;
                        }
                    }
                    if !judged.violated.is_empty() {
                        crash_red.push(format!("prefix={prefix} {:?}", judged.violated_names()));
                    }
                    let stream = SharedStream::new();
                    let mut devices = devices_of(&image, &stream);
                    match mount_writable_with_test_only_switches_and_deletion_switches(
                        &parameters(), &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches(history.batch_units),
                    ) {
                        Err(_refused) => mount_refused += 1,
                        Ok(_mounted) => {
                            let after = judge(&image_of(&devices, 4 << 30));
                            if !after.violated.is_empty() {
                                after_mount_red.push(format!("prefix={prefix} {:?} {}", after.violated_names(), after.violated[0].1));
                            }
                        }
                    }
                }
                let line = format!(
                    "CRASH_HISTORY index={} history={history:?} prefixes={prefixes} crash_image_red={} mount_refused={mount_refused} after_mount_red={} deletion_scan_unreadable_verdicts={crash_partial}{}{}",
                    index + from,
                    crash_red.len(),
                    after_mount_red.len(),
                    crash_red.first().map_or(String::new(), |first| format!(" first_crash_red={first}")),
                    after_mount_red.first().map_or(String::new(), |first| format!(" first_after_mount_red={first}")),
                );
                lines.lock().expect("锁").push((index, line));
            });
        }
    });
    let mut lines = lines.into_inner().expect("锁");
    lines.sort();
    for (_, line) in &lines {
        println!("{line}");
    }
    let prefixes: u64 = lines.iter().filter_map(|(_, line)| line.split(" prefixes=").nth(1)?.split(' ').next()?.parse::<u64>().ok()).sum();
    println!("CRASH_SUMMARY histories={} prefixes={prefixes} threads={threads} elapsed_seconds={:.1}", lines.len(), started.elapsed().as_secs_f64());
}
