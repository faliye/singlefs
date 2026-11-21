use singlefs_replay_prefix::gpu_gc_capacity_proofs::*;

const EDGES: [u64; 12] = [
    0,
    1,
    3,
    4,
    5,
    16,
    63,
    1 << 32,
    u64::MAX - 3,
    u64::MAX - 2,
    u64::MAX - 1,
    u64::MAX,
];

fn reference_fits(input: u64, entries: u64, capacity: &GpuChunkCapacity) -> bool {
    let input_bytes = u128::from(input).max(1).div_ceil(4) * 4;
    if input_bytes > u128::from(capacity.binding_bytes)
        || u128::from(entries) * u128::from(capacity.entry_buffer_bytes)
            > u128::from(capacity.binding_bytes)
        || entries > capacity.dispatch_entries
    {
        return false;
    }
    capacity.memory_limit.is_none_or(|limit| {
        (u128::from(entries) * u128::from(capacity.bytes_per_entry))
            .checked_add(input_bytes)
            .and_then(|total| total.checked_add(u128::from(capacity.fixed_bytes)))
            .is_some_and(|total| total <= u128::from(limit))
    })
}

// 三张映射表每项的字节数按产品交给核的布局逐个字段数出来（`crates/singlefs-background-gpu/src/region_record_kernels.rs`
// 文件头「缓冲布局」绑定 3、4、5 那三行：条目、落点索引、按 key 排的序号），不抄被测核里的乘数。
const WORD_BYTES: u128 = 4;
const KEY_LENGTH_WORDS: u128 = 1;
const KEY_WORDS: u128 = 7;
const LOCATIONS_PER_MAPPING_ENTRY: u128 = 2;
const WORDS_PER_LOCATION_OF_AN_ENTRY: u128 = 3;
const WORDS_PER_LOCATION_ITEM: u128 = 4;
const WORDS_PER_SORTED_INDEX_ITEM: u128 = 1;

fn mapping_entry_item_bytes() -> u128 {
    WORD_BYTES
        * (KEY_LENGTH_WORDS
            + KEY_WORDS
            + LOCATIONS_PER_MAPPING_ENTRY * WORDS_PER_LOCATION_OF_AN_ENTRY)
}

fn location_item_bytes() -> u128 {
    WORD_BYTES * WORDS_PER_LOCATION_ITEM
}

fn sorted_index_item_bytes() -> u128 {
    WORD_BYTES * WORDS_PER_SORTED_INDEX_ITEM
}

#[test]
fn chunk_admission_and_extension_match_full_width_arithmetic() {
    for binding in EDGES {
        for entry_bytes in EDGES {
            for limit in [None, Some(0), Some(100), Some(u64::MAX)] {
                let capacity = GpuChunkCapacity {
                    fixed_bytes: 16,
                    bytes_per_entry: 40,
                    entry_buffer_bytes: entry_bytes,
                    binding_bytes: binding,
                    dispatch_entries: 3,
                    memory_limit: limit,
                };
                for input in EDGES {
                    for entries in EDGES {
                        assert_eq!(
                            gpu_chunk_fits(input, entries, &capacity),
                            reference_fits(input, entries, &capacity)
                        );
                        for additional in [0, 1, 4, u64::MAX] {
                            let expected =
                                u64::try_from(u128::from(input) + u128::from(additional))
                                    .ok()
                                    .filter(|total| reference_fits(*total, entries, &capacity));
                            assert_eq!(
                                extended_gpu_chunk_input(input, additional, entries, &capacity),
                                expected
                            );
                        }
                    }
                }
            }
        }
    }
    let unlimited = GpuChunkCapacity {
        fixed_bytes: u64::MAX,
        bytes_per_entry: u64::MAX,
        entry_buffer_bytes: 0,
        binding_bytes: 4,
        dispatch_entries: 2,
        memory_limit: None,
    };
    // 无限额仍检查每个绑定和派发，但不把虚构的有限总额加进准入条件。
    assert!(gpu_chunk_fits(1, 2, &unlimited));
}

#[test]
fn largest_piece_matches_exhaustive_search_and_is_maximal() {
    for binding in 0..=40 {
        for fixed in [0, 1, 16, 40, u64::MAX] {
            for per_entry in [0, 1, 4, 40, u64::MAX] {
                for entry_buffer in [0, 4, 64] {
                    for dispatch in [0, 1] {
                        for limit in [
                            None,
                            Some(0),
                            Some(3),
                            Some(4),
                            Some(20),
                            Some(100),
                            Some(u64::MAX),
                        ] {
                            let capacity = GpuChunkCapacity {
                                fixed_bytes: fixed,
                                bytes_per_entry: per_entry,
                                entry_buffer_bytes: entry_buffer,
                                binding_bytes: binding,
                                dispatch_entries: dispatch,
                                memory_limit: limit,
                            };
                            let expected =
                                (1..=binding).rfind(|input| reference_fits(*input, 1, &capacity));
                            assert_eq!(
                                largest_gpu_piece_input(&capacity),
                                expected,
                                "binding={binding} fixed={fixed} per_entry={per_entry} entry_buffer={entry_buffer} dispatch={dispatch} limit={limit:?}"
                            );
                        }
                    }
                }
            }
        }
    }
    let capacity = GpuChunkCapacity {
        fixed_bytes: 0,
        bytes_per_entry: 0,
        entry_buffer_bytes: 0,
        binding_bytes: u64::MAX,
        dispatch_entries: 1,
        memory_limit: None,
    };
    assert_eq!(largest_gpu_piece_input(&capacity), Some(u64::MAX - 3));
    assert!(!gpu_chunk_fits(u64::MAX - 2, 1, &capacity));
}

#[test]
fn holds_releases_and_waits_preserve_the_exact_ledger() {
    for held in EDGES {
        for bytes in EDGES {
            for limit in [None, Some(0), Some(100), Some(u64::MAX)] {
                for pending in [false, true] {
                    let sum = u128::from(held) + u128::from(bytes);
                    let expected = if pending || limit.is_some_and(|bound| sum > u128::from(bound))
                    {
                        None
                    } else {
                        u64::try_from(sum).ok()
                    };
                    assert_eq!(
                        checked_gpu_memory_hold(held, bytes, pending, limit),
                        expected
                    );
                    if let Some(total) = expected {
                        assert_eq!(checked_gpu_memory_release(total, bytes), Some(held));
                        for _ in 0..3 {
                            assert_eq!(
                                gpu_bytes_after_submission_wait(total, bytes, false),
                                Some(total)
                            );
                        }
                        assert_eq!(
                            gpu_bytes_after_submission_wait(total, bytes, true),
                            Some(held)
                        );
                    }
                }
            }
            let release = if bytes > held {
                None
            } else {
                Some(held - bytes)
            };
            assert_eq!(checked_gpu_memory_release(held, bytes), release);
            assert_eq!(gpu_bytes_after_submission_wait(held, bytes, true), release);
            assert_eq!(
                gpu_bytes_after_submission_wait(held, bytes, false),
                (bytes <= held).then_some(held)
            );
        }
    }
}

#[test]
fn memory_caps_never_expand_and_allocation_failure_makes_progress() {
    for available in EDGES {
        for previous in [None, Some(0), Some(17), Some(u64::MAX)] {
            let actual = tightened_gpu_memory_limit(previous, available);
            assert_eq!(actual, previous.unwrap_or(u64::MAX).min(available));
        }
        let mut cap = available;
        let mut rounds = 0;
        while cap > 0 {
            let next = gpu_memory_cap_after_allocation_failure(cap);
            // 参照不做除法：折半的结果按它的定义判——它的两倍不超过失败的占用，它加一的两倍超过失败的占用。
            let doubled = 2 * u128::from(next);
            assert!(
                doubled <= u128::from(cap) && u128::from(cap) < doubled + 2,
                "cap={cap} next={next}：next 的两倍应不超过 cap，next 加一的两倍应超过 cap"
            );
            assert!(next < cap);
            cap = next;
            rounds += 1;
        }
        assert!(rounds <= 64);
        assert_eq!(gpu_memory_cap_after_allocation_failure(0), 0);
    }
}

#[test]
fn mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes() {
    for entries in EDGES {
        for locations in EDGES {
            let entry = mapping_entry_item_bytes() * u128::from(entries.max(1));
            let location = location_item_bytes() * u128::from(locations.max(1));
            let index = sorted_index_item_bytes() * u128::from(entries.max(1));
            let expected = u64::try_from(entry + location + index).ok().map(|total| {
                (
                    total,
                    u64::try_from(entry.max(location).max(index)).unwrap(),
                )
            });
            assert_eq!(
                checked_gpu_mapping_table_bytes(entries, locations),
                expected
            );
        }
    }
    assert_eq!(checked_gpu_mapping_table_bytes(0, 0), Some((76, 56)));
    for entries in [0, 1, 63, 64, 65, u32::MAX - 1, u32::MAX] {
        for per_group in [1, 2, 3, 32, 64, 256, u32::MAX] {
            let expected = u64::from(entries).div_ceil(u64::from(per_group));
            assert_eq!(
                u64::from(gpu_workgroups_for_entries(entries, per_group)),
                expected
            );
        }
    }
}
