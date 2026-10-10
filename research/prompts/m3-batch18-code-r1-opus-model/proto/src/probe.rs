//! 估时：一段删除历史的录制流切段、闭式状态数，单个镜像跑一遍池级 checker 要多久。
use std::time::Instant;

use singlefs_checker::walk::check_pool_image;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes};

use crate::common::{file_content, geometry, FIXED_WRITE_TIME_SECONDS};
use crate::pool::SparsePool;

pub fn run() {
    let started = Instant::now();
    let mut pool = SparsePool::build(4 << 30, &file_content());
    let payload = data_unit_payload_capacity();
    pool.sequential_write(&crate::pool::content_of(2 * payload + payload / 2), FIXED_WRITE_TIME_SECONDS + 10);
    let write_time = pool.output.inode_record.expect("记录").write_time_seconds;
    pool.intent(1, write_time).expect("意图");
    pool.steps_until_empty(1);
    println!("PROBE build_and_publish_elapsed={:?} publishes={}", started.elapsed(), pool.published_images.len());
    let operations = pool.stream.retained_operations();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("PROBE writes={} segments={} segment_lengths={lengths:?} closed_form_states={}", writes.len(), segments.len(), closed_form_state_count(&segments));
    let image = pool.image();
    let started = Instant::now();
    let rounds = 20;
    for _ in 0..rounds {
        let report = check_pool_image(&image);
        assert!(!report.is_empty());
    }
    println!("PROBE check_pool_image_mean={:?}", started.elapsed() / rounds);
}
