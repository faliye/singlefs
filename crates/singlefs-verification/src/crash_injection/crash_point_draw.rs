//! 按种子摆崩溃点：一段里持久哪几个写的真子集，短段整段枚举，与崩溃镜像落盘。

use super::*;

/// 摆崩溃状态的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;

/// 段内写数不超过这个数时，子集掩码直接用一个 u64 抽；更长的段逐写各抽一次。
pub(super) const WRITES_A_SUBSET_MASK_HOLDS: usize = 63;

/// 写表下标 `write` 落在第几段。
///
/// # Panics
/// 这个写不在任何一段里：段与写表不是同一条流切的（每个写都恰好落在一个段里，`split_into_segments`）。
pub(super) fn segment_containing_the_write(segments: &[Vec<usize>], write: usize) -> usize {
    segments
        .iter()
        .position(|segment| segment.contains(&write))
        .expect("每个写都恰好落在一个段里：段与写表是同一条流按同一条规则切的")
}

/// 按 `draw` 摆崩溃状态：候选段是「整段的写都落在 mkfs 之后、最后一个跑完的操作之前」的那几段，段内取真子集。
///
/// 起点那一段（取号、暖机、第一个文件）在内；mkfs 那几段不在内，与崩溃点重放同一条界：
/// mkfs 不是事务，它写到一半的盘面上还没有池，恢复报不出根是对的，拿事务的 oracle 去判只会判出假红
/// （崩溃点重放也从 `mkfs_operation_count` 之后起枚举）。失败那一步的写也不在内（模型还没判过它写出的根）。
///
/// 抽的那一路同一段同一个子集只摆一次；摆完按段号排好——基线只往前叠。
/// 靶向用例那一支（[`CrashPointDraw::ExactlyOneCrashState`]）只摆写死的那一个，持久集合可以越过段尾；写死的输入不对就 panic（见那一支）。
pub(super) fn draw_crash_points(
    seed: HistorySeed,
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
    marks: &StreamMarks,
    step_kinds: &[HistoryOperationKind],
    draw: CrashPointDraw,
) -> Vec<CrashPoint> {
    let candidate_segments: Vec<usize> = (0..segments.len())
        .filter(|segment_index| {
            segments[*segment_index].iter().all(|write| {
                (marks.after_make_filesystem..marks.after_the_last_finished_step)
                    .contains(&stream_indexes[*write])
            })
        })
        .collect();
    let mut source = SeededRandomSource::from_seed(seed.0 ^ CRASH_POINT_SEED_SALT);
    let mut chosen: BTreeSet<(usize, Vec<bool>)> = BTreeSet::new();
    match draw {
        CrashPointDraw::Sampled {
            crash_points_per_history,
        } => {
            // 一个候选段都没有时一个都不摆（下面按候选段数取随机下标，取不出）。
            if candidate_segments.is_empty() {
                return Vec::new();
            }
            for _ in 0..crash_points_per_history {
                let picked = usize::try_from(
                    source.below(u64::try_from(candidate_segments.len()).expect("候选段数")),
                )
                .expect("下标装得进 usize");
                let segment_index = candidate_segments[picked];
                let subset = draw_a_proper_subset(&mut source, segments[segment_index].len());
                chosen.insert((segment_index, subset));
            }
        }
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes,
            sampled_in_longer_segments,
        } => {
            for segment_index in candidate_segments {
                let length = segments[segment_index].len();
                if length <= segment_writes {
                    for mask in 0..(1u64 << length) - 1 {
                        chosen.insert((segment_index, bits_of(mask, length)));
                    }
                } else {
                    for _ in 0..sampled_in_longer_segments {
                        let subset = draw_a_proper_subset(&mut source, length);
                        chosen.insert((segment_index, subset));
                    }
                }
            }
        }
        CrashPointDraw::ExactlyOneCrashState {
            segment_index,
            writes_from_the_segment_on,
            withheld_among_them,
        } => {
            assert!(
                candidate_segments.contains(&segment_index),
                "靶向用例写死的第 {segment_index} 段不是候选段（整段落在 mkfs 之后、最后一个跑完的操作之前的那几段）：{candidate_segments:?}"
            );
            let first_write_of_the_segment = segments[segment_index][0];
            let writes_of_the_state =
                first_write_of_the_segment..first_write_of_the_segment + writes_from_the_segment_on;
            assert!(
                writes_from_the_segment_on > 0 && writes_of_the_state.end <= stream_indexes.len(),
                "靶向用例从第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过写表末尾（{} 个写）",
                stream_indexes.len()
            );
            // 段首已经在候选段里（落在 mkfs 之后）；这几个写的末尾不越过最后一个跑完的操作。
            assert!(
                stream_indexes[writes_of_the_state.end - 1] < marks.after_the_last_finished_step,
                "靶向用例从第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写越过了最后一个跑完的操作"
            );
            assert!(
                !withheld_among_them.is_empty()
                    && withheld_among_them
                        .iter()
                        .all(|withheld| *withheld < writes_from_the_segment_on),
                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从这一段起数的 {writes_from_the_segment_on} 个写里"
            );
            chosen.insert((
                segment_index,
                (0..writes_from_the_segment_on)
                    .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
                    .collect(),
            ));
        }
    }
    chosen
        .into_iter()
        .map(|(segment_index, persisted_within_the_segment)| {
            let first_withheld = persisted_within_the_segment
                .iter()
                .position(|persisted| !persisted)
                .expect("段内持久的是真子集，至少有一个写被扣下");
            // 段是写表里连续的一截：段内第 k 个写是段首加 k（靶向用例的持久集合越过段尾时也这样数）。
            let step = marks
                .step_containing(stream_indexes[segments[segment_index][0] + first_withheld])
                .expect("崩溃状态摆在最后一个跑完的操作之前，登记里一定找得到它那一步");
            let operation_kind = match step {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => step_kinds.get(step_index).copied(),
            };
            CrashPoint {
                segment_index,
                persisted_within_the_segment,
                step,
                operation_kind,
            }
        })
        .collect()
}

/// 掩码的低 `length` 位摊成逐写的持久与否。
fn bits_of(mask: u64, length: usize) -> Vec<bool> {
    (0..length).map(|bit| mask & (1 << bit) != 0).collect()
}

/// 抽一个真子集：段短时直接抽一个掩码（[0, 2^n − 1) 不含全持久），段长时逐写各抽一次、全中了再去掉一个。
/// 真子集这一条要守住：全持久等于「没崩在这一段」，那个状态由下一段的空子集摆。
fn draw_a_proper_subset(source: &mut SeededRandomSource, segment_length: usize) -> Vec<bool> {
    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {
        let mask = source.below((1u64 << segment_length) - 1);
        return bits_of(mask, segment_length);
    }
    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();
    if persisted.iter().all(|is_persisted| *is_persisted) {
        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))
            .expect("下标装得进 usize");
        persisted[withheld] = false;
    }
    persisted
}

/// 这个崩溃镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（F 那个 txg 上的根全属于被抛弃的实例）。
/// 与活盘面那一路（抬 F 之前的镜像 + 新 F）是同一个谓词、同一份实现，读的盘面不同：这里读崩溃镜像自己，
/// F 从镜像里最新那条根上取——抬 F 之后的任何一个崩溃状态都摆得出同一机理的盘面，不限于抬 F 那一步
/// （假阳那一半：写死成 None 会让收口表第 43 行那一形在崩溃状态上恒不匹配）。
fn raised_floor_of_the_newest_root_of_the_crash_image_lands_only_on_abandoned_roots(
    image: &CrashImage<'_>,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let newest_root = choose_root(image, &system_configuration)?;
    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
}

/// 一个崩溃状态的失败观察：拿去对「已知红」清单（`KNOWN_RED_FORMS`，与历史那一路同一张清单）。
pub(super) fn crash_state_observation(
    image: &CrashImage<'_>,
    crash_point: &CrashPoint,
    violations: Vec<(&'static str, String)>,
    model_disagreement: Option<ModelDisagreement>,
    record_check: RecordCheck,
) -> FailureObservation {
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count_of(image);
    FailureObservation {
        position: crash_point.step,
        operation_kind: crash_point.operation_kind,
        violations,
        panic: None,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement,
        raised_floor_lands_only_on_abandoned_roots:
            raised_floor_of_the_newest_root_of_the_crash_image_lands_only_on_abandoned_roots(image),
        record_check,
    }
}

/// 把一个崩溃状态落成镜像文件（一块盘一个稀疏文件）：只写这个状态里有内容的那些扇区，没写过的留成洞、读出全 0，
/// 与内存里那份逐字节相同。写不出来（目录建不出来、盘满）时交回 None：留现场是为了好查，不是判据，不因为它红。
pub(super) fn write_crash_image_files(
    image: &CrashImage<'_>,
    directory: &Path,
    seed: HistorySeed,
    crash_point: &CrashPoint,
    stage: CrashStateStage,
) -> Option<PathBuf> {
    use std::os::unix::fs::FileExt as _;
    let here = directory.join(format!(
        "seed-{}-segment-{}-{}",
        seed.0,
        crash_point.segment_index,
        stage.name()
    ));
    std::fs::create_dir_all(&here).ok()?;
    let sector_length = usize::try_from(SECTOR_BYTES).expect("512");
    for device in image.base.devices.keys().copied() {
        let file = std::fs::File::create(here.join(format!("device-{}.img", device.0))).ok()?;
        file.set_len(image.base.device_size_in_bytes).ok()?;
        let mut sectors: BTreeSet<u64> = image
            .base
            .devices
            .get(&device)
            .expect("这块盘在基线里")
            .written_sectors_in(DeviceOffsetInBytes(0), image.base.device_size_in_bytes)
            .into_iter()
            .collect();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let first_sector = write.offset.0 / SECTOR_BYTES;
            let sector_count = write.length_in_bytes() / SECTOR_BYTES;
            sectors.extend(first_sector..first_sector + sector_count);
        }
        for sector in sectors {
            let offset = DeviceOffsetInBytes(sector * SECTOR_BYTES);
            let bytes = PoolReader::read(image, device, offset, sector_length)?;
            file.write_all_at(&bytes, offset.0).ok()?;
        }
    }
    Some(here)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::generate_history;

    /// 同一个种子摆出来的崩溃状态逐项相同；段内持久的是真子集、按段号排好。
    #[test]
    fn crash_points_are_reproducible_proper_subsets_sorted_by_segment() {
        let history = generate_history(HistorySeed(11), 12);
        let first = inject_crashes_into_history(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        let second = inject_crashes_into_history(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            FOUR_CRASH_POINTS,
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        assert_eq!(first, second, "同一个种子、同一个跑法，逐项相同");
        assert!(
            first.tally.crash_points >= 1,
            "种子 11 的 12 步里摆得出崩溃状态：{:?}",
            first.tally
        );
        assert_eq!(
            first.tally.model_judgements, first.tally.crash_points,
            "每个崩溃状态都问过模型"
        );
        assert_eq!(
            first.tally.checker_runs, first.tally.crash_points,
            "每个崩溃状态都跑过 checker"
        );
        assert_eq!(
            first.tally.record_checks, first.tally.crash_points,
            "每个崩溃状态都跑过记录核对器"
        );
        assert!(
            first.crash_points.iter().all(|crash_point| crash_point
                .persisted_within_the_segment
                .iter()
                .any(|persisted| !persisted)),
            "段内持久的是真子集：{:?}",
            first.crash_points
        );
        assert!(
            first
                .crash_points
                .windows(2)
                .all(|pair| pair[0].segment_index <= pair[1].segment_index),
            "崩溃状态按段号排好"
        );
    }

    /// 段内扣下的写四种落点都见得到，段内有洞（后发的写先持久）的状态真的摆出来了，起点那一段也摆得到。
    #[test]
    fn crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment() {
        let mut tally = CrashInjectionTally::default();
        for seed in 0..8u64 {
            let history = generate_history(HistorySeed(seed), 16);
            let injection = inject_crashes_into_history(
                &history,
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                CrashPointDraw::Sampled {
                    crash_points_per_history: 6,
                },
                None,
                given_states_engine_for("crash_injection_unit_tests").engine(),
            );
            tally.absorb(&injection.tally);
        }
        for kind in [
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
        ] {
            assert!(
                tally
                    .crash_points_withholding_write_kind
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0)
                    >= 1,
                "八个种子里一次都没扣下 {}：{:?}",
                kind.name(),
                tally.crash_points_withholding_write_kind
            );
        }
        assert!(
            tally.crash_points_that_land_before_the_last_committed_version >= 1,
            "一个崩溃状态都没截回到最后一版之前"
        );
        assert!(
            tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
            "一个段内有洞的状态都没摆出来（等于还是只截前缀）：{tally:?}"
        );
        assert!(
            tally.crash_points_inside_the_starting_point >= 1,
            "起点那一段里一个崩溃状态都没摆到（起点段 2026-09-20 才放开）：{tally:?}"
        );
    }

    /// 起点那一段也在候选里：从第一个文件起的历史，崩溃状态落得到起点那一步
    /// （K1-d：默认批 7219 次写里有 627 次落在起点那一步）。
    #[test]
    fn crash_points_fall_inside_the_starting_point_too() {
        let history = GeneratedHistory {
            seed: HistorySeed(3),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: generate_history(HistorySeed(3), 10).operations,
        };
        let injection = inject_crashes_into_history(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            CrashPointDraw::EveryProperSubsetOfShortSegments {
                segment_writes: 2,
                sampled_in_longer_segments: 2,
            },
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        assert!(injection.tally.crash_points >= 1);
        assert!(
            injection
                .crash_points
                .iter()
                .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
            "起点那一段里一个崩溃状态都没摆到：{:?}",
            injection
                .crash_points
                .iter()
                .map(|crash_point| crash_point.step)
                .collect::<Vec<StepPosition>>()
        );
    }
}
