// ====== 攻方原型（m3-batch10z-xy-code-r1，opus 腿）：上一轮 m3-batch10z-vw-code-r1-opus-model 的 atk_vw_part*.rs 拷来改；攻方用例都以 atk_ 开头、#[ignore] ======
// 上面原样是冻结副本里那份 harness 测试文件，只借它的搭建函数。被判：ways_out_of_the_wide_protection 先核盘表次序（第二十四块）。
use singlefs_core::mount::MountOutput;
use singlefs_core::mounted_session::UserChangePublished as AtkPublished;

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
}

/// 一次用户改动（内容与写入时刻都定死）：被拒之后「照出路走」再发的是逐字节同一个改动。
#[derive(Clone, Debug)]
enum AtkChange {
    Sequential { content: Vec<u8>, write_time_seconds: u64 },
    Overwrite { content: Vec<u8>, write_time_seconds: u64 },
    Unlink { inode: InodeNumber, write_time_seconds: u64 },
    Inodes { count: u64, write_time_seconds: u64 },
}

impl AtkChange {
    fn as_change(&self) -> UserChange<'_> {
        match self {
            AtkChange::Sequential { content, write_time_seconds } => {
                UserChange::SequentialWrite(FirstFile { content, write_time_seconds: *write_time_seconds })
            }
            AtkChange::Overwrite { content, write_time_seconds } => {
                UserChange::Overwrite(FirstFile { content, write_time_seconds: *write_time_seconds })
            }
            AtkChange::Unlink { inode, write_time_seconds } => {
                UserChange::Unlink { inode: *inode, write_time_seconds: *write_time_seconds }
            }
            AtkChange::Inodes { count, write_time_seconds } => {
                UserChange::NewInodes { count: *count, write_time_seconds: *write_time_seconds }
            }
        }
    }
    fn content_after(&self) -> Option<&[u8]> {
        match self {
            AtkChange::Sequential { content, .. } | AtkChange::Overwrite { content, .. } => Some(content),
            AtkChange::Unlink { .. } | AtkChange::Inodes { .. } => None,
        }
    }
    fn time(&self) -> u64 {
        match self {
            AtkChange::Sequential { write_time_seconds, .. }
            | AtkChange::Overwrite { write_time_seconds, .. }
            | AtkChange::Unlink { write_time_seconds, .. }
            | AtkChange::Inodes { write_time_seconds, .. } => *write_time_seconds,
        }
    }
}

fn atk_seq_of<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, units: usize) -> AtkChange {
    pool.write_time_seconds += 1;
    AtkChange::Sequential {
        content: content_of(units * singlefs_core::unit::data_unit_payload_capacity(), pool.write_time_seconds),
        write_time_seconds: pool.write_time_seconds,
    }
}

fn atk_overwrite_of<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, bytes: usize) -> AtkChange {
    pool.write_time_seconds += 1;
    AtkChange::Overwrite { content: content_of(bytes, pool.write_time_seconds), write_time_seconds: pool.write_time_seconds }
}

fn atk_apply<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
    change: &AtkChange,
) -> Result<AtkPublished, UserChangeRefused> {
    pool.write_time_seconds = pool.write_time_seconds.max(change.time());
    pool.publish(change.as_change(), change.content_after())
}

fn atk_stuck_of(outcome: &Result<AtkPublished, UserChangeRefused>) -> Option<&StuckBehindTheWideProtection> {
    match outcome {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => match &no_space.wide_protection_at_the_ceiling {
            WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) => Some(stuck),
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => None,
        },
        _ => None,
    }
}

fn atk_versions_published_of(outcome: &Result<AtkPublished, UserChangeRefused>) -> usize {
    match outcome {
        Ok(published) => published.versions_published.len(),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => no_space.versions_published.len(),
        Err(UserChangeRefused::Publish { versions_published, .. }) => versions_published.len(),
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) => failed.versions_published.len(),
        Err(_) => 0,
    }
}

/// 一次改动的结局，短名：admitted、notwhat(…)、stuck、other(…)。
fn atk_kind(outcome: &Result<AtkPublished, UserChangeRefused>) -> String {
    match outcome {
        Ok(_) => "admitted".to_string(),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => match &no_space.wide_protection_at_the_ceiling {
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => {
                let stop = match &no_space.stop {
                    FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => "ceiling",
                    FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => "budget",
                    FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => "raise-refused",
                };
                format!("notwhat({stop})")
            }
            WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_) => "stuck".to_string(),
        },
        Err(UserChangeRefused::Publish { cause, .. }) => {
            format!("publish({})", atk_head(format!("{cause:?}")))
        }
        Err(other) => format!("other({})", format!("{other:?}").chars().take(90).collect::<String>()),
    }
}

fn atk_head(text: String) -> String {
    text.chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect()
}
