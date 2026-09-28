// 样本：只留探针要数的那几行，别的都不是这道阶段看的东西。
fn isolate_slots_referenced_only_by_abandoned_roots() {
    let key = (device.0, slot.0);
}

fn rebuilt_allocator() {
    allocator.reclaim_released_up_to(floor, ReclaimedReuse::Immediately);
}

fn raise_rollback_floor() {
    let reclaimed = allocator.reclaim_released_up_to(floor, ReclaimedReuse::HeldUntilFloorTakesEffect);
    allocator.release_reclaim_holds();
}
