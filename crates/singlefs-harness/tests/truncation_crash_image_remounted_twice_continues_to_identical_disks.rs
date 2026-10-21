//! 截断的续做幂等（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2；D8（核心索引结构） 已定项 16 第 7 条 ①b，截断照它的幂等纪律）：
//! 截断意图刚发完那一刻的镜像、意图写到根槽之前（单元 0 与记录落了、根槽没落）的崩溃镜像，各自拷两份，各自可写挂载一次、挂载续做
//! 按批大小 1 把截断走到底，两份盘逐字节相同，续做之后截断条目没了、第一个文件只剩 size 之内的那一个单元。三个单元的第一个文件截到 100 字节。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters};
use common_deletion_in_progress::pool_with_a_three_unit_file;
use common_truncation::{
    change_the_size_of_the_first_file_and_advance, deletion_switches_with_batches_of_one,
    truncation_entry_of,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger,
};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 一份崩溃镜像（`base` 叠上 `writes` 里 `persisted` 的那几条）在自己的两块内存盘上可写挂载一次、按批大小 1 续做，交回挂完之后的整份盘。
///
/// # Panics
/// 挂不上、或挂完那一版截断没走到底（还有截断条目、或单元不是一个）。
fn remount_a_copy_and_continue_the_truncation(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> MemoryPool {
    let stream = SharedStream::new();
    let mut devices = crash_state_devices(base, writes, persisted, &stream);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        deletion_switches_with_batches_of_one(),
    )
    .expect("这一份镜像可写挂载、续做");
    let PoolVersion::WithFile(current) = &mounted.current else {
        panic!("截断之后仍带文件");
    };
    assert!(
        truncation_entry_of(current).is_none() && current.data_pointers.len() == 1,
        "挂载续做把截断走到底：条目删了、只剩一个单元"
    );
    memory_pool_of_sparse_devices(&devices)
}

#[test]
fn an_image_right_after_the_truncation_intent_remounted_twice_continues_to_byte_identical_disks() {
    let mut pool = pool_with_a_three_unit_file("truncation-intent-image-remounted-twice");
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    let image_right_after_the_intent = pool.memory_pool();
    let first_copy =
        remount_a_copy_and_continue_the_truncation(&image_right_after_the_intent, &[], &[]);
    let second_copy =
        remount_a_copy_and_continue_the_truncation(&image_right_after_the_intent, &[], &[]);
    assert!(
        first_copy == second_copy,
        "截断意图刚发完的镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
    );
    assert!(first_copy != image_right_after_the_intent, "续做真的写了盘");
}

/// 意图写到根槽之前的崩溃镜像：意图那次的单元写（重写的单元 0 在内）与 journal 记录都落了，根槽与系统配置轮换没落。两份各自挂一次：
/// 恢复由那一条记录施加出意图那一版（D23（journal 的角色与格式） 已定项 15），再续做两批，两份盘逐字节相同。
#[test]
fn a_crash_image_with_the_truncation_intent_written_up_to_its_root_remounted_twice_continues_to_byte_identical_disks(
) {
    let mut pool = pool_with_a_three_unit_file("truncation-intent-crash-image-remounted-twice");
    let image_before_the_intent = pool.memory_pool();
    let operations_before_the_intent = pool.retained_operations().len();
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    let operations = pool.retained_operations();
    let (writes_of_the_intent, _segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &operations[operations_before_the_intent..],
            &geometry(),
        );
    let root_write = writes_of_the_intent
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("意图那次有一条根槽写");
    assert!(
        writes_of_the_intent[..root_write]
            .iter()
            .any(|write| write.kind == StepKind::JournalRecord)
            && writes_of_the_intent[..root_write]
                .iter()
                .any(|write| write.kind == StepKind::UnitWrite),
        "根槽写之前意图那次的单元与记录都写了"
    );
    let persisted_up_to_the_root: Vec<bool> = (0..writes_of_the_intent.len())
        .map(|write_index| write_index < root_write)
        .collect();
    let first_copy = remount_a_copy_and_continue_the_truncation(
        &image_before_the_intent,
        &writes_of_the_intent,
        &persisted_up_to_the_root,
    );
    let second_copy = remount_a_copy_and_continue_the_truncation(
        &image_before_the_intent,
        &writes_of_the_intent,
        &persisted_up_to_the_root,
    );
    assert!(
        first_copy == second_copy,
        "意图写到根槽之前的崩溃镜像各自重挂载续做，两份盘逐字节相同（D8 已定项 16 第 7 条 ①b）"
    );
}
