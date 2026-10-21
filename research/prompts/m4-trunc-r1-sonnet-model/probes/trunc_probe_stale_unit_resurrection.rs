//! 草稿探针（m4-trunc-r1 正推腿，不入库）：缩短之后"只改 inode 记录 size 的变长"会不会让扫描重建把缩短时放掉的单元认回来。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use common_deletion_in_progress::{change_the_inode_record, content_of_three_data_units};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::unit::data_unit_payload_capacity;

fn print_the_verdicts(label: &str, image: &singlefs_harness::memory_pool::MemoryPool) {
    for (invariant, verdict) in check_pool_image(image) {
        match &verdict {
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
            other => println!("{label}: {invariant:?} -> {other:?}"),
        }
        if invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX {
            println!("{label}: I-1.13 -> {verdict:?}");
        }
    }
}

#[test]
fn probe_record_only_extension_after_a_shorter_overwrite() {
    let mut pool = build_pool("trunc-probe");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
    let mut image = pool.memory_pool();
    print_the_verdicts("A 缩短之后（今天能走到的镜像）", &image);
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷");
    change_the_inode_record(&mut image, 1, |record| {
        let size = 3 * payload;
        record[40..48].copy_from_slice(&size.to_le_bytes());
        record[48..56].copy_from_slice(&size.div_ceil(512).to_le_bytes());
    });
    print_the_verdicts("B 再把记录的 size 改成三个净荷（模拟只改记录的变长）", &image);
}
