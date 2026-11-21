//! 写表与发布表：每次写落在哪些落点、之后与它重叠的写，给定状态在缝两边对不对得上，
//! 每次发布期待读回的那一版的数据块。

use super::*;

/// 写表：每次写的落点、根身份、根字节、之后与它重叠的写。
pub(super) fn judge_writes(
    enumerated: &[RetainedWrite],
    recorded: &[RetainedWrite],
    locations: &[JudgeLocation],
    location_index_by_start: &BTreeMap<(u32, u64), usize>,
) -> Result<Vec<JudgeWrite>, JudgeTablesRefusal> {
    let mut writes = Vec::with_capacity(enumerated.len());
    for (write_index, write) in enumerated.iter().enumerate() {
        let location = match write.kind {
            // discard 按段整段发、罩住几十个单元槽，清零写一次清 [0, 根环基址)、一个根环区域或整个 journal 环，都不落在哪一个落点上：
            // 它罩住的每个落点各把它记进自己的写（建落点那一步按字节范围记），版本的字节按叠法叠出来。写表里它这一行只给持久掩码与
            // 「之后与一份副本重叠的写」用。
            StepKind::Discard | StepKind::ZeroFill => None,
            StepKind::UnitWrite
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {
                let location_index = location_index_by_start
                    .get(&(write.device.0, write.offset.0))
                    .copied()
                    .ok_or(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index })?;
                if write.length_in_bytes() > u64::from(locations[location_index].length) {
                    return Err(JudgeTablesRefusal::WriteNotAlignedToALocation { write_index });
                }
                Some(u32::try_from(location_index).expect("下标装得进 u32"))
            }
        };
        let root_identity = (write.kind == StepKind::RootRecordFua).then(|| {
            let (instance, checkpoint_txg) = root_identity_of_write(write);
            (instance.0, checkpoint_txg.0)
        });
        let root_bytes = match (&write.contents, write.kind) {
            (WrittenContents::Bytes(bytes), StepKind::RootRecordFua) => bytes.clone(),
            (WrittenContents::Bytes(_) | WrittenContents::Zeros { .. }, _) => Vec::new(),
        };
        let later_overlapping_writes = if write.kind == StepKind::UnitWrite {
            later_overlapping_writes_of(enumerated, recorded, write_index)
        } else {
            Vec::new()
        };
        writes.push(JudgeWrite {
            device: write.device.0,
            offset: write.offset.0,
            length: u32::try_from(write.length_in_bytes()).expect("写长装得进 u32"),
            kind: write.kind,
            location,
            root_identity,
            root_bytes,
            later_overlapping_writes,
        });
    }
    Ok(writes)
}

/// 一份单元副本之后与它重叠的每次写，连同回收谓词的判定（与 `crash::unit_copy_is_missing_under_the_persisted_set` 同一个读法：
/// 判定只看写表，与状态无关）。谓词按录制流那张表算（枚举写表的扩展条目不在录制流里；副本在录制流里）。
fn later_overlapping_writes_of(
    enumerated: &[RetainedWrite],
    recorded: &[RetainedWrite],
    copy: usize,
) -> Vec<LaterOverlappingWrite> {
    let copy_write = &enumerated[copy];
    let copy_start = copy_write.offset.0;
    let copy_end = copy_start + copy_write.length_in_bytes();
    let earlier_publish_txg =
        crate::crash::checkpoint_txg_of_the_publish_that_made_the_write(enumerated, copy);
    let mut laters = Vec::new();
    for later_index in copy + 1..enumerated.len() {
        let later = &enumerated[later_index];
        if later.device != copy_write.device {
            continue;
        }
        let later_start = later.offset.0;
        let later_end = later_start + later.length_in_bytes();
        if !(later_start < copy_end && copy_start < later_end) {
            continue;
        }
        let mut covered_sectors = 0u64;
        let sector_count = usize::try_from((copy_end - copy_start) / SECTOR_BYTES).expect("扇区数");
        for sector in 0..sector_count {
            let sector_offset = copy_start + u64::try_from(sector).expect("扇区号") * SECTOR_BYTES;
            if later_start < sector_offset + SECTOR_BYTES && sector_offset < later_end {
                covered_sectors |= 1u64 << sector;
            }
        }
        // 谓词按写表里这次写之前的那一段算，枚举写表里录制流之外的条目（撕裂镜像、重放）不改它：它们接在录制流之后，
        // 在它们「之前」的就是整条录制流；`reuse_is_not_proven_illegal_by_the_reclaim_predicate` 只看录制流那一段的根槽写与系统配置槽写。
        let table: &[RetainedWrite] = if later_index < recorded.len() {
            recorded
        } else {
            enumerated
        };
        let reuse_is_not_proven_illegal = reuse_is_not_proven_illegal_by_the_reclaim_predicate(
            table,
            earlier_publish_txg,
            later_index.min(table.len()),
        );
        laters.push(LaterOverlappingWrite {
            write_index: u32::try_from(later_index).expect("下标装得进 u32"),
            reuse_is_not_proven_illegal,
            covered_sectors,
        });
    }
    laters
}

/// 发布表：与记录核对器同一个分法（`publishes_in`）。按段展开、或给定状态表不带接缝时是一条录制流（持久集合不进分法）；
/// 给定状态表带接缝时在接缝处断开，崩溃接缝之前那一截剔哪几次发布按这张表的持久掩码定（每个掩码在那几个根槽写上都一样，这里核），
/// 掉电接缝之前那一截要在每个掩码里全落了（这里核，不交 `publishes_in` 去 panic）。接缝的下标是录制流的下标（`recorded_write_count`
/// 之前）。`enumerated` 是发布表从哪一张写表分（调用方照 [`PublishesSplitFrom`] 给）：给整张枚举写表时，接在录制流之后的撕裂镜像与重放归最后一截。
pub(super) fn judge_publishes(
    enumerated: &[RetainedWrite],
    recorded_write_count: usize,
    state_source: &JudgeStateSource,
    seams_of_the_given_sets: &[RecordStreamSeam],
) -> Result<Vec<JudgePublish>, JudgeTablesRefusal> {
    let representative_persisted: Option<&Vec<bool>> = match state_source {
        JudgeStateSource::SegmentExpansions => {
            assert!(
                seams_of_the_given_sets.is_empty(),
                "按段展开那一路恒给空的接缝表（build_judge_tables_from_the_state_source 的两个调用点）"
            );
            None
        }
        JudgeStateSource::GivenPersistedSets {
            persisted_sets: given_persisted_sets,
            claimed_versions: _,
        } => {
            if seams_of_the_given_sets.is_empty() {
                None
            } else {
                given_persisted_sets_agree_across_the_seams(
                    enumerated,
                    recorded_write_count,
                    given_persisted_sets,
                    seams_of_the_given_sets,
                )?;
                given_persisted_sets.first()
            }
        }
    };
    let publishes = match representative_persisted {
        Some(persisted) => publishes_in(
            enumerated,
            persisted,
            RecordStreamContinuity::BrokenAtSeams(seams_of_the_given_sets),
        ),
        None => publishes_in(
            enumerated,
            &vec![false; enumerated.len()],
            RecordStreamContinuity::OneRecording,
        ),
    };
    Ok(judge_publishes_of(enumerated, publishes))
}

/// 给定状态表的接缝核三样（[`judge_publishes`]）：接缝按录制流下标从小到大、不越过录制流；每道掉电接缝之前那一截在每个掩码里全落了；
/// 每道崩溃接缝之前那一截（从上一道接缝起）里的每个根槽写在每个掩码里落没落都一样。
fn given_persisted_sets_agree_across_the_seams(
    enumerated: &[RetainedWrite],
    recorded_write_count: usize,
    given_persisted_sets: &[Vec<bool>],
    seams: &[RecordStreamSeam],
) -> Result<(), JudgeTablesRefusal> {
    let mut first_write_of_the_recording = 0usize;
    for (seam_index, seam) in seams.iter().enumerate() {
        let first_write_after_the_seam = seam.first_write_after_the_seam();
        if first_write_after_the_seam < first_write_of_the_recording
            || first_write_after_the_seam > recorded_write_count
        {
            return Err(JudgeTablesRefusal::SeamsNotInOrderWithinTheRecordedWrites { seam_index });
        }
        match seam {
            RecordStreamSeam::AfterAPowerLoss { .. } => {
                if let Some(position) = given_persisted_sets.iter().position(|persisted| {
                    !persisted[..first_write_after_the_seam]
                        .iter()
                        .all(|is_persisted| *is_persisted)
                }) {
                    return Err(
                        JudgeTablesRefusal::GivenPersistedSetNotWhollyPersistedBeforeAPowerLossSeam {
                            position,
                            seam_index,
                        },
                    );
                }
            }
            RecordStreamSeam::AfterACrash { .. } => {
                for root_write in (first_write_of_the_recording..first_write_after_the_seam)
                    .filter(|write_index| enumerated[*write_index].kind == StepKind::RootRecordFua)
                {
                    let Some(first) = given_persisted_sets.first() else {
                        break;
                    };
                    if let Some(position) = given_persisted_sets
                        .iter()
                        .position(|persisted| persisted[root_write] != first[root_write])
                    {
                        return Err(
                            JudgeTablesRefusal::GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam {
                                root_write,
                                position,
                            },
                        );
                    }
                }
            }
        }
        first_write_of_the_recording = first_write_after_the_seam;
    }
    Ok(())
}

/// `publishes_in` 分出的每次发布换成判器的发布表（单元写按偏移分组）。
fn judge_publishes_of(
    enumerated: &[RetainedWrite],
    publishes: Vec<crate::memory_pool::PublishWrites>,
) -> Vec<JudgePublish> {
    publishes
        .into_iter()
        .map(|publish| {
            let mut copies_by_offset: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for unit in &publish.units {
                copies_by_offset
                    .entry(enumerated[*unit].offset.0)
                    .or_default()
                    .push(u32::try_from(*unit).expect("下标装得进 u32"));
            }
            JudgePublish {
                root_write: u32::try_from(publish.root).expect("下标装得进 u32"),
                instance: publish.instance,
                checkpoint_txg: publish.checkpoint_txg,
                records: publish
                    .records
                    .iter()
                    .map(|record| u32::try_from(*record).expect("下标装得进 u32"))
                    .collect(),
                unit_copy_groups: copies_by_offset.into_values().collect(),
            }
        })
        .collect()
}

/// 数据单元的载荷容量：32768 − 头 105 − 预留 29。
fn data_unit_payload_capacity() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
        - usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105")
        - usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

/// 一个已发布版本的内容按写侧的切法切成单元，每个单元该有的载荷区 CRC（预留位 29 个 0 + 载荷 + 补 0 到单元末尾）与声明长度。
pub(super) fn expected_version_chunks(version: &PublishedVersion) -> ExpectedVersionChunks {
    let capacity = data_unit_payload_capacity();
    let unit_count = version.content.len().div_ceil(capacity).max(1);
    let payload_region = usize::try_from(DATA_UNIT_BYTES).expect("32768")
        - usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    let chunks = (0..unit_count)
        .map(|index| {
            let start = (index * capacity).min(version.content.len());
            let end = ((index + 1) * capacity).min(version.content.len());
            let chunk = &version.content[start..end];
            let mut region = vec![0u8; payload_region];
            region[reserved..reserved + chunk.len()].copy_from_slice(chunk);
            (
                crc32_castagnoli(&region),
                u32::try_from(chunk.len()).expect("声明长度装得进 u32"),
            )
        })
        .collect();
    ExpectedVersionChunks {
        instance: version.instance.0,
        checkpoint_txg: version.checkpoint_txg.0,
        file: FileOfTheExpectedVersion::WithAFile,
        chunks,
    }
}

#[cfg(test)]
mod tests {
    use super::judge_publishes;
    use crate::crash_judge_tables::{
        ClaimedVersionOfAGivenState, JudgePublish, JudgeStateSource, JudgeTablesRefusal,
    };
    use crate::memory_pool::{RecordStreamSeam, RetainedWrite, WrittenContents};
    use crate::segments::StepKind;
    use singlefs_core::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    };

    /// 一次合成的写：盘 0、`offset` 起 4096 字节；根槽写的字节里实例代号在偏移 24、checkpoint_txg 在偏移 28（`root_identity_of_write` 的读法）。
    fn retained_write_built_for_the_table_tests(
        kind: StepKind,
        offset: u64,
        root_identity: Option<(u32, u64)>,
    ) -> RetainedWrite {
        let mut bytes = vec![0u8; 4096];
        if let Some((instance, checkpoint_txg)) = root_identity {
            bytes[24..28].copy_from_slice(&instance.to_le_bytes());
            bytes[28..36].copy_from_slice(&checkpoint_txg.to_le_bytes());
        }
        RetainedWrite {
            device: DeviceIdentity(0),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(bytes),
        }
    }

    /// 历史两次发布（实例 1 的 txg 1、2）、挂载一次发布（实例 2 的 txg 2）：写表下标 0..6 是历史、6..8 是挂载。
    fn history_of_two_publishes_then_a_mount_of_one() -> Vec<RetainedWrite> {
        vec![
            retained_write_built_for_the_table_tests(StepKind::UnitWrite, 1 << 24, None),
            retained_write_built_for_the_table_tests(StepKind::JournalRecord, 1 << 22, None),
            retained_write_built_for_the_table_tests(
                StepKind::RootRecordFua,
                1 << 20,
                Some((1, 1)),
            ),
            retained_write_built_for_the_table_tests(
                StepKind::UnitWrite,
                (1 << 24) + (1 << 15),
                None,
            ),
            retained_write_built_for_the_table_tests(
                StepKind::JournalRecord,
                (1 << 22) + 4096,
                None,
            ),
            retained_write_built_for_the_table_tests(
                StepKind::RootRecordFua,
                (1 << 20) + 4096,
                Some((1, 2)),
            ),
            retained_write_built_for_the_table_tests(
                StepKind::UnitWrite,
                (1 << 24) + (2 << 15),
                None,
            ),
            retained_write_built_for_the_table_tests(
                StepKind::RootRecordFua,
                (1 << 20) + 8192,
                Some((2, 2)),
            ),
        ]
    }

    /// 给定状态表的发布表认崩溃接缝（第三截照候选丁）：第一次崩溃截在历史第二次发布的根之前、恢复落到 txg 1，
    /// 接缝放在历史与挂载之间时，历史那一截只留落了根的那一次，挂载那次发布只带挂载写的单元；接缝放错一个位置（放在历史两次发布之间），
    /// 历史第二次发布照样进表、挂载的单元也分错——两份发布表不同，GPU 的记录核对器读的就是这一张，判得出接缝放错了。
    #[test]
    fn the_publishes_of_given_persisted_sets_follow_where_the_crash_seam_is_put() {
        let writes = history_of_two_publishes_then_a_mount_of_one();
        let persisted = vec![true, true, true, true, true, false, false, false];
        let state_source = JudgeStateSource::GivenPersistedSets {
            persisted_sets: vec![persisted],
            claimed_versions: vec![ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery],
        };
        let seam_at = |first_write_after_the_crash: usize| RecordStreamSeam::AfterACrash {
            first_write_after_the_crash,
            version_landed_on_after_the_crash: Some((InstanceGeneration(1), CheckpointTxg(1))),
        };
        let at_the_seam = judge_publishes(&writes, writes.len(), &state_source, &[seam_at(6)])
            .expect("历史那一截的根槽写只有一个掩码，核得过");
        let roots_and_units = |publishes: &[JudgePublish]| -> Vec<(u32, u32, u64, Vec<Vec<u32>>)> {
            publishes
                .iter()
                .map(|publish| {
                    (
                        publish.root_write,
                        publish.instance,
                        publish.checkpoint_txg,
                        publish.unit_copy_groups.clone(),
                    )
                })
                .collect()
        };
        assert_eq!(
            roots_and_units(&at_the_seam),
            vec![(2, 1, 1, vec![vec![0]]), (7, 2, 2, vec![vec![6]])],
            "接缝在历史与挂载之间：历史第二次发布没落根、恢复也没落到它，剔掉；挂载那次只带挂载写的单元"
        );
        let one_place_early = judge_publishes(&writes, writes.len(), &state_source, &[seam_at(3)])
            .expect("接缝之前那一截只有一个根槽写，核得过");
        assert_ne!(
            roots_and_units(&one_place_early),
            roots_and_units(&at_the_seam),
            "接缝放错位置，发布表不同"
        );
        assert_eq!(
            roots_and_units(
                &judge_publishes(&writes, writes.len(), &state_source, &[])
                    .expect("不带接缝是一条录制流")
            ),
            vec![
                (2, 1, 1, vec![vec![0]]),
                (5, 1, 2, vec![vec![3]]),
                (7, 2, 2, vec![vec![6]])
            ],
            "不带接缝时三次发布全在表里（今天的一条录制流）"
        );
    }

    /// 崩溃接缝之前某个根槽写在两个掩码里一落一不落：一张表只能按一种剔法分发布，建表拒、点名那个根槽写与掩码，不 panic。
    #[test]
    fn given_persisted_sets_that_disagree_on_a_root_write_before_the_crash_seam_are_refused() {
        let writes = history_of_two_publishes_then_a_mount_of_one();
        let landed_on_the_first = vec![true, true, true, true, true, false, false, false];
        let landed_on_the_second = vec![true, true, true, true, true, true, true, false];
        let refusal = judge_publishes(
            &writes,
            writes.len(),
            &JudgeStateSource::GivenPersistedSets {
                persisted_sets: vec![landed_on_the_first, landed_on_the_second],
                claimed_versions: vec![
                    ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
                    2
                ],
            },
            &[RecordStreamSeam::AfterACrash {
                first_write_after_the_crash: 6,
                version_landed_on_after_the_crash: Some((InstanceGeneration(1), CheckpointTxg(1))),
            }],
        )
        .expect_err("第 5 个写（历史第二次发布的根）在两个掩码里一落一不落");
        assert_eq!(
            refusal,
            JudgeTablesRefusal::GivenPersistedSetsDisagreeOnARootWriteBeforeACrashSeam {
                root_write: 5,
                position: 1
            }
        );
    }
}
