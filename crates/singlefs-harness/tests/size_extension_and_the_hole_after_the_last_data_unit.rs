//! 变长与尾部的洞（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5，用户 2026-10-16 定「加标签声明洞」）：变长一次发布、
//! 不立意图；最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写；之后到 N 是洞，上段叶条目写标签 3（下段根）或 4（内联单元 0）；
//! 读者读洞答 0、不读盘。洞全被截掉、或整份覆盖写之后标签换回 1 或 2。每条用例钉标签与两条读路径读出的字节。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{
    build_pool, file_content, sequential_write_in_process_as_instance_one_and_advance_the_pool,
};
use common_deletion_in_progress::{
    content_of_three_data_units, pool_with_a_first_file_of_size_zero_and_one_data_unit,
    pool_with_a_three_unit_file,
};
use common_truncation::{
    assert_both_readers_read, batches_of_one, change_the_size_of_the_first_file_and_advance,
    payload_capacity, truncation_entry_of, upper_leaf_tag_of_the_first_file,
    walk_the_queues_and_advance,
};
use singlefs_core::address::DataUnitIndexInFile;
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::parse_data_unit;

/// `content` 补零到 `size` 字节。
fn padded_with_zeros(content: &[u8], size: u64) -> Vec<u8> {
    let mut padded = content.to_vec();
    padded.resize(usize::try_from(size).expect("长度"), 0);
    padded
}

fn declared_length_of(version: &TransactionOutput, unit: u64) -> u16 {
    parse_data_unit(
        &version
            .unit(TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .bytes,
    )
    .expect("数据单元解得开")
    .declared_length
}

/// 变长到单元内（3000 → 5000）：单元 0 补零重写到 5000，没有洞，标签 2；不签截断条目、之后走空一个写都不发。
#[test]
fn extending_within_the_only_data_unit_zero_fills_it_and_keeps_tag_two() {
    let mut pool = build_pool("extension-within-the-unit");
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, 5000);
    assert_eq!(
        (
            extended.data_pointers.len(),
            declared_length_of(&extended, 0),
            upper_leaf_tag_of_the_first_file(&extended),
        ),
        (1, 5000, 2)
    );
    assert!(truncation_entry_of(&extended).is_none(), "变长不立意图");
    assert!(walk_the_queues_and_advance(&mut pool, batches_of_one()).is_empty());
    assert_both_readers_read(
        &pool.memory_pool(),
        &extended.root,
        &padded_with_zeros(&file_content(), 5000),
        "变长到单元内",
    );
}

/// 一个单元的文件变长跨单元（3000 → 3 × 净荷 + 5）：单元 0 补零到满一个净荷，之后到 size 是洞，内联、标签 4；读出原内容加零。
#[test]
fn extending_a_one_unit_file_across_data_units_fills_its_unit_and_declares_the_hole_with_tag_four()
{
    let mut pool = build_pool("extension-across-units-inline");
    let new_size = 3 * payload_capacity() + 5;
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    assert_eq!(
        (
            extended.data_pointers.len(),
            u64::from(declared_length_of(&extended, 0)),
            upper_leaf_tag_of_the_first_file(&extended),
        ),
        (1, payload_capacity(), 4)
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &extended.root,
        &padded_with_zeros(&file_content(), new_size),
        "变长跨单元",
    );
}

/// 三个单元的文件变长跨单元（2.5 × 净荷 → 5 × 净荷 + 1）：单元 2 补零到满，下段之后到 size 是洞，标签 3。
#[test]
fn extending_a_three_unit_file_past_its_last_unit_declares_the_hole_with_tag_three() {
    let mut pool = pool_with_a_three_unit_file("extension-across-units-lower-segment");
    let new_size = 5 * payload_capacity() + 1;
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    assert_eq!(
        (
            extended.data_pointers.len(),
            u64::from(declared_length_of(&extended, 2)),
            upper_leaf_tag_of_the_first_file(&extended),
        ),
        (3, payload_capacity(), 3)
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &extended.root,
        &padded_with_zeros(&content_of_three_data_units(), new_size),
        "三单元文件变长跨单元",
    );
}

/// 从 size 0 的空文件变长（0 → 净荷 + 10）：声明长度 0 的单元 0 补零重写到满一个净荷，之后是洞，标签 4；读出全是零。
#[test]
fn extending_the_first_file_from_size_zero_zero_fills_unit_zero_and_reads_zeros() {
    let mut pool =
        pool_with_a_first_file_of_size_zero_and_one_data_unit("extension-from-size-zero");
    let new_size = payload_capacity() + 10;
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    assert_eq!(
        (
            u64::from(declared_length_of(&extended, 0)),
            upper_leaf_tag_of_the_first_file(&extended),
        ),
        (payload_capacity(), 4)
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &extended.root,
        &padded_with_zeros(&[], new_size),
        "从 size 0 变长",
    );
}

/// 变长之后再缩回洞里（3 × 净荷 + 5 → 2 × 净荷 + 1）：没有单元可放、单元 1 不存在不重写，意图与放 0 个的最后一批照走，标签仍是 4；
/// 再截到 100：单元 0 重写成声明长度 100，洞全被截掉，标签换回 2。
#[test]
fn truncating_back_into_the_hole_keeps_tag_four_until_the_hole_is_cut_away() {
    let mut pool = build_pool("truncation-back-into-the-hole");
    change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity() + 5);
    let into_the_hole = 2 * payload_capacity() + 1;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, into_the_hole);
    assert!(truncation_entry_of(&intent).is_some(), "缩短照走意图");
    let steps = walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(steps.len(), 1, "一个放 0 个单元的最后一批");
    assert_eq!(
        (
            pool.output.data_pointers.len(),
            upper_leaf_tag_of_the_first_file(&pool.output)
        ),
        (1, 4)
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &padded_with_zeros(&file_content(), into_the_hole),
        "缩回洞里",
    );
    let cut_away = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    walk_the_queues_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        (
            declared_length_of(&cut_away, 0),
            upper_leaf_tag_of_the_first_file(&pool.output)
        ),
        (100, 2),
        "洞全被截掉，标签换回 2"
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &pool.output.root,
        &file_content()[..100],
        "洞截掉之后",
    );
}

/// 变长之后整份覆盖写（3000 → 3 × 净荷，再顺序写 3 × 净荷 字节）：洞写成真字节的单元，三个单元、标签 1。
#[test]
fn a_whole_overwrite_after_an_extension_writes_the_hole_as_data_units_and_returns_to_tag_one() {
    let mut pool = build_pool("whole-overwrite-after-an-extension");
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity());
    assert_eq!(upper_leaf_tag_of_the_first_file(&extended), 4);
    let content: Vec<u8> = (0..3 * payload_capacity())
        .map(|index| u8::try_from((index * 11 + 1) % 241).expect("小于 256"))
        .collect();
    let overwritten =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(
        (
            overwritten.data_pointers.len(),
            upper_leaf_tag_of_the_first_file(&overwritten)
        ),
        (3, 1)
    );
    assert_both_readers_read(
        &pool.memory_pool(),
        &overwritten.root,
        &content,
        "整份覆盖写之后",
    );
}
