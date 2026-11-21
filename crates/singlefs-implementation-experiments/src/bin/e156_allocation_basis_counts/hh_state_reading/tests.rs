//! 岔路 1 的族 Hh：一版的事实、从盘上读的引用槽、门槛、洞数与逐盘多扣的槽，与一个状态的整份读数的单测。

use super::{
    reclaim_threshold_read_from_the_ring_image, valid_root_txgs_without_the_old_root_in_the_slot,
};
use std::collections::BTreeSet;

/// PC-同槽（第 7 次重跑登记 5.3、E25）：把洞的环槽当作这个进程写过之后，那一槽里的旧根不再算数，门槛升到下一条根。M54（改法变成不改）在这里红。
#[test]
fn treating_the_hole_slot_as_rewritten_raises_the_threshold_to_the_next_root() {
    let valid_roots = BTreeSet::from([4u64, 5, 6]);
    assert_eq!(
        reclaim_threshold_read_from_the_ring_image(0, &valid_roots),
        4,
        "不改：门槛钉在 txg 4 的旧根上"
    );
    let without_the_old_root =
        valid_root_txgs_without_the_old_root_in_the_slot(&valid_roots, Some(4));
    assert_eq!(
        reclaim_threshold_read_from_the_ring_image(0, &without_the_old_root),
        5,
        "改：门槛升到下一条根"
    );
    assert_eq!(
        valid_root_txgs_without_the_old_root_in_the_slot(&valid_roots, None),
        valid_roots,
        "那一槽里没有旧根：什么都不去掉"
    );
}
