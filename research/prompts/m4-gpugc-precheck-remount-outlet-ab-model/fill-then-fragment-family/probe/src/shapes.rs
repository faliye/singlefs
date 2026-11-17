//! 一个配置（一个种子）：两块内存稀疏盘（单元区在 1007 + 384 槽之外再多 [0, 0]、[256, 256]、[1024, 1024] 或 [256, 1024] 槽，按种子取），
//! 先照「文件大小」一档把池填到第一次被拒（起点历史：建 inode 一次 8 个、逐个整份写 1–2 或 1–8 个数据单元），
//! 再在同一次挂载里走一段打碎的历史（整份重写小文件、小覆盖写、大写、unlink、建小文件），每一次用户改动都经会话的 `publish_user_change` 发
//! （产品那一臂：先算照冻结副本，`N₀` 是产品值）。填满之后单元区里每段都留着活数据、defer 里有成对的槽，全空聚簇段常压到 0：
//! 落得下那一判先拒、槽对那一判的上界够得上需求时，推串之前合取 7 那一判（甲）判出的格比随机历史多。
//! 起点状态（盘的几何加文件大小那一档，共 8 个）每个只建、只填一次（头一个要它的线程建），之后各配置拷那一份（`Pool::copy`）。
//! 每一次改动之前拷一份池；产品那一臂报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 时，在拷贝上各发一次同一个改动：
//!   O 那一臂：先算整个关掉（放不下的盘那一判与推串之前合取 7 那一判都不判出），被拒就推、推满为止——写进了就是先算误拒；
//!   重挂那一臂：正常卸载、再可写挂载（先算照冻结副本）——写进了就是「卸载重挂」这条出路在这一格走得通。
//! 按判出它的那一判分（slot-pairs、formula、mixed、conjunct-before-any-series、both-empty、both-set），甲另按判出时每块盘全空段数的最小值分（0 与大于 0）。
//! 每一串推成的抬 F 记推前推后每块盘的全空聚簇段数（产品那一臂与 O 那一臂分开记）：变多、变少、不变分开数，推前少于 `N₀` 而推完不少于另数。
//! 每一次改动之后另核一遍：拿分配器此刻的记录重建一份分配器（`PoolAllocator::rebuild_from_records`），每块盘「两槽都不罩在未释放记录下」的段外槽对数
//! （不排聚簇段与排这次挂载的聚簇段两种）与全空聚簇段数要与增量维护的相同。
//! 误拒的底：产品那一臂推过一串之后写进的格数。
//! 判别子那一臂（`discriminator.rs`）：产品那一臂判出那一刻核心层只读分配器算的候选判别子，与同一格重挂那一臂写没写进对成四格表；
//! 别的臂与计数照第二族原样。

use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, PoisonError};

use singlefs_core::address::{InodeNumber, SlotNumber};
use singlefs_core::admission::JudgementThatFoundTheDeviceShort;
use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
use singlefs_core::mounted_session::refusal_precheck_oracle_probe as probe;
use singlefs_core::mounted_session::{
    NoSpaceEvenWithEverythingAFloorRaiseCouldRelease, UserChange, UserChangePublished,
    UserChangeRefused,
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
    /// 8 个起点状态各一份填满的池（下标见 [`index_of_the_start_state`]）：头一个要它的线程建池、填满、放进来，之后各配置拷它。
    filled_start_states: Vec<Mutex<Option<Pool>>>,
}

impl Options {
    pub fn new(steps: usize) -> Self {
        Self {
            steps,
            filled_start_states: (0..EXTRA_SLOTS_OF_THE_START_STATES.len() * 2)
                .map(|_| Mutex::new(None))
                .collect(),
        }
    }
}

/// 两块盘在 1007 + 384 槽之外各多几槽：三档等大、一档不等大。
const EXTRA_SLOTS_OF_THE_START_STATES: [[u64; 2]; 4] =
    [[0, 0], [256, 256], [1024, 1024], [256, 1024]];

/// 起点历史与之后建、重写的小文件各取几个数据单元。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FileSizes {
    OneOrTwoDataUnits,
    OneToEightDataUnits,
}

impl FileSizes {
    fn name(self) -> &'static str {
        match self {
            FileSizes::OneOrTwoDataUnits => "one-or-two",
            FileSizes::OneToEightDataUnits => "one-to-eight",
        }
    }
    fn choices(self) -> &'static [usize] {
        match self {
            FileSizes::OneOrTwoDataUnits => &[1, 2],
            FileSizes::OneToEightDataUnits => &[1, 2, 3, 4, 6, 8],
        }
    }
    fn seed_part(self) -> u64 {
        match self {
            FileSizes::OneOrTwoDataUnits => 1,
            FileSizes::OneToEightDataUnits => 2,
        }
    }
}

/// 起点历史至多几轮（一轮建 8 个 inode、逐个整份写）：单元区至多 2415 槽，1 个数据单元的文件也填不过 1300 个，轮数是上界不是取样。
const FILL_ROUNDS_AT_MOST: usize = 400;
const INODES_PER_FILL_ROUND: u64 = 8;

fn write_of(inode: u64, content: &[u8], write_time_seconds: u64) -> UserChange<'_> {
    UserChange::SequentialWriteOfAnInode {
        inode: InodeNumber(inode),
        content,
        write_time_seconds,
    }
}

/// 起点历史：建 inode 一次 8 个、逐个整份写（大小照 `sizes` 按起点状态定的种子取），到第一次被拒为止。交回写成了几个文件。
fn fill(pool: &mut Pool, extra: [u64; 2], sizes: FileSizes) -> u64 {
    let payload = data_unit_payload_capacity();
    let mut random = Random(0x4649_4C4C ^ (extra[0] << 20) ^ (extra[1] << 8) ^ sizes.seed_part());
    let mut files_written = 0_u64;
    for round in 0..FILL_ROUNDS_AT_MOST {
        let Some(first) = pool.inode_number_watermark() else {
            break;
        };
        pool.write_time_seconds += 1;
        let created_at = pool.write_time_seconds;
        let created = pool.publish(UserChange::NewInodes {
            count: INODES_PER_FILL_ROUND,
            write_time_seconds: created_at,
        });
        if created.is_err() {
            break;
        }
        let Some(past_the_last) = pool.inode_number_watermark() else {
            break;
        };
        let mut refused = false;
        for inode in first..past_the_last {
            pool.files.insert(inode, 0);
            let units = random.pick(sizes.choices());
            pool.write_time_seconds += 1;
            let written_at = pool.write_time_seconds;
            let content = content_of(units * payload, written_at);
            if pool.publish(write_of(inode, &content, written_at)).is_ok() {
                pool.files.insert(inode, units as u64);
                files_written += 1;
            } else {
                refused = true;
                break;
            }
        }
        if refused {
            break;
        }
        if round % 25 == 24 {
            eprintln!(
                "PROGRESS fill extra={extra:?} sizes={} rounds={} files={files_written}",
                sizes.name(),
                round + 1
            );
        }
    }
    files_written
}

/// 起点状态在 [`Options`] 里的下标：盘的几何那一档乘 2 加文件大小那一档。
fn index_of_the_start_state(extra_index: usize, sizes: FileSizes) -> usize {
    extra_index * 2
        + match sizes {
            FileSizes::OneOrTwoDataUnits => 0,
            FileSizes::OneToEightDataUnits => 1,
        }
}

/// 这个起点状态的一份拷贝：头一次有人要它时建池、填满、留一份（建的时候别的线程等着），之后都拷留下的那一份。
fn copy_of_the_start_state(
    options: &Options,
    extra_index: usize,
    sizes: FileSizes,
    tally: &mut Tally,
) -> Result<Pool, String> {
    let extra = EXTRA_SLOTS_OF_THE_START_STATES[extra_index];
    let mut filled = options.filled_start_states[index_of_the_start_state(extra_index, sizes)]
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if filled.is_none() {
        let mut pool = Pool::start(extra)?;
        let files = fill(&mut pool, extra, sizes);
        tally.add("fills", 1);
        tally.add(
            format!("fill files extra={extra:?} sizes={}", sizes.name()),
            files,
        );
        tally.add(
            format!(
                "fill empty-segments extra={extra:?} sizes={} {:?}",
                sizes.name(),
                probe::empty_cluster_segments_of(&pool.session().allocator)
            ),
            1,
        );
        *filled = Some(pool);
    }
    Ok(filled.as_ref().expect("上面刚填好").copy())
}

/// 一次发布的结局，连同这次调用里探针记下的：推成的每一串前后的全空段数、推串之前合取 7 那一判每一次判没判出。
struct Observed {
    result: Result<UserChangePublished, UserChangeRefused>,
    series: Vec<probe::EmptyClusterSegmentsAroundOneSeries>,
    conjunct_before_any_series: Vec<bool>,
}

fn publish_observed(
    pool: &mut Pool,
    change: UserChange<'_>,
    precheck_switched_off: bool,
) -> Observed {
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
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => {
            "NoSpaceAfterRaisingTheFloor".to_string()
        }
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
            if all_by(
                JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
            ) {
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
        if short_before && !short_after {
            tally.add(format!("series arm={arm} crossed-the-floor"), 1);
        }
    }
}

/// 拿分配器此刻的记录重建一份（与重开时同一个函数），比每块盘两槽都不罩在未释放记录下的段外槽对数（不排聚簇段、排这次挂载的聚簇段）
/// 与全空聚簇段数：不同就交回头一块对不上的盘。
fn counts_rebuilt_from_the_records_differ(allocator: &PoolAllocator) -> Option<String> {
    let device_maps: Vec<DeviceFreeMap> = allocator
        .devices
        .iter()
        .map(|device_map| {
            DeviceFreeMap::with_unit_area_start(
                device_map.device,
                device_map.device_size_in_bytes(),
                device_map.unit_area_start(),
            )
        })
        .collect();
    let rebuilt = PoolAllocator::rebuild_from_records(device_maps, allocator.records().to_vec());
    let no_cluster_segments: BTreeSet<SlotNumber> = BTreeSet::new();
    let cluster_segments = allocator.cluster_segments();
    for (live, again) in allocator.devices.iter().zip(&rebuilt.devices) {
        let live_pairs = (
            live.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                &no_cluster_segments,
            ),
            live.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                cluster_segments,
            ),
            live.empty_segments(),
        );
        let rebuilt_pairs = (
            again.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                &no_cluster_segments,
            ),
            again.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                cluster_segments,
            ),
            again.empty_segments(),
        );
        if live_pairs != rebuilt_pairs {
            return Some(format!(
                "device={:?} live={live_pairs:?} rebuilt={rebuilt_pairs:?}",
                live.device
            ));
        }
    }
    None
}

/// 核一次重建（[`counts_rebuilt_from_the_records_differ`]）；这个配置里核的时候 panic 过一次就不再核。
fn check_the_rebuild(pool: &Pool, tally: &mut Tally, context: &str, rebuild_checks_on: &mut bool) {
    if !*rebuild_checks_on {
        return;
    }
    let allocator = &pool.session().allocator;
    match catch_unwind(AssertUnwindSafe(|| {
        counts_rebuilt_from_the_records_differ(allocator)
    })) {
        Ok(None) => tally.add("rebuild checked", 1),
        Ok(Some(difference)) => {
            tally.add("rebuild checked", 1);
            tally.add("rebuild differ", 1);
            tally.example(format!("REBUILD_DIFFERS {context} {difference}"));
        }
        Err(_) => {
            tally.add("rebuild panicked", 1);
            tally.example(format!("REBUILD_PANICKED {context}"));
            *rebuild_checks_on = false;
        }
    }
}

/// 在产品那一臂发一次改动、核一次重建；判出放不下时在拷贝上跑 O 那一臂与重挂那一臂。交回产品那一臂写没写进。
fn observe_one(
    pool: &mut Pool,
    change: UserChange<'_>,
    kind: &str,
    tally: &mut Tally,
    context: &str,
    rebuild_checks_on: &mut bool,
) -> bool {
    let before = pool.copy();
    let floor = empty_cluster_segments_floor(pool);
    let empty_before = probe::empty_cluster_segments_of(&pool.session().allocator);
    discriminator::before_the_product_publish();
    let product = publish_observed(pool, change, false);
    let discriminators_at_the_refusal = discriminator::after_the_product_publish();
    tally.add("changes", 1);
    check_the_rebuild(pool, tally, context, rebuild_checks_on);
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
    tally.add(
        format!("changes kind={kind} refused={}", refusal_kind(refusal)),
        1,
    );
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = refusal
    else {
        return false;
    };
    let trigger = trigger_of(no_space);
    tally.add("fired", 1);
    tally.add(format!("fired by={trigger}"), 1);
    tally.add(format!("fired by={trigger} kind={kind}"), 1);
    let fewest_empty_segments = empty_before.iter().copied().min().unwrap_or(0);
    tally.add(
        format!(
            "fired by={trigger} min-empty={}",
            if fewest_empty_segments == 0 {
                "0"
            } else {
                "above-0"
            }
        ),
        1,
    );
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

pub fn run_configuration(seed: u64, options: &Options) -> Tally {
    let mut tally = Tally {
        configurations: 1,
        ..Tally::default()
    };
    let start_states = EXTRA_SLOTS_OF_THE_START_STATES.len() as u64;
    let extra_index = usize::try_from(seed % start_states).expect("下标");
    let extra = EXTRA_SLOTS_OF_THE_START_STATES[extra_index];
    let sizes = if (seed / start_states) % 2 == 0 {
        FileSizes::OneOrTwoDataUnits
    } else {
        FileSizes::OneToEightDataUnits
    };
    let mut pool = match copy_of_the_start_state(options, extra_index, sizes, &mut tally) {
        Ok(pool) => pool,
        Err(error) => {
            tally.add(format!("start-failed {error}"), 1);
            return tally;
        }
    };
    tally.add(
        format!("configurations extra={extra:?} sizes={}", sizes.name()),
        1,
    );
    let mut random = Random(seed ^ 0x4652_4147_4D45_0002);
    let payload = data_unit_payload_capacity();
    let mut rebuild_checks_on = true;
    for step in 0..options.steps {
        tally.add("steps", 1);
        if probe::empty_cluster_segments_of(&pool.session().allocator)
            .iter()
            .any(|count| *count == 0)
        {
            tally.add("steps empty-segments=zero-on-some-device", 1);
        }
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        let context = format!(
            "seed={seed} extra={extra:?} sizes={} step={step}",
            sizes.name()
        );
        let written: Vec<u64> = pool
            .files
            .iter()
            .filter(|(_, units)| **units > 0)
            .map(|(inode, _)| *inode)
            .collect();
        match random.below(100) {
            0..=34 => {
                if written.is_empty() {
                    continue;
                }
                let inode = random.pick(&written);
                let units = random.pick(sizes.choices());
                let content = content_of(units * payload, write_time_seconds);
                if observe_one(
                    &mut pool,
                    write_of(inode, &content, write_time_seconds),
                    "rewrite",
                    &mut tally,
                    &context,
                    &mut rebuild_checks_on,
                ) {
                    pool.files.insert(inode, units as u64);
                }
            }
            35..=44 => {
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
                    &mut rebuild_checks_on,
                );
            }
            45..=59 => {
                if written.is_empty() {
                    continue;
                }
                let inode = random.pick(&written);
                let units = random.pick(&[8_usize, 16, 32, 64]);
                let content = content_of(units * payload, write_time_seconds);
                if observe_one(
                    &mut pool,
                    write_of(inode, &content, write_time_seconds),
                    "big-write",
                    &mut tally,
                    &context,
                    &mut rebuild_checks_on,
                ) {
                    pool.files.insert(inode, units as u64);
                }
            }
            60..=79 => {
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
                    &mut rebuild_checks_on,
                );
                // 被拒时意图可能已发成：这个文件之后一概不碰。
                pool.files.remove(&inode);
            }
            _ => {
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
                    &mut rebuild_checks_on,
                );
                if !created {
                    continue;
                }
                pool.files.insert(first, 0);
                let units = random.pick(sizes.choices());
                let content = content_of(units * payload, write_time_seconds);
                if observe_one(
                    &mut pool,
                    write_of(first, &content, write_time_seconds),
                    "create-write",
                    &mut tally,
                    &context,
                    &mut rebuild_checks_on,
                ) {
                    pool.files.insert(first, units as u64);
                }
            }
        }
    }
    tally
}
