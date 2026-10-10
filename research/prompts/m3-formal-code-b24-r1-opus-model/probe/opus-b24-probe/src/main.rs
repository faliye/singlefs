//! 攻方副本探针入口（m3-formal-code-b24-r1 opus）。子命令：f3-tests、real、harm、b1-sweep。
//! f3-tests、harm 用的种子函数与四条用例由 tools/gen_seedfn.py 从 crash_injection.rs 原样抽进 seedfn.rs（可先施加一行变异），
//! seedfn_pristine.rs 是不施加变异抽的那一份。
#![allow(clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
#[allow(dead_code, unused_imports)]
mod common_admission;
mod seedfn;
mod seedfn_pristine;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
};
use singlefs_checker_tier::crash_injection::opus_b24_probe;
use singlefs_checker_tier::crash_injection::{
    mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::{
    HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
use std::collections::{BTreeMap, BTreeSet};

// ---------------- F3 ----------------

fn short_test_name(name: &str) -> &'static str {
    if name.starts_with("all_first_crash_states_of_a_segment_of_twelve") {
        "F3c"
    } else if name.contains("zero_to_sixty_three") {
        "A"
    } else if name.contains("twenty_eight") {
        "B"
    } else if name.contains("history_seeds_zero_and_one") {
        "C"
    } else {
        "?"
    }
}

/// 四条抽出的用例逐条跑，panic 记红；红时从 assert_eq 的消息里取 left（不同种子的个数）。
fn f3_tests() -> String {
    std::panic::set_hook(Box::new(|_| {}));
    let mut cells = Vec::new();
    for (name, test) in seedfn::TESTS {
        let verdict = match std::panic::catch_unwind(*test) {
            Ok(()) => "green".to_string(),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                let left = message
                    .split("left: ")
                    .nth(1)
                    .map(|rest| rest.split_whitespace().next().unwrap_or("").to_string())
                    .unwrap_or_else(|| "?".to_string());
                format!("red(left={left})")
            }
        };
        cells.push(format!("{}={verdict}", short_test_name(name)));
    }
    let _ = std::panic::take_hook();
    format!("f3-tests label={} | {} | extract_sha256={}", seedfn::LABEL, cells.join(" "), seedfn::EXTRACT_SHA256)
}

/// 攻方改法 F3-e（只在这个模型上量过、被攻过零轮）的三条，都只调种子函数：
/// A2 历史种子 0、段号 0..=63 再加 2^k 与 2^k + 1（k = 6..=20）、长 12 的段 4095 个子集，种子两两不同；
/// B3 历史种子 0、段号 16、长 32 的段上扣下 1 个或 2 个写的全部 528 个子集，种子两两不同（丢掉段内任何一个位置都让 {i, j} 与 {j} 撞）；
/// C2 段号 16、长 12 的 4095 个子集在历史种子 0、1、2^63 上各算一遍，12285 个两两不同。
fn f3_proposed() -> String {
    let mask_bits = |mask: u64, length: usize| -> Vec<bool> { (0..length).map(|k| (mask >> k) & 1 == 1).collect() };
    let mut segments: Vec<usize> = (0..=63).collect();
    for k in 6..=20 {
        segments.push(1 << k);
        segments.push((1 << k) + 1);
    }
    let a2: Vec<u64> = segments.iter().flat_map(|s| (0..4095u64).map(move |m| (*s, m))).map(|(s, m)| seedfn::seed_of(0, s, &mask_bits(m, 12))).collect();
    let mut b3 = Vec::new();
    for i in 0..32 {
        let mut p = vec![true; 32];
        p[i] = false;
        b3.push(seedfn::seed_of(0, 16, &p));
        for j in (i + 1)..32 {
            let mut q = p.clone();
            q[j] = false;
            b3.push(seedfn::seed_of(0, 16, &q));
        }
    }
    let c2: Vec<u64> = [0u64, 1, 1 << 63].iter().flat_map(|seed| (0..4095u64).map(move |m| (*seed, m))).map(|(seed, m)| seedfn::seed_of(seed, 16, &mask_bits(m, 12))).collect();
    let cell = |name: &str, v: &[u64]| {
        let d = v.iter().copied().collect::<BTreeSet<u64>>().len();
        format!("{name}={}({d}/{})", if d == v.len() { "green" } else { "red" }, v.len())
    };
    format!("f3-proposed label={} | {} {} {}", seedfn::LABEL, cell("A2", &a2), cell("B3", &b3), cell("C2", &c2))
}

/// 读 real 的输出（P 行），按历史分组：每段历史里第一次崩溃态撞种子的个数（态数 − 不同种子数），与全部历史合起来的不同种子数。
/// 一段历史 = (第几份输入, 历史种子)：战役各档同一个种子基起，同一个种子在不同档里是步数或比重不同的另一段历史，不并成一段。
fn harm(files: &[String]) -> String {
    let mut by_history: BTreeMap<(usize, u64), Vec<(usize, Vec<bool>)>> = BTreeMap::new();
    for (file_index, file) in files.iter().enumerate() {
        for line in std::fs::read_to_string(file).expect("读 dump").lines() {
            let mut parts = line.split_whitespace();
            if parts.next() != Some("P") {
                continue;
            }
            let seed: u64 = parts.next().unwrap().parse().unwrap();
            let segment: usize = parts.next().unwrap().parse().unwrap();
            let bits: Vec<bool> = parts.next().unwrap().chars().map(|c| c == '1').collect();
            by_history.entry((file_index, seed)).or_default().push((segment, bits));
        }
    }
    let states: usize = by_history.values().map(Vec::len).sum();
    let mut out = Vec::new();
    for (tag, f) in [("pristine", seedfn_pristine::seed_of as fn(u64, usize, &[bool]) -> u64), ("this", seedfn::seed_of)] {
        let mut sharing = 0usize;
        let mut histories_with_sharing = 0usize;
        let mut all = BTreeSet::new();
        for ((_file_index, seed), list) in &by_history {
            let distinct: BTreeSet<u64> = list.iter().map(|(g, p)| f(*seed, *g, p)).collect();
            sharing += list.len() - distinct.len();
            histories_with_sharing += usize::from(list.len() != distinct.len());
            all.extend(distinct);
        }
        out.push(format!(
            "{tag}: states_sharing_a_seed_within_a_history={sharing} histories_with_sharing={histories_with_sharing} distinct_over_all_histories={}/{states}",
            all.len()
        ));
    }
    format!("harm label={} histories={} states={states} | {}", seedfn::LABEL, by_history.len(), out.join(" | "))
}

// ---------------- B1 ----------------

/// 与入库 B1 用例 `crash_image_between_the_deletion_batches` 同一造法，五个数可调（覆盖写长度入库是 OVERWRITE_BYTES = 2999）。
fn crash_image(slots: u64, overwrites: usize, overwrite_bytes: usize, units: usize, batches: usize) -> Result<MemoryPool, String> {
    let built = std::panic::catch_unwind(|| {
        let mut pool =
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                slots,
            );
        for _ in 0..overwrites {
            pool.overwrite(overwrite_bytes).map_err(|e| format!("overwrite {e:?}"))?;
        }
        pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100)
            .map_err(|e| format!("sequential {e:?}"))?;
        let parameters = pool.parameters.clone();
        let write_time_seconds = pool.write_time_seconds;
        let session = pool.session.as_mut().ok_or("no session")?;
        let mut current = session.current.version_of_trees().ok_or("no trees")?.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), write_time_seconds, session.instance)
            .map_err(|e| format!("intent {e:?}"))?;
        for _ in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
                UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).unwrap()), session.instance)
                .map_err(|e| format!("batch {e:?}"))?;
        }
        drop(writer);
        pool.session = None;
        Ok::<MemoryPool, String>(pool.image())
    });
    match built {
        Ok(result) => result,
        Err(_) => Err("panic while building".to_string()),
    }
}

fn execution() -> HistoryExecution {
    HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::UnitAreaOf384Slots, space_admission: SpaceAdmission::JudgedByTheFormula }
}

/// 入库 B1 用例今天的前提块第二块（24-1 改法 B1-c）原样：同一份盘面另挂一次，发一次建 0 个 inode 的空发布。
fn precondition_publish_creating_no_inode(image: &MemoryPool) -> Result<(), String> {
    let execution = execution();
    let parameters = execution.device_width.parameters();
    let mut devices = plain_devices_on(image);
    let mut mounted = mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission)
        .map_err(|e| format!("mount:{}", format!("{e:?}").chars().take(30).collect::<String>()))?;
    let PoolVersion::WithFile(current) = &mounted.current else { return Err("not-with-file".to_string()) };
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_new_inodes(&mut writer, &mut mounted.allocator, current, 0, FIXED_WRITE_TIME_SECONDS, mounted.output.instance)
        .map(|_| ())
        .map_err(|e| format!("{:?}/{}", refusal_reason_of_publish_error(&e), format!("{e:?}").split(|c: char| c == '(' || c == ' ' || c == '{').next().unwrap_or("")))
}

/// 入库 B1 用例在这份盘面上判什么（`with_b1c` 决定带不带 24-1 那条新前提）；`env` 是开关（判法只看 Some、那一支建 0 个）。
fn b1_test_verdict(image: &MemoryPool, env: &[(&str, &str)], with_b1c: bool) -> String {
    for (k, v) in env { std::env::set_var(k, v); }
    let execution = execution();
    let parameters = execution.device_width.parameters();
    let verdict = (|| {
        let mut devices = plain_devices_on(image);
        let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission) {
            Ok(m) => m,
            Err(_) => return "red(pre:mount)".to_string(),
        };
        if !matches!(mounted.output.deletion_queue_continuation_outcome, DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }) {
            return "red(pre:continuation)".to_string();
        }
        let PoolVersion::WithFile(current) = &mounted.current else { return "red(pre:with-file)".to_string() };
        if current.inode_record.map(|r| r.nlink) != Some(NLINK_OF_AN_INODE_BEING_DELETED) {
            return "red(pre:nlink)".to_string();
        }
        if with_b1c {
            if let Err(e) = precondition_publish_creating_no_inode(image) {
                return format!("red(pre-b1c:{e})");
            }
        }
        let run = mount_and_publish_once_on_the_crash_image(image, &parameters, execution, HistorySeed(0),
            &CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None });
        match &run.outcome {
            WritableMountOutcome::PublishRefused { reason, .. } if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) => "green".to_string(),
            WritableMountOutcome::PublishRefused { reason, .. } => format!("red(reason:{reason:?})"),
            other => format!("red(outcome:{})", format!("{other:?}").chars().take(20).collect::<String>()),
        }
    })();
    for (k, _) in env { std::env::remove_var(k); }
    verdict
}

fn b1(slots: u64, overwrites: usize, overwrite_bytes: usize, units: usize, batches: usize) -> String {
    let tag = format!("b1 slots={slots} ow={overwrites} owbytes={overwrite_bytes} units={units} batches={batches}");
    let image = match crash_image(slots, overwrites, overwrite_bytes, units, batches) {
        Ok(image) => image,
        Err(e) => return format!("{tag} class=BUILD-FAILED build={}", e.chars().take(40).collect::<String>()),
    };
    let zero = [("OPUS_ARM", "count-zero")];
    let some = [("OPUS_K2B_SOME", "1")];
    let old_correct = b1_test_verdict(&image, &[], false);
    if old_correct.starts_with("red(pre:") {
        // 用例原有的前提先红（与开关无关：开关只动发布臂与判法），别的五次判不再跑
        return format!("{tag} class=RED-ON-CORRECT-BEFORE-B1C old(correct)=({old_correct})");
    }
    let old_zero = b1_test_verdict(&image, &zero, false);
    let old_some = b1_test_verdict(&image, &some, false);
    let now_correct = b1_test_verdict(&image, &[], true);
    let now_zero = b1_test_verdict(&image, &zero, true);
    let now_some = b1_test_verdict(&image, &some, true);
    let pre = match precondition_publish_creating_no_inode(&image) { Ok(()) => "Published".to_string(), Err(e) => e };
    let class = if now_correct == "green" {
        match (now_zero.starts_with("red"), now_some.starts_with("red")) {
            (true, true) => "NOW-GREEN-KILLS-BOTH",
            (false, true) => "NOW-GREEN-ZERO-SURVIVES",
            (true, false) => "NOW-GREEN-SOME-SURVIVES",
            (false, false) => "NOW-GREEN-KILLS-NONE",
        }
    } else if now_correct.starts_with("red(pre-b1c") && old_correct == "green" {
        if old_zero.starts_with("red") { "NOW-RED-AT-B1C-OLD-KILLED-ZERO" } else { "NOW-RED-AT-B1C-OLD-ZERO-SURVIVED" }
    } else {
        "RED-ON-CORRECT-BEFORE-B1C"
    };
    format!("{tag} class={class} b1c_publish0={pre} old(correct,zero,some)=({old_correct},{old_zero},{old_some}) now(correct,zero,some)=({now_correct},{now_zero},{now_some})")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = |i: usize| args[i].parse::<u64>().unwrap();
    let list = |i: usize| args[i].split(',').map(|x| x.parse::<u64>().unwrap()).collect::<Vec<u64>>();
    match args.get(1).map(String::as_str) {
        Some("f3-tests") => println!("{}", f3_tests()),
        Some("f3-proposed") => println!("{}", f3_proposed()),
        Some("real") => {
            // real <first_seed> <count> <operations> <weights> <width> <points>
            for offset in 0..n(3) {
                print!("{}", opus_b24_probe::real_history(n(2).wrapping_add(offset), n(4) as usize, &args[5], &args[6], n(7) as usize));
            }
        }
        Some("harm") => println!("{}", harm(&args[2..])),
        Some("b1-sweep") => {
            // b1-sweep <slots-from> <slots-to> <ow,..> <owbytes,..> <units,..> <batches,..>
            let parameters = execution().device_width.parameters();
            eprintln!("parameters.geometry.physical_block_size={}", parameters.geometry.physical_block_size);
            for ow in list(4) { for owb in list(5) { for u in list(6) { for b in list(7) {
                for slots in n(2)..=n(3) {
                    println!("{}", b1(slots, ow as usize, owb as usize, u as usize, b as usize));
                }
            }}}}
        }
        _ => eprintln!("用法：f3-tests | real <起种子> <段数> <步数> <broad|rollback> <384|4g> <每段几个，0 是稠密> | harm <dump…> | b1-sweep <槽起> <槽止> <覆盖写次数,…> <覆盖写字节,…> <单元,…> <批,…>"),
    }
}
