//! m2-final-code-r2 云端攻方 Z12：取号之前在分配器拷贝上预演的落点与真发的落点。只在冻结副本的拷贝上跑，
//! 容量开关靠副本里 `PoolWriter::new` 读的环境变量（SINGLEFS_R2_OPUS_CAPS / SINGLEFS_R2_OPUS_NAMED）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common::{
    build_pool, crash_state_devices, format_pool, parameters, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_rollback, mount_writable, MountError, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::transaction::{publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::{RecordedOperationKind, SharedStream};

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

fn ring_roots(image: &MemoryPool) -> Vec<(u32, u64)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<_> = readable_roots(
        image,
        &sc.immutable.region_devices,
        &sc.immutable.sizes,
        &sc.immutable.filesystem_identifier,
    )
    .iter()
    .map(|root| (root.instance.0, root.checkpoint_txg.0))
    .collect();
    roots.sort();
    roots
}

#[derive(Default, Debug)]
struct Tally {
    mounts_ok: usize,
    refused: std::collections::BTreeMap<String, usize>,
    panics: Vec<String>,
}

fn classify(result: &Result<(), MountError>) -> String {
    match result {
        Ok(()) => "ok".to_string(),
        Err(error) => {
            let text = format!("{error:?}");
            text.split(|c: char| c == ' ' || c == '{' || c == '(').next().unwrap_or("").to_string()
                + "/"
                + text.split(|c: char| c == '(' || c == '{').nth(1).unwrap_or("").trim()
        }
    }
}

/// 在一份镜像上试一次挂载（不改镜像），记结果；panic 也接住记下。
fn try_mount(image: &MemoryPool, label: &str, rollback: Option<RollbackTarget>, tally: &mut Tally) -> Option<Vec<singlefs_harness::RetainedOperation>> {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let outcome = catch_unwind(AssertUnwindSafe(|| match rollback {
        None => mount_writable(&parameters(), &mut devices).map(|_| ()),
        Some(target) => mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On).map(|_| ()),
    }));
    match outcome {
        Ok(result) => {
            let class = classify(&result);
            if result.is_ok() {
                tally.mounts_ok += 1;
                return Some(stream.retained_operations());
            }
            *tally.refused.entry(class).or_default() += 1;
            None
        }
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            tally.panics.push(format!("{label}: {}", message.chars().take(300).collect::<String>()));
            None
        }
    }
}

/// 带文件的池：第一个文件之后覆盖写到 txg `last_txg`；之后 `rounds` 轮「可写挂载 → 覆盖写 w 次（w 在 0..=2 之间轮换）」。
/// 每一轮挂载之前：对环里每一条根试一次回退（不落盘）；对上一轮最后那次覆盖写的录制流逐前缀截一次、各试一次可写挂载（不落盘）。
fn sweep_with_file(tag: &str, last_txg: u64, rounds: usize, new_inodes_every: usize) -> Tally {
    let mut pool = build_pool(tag);
    let mut current = pool.output.clone();
    let mut seed = 1;
    {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        while current.root.checkpoint_txg.0 < last_txg {
            current = publish_overwrite(
                &mut writer,
                &mut pool.allocator,
                &current,
                FirstFile { content: &content_of(seed), write_time_seconds: FIXED_WRITE_TIME_SECONDS + seed as u64 },
                InstanceGeneration(1),
            )
            .expect("覆盖写");
            seed += 1;
        }
    }
    let mut image = pool.memory_pool();
    drop(pool);
    let mut tally = Tally::default();
    let mut last_publish: Option<(MemoryPool, Vec<singlefs_harness::RetainedOperation>)> = None;
    for round in 0..rounds {
        for (instance, txg) in ring_roots(&image) {
            try_mount(
                &image,
                &format!("{tag} round {round} rollback ({instance},{txg})"),
                Some(RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) }),
                &mut tally,
            );
        }
        if let Some((before, ops)) = &last_publish {
            for prefix in 0..ops.len() {
                let mut cut = before.clone();
                cut.apply(&ops[..prefix]);
                try_mount(&cut, &format!("{tag} round {round} prefix {prefix}/{}", ops.len()), None, &mut tally);
            }
        }
        // 真挂载一次（落盘），接着在同一个进程里写 w 次。
        let stream = SharedStream::retaining_contents();
        let mut devices = crash_state_devices(&image, &[], &[], &stream);
        let mounted = catch_unwind(AssertUnwindSafe(|| mount_writable(&parameters(), &mut devices)));
        let mut mounted = match mounted {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                *tally.refused.entry(format!("main-line:{}", classify(&Err(e)))).or_default() += 1;
                break;
            }
            Err(_) => {
                tally.panics.push(format!("{tag} round {round} main-line mount panicked"));
                break;
            }
        };
        tally.mounts_ok += 1;
        if let PoolVersion::WithFile(v) = &mounted.current {
            let shape = format!("acct_nodes={} map_nodes={}", v.accounting_tree.node_count(), v.central_mapping_tree.node_count());
            *tally.refused.entry(format!("main-line-shape {shape}")).or_default() += 1;
        }
        let mut ops = stream.retained_operations();
        image.apply(&ops);
        let writes = round % 3;
        let PoolVersion::WithFile(mut version) = mounted.current.clone() else { panic!("带文件") };
        for w in 0..writes {
            let before = image.clone();
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            let publish_parameters = parameters();
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            let next = if new_inodes_every > 0 && (round + w) % new_inodes_every == 0 {
                publish_new_inodes(&mut writer, &mut mounted.allocator, &version, 3, FIXED_WRITE_TIME_SECONDS + 7, mounted.output.instance)
            } else {
                publish_overwrite(
                    &mut writer,
                    &mut mounted.allocator,
                    &version,
                    FirstFile { content: &content_of(seed), write_time_seconds: FIXED_WRITE_TIME_SECONDS + seed as u64 },
                    mounted.output.instance,
                )
            };
            seed += 1;
            match next {
                Ok(next) => version = next,
                Err(e) => {
                    *tally.refused.entry(format!("publish:{e:?}").chars().take(80).collect()).or_default() += 1;
                    break;
                }
            }
            ops = stream.retained_operations();
            image.apply(&ops);
            last_publish = Some((before, ops.clone()));
        }
    }
    tally
}

fn caps_label() -> String {
    format!(
        "caps={:?} named={:?}",
        std::env::var("SINGLEFS_R2_OPUS_CAPS").ok(),
        std::env::var("SINGLEFS_R2_OPUS_NAMED").ok()
    )
}

#[test]
fn z12_with_file_sweep() {
    let rounds: usize = std::env::var("R2_OPUS_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let inodes: usize = std::env::var("R2_OPUS_INODES_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let last_txg: u64 = std::env::var("R2_OPUS_LAST_TXG").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
    let tally = sweep_with_file("z12f", last_txg, rounds, inodes);
    println!("Z12 with-file {} rounds={rounds} inodes_every={inodes}: mounts_ok={} refused={:?} panics={}", caps_label(), tally.mounts_ok, tally.refused, tally.panics.len());
    for p in tally.panics.iter().take(5) {
        println!("  PANIC {p}");
    }
}

/// 树表 0 条的池：只做过 mkfs，连着可写挂载 `mounts` 次（每次都写行）；每次之前对环里每条根试回退（不落盘），
/// 对上一次挂载的录制流逐前缀截一次、各试一次可写挂载。
#[test]
fn z12_without_file_sweep() {
    let mounts: usize = std::env::var("R2_OPUS_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let pool = format_pool("z12n");
    let mut image = pool.memory_pool();
    drop(pool);
    let mut tally = Tally::default();
    let mut last: Option<(MemoryPool, Vec<singlefs_harness::RetainedOperation>)> = None;
    for round in 0..mounts {
        for (instance, txg) in ring_roots(&image) {
            try_mount(
                &image,
                &format!("z12n round {round} rollback ({instance},{txg})"),
                Some(RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) }),
                &mut tally,
            );
        }
        if let Some((before, ops)) = &last {
            for prefix in 0..ops.len() {
                let mut cut = before.clone();
                cut.apply(&ops[..prefix]);
                try_mount(&cut, &format!("z12n round {round} prefix {prefix}"), None, &mut tally);
            }
        }
        let before = image.clone();
        match try_mount(&image, &format!("z12n round {round} main"), None, &mut tally) {
            Some(ops) => {
                image.apply(&ops);
                last = Some((before, ops));
            }
            None => break,
        }
    }
    println!("Z12 without-file {} mounts={mounts}: mounts_ok={} refused={:?} panics={}", caps_label(), tally.mounts_ok, tally.refused, tally.panics.len());
    for p in tally.panics.iter().take(5) {
        println!("  PANIC {p}");
    }
}
