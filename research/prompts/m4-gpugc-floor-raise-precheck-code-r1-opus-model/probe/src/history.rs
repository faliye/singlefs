//! 一条历史：按种子随机放开用户动作（建 inode、整份写、覆盖写、unlink、截断、建快照、发布边界上崩了再挂、卸载再挂、
//! 重发上一次被拒的改动），每一次经会话的调用之前拷一份池；带先算的会话交回「全放出来也放不下」时，在拷贝上旁路先算、
//! 照先算之前的推法发同一个改动（至多再重发两次），写进了就是误拒。

use singlefs_core::address::InodeNumber;
use singlefs_core::mounted_session::floor_raise_precheck_probe as probe;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused, WideProtectionAtTheCeiling};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::data_unit_payload_capacity;

use crate::pool::{content_of, leading_text, Pool};
use crate::tally::{add, describe, Tally};

/// splitmix64：每条历史一个种子，结局只由种子定。
pub struct Random(u64);

impl Random {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    CreateInodes { count: u64 },
    WholeWrite { inode: u64, length: usize },
    Overwrite { inode: u64, length: usize },
    Unlink { inode: u64 },
    Truncate { inode: u64, new_size: u64 },
    CreateSnapshot,
    CrashThenMount,
    UnmountThenMount,
}

impl Action {
    fn name(&self) -> &'static str {
        match self {
            Action::CreateInodes { .. } => "create-inodes",
            Action::WholeWrite { .. } => "whole-write",
            Action::Overwrite { .. } => "overwrite",
            Action::Unlink { .. } => "unlink",
            Action::Truncate { .. } => "truncate",
            Action::CreateSnapshot => "create-snapshot",
            Action::CrashThenMount => "crash-then-mount",
            Action::UnmountThenMount => "unmount-then-mount",
        }
    }
}

/// 整份写的单元数取哪一档的上限：小写、几十单元、贴着这档盘剩下的、远超整个单元区的都有。
const WHOLE_WRITE_UNIT_CAPS: [u64; 8] = [1, 4, 16, 48, 96, 160, 240, 400];

fn pick_file(random: &mut Random, pool: &Pool, including_the_first: bool) -> Option<u64> {
    let candidates: Vec<u64> = pool
        .files
        .keys()
        .copied()
        .filter(|inode| including_the_first || *inode != FIRST_INODE_NUMBER)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    Some(candidates[usize::try_from(random.below(candidates.len() as u64)).expect("下标")])
}

fn choose(random: &mut Random, pool: &Pool, last_refused: &Option<Action>) -> Action {
    let payload = data_unit_payload_capacity() as u64;
    let file = pick_file(random, pool, true).unwrap_or(FIRST_INODE_NUMBER);
    match random.below(100) {
        0..=9 => Action::CreateInodes {
            count: 1 + random.below(3),
        },
        10..=43 => {
            let cap = WHOLE_WRITE_UNIT_CAPS[usize::try_from(random.below(8)).expect("下标")];
            let units = 1 + random.below(cap);
            let length = (units - 1) * payload + 1 + random.below(payload);
            Action::WholeWrite {
                inode: file,
                length: usize::try_from(length).expect("长度"),
            }
        }
        44..=57 => Action::Overwrite {
            inode: file,
            length: usize::try_from(1 + random.below(payload)).expect("长度"),
        },
        58..=69 => match pick_file(random, pool, false) {
            Some(inode) => Action::Unlink { inode },
            None => Action::CreateInodes { count: 1 },
        },
        70..=77 => {
            let size = pool.files.get(&file).copied().unwrap_or(0);
            let new_size = if random.below(4) == 0 {
                size + 1 + random.below(4 * payload)
            } else {
                random.below(size + 1)
            };
            Action::Truncate {
                inode: file,
                new_size,
            }
        }
        78..=79 if pool.snapshots_created < 3 => Action::CreateSnapshot,
        78..=79 => Action::Overwrite {
            inode: file,
            length: 1,
        },
        80..=84 => Action::CrashThenMount,
        85..=88 => Action::UnmountThenMount,
        _ => last_refused.clone().unwrap_or(Action::WholeWrite {
            inode: file,
            length: usize::try_from(1 + random.below(payload)).expect("长度"),
        }),
    }
}

fn change_for<'content>(
    action: &Action,
    content: &'content [u8],
    write_time_seconds: u64,
) -> UserChange<'content> {
    match *action {
        Action::CreateInodes { count } => UserChange::NewInodes {
            count,
            write_time_seconds,
        },
        Action::WholeWrite { inode, .. } => UserChange::SequentialWriteOfAnInode {
            inode: InodeNumber(inode),
            content,
            write_time_seconds,
        },
        Action::Overwrite { inode, .. } => UserChange::OverwriteOfAnInode {
            inode: InodeNumber(inode),
            content,
            write_time_seconds,
        },
        Action::Unlink { inode } => UserChange::Unlink {
            inode: InodeNumber(inode),
            write_time_seconds,
        },
        Action::Truncate { inode, new_size } => UserChange::Truncate {
            inode: InodeNumber(inode),
            new_size,
            write_time_seconds,
        },
        Action::CreateSnapshot => UserChange::CreateSnapshot,
        Action::CrashThenMount | Action::UnmountThenMount => unreachable!("挂载那两样不经会话发布"),
    }
}

#[allow(
    unreachable_patterns,
    reason = "探针只认成员名；核心层加了成员这里照样编得过、落到 other"
)]
fn refusal_kind(refusal: &UserChangeRefused) -> &'static str {
    match refusal {
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. } => {
            "DeviceTableOtherThanTheOneOfTheMount"
        }
        UserChangeRefused::NoFileVersionToChange { .. } => "NoFileVersionToChange",
        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => {
            "DevicesWithoutASelfVerifiedSystemConfiguration"
        }
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => {
            "OwnDeviceNumbersDifferFromTheDeviceTable"
        }
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => {
            "DevicesBehindTheCurrentVersion"
        }
        UserChangeRefused::Publish { .. } => "Publish",
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => "NoSpaceAfterRaisingTheFloor",
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => {
            "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease"
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {
            "FloorRaiseFailedWhilePushingForSpace"
        }
        _ => "other",
    }
}

fn versions_published_by(refusal: &UserChangeRefused) -> usize {
    match refusal {
        UserChangeRefused::NoFileVersionToChange {
            versions_published, ..
        }
        | UserChangeRefused::Publish {
            versions_published, ..
        } => versions_published.len(),
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => refused.versions_published.len(),
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused) => {
            refused.versions_published.len()
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused) => {
            refused.versions_published.len()
        }
        _ => 0,
    }
}

fn is_stuck_behind_the_wide_protection(refusal: &UserChangeRefused) -> bool {
    matches!(
        refusal,
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)
            if matches!(refused.wide_protection_at_the_ceiling, WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_))
    )
}

pub struct Options {
    pub bound_without_deferred: bool,
    pub soundness_without_deferred: bool,
}

/// 一条历史：盘宽、步数与每一步的动作都由种子定。
pub fn run_history(seed: u64, options: &Options) -> Tally {
    let mut tally = Tally {
        histories: 1,
        ..Tally::default()
    };
    let mut random = Random(seed);
    probe::set_bound_without_deferred(options.bound_without_deferred);
    let widths = [0_u64, 0, 0, 64, 256, 1024];
    let extra = [
        widths[usize::try_from(random.below(6)).expect("下标")],
        widths[usize::try_from(random.below(6)).expect("下标")],
    ];
    let steps = 60 + random.below(61);
    let mut pool = match Pool::start(extra) {
        Ok(pool) => pool,
        Err(refusal) => {
            add(
                &mut tally.setup_refused,
                format!("extra={extra:?} {}", leading_text(&refusal)),
                1,
            );
            return tally;
        }
    };
    let mut last_refused: Option<Action> = None;
    for step in 0..steps {
        tally.steps += 1;
        let action = choose(&mut random, &pool, &last_refused);
        let label = format!("seed={seed} extra={extra:?} step={step} action={action:?}");
        match action {
            Action::CrashThenMount | Action::UnmountThenMount => {
                let outcome = if matches!(action, Action::CrashThenMount) {
                    pool.crash_then_mount().map(|_| None)
                } else {
                    pool.unmount_then_mount()
                };
                match outcome {
                    Ok(unmount_refusal) => add(
                        &mut tally.calls,
                        format!(
                            "{} {}",
                            action.name(),
                            unmount_refusal.unwrap_or_else(|| "done".to_string())
                        ),
                        1,
                    ),
                    Err(refusal) => {
                        add(
                            &mut tally.mount_refused,
                            format!("{} {refusal}", action.name()),
                            1,
                        );
                        return tally;
                    }
                }
                continue;
            }
            _ => {}
        }
        let watermark = pool.inode_number_watermark();
        if matches!(action, Action::CreateInodes { .. }) && watermark.is_none() {
            add(
                &mut tally.calls,
                "create-inodes skipped-without-file".to_string(),
                1,
            );
            continue;
        }
        let copy = pool.copy();
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let content = match action {
            Action::WholeWrite { length, .. } | Action::Overwrite { length, .. } => {
                content_of(length, write_time_seconds)
            }
            _ => Vec::new(),
        };
        let change = change_for(&action, &content, write_time_seconds);
        probe::set_bypass(false);
        let _ = probe::take_prechecks();
        let _ = probe::take_raises();
        let outcome = pool
            .session
            .as_mut()
            .expect("这一刻有可写会话")
            .publish_user_change(&mut pool.devices, change);
        let prechecks = probe::take_prechecks();
        let raises = probe::take_raises();
        tally.note_prechecks(&prechecks, outcome.is_ok(), &label);
        tally.note_raises(&raises, options.soundness_without_deferred, &label);
        // 每一次先算点名的每块盘按哪一判、判没判出放不下计数（提交内生块那一判没有上界，也在这里数到）。
        for precheck in &prechecks {
            for short in &precheck.short_devices {
                let verdict = if precheck.fired { "fired" } else { "pushed" };
                add(
                    &mut tally.calls,
                    format!("named {} {verdict}", short.judgement),
                    1,
                );
            }
        }
        match &outcome {
            Ok(published) => {
                let pushed = if published.floor_raises.is_empty() {
                    "published"
                } else {
                    "published-after-pushing"
                };
                if !published.floor_raises.is_empty() {
                    tally.pushed_then_published_calls += 1;
                }
                add(&mut tally.calls, format!("{} {pushed}", action.name()), 1);
                last_refused = None;
                match action {
                    Action::CreateInodes { count } => {
                        let first = watermark.expect("上面判过有水位");
                        for offset in 0..count {
                            pool.files.insert(first + offset, 0);
                        }
                    }
                    Action::WholeWrite { inode, length } | Action::Overwrite { inode, length } => {
                        pool.files.insert(inode, length as u64);
                    }
                    Action::Unlink { inode } => {
                        pool.files.remove(&inode);
                    }
                    Action::Truncate { inode, new_size } => {
                        pool.files.insert(inode, new_size);
                    }
                    Action::CreateSnapshot => pool.snapshots_created += 1,
                    Action::CrashThenMount | Action::UnmountThenMount => {}
                }
            }
            Err(refusal) => {
                add(
                    &mut tally.calls,
                    format!("{} {}", action.name(), refusal_kind(refusal)),
                    1,
                );
                if !raises.is_empty() {
                    tally.pushed_then_refused_calls += 1;
                }
                if is_stuck_behind_the_wide_protection(refusal) {
                    tally.stuck_reported_with_the_precheck += 1;
                }
                if let Action::Unlink { inode } = action {
                    if versions_published_by(refusal) > 0 {
                        pool.files.remove(&inode);
                    }
                }
                last_refused = Some(action.clone());
                if let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
                    refused,
                ) = refusal
                {
                    let round = if refused.floor_raises.is_empty() {
                        tally.first_round_fired += 1;
                        "first-round"
                    } else {
                        tally.later_round_fired += 1;
                        // 推过之后才判出放不下：头一次被拒点名的是哪几判、判出放不下的那一次点名的是哪几判（落得下先拒、式子后判出的那一形在这里分得出）。
                        let judgements_of = |precheck: Option<&probe::PrecheckObserved>| {
                            precheck
                                .map(|precheck| {
                                    precheck
                                        .short_devices
                                        .iter()
                                        .map(|short| short.judgement)
                                        .collect::<Vec<_>>()
                                        .join("+")
                                })
                                .unwrap_or_default()
                        };
                        let first = judgements_of(prechecks.first());
                        let fired =
                            judgements_of(prechecks.iter().rev().find(|precheck| precheck.fired));
                        add(
                            &mut tally.calls,
                            format!("later-round-shape first={first} fired={fired}"),
                            1,
                        );
                        "later-round"
                    };
                    let named = prechecks
                        .iter()
                        .rev()
                        .find(|precheck| precheck.fired)
                        .map(describe)
                        .unwrap_or_default();
                    run_the_change_without_the_precheck(
                        &mut tally,
                        copy,
                        change,
                        options,
                        &format!("{label} round={round} named={named}"),
                        round,
                    );
                }
            }
        }
    }
    tally
}

/// 拷贝上旁路先算发同一个改动（先算之前的推法），推满仍不够就照用户重发那样再发，至多一共三次。写进了就记命中。
fn run_the_change_without_the_precheck(
    tally: &mut Tally,
    copy: Pool,
    change: UserChange<'_>,
    options: &Options,
    label: &str,
    round: &str,
) {
    let mut copy = copy;
    probe::set_bypass(true);
    for attempt in 0..3 {
        let _ = probe::take_prechecks();
        let outcome = copy
            .session
            .as_mut()
            .expect("拷贝上有可写会话")
            .publish_user_change(&mut copy.devices, change);
        let _ = probe::take_prechecks();
        let raises = probe::take_raises();
        tally.note_raises(&raises, options.soundness_without_deferred, label);
        match outcome {
            Ok(published) => {
                add(
                    &mut tally.oracle,
                    format!("{round} attempt={attempt} published"),
                    1,
                );
                let kind = if attempt == 0 {
                    "HIT_MISREFUSAL"
                } else {
                    "HIT_MISREFUSAL_AFTER_RESENDING"
                };
                tally.hits.push(format!(
                    "{kind} {label} attempt={attempt} floor_raises_of_the_publish={}",
                    published.floor_raises.len()
                ));
                break;
            }
            Err(refusal) => {
                let stuck = is_stuck_behind_the_wide_protection(&refusal);
                add(
                    &mut tally.oracle,
                    format!(
                        "{round} attempt={attempt} {}{}",
                        refusal_kind(&refusal),
                        if stuck { " stuck" } else { "" }
                    ),
                    1,
                );
                if stuck {
                    tally
                        .hits
                        .push(format!("STUCK_REPORT_LOST {label} attempt={attempt}"));
                }
                if !matches!(refusal, UserChangeRefused::NoSpaceAfterRaisingTheFloor(_)) {
                    break;
                }
            }
        }
    }
    probe::set_bypass(false);
}
