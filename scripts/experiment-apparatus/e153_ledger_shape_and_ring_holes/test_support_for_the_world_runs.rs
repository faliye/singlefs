//! 几份测试共用的辅助：建出新建池的世界、待回收记录的构造。

use super::*;

pub(super) fn new_pool_file_creation_world(geometry: GeometryCell) -> World {
    let mut world = World::mkfs(geometry);
    for _ in world.first_mount_warmup() {
    }
    let _ = world.publish(PublishKind::Overwrite);
    world
}

pub(super) fn pending_record(slot: u64, allocation_generation: u64, release_generation: u64) -> PendingRecord {
    PendingRecord { slot, span: 1, allocation_generation, release_generation }
}
