// 样本：只留探针要数的那几行，别的都不是这道阶段看的东西。
fn isolate_slots_referenced_only_by_abandoned_roots() {
    let key = (device.0, slot.0);
    let covered = (slot.0..end_slot).map(move |covered_slot| (device.0, covered_slot));
}

fn rebuilt_allocator() {
    allocator.reclaim_released_up_to(floor, ReclaimedReuse::Immediately);
    allocator.release_reclaim_holds();
}

fn reclaim_at_mount() {
    allocator.reclaim_released_up_to(floor, ReclaimedReuse::Immediately);
}

fn raise_rollback_floor() {
    let reclaimed = allocator.reclaim_released_up_to(floor, ReclaimedReuse::HeldUntilFloorTakesEffect);
    allocator.release_reclaim_holds();
}
