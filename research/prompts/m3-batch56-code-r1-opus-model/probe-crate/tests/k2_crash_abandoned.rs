//! K2-1（m3-batch56-code-r1 opus）：崩溃恢复抛弃段里写的容器。三棵容器树装旋钮（一片 OPUS_KNOB 条），先在内存盘上走一段前缀
//! （三棵树各写出几片），再录一个窗口（几次会往三棵树追加、开新片的发布）；窗口按层 0 的枚举域全量展开（`enumerate_layer0_*`，
//! 段内整写子集逐个、原地覆写三态），每个崩溃状态：① 崩溃镜像交池级 checker 带旋钮的新入口；② 按同一组旋钮可写挂载；
//! ③ 挂载之后接一段用户动作（建、删最大的活号、随便删一个、挂着回退到前缀里某一版，次序与参数按状态序号播种，放开不写死），
//! 每一步之后再交带旋钮的新入口。报各不变量违例的状态数、挂载拒绝、动作 panic；抛弃了段（恢复所取的根早于最新持久的根、
//! 或有写持久而它那次发布的根没持久）的状态单独数。
mod common_b56;

use common_b56::{geometry, first_file_content, violations, Knobs, Pool};
use singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs;
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_harness::history::SeededRandomSource;
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, some_publish_persisted_without_its_root,
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn env_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// 前缀：建 234 个（叶 [233, 2]），删两个（墓碑两片、退役一片），再建一个、删掉（墓碑、退役各多一片），回退两次（谱系两片）。
/// 交回池与前缀里发布过的每一版（实例 1）的 txg。
fn prefix(knobs: Knobs) -> (Pool<SparseBlockDevice>, Vec<u64>) {
    let mut pool = Pool::after_the_first_file();
    let mut txgs = vec![pool.txg()];
    pool.allocator.set_deletion_switches(knobs.switches());
    pool.create(234);
    txgs.push(pool.txg());
    let after_create = pool.txg();
    pool.unlink(235, knobs);
    pool.unlink(234, knobs);
    txgs.push(pool.txg());
    pool.create(1);
    pool.unlink(236, knobs);
    txgs.push(pool.txg());
    pool.roll_back((InstanceGeneration(1), after_create)).expect("前缀回退一");
    txgs.push(pool.txg());
    pool.roll_back((InstanceGeneration(1), after_create)).expect("前缀回退二");
    txgs.push(pool.txg());
    (pool, txgs)
}

/// 窗口：OPUS_WINDOW 点名的几次发布，在录制盘上做。
fn window<D: singlefs_core::block_device::BlockDevice>(pool: &mut Pool<D>, name: &str, knobs: Knobs, rollback_target: u64) {
    for step in name.split('+') {
        match step {
            "unlink-largest" => {
                let largest = *pool.live_inodes().iter().max().expect("有活着的号");
                pool.unlink(largest, knobs);
            }
            "create" => pool.create(1),
            "rollback" => pool.roll_back((InstanceGeneration(1), rollback_target)).expect("窗口回退"),
            other => panic!("不认识的窗口步 {other}"),
        }
    }
}

/// 一个崩溃状态的镜像：基线加持久了的写（写表是枚举器交来的那一份，撕裂镜像接在录制的写之后）。
fn crash_image(base: &MemoryPool, table: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut image = base.clone();
    for (write, is_persisted) in table.iter().zip(persisted) {
        if *is_persisted {
            let device = image.devices.get_mut(&write.device).expect("盘");
            write.contents.apply_to(device, write.offset);
        }
    }
    image
}

#[derive(Default)]
struct StateFindings {
    lines: Vec<String>,
    violated: BTreeMap<String, u64>,
    checks: u64,
    mounts_refused: u64,
    panics: u64,
    steps: BTreeMap<String, u64>,
}

fn judge(image: &MemoryPool, knobs: Knobs, at: &str, ordinal: usize, findings: &mut StateFindings) {
    findings.checks += 1;
    let found = violations(&check_pool_image_given_the_container_capacity_knobs(image, knobs.checker_knobs()));
    for line in &found {
        let invariant = line.split(':').next().unwrap_or("").to_string();
        *findings.violated.entry(invariant).or_default() += 1;
    }
    if !found.is_empty() && findings.lines.len() < 40 {
        findings.lines.push(format!("OPUS_K2C_VIOLATION state={ordinal} at={at} {:?}", found));
    }
}

/// 挂载之后接一段用户动作（按状态序号播种）：每一步之后交带旋钮的新入口。
fn continue_after_the_mount(image: &MemoryPool, knobs: Knobs, prefix_txgs: &[u64], ordinal: usize, steps: u64, findings: &mut StateFindings) {
    let mut pool = match Pool::mounted_from(image, knobs) {
        Ok(pool) => pool,
        Err(refusal) => {
            findings.mounts_refused += 1;
            if findings.lines.len() < 40 {
                findings.lines.push(format!("OPUS_K2C_MOUNT_REFUSED state={ordinal} {refusal:?}"));
            }
            return;
        }
    };
    judge(&pool.image(), knobs, "after-mount", ordinal, findings);
    let mut random = SeededRandomSource::from_seed(0xb56_0000 + u64::try_from(ordinal).expect("小"));
    // 迭代上界：steps。
    for step in 0..steps {
        let live = pool.live_inodes();
        let choice = random.below(100);
        let name = if choice < 25 || live.is_empty() {
            pool.create(1 + random.below(2));
            "create"
        } else if choice < 50 {
            pool.unlink(*live.iter().max().expect("非空"), knobs);
            "unlink-largest"
        } else if choice < 70 {
            pool.unlink(live[usize::try_from(random.below(u64::try_from(live.len()).expect("小"))).expect("小")], knobs);
            "unlink-any"
        } else {
            let target = prefix_txgs[usize::try_from(random.below(u64::try_from(prefix_txgs.len()).expect("小"))).expect("小")];
            match pool.roll_back((InstanceGeneration(1), target)) {
                Ok(()) => "rollback-ok",
                Err(_) => "rollback-refused",
            }
        };
        *findings.steps.entry(name.to_string()).or_default() += 1;
        judge(&pool.image(), knobs, &format!("step{step}-{name}"), ordinal, findings);
    }
}

#[test]
fn opus_probe_b56_crash_abandoned_containers() {
    let knob = env_u64("OPUS_KNOB", 1);
    let knobs = Knobs { tombstone: knob, lineage: knob, retirement: knob, node_entries: usize::try_from(env_u64("OPUS_NODE_ENTRIES", 0)).expect("小") };
    let window_name = env_string("OPUS_WINDOW", "unlink-largest");
    let steps_after_the_mount = env_u64("OPUS_STEPS", 3);
    let threads = usize::try_from(env_u64("OPUS_THREADS", 8)).expect("小");
    let state_cap = env_u64("OPUS_STATE_CAP", 1_000_000);
    let started = std::time::Instant::now();
    let (pool, prefix_txgs) = prefix(knobs);
    let rollback_target = prefix_txgs[1];
    eprintln!("OPUS_K2C prefix knobs={knobs:?} {} txgs={prefix_txgs:?}", pool.tree_summary());
    let base = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut recorded = Pool {
        devices: pool.devices.into_iter().map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))).collect::<Vec<_>>(),
        allocator: pool.allocator,
        output: pool.output,
        instance: pool.instance,
    };
    let txg_before_the_window = recorded.txg();
    window(&mut recorded, &window_name, knobs, rollback_target);
    let txg_after_the_window = recorded.txg();
    eprintln!("OPUS_K2C window={window_name} txgs={}..={} {}", txg_before_the_window + 1, txg_after_the_window, recorded.tree_summary());
    let operations = stream.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(&operations, &geometry());
    let closed_form = closed_form_state_count(&segments);
    let layer0_states = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    eprintln!(
        "OPUS_K2C segments={} lengths={:?} writes={} closed_form_state_count={closed_form} layer0_states={layer0_states}",
        segments.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        writes.len()
    );
    if env_u64("OPUS_COUNT_ONLY", 0) == 1 {
        return;
    }
    assert!(std::env::var("OPUS_TIMING_EXPAND_UP_TO").is_ok() || layer0_states <= state_cap, "这一段全量 {layer0_states} 个状态，超过上限 {state_cap}，不跑");
    let judged_root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("窗口里有根槽写");
    let versions: Vec<PublishedVersion> = prefix_txgs
        .iter()
        .copied()
        .chain(*prefix_txgs.last().expect("非空") + 1..=txg_after_the_window)
        .map(|txg| PublishedVersion { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(txg), content: first_file_content() })
        .collect();
    // 只供估时：OPUS_TIMING_EXPAND_UP_TO 设了时长于它的段不展开（正式跑不设，每一段都展开）。
    let timing_expand_up_to = std::env::var("OPUS_TIMING_EXPAND_UP_TO").ok().and_then(|value| value.parse::<usize>().ok());
    if let Some(limit) = timing_expand_up_to {
        eprintln!("OPUS_K2C timing_only expand_up_to={limit}（估时用，不是全量）");
    }
    let mut table: Option<Vec<RetainedWrite>> = None;
    let mut states: Vec<(Vec<bool>, Option<u64>, Option<u64>, bool)> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &base, &writes, &segments, judged_root_index, &versions, &|_segment_index, segment| timing_expand_up_to.is_none_or(|limit| segment.len() <= limit),
        &mut |image, report| {
            if table.is_none() {
                table = Some(image.writes.to_vec());
            }
            assert_eq!(table.as_ref().expect("上一句").len(), image.writes.len(), "写表每个状态同一份");
            let effective = report.effective_root.map(|(_, txg)| txg.0);
            let newest = newest_persisted_root(image.writes, &image.persisted).map(|(txg, _)| txg.0);
            let cut = some_publish_persisted_without_its_root(image.writes, &image.persisted);
            states.push((image.persisted.clone(), effective, newest, cut));
        },
    );
    eprintln!(
        "OPUS_K2C layer0 states={} violations={} failed_states={} checker_violated={:?} elapsed={:.1}s",
        tally.states, tally.violations, tally.failed_states, tally.checker_violated_states, started.elapsed().as_secs_f64()
    );
    let table = table.expect("至少一个状态");
    let abandoned = states.iter().filter(|(_, effective, newest, cut)| *cut || effective.zip(*newest).is_some_and(|(e, n)| e < n)).count();
    eprintln!("OPUS_K2C states_with_an_abandoned_segment={abandoned} of {}", states.len());
    let chunk = states.len().div_ceil(threads.max(1));
    let merged: Vec<StateFindings> = std::thread::scope(|scope| {
        let handles: Vec<_> = states
            .chunks(chunk.max(1))
            .enumerate()
            .map(|(chunk_index, part)| {
                let (base, table, prefix_txgs) = (&base, &table, &prefix_txgs);
                scope.spawn(move || {
                    let mut findings = StateFindings::default();
                    for (offset, (persisted, _, _, _)) in part.iter().enumerate() {
                        let ordinal = chunk_index * chunk + offset;
                        let image = crash_image(base, table, persisted);
                        judge(&image, knobs, "crash-image", ordinal, &mut findings);
                        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let mut inner = StateFindings::default();
                            continue_after_the_mount(&image, knobs, prefix_txgs, ordinal, steps_after_the_mount, &mut inner);
                            inner
                        }));
                        match outcome {
                            Ok(inner) => {
                                findings.lines.extend(inner.lines);
                                findings.checks += inner.checks;
                                findings.mounts_refused += inner.mounts_refused;
                                for (name, count) in inner.violated { *findings.violated.entry(name).or_default() += count; }
                                for (name, count) in inner.steps { *findings.steps.entry(name).or_default() += count; }
                            }
                            Err(_) => {
                                findings.panics += 1;
                                findings.lines.push(format!("OPUS_K2C_PANIC state={ordinal}"));
                            }
                        }
                    }
                    findings
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程")).collect()
    });
    let mut total = StateFindings::default();
    for part in merged {
        for line in part.lines.iter().take(10) { eprintln!("{line}"); }
        total.checks += part.checks;
        total.mounts_refused += part.mounts_refused;
        total.panics += part.panics;
        for (name, count) in part.violated { *total.violated.entry(name).or_default() += count; }
        for (name, count) in part.steps { *total.steps.entry(name).or_default() += count; }
    }
    eprintln!(
        "OPUS_K2C_SUMMARY window={window_name} knob={knob} states={} checks={} mounts_refused={} panics={} steps={:?} violated={:?} elapsed={:.1}s",
        states.len(), total.checks, total.mounts_refused, total.panics, total.steps, total.violated, started.elapsed().as_secs_f64()
    );
}
