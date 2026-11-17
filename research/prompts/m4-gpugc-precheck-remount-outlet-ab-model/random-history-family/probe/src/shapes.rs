//! 一个配置（一个种子）：两块一样大的内存稀疏盘（单元区在 1007 + 384 槽之外再多 0、64、256 或 1024 槽，按种子取），
//! 同一次挂载里走一段随机历史（建文件整份写、小覆盖写、整份重写、大写、unlink），每一次用户改动都经会话的 `publish_user_change` 发
//! （产品那一臂：先算照冻结副本）。每一次改动之前拷一份池；产品那一臂报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`
//! （先算判出放不下、这一串不推）时，在拷贝上各发一次同一个改动：
//!   O 那一臂：先算整个关掉（放不下的盘那一判与推串之前合取 7 那一判都不判出），被拒就推、推满为止——写进了就是先算误拒（推了写得进）；
//!   重挂那一臂：正常卸载、再可写挂载（先算照冻结副本）——写进了就是「卸载重挂」这条出路在这一格走得通。
//! 按判出它的那一判分：放不下的盘全短在段外槽对那一判（slot-pairs）、全短在式子那一判（formula）、两样都有（mixed）、
//! 推串之前合取 7 那一判（conjunct-before-any-series）；两个成员同时空（both-empty）或同时有（both-set）是类型没挡的两形，照数。
//! 每一串推成的抬 F 记推前推后每块盘的全空聚簇段数（产品那一臂与 O 那一臂分开记）：推完有一块盘的数变多、
//! 或推前有一块盘少于 `N₀` 而推完每块盘都不少于，分开数。
//! 误拒的底：产品那一臂推过一串之后写进的格数（先算一律判放不下的话这些格全是误拒；这个数是 0 就说明这一族测不出误拒）。
//! 判别子那一臂（`discriminator.rs`）：产品那一臂判出那一刻核心层只读分配器算的候选判别子，与同一格重挂那一臂写没写进对成四格表；
//! 别的臂与计数照第一族原样。

use singlefs_core::address::InodeNumber;
use singlefs_core::admission::JudgementThatFoundTheDeviceShort;
use singlefs_core::mounted_session::refusal_precheck_oracle_probe as probe;
use singlefs_core::mounted_session::{
    NoSpaceEvenWithEverythingAFloorRaiseCouldRelease, UserChange, UserChangePublished,
    UserChangeRefused, WideProtectionAtTheCeiling,
};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::data_unit_payload_capacity;

use crate::discriminator;
use crate::pool::{content_of, leading_text, Pool};
use crate::tally::Tally;

/// splitmix64：每个配置一个种子，结局只由种子定。
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
    fn pick<T: Copy>(&mut self, choices: &[T]) -> T {
        choices[usize::try_from(self.below(choices.len() as u64)).expect("下标")]
    }
}

pub struct Options {
    pub steps: usize,
}

/// 一次发布的结局，连同这次调用里探针记下的：推成的每一串前后的全空段数、推串之前合取 7 那一判每一次判没判出。
struct Observed {
    result: Result<UserChangePublished, UserChangeRefused>,
    series: Vec<probe::EmptyClusterSegmentsAroundOneSeries>,
    conjunct_before_any_series: Vec<bool>,
}

fn publish_observed(pool: &mut Pool, change: UserChange<'_>, precheck_switched_off: bool) -> Observed {
    probe::switch_off_the_precheck(precheck_switched_off);
    let _ = probe::take_series();
    let _ = probe::take_conjunct_before_any_series();
    let result = pool.publish(change);
    let series = probe::take_series();
    let conjunct_before_any_series = probe::take_conjunct_before_any_series();
    probe::switch_off_the_precheck(false);
    Observed {
        result,
        series,
        conjunct_before_any_series,
    }
}

#[allow(
    unreachable_patterns,
    reason = "探针只认几个成员名；核心层加了成员这里照样编得过、落到 other"
)]
fn refusal_kind(refusal: &UserChangeRefused) -> String {
    match refusal {
        UserChangeRefused::Publish { cause, .. } => {
            format!("Publish:{}", leading_text(&format!("{cause:?}")))
        }
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => "NoSpaceAfterRaisingTheFloor".to_string(),
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => {
            "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease".to_string()
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {
            "FloorRaiseFailedWhilePushingForSpace".to_string()
        }
        UserChangeRefused::NoFileVersionToChange { .. } => "NoFileVersionToChange".to_string(),
        other => leading_text(&format!("{other:?}")),
    }
}

#[allow(
    unreachable_patterns,
    reason = "探针只认带推串的成员；核心层加了成员这里照样编得过、当没推"
)]
fn series_pushed_in(refusal: &UserChangeRefused) -> usize {
    match refusal {
        UserChangeRefused::NoFileVersionToChange { floor_raises, .. }
        | UserChangeRefused::Publish { floor_raises, .. } => floor_raises.len(),
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused) => refused.floor_raises.len(),
        UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(refused) => {
            refused.floor_raises.len()
        }
        UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused) => {
            refused.floor_raises.len()
        }
        _ => 0,
    }
}

/// 判出放不下的是哪一判（模块文档里的六个名字）。
fn trigger_of(no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease) -> &'static str {
    match (
        no_space.devices_short.is_empty(),
        no_space.empty_cluster_segments_below_the_floor.is_some(),
    ) {
        (false, false) => {
            let all_by = |judgement: JudgementThatFoundTheDeviceShort| {
                no_space
                    .devices_short
                    .iter()
                    .all(|short| short.judged_short_by == judgement)
            };
            if all_by(JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits) {
                "slot-pairs"
            } else if all_by(JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand) {
                "formula"
            } else {
                "mixed"
            }
        }
        (true, true) => "conjunct-before-any-series",
        (true, false) => "both-empty",
        (false, true) => "both-set",
    }
}

fn empty_cluster_segments_floor(pool: &Pool) -> u64 {
    pool.session()
        .allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments()
}

fn tally_series(
    tally: &mut Tally,
    arm: &str,
    series: &[probe::EmptyClusterSegmentsAroundOneSeries],
    floor: u64,
) {
    for (before, after) in series {
        tally.add(format!("series arm={arm}"), 1);
        let raised = before.iter().zip(after).any(|(was, is)| is > was);
        let lowered = before.iter().zip(after).any(|(was, is)| is < was);
        let direction = match (raised, lowered) {
            (true, false) => "raised",
            (false, true) => "lowered",
            (true, true) => "mixed",
            (false, false) => "same",
        };
        tally.add(format!("series arm={arm} empty-segments={direction}"), 1);
        let short_before = before.iter().any(|was| *was < floor);
        let short_after = after.iter().any(|is| *is < floor);
        if short_before {
            tally.add(format!("series arm={arm} short-before"), 1);
            tally.add(format!("series arm={arm} short-before empty-segments={direction}"), 1);
        }
        if short_before && !short_after {
            tally.add(format!("series arm={arm} crossed-the-floor"), 1);
        }
    }
}

/// 产品那一臂被合取 7 自己拒（头一次被拒的就是它、不是落得下那一判）时：在拷贝上让会话为它推抬 F（先算整个关掉、合取 7 的被拒当成空间不够），
/// 记推成几串、每一串前后的全空段数、写没写进。写进就是「合取 7 的被拒推了就写得进」。
fn observe_the_conjunct_refusal_pushed_for(
    before: &Pool,
    change: UserChange<'_>,
    floor: u64,
    empty_before: &[u64],
    kind: &str,
    tally: &mut Tally,
    context: &str,
) {
    tally.add("conjunct-first refused", 1);
    if tally.count("conjunct-first refused") == 1 {
        tally.add("configs conjunct-first refused", 1);
    }
    let smallest = empty_before.iter().copied().min().unwrap_or(0);
    tally.add(format!("conjunct-first empty-before-smallest={}", smallest.min(2)), 1);
    let mut forced = before.copy();
    probe::push_for_the_conjunct_refusal(true);
    let forced_observed = publish_observed(&mut forced, change, true);
    probe::push_for_the_conjunct_refusal(false);
    drop(forced);
    tally_series(tally, "forced", &forced_observed.series, floor);
    let (forced_kind, forced_series) = match &forced_observed.result {
        Ok(published) => ("published".to_string(), published.floor_raises.len()),
        Err(refused) => (refusal_kind(refused), series_pushed_in(refused)),
    };
    tally.add(format!("conjunct-first forced-series={}", forced_series.min(3)), 1);
    tally.add(format!("conjunct-first forced={forced_kind}"), 1);
    tally.add(format!("conjunct-first kind={kind} forced={forced_kind}"), 1);
    if forced_kind == "published" {
        tally.example(format!(
            "CONJUNCT_FIRST_FORCED_PUBLISHED {context} kind={kind} empty_before={empty_before:?} forced_series={forced_series}"
        ));
    }
}

/// 槽对那一判的上界照老算法（每个放得开的槽各补一对）数、其余同产品：在拷贝上发同一个改动，交回结局的名字
///（写进、老上界也判出放不下由哪一判判出、推了串之后还是空间不够并带被宽护卡住与否、别的被拒）。
fn observe_the_old_slot_pair_bound(before: &Pool, change: UserChange<'_>, tally: &mut Tally) -> String {
    let mut old = before.copy();
    probe::use_the_old_slot_pair_bound(true);
    let observed = publish_observed(&mut old, change, false);
    probe::use_the_old_slot_pair_bound(false);
    drop(old);
    let outcome = match &observed.result {
        Ok(_) => "published".to_string(),
        Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) => {
            format!("fires-by-{}", trigger_of(no_space))
        }
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => {
            match &refused.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_) => "pushed-then-no-space-wide-protection-stuck".to_string(),
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "pushed-then-no-space-wide-protection-not".to_string(),
            }
        }
        Err(other) => format!("other-{}", refusal_kind(other)),
    };
    tally.add(format!("fired by=slot-pairs old-bound={outcome}"), 1);
    outcome
}

/// 在产品那一臂发一次改动；判出放不下时在拷贝上跑 O 那一臂与重挂那一臂。交回产品那一臂写没写进。
fn observe_one(
    pool: &mut Pool,
    change: UserChange<'_>,
    kind: &str,
    tally: &mut Tally,
    context: &str,
) -> bool {
    let before = pool.copy();
    let floor = empty_cluster_segments_floor(pool);
    let empty_before = probe::empty_cluster_segments_of(&pool.session().allocator);
    discriminator::before_the_product_publish();
    let product = publish_observed(pool, change, false);
    let discriminators_at_the_refusal = discriminator::after_the_product_publish();
    tally.add("changes", 1);
    tally_series(tally, "product", &product.series, floor);
    for found in &product.conjunct_before_any_series {
        tally.add("conjunct-before-any-series judged", 1);
        if *found {
            tally.add("conjunct-before-any-series found=true", 1);
        }
    }
    let refusal = match &product.result {
        Ok(published) => {
            tally.add(format!("changes kind={kind} published"), 1);
            if !published.floor_raises.is_empty() {
                tally.add("base published-after-a-series", 1);
                tally.add(format!("base published-after-a-series kind={kind}"), 1);
            }
            return true;
        }
        Err(refusal) => refusal,
    };
    tally.add(format!("changes kind={kind} refused={}", refusal_kind(refusal)), 1);
    if refusal_kind(refusal) == "Publish:EmptyClusterSegmentsBelowTheFloor" {
        observe_the_conjunct_refusal_pushed_for(
            &before,
            change,
            floor,
            &empty_before,
            kind,
            tally,
            context,
        );
    }
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = refusal
    else {
        return false;
    };
    let trigger = trigger_of(no_space);
    tally.add("fired", 1);
    tally.add(format!("fired by={trigger}"), 1);
    if tally.count(&format!("fired by={trigger}")) == 1 {
        tally.add(format!("configs fired by={trigger}"), 1);
    }
    let isolated_before: u64 = probe::isolated_slots_of(&before.session().allocator)
        .iter()
        .sum();
    let isolated_text = if isolated_before > 0 { "some" } else { "none" };
    tally.add(format!("fired by={trigger} isolated-before={isolated_text}"), 1);
    let count_below_the_floor = empty_before.iter().any(|count| *count < floor);
    let count_below_the_floor_text = if count_below_the_floor { "yes" } else { "no" };
    tally.add(format!("fired by={trigger} count-below-floor={count_below_the_floor_text}"), 1);
    tally.add(format!("fired by={trigger} kind={kind}"), 1);
    let versions_published = !no_space.versions_published.is_empty();
    tally.add(
        format!(
            "fired by={trigger} versions-published={}",
            if versions_published { "yes" } else { "no" }
        ),
        1,
    );
    tally.add(
        format!(
            "fired by={trigger} series-before={}",
            no_space.floor_raises.len().min(3)
        ),
        1,
    );
    let mut oracle = before.copy();
    let oracle_observed = publish_observed(&mut oracle, change, true);
    drop(oracle);
    tally_series(tally, "oracle", &oracle_observed.series, floor);
    let (oracle_kind, oracle_series) = match &oracle_observed.result {
        Ok(published) => ("published".to_string(), published.floor_raises.len()),
        Err(refused) => (refusal_kind(refused), series_pushed_in(refused)),
    };
    if oracle_series > no_space.floor_raises.len() {
        tally.add("oracle pushed-more-series-than-the-product", 1);
    }
    tally.add(format!("fired by={trigger} oracle={oracle_kind}"), 1);
    tally.add(format!("fired by={trigger} oracle-series={}", oracle_series.min(3)), 1);
    let oracle_wide_protection = match &oracle_observed.result {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused)) => {
            match &refused.wide_protection_at_the_ceiling {
                WideProtectionAtTheCeiling::StuckBehindTheWideProtection(_) => "stuck",
                WideProtectionAtTheCeiling::NotWhatStopsTheAdmission => "not",
            }
        }
        _ => "not-applicable",
    };
    tally.add(
        format!("fired by={trigger} oracle-wide-protection={oracle_wide_protection}"),
        1,
    );
    let oracle_series_text: Vec<String> = oracle_observed
        .series
        .iter()
        .map(|(was, is)| format!("{was:?}->{is:?}"))
        .collect();
    if oracle_kind == "published" {
        tally.add(format!("misrefused by={trigger}"), 1);
        tally.add(format!("misrefused by={trigger} kind={kind}"), 1);
        tally.example(format!(
            "MISREFUSED {context} kind={kind} by={trigger} empty_before={empty_before:?} floor={floor} devices_short={:?} below_the_floor={:?} last_refusal={} series_before={} versions_published={} oracle_series={oracle_series} oracle_empty_segments=[{}]",
            no_space.devices_short,
            no_space.empty_cluster_segments_below_the_floor,
            leading_text(&format!("{:?}", no_space.last_refusal)),
            no_space.floor_raises.len(),
            no_space.versions_published.len(),
            oracle_series_text.join(" "),
        ));
    }
    let old_bound_outcome = if trigger == "slot-pairs" {
        observe_the_old_slot_pair_bound(&before, change, tally)
    } else {
        "not-run".to_string()
    };
    let mut remounted = before;
    let mut remount_published = false;
    let mut remount_failed = false;
    match remounted.unmount_then_mount() {
        Err(error) => {
            tally.add(format!("fired by={trigger} remount=failed:{error}"), 1);
            remount_failed = true;
        }
        Ok(()) => {
            let remount_observed = publish_observed(&mut remounted, change, false);
            let remount_kind = match &remount_observed.result {
                Ok(_) => "published".to_string(),
                Err(refused) => refusal_kind(refused),
            };
            tally.add(format!("fired by={trigger} remount={remount_kind}"), 1);
            remount_published = remount_kind == "published";
            if remount_kind == "published" {
                tally.add(format!("remount published by={trigger}"), 1);
                tally.add(
                    format!("remount published by={trigger} oracle-wide-protection={oracle_wide_protection}"),
                    1,
                );
                tally.add(
                    format!("remount published by={trigger} oracle-series={}", oracle_series.min(3)),
                    1,
                );
                tally.add(format!("remount published by={trigger} isolated-before={isolated_text}"), 1);
                tally.add(format!("remount published by={trigger} old-bound={old_bound_outcome}"), 1);
                tally.add(
                    format!("remount published by={trigger} count-below-floor={count_below_the_floor_text}"),
                    1,
                );
                tally.example(format!(
                    "REMOUNT_PUBLISHED {context} kind={kind} by={trigger} empty_before={empty_before:?} devices_short={:?} oracle={oracle_kind}",
                    no_space.devices_short
                ));
            }
        }
    }
    discriminator::tally_the_cell(
        tally,
        trigger,
        discriminators_at_the_refusal,
        remount_published,
        remount_failed,
    );
    false
}

fn write_of(inode: u64, content: &[u8], write_time_seconds: u64) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode: InodeNumber(inode),
        content,
        write_time_seconds,
    }
}

pub fn run_configuration(seed: u64, options: &Options) -> Tally {
    let mut tally = Tally {
        configurations: 1,
        ..Tally::default()
    };
    let mut random = Random(seed ^ 0x4F52_4143_4C45_0001);
    let extra = random.pick(&[0_u64, 64, 256, 1024]);
    let mut pool = match Pool::start([extra, extra]) {
        Ok(pool) => pool,
        Err(error) => {
            tally.add(format!("start-failed {error}"), 1);
            return tally;
        }
    };
    tally.add(format!("configurations extra={extra}"), 1);
    let payload = data_unit_payload_capacity();
    for step in 0..options.steps {
        tally.add("steps", 1);
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let context = format!("seed={seed} extra={extra} step={step}");
        let written: Vec<u64> = pool
            .files
            .iter()
            .filter(|(_, units)| **units > 0)
            .map(|(inode, _)| *inode)
            .collect();
        match random.below(100) {
            0..=24 => {
                let Some(first) = pool.inode_number_watermark() else {
                    continue;
                };
                let created = observe_one(
                    &mut pool,
                    UserChange::NewInodes {
                        count: 1,
                        write_time_seconds,
                    },
                    "create-inode",
                    &mut tally,
                    &context,
                );
                if !created {
                    continue;
                }
                pool.files.insert(first, 0);
                let units = random.pick(&[1_usize, 2, 4, 8, 16, 32]);
                let content = content_of(units * payload, write_time_seconds);
                if observe_one(
                    &mut pool,
                    write_of(first, &content, write_time_seconds),
                    "create-write",
                    &mut tally,
                    &context,
                ) {
                    pool.files.insert(first, units as u64);
                }
            }
            25..=54 => {
                if written.is_empty() {
                    continue;
                }
                let inode = random.pick(&written);
                let length = 1 + usize::try_from(random.below(payload as u64)).expect("长度");
                let content = content_of(length, write_time_seconds);
                let _ = observe_one(
                    &mut pool,
                    UserChange::OverwriteOfAnInode {
                        inode: InodeNumber(inode),
                        content: &content,
                        write_time_seconds,
                    },
                    "small-overwrite",
                    &mut tally,
                    &context,
                );
            }
            55..=69 | 82..=99 => {
                if written.is_empty() {
                    continue;
                }
                let inode = random.pick(&written);
                let (units, kind) = if random.below(100) < 45 {
                    (random.pick(&[1_usize, 2, 4, 8, 16, 32, 64]), "rewrite")
                } else {
                    (random.pick(&[32_usize, 64, 128, 256]), "big-write")
                };
                let content = content_of(units * payload, write_time_seconds);
                if observe_one(
                    &mut pool,
                    write_of(inode, &content, write_time_seconds),
                    kind,
                    &mut tally,
                    &context,
                ) {
                    pool.files.insert(inode, units as u64);
                }
            }
            _ => {
                let candidates: Vec<u64> = pool
                    .files
                    .keys()
                    .copied()
                    .filter(|inode| *inode != FIRST_INODE_NUMBER)
                    .collect();
                if candidates.is_empty() {
                    continue;
                }
                let inode = random.pick(&candidates);
                let _ = observe_one(
                    &mut pool,
                    UserChange::Unlink {
                        inode: InodeNumber(inode),
                        write_time_seconds,
                    },
                    "unlink",
                    &mut tally,
                    &context,
                );
                // 被拒时意图可能已发成：这个文件之后一概不碰。
                pool.files.remove(&inode);
            }
        }
    }
    tally
}
