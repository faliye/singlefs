
// ---------- K3：造一形把 F、H 推完之后那一段与 walkSkipped 分开（卡在走空里、预演里推环做得成） ----------

/// 每块盘没挡的槽标成已分配、只留 `leave` 个（注入级，与实现员 `pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent` 同一造法）。
fn atk_leave_unblocked(pool: &mut PoolUnderTest<SparseBlockDevice>, leave: u64) {
    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= leave {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// C 那一圈罩着、这一刻隔离着的槽，每块盘只留 `keep` 个隔离着（其余标成已分配，隔离位不动：推环解开隔离之后它们照旧占着，
/// 推环放不出它们；注入级）。交回每块盘标了几个。
fn atk_keep_only_isolated_by_c(pool: &mut PoolUnderTest<SparseBlockDevice>, keep: u64) -> Vec<u64> {
    let session = pool.session.as_mut().expect("挂着的会话");
    let placements: Vec<PlacementOnDevice> = session
        .allocator
        .root_ring_occupancy()
        .expect("根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_values()
        .flatten()
        .collect();
    let mut marked = Vec::new();
    for device_map in &mut session.allocator.devices {
        let mut kept = 0u64;
        let mut count = 0u64;
        let mut seen = std::collections::BTreeSet::new();
        let device = device_map.device;
        for placement in placements.iter().filter(|placement| placement.device == device) {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                if !seen.insert(slot) || !device_map.is_isolated(SlotNumber(slot)) {
                    continue;
                }
                if kept < keep {
                    kept += 1;
                } else {
                    device_map.mark_allocated(SlotNumber(slot), 1);
                    count += 1;
                }
            }
        }
        marked.push(count);
    }
    marked
}

/// 照出路二真走、推完之后发 `then`（不必是被拒的那一个改动）：交回推环短名与再发结局短名。
fn atk_push_then(snap: &AtkSnap, then: &AtkChange) -> String {
    let mut pool = snap.pool();
    match push_the_root_ring_of(&mut pool) {
        Ok(done) => format!("pushed({})->{}", done.versions_published.len(), atk_kind(&atk_apply(&mut pool, then))),
        Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
    }
}

/// 搜：K1 那一形挂上之后长 13 个单元被拒一次，再覆盖写 `ow` 次（推环要推的次数随之变少），再把没挡的槽压到 `leave` 个，unlink 第一个文件：
/// 卡在意图之后的走空里（H）的，要出路、照出路二真走（推完再发同一个 unlink；推完建一个 inode）；再从那一刻发覆盖写与顺序写：
/// 卡在这次改动之前的走空里（F）的，要出路、照出路二真走。
