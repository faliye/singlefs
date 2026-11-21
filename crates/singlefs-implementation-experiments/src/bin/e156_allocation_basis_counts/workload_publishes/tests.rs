//! 空发布与工作负载发布，与一次发布自己释放的槽的单测。

use super::new_pool_file_creation_state;
use super::self_release_slots_of_this_publish;
use singlefs_core::address::DeviceIdentity;

/// U7：新池新建文件那一次发布的 R8「自己的释放」= 1 槽（mkfs 的树表单元）。
#[test]
fn new_pool_file_creation_self_release_is_one_slot() {
    let (allocator, first, _devices) = new_pool_file_creation_state();
    let released = self_release_slots_of_this_publish(
        &allocator,
        DeviceIdentity(0),
        first.root.checkpoint_txg,
    );
    assert_eq!(
        released, 1,
        "U7：新池新建文件自己的释放应为 1 槽（mkfs 的树表单元）"
    );
}
