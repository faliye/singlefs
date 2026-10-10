//! K4（m3-batch57-code-r1 opus）：三片容器、第一片装满旋钮值 2、中间那片只 1 条（写的时候中途把旋钮从 2 换成 1：[2, 1, 1]），
//! 带旋钮 2 交池级 checker 新入口。第 57 批 H1 的判法应当在中间那片上红（不是最右的容器 2 装了 1 条，不是旋钮值 2）；
//! 第 57 批的会红用例全是两片的形状，「只判第一片」那种改坏在它们上面照绿、在这一张上才分得出。三棵树各一张。
mod common_b57;

use common_b57::{violations, verdict_name, Knobs, Pool};
use singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs;
use singlefs_format::INODE_LEAF_RECORDS;

fn judged_with_the_knob_two(pool: &Pool<singlefs_harness::memory_pool::SparseBlockDevice>, tree: &str, knobs: Knobs) {
    let verdicts = check_pool_image_given_the_container_capacity_knobs(&pool.image(), knobs.checker_knobs());
    eprintln!(
        "OPUS_K4M tree={tree} {} judged_with={knobs:?} I-9.22={} I-9.37={} violations={:?}",
        pool.tree_summary(),
        verdict_name(&verdicts, "I-9.22"),
        verdict_name(&verdicts, "I-9.37"),
        violations(&verdicts)
    );
}

#[test]
fn opus_probe_b57_middle_short_container_under_the_knob_two() {
    let base = Pool::after_the_first_file();
    let knob = |tombstone, lineage, retirement| Knobs { tombstone, lineage, retirement, node_entries: 0 };

    // 墓碑树：旋钮 2 下删两个号（一片 2 条），换旋钮 1 再删两个（各 1 条）。
    let mut pool = base.clone();
    pool.create(8);
    for (inode, written_with) in [(2, 2), (3, 2), (4, 1), (5, 1)] {
        pool.allocator.set_deletion_switches(knob(written_with, 0, 0).switches());
        pool.unlink(inode, knob(written_with, 0, 0));
    }
    judged_with_the_knob_two(&pool, "tombstone", knob(2, 0, 0));

    // 谱系树：旋钮 2 下回退两次，换旋钮 1 再回退两次（每次退到紧前那一版）。
    let mut pool = base.clone();
    pool.create(1);
    let mut previous = (pool.instance, pool.txg());
    pool.create(1);
    for written_with in [2, 2, 1, 1] {
        pool.allocator.set_deletion_switches(knob(0, written_with, 0).switches());
        let target = previous;
        previous = (pool.instance, pool.txg());
        pool.roll_back(target).expect("回退");
    }
    judged_with_the_knob_two(&pool, "lineage", knob(0, 2, 0));

    // 退役树：建满一片叶之后每次建一个号（开新叶）再删到底（那片叶删空、退役一条）。
    let mut pool = base.clone();
    pool.create(u64::from(INODE_LEAF_RECORDS) - 1);
    for written_with in [2, 2, 1, 1] {
        pool.allocator.set_deletion_switches(knob(0, 0, written_with).switches());
        pool.create(1);
        let largest = *pool.live_inodes().iter().max().expect("有活号");
        pool.unlink(largest, knob(0, 0, written_with));
    }
    judged_with_the_knob_two(&pool, "retirement", knob(0, 0, 2));
}
