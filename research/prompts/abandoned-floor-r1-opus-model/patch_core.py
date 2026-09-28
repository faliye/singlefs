import sys
root = sys.argv[1]
def patch(path, pairs):
    s = open(path).read()
    for old, new in pairs:
        n = s.count(old)
        assert n == 1, (path, old[:90], n)
        s = s.replace(old, new)
    open(path, 'w').write(s)
    print("patched", path)

mount = root + "/singlefs-core/src/mount.rs"
patch(mount, [
# raise: 乙 / 丙
("""    let oldest_valid_root = roots
        .iter()
        .filter(|root| !abandoned_by_table(root, &table))
        .map(|root| root.checkpoint_txg)
        .min();
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多""",
"""    let oldest_valid_root = roots
        .iter()
        .filter(|root| !abandoned_by_table(root, &table))
        .map(|root| root.checkpoint_txg)
        .min();
    // AF（abandoned-floor-r1 攻方副本）：候选乙（抬 F 不许落进空档）与丙（抬 F 时记账放掉）。
    let af_candidate = std::env::var("AF_CANDIDATE").unwrap_or_default();
    let mut new_floor = new_floor;
    if af_candidate == "yi_adjust" || af_candidate == "yi_refuse" {
        let roots_at_the_floor: Vec<&RootRecord> = roots
            .iter()
            .filter(|root| root.checkpoint_txg == new_floor)
            .collect();
        if !roots_at_the_floor.is_empty()
            && roots_at_the_floor.iter().all(|root| abandoned_by_table(root, &table))
        {
            if af_candidate == "yi_refuse" {
                return Err(MountError::RollbackFloorAboveCeiling {
                    requested: new_floor,
                    ceiling: CheckpointTxg(u64::MAX),
                });
            }
            let next_txg_not_all_abandoned = roots
                .iter()
                .filter(|root| root.checkpoint_txg > new_floor && !abandoned_by_table(root, &table))
                .map(|root| root.checkpoint_txg)
                .min();
            match next_txg_not_all_abandoned {
                Some(next) if ceiling_judged_by_the_admission.is_none_or(|ceiling| next <= ceiling) => {
                    if std::env::var_os("AF_DUMP").is_some() {
                        eprintln!("AF-DUMP yi_adjust {} -> {}", new_floor.0, next.0);
                    }
                    new_floor = next;
                }
                _ => {
                    return Err(MountError::RollbackFloorAboveCeiling {
                        requested: new_floor,
                        ceiling: CheckpointTxg(u64::MAX - 1),
                    });
                }
            }
        }
    }
    let af_reclaim_threshold = if af_candidate.starts_with("bing") {
        roots
            .iter()
            .filter(|root| !abandoned_by_table(root, &table) && root.checkpoint_txg >= new_floor)
            .map(|root| root.checkpoint_txg)
            .min()
            .map_or(new_floor, |lowest_candidate| lowest_candidate.max(new_floor))
    } else {
        new_floor
    };
    if std::env::var_os("AF_DUMP").is_some() {
        eprintln!("AF-DUMP raise new_floor={} reclaim_threshold={} oldest_valid_root={:?}", new_floor.0, af_reclaim_threshold.0, oldest_valid_root.map(|t| t.0));
    }
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多"""),
("""    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );""",
"""    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(af_reclaim_threshold, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );"""),
# mount: 丙_all
("""    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );""",
"""    let af_mount_reclaim_threshold = if std::env::var("AF_CANDIDATE").unwrap_or_default() == "bing_all" {
        roots
            .iter()
            .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= effective_floor)
            .map(|root| root.checkpoint_txg)
            .min()
            .map_or(effective_floor, |lowest_candidate| lowest_candidate.max(effective_floor))
    } else {
        effective_floor
    };
    if std::env::var_os("AF_DUMP").is_some() {
        eprintln!("AF-DUMP mount effective_floor={} reclaim_threshold={} oldest_valid_root={:?}", effective_floor.0, af_mount_reclaim_threshold.0, oldest_valid_root.map(|t| t.0));
    }
    allocator.reclaim_released_up_to(
        reclaim_floor(af_mount_reclaim_threshold, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );"""),
])

recovery = root + "/singlefs-core/src/recovery.rs"
patch(recovery, [
("""    let highest_on_the_roots = newest_valid_root_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .max();""",
"""    // AF（abandoned-floor-r1 攻方副本）：AF_ABANDONED_F=count 时被抛弃根带的 F 也算进 F_生效（第 ② 行「算进」）。
    let af_highest_on_abandoned_roots = if std::env::var("AF_ABANDONED_F").unwrap_or_default() == "count" {
        roots_with_ring_slots
            .iter()
            .filter(|(_, root)| newest_roots_table.is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table)))
            .map(|(_, root)| root.rollback_floor)
            .max()
    } else {
        None
    };
    let highest_on_the_roots = newest_valid_root_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .chain(af_highest_on_abandoned_roots)
        .max();"""),
])
