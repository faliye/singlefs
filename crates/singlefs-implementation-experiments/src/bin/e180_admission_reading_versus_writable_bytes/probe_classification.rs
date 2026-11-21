//! 探测：跑一次探测、按到盘的写与拒绝的原因归类，与状态上的探测器。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// T 的探测（登记 5.1）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ProbeVerdict {
    /// 数据单元的写到达了设备：三道空间闸都放行。
    Admitted,
    /// 整写做成了（只在整写同判那一步出现）。
    Completed,
    RefusedForSpace,
    /// 在飞记录数超过上限：不是空间不够，也盖住了后面的式子那一判。
    JournalLimit {
        records_of_the_publish: u64,
        record_limit: u64,
    },
    Other(String),
}

pub(super) fn run_probe(
    devices: &[(DeviceIdentity, Disk<'static>)],
    mut session: MountedSession,
    units: u64,
    policy: DataWritePolicy,
) -> ProbeVerdict {
    let mut overlay: Vec<(DeviceIdentity, Disk<'_>)> = devices
        .iter()
        .map(|(identity, disk)| (*identity, disk.overlay(policy)))
        .collect();
    let change = UserChange::SequentialWrite(FirstFile {
        content: zero_bytes_of_units(units),
        write_time_seconds: WRITE_TIME_SECONDS + 1,
    });
    let result = session.publish_user_change(&mut overlay, change);
    let data_write_reached = overlay.iter().any(|(_, disk)| disk.data_write_reached);
    classify_probe_result(result, data_write_reached)
}

/// 一次探测的会话结果换成探测结局：数据写到达了设备 ⇒ 放行；空间不够的两种 ⇒ 被空间拒；记录数超上限 ⇒ 不是空间不够；别的 ⇒ 取不出。
pub(super) fn classify_probe_result(
    result: Result<UserChangePublished, UserChangeRefused>,
    data_write_reached: bool,
) -> ProbeVerdict {
    match result {
        Ok(_) if data_write_reached => {
            ProbeVerdict::Other("data_write_reached_yet_published".to_string())
        }
        Ok(_) => ProbeVerdict::Completed,
        Err(_) if data_write_reached => ProbeVerdict::Admitted,
        Err(
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)
            | UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_),
        ) => ProbeVerdict::RefusedForSpace,
        Err(UserChangeRefused::Publish { cause, .. }) => {
            if refusal_is_short_of_space(&cause) {
                ProbeVerdict::RefusedForSpace
            } else if let PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish,
                record_limit,
            } = cause
            {
                ProbeVerdict::JournalLimit {
                    records_of_the_publish,
                    record_limit,
                }
            } else {
                ProbeVerdict::Other(short_debug(&cause))
            }
        }
        Err(other) => ProbeVerdict::Other(short_debug(&other)),
    }
}

/// 一个状态上的探测器：拿着摘掉了大字节的会话（[`strip_bytes_the_probe_does_not_need`]），每次探测拷一份会话、盘上套叠层。
pub(super) struct StateProber<'pool> {
    pub(super) pool: &'pool Pool,
    pub(super) probe_threads: usize,
    pub(super) probes_run: AtomicU64,
}

/// 探测搜索要的东西：一批单元数各探一次（数据单元的写到达就放行），和一轮最多并行探几个。真探测器是 [`StateProber`]，单测里换假的。
pub(super) trait UnitsProbe {
    fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict>;
    fn parallel_probe_count(&self) -> usize;
    fn probe_one(&self, units: u64) -> ProbeVerdict {
        self.probe_in_parallel(&[units]).remove(0)
    }
}

impl UnitsProbe for StateProber<'_> {
    fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict> {
        self.probe_many(unit_counts, DataWritePolicy::FailTheFirstDataWriteAndNoteIt)
    }

    fn parallel_probe_count(&self) -> usize {
        self.probe_threads
    }
}

impl StateProber<'_> {
    fn probe_many(&self, unit_counts: &[u64], policy: DataWritePolicy) -> Vec<ProbeVerdict> {
        self.probes_run.fetch_add(
            u64::try_from(unit_counts.len()).expect("探测数"),
            Ordering::Relaxed,
        );
        let sessions: Vec<MountedSession> = unit_counts
            .iter()
            .map(|_| clone_session(&self.pool.session))
            .collect();
        let devices = &self.pool.devices;
        if sessions.len() == 1 {
            let session = sessions.into_iter().next().expect("一份会话");
            return vec![run_probe(devices, session, unit_counts[0], policy)];
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = sessions
                .into_iter()
                .zip(unit_counts.iter().copied())
                .map(|(session, units)| {
                    std::thread::Builder::new()
                        .stack_size(THREAD_STACK_BYTES)
                        .spawn_scoped(scope, move || run_probe(devices, session, units, policy))
                        .expect("起探测线程")
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("探测线程没 panic"))
                .collect()
        })
    }
}
