//! 攻方改法 G7（只在攻方副本上量过、被攻过零轮）：产品批大小在任意大的文件上都是一片下段叶（144 个单元），纯算、不建池。
//! 有限几个建池的点罩不住「批数封顶 N」那一族（N ≥ 32 时 4608 个单元以内与产品逐项相同），这里在 2^40 个单元上也比。
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{next_unlink_step, UnlinkBatchSize, UnlinkStep};
use singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

#[test]
fn the_product_batch_releases_one_extent_lower_leaf_however_large_the_file_is() {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("32634");
    for remaining in [145_u64, 433, 4_609, 1_000_000, 1 << 40] {
        let kept = remaining - EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
        assert_eq!(
            next_unlink_step(remaining, UnlinkBatchSize::OneExtentLowerLeaf),
            UnlinkStep::Batch { data_units_kept: kept, size_after_the_batch_in_bytes: kept * payload },
            "剩 {remaining} 个单元时这一批换下恰好 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个"
        );
    }
}
