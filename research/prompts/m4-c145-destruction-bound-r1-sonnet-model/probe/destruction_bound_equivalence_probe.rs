//! 推论轮 m4-c145-destruction-bound-r1 正推腿的探针（只在草稿目录的仓副本里，不进仓）：
//! 用 core 的真写路径枚举短历史（覆盖写、建快照、写销毁意图、放一批、可写挂载续做），每个历史的最后一版上
//! ① 对每个“销毁中且下一侧是活头”的快照，取盘上读得到的、`entry_is_pending_release` 按无穷上界会算进待放集的条目
//!    （头 = 活头、纪元 >= lo、birth > P），核它们的纪元都 < 写意图那次发布的 txg，也 < 本版的 txg；
//! ② 跑真 checker `check_pool_image`，数红的条数。
//! 环境变量 DEPTH（默认 5）。
mod common;

use std::collections::BTreeMap;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{InstanceGeneration, TreeIdentifier};
use singlefs_core::mount::mount_writable;
use singlefs_core::records::{birth_txg_of_a_mapping_key, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::transaction::{
    next_side_of_the_snapshot, publish_create_snapshot, publish_snapshot_destruction_intent,
    publish_snapshot_destruction_step, NextSideOfASnapshot, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

#[derive(Clone, Copy, Debug)]
enum Step {
    Overwrite,
    CreateSnapshot,
    IntentOnTheFirstSnapshot,
    IntentOnTheLastSnapshot,
    OneBatch,
    WritableMountContinues,
}

const STEPS: [Step; 6] = [
    Step::Overwrite,
    Step::CreateSnapshot,
    Step::IntentOnTheFirstSnapshot,
    Step::IntentOnTheLastSnapshot,
    Step::OneBatch,
    Step::WritableMountContinues,
];

fn overwrite_content(seed: u32) -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect()
}

fn snapshot_trees(version: &TransactionOutput) -> Vec<TreeIdentifier> {
    let mut trees: Vec<TreeIdentifier> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .collect();
    trees.sort();
    trees
}

#[derive(Default, Debug)]
struct Observation {
    histories: u64,
    refused_steps: u64,
    flagged_head_next_versions: u64,
    entries_the_infinite_bound_counts_as_pending: u64,
    entries_with_epoch_at_or_after_the_intent_txg: u64,
    entries_with_epoch_at_or_after_the_version_txg: u64,
    checker_violations: u64,
    first_violations: Vec<String>,
}

/// 从头施加 `history`；某一步被拒（返回 None）。交回最后一版与每个销毁意图那次发布的 txg。
fn replay(history: &[Step], tag: &str) -> Option<(BuiltPool, BTreeMap<TreeIdentifier, u64>)> {
    let mut pool = build_pool(tag);
    let mut intent_txgs: BTreeMap<TreeIdentifier, u64> = BTreeMap::new();
    for (index, step) in history.iter().enumerate() {
        let parameters = parameters();
        let previous = pool.output.clone();
        match step {
            Step::Overwrite => {
                let seed = u32::try_from(index).expect("小") * 7 + 5;
                pool.output = publish_overwrite_in_process(
                    &mut pool,
                    &previous,
                    &overwrite_content(seed),
                    FIXED_WRITE_TIME_SECONDS + 60,
                    INSTANCE,
                )
                .ok()?;
            }
            Step::CreateSnapshot => {
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                let created =
                    publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
                        .ok()?;
                pool.output = created;
            }
            Step::IntentOnTheFirstSnapshot | Step::IntentOnTheLastSnapshot => {
                let trees = snapshot_trees(&previous);
                let snapshot = match step {
                    Step::IntentOnTheFirstSnapshot => *trees.first()?,
                    // 只有一个快照时“最后一个”与“第一个”是同一个：当重复剪掉
                    _ if trees.len() < 2 => return None,
                    _ => *trees.last()?,
                };
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                let intent = publish_snapshot_destruction_intent(
                    &mut writer,
                    &mut pool.allocator,
                    &previous,
                    snapshot,
                    INSTANCE,
                )
                .ok()?;
                intent_txgs.insert(snapshot, intent.root.checkpoint_txg.0);
                pool.output = intent;
            }
            Step::OneBatch => {
                let snapshot = *previous.snapshots_being_destroyed.first()?;
                let devices = pool.devices.as_mut().expect("镜像还开着");
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                let batch = publish_snapshot_destruction_step(
                    &mut writer,
                    &mut pool.allocator,
                    &previous,
                    snapshot,
                    UnlinkBatchSize::CappedForTests(
                        UnlinkBatchDataUnitsCappedForTests::new(1).expect("压成 1"),
                    ),
                    INSTANCE,
                )
                .ok()?;
                pool.output = batch;
            }
            Step::WritableMountContinues => {
                if previous.snapshots_being_destroyed.is_empty() {
                    return None;
                }
                let mut devices = pool.reopen_recorded();
                let mounted = mount_writable(&parameters, &mut devices).ok()?;
                pool.devices = Some(devices);
                pool.output = mounted.current.version_of_trees()?.clone();
                pool.allocator = mounted.allocator;
            }
        }
    }
    Some((pool, intent_txgs))
}

fn observe(pool: &BuiltPool, intent_txgs: &BTreeMap<TreeIdentifier, u64>, history: &[Step], seen: &mut Observation) {
    let version = &pool.output;
    let version_txg = version.root.checkpoint_txg.0;
    for destroyed in &version.snapshots_being_destroyed {
        let Some(next) = next_side_of_the_snapshot(&version.tree_table_entries, *destroyed) else {
            continue;
        };
        if next != NextSideOfASnapshot::LiveHead {
            continue;
        }
        let entry = version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == *destroyed)
            .expect("销毁中的快照在树表里");
        seen.flagged_head_next_versions += 1;
        let lo = entry.birth_txg.0;
        let predecessor = entry.previous_snapshot_txg.0;
        let intent_txg = intent_txgs.get(destroyed).copied();
        for deadlist_entry in &version.deadlist_entries {
            let birth = birth_txg_of_a_mapping_key(&deadlist_entry.mapping_key).map_or(0, |b| b.0);
            if deadlist_entry.head.0 == entry.head_identifier
                && deadlist_entry.epoch.0 >= lo
                && birth > predecessor
            {
                seen.entries_the_infinite_bound_counts_as_pending += 1;
                if let Some(h) = intent_txg {
                    if deadlist_entry.epoch.0 >= h {
                        seen.entries_with_epoch_at_or_after_the_intent_txg += 1;
                    }
                }
                if deadlist_entry.epoch.0 >= version_txg {
                    seen.entries_with_epoch_at_or_after_the_version_txg += 1;
                }
            }
        }
    }
    for (name, verdict) in check_pool_image(&pool.memory_pool()) {
        if let InvariantVerdict::Violated(message) = verdict {
            seen.checker_violations += 1;
            if seen.first_violations.len() < 3 {
                let short: String = message.chars().take(200).collect();
                seen.first_violations.push(format!("{history:?} {name} {short}"));
            }
        }
    }
}

fn explore(prefix: &[Step], max_depth: usize, tag: &str) -> Observation {
    let mut seen = Observation::default();
    let mut stack: Vec<Vec<Step>> = vec![prefix.to_vec()];
    while let Some(history) = stack.pop() {
        let Some((pool, intent_txgs)) = replay(&history, tag) else {
            seen.refused_steps += 1;
            continue;
        };
        seen.histories += 1;
        observe(&pool, &intent_txgs, &history, &mut seen);
        drop(pool);
        let terminal = matches!(history.last(), Some(Step::WritableMountContinues));
        if history.len() < max_depth && !terminal {
            for step in STEPS {
                let mut longer = history.clone();
                longer.push(step);
                stack.push(longer);
            }
        }
    }
    seen
}

fn depth() -> usize {
    std::env::var("DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(5)
}

macro_rules! branch {
    ($name:ident, $depth:expr, $($step:expr),+) => {
        #[test]
        fn $name() {
            let prefix = [$($step),+];
            let seen = explore(&prefix, $depth, stringify!($name));
            println!("PROBE {} depth={} {seen:?}", stringify!($name), $depth);
            assert_eq!(seen.entries_with_epoch_at_or_after_the_intent_txg, 0);
            assert_eq!(seen.entries_with_epoch_at_or_after_the_version_txg, 0);
            assert_eq!(seen.checker_violations, 0);
        }
    };
}

// 一步的历史只看它们自己（深度 1）；两步的前缀各自往下探到 DEPTH。被拒的两步前缀（先写意图、先放批、先挂载）不列。
branch!(single_overwrite, 1, Step::Overwrite);
branch!(single_create_snapshot, 1, Step::CreateSnapshot);
branch!(prefix_overwrite_then_overwrite, depth(), Step::Overwrite, Step::Overwrite);
branch!(prefix_overwrite_then_create_snapshot, depth(), Step::Overwrite, Step::CreateSnapshot);
branch!(prefix_create_snapshot_then_overwrite, depth(), Step::CreateSnapshot, Step::Overwrite);
branch!(prefix_create_snapshot_then_create_snapshot, depth(), Step::CreateSnapshot, Step::CreateSnapshot);
branch!(prefix_create_snapshot_then_intent, depth(), Step::CreateSnapshot, Step::IntentOnTheFirstSnapshot);
