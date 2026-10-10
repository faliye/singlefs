//! 腿自己的探针（只放进草稿副本编）：C551 新用例的同一段历史（build_pool 之后连续覆盖写到 txg 30），每一次发布之后都对当时的镜像跑池级 checker，
//! 记哪一个 txg 起有 Violated、哪几条不变量。基线应当一条都没有；变异 d（环转之前提前回收释放代 3）下看红不红。
mod common;
use common::{build_pool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InstanceGeneration;

#[test]
fn per_txg_checker_over_the_history_of_the_c551_test() {
    let mut pool = build_pool("zz-per-txg-checker");
    let mut first_violation: Option<(u64, Vec<String>)> = None;
    let mut index = 0usize;
    while pool.output.root.checkpoint_txg.0 < 30 {
        let previous = pool.output.clone();
        let content: Vec<u8> = (0..3000 + index * 37)
            .map(|i| u8::try_from((i * 11 + index * 5 + 1) % 251).unwrap())
            .collect();
        pool.output = common::publish_overwrite_in_process(&mut pool, &previous, &content, FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1)).unwrap();
        index += 1;
        let violated: Vec<String> = check_pool_image(&pool.memory_pool())
            .into_iter()
            .filter_map(|(name, verdict)| match verdict {
                InvariantVerdict::Violated(_) => Some(name.to_string()),
                _ => None,
            })
            .collect();
        if !violated.is_empty() && first_violation.is_none() {
            first_violation = Some((pool.output.root.checkpoint_txg.0, violated));
        }
    }
    println!("PERTXG first_violation={first_violation:?}");
    assert!(first_violation.is_none(), "{first_violation:?}");
}
