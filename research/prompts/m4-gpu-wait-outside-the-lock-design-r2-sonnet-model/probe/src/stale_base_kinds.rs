//! 回锁复核用不含「现行那一版没换」的弱判据时，锁外那一批建在取输入那一刻的那一版上发出去。第一轮攻方探针只摆一种前台改动
//! （部分写第一个文件里的一个单元）：45 段旧底历史的整理第一次发布全被 `ReleaseTargetAlreadyReleased` 拒掉，`lost_foreground=0` 只说明「被拒的那一格没丢」。
//! 这里放开「前台在锁外期间发的是什么」：
//!   overwrite_unit_of_the_first_file：同第一轮（对照，复现第一轮的拒绝）；
//!   sequential_write_of_the_second_inode：写第二个 inode 的内容，不换下第一个文件的任何数据单元，只动映射树与记账；
//!   new_inodes：建一个 inode，只动实例表一类，不碰第一个文件的映射树。
//! 五臂（取样同第一轮：现行那一版换回克隆的那一版，再走显式整理入口，与后台入口推一批同一个 `advance_compaction_by_one_publish`）：
//!   stale_weak：换回之后整理，结局原样留着（第一轮 `stale_base_live_intent_check` 臂）；
//!   stale_weak_restored：同上，但整理一次都没发成时把现行那一版换回前台发完之后的那一版（第一轮那一臂把现行那一版留在旧底上，整理被拒之后前台再发的写被拒，是探针手造的）；
//!   current_base：在现行那一版上整理（对照，读回的内容当「该读到的」）；
//!   txg_guard：txg 不等就不整理；
//!   planted_rollback：判法自证，前台发完把盘换回发之前那一份。
//! 判丢：每段历史末尾丢会话（写全落盘的断电），只读挂载读回一串读数，与同一 (种类, k, m) 下 current_base 臂读回的逐字比，不等就是丢（或坏）。
//! 没取崩溃状态；没摆建快照与删快照（读回判不出）；没摆「第一段算好计划、第三段照那份计划发」（探针是旧底上现算）。
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::FIRST_INODE_NUMBER;

use crate::pool::{copied, offset_of_the_unit, parameters, start_pool, write_one_byte, Devices, WRITE_TIME_SECONDS};

/// 第一轮 k 取到 8 为止；被释放的节点要过了 defer 窗口（根环 3S 槽，S 取默认 8，`16-发布语义.md` 的「可再分配」那一行）才可能被再分配，所以加 24、32、48。
const FOREGROUND_PUBLISHES: [usize; 7] = [1, 2, 3, 8, 24, 32, 48];
const PUBLISHES_AFTER_THE_COMPACTION: [usize; 2] = [0, 1];
const UNIT_WRITTEN_AFTER_THE_COMPACTION: usize = 5;
const SECOND_INODE: u64 = FIRST_INODE_NUMBER + 1;
const FIRST_INODE_MADE_BY_THE_FOREGROUND: u64 = FIRST_INODE_NUMBER + 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    OverwriteUnitOfTheFirstFile,
    SequentialWriteOfTheSecondInode,
    NewInodes,
}

impl Kind {
    const ALL: [Kind; 3] = [Kind::OverwriteUnitOfTheFirstFile, Kind::SequentialWriteOfTheSecondInode, Kind::NewInodes];
    fn name(self) -> &'static str {
        match self {
            Kind::OverwriteUnitOfTheFirstFile => "overwrite_unit_of_the_first_file",
            Kind::SequentialWriteOfTheSecondInode => "sequential_write_of_the_second_inode",
            Kind::NewInodes => "new_inodes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    StaleWeak,
    StaleWeakRestored,
    CurrentBase,
    TxgGuard,
    PlantedRollback,
}

impl Arm {
    const ALL: [Arm; 5] = [Arm::CurrentBase, Arm::StaleWeak, Arm::StaleWeakRestored, Arm::TxgGuard, Arm::PlantedRollback];
    fn name(self) -> &'static str {
        match self {
            Arm::StaleWeak => "stale_weak",
            Arm::StaleWeakRestored => "stale_weak_restored",
            Arm::CurrentBase => "current_base",
            Arm::TxgGuard => "txg_guard",
            Arm::PlantedRollback => "planted_rollback",
        }
    }
}

fn short_to(text: &str, limit: usize) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_").chars().take(limit).collect()
}

fn foreground_change(kind: Kind, index: usize, session: &mut MountedSession, devices: &mut Devices) -> Result<u64, String> {
    let marker = 0xA0 + u8::try_from(index).expect("小于 256");
    match kind {
        Kind::OverwriteUnitOfTheFirstFile => write_one_byte(session, devices, 1, marker),
        Kind::SequentialWriteOfTheSecondInode => {
            let content = vec![marker; 3000];
            session
                .publish_user_change(
                    devices,
                    UserChange::SequentialWriteOfAnInode {
                        inode: InodeNumber(SECOND_INODE),
                        content: &content,
                        write_time_seconds: WRITE_TIME_SECONDS + 9,
                    },
                )
                .map(|_published| session.current.root().checkpoint_txg.0)
                .map_err(|refused| short_to(&format!("{refused:?}"), 160))
        }
        Kind::NewInodes => session
            .publish_user_change(
                devices,
                UserChange::NewInodes { count: 1, write_time_seconds: WRITE_TIME_SECONDS + 3 },
            )
            .map(|_published| session.current.root().checkpoint_txg.0)
            .map_err(|refused| short_to(&format!("{refused:?}"), 160)),
    }
}

/// 只读挂载之后读回的一串读数：第一个文件四个单元的第一个字节、第二个 inode 的第一个字节、前台建的三个 inode 开不开得出。
fn observation(devices: &Devices, last_new_inode: u64) -> String {
    let mounted = match mount_read_only(devices) {
        Ok(mounted) => mounted,
        Err(failure) => return short_to(&format!("mount:{failure:?}"), 80),
    };
    let places: [(u64, u64); 9] = [
        (FIRST_INODE_NUMBER, offset_of_the_unit(1).0),
        (FIRST_INODE_NUMBER, offset_of_the_unit(2).0),
        (FIRST_INODE_NUMBER, offset_of_the_unit(39).0),
        (FIRST_INODE_NUMBER, offset_of_the_unit(UNIT_WRITTEN_AFTER_THE_COMPACTION).0),
        (SECOND_INODE, 0),
        (FIRST_INODE_MADE_BY_THE_FOREGROUND, 0),
        (FIRST_INODE_MADE_BY_THE_FOREGROUND + 1, 0),
        (FIRST_INODE_MADE_BY_THE_FOREGROUND + 2, 0),
        (last_new_inode, 0),
    ];
    let mut parts: Vec<String> = Vec::new();
    for (inode, offset) in places {
        let text = match mounted.mounted.open_file(devices, InodeNumber(inode)) {
            Err(failure) => format!("open_err:{}", short_to(&format!("{failure:?}"), 24)),
            Ok(file) => match file.read_at(devices, FileOffsetInBytes(offset), 1) {
                Ok(read) => match read.bytes.first() {
                    Some(byte) => format!("0x{byte:02X}"),
                    None => "empty".to_string(),
                },
                Err(failure) => format!("read_err:{}", short_to(&format!("{failure:?}"), 24)),
            },
        };
        parts.push(format!("i{inode}@{offset}={text}"));
    }
    parts.join(" ")
}

struct Outcome {
    line: String,
    observation: String,
    stale_publish_made: bool,
    refused_by_the_release_check: bool,
}

fn one_history(start: &Devices, kind: Kind, foreground_publishes: usize, publishes_after: usize, arm: Arm) -> Result<Outcome, String> {
    let parameters = parameters();
    let mut devices = copied(start);
    let devices_before_the_foreground = copied(&devices);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|e| short_to(&format!("mount:{e:?}"), 80))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let snapshot = session.current.clone();
    let snapshot_txg = snapshot.root().checkpoint_txg.0;
    let mut acknowledged = Vec::new();
    for index in 0..foreground_publishes {
        let txg = foreground_change(kind, index, &mut session, &mut devices)
            .map_err(|refused| format!("foreground_refused:{refused}"))?;
        acknowledged.push(txg);
    }
    let live_before_the_compaction = session.current.clone();
    let txg_before_the_compaction = session.current.root().checkpoint_txg.0;
    let compaction = match arm {
        Arm::StaleWeak | Arm::StaleWeakRestored => {
            session.current = snapshot;
            Some(session.run_compaction(&mut devices))
        }
        Arm::CurrentBase => Some(session.run_compaction(&mut devices)),
        Arm::TxgGuard if snapshot_txg == txg_before_the_compaction => Some(session.run_compaction(&mut devices)),
        Arm::TxgGuard | Arm::PlantedRollback => None,
    };
    let published = match &compaction {
        Some(Ok(run)) => run.versions_published.len(),
        _ => 0,
    };
    let compaction_text = match &compaction {
        None => "discarded".to_string(),
        Some(Ok(run)) => format!("published={published} outcome={}", short_to(&format!("{:?}", run.outcome), 360)),
        Some(Err(refused)) => format!("refused:{}", short_to(&format!("{refused:?}"), 360)),
    };
    if arm == Arm::StaleWeakRestored && published == 0 {
        session.current = live_before_the_compaction;
    }
    if arm == Arm::PlantedRollback {
        devices = copied(&devices_before_the_foreground);
    }
    let mut after_refused = 0_u32;
    let publishes_after = if arm == Arm::PlantedRollback { 0 } else { publishes_after };
    for index in 0..publishes_after {
        let marker = 0xC0 + u8::try_from(index).expect("小于 256");
        if write_one_byte(&mut session, &mut devices, UNIT_WRITTEN_AFTER_THE_COMPACTION, marker).is_err() {
            after_refused += 1;
        }
    }
    drop(session);
    let last_new_inode = match kind {
        Kind::NewInodes => FIRST_INODE_MADE_BY_THE_FOREGROUND + u64::try_from(foreground_publishes).expect("次数") - 1,
        Kind::OverwriteUnitOfTheFirstFile | Kind::SequentialWriteOfTheSecondInode => FIRST_INODE_MADE_BY_THE_FOREGROUND,
    };
    let observed = observation(&devices, last_new_inode);
    let mut remount_devices = copied(&devices);
    let writable_remount = match mount_writable(&parameters, &mut remount_devices) {
        Ok(_) => "ok".to_string(),
        Err(error) => short_to(&format!("{error:?}"), 60),
    };
    let line = format!(
        "HISTORY name=stale_base_kinds arm={} kind={} k={} m={} snapshot_txg={} acknowledged_txgs={} txg_before_compaction={} compaction=[{}] after_refused={} writable_remount={} observation=[{}]",
        arm.name(), kind.name(), foreground_publishes, publishes_after, snapshot_txg,
        acknowledged.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        txg_before_the_compaction, compaction_text, after_refused, writable_remount, observed,
    );
    let stale_arm = matches!(arm, Arm::StaleWeak | Arm::StaleWeakRestored);
    Ok(Outcome {
        line,
        observation: observed,
        stale_publish_made: stale_arm && published > 0,
        refused_by_the_release_check: stale_arm && compaction_text.contains("ReleaseTargetAlreadyReleased"),
    })
}

#[derive(Default)]
struct Tally {
    histories: u64,
    lost: u64,
    stale_publish_made: u64,
    refused_by_the_release_check: u64,
    panicked: u64,
    setup_failed: u64,
}

pub fn run() {
    let start = start_pool();
    let mut tallies: Vec<(Kind, Arm, Tally)> = Vec::new();
    for kind in Kind::ALL {
        for arm in Arm::ALL {
            tallies.push((kind, arm, Tally::default()));
        }
    }
    for kind in Kind::ALL {
        for foreground_publishes in FOREGROUND_PUBLISHES {
            for publishes_after in PUBLISHES_AFTER_THE_COMPACTION {
                let mut control: Option<String> = None;
                for arm in Arm::ALL {
                    let result = catch_unwind(AssertUnwindSafe(|| one_history(&start, kind, foreground_publishes, publishes_after, arm)));
                    let tally = &mut tallies.iter_mut().find(|(k, a, _)| *k == kind && *a == arm).expect("每格都有一份").2;
                    tally.histories += 1;
                    match result {
                        Ok(Ok(outcome)) => {
                            if arm == Arm::CurrentBase {
                                control = Some(outcome.observation.clone());
                            }
                            let lost = control.as_deref().is_some_and(|expected| expected != outcome.observation);
                            tally.lost += u64::from(lost);
                            tally.stale_publish_made += u64::from(outcome.stale_publish_made);
                            tally.refused_by_the_release_check += u64::from(outcome.refused_by_the_release_check);
                            println!("{} lost={}", outcome.line, u8::from(lost));
                        }
                        Ok(Err(failure)) => {
                            tally.setup_failed += 1;
                            println!("HISTORY name=stale_base_kinds arm={} kind={} k={} m={} setup_failed={}", arm.name(), kind.name(), foreground_publishes, publishes_after, failure);
                        }
                        Err(_payload) => {
                            tally.panicked += 1;
                            println!("HISTORY name=stale_base_kinds arm={} kind={} k={} m={} panicked=1", arm.name(), kind.name(), foreground_publishes, publishes_after);
                        }
                    }
                }
            }
        }
    }
    for (kind, arm, tally) in &tallies {
        println!(
            "RESULT name=stale_base_kinds kind={} arm={} histories={} lost={} stale_publish_made={} refused_by_the_release_check={} panicked={} setup_failed={}",
            kind.name(), arm.name(), tally.histories, tally.lost, tally.stale_publish_made, tally.refused_by_the_release_check, tally.panicked, tally.setup_failed
        );
    }
}
