
// ====================== 攻方原型（m3-batch10z-tu-code-r1，opus 腿）======================
// 下面全是攻方腿自己的用例，名字都以 atk_ 开头、#[ignore]；上面原样是冻结副本里同名 harness 测试文件，只借它的搭建函数。

use singlefs_core::mount::MountOutput;
use singlefs_core::mounted_session::{
    UserChange as AtkUserChange, UserChangePublished as AtkPublished,
};
use singlefs_core::transaction::FirstFile as AtkFirstFile;

/// 一个池某一刻的样子（盘面、会话逐字段拷、每条根的内容）：同一刻起几份池各走各的。
struct AtkSnap {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    session: Option<MountedSession>,
    contents: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    content_now: Vec<u8>,
    write_time_seconds: u64,
}

impl AtkSnap {
    fn of<Device: DeviceOverASparseImage>(pool: &PoolUnderTest<Device>) -> Self {
        AtkSnap {
            parameters: pool.parameters.clone(),
            device_bytes: pool.device_bytes,
            image: pool.image(),
            session: pool.session.as_ref().map(copy_of_the_session),
            contents: pool.content_of_each_root.clone(),
            content_now: pool.content_of_the_current_version.clone(),
            write_time_seconds: pool.write_time_seconds,
        }
    }

    fn pool(&self) -> PoolUnderTest<SparseBlockDevice> {
        PoolUnderTest {
            parameters: self.parameters.clone(),
            device_bytes: self.device_bytes,
            devices: plain_devices_on(&self.image),
            session: self.session.as_ref().map(copy_of_the_session),
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: self.content_now.clone(),
            content_of_each_root: self.contents.clone(),
        }
    }

    fn recording_pool(&self) -> (PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>, SharedStream) {
        let stream = SharedStream::retaining_contents();
        let devices = plain_devices_on(&self.image)
            .into_iter()
            .map(|(identity, inner)| {
                (identity, RecordingBlockDevice::with_shared_stream(identity, inner, stream.clone()))
            })
            .collect();
        (
            PoolUnderTest {
                parameters: self.parameters.clone(),
                device_bytes: self.device_bytes,
                devices,
                session: self.session.as_ref().map(copy_of_the_session),
                write_time_seconds: self.write_time_seconds,
                content_of_the_current_version: self.content_now.clone(),
                content_of_each_root: self.contents.clone(),
            },
            stream,
        )
    }
}

/// 一次顺序写（整份内容与写入时刻都定死）：被拒之后「照出路走」再发的是逐字节同一个改动。
#[derive(Clone)]
struct AtkWrite {
    content: Vec<u8>,
    write_time_seconds: u64,
    /// 覆盖写（`UserChange::Overwrite`）而不是顺序写。
    overwrite: bool,
}

fn atk_write_of<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, units: usize) -> AtkWrite {
    pool.write_time_seconds += 1;
    AtkWrite {
        content: common_unwitnessed_publish::content_of(
            units * singlefs_core::unit::data_unit_payload_capacity(),
            pool.write_time_seconds,
        ),
        write_time_seconds: pool.write_time_seconds,
        overwrite: false,
    }
}

fn atk_apply<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    write: &AtkWrite,
) -> Result<AtkPublished, UserChangeRefused> {
    pool.write_time_seconds = pool.write_time_seconds.max(write.write_time_seconds);
    let file = AtkFirstFile {
        content: &write.content,
        write_time_seconds: write.write_time_seconds,
    };
    let change = if write.overwrite { AtkUserChange::Overwrite(file) } else { AtkUserChange::SequentialWrite(file) };
    pool.publish(change, Some(&write.content))
}

fn atk_stuck_of(
    outcome: &Result<AtkPublished, UserChangeRefused>,
) -> Option<&StuckBehindTheWideProtection> {
    match outcome {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
            match &no_space.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) => Some(stuck),
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => None,
            }
        }
        _ => None,
    }
}

/// 一次改动的结局，短名：admitted、notwhat、stuck、other(…)。
fn atk_kind(outcome: &Result<AtkPublished, UserChangeRefused>) -> String {
    match outcome {
        Ok(_) => "admitted".to_string(),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
            match &no_space.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => {
                    let stop = match &no_space.stop {
                        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => "ceiling",
                        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => "budget",
                        FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => "raise-refused",
                    };
                    format!("notwhat({stop})")
                }
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_) => "stuck".to_string(),
            }
        }
        Err(other) => format!("other({})", format!("{other:?}").chars().take(90).collect::<String>()),
    }
}

/// 预演的结局：(adm / still / pushref, 预演里每块盘放出来的隔离槽数, 预演里交回的拒绝原样)。
fn atk_rehearsal_of(stuck: &StuckBehindTheWideProtection) -> (&'static str, Vec<u64>, String) {
    match &stuck.pushing_the_root_ring {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        } => (
            "adm",
            isolated_slots_released_by_the_push_on_each_device.iter().map(|(_, n)| *n).collect(),
            String::new(),
        ),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
            refusal,
        } => (
            "still",
            isolated_slots_released_by_the_push_on_each_device.iter().map(|(_, n)| *n).collect(),
            format!("{refusal:?}"),
        ),
        RootRingPushRehearsal::PushRefused(refusal) => ("pushref", Vec::new(), format!("{refusal:?}")),
    }
}
