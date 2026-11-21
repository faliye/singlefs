//! 拒绝的分类、发布了的写与一次写的记账和合计。

use super::*;

/// 一次被拒的写是哪一道拒的（登记 5.1 节「记每次是哪一道拒」）。先拒那一道（落得下那一判）与式子那一判报同一个成员，
/// 装置不拆开，带着不够的盘的读数原样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RefusalClass {
    /// `shortfall_bytes`：不够的盘里需求减可用的最大值（字节，每块盘一份），填满函数按它折成要少写几个单元。
    SpaceAdmission {
        short_devices: String,
        shortfall_bytes: i128,
    },
    FixedPointPlacement {
        unit: String,
        refusal: String,
    },
    /// `journal_records_over_the_limit`：被拒的是这次发布的 journal 记录数超过上限时，超出的条数（条数减上限）；别的拒因是 `None`。
    Other {
        description: String,
        journal_records_over_the_limit: Option<u64>,
    },
}

impl RefusalClass {
    pub(super) fn of(error: &PublishError) -> Self {
        match error {
            PublishError::SpaceAdmissionRefused(refused) => RefusalClass::SpaceAdmission {
                short_devices: format!("{:?}", refused.short_devices),
                shortfall_bytes: refused
                    .short_devices
                    .iter()
                    .map(|short| i128::from(short.demand.0) - short.available.0)
                    .max()
                    .unwrap_or(0),
            },
            PublishError::PlacementRefused { unit, refusal } => RefusalClass::FixedPointPlacement {
                unit: format!("{unit:?}"),
                refusal: format!("{refusal:?}"),
            },
            PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish,
                record_limit,
            } => RefusalClass::Other {
                description: format!("{error:?}"),
                journal_records_over_the_limit: Some(
                    records_of_the_publish.saturating_sub(*record_limit),
                ),
            },
            PublishError::EmptyClusterSegmentsBelowTheFloor(_)
            | PublishError::MultiLevelCodeTwoTreeRefused { .. }
            | PublishError::ReleaseNotInMapping { .. }
            | PublishError::ReleaseTargetNotAllocated { .. }
            | PublishError::ReleaseTargetAlreadyReleased { .. }
            | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
            | PublishError::ReleaseSpanMismatch { .. }
            | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
            | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
            | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
            | PublishError::ContentExceedsDataUnit { .. }
            | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
            | PublishError::InodeTreeWriteRefused(_)
            | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
            | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
            | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
            | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
            | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
            | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
            | PublishError::WriteToAnInodeBeingDeleted { .. }
            | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
            | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
            | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
            | PublishError::DeletedInodesQueueRefused(_)
            | PublishError::SnapshotCreationRefused(_)
            | PublishError::SnapshotDestructionRefused(_)
            | PublishError::DeadlistEntryNotReleasable { .. }
            | PublishError::TruncationRefused(_)
            | PublishError::BlockDevice(_)
            | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
            | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
                _,
            ) => RefusalClass::Other {
                description: format!("{error:?}"),
                journal_records_over_the_limit: None,
            },
        }
    }

    /// 产物行里写的拒因细节：不够的盘的读数、落点那一道的单元与拒因、别的拒因的原文。
    pub(super) fn detail(&self) -> String {
        match self {
            RefusalClass::SpaceAdmission { short_devices, .. } => short_devices.clone(),
            RefusalClass::FixedPointPlacement { unit, refusal } => format!("{unit} {refusal}"),
            RefusalClass::Other { description, .. } => description.clone(),
        }
    }

    pub(super) fn name(&self) -> &'static str {
        match self {
            RefusalClass::SpaceAdmission { .. } => "space_admission_refused",
            RefusalClass::FixedPointPlacement { .. } => "placement_refused_in_the_fixed_point",
            RefusalClass::Other { .. } => "other_refusal",
        }
    }

    /// 空间准入那一道拒时，不够的盘里需求减可用的最大值（字节）；别的拒因没有缺口读数。
    pub(super) fn space_shortfall_bytes(&self) -> Option<i128> {
        match self {
            RefusalClass::SpaceAdmission {
                shortfall_bytes, ..
            } => Some(*shortfall_bytes),
            RefusalClass::FixedPointPlacement { .. } | RefusalClass::Other { .. } => None,
        }
    }

    /// 哪一道拒、缺口多少（没有空格，可以放进产物行的字段）：标准错误上被拒那一行与边查找的产物行都用它。
    pub(super) fn gate_and_gap_text(&self) -> String {
        match self {
            RefusalClass::SpaceAdmission {
                shortfall_bytes, ..
            } => {
                format!("{}(shortfall_bytes={shortfall_bytes})", self.name())
            }
            RefusalClass::FixedPointPlacement { .. } => self.name().to_string(),
            RefusalClass::Other {
                journal_records_over_the_limit: Some(surplus),
                ..
            } => format!("{}(journal_records_over_the_limit={surplus})", self.name()),
            RefusalClass::Other {
                journal_records_over_the_limit: None,
                ..
            } => self.name().to_string(),
        }
    }
}

/// 一块盘上缺 `shortfall_bytes` 字节，要少写几个数据单元才补上：一个单元在一块盘上占两槽（32768 字节），向上取整；
/// 缺口读数不是正数时按一字节算。被拒之后往下找（[`units_to_try_after_a_refusal`]）与二分按缺口预测边（[`RefusalsSeen::predicted_edge`]）共用。
pub(super) fn units_a_shortfall_is_worth(shortfall_bytes: i128) -> i128 {
    let bytes_per_unit = i128::try_from(DATA_UNIT_BYTES_LOCAL).expect("单元字节装得进 i128");
    (shortfall_bytes.max(1) + bytes_per_unit - 1) / bytes_per_unit
}

/// 一次发成的写的账。
#[derive(Clone, Debug)]
pub(super) struct PublishedWrite {
    pub(super) units: u64,
    pub(super) height_before: u64,
    pub(super) height_after: u64,
    pub(super) true_fixed_point: u64,
    pub(super) true_fixed_point_by_records: Vec<(DeviceIdentity, u64)>,
    pub(super) checkpoint_cost: u64,
    pub(super) local_checkpoint_cost: u64,
    pub(super) checkpoint_cost_with_ordinary_mapping_changes: u64,
    pub(super) ordinary: OrdinaryMappingChanges,
}

impl PublishedWrite {
    /// Q1：P1 − 真写的固定点。
    pub(super) fn checkpoint_cost_with_ordinary_mapping_changes_minus_true(&self) -> i64 {
        signed_difference(
            self.checkpoint_cost_with_ordinary_mapping_changes,
            self.true_fixed_point,
        )
    }
    /// Q2：ckpt_cost − 真写的固定点（今天那一判的余量）。
    pub(super) fn checkpoint_cost_minus_true(&self) -> i64 {
        signed_difference(self.checkpoint_cost, self.true_fixed_point)
    }
    /// Q3：P1 − ckpt_cost（P1 比今天多扣的槽）。
    pub(super) fn checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost(
        &self,
    ) -> i64 {
        signed_difference(
            self.checkpoint_cost_with_ordinary_mapping_changes,
            self.checkpoint_cost,
        )
    }
}

pub(super) fn signed_difference(left: u64, right: u64) -> i64 {
    i64::try_from(left).expect("槽数装得进 i64") - i64::try_from(right).expect("槽数装得进 i64")
}

#[derive(Clone, Debug)]
pub(super) enum WriteOutcome {
    Published(Box<PublishedWrite>),
    Refused(RefusalClass),
}

/// 整份内容的缓冲：一次写 n 个数据单元就是前 n × 32768 字节，缓冲只长不缩。
pub(super) struct ContentBuffer {
    pub(super) bytes: Vec<u8>,
}

impl ContentBuffer {
    pub(super) fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub(super) fn of_units(&mut self, units: usize) -> &[u8] {
        let wanted = units * DATA_UNIT_BYTES_LOCAL;
        if self.bytes.len() < wanted {
            let already = self.bytes.len();
            self.bytes.extend(
                (already..wanted)
                    .map(|index| u8::try_from((index / 7 + index % 251) % 256).expect("小于 256")),
            );
        }
        &self.bytes[..wanted]
    }
}

pub(super) fn device_identities(pool: &Pool) -> Vec<DeviceIdentity> {
    pool.devices.iter().map(|(identity, _)| *identity).collect()
}

/// 接着现行那一版整份换写成 `units` 个数据单元（`publish_sequential_write`），发成时账里带三个量：
/// 真写的固定点、核心层 ckpt_cost、本地 P1；每次先拿本地 ckpt_cost（普通分配删插为 0）与核心层的同名量对拍
/// （B3，对不上返回 `Err`，由调用方停机）。
pub(super) fn write_units_and_account(
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content: &mut ContentBuffer,
) -> Result<WriteOutcome, String> {
    write_units_from_bytes_and_account(pool, units, capacities, content.of_units(units))
}

/// 标准错误上一次写的进度行（只给人看，不进产物）：单元数、发布用时，发成，或被拒加拒因（哪一道拒、缺口多少）。
pub(super) fn progress_line_of_a_write(
    units: usize,
    publish_seconds: f64,
    published: &Result<TransactionOutput, PublishError>,
) -> String {
    let result = match published {
        Ok(_) => "发成".to_string(),
        Err(error) => format!("被拒，拒因 {}", RefusalClass::of(error).gate_and_gap_text()),
    };
    format!("[e181] 一次写 {units} 个单元的发布用了 {publish_seconds:.1} 秒，{result}")
}

/// 同 [`write_units_and_account`]，内容由调用方给（几个线程共用同一份只读的内容）：`content_bytes` 恰是 `units` 个数据单元。
pub(super) fn write_units_from_bytes_and_account(
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content_bytes: &[u8],
) -> Result<WriteOutcome, String> {
    assert_eq!(content_bytes.len(), units * DATA_UNIT_BYTES_LOCAL);
    let checkpoint_cost = core_checkpoint_cost(&pool.current, &pool.allocator, capacities);
    let local_without_ordinary = local_checkpoint_cost(
        &pool.current,
        &pool.allocator,
        capacities,
        OrdinaryMappingChanges::default(),
    );
    if local_without_ordinary.total != checkpoint_cost {
        return Err(format!(
            "本地 checkpoint 代价 {local_without_ordinary:?} 与核心层的 {checkpoint_cost} 对不上（本地与核心层的 checkpoint 代价对拍停机）"
        ));
    }
    let height_before = pool
        .current
        .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping);
    let started = Instant::now();
    let published = {
        let mut writer = PoolWriter::new(&pool.parameters, &mut pool.devices);
        if let CodeTwoTreeNodeCapacities::CappedForTests { .. } = capacities {
            writer.set_code_two_tree_node_capacities(capacities);
        }
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &pool.current,
            FirstFile {
                content: content_bytes,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            pool.instance,
        )
    };
    if units >= LOGGED_WRITE_UNITS {
        eprintln!(
            "{}",
            progress_line_of_a_write(units, started.elapsed().as_secs_f64(), &published)
        );
    }
    match published {
        Err(error) => Ok(WriteOutcome::Refused(RefusalClass::of(&error))),
        Ok(output) => {
            let devices = device_identities(pool);
            let account = fixed_point_account(&output, &devices);
            let ordinary = ordinary_mapping_changes(&pool.current, &output);
            let local_with_ordinary =
                local_checkpoint_cost(&pool.current, &pool.allocator, capacities, ordinary);
            let report = PublishedWrite {
                units: u64::try_from(units).expect("单元数"),
                height_before,
                height_after: output
                    .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
                true_fixed_point: account.by_roles,
                true_fixed_point_by_records: account.by_records_per_device,
                checkpoint_cost,
                local_checkpoint_cost: local_without_ordinary.total,
                checkpoint_cost_with_ordinary_mapping_changes: local_with_ordinary.total,
                ordinary,
            };
            pool.current = output;
            Ok(WriteOutcome::Published(Box::new(report)))
        }
    }
}

/// 一类写（历史、几何、中央映射树的高）的 Q1、Q2、Q3 汇总：次数、最小值、最大值、为负的次数。
#[derive(Clone, Debug, Default)]
pub(super) struct WriteTally {
    pub(super) writes: u64,
    pub(super) checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum: Option<i64>,
    pub(super) checkpoint_cost_with_ordinary_mapping_changes_below_true_writes: u64,
    pub(super) checkpoint_cost_minus_true_minimum: Option<i64>,
    pub(super) checkpoint_cost_below_true_writes: u64,
    pub(super) checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum:
        Option<i64>,
    pub(super) checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum:
        Option<i64>,
    pub(super) true_fixed_point_maximum: u64,
}

impl WriteTally {
    pub(super) fn add(&mut self, write: &PublishedWrite) {
        self.writes += 1;
        let checkpoint_cost_with_ordinary_mapping_changes_minus_true =
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_true();
        let checkpoint_cost_minus_true = write.checkpoint_cost_minus_true();
        let checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost =
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost();
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_true,
                    |old| old.min(checkpoint_cost_with_ordinary_mapping_changes_minus_true),
                ),
        );
        self.checkpoint_cost_minus_true_minimum = Some(
            self.checkpoint_cost_minus_true_minimum
                .map_or(checkpoint_cost_minus_true, |old| {
                    old.min(checkpoint_cost_minus_true)
                }),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost,
                    |old| {
                        old.max(checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost)
                    },
                ),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum = Some(
            self.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum
                .map_or(
                    checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost,
                    |old| {
                        old.min(checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost)
                    },
                ),
        );
        self.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes +=
            u64::from(checkpoint_cost_with_ordinary_mapping_changes_minus_true < 0);
        self.checkpoint_cost_below_true_writes += u64::from(checkpoint_cost_minus_true < 0);
        self.true_fixed_point_maximum = self.true_fixed_point_maximum.max(write.true_fixed_point);
    }
}
