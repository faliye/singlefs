//! K2-2（m3-batch56-code-r1 opus）：旋钮下「非最右那片不满」。写侧在旋钮 K 下满 K 条才关（D18（块里携带什么信息） 已定项 24 第 3 条
//! 「满了才关；关闭路径加只供测试的容量旋钮」），所以旋钮 K ≥ 2 的写侧写不出「不是最右的那片只装 1 条」。这里拿旋钮 1 写出三棵树各几片、
//! 每片 1 条的镜像，交带旋钮 2 的新入口：看 I-9.22、I-9.37 红不红。同一份镜像带旋钮 1 交新入口是合法对照。
mod common_b56;

use common_b56::{verdict_name, violations, Knobs, Pool};
use singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs;
use singlefs_core::address::InstanceGeneration;

#[test]
fn opus_probe_b56_knob_two_judges_containers_closed_after_one_record() {
    let written_under = Knobs { tombstone: 1, lineage: 1, retirement: 1, node_entries: 0 };
    let mut pool = Pool::after_the_first_file();
    pool.allocator.set_deletion_switches(written_under.switches());
    pool.create(234);
    let after_create = pool.txg();
    pool.unlink(235, written_under);
    pool.unlink(234, written_under);
    pool.create(1);
    pool.unlink(236, written_under);
    pool.roll_back((InstanceGeneration(1), after_create)).expect("回退一");
    pool.roll_back((InstanceGeneration(1), after_create)).expect("回退二");
    let image = pool.image();
    eprintln!("OPUS_K2K trees {}", pool.tree_summary());
    for (name, knob) in [("knob1", 1u64), ("knob2", 2), ("knob3", 3)] {
        let judged_with = Knobs { tombstone: knob, lineage: knob, retirement: knob, node_entries: 0 };
        let verdicts = check_pool_image_given_the_container_capacity_knobs(&image, judged_with.checker_knobs());
        eprintln!(
            "OPUS_K2K judged_with={name} I-9.22={} I-9.37={} violations={:?}",
            verdict_name(&verdicts, "I-9.22"),
            verdict_name(&verdicts, "I-9.37"),
            violations(&verdicts)
        );
    }
}
