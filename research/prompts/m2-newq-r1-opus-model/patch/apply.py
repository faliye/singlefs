#!/usr/bin/env python3
"""把 m2-newq-r1 云端攻方腿的最小释放核验打进副本（只动 /tmp/claude-1000/m2-newq-opus/repo）。"""
import pathlib, sys
root = pathlib.Path(sys.argv[1])
tx = root / "crates/singlefs-core/src/transaction.rs"
mount = root / "crates/singlefs-core/src/mount.rs"
here = pathlib.Path(__file__).parent

def replace_once(path, old, new):
    text = path.read_text()
    n = text.count(old)
    assert n == 1, (path, n, old[:80])
    path.write_text(text.replace(old, new))

# 1. 新错误成员
replace_once(tx, """    BlockDevice(BlockDeviceError),
}

impl From<BlockDeviceError> for PublishError {""", """    BlockDevice(BlockDeviceError),
    /// 副本（m2-newq-r1 攻方）：N1-B，释放前读盘核时读本身失败，发布失败交回。
    ReleaseCheckReadFailed { unit: TransactionUnit, slot: SlotNumber },
    /// 副本（m2-newq-r1 攻方）：N1-C，交给 D23 已定项 14 的失败表（探针写与只读复核由驱动模拟）。
    ReleaseCheckReadFailedGoesToFailureTable { unit: TransactionUnit, device: DeviceIdentity, slot: SlotNumber },
}

impl From<BlockDeviceError> for PublishError {""")

# 2. 发布路径里调核验
replace_once(tx, """    publish_admission(allocator, &rewritten, resolved.inode_tree.containers.len())?;
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let outcome = publish_admitted(pool, allocator, &plan, previous, &resolved, &release, trees);
    if outcome.is_err() {
        *allocator = allocator_before_this_publish;
    }
    outcome
}""", """    // 副本（m2-newq-r1 攻方）：释放前读盘核校验和。关着时 plan 为空、下面逐字节同今天。
    let check_plan = match previous {
        Some(previous_version) if release_check_model::arms().enabled => {
            release_check_before_release(&*pool, previous_version, &rewritten)?
        }
        _ => release_check_model::CheckPlan::default(),
    };
    let release: Vec<Placement> = release
        .into_iter()
        .filter(|placement| !check_plan.keep_allocated.contains(placement))
        .collect();
    publish_admission(allocator, &rewritten, resolved.inode_tree.containers.len())?;
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let outcome = publish_admitted(pool, allocator, &plan, previous, &resolved, &release, trees);
    if outcome.is_err() {
        *allocator = allocator_before_this_publish;
    } else {
        for (placement, devices) in &check_plan.isolate {
            for device in devices {
                allocator.isolate_abandoned(*device, placement.slot, placement.span);
                release_check_model::note_isolated(*device, placement.span);
            }
        }
    }
    outcome
}

/// 副本（m2-newq-r1 攻方）：对这次换下的每个进映射的单元，按上一版映射条目里的位置项读盘、比校验和，按装上的臂给出处置。
fn release_check_before_release<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    previous: &TransactionOutput,
    rewritten: &[TransactionUnit],
) -> Result<release_check_model::CheckPlan, PublishError> {
    use release_check_model::{IsolationScope, MirrorRule, MirrorVerdict, OnMismatch, OnReadFailure};
    let arms = release_check_model::arms();
    let mapping_node_bytes = &previous.unit(TransactionUnit::MappingTree).bytes;
    let mut plan = release_check_model::CheckPlan::default();
    for identity in rewritten.iter().copied() {
        if let TransactionUnit::InodeLeafContainer(index) = identity {
            if index.position() >= previous.inode_leaf_containers.len() {
                continue;
            }
        }
        let is_mapped = matches!(
            identity,
            TransactionUnit::Data
                | TransactionUnit::ExtentRoot
                | TransactionUnit::InodeLeafContainer(_)
                | TransactionUnit::InodeRoot
                | TransactionUnit::AllocationTree
                | TransactionUnit::AccountingTree
        );
        if !is_mapped {
            continue; // 自举豁免三类不适用硬规则 1（D19 已定项 8）。
        }
        let (_, key) = previous
            .mapped_units
            .iter()
            .find(|(mapped, _)| *mapped == identity)
            .expect("进映射的单元每个一把 key");
        let MappingLookup::Found(locations) = mapping_locations_for_key(mapping_node_bytes, key) else {
            unreachable!("placements_to_release_via_mapping 刚查过");
        };
        let slot = locations[0].slot;
        let placement = Placement { slot, span: identity.span_slots() };
        let verdicts = release_check_model::verdicts(
            pool.devices,
            &locations,
            identity.span_slots(),
            arms.mirror_rule == MirrorRule::AnyMirror,
        );
        let any_match = verdicts.iter().any(|(_, v)| *v == MirrorVerdict::Match);
        let all_match = verdicts.len() == locations.len() && verdicts.iter().all(|(_, v)| *v == MirrorVerdict::Match);
        let any_read_failed = verdicts.iter().any(|(_, v)| *v == MirrorVerdict::ReadFailed);
        let passes = match arms.mirror_rule {
            MirrorRule::AnyMirror => any_match,
            MirrorRule::EveryMirror => all_match,
        };
        let mut outcome = "pass";
        if !passes {
            if any_read_failed {
                let failed_device = verdicts
                    .iter()
                    .find(|(_, v)| *v == MirrorVerdict::ReadFailed)
                    .map(|(device, _)| *device)
                    .expect("有一份读失败");
                match arms.on_read_failure {
                    OnReadFailure::AsMismatch => {}
                    OnReadFailure::FailPublish => {
                        release_check_model::LOG.with(|log| log.borrow_mut().push(release_check_model::CheckEvent {
                            unit: identity.tag(), slot, verdicts: verdicts.clone(), outcome: "publish-failed" }));
                        return Err(PublishError::ReleaseCheckReadFailed { unit: identity, slot });
                    }
                    OnReadFailure::FailureTable => {
                        release_check_model::LOG.with(|log| log.borrow_mut().push(release_check_model::CheckEvent {
                            unit: identity.tag(), slot, verdicts: verdicts.clone(), outcome: "failure-table" }));
                        return Err(PublishError::ReleaseCheckReadFailedGoesToFailureTable {
                            unit: identity, device: failed_device, slot });
                    }
                }
            }
            let bad_devices: Vec<DeviceIdentity> = pool
                .devices
                .iter()
                .map(|(device, _)| *device)
                .filter(|device| {
                    !verdicts.iter().any(|(d, v)| d == device && *v == MirrorVerdict::Match)
                })
                .collect();
            match arms.on_mismatch {
                OnMismatch::ReleaseRecordAndIsolate => {
                    let devices = match arms.scope {
                        IsolationScope::EveryDevice => pool.devices.iter().map(|(d, _)| *d).collect(),
                        IsolationScope::MismatchingDeviceOnly => bad_devices,
                    };
                    plan.isolate.push((placement, devices));
                    outcome = "released-and-isolated";
                }
                OnMismatch::KeepRecordAllocated => {
                    plan.keep_allocated.push(placement);
                    outcome = "record-kept-allocated";
                }
            }
        }
        release_check_model::LOG.with(|log| log.borrow_mut().push(release_check_model::CheckEvent {
            unit: identity.tag(), slot, verdicts, outcome }));
    }
    Ok(plan)
}""")

# 3. 模块本体
text = tx.read_text()
tx.write_text(text + (here / "release_check_model.rs").read_text())

# 4. mount：重挂现算那一臂
replace_once(mount, """    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    let abandoned_roots_unreadable = match shadow_ledger {""", """    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    // 副本（m2-newq-r1 攻方）：N3「重挂时现算」——分配记录已释放、却被所选根那一版的单元引用着的槽，隔离。
    if crate::transaction::release_check_model::arms().recompute_at_mount {
        if let PreviousVersion::WithFile { output, .. } = previous {
            let referenced: Vec<(u64, u64)> = output
                .units
                .iter()
                .map(|unit| (unit.slot.0, unit.identity.span_slots()))
                .collect();
            let released: Vec<AllocationRecord> = allocator
                .records()
                .iter()
                .filter(|record| record.is_released)
                .copied()
                .collect();
            for record in released {
                let start = record.slot.0;
                let end = start + u64::from(record.span_slots);
                if referenced.iter().any(|(slot, span)| *slot < end && start < slot + span) {
                    allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
                    crate::transaction::release_check_model::ISOLATED_AT_MOUNT
                        .with(|count| *count.borrow_mut() += u64::from(record.span_slots));
                }
            }
        }
    }
    let abandoned_roots_unreadable = match shadow_ledger {""")
print("patched")
