//! 一个配置：两块盘宽、一段在同一次挂载里的前缀（建文件整份写、小覆盖写、unlink、整份重写，按种子放开）、
//! 要不要在前缀之后崩在一次写的第一条记录之后再挂（C 那一圈只罩那一槽的自然崩溃点，不带任何读故障），再挂之后几次小写；
//! 然后在探测点上把一次大写的数据单元数 g 从「段外槽对最少的那块盘」往上扫到「全部算上也放不下」之后几格，每个 g 在三份拷贝上发：
//!   A 照冻结副本（上界算聚簇段里那一截）；B 那一截读成 0（第一轮修补之前）；A 被拒的拷贝上照出路正常卸载再可写挂载、再发同一个写（D）。
//! 只在 A 被拒、而 A 与 B 判得不一样、或 A 报被宽护卡住、或 A 在 C 那一圈还罩着时判出放不下时走 D。

use std::collections::BTreeSet;

use singlefs_core::address::InodeNumber;
use singlefs_core::mount::RaisedFloor;
use singlefs_core::mounted_session::floor_raise_precheck_r2_probe::{
    self as probe, InsideTerm, PrecheckSeen,
};
use singlefs_core::mounted_session::{UserChange, UserChangeRefused, WideProtectionAtTheCeiling};
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

/// A 那一臂的聚簇段那一截怎么读：照代码（全量），读成 0（自证：A 与 B 必须处处相同），读成大数（自证：判法与分类接得上）。
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub arm_a: InsideTerm,
}

const NO_SPACE_EVEN: &str = "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease";
/// 一个探测点上至多走几次卸载再挂（D）。
const REMOUNTS_PER_PROBE_POINT: u32 = 10;
/// 一个探测点上至多扫几个 g。
const SIZES_PER_PROBE_POINT: u64 = 160;
/// A 与 B 都头一次就判出放不下连着这么多格，就停扫（再往上需求只更大）。
const BOTH_FIRED_IN_A_ROW_TO_STOP: u32 = 6;

#[derive(Clone, Debug)]
struct Outcome {
    kind: &'static str,
    series: usize,
    empty_publishes: usize,
    stuck: bool,
    floor_before: u64,
    floor_after: u64,
    seen: Vec<PrecheckSeen>,
}

impl Outcome {
    fn published(&self) -> bool {
        self.kind == "published"
    }
    fn fired_first_round(&self) -> bool {
        self.kind == NO_SPACE_EVEN && self.series == 0
    }
    fn label(&self) -> String {
        format!(
            "{}{}{}",
            self.kind,
            if self.series > 0 { "+pushed" } else { "" },
            if self.stuck { "+stuck" } else { "" }
        )
    }
    fn slot_pairs_seen(&self) -> bool {
        self.seen.iter().any(|seen| seen.judgement == "slot-pairs")
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
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => "NoSpaceAfterRaisingTheFloor",
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

fn is_stuck(refusal: &UserChangeRefused) -> bool {
    matches!(
        refusal,
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)
            if matches!(refused.wide_protection_at_the_ceiling, WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_))
    )
}

fn write_of(inode: u64, content: &[u8], write_time_seconds: u64) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode: InodeNumber(inode),
        content,
        write_time_seconds,
    }
}

fn publish_observed(pool: &mut Pool, change: UserChange<'_>, term: InsideTerm) -> Outcome {
    probe::set_inside_term(term);
    let _ = probe::take_seen();
    let (floor_before, _) = pool.floor_and_txg();
    let result = pool.publish(change);
    let seen = probe::take_seen();
    probe::set_inside_term(InsideTerm::AsCoded);
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
            floor_before,
            floor_after,
            seen,
        },
        Err(refusal) => {
            let (series, empty_publishes) = series_of(&refusal);
            Outcome {
                kind: refusal_kind(&refusal),
                series,
                empty_publishes,
                stuck: is_stuck(&refusal),
                floor_before,
                floor_after,
                seen,
            }
        }
    }
}

/// 一块盘这一刻的槽对读数（分配器增量维护的数，与先算读的同一组）。
#[derive(Clone, Copy, Debug)]
struct DeviceReading {
    outside: u64,
    inside: u64,
    deferred: u64,
    isolated: u64,
    free_but_blocked: u64,
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

fn signed_bucket(value: i64) -> String {
    if value < 0 {
        format!("-{}", bucket(value.unsigned_abs()))
    } else {
        bucket(value.unsigned_abs()).to_string()
    }
}

/// 先算读的第一块槽对那一判点名的盘与最后一块（槽数）：聚簇段那一截、两种上界、需求。
fn describe_seen(seen: &[PrecheckSeen]) -> String {
    let slots = |bytes: i128| bytes / i128::from(SLOT_BYTES);
    let one = |record: &PrecheckSeen| {
        format!(
            "[{}#{} dev={} available={} demand={} deferred={} isolated={} blocked={} inside_pairs={} bound_without={} bound_with={} fired={}]",
            record.judgement,
            record.refusal_index,
            record.device,
            slots(record.available_bytes),
            slots(i128::from(record.demand_bytes)),
            record.deferred,
            record.isolated,
            record.free_but_blocked,
            record.inside_pairs,
            record.bound_without_inside.map(slots).map_or("-".to_string(), |v| v.to_string()),
            record.bound_with_inside.map(slots).map_or("-".to_string(), |v| v.to_string()),
            record.fired
        )
    };
    match (seen.first(), seen.last()) {
        (Some(first), Some(last)) if seen.len() > 1 => format!("{}..{}", one(first), one(last)),
        (Some(first), _) => one(first),
        _ => "[]".to_string(),
    }
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

/// 探测点：扫 g，每个 g 走 A、B 两臂，要的时候在 A 被拒的拷贝上照出路卸载再挂（D）。
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
    let before = readings(pool);
    let holding = holding_ring_slots(pool);
    if holding > 0 {
        tally.add("c-ring-holding-at-probe", 1);
    }
    let outside_min = before.iter().map(|reading| reading.outside).min().unwrap_or(0);
    let inside_min = before.iter().map(|reading| reading.inside).min().unwrap_or(0);
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
    tally.add(
        format!("probe-point {family_name} holding={} inside-pairs {}", holding > 0, bucket(inside_min)),
        1,
    );
    let (floor, txg) = pool.floor_and_txg();
    let low = outside_min.saturating_sub(2).max(1);
    let high = releasable_max + 3;
    let stride = ((high - low) / SIZES_PER_PROBE_POINT).max(1);
    let write_time_seconds = pool.write_time_seconds + 1;
    let mut remounts_left = REMOUNTS_PER_PROBE_POINT;
    let mut retried_after_a_small_write = false;
    let mut both_fired_in_a_row = 0;
    let mut vain_sizes = 0_u64;
    let mut g = low;
    while g <= high {
        let content = content_of(usize::try_from(g).expect("g") * payload, write_time_seconds);
        let change = write_of(target_inode, &content, write_time_seconds);
        let mut arm_a = pool.copy();
        let a = publish_observed(&mut arm_a, change, options.arm_a);
        let mut arm_b = pool.copy();
        let b = publish_observed(&mut arm_b, change, InsideTerm::Zero);
        drop(arm_b);
        if a.slot_pairs_seen() || b.slot_pairs_seen() {
            tally.add("slot-pair-refusal-calls", 1);
        }
        tally.add(format!("{family_name} A={} B={}", a.label(), b.label()), 1);
        if a.kind != b.kind || a.series != b.series || a.stuck != b.stuck {
            tally.add("arms-differ", 1);
        }
        if a.stuck {
            tally.add("stuck-reports", 1);
        }
        let vain = !a.published() && a.series > 0 && b.fired_first_round();
        let published_where_the_old_bound_fired = a.published() && b.fired_first_round();
        let lost_with_the_c_ring = a.kind == NO_SPACE_EVEN && holding > 0;
        let head = format!(
            "{label} target={target_kind} holding={holding} F={floor} txg={txg} outside_min={outside_min} inside_min={inside_min} g={g} A={} A_series={} A_empty_publishes={} A_F={}->{} B={}",
            a.label(),
            a.series,
            a.empty_publishes,
            a.floor_before,
            a.floor_after,
            b.label()
        );
        if published_where_the_old_bound_fired {
            tally.add("published-where-the-old-bound-fired", 1);
            tally.example(format!(
                "PUBLISHED_WHERE_THE_OLD_BOUND_FIRED {head} B_seen={}",
                describe_seen(&b.seen)
            ));
        }
        if vain {
            vain_sizes += stride;
            tally.add("vain-series", 1);
            tally.add("vain-empty-publishes", a.empty_publishes as u64);
            if a.floor_after > a.floor_before {
                tally.add("vain-series floor-raised", 1);
            }
            tally.add(
                if a.stuck {
                    "vain-series stuck-reported"
                } else {
                    "vain-series no-outlet-reported"
                },
                1,
            );
            tally.add(format!("vain-series final {} holding={}", a.label(), holding > 0), 1);
        }
        if a.fired_first_round() && b.fired_first_round() {
            both_fired_in_a_row += 1;
        } else {
            both_fired_in_a_row = 0;
        }
        let follow_the_way_out = vain || a.stuck || lost_with_the_c_ring || published_where_the_old_bound_fired;
        if follow_the_way_out && !a.published() && remounts_left > 0 {
            remounts_left -= 1;
            if vain && !retried_after_a_small_write {
                retried_after_a_small_write = true;
                let mut again = arm_a.copy();
                let small = small_overwrite_of_the_first_file(&mut again);
                let retried = publish_observed(&mut again, change, InsideTerm::AsCoded);
                tally.add(
                    format!(
                        "vain-series retry-after-one-small-write small={small} series={} final={}",
                        retried.series.min(3),
                        retried.label()
                    ),
                    1,
                );
            }
            let before_the_remount = readings(&arm_a);
            match arm_a.unmount_then_mount() {
                Err(refusal) => tally.add(format!("remount-refused {refusal}"), 1),
                Ok(()) => {
                    let after_the_remount = readings(&arm_a);
                    let d = publish_observed(&mut arm_a, change, InsideTerm::AsCoded);
                    let verdict = if d.published() { "published" } else { "refused" };
                    if vain {
                        tally.add(format!("vain-series remount-retry {verdict}"), 1);
                    }
                    if a.stuck {
                        tally.add(format!("stuck remount-retry {verdict}"), 1);
                        if b.fired_first_round() && d.published() {
                            tally.add("outlet-kept-by-the-inside-term", 1);
                        }
                    }
                    if lost_with_the_c_ring {
                        let round = if a.series > 0 {
                            "outlet-lost-after-a-series"
                        } else {
                            "outlet-lost-first-round"
                        };
                        tally.add(format!("{round} remount-retry {verdict}"), 1);
                    }
                    let mut gained_minus_counted = Vec::new();
                    for (reading_before, reading_after) in before_the_remount.iter().zip(after_the_remount.iter()) {
                        let gained = i64::try_from(reading_after.outside).expect("槽对数")
                            - i64::try_from(reading_before.outside).expect("槽对数");
                        let counted = i64::try_from(reading_before.inside).expect("槽对数");
                        tally.add(
                            format!("remount pairs-gained-minus-inside-counted {}", signed_bucket(gained - counted)),
                            1,
                        );
                        gained_minus_counted.push(gained - counted);
                    }
                    tally.example(format!(
                        "FOLLOWED {head} D={} remount_gained_minus_inside={gained_minus_counted:?} A_seen={} B_seen={}",
                        d.label(),
                        describe_seen(&a.seen),
                        describe_seen(&b.seen)
                    ));
                }
            }
        }
        if both_fired_in_a_row >= BOTH_FIRED_IN_A_ROW_TO_STOP {
            break;
        }
        g += stride;
    }
    tally.add(format!("vain-sizes-per-probe-point {}", bucket(vain_sizes)), 1);
}

/// 一个配置：盘宽、家族、前缀长度都由种子定。
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
