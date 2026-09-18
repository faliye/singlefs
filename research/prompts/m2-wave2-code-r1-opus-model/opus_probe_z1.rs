//! m2-wave2-code-r1 云端攻方腿（Opus）Z1 探针：上界准入。
//! z1a：实例表只增不减，第 371 号实例的写行发布在取号之后 panic（准入只算两棵记录树）。
//! z1b：按上界、按「抵扣已回收记录」、按拷贝上真分配三种算法逐次挂载比，扫初始负载与每次挂载之后的用户覆盖写次数。

mod opus_probe_common;

use opus_probe_common::*;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::transaction::{PublishError, PublishShape, TransactionUnit};
use singlefs_core::mount::MountError;

fn rows_of(output: &singlefs_core::transaction::TransactionOutput) -> usize {
    InstanceTableRecords::parse(&output.unit(TransactionUnit::InstanceTable).bytes)
        .expect("实例表解得开")
        .rows
        .len()
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

/// 上界算法（今天的代码）：第几次（0 = 写行）越过 812；都不越过是 None。
fn upper_bound_refusal(base: usize, planned: usize) -> Option<(usize, usize)> {
    let mut records = base;
    for index in 0..=planned {
        records += if index == 0 { 10 } else { 8 };
        if records > CAPACITY {
            return Some((index, records));
        }
    }
    None
}

/// 抵扣算法 D-a：把已回收的记录按「每盘每个角色先用掉一条」全额抵扣（最宽的那种抵扣）。
fn deduct_reclaimed_refusal(base: usize, reclaimed: usize, planned: usize) -> Option<(usize, usize)> {
    let mut remaining = reclaimed;
    let mut records = base;
    for index in 0..=planned {
        let upper = if index == 0 { 10 } else { 8 };
        let reused = upper.min(remaining);
        remaining -= reused;
        records += upper - reused;
        if records > CAPACITY {
            return Some((index, records));
        }
    }
    None
}

/// 拷贝上真分配：第几次越过 812（或分配不出来记成 usize::MAX）。
fn exact_refusal(exact: &[Option<usize>]) -> Option<(usize, usize)> {
    for (index, count) in exact.iter().enumerate() {
        match count {
            Some(records) if *records > CAPACITY => return Some((index, *records)),
            Some(_) => {}
            None => return Some((index, usize::MAX)),
        }
    }
    None
}

/// 发布路径那一遍（也是上界，按真实的「这一次之前」条数加这一次的角色数 × 盘数）：取号之前换成更松的算法之后，它会不会在取号之后拒。
fn publish_path_refusal(base: usize, exact: &[Option<usize>]) -> Option<(usize, usize)> {
    let mut before = base;
    for (index, count) in exact.iter().enumerate() {
        let upper = if index == 0 { 10 } else { 8 };
        if before + upper > CAPACITY {
            return Some((index, before + upper));
        }
        match count {
            Some(records) => before = *records,
            None => return None,
        }
    }
    None
}

#[test]
fn z1a_instance_table_rows_overflow_after_acquisition() {
    let per_session = env_usize("OPUS_Z1A_PER_SESSION", 0);
    let max_mounts = env_usize("OPUS_Z1A_MOUNTS", 400);
    let mut pool = fresh_pool();
    let mut seed = 0usize;
    println!("z1a per_session_overwrites={per_session}");
    for mount_number in 1..=max_mounts {
        let superblocks_before = superblock_instances(&pool);
        let outcome = remount(&mut pool);
        let log = take_probe_log();
        let pre = log.last().cloned().expect("带文件的一版：插桩记一条");
        let upper = upper_bound_refusal(pre.base_records, pre.planned_warm_ups);
        match outcome {
            MountOutcome::Mounted(mounted) => {
                let row = mounted.output.row_publish.file_version().expect("带文件");
                let mut real = vec![row.allocation_records.len()];
                for warm_up in &mounted.output.warm_up_publishes {
                    real.push(warm_up.file_version().expect("带文件").allocation_records.len());
                }
                let exact: Vec<usize> = pre.exact_counts.iter().map(|count| count.expect("分配得出")).collect();
                assert_eq!(exact, real, "拷贝上真分配的条数 == 真挂载各次发布之后的条数（插桩自检）");
                let rows = rows_of(row);
                if mount_number <= 3 || mount_number % 50 == 0 || rows >= 366 {
                    println!(
                        "mount={mount_number} instance={} rows_before={} rows_after={rows} base={} reclaimed={} planned={} real={real:?} upper_refusal={upper:?}",
                        mounted.output.instance.0, pre.table_rows_before, pre.base_records, pre.reclaimed, pre.planned_warm_ups
                    );
                }
                for _ in 0..per_session {
                    seed += 1;
                    if let Err(error) = overwrite(&mut pool, seed) {
                        println!("mount={mount_number} in-session overwrite refused: {error:?}");
                        break;
                    }
                }
            }
            MountOutcome::Refused(error) => {
                println!(
                    "mount={mount_number} REFUSED base={} reclaimed={} planned={} exact={:?} upper_refusal={upper:?} error={error:?}",
                    pre.base_records, pre.reclaimed, pre.planned_warm_ups, pre.exact_counts
                );
                break;
            }
            MountOutcome::Panicked(message) => {
                println!(
                    "mount={mount_number} PANICKED instance_to_acquire={} rows_before={} rows_to_write>=1 base={} planned={} exact={:?} upper_refusal={upper:?}",
                    pre.instance_to_acquire.0, pre.table_rows_before, pre.base_records, pre.planned_warm_ups, pre.exact_counts
                );
                println!("panic message: {message}");
                println!("superblock instances before this mount: {superblocks_before:?}");
                println!("superblock instances after the panic:   {:?}", superblock_instances(&pool));
                // 再试两次：每次都再烧一个代号。
                for retry in 1..=2 {
                    let outcome = remount(&mut pool);
                    let log = take_probe_log();
                    let pre = log.last().cloned().expect("插桩记一条");
                    let kind = match outcome {
                        MountOutcome::Mounted(_) => "mounted".to_string(),
                        MountOutcome::Refused(error) => format!("refused {error:?}"),
                        MountOutcome::Panicked(message) => format!("panicked: {message}"),
                    };
                    println!(
                        "retry={retry} instance_to_acquire={} rows_before={} -> {kind}; superblock instances now {:?}",
                        pre.instance_to_acquire.0, pre.table_rows_before, superblock_instances(&pool)
                    );
                }
                return;
            }
        }
    }
    panic!("{max_mounts} 次挂载都没 panic");
}

/// 一段历史：初始 `initial_overwrites` 次覆盖写 + `initial_empty` 次空发布（实例 1 里），之后挂载 → 会话里覆盖写 `per_session` 次 → 挂载……
/// 每次挂载记一行：上界、D-a 抵扣、拷贝上真分配三种算法的判定，以及取号之前换成后两种时发布路径那一遍会不会在取号之后拒。
fn run_history(initial_overwrites: usize, initial_empty: usize, per_session: usize, max_mounts: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pool = fresh_pool();
    let mut seed = 0usize;
    for _ in 0..initial_overwrites {
        seed += 1;
        if overwrite(&mut pool, seed).is_err() {
            lines.push(format!("h=({initial_overwrites},{initial_empty},{per_session}) setup overwrite refused"));
            return lines;
        }
    }
    for _ in 0..initial_empty {
        if empty_publish(&mut pool).is_err() {
            lines.push(format!("h=({initial_overwrites},{initial_empty},{per_session}) setup empty refused"));
            return lines;
        }
    }
    for mount_number in 1..=max_mounts {
        let outcome = remount(&mut pool);
        let log = take_probe_log();
        let Some(pre) = log.last().cloned() else {
            let status = match &outcome {
                MountOutcome::Mounted(_) => "mounted".to_string(),
                MountOutcome::Refused(error) => format!("refused {error:?}"),
                MountOutcome::Panicked(message) => format!("panicked {message}"),
            };
            lines.push(format!(
                "h=({initial_overwrites},{initial_empty},{per_session}) mount={mount_number} BEFORE_ADMISSION status={status}"
            ));
            return lines;
        };
        let upper = upper_bound_refusal(pre.base_records, pre.planned_warm_ups);
        let deduct = deduct_reclaimed_refusal(pre.base_records, pre.reclaimed, pre.planned_warm_ups);
        let exact = exact_refusal(&pre.exact_counts);
        let path = publish_path_refusal(pre.base_records, &pre.exact_counts);
        let class = match (upper.is_some(), exact.is_some(), deduct.is_some()) {
            (true, false, _) => "FALSE_REFUSAL",
            (false, true, _) => "LEAK_UPPER",
            (_, true, false) => "LEAK_DEDUCT",
            (true, true, _) => "TRUE_REFUSAL",
            (false, false, _) => "ok",
        };
        let path_note = if exact.is_none() && path.is_some() { " PATH_REFUSES_AFTER_ACQ_IF_PREACQ_EXACT" } else { "" };
        let deduct_path_note = if deduct.is_none() && path.is_some() { " PATH_REFUSES_AFTER_ACQ_IF_PREACQ_DEDUCT" } else { "" };
        let status = match &outcome {
            MountOutcome::Mounted(_) => "mounted".to_string(),
            MountOutcome::Refused(MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }) => "refused_row".to_string(),
            MountOutcome::Refused(MountError::WarmUpAdmissionRefusedBeforeAcquisition { warm_up_publish_index, .. }) => format!("refused_warm_up_{warm_up_publish_index}"),
            MountOutcome::Refused(other) => format!("refused_other {other:?}"),
            MountOutcome::Panicked(message) => format!("panicked {message}"),
        };
        lines.push(format!(
            "h=({initial_overwrites},{initial_empty},{per_session}) mount={mount_number} base={} reclaimed={} planned={} exact={:?} upper={upper:?} deduct={deduct:?} exact_refusal={exact:?} path={path:?} class={class}{path_note}{deduct_path_note} status={status}",
            pre.base_records, pre.reclaimed, pre.planned_warm_ups, pre.exact_counts
        ));
        match outcome {
            MountOutcome::Mounted(mounted) => {
                let row = mounted.output.row_publish.file_version().expect("带文件");
                let mut real = vec![row.allocation_records.len()];
                for warm_up in &mounted.output.warm_up_publishes {
                    real.push(warm_up.file_version().expect("带文件").allocation_records.len());
                }
                let exact_counts: Vec<usize> = pre.exact_counts.iter().map(|count| count.expect("分配得出")).collect();
                assert_eq!(exact_counts, real, "插桩自检");
            }
            _ => return lines,
        }
        for _ in 0..per_session {
            seed += 1;
            let before = pool.allocator.records().len();
            let exact_overwrite = singlefs_core::transaction::opus_probe_exact_record_counts(
                &pool.allocator,
                &[PublishShape { rewrites_file_version: true, rewrites_instance_table: false }],
            );
            match overwrite(&mut pool, seed) {
                Ok(_) => {}
                Err(PublishError::AllocationRecordsExceedOneNode { records, .. }) => {
                    let fits = exact_overwrite[0].is_some_and(|count| count <= CAPACITY);
                    lines.push(format!(
                        "h=({initial_overwrites},{initial_empty},{per_session}) mount={mount_number} in-session overwrite refused: before={before} upper={records} exact={:?} {}",
                        exact_overwrite,
                        if fits { "IN_SESSION_FALSE_REFUSAL" } else { "in_session_true_refusal" }
                    ));
                    break;
                }
                Err(other) => {
                    lines.push(format!("in-session overwrite other error {other:?}"));
                    break;
                }
            }
        }
    }
    lines
}

/// 标定：实现员那一例（48 次覆盖写 + 1 次空发布，挂载一次）。
#[test]
fn z1b_calibration_implementer_case() {
    for line in run_history(48, 1, 0, 1) {
        println!("{line}");
    }
    for line in run_history(47, 2, 0, 1) {
        println!("{line}");
    }
    for line in run_history(49, 1, 0, 1) {
        println!("{line}");
    }
}

/// 扫：初始负载 × 每次挂载之后的覆盖写次数，挂载到被拒或到上限。
#[test]
fn z1b_sweep() {
    let max_mounts = env_usize("OPUS_Z1B_MOUNTS", 40);
    let initial: Vec<usize> = std::env::var("OPUS_Z1B_INITIAL")
        .ok()
        .map(|text| text.split(',').filter_map(|part| part.parse().ok()).collect())
        .unwrap_or_else(|| vec![0, 20, 40, 44, 45, 46, 47, 48, 49]);
    let per_session_values: Vec<usize> = std::env::var("OPUS_Z1B_PER_SESSION")
        .ok()
        .map(|text| text.split(',').filter_map(|part| part.parse().ok()).collect())
        .unwrap_or_else(|| vec![0, 1, 2, 3, 5, 8, 12, 20]);
    let empties: Vec<usize> = std::env::var("OPUS_Z1B_EMPTY")
        .ok()
        .map(|text| text.split(',').filter_map(|part| part.parse().ok()).collect())
        .unwrap_or_else(|| vec![0, 1, 2, 3]);
    for &initial_overwrites in &initial {
        for &initial_empty in &empties {
            for &per_session in &per_session_values {
                for line in run_history(initial_overwrites, initial_empty, per_session, max_mounts) {
                    println!("{line}");
                }
            }
        }
    }
}
