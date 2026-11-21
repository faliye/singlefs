use super::*;

// ============================================================================================
// X14：一次尝试 = mkfs → 可写挂载 → 首个文件 → 正常卸载 → 可写挂载；每一步成不成、挂载的准入怎么判的、读数。
// ============================================================================================

/// 可写挂载的准入结局（`MountSpaceAdmission` 的四个成员，名字写成报得出的字符串）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AdmissionClass {
    AdmittedBeforeAcquisition,
    AdmittedAfterTheFloorRaises,
    StillShortAfterTheFloorRaises,
    NotJudgedByTheTestOnlySwitch,
}

impl AdmissionClass {
    pub(super) fn of_the_mount(space_admission: &MountSpaceAdmission) -> Self {
        match space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => {
                AdmissionClass::AdmittedBeforeAcquisition
            }
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { .. } => {
                AdmissionClass::AdmittedAfterTheFloorRaises
            }
            MountSpaceAdmission::StillShortAfterTheFloorRaises { .. } => {
                AdmissionClass::StillShortAfterTheFloorRaises
            }
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
                AdmissionClass::NotJudgedByTheTestOnlySwitch
            }
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            AdmissionClass::AdmittedBeforeAcquisition => "AdmittedBeforeAcquisition",
            AdmissionClass::AdmittedAfterTheFloorRaises => "AdmittedAfterTheFloorRaises",
            AdmissionClass::StillShortAfterTheFloorRaises => "StillShortAfterTheFloorRaises",
            AdmissionClass::NotJudgedByTheTestOnlySwitch => "NotJudgedByTheTestOnlySwitch",
        }
    }

    /// X14：「成」只认放行的两种；推满仍不够不算成（另记），没判的也不算（它只在 PC-准入 里出现）。
    fn counts_as_admitted(self) -> bool {
        match self {
            AdmissionClass::AdmittedBeforeAcquisition => true,
            AdmissionClass::AdmittedAfterTheFloorRaises => true,
            AdmissionClass::StillShortAfterTheFloorRaises => false,
            AdmissionClass::NotJudgedByTheTestOnlySwitch => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StageOutcome<Success> {
    Succeeded(Success),
    /// 这一步报错（变体名）或 panic。
    Failed(String),
    /// 前一步没成，没走到。
    NotReached,
}

impl<Success> StageOutcome<Success> {
    pub(super) fn text(&self, describe: impl Fn(&Success) -> String) -> String {
        match self {
            StageOutcome::Succeeded(success) => describe(success),
            StageOutcome::Failed(reason) => format!("failed:{reason}"),
            StageOutcome::NotReached => "not_reached".to_string(),
        }
    }
}

/// 一次可写挂载做成之后留下的东西：准入结局、两份读数、这次挂载的读写计数、盘上实例表链的行数、此刻两块盘上可用(d) 的最小值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MountReport {
    pub(super) class: AdmissionClass,
    pub(super) readings: Vec<ReadingSnapshot>,
    pub(super) instance_rows_by_the_allocator: u64,
    pub(super) instance_rows_read_back_from_the_disk: Option<u64>,
    pub(super) mount_tallies: Vec<DeviceTally>,
    pub(super) available_minimum_bytes: i128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Trial {
    pub(super) device_bytes: u64,
    pub(super) make_filesystem: StageOutcome<()>,
    pub(super) empty_mount: StageOutcome<MountReport>,
    pub(super) first_file: StageOutcome<()>,
    pub(super) unmount: StageOutcome<()>,
    pub(super) file_mount: StageOutcome<MountReport>,
}

/// 搜索只看这几个布尔（登记 X14 的三个最小盘与「挂上但推满仍不够」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TrialFlags {
    pub(super) device_bytes: u64,
    pub(super) make_filesystem_succeeded: bool,
    pub(super) empty_mount_admitted: bool,
    pub(super) file_mount_admitted: bool,
    pub(super) empty_mount_still_short: bool,
    pub(super) file_mount_still_short: bool,
}

impl Trial {
    pub(super) fn flags(&self) -> TrialFlags {
        let admitted = |outcome: &StageOutcome<MountReport>| matches!(outcome, StageOutcome::Succeeded(report) if report.class.counts_as_admitted());
        let still_short = |outcome: &StageOutcome<MountReport>| {
            matches!(outcome, StageOutcome::Succeeded(report)
                if report.class == AdmissionClass::StillShortAfterTheFloorRaises)
        };
        let file_chain_succeeded = matches!(self.first_file, StageOutcome::Succeeded(()))
            && matches!(self.unmount, StageOutcome::Succeeded(()));
        TrialFlags {
            device_bytes: self.device_bytes,
            make_filesystem_succeeded: matches!(self.make_filesystem, StageOutcome::Succeeded(())),
            empty_mount_admitted: admitted(&self.empty_mount),
            file_mount_admitted: admitted(&self.empty_mount)
                && file_chain_succeeded
                && admitted(&self.file_mount),
            empty_mount_still_short: still_short(&self.empty_mount),
            file_mount_still_short: file_chain_succeeded && still_short(&self.file_mount),
        }
    }
}

fn report_of_a_mount(
    mounted: &Mounted,
    parameters: &MakeFilesystemParameters,
    pool: &CountedPool,
    tallies_before_the_mount: &[DeviceTally],
) -> MountReport {
    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
    let readings = both_readings_of_a_mount(mounted, slots_per_region);
    let available_minimum_bytes = readings[0]
        .devices
        .iter()
        .map(|terms| terms.available_bytes)
        .min()
        .expect("两块盘");
    let instance_rows_read_back_from_the_disk =
        instance_table_chain_of_root(pool, mounted.current.root())
            .ok()
            .map(|chain| u64::try_from(chain.records.rows.len()).expect("行数装得进 u64"));
    MountReport {
        class: AdmissionClass::of_the_mount(&mounted.output.space_admission),
        readings,
        instance_rows_by_the_allocator: mounted
            .allocator
            .instance_rows_after_this_mounts_row_publish(),
        instance_rows_read_back_from_the_disk,
        mount_tallies: tally_difference(pool, tallies_before_the_mount),
        available_minimum_bytes,
    }
}

/// panic 也记成这一步没成（极小的盘上可能走到实现里的断言），文字只留 panic 消息头一行。
pub(super) fn guarded<Success>(
    step: impl FnOnce() -> Result<Success, String>,
) -> Result<Success, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(step)) {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("panic_without_message");
            Err(format!(
                "panicked:{}",
                message.lines().next().unwrap_or("").replace(' ', "_")
            ))
        }
    }
}

/// 一次尝试：同一对稀疏盘上依次 mkfs、可写挂载、首个文件、正常卸载、可写挂载；某一步没成，后面的记「没走到」。
pub(super) fn run_trial(
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> Trial {
    let parameters = make_filesystem_parameters(journal_ring_bytes, slots_per_region);
    let mut pool = counted_pool(device_bytes, journal_ring_bytes, slots_per_region);
    let mut trial = Trial {
        device_bytes,
        make_filesystem: StageOutcome::NotReached,
        empty_mount: StageOutcome::NotReached,
        first_file: StageOutcome::NotReached,
        unmount: StageOutcome::NotReached,
        file_mount: StageOutcome::NotReached,
    };
    let made = guarded(|| {
        make_filesystem(&parameters, &mut pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    match made {
        Ok(()) => trial.make_filesystem = StageOutcome::Succeeded(()),
        Err(reason) => {
            trial.make_filesystem = StageOutcome::Failed(reason);
            return trial;
        }
    }
    let tallies_before_the_first_mount = tallies_of(&pool);
    let first_mount = guarded(|| {
        mount_writable(&parameters, &mut pool)
            .map(|mounted| {
                let report = report_of_a_mount(
                    &mounted,
                    &parameters,
                    &pool,
                    &tallies_before_the_first_mount,
                );
                (mounted, report)
            })
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    let (mounted, empty_report) = match first_mount {
        Ok(pair) => pair,
        Err(reason) => {
            trial.empty_mount = StageOutcome::Failed(reason);
            return trial;
        }
    };
    trial.empty_mount = StageOutcome::Succeeded(empty_report);
    if !trial.flags().empty_mount_admitted {
        return trial;
    }
    let content = first_file_content();
    let published = guarded(|| {
        let Mounted {
            output,
            mut allocator,
            current,
        } = mounted;
        let mut writer = PoolWriter::new(&parameters, pool.as_mut_slice());
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            output.instance,
            current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))?;
        Ok((allocator, PoolVersion::of_a_version_of_trees(first)))
    });
    let (mut allocator, mut current) = match published {
        Ok(pair) => pair,
        Err(reason) => {
            trial.first_file = StageOutcome::Failed(reason);
            return trial;
        }
    };
    trial.first_file = StageOutcome::Succeeded(());
    let unmounted = guarded(|| {
        unmount(
            &parameters,
            &mut pool,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    match unmounted {
        Ok(()) => trial.unmount = StageOutcome::Succeeded(()),
        Err(reason) => {
            trial.unmount = StageOutcome::Failed(reason);
            return trial;
        }
    }
    let tallies_before_the_second_mount = tallies_of(&pool);
    let second_mount = guarded(|| {
        mount_writable(&parameters, &mut pool)
            .map(|remounted| {
                report_of_a_mount(
                    &remounted,
                    &parameters,
                    &pool,
                    &tallies_before_the_second_mount,
                )
            })
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    });
    trial.file_mount = match second_mount {
        Ok(report) => StageOutcome::Succeeded(report),
        Err(reason) => StageOutcome::Failed(reason),
    };
    trial
}

#[cfg(test)]
mod tests;
