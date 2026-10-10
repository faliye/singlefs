//! N8：GPU 对拍那几条删除线的流（新池新建文件 → 顺序写三个单元 → 意图 → 批大小 1 的第一批）上，展开上限 8（`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`
//! 的缺省）展开了哪几段、比了几个状态；没展开的段里（单个写没落的那几个状态）CPU 池级 checker 判什么——GPU 那一侧在这些状态上要给同一组违例位，
//! 而对拍一个都没比。只用 CPU 判法，GPU 不跑。
use singlefs_checker_tier::crash::{layer0_plan_state_count, Layer0SegmentExpansion};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes, MemoryPool};
use singlefs_harness::segments::StepKind;

use crate::common::{file_content, geometry, FIXED_WRITE_TIME_SECONDS};
use crate::judge::judge;
use crate::pool::{content_of, SparsePool};

pub fn run() {
    let payload = data_unit_payload_capacity();
    let mut pool = SparsePool::build(4 << 30, &file_content());
    pool.sequential_write(&content_of(2 * payload + payload / 2), FIXED_WRITE_TIME_SECONDS + 60);
    let time = pool.output.inode_record.expect("记录").write_time_seconds;
    pool.intent(1, time).expect("意图");
    pool.step(1, 1).expect("第一批");
    let operations = pool.stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0), singlefs_core::address::DeviceIdentity(1)], 4 << 30);
    base.apply(&operations[..pool.mkfs_operation_count]);
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let limit = 8usize;
    let limited = move |_index: usize, segment: &[usize]| {
        if segment.len() <= limit { Layer0SegmentExpansion::EveryProperSubset } else { Layer0SegmentExpansion::NotExpanded }
    };
    let full = |_index: usize, _segment: &[usize]| Layer0SegmentExpansion::EveryProperSubset;
    let compared = layer0_plan_state_count(&base, &writes, &segments, &limited);
    let whole = layer0_plan_state_count(&base, &writes, &segments, &full);
    let not_expanded: Vec<(usize, usize)> = segments.iter().enumerate().filter(|(_, segment)| segment.len() > limit).map(|(index, segment)| (index, segment.len())).collect();
    println!(
        "GPUGAP flow=three_units_intent_first_batch writes={} segments={} lengths={:?} closed_form={} plan_states_limit8={compared} plan_states_full={whole} not_expanded_segments={not_expanded:?}",
        writes.len(),
        segments.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        closed_form_state_count(&segments)
    );
    // 没展开的每一段：它之前的段全落、这一段只差一个写（段内整写子集里的一种），CPU 池级 checker 的六条判定与违例。
    for (segment_index, _) in &not_expanded {
        let segment = &segments[*segment_index];
        let kinds: Vec<&str> = segment.iter().map(|write| writes[*write].kind.name()).collect();
        let unit_writes = segment.iter().filter(|write| writes[**write].kind == StepKind::UnitWrite).count();
        let mut before = base.clone();
        for earlier in &segments[..*segment_index] {
            let earlier_writes: Vec<_> = earlier.iter().map(|write| writes[*write].clone()).collect();
            before.apply_writes(&earlier_writes);
        }
        let mut summary = std::collections::BTreeMap::<String, u64>::new();
        let mut red = 0;
        for missing in segment {
            let mut image = before.clone();
            let persisted: Vec<_> = segment.iter().filter(|write| *write != missing).map(|write| writes[*write].clone()).collect();
            image.apply_writes(&persisted);
            let judged = judge(&image);
            *summary.entry(judged.watched_string()).or_insert(0) += 1;
            if !judged.violated.is_empty() {
                red += 1;
            }
        }
        println!(
            "GPUGAP_SEGMENT index={segment_index} length={} unit_writes={unit_writes} kinds={kinds:?} states_all_but_one={} cpu_red_states={red} cpu_watched={summary:?}",
            segment.len(),
            segment.len()
        );
    }
}
