//! b30 攻方腿 M1 零轮改法 B3 的样子（只在攻方副本上量过、被攻过零轮）：胶水把实现交回的叶容器号映到模型角色这一句，
//! 一个根装得下的 135 片逐片映到互不相同的角色、第 k 片映到模型落点函数的第 k 片。不跑历史，封顶在任何一片的胶水变体都红。

use std::collections::BTreeSet;

use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::transaction::TransactionUnit;
use singlefs_harness::model::ModelUnitRole;
use singlefs_harness::model_comparison::model_unit_role;

const LEAF_CONTAINERS_ONE_ROOT_HOLDS: u32 = 135;

#[test]
fn every_inode_leaf_container_index_one_root_holds_maps_to_its_own_model_role() {
    let mut seen = BTreeSet::new();
    for index in 0..LEAF_CONTAINERS_ONE_ROOT_HOLDS {
        let role = model_unit_role(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree(index),
        ))
        .expect("叶容器有角色");
        assert_eq!(
            role,
            ModelUnitRole::inode_leaf_container_at(u64::from(index)),
            "第 {index} 片映到模型的第 {index} 片"
        );
        assert!(seen.insert(role), "第 {index} 片与前面某一片映到同一个角色");
    }
}
