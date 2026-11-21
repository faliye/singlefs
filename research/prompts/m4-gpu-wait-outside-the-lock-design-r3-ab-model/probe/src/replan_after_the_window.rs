//! 判决「三、改法」第 5 条（回锁照现行那一版重排计划、不作废）的 A/B。
//! 一段历史：起点池拷一份、可写挂载 → 签出活意图（R 定下）→ 取票：现行那一版（票里那一版）、R 里起点在 R 的
//! 未释放记录、对 R 扫一次、照今天的计划函数算这一批（`core-patch` 里的 `ticket_of_the_batch`）→ 前台做 k 次改动、每次都发布（锁外那一段期间）
//! → 回锁，按臂发这一批 → 记下 R 里还剩几条未释放记录 → 会话丢掉 → 池级 checker → 只读挂载读回第一个文件与第二个 inode。
//! 签发分两种（`region_choice`）：
//!   greedy：照今天的代码在 CPU 上推一次整理发布，今天的选段挑 R（`core-patch` 的 `issue_an_intent`）。起点池上它挑中的段里没有第一个文件的数据单元。
//!   batch_moving_units_of_the_first_file：探针先挑 R——含第一个文件数据单元、照今天的计划函数算出的这一批里有第一个文件的数据单元、
//!     按记录活槽最少的整段（`core-patch` 的 `region_whose_batch_moves_units_of_the_first_file`），再经只供测试的选段开关在它上面照今天的代码签发
//!     （`issue_an_intent_on_the_region`）。覆盖写 R 里单元与删文件两种改动走这一种，前台改动的目标才是这一批计划要搬的单元。
//! 四臂各自从起点池重放同一段历史（会话不能拷，重放是确定的；自证核四臂看到的票与前台之后的 txg 逐项相同）：
//!   replan_pruning：重取隔离集，比整条记录与集合，记录变了的单元从票里的计划剔掉，其余在现行那一版上发；剔完一个不剩不发。
//!   drop_when_the_txg_moved：现行那一版的 txg 与票里那一版不等就不发。
//!   recompute_in_the_lock_as_today：在现行那一版上照今天的代码重扫、重算、发（参照）。
//!   replan_without_pruning：阳性对照，票里的计划原样在现行那一版上发，不比记录。
//! 前台改动四种：
//!   overwrite_a_unit_in_the_region：覆盖写票里计划要搬的第一个文件的一个数据单元（按槽升序，k 次各写一个不同的单元，不够就从头轮），k 取 0、1、2、4、8。
//!   write_an_inode_outside_the_region：顺序写第二个 inode（它没有数据落在 R 里；每次写一个单元的内容），k 取 0、1、2、4、8。
//!   new_inode：建一个 inode，k 取 0、1、2、4、8。
//!   unlink_a_file_with_units_in_the_region：删第一个文件（它在票里的计划里有单元）；k = 1 删一次，k = 2 删完再建一个 inode。
//! 汇总每臂一行 `RESULT`（全部改动合计），另每臂每种改动一行 `RESULT … kind=…`。
//! 没造的：隔离（前台改动没有读盘核出对不上的副本，「重取隔离集」那一判每段都走、都判不出隔离）；回退复活（运行库没有回退入口）；快照。
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use singlefs_core::address::InodeNumber;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::probe_replan_after_the_window::{
    issue_an_intent, issue_an_intent_on_the_region, publish_after_the_relock, publish_as_today,
    region_whose_batch_moves_units_of_the_first_file, ticket_of_the_batch, units_of_the_first_file_by_slot,
    ArmOfTheRelock, OutcomeOfTheRelock, TicketOfTheBatch,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::data_unit_payload_capacity;

use crate::judged::{short, unreleased_records_in, violated_invariants};
use crate::pool::{
    copied, parameters, start_pool, whole_file_after_a_read_only_mount, write_one_byte, Devices,
    WRITE_TIME_SECONDS,
};

const SECOND_INODE: u64 = FIRST_INODE_NUMBER + 1;
/// 先跑一段估全量，超过这么多秒就停、不跑全量（40 分钟）。
const ESTIMATE_LIMIT_SECONDS: f64 = 2400.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    OverwriteAUnitInTheRegion,
    WriteAnInodeOutsideTheRegion,
    NewInode,
    UnlinkAFileWithUnitsInTheRegion,
}

/// 签发怎么挑 R（文件头「签发分两种」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegionChoiceOfTheProbe {
    Greedy,
    BatchMovingUnitsOfTheFirstFile,
}

impl Kind {
    const ALL: [Kind; 4] = [
        Kind::OverwriteAUnitInTheRegion,
        Kind::WriteAnInodeOutsideTheRegion,
        Kind::NewInode,
        Kind::UnlinkAFileWithUnitsInTheRegion,
    ];
    fn name(self) -> &'static str {
        match self {
            Kind::OverwriteAUnitInTheRegion => "overwrite_a_unit_in_the_region",
            Kind::WriteAnInodeOutsideTheRegion => "write_an_inode_outside_the_region",
            Kind::NewInode => "new_inode",
            Kind::UnlinkAFileWithUnitsInTheRegion => "unlink_a_file_with_units_in_the_region",
        }
    }
    /// 这一种改动取哪几个 k。
    fn counts(self) -> &'static [usize] {
        match self {
            Kind::OverwriteAUnitInTheRegion | Kind::WriteAnInodeOutsideTheRegion | Kind::NewInode => &[0, 1, 2, 4, 8],
            Kind::UnlinkAFileWithUnitsInTheRegion => &[1, 2],
        }
    }
    fn region_choice(self) -> RegionChoiceOfTheProbe {
        match self {
            Kind::OverwriteAUnitInTheRegion | Kind::UnlinkAFileWithUnitsInTheRegion => {
                RegionChoiceOfTheProbe::BatchMovingUnitsOfTheFirstFile
            }
            Kind::WriteAnInodeOutsideTheRegion | Kind::NewInode => RegionChoiceOfTheProbe::Greedy,
        }
    }
    /// 前台改动的目标要在票里的计划里（自证核 k ≥ 1 的历史 `targets_in_plan` 大于 0）。
    fn targets_the_plan(self) -> bool {
        self.region_choice() == RegionChoiceOfTheProbe::BatchMovingUnitsOfTheFirstFile
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Arm {
    ReplanPruning,
    DropWhenTheTxgMoved,
    RecomputeInTheLockAsToday,
    ReplanWithoutPruning,
}

impl Arm {
    const ALL: [Arm; 4] = [
        Arm::ReplanPruning,
        Arm::DropWhenTheTxgMoved,
        Arm::RecomputeInTheLockAsToday,
        Arm::ReplanWithoutPruning,
    ];
    fn name(self) -> &'static str {
        match self {
            Arm::ReplanPruning => "replan_pruning",
            Arm::DropWhenTheTxgMoved => "drop_when_the_txg_moved",
            Arm::RecomputeInTheLockAsToday => "recompute_in_the_lock_as_today",
            Arm::ReplanWithoutPruning => "replan_without_pruning",
        }
    }
}

/// 前台写下了什么（读回时核）。
#[derive(Default)]
struct Foreground {
    /// 第一个文件：单元号 → 最后一次写到它第一个字节的值。
    last_byte_of_each_unit_of_the_first_file: BTreeMap<usize, u8>,
    /// 第二个 inode 最后一次顺序写的内容。
    last_content_of_the_second_inode: Option<Vec<u8>>,
}

impl Foreground {
    /// 前台写下的字节在这两份读回里都在吗：`n/a` 是这一种改动不写字节（建 inode、删文件、k = 0）。
    fn found_in(&self, first_file: &Result<Vec<u8>, String>, second_inode: &Result<Vec<u8>, String>) -> &'static str {
        if self.last_byte_of_each_unit_of_the_first_file.is_empty() && self.last_content_of_the_second_inode.is_none() {
            return "n/a";
        }
        let first_ok = self.last_byte_of_each_unit_of_the_first_file.iter().all(|(unit, byte)| {
            first_file
                .as_ref()
                .is_ok_and(|bytes| bytes.get(unit * data_unit_payload_capacity()) == Some(byte))
        });
        let second_ok = self
            .last_content_of_the_second_inode
            .as_ref()
            .is_none_or(|content| second_inode.as_ref().is_ok_and(|bytes| bytes == content));
        if first_ok && second_ok {
            "ok"
        } else {
            "missing"
        }
    }
}

/// 票里那一版上第一个文件的数据单元：落在票里计划里的那几个的单元号（按计划的次序，即按槽升序），与落在 R 里的个数。
fn units_of_the_first_file_in_the_ticket(ticket: &TicketOfTheBatch) -> (Vec<usize>, usize) {
    let region = ticket.intent.region;
    let units_by_slot = units_of_the_first_file_by_slot(&ticket.version);
    let in_the_plan: Vec<usize> = ticket
        .planned
        .iter()
        .filter_map(|relocation| units_by_slot.get(&relocation.source.slot))
        .map(|index| usize::try_from(*index).expect("单元号"))
        .collect();
    let in_the_region = units_by_slot
        .keys()
        .filter(|slot| slot.0 >= region.start_slot().0 && slot.0 < region.end_slot().0)
        .count();
    (in_the_plan, in_the_region)
}

fn new_inode(session: &mut MountedSession, devices: &mut Devices) -> Result<(), String> {
    session
        .publish_user_change(
            devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: WRITE_TIME_SECONDS + 9,
            },
        )
        .map(|_published| ())
        .map_err(|refused| short(&format!("{refused:?}"), 160))
}

fn foreground(
    kind: Kind,
    count: usize,
    targets_in_the_plan: &[usize],
    session: &mut MountedSession,
    devices: &mut Devices,
) -> Result<Foreground, String> {
    let mut written = Foreground::default();
    for index in 0..count {
        let byte = 0xB0_u8.wrapping_add(u8::try_from(index % 64).expect("小于 64"));
        match kind {
            Kind::OverwriteAUnitInTheRegion => {
                let Some(unit) = targets_in_the_plan.get(index % targets_in_the_plan.len().max(1)).copied() else {
                    return Err("no_unit_of_the_first_file_in_the_plan".to_string());
                };
                write_one_byte(session, devices, unit, byte)?;
                written.last_byte_of_each_unit_of_the_first_file.insert(unit, byte);
            }
            Kind::WriteAnInodeOutsideTheRegion => {
                let content = vec![byte; data_unit_payload_capacity()];
                session
                    .publish_user_change(
                        devices,
                        UserChange::SequentialWriteOfAnInode {
                            inode: InodeNumber(SECOND_INODE),
                            content: &content,
                            write_time_seconds: WRITE_TIME_SECONDS + 9,
                        },
                    )
                    .map_err(|refused| short(&format!("{refused:?}"), 160))?;
                written.last_content_of_the_second_inode = Some(content);
            }
            Kind::NewInode => new_inode(session, devices)?,
            Kind::UnlinkAFileWithUnitsInTheRegion => {
                if index == 0 {
                    if targets_in_the_plan.is_empty() {
                        return Err("first_file_without_a_unit_in_the_plan".to_string());
                    }
                    session
                        .publish_user_change(
                            devices,
                            UserChange::Unlink {
                                inode: InodeNumber(FIRST_INODE_NUMBER),
                                write_time_seconds: WRITE_TIME_SECONDS + 9,
                            },
                        )
                        .map(|_published| ())
                        .map_err(|refused| format!("unlink:{}", short(&format!("{refused:?}"), 160)))?;
                } else {
                    new_inode(session, devices).map_err(|refused| format!("new_inode_after_the_unlink:{refused}"))?;
                }
            }
        }
    }
    Ok(written)
}

struct Row {
    line: String,
    /// 票与前台之后的现行那一版：同一格四臂要逐项相同（重放是确定的）。
    fingerprint: String,
    outcome: OutcomeOfTheRelock,
    violated: Vec<&'static str>,
    readback_ok: bool,
    foreground_in_the_reference: &'static str,
    targets_in_the_plan: usize,
}

fn one_history(start: &Devices, kind: Kind, count: usize, arm: Arm) -> Result<Row, String> {
    let parameters = parameters();
    let mut devices = copied(start);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| short(&format!("mount:{error:?}"), 120))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let (issued, region_choice_text) = match kind.region_choice() {
        RegionChoiceOfTheProbe::Greedy => (
            issue_an_intent(&mut session, devices.as_mut_slice()).map_err(|failure| format!("issue:{failure}"))?,
            "region_choice=greedy".to_string(),
        ),
        RegionChoiceOfTheProbe::BatchMovingUnitsOfTheFirstFile => {
            let chosen = region_whose_batch_moves_units_of_the_first_file(&mut session, devices.as_slice())
                .map_err(|failure| format!("choose_region:{failure}"))?;
            let issued = issue_an_intent_on_the_region(&mut session, devices.as_mut_slice(), chosen.region)
                .map_err(|failure| format!("issue_on_the_chosen_region:{failure}"))?;
            (
                issued,
                format!(
                    "region_choice=batch_moving_units_of_the_first_file region_live_slots={} region_planned_units_when_chosen={} region_units_of_the_first_file_planned_when_chosen={} segments_judged={}",
                    chosen.live_slots, chosen.planned_units, chosen.units_of_the_first_file_planned, chosen.segments_judged
                ),
            )
        }
    };
    let ticket = ticket_of_the_batch(&mut session, devices.as_slice()).map_err(|failure| format!("ticket:{failure}"))?;
    if ticket.intent != issued {
        return Err("ticket_intent_differs_from_the_issued_one".to_string());
    }
    let region = ticket.intent.region;
    let (targets_in_the_plan, targets_in_the_region) = units_of_the_first_file_in_the_ticket(&ticket);
    let written = foreground(kind, count, &targets_in_the_plan, &mut session, &mut devices)
        .map_err(|refused| format!("foreground_refused:{refused}"))?;
    let current_txg = session.current.root().checkpoint_txg.0;
    let first_file_before = whole_file_after_a_read_only_mount(&devices, FIRST_INODE_NUMBER);
    let second_inode_before = whole_file_after_a_read_only_mount(&devices, SECOND_INODE);
    let foreground_in_the_reference = written.found_in(&first_file_before, &second_inode_before);
    let outcome = match arm {
        Arm::ReplanPruning => publish_after_the_relock(&mut session, devices.as_mut_slice(), &ticket, ArmOfTheRelock::ReplanPruning),
        Arm::DropWhenTheTxgMoved => {
            publish_after_the_relock(&mut session, devices.as_mut_slice(), &ticket, ArmOfTheRelock::DropWhenTheTxgMoved)
        }
        Arm::RecomputeInTheLockAsToday => publish_as_today(&mut session, devices.as_mut_slice()),
        Arm::ReplanWithoutPruning => {
            publish_after_the_relock(&mut session, devices.as_mut_slice(), &ticket, ArmOfTheRelock::ReplanWithoutPruning)
        }
    };
    let records_left_in_the_region = session
        .current
        .version_of_trees()
        .map_or_else(|| "no_trees".to_string(), |version| unreleased_records_in(version, region).len().to_string());
    let txg_after = session.current.root().checkpoint_txg.0;
    drop(session);
    let violated = violated_invariants(&devices);
    let first_file_after = whole_file_after_a_read_only_mount(&devices, FIRST_INODE_NUMBER);
    let second_inode_after = whole_file_after_a_read_only_mount(&devices, SECOND_INODE);
    let foreground_after = written.found_in(&first_file_after, &second_inode_after);
    let first_file_same = first_file_after == first_file_before;
    let second_inode_same = second_inode_after == second_inode_before;
    let readback_ok = first_file_same && second_inode_same && foreground_after != "missing";
    let fingerprint = format!(
        "{} ticket_txg={} region_device={} region_start={} ticket_kind={} ticket_planned_units={} ticket_first_planned_slot={} ticket_records_in_region={} targets_in_region={} targets_in_plan={} current_txg_before_relock={}",
        region_choice_text,
        ticket.version.root.checkpoint_txg.0,
        region.device().0,
        region.start_slot().0,
        ticket.kind_name,
        ticket.planned.len(),
        ticket.planned.first().map_or_else(|| "none".to_string(), |planned| planned.source.slot.0.to_string()),
        ticket.records_in_the_region.len(),
        targets_in_the_region,
        targets_in_the_plan.len(),
        current_txg,
    );
    let line = format!(
        "HISTORY name=replan_after_the_window arm={} kind={} k={} {} published={} publish_kind={} not_published_because=[{}] refused={} units_moved={} units_pruned={} changed_records={} record_sets_equal={} quarantine_in_region_after_retaking={} skipped_as_rewritten_anyway={} txg_after={} checker_red={} violated=[{}] readback={} first_file={} second_inode={} foreground_bytes_after={} foreground_bytes_in_reference={} records_left_in_region={}",
        arm.name(),
        kind.name(),
        count,
        fingerprint,
        outcome.published,
        if outcome.kind_name.is_empty() { "none" } else { outcome.kind_name.as_str() },
        outcome.not_published_because,
        if outcome.refusal_member.is_empty() { "none" } else { outcome.refusal_member.as_str() },
        outcome.units_moved,
        outcome.units_pruned,
        outcome.changed_records,
        outcome.record_sets_equal,
        outcome.quarantine_in_the_region_after_retaking,
        outcome.units_skipped_as_rewritten_anyway,
        txg_after,
        !violated.is_empty(),
        violated.join(","),
        if readback_ok { "ok" } else { "bad" },
        if first_file_same { "same" } else { "differs" },
        if second_inode_same { "same" } else { "differs" },
        foreground_after,
        foreground_in_the_reference,
        records_left_in_the_region,
    );
    Ok(Row {
        line,
        fingerprint,
        outcome,
        violated,
        readback_ok,
        foreground_in_the_reference,
        targets_in_the_plan: targets_in_the_plan.len(),
    })
}

#[derive(Default)]
struct Tally {
    histories: u64,
    published: u64,
    batches_with_units_moved: u64,
    units_moved: u64,
    units_pruned: u64,
    refused_by_the_publish_path: u64,
    checker_red_histories: u64,
    readback_bad_histories: u64,
    setup_failed: u64,
    panicked: u64,
}

impl Tally {
    fn count_the_row(&mut self, row: &Row) {
        self.histories += 1;
        self.published += u64::from(row.outcome.published);
        self.batches_with_units_moved += u64::from(row.outcome.published && row.outcome.units_moved > 0);
        self.units_moved += u64::try_from(row.outcome.units_moved).expect("单元数");
        self.units_pruned += u64::try_from(row.outcome.units_pruned).expect("单元数");
        self.refused_by_the_publish_path += u64::from(!row.outcome.refusal_member.is_empty());
        self.checker_red_histories += u64::from(!row.violated.is_empty());
        self.readback_bad_histories += u64::from(!row.readback_ok);
    }
    fn count_the_setup_failure(&mut self) {
        self.histories += 1;
        self.setup_failed += 1;
    }
    fn count_the_panic(&mut self) {
        self.histories += 1;
        self.panicked += 1;
    }
    fn fields(&self) -> String {
        format!(
            "histories={} published={} batches_with_units_moved={} units_moved={} units_pruned={} refused_by_the_publish_path={} checker_red_histories={} readback_bad_histories={} setup_failed={} panicked={}",
            self.histories,
            self.published,
            self.batches_with_units_moved,
            self.units_moved,
            self.units_pruned,
            self.refused_by_the_publish_path,
            self.checker_red_histories,
            self.readback_bad_histories,
            self.setup_failed,
            self.panicked
        )
    }
}

pub fn run() {
    let start = start_pool();
    let baseline = violated_invariants(&start);
    println!("BASELINE name=replan_after_the_window start_pool_violated=[{}]", baseline.join(","));
    let histories_in_total: usize = Kind::ALL.iter().map(|kind| kind.counts().len()).sum::<usize>() * Arm::ALL.len();
    let mut tallies: BTreeMap<Arm, Tally> = Arm::ALL.iter().map(|arm| (*arm, Tally::default())).collect();
    let mut tallies_by_kind: BTreeMap<(Arm, Kind), Tally> = BTreeMap::new();
    let mut rows: BTreeMap<(Kind, usize, Arm), Row> = BTreeMap::new();
    let mut histories_run = 0_usize;
    let started = Instant::now();
    for kind in Kind::ALL {
        for count in kind.counts().iter().copied() {
            for arm in Arm::ALL {
                let result = catch_unwind(AssertUnwindSafe(|| one_history(&start, kind, count, arm)));
                histories_run += 1;
                let tally = tallies.get_mut(&arm).expect("每臂一份");
                let tally_of_the_kind = tallies_by_kind.entry((arm, kind)).or_default();
                match result {
                    Ok(Ok(row)) => {
                        println!("{}", row.line);
                        tally.count_the_row(&row);
                        tally_of_the_kind.count_the_row(&row);
                        rows.insert((kind, count, arm), row);
                    }
                    Ok(Err(failure)) => {
                        tally.count_the_setup_failure();
                        tally_of_the_kind.count_the_setup_failure();
                        println!(
                            "HISTORY name=replan_after_the_window arm={} kind={} k={} setup_failed=[{}]",
                            arm.name(),
                            kind.name(),
                            count,
                            short(&failure, 200)
                        );
                    }
                    Err(_payload) => {
                        tally.count_the_panic();
                        tally_of_the_kind.count_the_panic();
                        println!(
                            "HISTORY name=replan_after_the_window arm={} kind={} k={} panicked=1",
                            arm.name(),
                            kind.name(),
                            count
                        );
                    }
                }
                if histories_run == 1 {
                    let first_seconds = started.elapsed().as_secs_f64();
                    let estimated_total_seconds = first_seconds * f64::from(u32::try_from(histories_in_total).expect("段数"));
                    println!(
                        "ESTIMATE name=replan_after_the_window histories={histories_in_total} first_history_seconds={first_seconds:.3} estimated_total_seconds={estimated_total_seconds:.1} limit_seconds={ESTIMATE_LIMIT_SECONDS}"
                    );
                    if estimated_total_seconds > ESTIMATE_LIMIT_SECONDS {
                        println!("  ✗ 估的全量超过 40 分钟，不跑全量：缩 k 的取样再跑");
                        std::process::exit(3);
                    }
                }
            }
        }
    }
    for (arm, tally) in &tallies {
        println!("RESULT name=replan_after_the_window arm={} {}", arm.name(), tally.fields());
    }
    for ((arm, kind), tally) in &tallies_by_kind {
        println!("RESULT name=replan_after_the_window arm={} kind={} {}", arm.name(), kind.name(), tally.fields());
    }
    println!("WALL_SECONDS name=replan_after_the_window seconds={:.1}", started.elapsed().as_secs_f64());
    let mut checked = 0_u64;
    let mut failures: Vec<String> = Vec::new();
    checked += 1;
    if !baseline.is_empty() {
        failures.push(format!("起点池本身就有红项 [{}]：读到的红不全是这一批造的", baseline.join(",")));
    }
    for kind in Kind::ALL {
        for count in kind.counts().iter().copied() {
            checked += 1;
            let cell: Vec<(Arm, &Row)> = Arm::ALL
                .iter()
                .filter_map(|arm| rows.get(&(kind, count, *arm)).map(|row| (*arm, row)))
                .collect();
            if cell.len() != Arm::ALL.len() {
                failures.push(format!("kind={} k={} 四臂只有 {} 行（别的 setup_failed 或 panicked）", kind.name(), count, cell.len()));
            } else if cell.iter().any(|(_, row)| row.fingerprint != cell[0].1.fingerprint) {
                failures.push(format!("kind={} k={} 四臂的票或前台之后的 txg 不同：重放不确定，臂之间不可比", kind.name(), count));
            }
            checked += 1;
            if cell.iter().any(|(_, row)| row.foreground_in_the_reference == "missing") {
                failures.push(format!("kind={} k={} 回锁之前的读回里就没有前台写下的字节：探针的前台改动或读回写错了", kind.name(), count));
            }
            if kind.targets_the_plan() && count >= 1 {
                checked += 1;
                if cell.len() != Arm::ALL.len() || cell.iter().any(|(_, row)| row.targets_in_the_plan == 0) {
                    failures.push(format!(
                        "kind={} k={} 不是四臂都有 targets_in_plan>0 的行：前台改动的目标不在票里的计划里，这一种改动没造出来",
                        kind.name(),
                        count
                    ));
                }
            }
        }
        if !kind.counts().contains(&0) {
            continue;
        }
        // 自证一：k = 0 时回锁重排与今天锁内重算搬的单元数相同（而且搬了）。
        checked += 1;
        match (rows.get(&(kind, 0, Arm::ReplanPruning)), rows.get(&(kind, 0, Arm::RecomputeInTheLockAsToday))) {
            (Some(replan), Some(today))
                if replan.outcome.published
                    && today.outcome.published
                    && replan.outcome.units_moved == today.outcome.units_moved
                    && replan.outcome.units_moved > 0 => {}
            (Some(replan), Some(today)) => failures.push(format!(
                "kind={} k=0 回锁重排（published={} units_moved={}）与今天锁内重算（published={} units_moved={}）搬的单元数不同或没搬：重排那一臂与今天的计划不是同一份",
                kind.name(),
                replan.outcome.published,
                replan.outcome.units_moved,
                today.outcome.published,
                today.outcome.units_moved
            )),
            _ => failures.push(format!("kind={} k=0 回锁重排或今天锁内重算那一行缺", kind.name())),
        }
    }
    // 自证二：阳性对照在覆盖写 R 里单元、k ≥ 1 的历史上被发布路径拒或 checker 红。
    for count in Kind::OverwriteAUnitInTheRegion.counts().iter().copied().filter(|count| *count >= 1) {
        checked += 1;
        match rows.get(&(Kind::OverwriteAUnitInTheRegion, count, Arm::ReplanWithoutPruning)) {
            Some(row) if !row.outcome.refusal_member.is_empty() || !row.violated.is_empty() => {}
            Some(row) => failures.push(format!(
                "阳性对照 kind={} k={} 既没被发布路径拒（refused=none）checker 也没红（published={} units_moved={}）：A/B 的判法抓不到不剔的那一臂",
                Kind::OverwriteAUnitInTheRegion.name(),
                count,
                row.outcome.published,
                row.outcome.units_moved
            )),
            None => failures.push(format!("阳性对照 kind={} k={} 那一行缺", Kind::OverwriteAUnitInTheRegion.name(), count)),
        }
    }
    println!("SELF name=replan_after_the_window checked={checked} failed={}", failures.len());
    for failure in &failures {
        println!("  ✗ 自检：{failure}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}
