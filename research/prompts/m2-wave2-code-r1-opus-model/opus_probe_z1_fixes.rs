//! m2-wave2-code-r1 云端攻方腿（Opus）Z1 改法臂：副本里 `OPUS_ADMISSION_MODE` 切四种准入（0 原样、1 抵扣已回收两处、
//! 2 取号之前真分配而发布路径原样、3 两处都真分配），在第 z1b 扫出的几段历史上各挂载一次，看被拒、在取号之后被拒（烧号）、还是 panic。
//! 只在攻方副本上量过，改法被攻过零轮。

mod opus_probe_common;

use opus_probe_common::*;
use singlefs_core::transaction::OPUS_ADMISSION_MODE;

fn outcome_text(outcome: &MountOutcome) -> String {
    match outcome {
        MountOutcome::Mounted(mounted) => {
            let mut counts = vec![mounted.output.row_publish.file_version().expect("带文件").allocation_records.len()];
            counts.extend(mounted.output.warm_up_publishes.iter().map(|version| version.file_version().expect("带文件").allocation_records.len()));
            format!("mounted instance={} records_after_each={counts:?}", mounted.output.instance.0)
        }
        MountOutcome::Refused(error) => format!("refused {error:?}"),
        MountOutcome::Panicked(message) => format!("PANICKED {message}"),
    }
}

fn run(mode: u8, initial_overwrites: usize, initial_empty: usize, per_session: usize, mounts: usize) {
    OPUS_ADMISSION_MODE.with(|cell| cell.set(0));
    let mut pool = fresh_pool();
    let mut seed = 0;
    for _ in 0..initial_overwrites {
        seed += 1;
        overwrite(&mut pool, seed).expect("初始覆盖写");
    }
    for _ in 0..initial_empty {
        empty_publish(&mut pool).expect("初始空发布");
    }
    OPUS_ADMISSION_MODE.with(|cell| cell.set(mode));
    for mount_number in 1..=mounts {
        let before = superblock_instances(&pool);
        let outcome = remount(&mut pool);
        let _ = take_probe_log();
        let after = superblock_instances(&pool);
        let burned = before != after && !matches!(outcome, MountOutcome::Mounted(_));
        println!(
            "mode={mode} h=({initial_overwrites},{initial_empty},{per_session}) mount={mount_number} {} superblocks {before:?} -> {after:?}{}",
            outcome_text(&outcome),
            if burned { " INSTANCE_BURNED" } else { "" }
        );
        if !matches!(outcome, MountOutcome::Mounted(_)) {
            break;
        }
        for _ in 0..per_session {
            seed += 1;
            if let Err(error) = overwrite(&mut pool, seed) {
                println!("  in-session overwrite refused {error:?}");
                break;
            }
        }
    }
    OPUS_ADMISSION_MODE.with(|cell| cell.set(0));
}

#[test]
fn z1_fix_arms_on_the_swept_cells() {
    let mounts = std::env::var("OPUS_FIX_MOUNTS").ok().and_then(|value| value.parse().ok()).unwrap_or(1);
    let per_session = std::env::var("OPUS_FIX_PER_SESSION").ok().and_then(|value| value.parse().ok()).unwrap_or(0);
    for (initial_overwrites, initial_empty) in [(48usize, 1usize), (47, 2), (47, 3), (49, 0), (48, 2), (49, 1), (48, 3)] {
        for mode in [0u8, 1, 2, 3] {
            run(mode, initial_overwrites, initial_empty, per_session, mounts);
        }
    }
}

/// 最自然的一段：mkfs → 第一个文件 → 可写挂载 → 一路覆盖写到被准入拒 → 再可写挂载（四种准入各一遍），之后再挂载几次、每次写到被拒。
#[test]
fn z1_fix_arms_write_until_refused_then_remount() {
    let mounts = std::env::var("OPUS_FIX_MOUNTS").ok().and_then(|value| value.parse().ok()).unwrap_or(4);
    for mode in [0u8, 1, 2, 3] {
        OPUS_ADMISSION_MODE.with(|cell| cell.set(mode));
        let mut pool = fresh_pool();
        let mut seed = 0;
        for mount_number in 1..=mounts {
            let before = superblock_instances(&pool);
            let outcome = remount(&mut pool);
            let log = take_probe_log();
            let after = superblock_instances(&pool);
            let burned = before != after && !matches!(outcome, MountOutcome::Mounted(_));
            let pre = log.last().map(|entry| format!("base={} reclaimed={} exact={:?}", entry.base_records, entry.reclaimed, entry.exact_counts)).unwrap_or_default();
            println!("mode={mode} mount={mount_number} {pre} {}{}", outcome_text(&outcome), if burned { " INSTANCE_BURNED" } else { "" });
            if !matches!(outcome, MountOutcome::Mounted(_)) {
                break;
            }
            let mut writes = 0;
            loop {
                seed += 1;
                let before_records = pool.allocator.records().len();
                let reclaimed = pool.allocator.opus_probe_reclaimed_count();
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| overwrite(&mut pool, seed))) {
                    Ok(Ok(_)) => writes += 1,
                    Ok(Err(error)) => {
                        println!("  wrote {writes} then refused: {}", format!("{error:?}").chars().take(80).collect::<String>());
                        break;
                    }
                    Err(_) => {
                        println!("  wrote {writes} then the next overwrite PANICKED (records before={before_records}, reclaimed={reclaimed})");
                        break;
                    }
                }
                if writes >= 400 {
                    println!("  wrote 400");
                    break;
                }
            }
        }
    }
    OPUS_ADMISSION_MODE.with(|cell| cell.set(0));
}
