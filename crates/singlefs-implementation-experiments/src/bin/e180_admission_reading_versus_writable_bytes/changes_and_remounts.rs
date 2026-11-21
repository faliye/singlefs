//! 用户改动、它的结局与卸载后再挂载。

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChangeKind {
    /// 覆盖写：内容一个数据单元（`publish_overwrite` 的契约），第一个文件变成一个单元。
    OverwriteWithOneUnit,
    /// 顺序写：第一个文件整份换成这么多个数据单元。
    SequentialWriteOfUnits(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ChangeOutcome {
    Published { floor_raises: usize },
    RefusedForSpace,
    RefusedForAnotherReason(String),
}

impl ChangeOutcome {
    pub(super) fn label(&self) -> String {
        match self {
            ChangeOutcome::Published { floor_raises } => {
                format!("published_floor_raises_{floor_raises}")
            }
            ChangeOutcome::RefusedForSpace => "refused_for_space".to_string(),
            ChangeOutcome::RefusedForAnotherReason(reason) => format!("refused_other_{reason}"),
        }
    }
}

pub(super) fn short_debug(value: &dyn std::fmt::Debug) -> String {
    let text = format!("{value:?}");
    let shortened: String = text.chars().take(80).collect();
    shortened.replace([' ', '\n', '\t', '"'], "_")
}

pub(super) fn apply_change(pool: &mut Pool, kind: ChangeKind) -> ChangeOutcome {
    let write_time_seconds =
        WRITE_TIME_SECONDS + u64::try_from(pool.log.user_change_txgs.len()).expect("计数");
    let content = match kind {
        ChangeKind::OverwriteWithOneUnit => zero_bytes_of_units(1),
        ChangeKind::SequentialWriteOfUnits(units) => zero_bytes_of_units(units),
    };
    let file = FirstFile {
        content,
        write_time_seconds,
    };
    let change = match kind {
        ChangeKind::OverwriteWithOneUnit => UserChange::Overwrite(file),
        ChangeKind::SequentialWriteOfUnits(_) => UserChange::SequentialWrite(file),
    };
    match pool.session.publish_user_change(&mut pool.devices, change) {
        Ok(published) => {
            pool.log.note_published(&published);
            ChangeOutcome::Published {
                floor_raises: published.floor_raises.len(),
            }
        }
        Err(refusal) => {
            pool.log.note_refusal(&refusal);
            match &refusal {
                UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)
                | UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => {
                    ChangeOutcome::RefusedForSpace
                }
                UserChangeRefused::Publish { cause, .. } if refusal_is_short_of_space(cause) => {
                    ChangeOutcome::RefusedForSpace
                }
                other @ (UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
                    ..
                }
                | UserChangeRefused::NoFileVersionToChange { .. }
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    ..
                }
                | UserChangeRefused::Publish { .. }
                | UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_)) => {
                    ChangeOutcome::RefusedForAnotherReason(short_debug(other))
                }
            }
        }
    }
}

/// 一次正常卸载推了几次空发布、卸载那一刻现行 txg 是多少（登记 A3 核：现行 txg mod 3 = 1 时 3 次，= 0 或 2 时 2 次）。
pub(super) struct UnmountSequenceObserved {
    pub(super) current_txg_before_the_unmount: u64,
    pub(super) empty_publishes_pushed: usize,
}

/// 正常卸载再可写挂载（多次挂载族、被抛弃根与净释放族的「一轮」）：卸载推空发布的 txg 记进日志，挂载写行与暖机的 txg 同样记。
pub(super) fn unmount_and_mount_again(pool: &mut Pool) -> UnmountSequenceObserved {
    let current_txg_before_the_unmount = txg_of_pool_version(&pool.session.current);
    let mut empty_publishes_pushed = 0usize;
    match unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.session.allocator,
        &mut pool.session.current,
        pool.session.shadow_ledger,
    ) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            empty_publishes_pushed = raised.publishes.len();
            for publish in &raised.publishes {
                pool.log
                    .empty_publish_txgs
                    .insert(publish.root.checkpoint_txg.0);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {}
        Err(error) => stop_with(
            "unmount_refused",
            &format!(
                "geometry={} error={}",
                pool.geometry.name,
                short_debug(&error)
            ),
        ),
    }
    mount_again(pool);
    UnmountSequenceObserved {
        current_txg_before_the_unmount,
        empty_publishes_pushed,
    }
}

pub(super) fn mount_again(pool: &mut Pool) -> MountOutput {
    let mounted = match mount_writable(&pool.parameters, &mut pool.devices) {
        Ok(mounted) => mounted,
        Err(error) => stop_with(
            "remount_refused",
            &format!(
                "geometry={} error={}",
                pool.geometry.name,
                short_debug(&error)
            ),
        ),
    };
    let (output, session) = MountedSession::of_the_mount(mounted);
    pool.log.note_mount(&output);
    pool.session = session;
    output
}
