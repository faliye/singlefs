//! 本地常量（登记 5.7、停机条款 S1）：盘、段长表与写数、取样批的除数，与两条流的名字的单测。

use super::*;

/// 两条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
#[test]
fn local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests() {
    let closed_form = |lengths: &[usize]| -> u64 {
        1 + lengths
            .iter()
            .map(|length| (1u64 << length) - 1)
            .sum::<u64>()
    };
    assert_eq!(
        closed_form(&LOCAL_FIRST_STREAM_SEGMENT_LENGTHS),
        LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
    );
    assert_eq!(
        LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
        LOCAL_FIRST_STREAM_WRITES
    );
    assert_eq!(
        closed_form(&LOCAL_SECOND_STREAM_SEGMENT_LENGTHS),
        LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
    );
    assert_eq!(
        LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
        LOCAL_SECOND_STREAM_WRITES
    );
    assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_678_377_095);
    assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_243);
}

/// 本地段模型里一段是哪种写：系统配置槽段取三态（原地覆写），单元段在甲二里只取全不落，记录段与根槽段同两态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalSegmentKind {
    SystemConfigurationSlots,
    Units,
    JournalRecords,
    RootSlot,
}

/// 本地段模型的一段：种类与写数。
#[derive(Clone, Copy, Debug)]
struct LocalSegment {
    kind: LocalSegmentKind,
    writes: usize,
}

/// 一次取号：[`LOCAL_INSTANCE_ACQUISITION_WRITE_ROUNDS`] 轮，每轮逐盘一写、轮后屏障，各成一段。
fn instance_acquisition_segments() -> Vec<LocalSegment> {
    (0..LOCAL_INSTANCE_ACQUISITION_WRITE_ROUNDS)
        .map(|_round| LocalSegment {
            kind: LocalSegmentKind::SystemConfigurationSlots,
            writes: LOCAL_DEVICE_COUNT,
        })
        .collect()
}

/// 一次发布：单元段（`unit_writes` 为 0 的空发布没有这一段）、两盘镜像的记录段、根槽段、逐盘一写的系统配置槽轮换段。
fn publish_segments(unit_writes: usize) -> Vec<LocalSegment> {
    let units = (unit_writes > 0).then_some(LocalSegment {
        kind: LocalSegmentKind::Units,
        writes: unit_writes,
    });
    units
        .into_iter()
        .chain([
            LocalSegment {
                kind: LocalSegmentKind::JournalRecords,
                writes: LOCAL_DEVICE_COUNT,
            },
            LocalSegment {
                kind: LocalSegmentKind::RootSlot,
                writes: 1,
            },
            LocalSegment {
                kind: LocalSegmentKind::SystemConfigurationSlots,
                writes: LOCAL_DEVICE_COUNT,
            },
        ])
        .collect()
}

/// 抬 F 与卸载开头先写新 F：逐盘一写，自成一段。
fn new_floor_segment() -> LocalSegment {
    LocalSegment {
        kind: LocalSegmentKind::SystemConfigurationSlots,
        writes: LOCAL_DEVICE_COUNT,
    }
}

/// 段模型推出的段长、写数、根槽写数、全量态数、甲二态数、两态闭式。
struct LocalModelCounts {
    segment_lengths: Vec<usize>,
    writes: usize,
    root_writes: usize,
    full_states: u64,
    in_place_subsets_states: u64,
    two_state_closed_form: u64,
}

fn local_model_counts(segments: &[LocalSegment]) -> LocalModelCounts {
    let two_states = |writes: usize| (1u64 << writes) - 1;
    let three_states = |writes: usize| 3u64.pow(u32::try_from(writes).expect("段长装得下 u32")) - 1;
    let full_states = 1 + segments
        .iter()
        .map(|segment| match segment.kind {
            LocalSegmentKind::SystemConfigurationSlots => three_states(segment.writes),
            LocalSegmentKind::Units
            | LocalSegmentKind::JournalRecords
            | LocalSegmentKind::RootSlot => two_states(segment.writes),
        })
        .sum::<u64>();
    let in_place_subsets_states = 1 + segments
        .iter()
        .map(|segment| match segment.kind {
            LocalSegmentKind::SystemConfigurationSlots => three_states(segment.writes),
            LocalSegmentKind::Units => 1,
            LocalSegmentKind::JournalRecords | LocalSegmentKind::RootSlot => {
                two_states(segment.writes)
            }
        })
        .sum::<u64>();
    LocalModelCounts {
        segment_lengths: segments.iter().map(|segment| segment.writes).collect(),
        writes: segments.iter().map(|segment| segment.writes).sum(),
        root_writes: segments
            .iter()
            .filter(|segment| segment.kind == LocalSegmentKind::RootSlot)
            .count(),
        full_states,
        in_place_subsets_states,
        two_state_closed_form: 1 + segments
            .iter()
            .map(|segment| two_states(segment.writes))
            .sum::<u64>(),
    }
}

/// 流一的本地段模型：取号两轮、暖机两次空发布、A（24 个单元写）。拼出的段长表与本地段长表逐段相等，推出的写数、根槽写数、
/// 三个态数与本地常量逐个相等（段长表少了取号的一轮，这里红）。
#[test]
fn local_first_stream_segment_model_opens_with_two_acquisition_rounds() {
    let segments: Vec<LocalSegment> = instance_acquisition_segments()
        .into_iter()
        .chain(publish_segments(0))
        .chain(publish_segments(0))
        .chain(publish_segments(24))
        .collect();
    let counts = local_model_counts(&segments);
    assert_eq!(
        counts.segment_lengths,
        LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.to_vec(),
        "取号两轮各一段，再是两次暖机与 A"
    );
    assert_eq!(counts.writes, LOCAL_FIRST_STREAM_WRITES);
    assert_eq!(counts.root_writes, LOCAL_FIRST_STREAM_ROOT_WRITES);
    assert_eq!(counts.full_states, LOCAL_FIRST_STREAM_FULL_STATES);
    assert_eq!(
        counts.in_place_subsets_states,
        LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES
    );
    assert_eq!(
        counts.two_state_closed_form,
        LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
    );
}

/// 流二的本地段模型：取号两轮、暖机两次、A、B，重开可写挂载再取号两轮、写行、两次暖机、C、回退 D、五次覆盖写，抬 F（先写新 F、两次空发布）、
/// E，卸载（先写新 F、两次空发布）。拼出的段长表与本地段长表逐段相等，推出的写数、根槽写数、三个态数与本地常量逐个相等
/// （段长表少了哪一次取号的一轮，这里红）。
#[test]
fn local_second_stream_segment_model_acquires_twice_in_two_rounds() {
    let mut segments: Vec<LocalSegment> = instance_acquisition_segments();
    for unit_writes in [0, 0, 24, 24] {
        segments.extend(publish_segments(unit_writes));
    }
    segments.extend(instance_acquisition_segments());
    for unit_writes in [18, 16, 16, 24, 24, 28, 28, 28, 28, 28] {
        segments.extend(publish_segments(unit_writes));
    }
    segments.push(new_floor_segment());
    for unit_writes in [16, 16, 28] {
        segments.extend(publish_segments(unit_writes));
    }
    segments.push(new_floor_segment());
    for unit_writes in [16, 16] {
        segments.extend(publish_segments(unit_writes));
    }
    let counts = local_model_counts(&segments);
    assert_eq!(
        counts.segment_lengths,
        LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.to_vec(),
        "两次取号各两轮、各两段"
    );
    assert_eq!(counts.writes, LOCAL_SECOND_STREAM_WRITES);
    assert_eq!(counts.root_writes, LOCAL_SECOND_STREAM_ROOT_WRITES);
    assert_eq!(counts.full_states, LOCAL_SECOND_STREAM_FULL_STATES);
    assert_eq!(
        counts.in_place_subsets_states,
        LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES
    );
    assert_eq!(
        counts.two_state_closed_form,
        LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
    );
}
