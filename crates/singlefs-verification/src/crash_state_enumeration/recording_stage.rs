//! 录入段：流按步切成前缀树的节点、每个节点的崩溃点与块键，录进库（GPU 那一路连判表一起录）。

use super::*;

// ============================== 录入段 ==============================

/// 项名：崩溃点重放的项名也是崩溃点的名字（路径段、节点名的一截），所以 `/` 与 `|` 都不许。
pub(super) fn item_name_of(item: CrashPointReplayItem<'_>) -> ItemName {
    assert!(
        !item.name.contains('/') && !item.name.contains('|'),
        "崩溃点重放项名 {:?} 里不许有 / 与 |：它也是这条流那个崩溃点的名字，是路径段、节点名的一截",
        item.name
    );
    ItemName::new(item.name).unwrap_or_else(|refusal| panic!("崩溃点重放项名：{refusal}"))
}

pub(super) fn flow_of_the_inputs<'flow>(
    item: CrashPointReplayItem<'_>,
    base: &'flow MemoryPool,
    writes: &'flow [RetainedWrite],
    segments: &'flow [Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'flow>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> FlowToRecord<'flow> {
    FlowToRecord {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion_by_segment: segments
            .iter()
            .enumerate()
            .map(|(segment_index, segment)| expansion(segment_index, segment))
            .collect(),
        crash_points: crash_points_of_the_steps(item, writes, segments),
    }
}

/// 一条崩溃枚举用例的流按步切成前缀树的节点（L6）：写下标区间互相交叠的几段并成一步，每一步一个崩溃点，点的写区间首尾相接罩住整张写表；
/// 点名 `<项名>.<周期>.<步序>`（周期取这一步最后一次写的，`crash_amplification::period_of_write`）只作索引。块键照崩溃放量的同一个算法
/// （段写表、起步镜像、计划哈希），几条流共有的前几步就是同几个节点，只判一次。切不成首尾相接的几步（某一段的写落在前一步之前）就
/// 整条流一个点。实验装置（E184（常规层与大负载层的代价计数））也调它，按同一套切法数节点，不另抄一份。
pub fn crash_points_of_the_steps(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) -> Vec<CrashPointSpan> {
    let whole_flow = || {
        vec![CrashPointSpan {
            name: item.name.to_string(),
            code_file_in_repository: item.setting_file.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }]
    };
    // 每一步：（最低的写下标，最高的写下标），按段的次序并
    let mut steps: Vec<(usize, usize)> = Vec::new();
    for segment in segments {
        let (Some(lowest), Some(highest)) = (segment.iter().min(), segment.iter().max()) else {
            continue;
        };
        match steps.last_mut() {
            Some((step_lowest, step_highest)) if *lowest <= *step_highest => {
                if *lowest < *step_lowest {
                    return whole_flow();
                }
                *step_highest = (*step_highest).max(*highest);
            }
            Some(_) | None => steps.push((*lowest, *highest)),
        }
    }
    if steps.len() < 2 || !steps.windows(2).all(|pair| pair[0].1 < pair[1].0) {
        return whole_flow();
    }
    let starts: Vec<usize> = std::iter::once(0)
        .chain(steps.iter().skip(1).map(|(lowest, _highest)| *lowest))
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(step_index, start)| {
            let end = starts.get(step_index + 1).copied().unwrap_or(writes.len());
            let (_lowest, highest) = steps[step_index];
            CrashPointSpan {
                name: format!(
                    "{}.{}.{step_index}",
                    item.name,
                    crate::crash_amplification::period_of_write(&writes[highest])
                ),
                code_file_in_repository: item.setting_file.to_string(),
                writes: *start..end,
                ignored: false,
            }
        })
        .collect()
}

/// 录入段把 GPU 判表写进库（配了 GPU 的那一趟），交回它的摘要。建表、打包、算摘要在锁外。
#[cfg(feature = "gpu")]
pub(super) fn record_the_judge_tables(
    library: &CrashLibrary,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Result<[u8; DIGEST_BYTES], String> {
    assert_this_thread_holds_no_library_lock("录入段建表、打包判表");
    let packed_bytes =
        crate::crash_amplification::packed_judge_tables_of_the_plan(flow, plan)?.to_bytes();
    let digest = sha256_digest(&packed_bytes);
    library
        .lock()
        .store_packed_judge_tables(&digest, &packed_bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    Ok(digest)
}

/// 没编进 `gpu` 特性选路不会要 GPU 判表（[`crash_point_replay_judge_route`]）。
#[cfg(not(feature = "gpu"))]
pub(super) fn record_the_judge_tables(
    _library: &CrashLibrary,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
) -> Result<[u8; DIGEST_BYTES], String> {
    Err("这份二进制没编进 gpu 特性，建不了 GPU 判表（选路不该要它）".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L6：崩溃枚举用例按步切成前缀树的节点，节点键与崩溃放量同一个算法：两条流共前三步（单元写与 journal 记录、根槽写、系统配置槽轮换），
    /// 前三个节点的块键逐个相同，第四步（单元写的内容不同）起不同。判别力（`crates/mutations.tsv`）：把前缀树退成整条流一个点，这条红。
    #[test]
    fn two_flows_sharing_their_first_steps_share_the_keys_of_those_nodes_and_differ_at_the_next() {
        let shared: [&[(StepKind, u8, u64)]; 3] = [
            &[
                (StepKind::UnitWrite, 1, 0),
                (StepKind::UnitWrite, 1, 0),
                (StepKind::JournalRecord, 1, 0),
            ],
            &[(StepKind::RootRecordFua, 0, 1)],
            &[
                (StepKind::SystemConfigurationSlot, 1, 0),
                (StepKind::SystemConfigurationSlot, 1, 0),
            ],
        ];
        let tail_of = |byte: u8| -> Vec<Vec<(StepKind, u8, u64)>> {
            vec![
                vec![
                    (StepKind::UnitWrite, byte, 0),
                    (StepKind::JournalRecord, 2, 0),
                ],
                vec![(StepKind::RootRecordFua, 0, 2)],
            ]
        };
        let flow_with_the_tail = |byte: u8| {
            let tail = tail_of(byte);
            let mut segments_of_writes: Vec<&[(StepKind, u8, u64)]> = shared.to_vec();
            segments_of_writes.extend(tail.iter().map(Vec::as_slice));
            writes_and_segments_of(&segments_of_writes)
        };
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let item = CrashPointReplayItem {
            name: "prefix_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };
        let (writes_with_the_tail_writing_two, segments_with_the_tail_writing_two) =
            flow_with_the_tail(2);
        let (writes_with_the_tail_writing_three, segments_with_the_tail_writing_three) =
            flow_with_the_tail(3);
        let nodes_with_the_tail_writing_two = block_keys_of_each_node(
            item,
            &base,
            &writes_with_the_tail_writing_two,
            &segments_with_the_tail_writing_two,
        );
        let nodes_with_the_tail_writing_three = block_keys_of_each_node(
            item,
            &base,
            &writes_with_the_tail_writing_three,
            &segments_with_the_tail_writing_three,
        );
        assert_eq!(
            nodes_with_the_tail_writing_two.len(),
            6,
            "五步五个节点加末节点"
        );
        assert_eq!(nodes_with_the_tail_writing_three.len(), 6);
        assert!(
            nodes_with_the_tail_writing_two
                .iter()
                .all(|keys| !keys.is_empty()),
            "每个节点都有块"
        );
        assert_eq!(
            nodes_with_the_tail_writing_two[..3],
            nodes_with_the_tail_writing_three[..3],
            "共有的前三步是同三个节点：块键逐个相同"
        );
        assert_ne!(
            nodes_with_the_tail_writing_two[3], nodes_with_the_tail_writing_three[3],
            "第四步内容不同，节点键不同"
        );
    }

    #[test]
    fn a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash() {
        let item = CrashPointReplayItem {
            name: "fine_name",
            setting_file: "crates/singlefs-verification/tests/x.rs",
        };
        assert_eq!(item_name_of(item).as_str(), "fine_name");
        let caught = std::panic::catch_unwind(|| {
            item_name_of(CrashPointReplayItem {
                name: "a/b",
                setting_file: "crates/singlefs-verification/tests/x.rs",
            })
        });
        assert!(caught.is_err(), "带 / 的项名要停下");
    }
}
