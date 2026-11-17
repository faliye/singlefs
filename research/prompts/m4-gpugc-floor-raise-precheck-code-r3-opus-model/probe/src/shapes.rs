//! 一个配置：两块盘宽、一段在同一次挂载里的前缀（建文件整份写、小覆盖写、unlink、整份重写，按种子放开）、
//! 要不要在前缀之后崩在一次写的第一条记录之后再挂（C 那一圈只罩那一槽的自然崩溃点，不带任何读故障），再挂之后几次小写；
//! 探测点上先挑目标（重写第一个文件或新建 inode），再发 0 / 4 / 8 次小覆盖写（每次一个改用户可见状态的发布，抬 F 的上限跟着往上走），
//! 然后把一次大写的数据单元数 g 从「段外槽对最少那块盘」往上扫到「全部算上也放不下」之后几格，每个 g 在两份拷贝上发：
//!   C 照冻结副本（撤回之后的先算）；O 关掉先算（被拒就推，推满为止）。
//! C 推了一串而没写进（白推）的格按头一次被拒是哪一判、最后报什么分类；C 判出放不下而 O 写进的格是误拒。
//! 出路（每一类在每个探测点各有份额，按这一类第 1、2、4、8、16 格取，不让先到的一类吃光份额）：
//!   被宽护卡住（stuck）：要出路时的推环预演，再照「正常卸载、再可写挂载」走一次、重发同一个写；
//!   C 那一圈罩着而先算头一次就判出放不下（lost-first）、推过一串之后判出（lost-after）、白推而没报出路（white-no-stuck）：照卸载重挂走一次、重发。

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use singlefs_core::address::InodeNumber;
use singlefs_core::mount::{FloorRaiseStop, RaisedFloor};
use singlefs_core::mounted_session::floor_raise_precheck_r3_probe::{
    self as probe, Precheck, PrecheckSeen,
};
use singlefs_core::mounted_session::{
    RootRingPushRehearsal, StuckBehindTheWideProtection, UserChange, UserChangeRefused,
    WideProtectionAtTheCeiling,
};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;

use crate::pool::{content_of, leading_text, Pool};
use crate::tally::Tally;

/// splitmix64：每个配置一个种子，结局只由种子定。
pub struct Random(pub u64);

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
    fn pick<T: Copy>(&mut self, choices: &[T]) -> T {
        choices[usize::try_from(self.below(choices.len() as u64)).expect("下标")]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    OneLongMount,
    CrashAfterTheFirstRecord,
}

/// C 那一臂的先算怎么交结局：照代码（全量），一律放得下（自证：C 与 O 处处相同），一律放不下（自证：误拒那一数要能红）。
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub arm_c: Precheck,
}

const NO_SPACE_EVEN: &str = "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease";
const NO_SPACE_AFTER: &str = "NoSpaceAfterRaisingTheFloor";
/// 一个探测点上每一类出路至多走几次。
const FOLLOWS_PER_CATEGORY: u32 = 5;
/// 一个探测点上至多扫几个 g。
const SIZES_PER_PROBE_POINT: u64 = 160;
/// C 与 O 都推不动、都没写进连着这么多格，就停扫（再往上需求只更大）。
const BOTH_HOPELESS_IN_A_ROW_TO_STOP: u32 = 6;

struct Outcome {
    kind: &'static str,
    series: usize,
    empty_publishes: usize,
    stuck: bool,
    stop: &'static str,
    cause: String,
    versions_published: usize,
    floor_before: u64,
    floor_after: u64,
    seen: Vec<PrecheckSeen>,
    refusals: Vec<String>,
    refusal: Option<UserChangeRefused>,
}

impl Outcome {
    fn published(&self) -> bool {
        self.kind == "published"
    }
    fn fired_first(&self) -> bool {
        self.kind == NO_SPACE_EVEN && self.series == 0
    }
    fn fired_after_a_series(&self) -> bool {
        self.kind == NO_SPACE_EVEN && self.series > 0
    }
    /// 推了至少一串、这次改动没写进。
    fn white(&self) -> bool {
        !self.published() && self.series > 0
    }
    fn label(&self) -> String {
        format!(
            "{}{}{}",
            self.kind,
            if self.series > 0 { "+pushed" } else { "" },
            if self.stuck { "+stuck" } else { "" }
        )
    }
    /// 白推的最后一格是什么：卡在上限（不报出路）、预算、抬 F 那一串自己被空间拒、被宽护卡住、推过之后先算判出、别的拒绝。
    fn final_class(&self) -> String {
        if self.stuck {
            return "stuck".to_string();
        }
        match self.kind {
            NO_SPACE_AFTER => format!("after-{}", self.stop),
            NO_SPACE_EVEN => "fired-after-a-series".to_string(),
            "Publish" => "publish-refusal".to_string(),
            other => format!("other-{other}"),
        }
    }
    /// 头一次被拒是哪一判（先算读到的那几块盘的判，去重）；先算没读盘（取不到落点、别的拒绝）就是那一次拒绝的成员名。
    fn first_judgement(&self) -> String {
        let judgements: BTreeSet<&str> = self
            .seen
            .iter()
            .filter(|seen| seen.refusal_index == 1)
            .map(|seen| seen.judgement)
            .collect();
        if judgements.is_empty() {
            self.refusals.first().cloned().unwrap_or_else(|| "-".to_string())
        } else {
            judgements.into_iter().collect::<Vec<_>>().join("+")
        }
    }
    fn stuck_report(&self) -> Option<&StuckBehindTheWideProtection> {
        match &self.refusal {
            Some(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => {
                match &refused.wide_protection_at_the_ceiling {
                    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) => {
                        Some(stuck.as_ref())
                    }
                    WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => None,
                }
            }
            _ => None,
        }
    }
}

#[allow(
    unreachable_patterns,
    reason = "探针只认成员名；核心层加了成员这里照样编得过、落到 other"
)]
fn refusal_kind(refusal: &UserChangeRefused) -> &'static str {
    match refusal {
        UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. } => "DeviceTable",
        UserChangeRefused::NoFileVersionToChange { .. } => "NoFileVersionToChange",
        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => {
            "DevicesWithoutASelfVerifiedSystemConfiguration"
        }
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => "OwnDeviceNumbers",
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. } => {
            "DevicesBehind"
        }
        UserChangeRefused::Publish { .. } => "Publish",
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => NO_SPACE_AFTER,
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => NO_SPACE_EVEN,
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {
            "FloorRaiseFailedWhilePushingForSpace"
        }
        _ => "other",
    }
}

#[allow(
    unreachable_patterns,
    reason = "探针只认带推串的成员；核心层加了成员这里照样编得过、当没推"
)]
fn series_of(refusal: &UserChangeRefused) -> (usize, usize) {
    let raises: &[RaisedFloor] = match refusal {
        UserChangeRefused::NoFileVersionToChange { floor_raises, .. }
        | UserChangeRefused::Publish { floor_raises, .. } => floor_raises.as_slice(),
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => refused.floor_raises.as_slice(),
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused) => {
            refused.floor_raises.as_slice()
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused) => {
            refused.floor_raises.as_slice()
        }
        _ => &[],
    };
    (
        raises.len(),
        raises.iter().map(|raised| raised.publishes.len()).sum(),
    )
}

#[allow(
    unreachable_patterns,
    reason = "探针只认带已发成那几版的成员；核心层加了成员这里照样编得过、当 0"
)]
fn versions_of(refusal: &UserChangeRefused) -> usize {
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

fn is_stuck(refusal: &UserChangeRefused) -> bool {
    matches!(
        refusal,
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)
            if matches!(refused.wide_protection_at_the_ceiling, WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_))
    )
}

#[allow(
    unreachable_patterns,
    reason = "探针只认成员名；核心层加了成员这里照样编得过、落到 other"
)]
fn stop_of(refusal: &UserChangeRefused) -> &'static str {
    match refusal {
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => match &refused.stop {
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => "budget",
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => "ceiling",
            FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => "raise-refused-for-space",
            _ => "other",
        },
        _ => "-",
    }
}

fn cause_of(refusal: &UserChangeRefused) -> String {
    match refusal {
        UserChangeRefused::Publish { cause, .. } => leading_text(&format!("{cause:?}")),
        _ => "-".to_string(),
    }
}

fn write_of(inode: u64, content: &[u8], write_time_seconds: u64) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode: InodeNumber(inode),
        content,
        write_time_seconds,
    }
}

fn publish_observed(pool: &mut Pool, change: UserChange<'_>, mode: Precheck) -> Outcome {
    probe::set_precheck(mode);
    let _ = probe::take_seen();
    let _ = probe::take_refusals();
    let (floor_before, _) = pool.floor_and_txg();
    let result = pool.publish(change);
    let seen = probe::take_seen();
    let refusals = probe::take_refusals();
    probe::set_precheck(Precheck::AsCoded);
    let (floor_after, _) = pool.floor_and_txg();
    match result {
        Ok(published) => Outcome {
            kind: "published",
            series: published.floor_raises.len(),
            empty_publishes: published
                .floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .sum(),
            stuck: false,
            stop: "-",
            cause: "-".to_string(),
            versions_published: published.versions_published.len(),
            floor_before,
            floor_after,
            seen,
            refusals,
            refusal: None,
        },
        Err(refusal) => {
            let (series, empty_publishes) = series_of(&refusal);
            Outcome {
                kind: refusal_kind(&refusal),
                series,
                empty_publishes,
                stuck: is_stuck(&refusal),
                stop: stop_of(&refusal),
                cause: cause_of(&refusal),
                versions_published: versions_of(&refusal),
                floor_before,
                floor_after,
                seen,
                refusals,
                refusal: Some(refusal),
            }
        }
    }
}

/// 一块盘这一刻的读数（分配器增量维护的数，与先算读的同一组），加记录口径的全空聚簇段数（合取 7 读的那一个）。
#[derive(Clone, Copy, Debug)]
struct DeviceReading {
    outside: u64,
    inside: u64,
    deferred: u64,
    isolated: u64,
    free_but_blocked: u64,
    empty_segments: u64,
}

fn readings(pool: &Pool) -> Vec<DeviceReading> {
    let allocator = &pool.session().allocator;
    let segments = allocator.cluster_segments();
    allocator
        .devices
        .iter()
        .map(|device_map| {
            let all = device_map.unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new());
            let outside = device_map.unblocked_slot_pairs_outside_the_cluster_segments(segments);
            DeviceReading {
                outside,
                inside: all - outside,
                deferred: device_map.deferred_slots(),
                isolated: device_map.isolated_slots(),
                free_but_blocked: device_map.free_slots() - device_map.unblocked_slots(),
                empty_segments: device_map.empty_segments(),
            }
        })
        .collect()
}

/// C 那一圈这一刻还记着落点的槽数。
fn holding_ring_slots(pool: &Pool) -> usize {
    pool.session()
        .allocator
        .root_ring_occupancy()
        .map(|occupancy| {
            occupancy
                .ring_slots_holding_placements_named_by_unapplied_journal_records()
                .len()
        })
        .unwrap_or(0)
}

/// 合取 7（全空聚簇段数 ≥ N₀）这一刻在哪一块盘上不成立：有一块就是 true。
fn conjunct7_short(pool: &Pool) -> bool {
    let allocator = &pool.session().allocator;
    let floor = allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments();
    allocator
        .devices
        .iter()
        .any(|device_map| device_map.empty_segments() < floor)
}

fn bucket(value: u64) -> &'static str {
    match value {
        0 => "0",
        1 => "1",
        2..=3 => "2-3",
        4..=7 => "4-7",
        8..=15 => "8-15",
        16..=31 => "16-31",
        32..=63 => "32-63",
        64..=127 => "64-127",
        _ => "128+",
    }
}

/// 先算读的头一块与最后一块盘（槽数）：判、可用、需求、三样槽数、上界、照代码判没判出。
fn describe_seen(seen: &[PrecheckSeen]) -> String {
    let slots = |bytes: i128| bytes / i128::from(SLOT_BYTES);
    let one = |record: &PrecheckSeen| {
        format!(
            "[{}#{} dev={} available={} demand={} deferred={} isolated={} blocked={} bound={} fired={}]",
            record.judgement,
            record.refusal_index,
            record.device,
            slots(record.available_bytes),
            slots(i128::from(record.demand_bytes)),
            record.deferred,
            record.isolated,
            record.free_but_blocked,
            record.bound.map(slots).map_or("-".to_string(), |v| v.to_string()),
            record.fired_as_coded
        )
    };
    match (seen.first(), seen.last()) {
        (Some(first), Some(last)) if seen.len() > 1 => format!("{}..{}", one(first), one(last)),
        (Some(first), _) => one(first),
        _ => "[]".to_string(),
    }
}

/// 白推的格为什么推了也写不进（只读 C 推完之后那一刻的分配器，按头一次被拒那几块盘）：
/// 槽对那一判——推完之后段外槽对仍少于需求，而推之前上界不小于需求（「每个放得开的槽各补一对」那一截推抬 F 放不出）；
/// 式子那一判——推完之后 defer 里还剩的槽（这一串的上限够不到的）从上界里减掉就小于需求。
fn white_push_explanations(outcome: &Outcome, after: &[DeviceReading], devices: &[u32]) -> Vec<&'static str> {
    let slots_of_a_pair = 2_i128;
    let slot = i128::from(SLOT_BYTES);
    let mut explanations = Vec::new();
    for seen in outcome.seen.iter().filter(|seen| seen.refusal_index == 1) {
        let Some(position) = devices.iter().position(|device| *device == seen.device) else {
            continue;
        };
        let Some(reading) = after.get(position) else {
            continue;
        };
        let demand = i128::from(seen.demand_bytes);
        let Some(bound) = seen.bound else {
            continue;
        };
        match seen.judgement {
            "slot-pairs" => {
                if i128::from(reading.outside) * slots_of_a_pair * slot < demand && bound >= demand {
                    explanations.push("slot-pairs-bound-loose");
                }
            }
            "formula" => {
                if bound - i128::from(reading.deferred) * slot < demand {
                    explanations.push("formula-unreachable-defer");
                }
            }
            _ => {}
        }
    }
    explanations
}

/// 前缀的一步（同一次挂载里）：建文件整份写、小覆盖写、unlink、整份重写。被拒照常记（推过的串留在盘上），不还原。
fn prefix_step(random: &mut Random, pool: &mut Pool, tally: &mut Tally) {
    let payload = data_unit_payload_capacity();
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    let written: Vec<u64> = pool
        .files
        .iter()
        .filter(|(_, units)| **units > 0)
        .map(|(inode, _)| *inode)
        .collect();
    match random.below(100) {
        0..=34 => {
            let Some(first) = pool.inode_number_watermark() else {
                return;
            };
            if let Err(refusal) = pool.publish(UserChange::NewInodes {
                count: 1,
                write_time_seconds,
            }) {
                tally.add(format!("prefix create {}", refusal_kind(&refusal)), 1);
                return;
            }
            pool.files.insert(first, 0);
            let units = random.pick(&[1_usize, 2, 4, 8, 16, 24]);
            let content = content_of(units * payload, write_time_seconds);
            match pool.publish(write_of(first, &content, write_time_seconds)) {
                Ok(_) => {
                    pool.files.insert(first, units as u64);
                    tally.add("prefix write published", 1);
                }
                Err(refusal) => tally.add(format!("prefix write {}", refusal_kind(&refusal)), 1),
            }
        }
        35..=69 => {
            if written.is_empty() {
                return;
            }
            let inode = random.pick(&written);
            let length = 1 + usize::try_from(random.below(payload as u64)).expect("长度");
            let content = content_of(length, write_time_seconds);
            let change = UserChange::OverwriteOfAnInode {
                inode: InodeNumber(inode),
                content: &content,
                write_time_seconds,
            };
            match pool.publish(change) {
                Ok(_) => tally.add("prefix overwrite published", 1),
                Err(refusal) => tally.add(format!("prefix overwrite {}", refusal_kind(&refusal)), 1),
            }
        }
        70..=84 => {
            let candidates: Vec<u64> = pool
                .files
                .keys()
                .copied()
                .filter(|inode| *inode != FIRST_INODE_NUMBER)
                .collect();
            if candidates.is_empty() {
                return;
            }
            let inode = random.pick(&candidates);
            let outcome = pool.publish(UserChange::Unlink {
                inode: InodeNumber(inode),
                write_time_seconds,
            });
            // 被拒时意图可能已发成：这个文件之后一概不碰。
            pool.files.remove(&inode);
            match outcome {
                Ok(_) => tally.add("prefix unlink published", 1),
                Err(refusal) => tally.add(format!("prefix unlink {}", refusal_kind(&refusal)), 1),
            }
        }
        _ => {
            if written.is_empty() {
                return;
            }
            let inode = random.pick(&written);
            let units = random.pick(&[1_usize, 2, 4, 8, 16]);
            let content = content_of(units * payload, write_time_seconds);
            match pool.publish(write_of(inode, &content, write_time_seconds)) {
                Ok(_) => {
                    pool.files.insert(inode, units as u64);
                    tally.add("prefix rewrite published", 1);
                }
                Err(refusal) => tally.add(format!("prefix rewrite {}", refusal_kind(&refusal)), 1),
            }
        }
    }
}

fn small_overwrite_of_the_first_file(pool: &mut Pool) -> bool {
    pool.write_time_seconds += 1;
    let content = content_of(100, pool.write_time_seconds);
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(UserChange::OverwriteOfAnInode {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        content: &content,
        write_time_seconds,
    })
    .is_ok()
}

/// 照一类出路走一次（在 C 那一臂被拒之后的拷贝上）：报了被宽护卡住的先要出路（推环预演，一个写都不落盘），
/// 再正常卸载、再可写挂载、重发同一个写，记下重挂之后 C 那一圈还罩不罩着、重发的结局。
fn follow_the_way_out(
    mut arm: Pool,
    outcome: &Outcome,
    change: UserChange<'_>,
    category: &str,
    tally: &mut Tally,
    head: &str,
) {
    tally.add(format!("outlet {category} followed"), 1);
    let mut rehearsal = "-";
    if let Some(stuck) = outcome.stuck_report() {
        probe::set_precheck(Precheck::AsCoded);
        rehearsal = match arm
            .session()
            .ways_out_of_the_wide_protection(&arm.devices, stuck)
        {
            Ok(ways) => match ways.pushing_the_root_ring {
                RootRingPushRehearsal::ChangeAdmittedAfterThePush { .. } => "admitted",
                RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. } => "still-refused",
                RootRingPushRehearsal::PushRefused(_) => "push-refused",
            },
            Err(_) => "not-rehearsed",
        };
        let _ = probe::take_seen();
        let _ = probe::take_refusals();
        tally.add(format!("outlet {category} push-ring-rehearsal {rehearsal}"), 1);
    }
    match arm.unmount_then_mount() {
        Err(refusal) => tally.add(format!("outlet {category} remount-refused {refusal}"), 1),
        Ok(()) => {
            let holding_after = holding_ring_slots(&arm);
            let retried = publish_observed(&mut arm, change, Precheck::AsCoded);
            let verdict = if retried.published() {
                "published"
            } else {
                "refused"
            };
            tally.add(format!("outlet {category} remount {verdict}"), 1);
            tally.add(
                format!(
                    "outlet {category} remount holding-after={} D={}",
                    holding_after > 0,
                    retried.label()
                ),
                1,
            );
            tally.example(format!(
                "FOLLOWED {category} {head} rehearsal={rehearsal} holding_after={holding_after} D={} D_seen={}",
                retried.label(),
                describe_seen(&retried.seen)
            ));
        }
    }
}

/// C 推了一串而没写进的那一格：按最后报什么、头一次被拒是哪一判分类，再按推完之后的分配器给出推了也写不进的缘由。
fn tally_the_white_push(
    c: &Outcome,
    arm_c: &Pool,
    device_numbers: &[u32],
    holding: usize,
    conjunct7: bool,
    tally: &mut Tally,
    head: &str,
) {
    let first = c.first_judgement();
    let first_class = match first.as_str() {
        "slot-pairs" => "slot-pairs",
        "formula" => "formula",
        _ => "other",
    };
    let final_class = c.final_class();
    tally.add("c-white", 1);
    tally.add(format!("c-white final={final_class}"), 1);
    tally.add(format!("c-white first={first_class}"), 1);
    tally.add(
        format!(
            "c-white detail first={first} final={final_class} cause={} holding={} conjunct7-short={conjunct7}",
            c.cause,
            holding > 0
        ),
        1,
    );
    tally.add("c-white-empty-publishes", c.empty_publishes as u64);
    if final_class == "publish-refusal" && conjunct7 {
        tally.add("c-white publish-refusal conjunct7-short-at-probe", 1);
    }
    let after = readings(arm_c);
    let mut explanations = white_push_explanations(c, &after, device_numbers);
    explanations.sort_unstable();
    explanations.dedup();
    for explanation in &explanations {
        tally.add(format!("c-white {explanation}"), 1);
    }
    tally.example(format!(
        "WHITE {head} explanations={explanations:?} C_seen={} refusals={:?}",
        describe_seen(&c.seen),
        c.refusals
    ));
}

/// 探测点：挑目标、发 0 / 4 / 8 次小覆盖写抬上限，扫 g，每个 g 走 C、O 两臂，各类出路按份额走。
fn probe_point(
    pool: &mut Pool,
    random: &mut Random,
    tally: &mut Tally,
    label: &str,
    family_name: &str,
    options: &Options,
) {
    let payload = data_unit_payload_capacity();
    pool.write_time_seconds += 1;
    let target_creation_time = pool.write_time_seconds;
    let mut target = (FIRST_INODE_NUMBER, "rewrite-first");
    if random.below(2) == 0 {
        if let Some(first) = pool.inode_number_watermark() {
            if pool
                .publish(UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: target_creation_time,
                })
                .is_ok()
            {
                pool.files.insert(first, 0);
                target = (first, "new-inode");
            }
        }
    }
    let (target_inode, target_kind) = target;
    let lifts = random.pick(&[0_u32, 4, 8]);
    let mut lifts_published = 0_u32;
    for _ in 0..lifts {
        if small_overwrite_of_the_first_file(pool) {
            lifts_published += 1;
        }
    }
    tally.add(
        format!("probe-point lifts={lifts} all-published={}", lifts_published == lifts),
        1,
    );
    let before = readings(pool);
    let device_numbers: Vec<u32> = pool
        .session()
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.device.0)
        .collect();
    let holding = holding_ring_slots(pool);
    if holding > 0 {
        tally.add("c-ring-holding-at-probe", 1);
    }
    let conjunct7 = conjunct7_short(pool);
    let queue_waiting = !pool.queue_is_empty();
    if queue_waiting {
        tally.add("queue-waiting-at-probe", 1);
    }
    tally.add(
        format!(
            "probe-point {family_name} holding={} conjunct7-short={conjunct7} queue-waiting={queue_waiting} empty-segments-min={}",
            holding > 0,
            bucket(before.iter().map(|reading| reading.empty_segments).min().unwrap_or(0))
        ),
        1,
    );
    let outside_min = before.iter().map(|reading| reading.outside).min().unwrap_or(0);
    let releasable_max = before
        .iter()
        .map(|reading| {
            reading.outside
                + reading.deferred
                + reading.isolated
                + reading.free_but_blocked
                + reading.inside
        })
        .max()
        .unwrap_or(0);
    let (floor, txg) = pool.floor_and_txg();
    let low = outside_min.saturating_sub(2).max(1);
    let high = releasable_max + 3;
    let stride = ((high - low) / SIZES_PER_PROBE_POINT).max(1);
    let write_time_seconds = pool.write_time_seconds + 1;
    let mut seen_per_category: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut followed_per_category: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut both_hopeless_in_a_row = 0;
    let mut g = low;
    while g <= high {
        let content = content_of(usize::try_from(g).expect("g") * payload, write_time_seconds);
        let change = write_of(target_inode, &content, write_time_seconds);
        let mut arm_c = pool.copy();
        let c = publish_observed(&mut arm_c, change, options.arm_c);
        let mut arm_o = pool.copy();
        let o = publish_observed(&mut arm_o, change, Precheck::NeverFires);
        drop(arm_o);
        tally.add("cells", 1);
        tally.add(format!("{family_name} C={} O={}", c.label(), o.label()), 1);
        if c.published() {
            tally.add("c published", 1);
        }
        if c.kind == NO_SPACE_EVEN {
            tally.add("c-fired", 1);
        }
        if c.kind != o.kind || c.series != o.series || c.stuck != o.stuck {
            tally.add("arms-differ", 1);
        }
        let head = format!(
            "{label} target={target_kind} lifts={lifts} holding={holding} conjunct7_short={conjunct7} queue_waiting={queue_waiting} F={floor} txg={txg} outside_min={outside_min} g={g} C={} C_series={} C_empty={} C_F={}->{} C_stop={} C_cause={} C_versions={} O={} O_series={} O_empty={}",
            c.label(),
            c.series,
            c.empty_publishes,
            c.floor_before,
            c.floor_after,
            c.stop,
            c.cause,
            c.versions_published,
            o.label(),
            o.series,
            o.empty_publishes
        );
        if o.published() && o.series > 0 {
            tally.add("oracle published-after-a-series", 1);
        }
        if c.kind == NO_SPACE_EVEN && o.published() {
            tally.add("misrefused", 1);
            tally.add(
                format!(
                    "misrefused {}",
                    if c.series == 0 { "first-round" } else { "after-a-series" }
                ),
                1,
            );
            tally.example(format!("MISREFUSED {head} C_seen={}", describe_seen(&c.seen)));
        }
        if c.fired_first() && !o.published() && o.series > 0 {
            tally.add("precheck-saved-a-vain-series", 1);
        }
        if o.white() {
            tally.add("o-white", 1);
        }
        if c.fired_first() && c.versions_published > 0 {
            tally.add("fired-first-with-versions-published", 1);
            tally.example(format!("FIRED_FIRST_WITH_VERSIONS {head}"));
        }
        if c.white() {
            tally_the_white_push(&c, &arm_c, &device_numbers, holding, conjunct7, tally, &head);
        }
        let category: Option<&'static str> = if c.stuck {
            Some("stuck")
        } else if c.kind == NO_SPACE_EVEN && holding > 0 {
            Some(if c.series == 0 {
                "lost-first"
            } else {
                "lost-after"
            })
        } else if c.white() {
            Some("white-no-stuck")
        } else {
            None
        };
        if c.fired_first() && !o.published() {
            both_hopeless_in_a_row += 1;
        } else {
            both_hopeless_in_a_row = 0;
        }
        let mut arm_c = Some(arm_c);
        if let Some(category) = category {
            let seen_count = seen_per_category.entry(category).or_insert(0);
            *seen_count += 1;
            let followed_count = followed_per_category.entry(category).or_insert(0);
            if seen_count.is_power_of_two() && *followed_count < FOLLOWS_PER_CATEGORY {
                *followed_count += 1;
                if let Some(arm) = arm_c.take() {
                    follow_the_way_out(arm, &c, change, category, tally, &head);
                }
            }
        }
        drop(arm_c);
        if both_hopeless_in_a_row >= BOTH_HOPELESS_IN_A_ROW_TO_STOP {
            break;
        }
        g += stride;
    }
    for (category, count) in &seen_per_category {
        tally.add(format!("probe-point category {category} cells {}", bucket(u64::from(*count))), 1);
    }
}

/// 一个配置：盘宽、家族、前缀长度都由种子定（与第二轮攻方探针同一套抽法，同一个种子前缀相同）。
pub fn run_configuration(seed: u64, options: &Options) -> Tally {
    let mut tally = Tally {
        configurations: 1,
        ..Tally::default()
    };
    let mut random = Random(seed);
    let extra = [random.pick(&[0_u64, 64, 256]), random.pick(&[0_u64, 64, 256])];
    let family = if random.below(2) == 0 {
        Family::OneLongMount
    } else {
        Family::CrashAfterTheFirstRecord
    };
    let prefix_steps = random.pick(&[40_u32, 120, 300]);
    let family_name = match family {
        Family::OneLongMount => "one-long-mount",
        Family::CrashAfterTheFirstRecord => "crash-after-the-first-record",
    };
    let label = format!("seed={seed} extra={extra:?} family={family_name} prefix={prefix_steps}");
    let mut pool = match Pool::start(extra) {
        Ok(pool) => pool,
        Err(refusal) => {
            tally.add(format!("setup-refused {}", leading_text(&refusal)), 1);
            return tally;
        }
    };
    for _ in 0..prefix_steps {
        prefix_step(&mut random, &mut pool, &mut tally);
    }
    let mut label = label;
    if family == Family::CrashAfterTheFirstRecord {
        let payload = data_unit_payload_capacity();
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let Some(first) = pool.inode_number_watermark() else {
            tally.add("c-ring setup no-file-version", 1);
            return tally;
        };
        if pool
            .publish(UserChange::NewInodes {
                count: 1,
                write_time_seconds,
            })
            .is_err()
        {
            tally.add("c-ring setup create-refused", 1);
            return tally;
        }
        pool.files.insert(first, 0);
        let most_units = random.pick(&[1_usize, 2, 4]);
        let mut chosen = None;
        for units in (1..=most_units).rev() {
            let content = content_of(units * payload, write_time_seconds);
            let mut trial = pool.copy();
            if let Ok(published) = trial.publish(write_of(first, &content, write_time_seconds)) {
                if published.floor_raises.is_empty() {
                    chosen = Some(units);
                    break;
                }
            }
        }
        let Some(units) = chosen else {
            tally.add("c-ring setup write-does-not-fit-without-pushing", 1);
            return tally;
        };
        let content = content_of(units * payload, write_time_seconds);
        match pool.crash_after_the_first_record_of(write_of(first, &content, write_time_seconds)) {
            Ok(cut) => tally.add(
                format!(
                    "c-ring setup cut={cut} holding-after-mount={}",
                    holding_ring_slots(&pool) > 0
                ),
                1,
            ),
            Err(refusal) => {
                tally.add(format!("c-ring setup mount-refused {refusal}"), 1);
                return tally;
            }
        }
        let small_writes = random.pick(&[0_u32, 1, 3, 6]);
        for _ in 0..small_writes {
            if !small_overwrite_of_the_first_file(&mut pool) {
                tally.add("c-ring small-write refused", 1);
            }
        }
        label = format!("{label} r_units={units} small_writes={small_writes}");
    }
    probe_point(&mut pool, &mut random, &mut tally, &label, family_name, options);
    tally
}
