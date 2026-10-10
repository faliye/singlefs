//! K2（m3-batch58-code-r1 opus）：被判 J4 那条流（产品容量下墓碑树 [815, 1]）按原造法（mkfs 之后整条录制流进写表，第 816 个 unlink 之前的段不展开）
//! 与新造法（第 816 个 unlink 之前的录制流施加成起点镜像，写表只录那一次 unlink）各建一份层 0 枚举表，逐项比：状态数、每个展开段的写与段长、
//! 每次写取几态（原地覆写的第三态）、撕裂镜像；再按新造法建 GPU 判表（不上卡），报表的大小与落点数，与原造法建表的拒绝。
//! 不改号（`renumber_the_last_container_of_the_flow` 不调：它只改第 816 个 unlink 里那几次写的字节，不改写的个数、落点与切段）。
mod flows_b58;

use flows_b58::{geometry, SparsePool, FIRST_FILE};
use singlefs_checker_tier::crash::{layer0_enumeration_tables, Layer0SegmentExpansion};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables_against_the_version_table, JudgeFlow, JudgeTablesWords};
use singlefs_core::container_tree::ContainerTreeCapacities;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes};
use singlefs_harness::segments::StepKind;

const EXPAND_UP_TO: usize = 8;

fn words_of(words: &JudgeTablesWords) -> (usize, usize) {
    let total: usize = words.every_table().iter().map(|table| table.len()).sum();
    (total, words.version_bytes.len())
}

#[test]
fn opus_probe_b58_k2_j4_state_sets_of_the_two_constructions() {
    let started = std::time::Instant::now();
    let filling = singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
    let mut pool = SparsePool::with_new_inodes(filling + 1);
    let mut first_txg_past_1024_records: Option<u64> = None;
    for inode in FIRST_FILE + 1..=FIRST_FILE + filling {
        pool.unlink_to_the_end_under(inode, ContainerTreeCapacities::FROM_THE_FORMAT);
        let records = pool.output.allocation_records.len();
        if records > 1024 && first_txg_past_1024_records.is_none() {
            first_txg_past_1024_records = Some(pool.output.root.checkpoint_txg.0);
        }
        if (inode - FIRST_FILE) % 100 == 0 {
            eprintln!(
                "OPUS_K2 growth unlinked={} txg={} allocation_records={} released={}",
                inode - FIRST_FILE,
                pool.output.root.checkpoint_txg.0,
                records,
                pool.output.allocation_records.iter().filter(|record| record.is_released).count()
            );
        }
    }
    eprintln!("OPUS_K2 growth first_txg_with_more_than_1024_allocation_records={first_txg_past_1024_records:?}");
    let operations_before_the_last_unlink = pool.stream.operation_count();
    eprintln!(
        "OPUS_K2 version-before-the-last-unlink txg={} allocation_records={} released={} mapping_keys={} accounting_entries={}",
        pool.output.root.checkpoint_txg.0,
        pool.output.allocation_records.len(),
        pool.output.allocation_records.iter().filter(|record| record.is_released).count(),
        pool.output.mapping_keys.len(),
        pool.output.accounting_entries.len()
    );
    pool.unlink_to_the_end_under(FIRST_FILE + filling + 1, ContainerTreeCapacities::FROM_THE_FORMAT);
    let counts: Vec<usize> = pool.output.tombstone_containers.iter().map(|c| c.contents.records.len()).collect();
    assert_eq!(counts, [815, 1]);
    let versions = pool.versions_with_the_first_file_through_the_last_publish();
    let operations = pool.retained_operations();
    eprintln!(
        "OPUS_K2 pool operations={} mkfs_operations={} operations_before_the_last_unlink={} versions={} last_txg={} built_in_s={}",
        operations.len(),
        pool.mkfs_operation_count,
        operations_before_the_last_unlink,
        versions.len(),
        pool.output.root.checkpoint_txg.0,
        started.elapsed().as_secs()
    );
    eprintln!(
        "OPUS_K2 last-version allocation_records={} released={} mapping_keys={} accounting_entries={} inode_leaf_containers={} inode_records={} tombstone_containers={:?}",
        pool.output.allocation_records.len(),
        pool.output.allocation_records.iter().filter(|record| record.is_released).count(),
        pool.output.mapping_keys.len(),
        pool.output.accounting_entries.len(),
        pool.output.inode_leaf_containers.len(),
        pool.output.inode_leaf_containers.iter().map(|leaf| leaf.contents.records.len()).sum::<usize>(),
        counts
    );

    // 原造法
    let base_old = pool.memory_pool_after_mkfs();
    let (writes_old, segments_old, stream_indexes_old) =
        writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let boundary_relative = operations_before_the_last_unlink - pool.mkfs_operation_count;
    let first_write_of_the_last_unlink = stream_indexes_old
        .iter()
        .position(|stream_index| *stream_index >= boundary_relative)
        .expect("第 816 个 unlink 有写");
    let expansion_old = move |_index: usize, segment: &[usize]| {
        let of_the_last_unlink = segment.first().is_some_and(|first| *first >= first_write_of_the_last_unlink);
        if of_the_last_unlink && segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let straddling: Vec<usize> = segments_old
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            segment.first().is_some_and(|first| *first < first_write_of_the_last_unlink)
                && segment.last().is_some_and(|last| *last >= first_write_of_the_last_unlink)
        })
        .map(|(index, _)| index)
        .collect();
    eprintln!(
        "OPUS_K2 old writes={} segments={} first_write_of_the_last_unlink={} segments_straddling_the_boundary={straddling:?}",
        writes_old.len(),
        segments_old.len(),
        first_write_of_the_last_unlink
    );
    let tables_started = std::time::Instant::now();
    let tables_old = layer0_enumeration_tables(&base_old, &writes_old, &segments_old, &expansion_old);
    eprintln!(
        "OPUS_K2 old-tables states={} enumerated_writes={} torn_images={} built_in_s={}",
        tables_old.state_count,
        tables_old.enumerated_writes.len(),
        tables_old.torn_images.len(),
        tables_started.elapsed().as_secs()
    );

    // 新造法（被判那一份）
    let mut base_new = pool.memory_pool_after_mkfs();
    base_new.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);
    let (writes_new, segments_new, _) =
        writes_and_segments_with_stream_indexes(&operations[operations_before_the_last_unlink..], &geometry());
    let expansion_new = |_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let tables_new = layer0_enumeration_tables(&base_new, &writes_new, &segments_new, &expansion_new);
    let longest_new = segments_new.iter().map(Vec::len).max().unwrap_or(0);
    let closed_form_new = if longest_new < 60 { format!("{}", closed_form_state_count(&segments_new)) } else { format!(">2^{longest_new}") };
    eprintln!(
        "OPUS_K2 new writes={} segments={} segment_lengths={:?} closed_form_full={closed_form_new} states={} enumerated_writes={} torn_images={}",
        writes_new.len(),
        segments_new.len(),
        segments_new.iter().map(Vec::len).collect::<Vec<_>>(),
        tables_new.state_count,
        tables_new.enumerated_writes.len(),
        tables_new.torn_images.len()
    );

    // 逐项比：原造法里状态不空的段 ↔ 新造法的段（写下标减去分界）
    let offset = first_write_of_the_last_unlink;
    let old_tail_segments: Vec<Vec<usize>> = segments_old
        .iter()
        .filter(|segment| segment.first().is_some_and(|first| *first >= offset))
        .map(|segment| segment.iter().map(|index| index - offset).collect())
        .collect();
    let same_segments = old_tail_segments == segments_new;
    let same_writes = writes_old[offset..] == writes_new[..];
    let tearable_old_tail: Vec<bool> = tables_old.is_tearable_by_write[offset..writes_old.len()].to_vec();
    let tearable_new: Vec<bool> = tables_new.is_tearable_by_write[..writes_new.len()].to_vec();
    let tearable_differences: Vec<usize> = tearable_old_tail
        .iter()
        .zip(&tearable_new)
        .enumerate()
        .filter(|(_, (old, new))| old != new)
        .map(|(index, _)| index)
        .collect();
    let old_ranges: Vec<u64> = tables_old
        .state_ranges_by_segment
        .iter()
        .zip(&segments_old)
        .filter(|(_, segment)| segment.first().is_some_and(|first| *first >= offset))
        .map(|(range, _)| range.end - range.start)
        .collect();
    let new_ranges: Vec<u64> = tables_new.state_ranges_by_segment.iter().map(|range| range.end - range.start).collect();
    let old_states_before_the_boundary: u64 = tables_old
        .state_ranges_by_segment
        .iter()
        .zip(&segments_old)
        .filter(|(_, segment)| !segment.first().is_some_and(|first| *first >= offset))
        .map(|(range, _)| range.end - range.start)
        .sum();
    let torn_old: Vec<(usize, &singlefs_harness::memory_pool::RetainedWrite, usize)> = tables_old
        .torn_images
        .iter()
        .filter(|row| row.write_index >= offset)
        .map(|row| (row.write_index - offset, &tables_old.enumerated_writes[row.torn_image_index], row.replays.len()))
        .collect();
    let torn_new: Vec<(usize, &singlefs_harness::memory_pool::RetainedWrite, usize)> = tables_new
        .torn_images
        .iter()
        .map(|row| (row.write_index, &tables_new.enumerated_writes[row.torn_image_index], row.replays.len()))
        .collect();
    let in_place_new = writes_new.iter().filter(|write| write.kind != StepKind::UnitWrite).count();
    eprintln!(
        "OPUS_K2 compare same_segments={same_segments} same_writes={same_writes} tearable_differences={tearable_differences:?} tearable_new={} in_place_writes_new={in_place_new} same_state_ranges={} old_states_before_the_boundary={old_states_before_the_boundary} same_torn_images={} state_count_old={} state_count_new={}",
        tearable_new.iter().filter(|tearable| **tearable).count(),
        old_ranges == new_ranges,
        torn_old == torn_new,
        tables_old.state_count,
        tables_new.state_count
    );

    // GPU 判表：新造法建得出、原造法撞掩码宽
    let judge_old = build_judge_tables_against_the_version_table(
        &JudgeFlow {
            base: &base_old,
            writes: &writes_old,
            segments: &segments_old,
            judged_root_index: writes_old.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("根槽写"),
            versions: &versions,
            expansion: &expansion_old,
        },
        &[],
    );
    eprintln!("OPUS_K2 judge-tables-old {:?}", judge_old.as_ref().map(|tables| tables.state_count).map_err(|refusal| format!("{refusal:?}")));
    let judge_new = build_judge_tables_against_the_version_table(
        &JudgeFlow {
            base: &base_new,
            writes: &writes_new,
            segments: &segments_new,
            judged_root_index: writes_new.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("根槽写"),
            versions: &versions,
            expansion: &expansion_new,
        },
        &[],
    )
    .expect("新造法建得出表");
    let (total_words, version_byte_words) = words_of(&judge_new.words());
    eprintln!(
        "OPUS_K2 judge-tables-new states={} locations={} total_words={total_words} version_bytes_words={version_byte_words} table_mebibytes={}",
        judge_new.state_count,
        judge_new.locations.len(),
        (total_words * 4).div_ceil(1024 * 1024)
    );
}
