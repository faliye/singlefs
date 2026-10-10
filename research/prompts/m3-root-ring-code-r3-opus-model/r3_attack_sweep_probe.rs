//! 攻方腿原型（m3-root-ring-code-r3，K1）：不入库。
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, devices_of, parameters_of_the_ring_hole_pools, pool_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table_sweep::{
    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
    SweepPublishPlacements,
};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable_with_instance_table_sweep_configuration, unmount, ShadowLedger};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

pub fn any_row() -> InstanceTableSweepConfiguration {
    InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    }
}

pub fn violated(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{}", detail.chars().take(160).collect::<String>())),
            _ => None,
        })
        .collect()
}

pub fn partially(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::PartiallyJudged(detail) => Some(format!("{invariant}:{}", detail.chars().take(260).collect::<String>())),
            _ => None,
        })
        .collect()
}

pub fn outcome_name(sweep: &InstanceTableSweepAtTheEndOfTheMount) -> String {
    let full = format!("{sweep:?}");
    match sweep {
        InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted { rows_deleted, heads_erased }) => {
            format!("RowsDeleted(rows={:?},heads={})", rows_deleted.iter().map(|r| r.instance.0).collect::<Vec<_>>(), heads_erased.len())
        }
        _ => full.chars().take(220).collect(),
    }
}

fn fresh() -> RingHoleDevices {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    devices
}

/// 一段历史：每次挂载覆盖写 ks[i] 次，ends[i] 为 true 时正常卸载、为 false 时不卸载（崩在最后一次发布之后）。
fn run_history(ks: &[u64], ends: &[bool], label: &str) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = fresh();
    let mut deleted_any = false;
    for (index, (&k, &end)) in ks.iter().zip(ends).enumerate() {
        let mounted = mount_writable_with_instance_table_sweep_configuration(
            &parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, any_row(),
        )
        .expect("可写挂载");
        let instance = mounted.output.instance;
        let sweep_text = outcome_name(&mounted.output.instance_table_sweep);
        if sweep_text.starts_with("RowsDeleted") {
            deleted_any = true;
        }
        let mut allocator = mounted.allocator;
        let mut current = mounted.current;
        let rows = instance_table_chain_of_root(&pool_of(&devices), current.root()).map(|c| c.records.rows.len()).unwrap_or(9999);
        let image = pool_of(&devices);
        let red = violated(&image);
        println!(
            "H {label} mount={index} inst={} txg={} F={} rows_after={rows} sweep={sweep_text} red={red:?}",
            instance.0,
            current.root().checkpoint_txg.0,
            current.root().rollback_floor.0
        );
        if matches!(current, PoolVersion::WithoutFile(_)) {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let out = publish_first_file(
                &mut writer, &mut allocator, current.root(),
                FirstFile { content: &content_of_the_version(0), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS },
                instance, current.record_bytes(),
            )
            .expect("第一个文件");
            current = PoolVersion::of_a_version_of_trees(out);
        }
        for _ in 0..k {
            let previous = current.into_file_version().expect("带文件");
            let salt = previous.root.checkpoint_txg.0 + 1;
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            let out = publish_overwrite(
                &mut writer, &mut allocator, &previous,
                FirstFile { content: &content_of_the_version(salt), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt },
                instance,
            )
            .expect("覆盖写");
            current = PoolVersion::of_a_version_of_trees(out);
        }
        if end {
            unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On).expect("卸载");
        }
    }
    println!("SUMMARY {label} deleted_any={deleted_any}");
}

#[test]
fn k1_unmount_cycles() {
    let scale: usize = std::env::var("PROBE_CYCLES").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
    for k in 0..=4u64 {
        for pattern in ["U", "C", "UC"] {
            let ends: Vec<bool> = (0..scale)
                .map(|i| match pattern { "U" => true, "C" => false, _ => i % 2 == 0 })
                .collect();
            let ks = vec![k; scale];
            run_history(&ks, &ends, &format!("k{k}-{pattern}"));
        }
    }
}

/// 从挂载之前的盘面起一次可写挂载（清扫有一行就扫），之后覆盖写 k 次、end 时卸载；交回清扫结局与之后的盘面。
fn one_mount(pre: &MemoryPool, k: u64, end: bool) -> (String, MemoryPool) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(pre);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, any_row(),
    )
    .expect("可写挂载");
    let instance = mounted.output.instance;
    let sweep_text = outcome_name(&mounted.output.instance_table_sweep);
    let mut allocator = mounted.allocator;
    let mut current = mounted.current;
    if matches!(current, PoolVersion::WithoutFile(_)) {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let out = publish_first_file(
            &mut writer, &mut allocator, current.root(),
            FirstFile { content: &content_of_the_version(0), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS },
            instance, current.record_bytes(),
        )
        .expect("第一个文件");
        current = PoolVersion::of_a_version_of_trees(out);
    }
    for _ in 0..k {
        let previous = current.into_file_version().expect("带文件");
        let salt = previous.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let out = publish_overwrite(
            &mut writer, &mut allocator, &previous,
            FirstFile { content: &content_of_the_version(salt), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt },
            instance,
        )
        .expect("覆盖写");
        current = PoolVersion::of_a_version_of_trees(out);
    }
    if end {
        unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On).expect("卸载");
    }
    (sweep_text, pool_of(&devices))
}

thread_local! {
    static DEEPEST: std::cell::RefCell<(Vec<(u64, bool)>, Vec<String>)> = std::cell::RefCell::new((Vec::new(), Vec::new()));
}

const CHOICES: [(u64, bool); 12] = [
    (0, true), (0, false), (1, true), (1, false), (2, true), (2, false),
    (3, true), (3, false), (4, true), (4, false), (5, true), (5, false),
];

/// 深搜：pre 是一次挂载之前的盘面，它那次挂载的清扫已知没删行；选这次挂载的 (k, end)，要下一次挂载的清扫也不删行。
fn dfs(pre: &MemoryPool, depth: usize, goal: usize, path: &mut Vec<(u64, bool)>, outs: &mut Vec<String>, budget: &mut u64) -> bool {
    if depth == goal {
        return true;
    }
    for &(k, end) in &CHOICES {
        if *budget < 2 {
            return false;
        }
        *budget -= 2;
        let (_sweep_of_this_mount, post) = one_mount(pre, k, end);
        let (sweep_of_the_next, _) = one_mount(&post, 0, false);
        if sweep_of_the_next.starts_with("RowsDeleted") {
            continue;
        }
        path.push((k, end));
        outs.push(sweep_of_the_next);
        if path.len() > DEEPEST.with(|d| d.borrow().0.len()) {
            DEEPEST.with(|d| *d.borrow_mut() = (path.clone(), outs.clone()));
        }
        if dfs(&post, depth + 1, goal, path, outs, budget) {
            return true;
        }
        path.pop();
        outs.pop();
    }
    false
}

#[test]
fn k1_adversarial_schedule_keeps_every_sweep_from_deleting() {
    let goal: usize = std::env::var("PROBE_GOAL").ok().and_then(|s| s.parse().ok()).unwrap_or(16);
    let warm: usize = std::env::var("PROBE_WARM").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    // 前缀：mkfs 之后 warm 次「覆盖写 1 次、卸载」的挂载（这几次清扫的结局不管）。
    let mut pre = pool_of(&fresh());
    for _ in 0..warm {
        pre = one_mount(&pre, 1, true).1;
    }
    let mut path = Vec::new();
    let mut outs = Vec::new();
    let mut budget: u64 = std::env::var("PROBE_BUDGET").ok().and_then(|s| s.parse().ok()).unwrap_or(4000);
    let found = dfs(&pre, 0, goal, &mut path, &mut outs, &mut budget);
    println!("ADV warm={warm} goal={goal} found={found} budget_left={budget} path={path:?}");
    let (deep_path, deep_outs) = DEEPEST.with(|d| d.borrow().clone());
    println!("ADV-DEEPEST len={} path={deep_path:?}", deep_path.len());
    for (index, out) in deep_outs.iter().enumerate() {
        println!("ADV-DEEP {index} {out}");
    }
    for (index, out) in outs.iter().enumerate() {
        println!("ADV-STEP {index} {out}");
    }
}

/// 照固定调度长跑：第一次挂载 (0, 卸载)，之后 (1, 卸载)、(1, 不卸载) 交替（深搜找到的那条），threshold 可调。
fn one_mount_with(pre: &MemoryPool, k: u64, end: bool, configuration: InstanceTableSweepConfiguration) -> (String, MemoryPool, usize, u32) {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(pre);
    let mounted = mount_writable_with_instance_table_sweep_configuration(
        &parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, configuration,
    )
    .expect("可写挂载");
    let instance = mounted.output.instance;
    let sweep_text = outcome_name(&mounted.output.instance_table_sweep);
    let mut allocator = mounted.allocator;
    let mut current = mounted.current;
    let rows = instance_table_chain_of_root(&pool_of(&devices), current.root()).map(|c| c.records.rows.len()).unwrap_or(9999);
    if matches!(current, PoolVersion::WithoutFile(_)) {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let out = publish_first_file(
            &mut writer, &mut allocator, current.root(),
            FirstFile { content: &content_of_the_version(0), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS },
            instance, current.record_bytes(),
        )
        .expect("第一个文件");
        current = PoolVersion::of_a_version_of_trees(out);
    }
    for _ in 0..k {
        let previous = current.into_file_version().expect("带文件");
        let salt = previous.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let out = publish_overwrite(
            &mut writer, &mut allocator, &previous,
            FirstFile { content: &content_of_the_version(salt), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt },
            instance,
        )
        .expect("覆盖写");
        current = PoolVersion::of_a_version_of_trees(out);
    }
    if end {
        unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On).expect("卸载");
    }
    (sweep_text, pool_of(&devices), rows, instance.0)
}

#[test]
fn k1_periodic_schedule_long_run() {
    let mounts: usize = std::env::var("PROBE_MOUNTS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);
    let threshold: u64 = std::env::var("PROBE_THRESHOLD").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let check_every: usize = std::env::var("PROBE_CHECK_EVERY").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
    let schedule = std::env::var("PROBE_SCHEDULE").unwrap_or_else(|_| "1U,1C".to_string());
    let steps: Vec<(u64, bool)> = schedule.split(',').map(|s| (s[..s.len()-1].parse().unwrap(), s.ends_with('U'))).collect();
    let configuration = InstanceTableSweepConfiguration {
        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
        row_count_threshold: InstanceTableRowCountThreshold(threshold),
        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
    };
    let mut pre = pool_of(&fresh());
    let mut counts: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut red_total = 0usize;
    let mut last_rows = 0;
    for index in 0..mounts {
        let (k, end) = if index == 0 { (0, true) } else { steps[(index - 1) % steps.len()] };
        let (sweep, post, rows, instance) = one_mount_with(&pre, k, end, configuration);
        let name: String = sweep.split(|c: char| c == '{' || c == '(').take(3).collect::<Vec<_>>().join("(");
        *counts.entry(name.clone()).or_default() += 1;
        last_rows = rows;
        if index % check_every == 0 || index + 1 == mounts {
            let red = violated(&post);
            red_total += red.len();
            let partial = partially(&post);
            println!("LONG mount={index} inst={instance} rows_after_sweep={rows} sweep={} red={red:?} partial={partial:?}", sweep.chars().take(200).collect::<String>());
        }
        pre = post;
    }
    println!("LONG-SUMMARY schedule={schedule} mounts={mounts} threshold={threshold} last_rows={last_rows} red_total={red_total} counts={counts:?}");
}

/// 扫全部周期 1 与周期 2 的调度（每步 k ∈ 0..=5、卸载或不卸载），各跑 mounts 次挂载，记清扫结局分布与最后行数。
#[test]
fn k1_scan_periodic_schedules() {
    let mounts: usize = std::env::var("PROBE_MOUNTS").ok().and_then(|s| s.parse().ok()).unwrap_or(40);
    let threads: usize = std::env::var("PROBE_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(8);
    let mut schedules: Vec<Vec<(u64, bool)>> = Vec::new();
    for a in 0..12 {
        schedules.push(vec![CHOICES[a]]);
        for b in (a + 1)..12 {
            schedules.push(vec![CHOICES[a], CHOICES[b]]);
        }
    }
    let total = schedules.len();
    let chunks: Vec<Vec<Vec<(u64, bool)>>> = (0..threads).map(|t| schedules.iter().skip(t).step_by(threads).cloned().collect()).collect();
    let handles: Vec<_> = chunks.into_iter().map(|chunk| std::thread::spawn(move || {
        let mut lines = Vec::new();
        for steps in chunk {
            let mut pre = pool_of(&fresh());
            let (mut deleted, mut blocked_account, mut no_candidate, mut other, mut red_total) = (0u64, 0u64, 0u64, 0u64, 0usize);
            let mut last_rows = 0;
            let mut longest_blocked_streak = 0u64;
            let mut streak = 0u64;
            for index in 0..mounts {
                let (k, end) = if index == 0 { (0, true) } else { steps[(index - 1) % steps.len()] };
                let (sweep, post, rows, _instance) = one_mount_with(&pre, k, end, any_row());
                last_rows = rows;
                if sweep.starts_with("RowsDeleted") { deleted += 1; streak = 0; }
                else if sweep.contains("AccountOfARingRootUnreadable") { blocked_account += 1; streak += 1; longest_blocked_streak = longest_blocked_streak.max(streak); }
                else if sweep.contains("NoCandidateRow") { no_candidate += 1; }
                else { other += 1; }
                if index + 1 == mounts { red_total += violated(&post).len(); }
                pre = post;
            }
            lines.push(format!("SCAN schedule={steps:?} mounts={mounts} deleted={deleted} blocked_by_account={blocked_account} longest_blocked_streak={longest_blocked_streak} no_candidate={no_candidate} other={other} last_rows={last_rows} red_at_end={red_total}"));
        }
        lines
    })).collect();
    let mut all: Vec<String> = handles.into_iter().flat_map(|h| h.join().expect("线程")).collect();
    all.sort();
    for line in &all { println!("{line}"); }
    println!("SCAN-TOTAL schedules={total} lines={}", all.len());
}
