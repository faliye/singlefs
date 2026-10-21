//! m4-trunc-r1 攻方原型（只在草稿目录的仓副本里）：用镜像改字节模拟「只改 size」的变长（W5 甲）与「意图那次就写 size = N、尾部单元逐批放」
//! 的截断中间态（W2 甲），看池级 checker 的 I-1.13（扫描重建与遍历方向的索引相同）判什么。
//! 环境变量 M4_TRUNC_R1_Z_UNCLAIMED 设了时，副本里的 `rebuild.rs` 把「size 之外的数据单元」从报不适用改成不认领（读法 Z 落成不认领）。

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_admission::start_plain;
use common_deletion_in_progress::change_the_inode_record;
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::MemoryPool;

fn set_the_size_of_the_first_file(image: &mut MemoryPool, size: u64) {
    change_the_inode_record(image, 1, |record| {
        record[40..48].copy_from_slice(&size.to_le_bytes());
        record[48..56].copy_from_slice(&size.div_ceil(512).to_le_bytes());
    });
}

fn report(case: &str, image: &MemoryPool) -> InvariantVerdict {
    let verdicts = check_pool_image(image);
    let mut scan = None;
    for (invariant, verdict) in &verdicts {
        if *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX {
            scan = Some(verdict.clone());
        }
        if matches!(verdict, InvariantVerdict::Violated(_)) {
            println!("M4TRUNC case={case} violated {invariant}: {verdict:?}");
        }
    }
    let scan = scan.expect("I-1.13 在清单里");
    println!("M4TRUNC case={case} I-1.13 = {scan:?}");
    println!(
        "M4TRUNC case={case} violated_count = {}",
        verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .count()
    );
    scan
}

fn payload() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("净荷")
}

/// W5 甲 × 读法 Z：三个单元的文件覆盖写成一个单元（后两个单元已放、盘上还在、同一对象出生代），再把 size 改大到三个单元（只改 size 的变长）。
#[test]
fn extend_by_size_only_after_a_shorter_overwrite() {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let p = payload();
    pool.sequential_write(usize::try_from(2 * p + p / 2).expect("长度"))
        .expect("顺序写三个单元");
    pool.overwrite(1000).expect("覆盖写成一个单元");
    let mut image = pool.image();
    report("shorter-overwrite-before-extend", &image);
    set_the_size_of_the_first_file(&mut image, 3 * p - 1);
    report("extend-after-shorter-overwrite", &image);
}

/// 对照：没有留下的旧单元时同样只改 size 变长。
#[test]
fn extend_by_size_only_without_left_units() {
    let pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let mut image = pool.image();
    report("first-file-before-extend", &image);
    set_the_size_of_the_first_file(&mut image, 3 * payload() - 1);
    report("extend-without-left-units", &image);
}

/// W2 甲 的截断中间态：三个单元的文件，size 写成恰好一个单元（截断点落在单元边界，不重写尾单元），后两个单元还挂在 extent 树里。
#[test]
fn truncation_intermediate_with_the_target_size_written_at_the_intent() {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let p = payload();
    pool.sequential_write(usize::try_from(2 * p + p / 2).expect("长度"))
        .expect("顺序写三个单元");
    let mut image = pool.image();
    report("three-units-before-truncation", &image);
    set_the_size_of_the_first_file(&mut image, p);
    report("truncation-intermediate-size-written-at-intent", &image);
}
