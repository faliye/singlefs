//! 给定状态的两段判：mkfs 之后的历史与第一次崩溃的状态表，在挂载之后的池与二次崩溃上接着判记录核对器与参照文件系统。

use super::*;

impl<'history> HistoryAfterTheMakeFilesystem<'history> {
    /// 空盘叠上 mkfs 的写就是基线；写表从 mkfs 第一个不是清零的写起（`writes` 是整条写表，与 `stream_indexes` 逐条对应，
    /// `after_make_filesystem` 是 mkfs 写完时录制流的下标）。
    ///
    /// # Panics
    /// mkfs 的清零写不都在它的数据写之前（叠回去的数据写会盖掉清零，基线就不再是这张写表在全落时的样子）。
    #[must_use]
    pub fn of(
        writes: &'history [RetainedWrite],
        segments: &[Vec<usize>],
        stream_indexes: &[usize],
        after_make_filesystem: usize,
        execution: HistoryExecution,
    ) -> Self {
        let writes_of_the_make_filesystem =
            writes_of_the_make_filesystem(stream_indexes, after_make_filesystem);
        let first_write_of_the_table = writes[..writes_of_the_make_filesystem]
            .iter()
            .position(|write| write.kind != StepKind::ZeroFill)
            .unwrap_or(writes_of_the_make_filesystem);
        assert!(
            writes[first_write_of_the_table..writes_of_the_make_filesystem]
                .iter()
                .all(|write| write.kind != StepKind::ZeroFill),
            "mkfs 先发清零、再发数据写（make_filesystem 的次序）：清零之后再没有清零"
        );
        let mut base = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        base.apply_writes(&writes[..writes_of_the_make_filesystem]);
        Self {
            base,
            writes: &writes[first_write_of_the_table..],
            segments: segments_from_the_write(segments, first_write_of_the_table),
            first_write_of_the_table,
            writes_of_the_make_filesystem_in_the_table: writes_of_the_make_filesystem
                - first_write_of_the_table,
        }
    }

    #[must_use]
    pub fn base_pool(&self) -> &MemoryPool {
        &self.base
    }

    /// 这张写表：mkfs 的数据写在头上，接着是 mkfs 之后的历史。
    #[must_use]
    pub fn writes(&self) -> &'history [RetainedWrite] {
        self.writes
    }

    /// 这张写表的切段（下标按这张写表）。
    #[must_use]
    pub fn segments(&self) -> &[Vec<usize>] {
        &self.segments
    }

    /// 这张写表头上 mkfs 写的那几个（创世那几次发布的单元写、根槽写与系统配置槽写）。
    #[must_use]
    pub fn writes_of_the_make_filesystem(&self) -> &'history [RetainedWrite] {
        &self.writes[..self.writes_of_the_make_filesystem_in_the_table]
    }

    /// 整条写表的下标换成这张写表的下标。
    ///
    /// # Panics
    /// 那个写是 mkfs 的清零写（不在这张写表里）。
    #[must_use]
    pub fn index_in_the_table(&self, index_over_the_whole_write_table: usize) -> usize {
        index_over_the_whole_write_table
            .checked_sub(self.first_write_of_the_table)
            .expect("清零写只在基线里：问到的写在 mkfs 的清零写之后")
    }

    /// 整条写表上的一个持久集合换成这张写表的。
    ///
    /// # Panics
    /// mkfs 的写在这个持久集合里有一个没落（崩溃状态只摆在整段落在 mkfs 之后的段上，`draw_crash_points` 的候选段）。
    #[must_use]
    pub fn persisted_set_of_the_table(
        &self,
        persisted_over_the_whole_write_table: &[bool],
    ) -> Vec<bool> {
        assert!(
            persisted_over_the_whole_write_table
                [..self.first_write_of_the_table + self.writes_of_the_make_filesystem_in_the_table]
                .iter()
                .all(|is_persisted| *is_persisted),
            "崩溃状态摆在整段落在 mkfs 之后的段上：mkfs 的写都已落"
        );
        persisted_over_the_whole_write_table[self.first_write_of_the_table..].to_vec()
    }

    /// 整条写表下标上的掉电之后第一个写换成这张写表上的接缝。
    ///
    /// # Panics
    /// 那次掉电在 mkfs 写完之前（mkfs 不掉电：随机历史的掉电是历史里一步，在起点之后）。
    fn power_loss_seams_of_the_table(
        &self,
        first_write_after_each_power_loss: &[usize],
    ) -> Vec<RecordStreamSeam> {
        power_loss_seams(
            &first_write_after_each_power_loss
                .iter()
                .map(|first_write_after_the_power_loss| {
                    assert!(
                        *first_write_after_the_power_loss
                            >= self.first_write_of_the_table
                                + self.writes_of_the_make_filesystem_in_the_table,
                        "掉电是历史里的一步，在 mkfs 写完之后"
                    );
                    self.index_in_the_table(*first_write_after_the_power_loss)
                })
                .collect::<Vec<usize>>(),
        )
    }
}

/// 第一截：这段历史上每个第一次崩溃状态的池级 checker、两遍恢复、记录核对器交 `judge` 判，掉电接缝相同的几个状态一张表
/// （记录核对器照每个状态之前掉过的电断开，第一次崩溃之前没掉过电的是一条录制流）。交回与 `crash_points` 同序的结论；
/// 计数（判绿、没判到）并进 `tally`。判不了就 panic：配了 GPU 卡用不上、判定库读写失败都判红，不退回 CPU。
#[allow(
    clippy::too_many_arguments,
    reason = "判器、历史、崩溃状态、整条写表的切段与掉电接缝、计数，各是各的东西"
)]
pub(super) fn judge_the_first_crashes_on_given_states_tables(
    judge: &dyn GivenStatesJudge,
    seed: HistorySeed,
    history_after_the_make_filesystem: &HistoryAfterTheMakeFilesystem<'_>,
    crash_points: &[CrashPoint],
    segments: &[Vec<usize>],
    write_count: usize,
    first_write_after_each_power_loss: &[usize],
    tally: &mut CrashInjectionTally,
) -> Vec<GivenStateConclusion> {
    let mut crash_points_by_power_losses: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
    for (crash_point_index, crash_point) in crash_points.iter().enumerate() {
        let first_write_of_the_segment = segments[crash_point.segment_index][0];
        let power_losses_before_the_crash: Vec<usize> = first_write_after_each_power_loss
            .iter()
            .copied()
            .filter(|first_write_after_the_power_loss| {
                *first_write_after_the_power_loss <= first_write_of_the_segment
            })
            .collect();
        crash_points_by_power_losses
            .entry(power_losses_before_the_crash)
            .or_default()
            .push(crash_point_index);
    }
    let mut conclusions: Vec<Option<GivenStateConclusion>> = vec![None; crash_points.len()];
    for (power_losses, crash_point_indexes) in crash_points_by_power_losses {
        let seams = history_after_the_make_filesystem.power_loss_seams_of_the_table(&power_losses);
        let persisted_sets: Vec<Vec<bool>> = crash_point_indexes
            .iter()
            .map(|crash_point_index| {
                let crash_point = &crash_points[*crash_point_index];
                history_after_the_make_filesystem.persisted_set_of_the_table(
                    &persisted_over_the_whole_write_table(
                        write_count,
                        segments[crash_point.segment_index][0],
                        &crash_point.persisted_within_the_segment,
                    ),
                )
            })
            .collect();
        let table = GivenStatesOfATable {
            table_name: format!("seed={}/first-crash/power-losses={power_losses:?}", seed.0),
            base: &history_after_the_make_filesystem.base,
            writes: history_after_the_make_filesystem.writes,
            segments: &history_after_the_make_filesystem.segments,
            seams: &seams,
            persisted_sets,
        };
        let judged = judge
            .judge_the_given_states(&table)
            .unwrap_or_else(|reason| {
                panic!(
                    "第一截给定状态表 {} 判不了（不退回别处判）：{reason}",
                    table.table_name
                )
            });
        assert_eq!(
            judged.states.len(),
            crash_point_indexes.len(),
            "判器逐状态交回结论，与表里的状态同序"
        );
        absorb_the_pool_checker_counts_of_the_table(tally, &judged.count_vector);
        count_a_table_left_to_another_host(tally, judged.source);
        for (crash_point_index, conclusion) in crash_point_indexes.into_iter().zip(judged.states) {
            conclusions[crash_point_index] = Some(conclusion);
        }
    }
    conclusions
        .into_iter()
        .map(|conclusion| conclusion.expect("每个崩溃状态恰好落在一组掉电接缝里"))
        .collect()
}

impl<'history> GivenStatesAfterTheFirstCrash<'history> {
    /// `persisted_over_the_whole_history` 是这个第一次崩溃状态的持久集合，与整条历史写表（mkfs 的清零写也算）逐条对应；
    /// `first_write_after_each_power_loss_before_the_crash` 是第一次崩溃之前每次掉电之后第一个写在整条历史写表里的下标，从小到大
    /// （没掉过电的给空表）；`segments_of_the_publish_after_the_mount` 按之后那一次发布自己那段写表的下标。
    ///
    /// # Panics
    /// mkfs 的写在持久集合里有一个没落（[`HistoryAfterTheMakeFilesystem::persisted_set_of_the_table`]）；一道掉电接缝落在 mkfs 写完之前。
    #[must_use]
    pub fn new(
        judge: &'history dyn GivenStatesJudge,
        history_after_the_make_filesystem: &'history HistoryAfterTheMakeFilesystem<'history>,
        persisted_over_the_whole_history: &[bool],
        first_write_after_each_power_loss_before_the_crash: &[usize],
        version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
        writes_of_the_publish_after_the_mount: &'history [RetainedWrite],
        segments_of_the_publish_after_the_mount: &'history [Vec<usize>],
    ) -> Self {
        Self {
            judge,
            history_after_the_make_filesystem,
            persisted_history_of_the_table: history_after_the_make_filesystem
                .persisted_set_of_the_table(persisted_over_the_whole_history),
            power_loss_seams_of_the_table: history_after_the_make_filesystem
                .power_loss_seams_of_the_table(first_write_after_each_power_loss_before_the_crash),
            version_landed_on_after_the_first_crash,
            writes_of_the_publish_after_the_mount,
            segments_of_the_publish_after_the_mount,
        }
    }

    /// 第二截那张表：挂载与之后那一次发布之后的池（实现恢复后的镜像：挂载那一段与之后那次发布全落）与第一次崩溃那份镜像（崩溃态镜像：
    /// 挂载那一段与之后那次发布一个写都没落）是这张表的两个状态，次序照 [`StateOfTheTableAfterTheWritableMount::ALL`]。
    /// 两个状态在历史那一截上的持久集合相同，记录核对器两份镜像各判一条（[`ConclusionsAfterTheWritableMount::record_check`]）。
    /// 挂载之后的池那个状态那一栏恢复自称的那一版写挂载与那次发布写出的最新那条根（[`newest_root_written_after_the_crash`]，D13（验证路线）
    /// 已定项 7「照字面改：表里记下根的来路」），判段照它判第二条判据、与恢复落到的根对不上判红；第一次崩溃那份镜像那个状态不写。
    #[must_use]
    pub fn table_after_the_writable_mount(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        table_name: String,
    ) -> GivenStatesTableAfterTheFirstCrash<'history> {
        let states: Vec<(Vec<bool>, PublishAfterTheMountInTheState)> =
            StateOfTheTableAfterTheWritableMount::ALL
                .iter()
                .map(|state| match state {
                    StateOfTheTableAfterTheWritableMount::PoolAfterTheMount => (
                        vec![true; mount_writes.len()],
                        PublishAfterTheMountInTheState::Landed,
                    ),
                    StateOfTheTableAfterTheWritableMount::FirstCrashImage => (
                        vec![false; mount_writes.len()],
                        PublishAfterTheMountInTheState::NotWrittenYet,
                    ),
                })
                .collect();
        let mut table = self.table_of_the_states(mount_writes, mount_segments, &states, table_name);
        let history_write_count = self.history_after_the_make_filesystem.writes.len();
        table.claimed_versions
            [StateOfTheTableAfterTheWritableMount::PoolAfterTheMount.position()] =
            ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(
                newest_root_written_after_the_crash(
                    &table.writes[history_write_count..],
                    self.version_landed_on_after_the_first_crash,
                ),
            );
        table
    }

    /// 第二截：[`Self::table_after_the_writable_mount`] 那张表交判器判，交回两个状态各自的结论。判不了就 panic
    /// （同 [`judge_the_first_crashes_on_given_states_tables`]）。
    #[must_use]
    pub fn judge_the_pool_after_the_writable_mount(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        table_name: String,
    ) -> ConclusionsAfterTheWritableMount {
        let table = self.table_after_the_writable_mount(mount_writes, mount_segments, table_name);
        let judged = judge_the_given_states_after_the_mount(self, &table);
        let conclusion_of =
            |state: StateOfTheTableAfterTheWritableMount| judged.states[state.position()].clone();
        ConclusionsAfterTheWritableMount {
            on_the_pool_after_the_mount: conclusion_of(
                StateOfTheTableAfterTheWritableMount::PoolAfterTheMount,
            ),
            on_the_first_crash_image: conclusion_of(
                StateOfTheTableAfterTheWritableMount::FirstCrashImage,
            ),
            source: judged.source,
        }
    }

    /// 第二、三截那几张表的同一种建法：写表是 mkfs 之后的历史接挂载那一段、再接之后那一次发布，接缝 = 第一次崩溃之前的掉电接缝 +
    /// 历史与挂载之间那一道崩溃接缝；`states` 里每个状态给挂载那一段的持久集合（与整条挂载流逐条对应）与之后那一次发布落没落，
    /// 持久集合 = 历史那一截（这个第一次崩溃状态的）接上它们。
    ///
    /// # Panics
    /// 一个状态的挂载那一段持久集合与整条挂载流不一样长。
    fn table_of_the_states(
        &self,
        mount_writes: &[RetainedWrite],
        mount_segments: &[Vec<usize>],
        states: &[(Vec<bool>, PublishAfterTheMountInTheState)],
        table_name: String,
    ) -> GivenStatesTableAfterTheFirstCrash<'history> {
        let history = self.history_after_the_make_filesystem;
        let history_write_count = history.writes.len();
        let writes: Vec<RetainedWrite> = history
            .writes
            .iter()
            .chain(mount_writes)
            .chain(self.writes_of_the_publish_after_the_mount)
            .cloned()
            .collect();
        let shifted = |segments: &[Vec<usize>], by: usize| -> Vec<Vec<usize>> {
            segments
                .iter()
                .map(|segment| segment.iter().map(|write| write + by).collect())
                .collect()
        };
        let segments: Vec<Vec<usize>> = history
            .segments
            .iter()
            .cloned()
            .chain(shifted(mount_segments, history_write_count))
            .chain(shifted(
                self.segments_of_the_publish_after_the_mount,
                history_write_count + mount_writes.len(),
            ))
            .collect();
        let mut seams = self.power_loss_seams_of_the_table.clone();
        seams.push(RecordStreamSeam::AfterACrash {
            first_write_after_the_crash: history_write_count,
            version_landed_on_after_the_crash: self.version_landed_on_after_the_first_crash,
        });
        let persisted_sets: Vec<Vec<bool>> = states
            .iter()
            .map(|(persisted_in_the_mount, publish_after_the_mount)| {
                assert_eq!(
                    persisted_in_the_mount.len(),
                    mount_writes.len(),
                    "挂载那一段的持久集合与整条挂载流逐条对应"
                );
                let publish_after_the_mount_persisted = match publish_after_the_mount {
                    PublishAfterTheMountInTheState::Landed => true,
                    PublishAfterTheMountInTheState::NotWrittenYet => false,
                };
                self.persisted_history_of_the_table
                    .iter()
                    .chain(persisted_in_the_mount)
                    .copied()
                    .chain(std::iter::repeat_n(
                        publish_after_the_mount_persisted,
                        self.writes_of_the_publish_after_the_mount.len(),
                    ))
                    .collect()
            })
            .collect();
        let claimed_versions =
            vec![ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery; persisted_sets.len()];
        GivenStatesTableAfterTheFirstCrash {
            table_name,
            base: &history.base,
            writes,
            segments,
            seams,
            persisted_sets,
            claimed_versions,
        }
    }
}

/// 第二、三截的一张给定状态表（[`GivenStatesAfterTheFirstCrash`] 建）：基线是 mkfs 之后的池，写表是 mkfs 之后的历史接挂载那一段、
/// 再接之后那一次发布，接缝 = 第一次崩溃之前的掉电接缝 + 历史与挂载之间那一道崩溃接缝（第三截照候选丁，
/// 与 [`HistoryThenWritableMountRecords`] 交 CPU 记录核对器的同一种断法；GPU 的发布表照它分），每个状态的持久集合与写表逐条对应，
/// 每个状态那一栏恢复自称的那一版（`claimed_versions`，与持久集合逐个对应）：第二截挂载之后的池那个状态写挂载与那次发布写出的最新那条根，
/// 别的状态不写（[`ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery`]）。
pub struct GivenStatesTableAfterTheFirstCrash<'history> {
    pub table_name: String,
    pub base: &'history MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub seams: Vec<RecordStreamSeam>,
    pub persisted_sets: Vec<Vec<bool>>,
    pub claimed_versions: Vec<ClaimedVersionOfAGivenState>,
}

impl GivenStatesTableAfterTheFirstCrash<'_> {
    /// 当成一张给定状态表交判器（借这一份的写表、切段与接缝；恢复自称的那一版那一栏另取 [`Self::claimed_versions_of_the_states`]）。
    #[must_use]
    pub fn given_states_table(&self) -> GivenStatesOfATable<'_> {
        GivenStatesOfATable {
            table_name: self.table_name.clone(),
            base: self.base,
            writes: &self.writes,
            segments: &self.segments,
            seams: &self.seams,
            persisted_sets: self.persisted_sets.clone(),
        }
    }

    /// 这张表每个状态那一栏恢复自称的那一版（与 [`Self::given_states_table`] 一起交判器：[`GivenStatesOfATableWithItsColumns`]）。
    #[must_use]
    pub fn claimed_versions_of_the_states(&self) -> ClaimedVersionsOfTheStates<'_> {
        ClaimedVersionsOfTheStates::OfEachState(&self.claimed_versions)
    }
}

/// 挂载与之后那一次发布写出的最新那条根（D13（验证路线） 已定项 7 第二截「恢复自称的那一版取挂载与那次发布写出的最新那条根，
/// 挂载一条根都没写出时退回第一次崩溃落到的那一版」）：`writes_after_the_crash`（第一次崩溃那道接缝之后的写表：挂载那一段接之后那次发布）里
/// 按写的次序最后一个根槽写写出的根；一个根槽写都没有时是 `version_landed_on_after_the_first_crash`（第一次崩溃之后看 journal 那一遍恢复落到的那一版，
/// 那一遍恢复失败是 None）。第二截给定状态表那一栏（[`GivenStatesAfterTheFirstCrash::table_after_the_writable_mount`]）与两份镜像那一路
/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）都照它取。
///
/// # Panics
/// 根槽写不带字节（根槽 FUA 写是普通写，录制器照字节录）。
#[must_use]
pub fn newest_root_written_after_the_crash(
    writes_after_the_crash: &[RetainedWrite],
    version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
) -> Option<(InstanceGeneration, CheckpointTxg)> {
    writes_after_the_crash
        .iter()
        .rev()
        .find(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| {
            root_identity_written_by(write.bytes().expect("根槽 FUA 写是普通写，带着字节"))
        })
        .or(version_landed_on_after_the_first_crash)
}

/// 第二截那张表里的两个状态（[`GivenStatesAfterTheFirstCrash::table_after_the_writable_mount`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateOfTheTableAfterTheWritableMount {
    /// 挂载与之后那一次发布之后的池：D13（验证路线） 已定项 7 的实现恢复后的镜像（可写挂载会改盘）。
    PoolAfterTheMount,
    /// 第一次崩溃那份镜像：D13（验证路线） 已定项 7 的崩溃态镜像。
    FirstCrashImage,
}

impl StateOfTheTableAfterTheWritableMount {
    /// 表里状态的次序。
    pub const ALL: [StateOfTheTableAfterTheWritableMount; 2] = [
        StateOfTheTableAfterTheWritableMount::PoolAfterTheMount,
        StateOfTheTableAfterTheWritableMount::FirstCrashImage,
    ];

    /// 这个状态在表里的下标（[`Self::ALL`] 里的位置）。
    #[must_use]
    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|state| *state == self)
            .expect("ALL 里每个成员恰好出现一次")
    }
}

/// 第二截那张表的结论：挂载与之后那一次发布之后的池上那个状态的、第一次崩溃那份镜像上那个状态的，连同这张表的结论从哪来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionsAfterTheWritableMount {
    /// 池级 checker 判的就是这一份（挂完的盘）。
    pub on_the_pool_after_the_mount: GivenStateConclusion,
    /// 只取它的记录核对器第一条判据（[`Self::record_check`]）；它的池级 checker 与两遍恢复第一截在同一份镜像上已判过。
    pub on_the_first_crash_image: GivenStateConclusion,
    pub source: GivenStatesConclusionsSource,
}

impl ConclusionsAfterTheWritableMount {
    /// 第二截记录核对器的两条判据（D13（验证路线） 已定项 7「崩溃后镜像是两份」）：第一条（根在而记录一条都不在）读崩溃态镜像，
    /// 取第一次崩溃那个状态的；第二条（恢复自称新态而单元缺席）读实现恢复后的镜像，取挂载之后的池那个状态的。两个状态写表与接缝相同、
    /// 历史那一截的持久集合相同，分出的发布也相同（`publishes_in` 在崩溃接缝之前只看那几次发布的根槽写落没落，接缝之后整截都分），
    /// 所以第一次崩溃那个状态上的第一条判据就是两份镜像那一判的第一条（`crate::crash::check_records_against` 的第一条判据只读崩溃态镜像）。
    #[must_use]
    pub fn record_check(&self) -> RecordCheck {
        RecordCheck {
            root_without_record: self
                .on_the_first_crash_image
                .record_check()
                .root_without_record,
            claimed_state_missing_unit: self
                .on_the_pool_after_the_mount
                .record_check()
                .claimed_state_missing_unit,
        }
    }

    /// 第二截挂载之后的池上判红的那几样（池级 checker 判的就是这一份）：池级 checker 违例逐条，再加一条
    /// [`RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN`]——这个状态那一栏写的是挂载与那次发布写出的
    /// 最新那条根（D13（验证路线） 已定项 7），看 journal 那一遍恢复在这一份上落到的根却不是它（恢复落旧了、或落到更新的一版）。
    #[must_use]
    pub fn violations_on_the_pool_after_the_mount(&self) -> Vec<(&'static str, String)> {
        let mut violations = self.on_the_pool_after_the_mount.pool_checker_violations();
        if self
            .on_the_pool_after_the_mount
            .claimed_version_is_not_landed_on()
        {
            let landed_on = self
                .on_the_pool_after_the_mount
                .words
                .consulted_pass_landed_root;
            let landed_on_text = landed_on.landed_root().map_or_else(
                || format!("没落到根（结论里落到的根那几个字是 {landed_on:?}）"),
                |(instance, checkpoint_txg)| {
                    format!("实例 {}、checkpoint_txg {}", instance.0, checkpoint_txg.0)
                },
            );
            violations.push((
                RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN,
                format!(
                    "挂载与那次发布之后的池上看 journal 那一遍恢复落到的根（{landed_on_text}）不是挂载与那次发布写出的最新那条根"
                ),
            ));
        }
        violations
    }
}

/// 第二截挂载之后的池上那一条判红的名字（[`ConclusionsAfterTheWritableMount::violations_on_the_pool_after_the_mount`]）。
pub const RECOVERY_AFTER_THE_MOUNT_LANDED_ON_ANOTHER_VERSION_THAN_THE_NEWEST_ROOT_WRITTEN: &str =
    "恢复落到的根不是挂载与那次发布写出的最新那条根";

/// 第三截：这个第一次崩溃状态上每个二次崩溃的池级 checker、两遍恢复、记录核对器交判器判（一张表第三截照候选丁；
/// 两份崩溃后镜像都是二次崩溃镜像，只读恢复不改盘）。`persisted_in_the_mount_of_each_second_crash` 里每个与整条挂载流逐条对应；
/// 之后那一次发布一个写都还没发。交回与它同序的结论。判不了就 panic（同 [`judge_the_first_crashes_on_given_states_tables`]）。
pub(super) fn judge_the_second_crashes_on_a_given_states_table(
    given: &GivenStatesAfterTheFirstCrash<'_>,
    mount_writes: &[RetainedWrite],
    mount_segments: &[Vec<usize>],
    persisted_in_the_mount_of_each_second_crash: &[Vec<bool>],
    table_name: String,
) -> GivenStatesConclusionsOfATable {
    let states: Vec<(Vec<bool>, PublishAfterTheMountInTheState)> =
        persisted_in_the_mount_of_each_second_crash
            .iter()
            .map(|persisted_in_the_mount| {
                (
                    persisted_in_the_mount.clone(),
                    PublishAfterTheMountInTheState::NotWrittenYet,
                )
            })
            .collect();
    let table = given.table_of_the_states(mount_writes, mount_segments, &states, table_name);
    judge_the_given_states_after_the_mount(given, &table)
}

/// 第二、三截的一张表交判器判，交回与表里的状态同序的结论；判不了就 panic（不退回别处判）。
fn judge_the_given_states_after_the_mount(
    given: &GivenStatesAfterTheFirstCrash<'_>,
    table_after_the_first_crash: &GivenStatesTableAfterTheFirstCrash<'_>,
) -> GivenStatesConclusionsOfATable {
    let table = table_after_the_first_crash.given_states_table();
    let judged = given
        .judge
        .judge_the_given_states_with_its_columns(GivenStatesOfATableWithItsColumns {
            table: &table,
            container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
            claimed_versions: table_after_the_first_crash.claimed_versions_of_the_states(),
        })
        .unwrap_or_else(|reason| {
            panic!(
                "挂载之后的给定状态表 {} 判不了（不退回别处判）：{reason}",
                table.table_name
            )
        });
    assert_eq!(
        judged.states.len(),
        table.persisted_sets.len(),
        "判器逐状态交回结论，与表里的状态同序"
    );
    judged
}

/// RefFS 那一票（第一截）：照崩溃时刻交事件、判恢复读回的东西。比的是 RefFS 翻译的历史与实现读回的内容，不是一张镜像，上不了 GPU
/// （豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 `reference-filesystem` 那一行点头）。
// gpu-exempt: reference-filesystem
pub(super) fn reference_filesystem_judgement_of_a_crash_image(
    reference_filesystem_history: &TranslatedHistory,
    segment_the_crash_closes: usize,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemJudgementOfACrash {
    reference_filesystem_history.judge_a_crash(
        reference_filesystem_history.crash_moment_in_segment(segment_the_crash_closes),
        read_back,
    )
}

/// RefFS 那一票（二次崩溃那一截）：接着第一次崩溃判绿时重新起头的 RefFS 判（理由同 [`reference_filesystem_judgement_of_a_crash_image`]）。
// gpu-exempt: reference-filesystem
pub(super) fn reference_filesystem_vote_on_a_second_crash(
    reference_filesystem_at_the_first_crash: &ReferenceFileSystemJudgementOfACrash,
    read_back_after_the_second_crash: &ObservedReadBack,
) -> ReferenceFileSystemVote {
    judge_a_second_crash_after_the_restart(
        reference_filesystem_at_the_first_crash,
        read_back_after_the_second_crash,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 盖掉创世树表单元两份副本的两个单元写（与 mkfs 写的那两份同盘、同偏移、同长，内容换成别的字节），交回时连同 mkfs 那份副本在写表里的下标。
    /// 树表那一份是 mkfs 单元写里偏移最大的那一组（实例表在单元区起点、树表紧跟它，`make_filesystem` 的 `tree_table_genesis_placement_at`）。
    fn writes_overwriting_the_genesis_tree_table(
        history_after_the_make_filesystem: &HistoryAfterTheMakeFilesystem<'_>,
    ) -> (Vec<RetainedWrite>, usize) {
        let genesis_units: Vec<(usize, &RetainedWrite)> = history_after_the_make_filesystem
            .writes_of_the_make_filesystem()
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::UnitWrite)
            .collect();
        let tree_table_offset = genesis_units
            .iter()
            .map(|(_, write)| write.offset)
            .max()
            .expect("mkfs 写了实例表与树表两个单元");
        let tree_table_copies: Vec<(usize, &RetainedWrite)> = genesis_units
            .into_iter()
            .filter(|(_, write)| write.offset == tree_table_offset)
            .collect();
        assert_eq!(
            tree_table_copies.len(),
            2,
            "前提：创世树表单元两块盘各一份副本"
        );
        let overwrites = tree_table_copies
            .iter()
            .map(|(_, copy)| RetainedWrite {
                device: copy.device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: copy.offset,
                contents: crate::memory_pool::WrittenContents::Bytes(vec![
                    0xA5;
                    usize::try_from(
                        copy.length_in_bytes()
                    )
                    .expect(
                        "单元长装得进 usize"
                    )
                ]),
            })
            .collect();
        (overwrites, tree_table_copies[0].0)
    }

    /// 创世发布进记录核对器（「先查 GPU 能不能核，不能就 CPU 核一遍」，查下来 GPU 核得到、走 GPU 的发布表）：
    /// 那段历史跑完、每个写都落了，末尾再种两个盖掉创世树表单元两份副本的单元写（根环里还有创世那三条根，回收谓词证得出这次复用不合法），
    /// 给定状态表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU「能走两段的都走两段」），
    /// 记录核对器判「恢复自称的状态缺了一次发布的单元」；不种那两个写时同一张表不红。
    /// 判别力：给定状态表的写表把 mkfs 的写整截留在表外（基线照旧是 mkfs 之后），创世发布不进发布表，种了也不红（`crates/mutations.tsv` 那一行）。
    // crash-case-check:not-a-crash-case 只判写死的两个给定状态（种与不种），不枚举崩溃状态
    #[test]
    fn genesis_unit_overwritten_by_a_reuse_the_reclaim_predicate_refuses_is_red_on_the_record_checker_of_the_given_states_judge(
    ) {
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, stream_indexes, after_make_filesystem) =
            whole_write_table_of_the_history(&history, execution);
        let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
            &writes,
            &segments,
            &stream_indexes,
            after_make_filesystem,
            execution,
        );
        let (overwrites, genesis_copy) =
            writes_overwriting_the_genesis_tree_table(&history_after_the_make_filesystem);
        let healthy_writes = history_after_the_make_filesystem.writes();
        let planted_writes: Vec<RetainedWrite> =
            healthy_writes.iter().chain(&overwrites).cloned().collect();
        assert!(
            !crate::crash::reuse_is_not_proven_illegal_by_the_reclaim_predicate(
                &planted_writes,
                crate::crash::checkpoint_txg_of_the_publish_that_made_the_write(
                    &planted_writes,
                    genesis_copy
                ),
                planted_writes.len() - 1,
            ),
            "前提：创世那几条根还在根环里，回收谓词证得出盖掉创世树表单元不合法"
        );
        let engine_on_its_own_library = given_states_engine_for("crash_injection_unit_tests");
        let engine = engine_on_its_own_library.engine();
        let record_check_with_every_write_persisted = |table_writes: &[RetainedWrite]| {
            let mut segments_of_the_table = history_after_the_make_filesystem.segments().to_vec();
            if table_writes.len() > healthy_writes.len() {
                segments_of_the_table.push((healthy_writes.len()..table_writes.len()).collect());
            }
            let conclusions = engine
                .judge_the_given_states(&GivenStatesOfATable {
                    table_name: format!(
                        "first-file-and-one-overwrite/every-write-persisted/writes={}",
                        table_writes.len()
                    ),
                    base: history_after_the_make_filesystem.base_pool(),
                    writes: table_writes,
                    segments: &segments_of_the_table,
                    seams: &[],
                    persisted_sets: vec![vec![true; table_writes.len()]],
                })
                .unwrap_or_else(|reason| panic!("引擎判得了这张表：{reason}"));
            assert_eq!(conclusions.states.len(), 1, "这张表一个状态");
            (conclusions.source, conclusions.states[0].record_check())
        };
        let (healthy_source, healthy_record_check) =
            record_check_with_every_write_persisted(healthy_writes);
        if healthy_source == GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly {
            // 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑、交占位结论，判归带特性那一趟（引擎开跑时已打本次未跑那一行）
            return;
        }
        assert_eq!(
            healthy_record_check,
            RecordCheck::default(),
            "不种：跑完的历史每个写都落了，记录核对器不红"
        );
        assert!(
            record_check_with_every_write_persisted(&planted_writes).1.claimed_state_missing_unit,
            "种了：创世树表单元两份副本都被不合法地盖掉，记录核对器判恢复自称的状态缺了创世发布的单元"
        );
    }

    /// GPU 判器的发布表照 CPU 记录核对器同一个分法分（给定状态表）：创世那几次发布在表里、带 mkfs 的单元写；枚举写表接在录制流后面的撕裂根槽写
    /// （mkfs 的根槽写叠在 mkfs 之后的基线上可撕，撕出来与原字节相同）不分出另一次发布。不要卡：只建表、不判。
    /// 判别力：发布表照整张枚举写表分（`PublishesSplitFrom::TheEnumeratedWrites`），多出几次根在录制流之后的发布；写表把 mkfs 的写留在表外，
    /// 创世发布不在表里（`crates/mutations.tsv` 两行）。
    #[test]
    fn the_gpu_publishes_of_a_given_states_table_are_split_as_the_cpu_record_checker_splits_them_genesis_included(
    ) {
        use crate::crash_judge_tables::{
            build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams, JudgeFlow,
        };
        let (history, execution) = history_of_the_first_file_and_one_overwrite();
        let (writes, segments, stream_indexes, after_make_filesystem) =
            whole_write_table_of_the_history(&history, execution);
        let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
            &writes,
            &segments,
            &stream_indexes,
            after_make_filesystem,
            execution,
        );
        let table_writes = history_after_the_make_filesystem.writes();
        let persisted = vec![true; table_writes.len()];
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            crate::crash::CrashPointReplaySegmentExpansion::NotExpanded
        };
        let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
            history_after_the_make_filesystem.base_pool(),
            table_writes,
            history_after_the_make_filesystem.segments(),
            &not_expanded,
        )
        .enumerated_writes
        .len();
        assert!(
            enumerated_write_count > table_writes.len(),
            "前提：枚举写表接了撕裂镜像（mkfs 的根槽写与系统配置槽写叠在基线上可撕）"
        );
        let mut mask = persisted.clone();
        mask.resize(enumerated_write_count, false);
        let tables = build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams(
            &JudgeFlow {
                base: history_after_the_make_filesystem.base_pool(),
                writes: table_writes,
                segments: history_after_the_make_filesystem.segments(),
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            vec![mask],
            &[],
        )
        .expect("建得出这张表");
        let sorted = |mut units: Vec<usize>| {
            units.sort_unstable();
            units
        };
        let gpu_publishes: Vec<(usize, Vec<usize>)> = tables
            .publishes
            .iter()
            .map(|publish| {
                (
                    usize::try_from(publish.root_write).expect("下标装得进 usize"),
                    sorted(
                        publish
                            .unit_copy_groups
                            .iter()
                            .flatten()
                            .map(|copy| usize::try_from(*copy).expect("下标装得进 usize"))
                            .collect(),
                    ),
                )
            })
            .collect();
        let cpu_publishes: Vec<(usize, Vec<usize>)> = crate::memory_pool::publishes_in(
            table_writes,
            &persisted,
            RecordStreamContinuity::OneRecording,
        )
        .into_iter()
        .map(|publish| (publish.root, sorted(publish.units)))
        .collect();
        assert_eq!(
            gpu_publishes, cpu_publishes,
            "GPU 的发布表与 CPU 记录核对器分出的发布逐次相同（根槽写、单元写）"
        );
        let genesis_writes = history_after_the_make_filesystem
            .writes_of_the_make_filesystem()
            .len();
        let genesis_unit_writes: Vec<usize> = (0..genesis_writes)
            .filter(|write| table_writes[*write].kind == StepKind::UnitWrite)
            .collect();
        assert!(
            !genesis_unit_writes.is_empty()
                && gpu_publishes
                    .iter()
                    .any(|(root, units)| *root < genesis_writes && *units == genesis_unit_writes),
            "创世那次发布在 GPU 的发布表里、带 mkfs 的单元写 {genesis_unit_writes:?}：{gpu_publishes:?}"
        );
    }
}
