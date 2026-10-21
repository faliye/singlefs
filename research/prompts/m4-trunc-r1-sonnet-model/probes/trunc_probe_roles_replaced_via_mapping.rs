//! 草稿探针（m4-trunc-r1 正推腿，不入库）：只重写 Data(0) 的发布（截到一个单元以内的尾单元重写）在今天的换下清单里会不会连带把后面的单元也算作换下。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::build_pool;
use common::sequential_write_in_process_as_instance_one_and_advance_the_pool;
use common_deletion_in_progress::content_of_three_data_units;
use singlefs_core::address::DataUnitIndexInFile;
use singlefs_core::transaction::{probe_roles_replaced_via_mapping, TransactionUnit};

#[test]
fn probe_a_publish_that_rewrites_only_the_unit_at_a_given_index() {
    let mut pool = build_pool("trunc-probe-roles");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    let previous = &pool.output;
    println!("上一版数据单元数 = {}", previous.data_pointers.len());
    for index in [0u64, 1, 2] {
        let roles = [TransactionUnit::Data(DataUnitIndexInFile(index))];
        println!(
            "只重写 Data({index}) -> 换下清单 {:?}",
            probe_roles_replaced_via_mapping(previous, &roles)
        );
    }
}
