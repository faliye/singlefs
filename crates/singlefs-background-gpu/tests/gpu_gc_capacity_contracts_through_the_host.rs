use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

use singlefs_background_gpu::memory_limit_and_chunks::{
    chunks_that_fit_on_the_gpu, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu, ChunkFootprint,
    EntryCountOfAChunk, GpuMemoryAccount, InputLengthOnTheHostInBytes, SubmissionOfTheChunk,
};
use singlefs_background_gpu::region_record_kernels::MappingTablesForTheKernels;
use singlefs_background_gpu::waiting_for_the_gpu::{
    CompletionCheck, GpuWaitDeadline, MonotonicClock,
};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::background_compute::GpuMemoryInBytes;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::region_scan::CentralMappingEntry;
use singlefs_replay_prefix::gpu_gc_capacity_proofs::checked_gpu_mapping_table_bytes;

fn footprint() -> ChunkFootprint {
    ChunkFootprint {
        bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(4),
        bytes_per_entry_besides_its_input: GpuMemoryInBytes(4),
        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(4),
    }
}

fn on_the_host(lengths: &[u64]) -> Vec<InputLengthOnTheHostInBytes> {
    lengths
        .iter()
        .copied()
        .map(InputLengthOnTheHostInBytes)
        .collect()
}

fn reference_fits(lengths: &[u64], binding: u64, dispatch: u64, limit: Option<u64>) -> bool {
    let input: u128 = lengths.iter().map(|length| u128::from(*length)).sum();
    let aligned = input.max(1).div_ceil(4) * 4;
    let count = lengths.len() as u128;
    aligned <= u128::from(binding)
        && 4 * count <= u128::from(binding)
        && count <= u128::from(dispatch)
        && limit.is_none_or(|limit| 2 * aligned + 4 * count + 4 <= u128::from(limit))
}

#[test]
fn greedy_chunks_match_independent_prefix_search() {
    let choices = [0, 1, 3, 4, 5, 9];
    for pattern in 0..6usize.pow(4) {
        let mut encoded = pattern;
        let lengths: Vec<_> = (0..4)
            .map(|_| {
                let value = choices[encoded % 6];
                encoded /= 6;
                value
            })
            .collect();
        for binding in [0, 4, 8, 16] {
            for dispatch in [0, 1, 3] {
                for limit in [None, Some(0), Some(12), Some(24), Some(80)] {
                    let bounds = ChunkBoundsOfAGpu {
                        memory_limit: limit.map(GpuMemoryInBytes),
                        largest_storage_binding_in_bytes: GpuMemoryInBytes(binding),
                        largest_entry_count_per_dispatch: EntryCountOfAChunk(dispatch),
                    };
                    let result =
                        chunks_that_fit_on_the_gpu(&on_the_host(&lengths), footprint(), bounds);
                    if let Some(position) = lengths
                        .iter()
                        .position(|length| !reference_fits(&[*length], binding, dispatch, limit))
                    {
                        let failure = result.unwrap_err();
                        assert_eq!(failure.entry_position, position);
                        assert_eq!(
                            failure.input_bytes,
                            InputLengthOnTheHostInBytes(lengths[position])
                        );
                    } else {
                        let mut expected = Vec::new();
                        let mut start = 0;
                        while start < lengths.len() {
                            let end = (start + 1..=lengths.len())
                                .filter(|end| {
                                    reference_fits(&lengths[start..*end], binding, dispatch, limit)
                                })
                                .max()
                                .unwrap();
                            expected.push(start..end);
                            start = end;
                        }
                        assert_eq!(
                            result.unwrap(),
                            expected,
                            "pattern={pattern} binding={binding} dispatch={dispatch} limit={limit:?}"
                        );
                    }
                    assert!(chunks_that_fit_on_the_gpu(&[], footprint(), bounds)
                        .unwrap()
                        .is_empty());
                }
            }
        }
    }
}

#[test]
fn full_width_inputs_split_without_sum_or_endpoint_wraparound() {
    let zero_overhead = ChunkFootprint {
        bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(0),
        bytes_per_entry_besides_its_input: GpuMemoryInBytes(0),
        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(0),
    };
    let bounds = ChunkBoundsOfAGpu {
        memory_limit: None,
        largest_storage_binding_in_bytes: GpuMemoryInBytes(u64::MAX),
        largest_entry_count_per_dispatch: EntryCountOfAChunk(2),
    };
    assert_eq!(
        chunks_that_fit_on_the_gpu(&on_the_host(&[u64::MAX - 3, 8]), zero_overhead, bounds)
            .unwrap(),
        vec![0..1, 1..2]
    );
    assert_eq!(
        chunks_that_fit_on_the_gpu(&on_the_host(&[u64::MAX]), zero_overhead, bounds)
            .unwrap_err()
            .entry_position,
        0
    );
    let pieces =
        pieces_cut_to_fit_on_the_gpu(&on_the_host(&[u64::MAX]), zero_overhead, bounds).unwrap();
    assert_eq!(pieces.len(), 2);
    assert_eq!(
        pieces[0][0].input_bytes,
        InputLengthOnTheHostInBytes(0)..InputLengthOnTheHostInBytes(u64::MAX - 3)
    );
    assert_eq!(
        pieces[1][0].input_bytes,
        InputLengthOnTheHostInBytes(u64::MAX - 3)..InputLengthOnTheHostInBytes(u64::MAX)
    );
    assert_eq!(pieces[0][0].entry_position, 0);
    assert_eq!(pieces[1][0].entry_position, 0);
    let wide_entry = ChunkFootprint {
        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(u64::MAX),
        ..zero_overhead
    };
    assert_eq!(
        chunks_that_fit_on_the_gpu(&on_the_host(&[1, 1]), wide_entry, bounds).unwrap(),
        vec![0..1, 1..2]
    );
}

#[test]
fn pieces_cover_every_byte_and_every_chunk_fits() {
    for length in 0..=64 {
        for binding in [4, 8, 16] {
            for limit in [16, 32, 56] {
                let bounds = ChunkBoundsOfAGpu {
                    memory_limit: Some(GpuMemoryInBytes(limit)),
                    largest_storage_binding_in_bytes: GpuMemoryInBytes(binding),
                    largest_entry_count_per_dispatch: EntryCountOfAChunk(3),
                };
                let lengths = [length, 0, length + 1];
                let chunks =
                    pieces_cut_to_fit_on_the_gpu(&on_the_host(&lengths), footprint(), bounds)
                        .unwrap();
                let mut ends = [0; 3];
                let mut seen = [false; 3];
                let mut last_entry = 0;
                for chunk in chunks {
                    let piece_lengths: Vec<_> = chunk
                        .iter()
                        .map(|piece| piece.input_byte_count().0)
                        .collect();
                    assert!(reference_fits(&piece_lengths, binding, 3, Some(limit)));
                    for piece in chunk {
                        assert!(piece.entry_position >= last_entry);
                        last_entry = piece.entry_position;
                        assert_eq!(piece.input_bytes.start.0, ends[piece.entry_position]);
                        assert!(piece.input_bytes.end.0 <= lengths[piece.entry_position]);
                        assert!(
                            piece.input_bytes.end > piece.input_bytes.start
                                || lengths[piece.entry_position] == 0
                        );
                        ends[piece.entry_position] = piece.input_bytes.end.0;
                        seen[piece.entry_position] = true;
                    }
                }
                assert_eq!(ends, lengths);
                assert_eq!(seen, [true; 3]);
            }
        }
    }
}

struct ManualClock(Cell<Duration>);
impl MonotonicClock for ManualClock {
    fn time_since_the_origin(&self) -> Duration {
        self.0.get()
    }
}

#[test]
fn failed_waits_retain_the_ledger_and_completion_releases_exactly_once() {
    let mut account = GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts(
        Some(GpuMemoryInBytes(100)),
        GpuMemoryInBytes(17),
    );
    account.hold(GpuMemoryInBytes(43));
    account.settle_the_chunk_after_its_submission(
        GpuMemoryInBytes(43),
        SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(7),
    );
    let clock = ManualClock(Cell::new(Duration::ZERO));
    for _ in 0..3 {
        let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(1), &clock);
        let result = account.wait_for_the_submission_running_past_its_wait(
            &deadline,
            &clock,
            |submission, left| {
                assert_eq!(*submission, 7);
                clock.0.set(clock.0.get() + left);
                CompletionCheck::NotYetComplete
            },
        );
        assert!(result.is_err());
        assert_eq!(account.bytes_held(), GpuMemoryInBytes(60));
        assert!(account.has_a_submission_running_past_its_wait());
    }
    assert!(catch_unwind(AssertUnwindSafe(|| account.hold(GpuMemoryInBytes(0)))).is_err());
    assert_eq!(account.bytes_held(), GpuMemoryInBytes(60));
    let deadline = GpuWaitDeadline::starting_now(Duration::from_secs(1), &clock);
    account
        .wait_for_the_submission_running_past_its_wait(&deadline, &clock, |submission, _| {
            assert_eq!(*submission, 7);
            CompletionCheck::Completed(())
        })
        .unwrap();
    assert_eq!(account.bytes_held(), GpuMemoryInBytes(17));
    assert!(!account.has_a_submission_running_past_its_wait());
    account
        .wait_for_the_submission_running_past_its_wait(&deadline, &clock, |_, _| {
            panic!("完成后不能再次等待或释放")
        })
        .unwrap();
    assert_eq!(account.bytes_held(), GpuMemoryInBytes(17));
    let mut full = GpuMemoryAccount::<()>::with_the_bytes_leaked_by_earlier_mounts(
        None,
        GpuMemoryInBytes(u64::MAX),
    );
    assert!(catch_unwind(AssertUnwindSafe(|| full.hold(GpuMemoryInBytes(1)))).is_err());
    assert_eq!(full.bytes_held(), GpuMemoryInBytes(u64::MAX));
    assert!(catch_unwind(AssertUnwindSafe(|| account.release(GpuMemoryInBytes(18)))).is_err());
    assert_eq!(account.bytes_held(), GpuMemoryInBytes(17));
}

#[test]
fn verified_mapping_sizes_match_the_actual_packed_tables() {
    for entries in 0..=16 {
        for same_location in [false, true] {
            let records: Vec<_> = (0..entries)
                .map(|entry| {
                    let first = LocationEntry {
                        device: DeviceIdentity(0),
                        slot: SlotNumber(entry + 64),
                        unit_checksum: 7,
                    };
                    let second = if same_location {
                        first
                    } else {
                        LocationEntry {
                            device: DeviceIdentity(1),
                            ..first
                        }
                    };
                    CentralMappingEntry {
                        key: vec![u8::try_from(entry).expect("用例里的条目号都小于 256")],
                        locations: [first, second],
                    }
                })
                .collect();
            let tables = MappingTablesForTheKernels::of_the_mapping_entries(&records).unwrap();
            let locations = entries * if same_location { 1 } else { 2 };
            assert_eq!(
                checked_gpu_mapping_table_bytes(entries, locations),
                Some((tables.bytes_on_the_gpu().0, tables.largest_buffer_bytes().0))
            );
        }
    }
}
