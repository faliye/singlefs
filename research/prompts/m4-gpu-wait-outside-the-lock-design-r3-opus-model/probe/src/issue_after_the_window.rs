//! 第四节第 6 条签发那一支：锁外照取输入那一刻的一版（V0）选段，锁外期间前台的用户数据落进选中的段（候选段不排他；用户数据取单元区里
//! 这次挂载开过的聚簇段与活 R 之外最低的空槽，`allocator.rs` 的 `try_allocate_user_data`），回锁里在现行那一版（V_cur）上签发。
//! 形态没写意图里的 `c0`、`segs0` 照哪一版取，两臂：
//!   ticket：照 V0 取——锁外照今天的代码在 V0 上建好的那条意图原样带回锁里签；
//!   current：照 V_cur 取——`c0` 取 V_cur 的 txg（签发那次发布的上一版，与今天 `mounted_session.rs` 签发那一支同一个取法），`segs0` 照 V_cur 的分配记录数。
//! 选段不在探针里另写：起点池拷一份、挂上、走今天的显式整理入口，读它发出的第一条带意图的那一版 logged_ops 里的意图（就是今天在 V0 上会签的那一条）。
//! 前台改动（用户定的，放开扫）：one_byte_writes（第一个文件部分写一个字节 k 次，每次换一个新数据单元）、sequential_write_of_the_second_inode
//! （第二个 inode 一次顺序写 k 个数据单元）、unlink_first_file（删第一个文件，选中的段可能就此空了）。
//! 签发照会话那一支的写法：先在分配器上 `issue`（R 从签发那次起排他）、经 `transaction::publish_compaction_step` 发 `Issue`、没发成撤掉。
//! 判：签发那一版落盘之后、再走一趟今天的显式整理入口之后，各对整个池跑一遍池级 checker（`singlefs_checker::walk::check_pool_image`），
//! 读 I-5.6（活整理意图的区域里没有签发之后分配的记录）与别的红项；最后拷一份可写挂载。
//! 资格（这次挂载开过的聚簇段、R 里钉住的槽）照 V_cur 只报不拦：两臂都发，资格那一判由读者对着行判。
//! 没取崩溃状态：判的是签发那一版本身（不崩也在盘上），崩溃状态 0 个。
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use singlefs_checker::image::{
    InvariantVerdict, COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InodeNumber;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::compaction_plan::{empty_cluster_segments_by_the_records, UnitAreaOfOneDevice};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{
    publish_compaction_step, CompactionIntentStep, PoolVersion, PoolWriter, TransactionOutput,
    TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;

use crate::pool::{
    copied, memory_pool_of, parameters, start_pool, write_one_byte, Devices, WRITE_TIME_SECONDS,
};

const SECOND_INODE: u64 = FIRST_INODE_NUMBER + 1;
/// 先跑一段估全量，超过这么多秒就停、不跑全量（40 分钟）。
const ESTIMATE_LIMIT_SECONDS: f64 = 2400.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    OneByteWrites,
    SequentialWriteOfTheSecondInode,
    UnlinkFirstFile,
}

impl Kind {
    const ALL: [Kind; 3] = [
        Kind::OneByteWrites,
        Kind::SequentialWriteOfTheSecondInode,
        Kind::UnlinkFirstFile,
    ];
    fn name(self) -> &'static str {
        match self {
            Kind::OneByteWrites => "one_byte_writes",
            Kind::SequentialWriteOfTheSecondInode => "sequential_write_of_the_second_inode",
            Kind::UnlinkFirstFile => "unlink_first_file",
        }
    }
    /// k 的取样：部分写到 48 次（起点池单元区里选中的段之外的洞填满之后才轮到选中的段），顺序写一次到 32 个单元。
    fn counts(self) -> &'static [usize] {
        match self {
            Kind::OneByteWrites => &[0, 1, 2, 4, 8, 16, 24, 32, 48],
            Kind::SequentialWriteOfTheSecondInode => &[1, 2, 4, 8, 16, 32],
            Kind::UnlinkFirstFile => &[1],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Ticket,
    Current,
}

impl Arm {
    const ALL: [Arm; 2] = [Arm::Ticket, Arm::Current];
    fn name(self) -> &'static str {
        match self {
            Arm::Ticket => "ticket",
            Arm::Current => "current",
        }
    }
}

fn short(text: &str, limit: usize) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_").chars().take(limit).collect()
}

fn foreground(kind: Kind, count: usize, session: &mut MountedSession, devices: &mut Devices) -> Result<(), String> {
    match kind {
        Kind::OneByteWrites => {
            for index in 0..count {
                let byte = 0xA0_u8.wrapping_add(u8::try_from(index % 64).expect("小于 64"));
                // 起点池里没被部分写过的那一半单元（1、3、…、39），轮着写。
                write_one_byte(session, devices, 2 * (index % 20) + 1, byte)?;
            }
            Ok(())
        }
        Kind::SequentialWriteOfTheSecondInode => {
            let content = vec![0x5C_u8; count * data_unit_payload_capacity()];
            session
                .publish_user_change(
                    devices,
                    UserChange::SequentialWriteOfAnInode {
                        inode: InodeNumber(SECOND_INODE),
                        content: &content,
                        write_time_seconds: WRITE_TIME_SECONDS + 9,
                    },
                )
                .map(|_published| ())
                .map_err(|refused| short(&format!("{refused:?}"), 160))
        }
        Kind::UnlinkFirstFile => session
            .publish_user_change(
                devices,
                UserChange::Unlink {
                    inode: InodeNumber(FIRST_INODE_NUMBER),
                    write_time_seconds: WRITE_TIME_SECONDS + 9,
                },
            )
            .map(|_published| ())
            .map_err(|refused| short(&format!("{refused:?}"), 160)),
    }
}

/// 今天的代码在 V0 上签的那一条意图：起点池拷一份、挂上、走显式整理入口，读发出的第一条带活意图的那一版。交回意图与那一趟的结局。
fn intent_chosen_on_the_ticket_version(start: &Devices) -> Result<(CompactionIntent, String), String> {
    let parameters = parameters();
    let mut devices = copied(start);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| short(&format!("mount:{error:?}"), 120))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let run = session.run_compaction(&mut devices).map_err(|refused| short(&format!("{refused:?}"), 160))?;
    let outcome = short(&format!("{:?}", run.outcome), 200);
    let issued = run
        .versions_published
        .iter()
        .find_map(|version| live_compaction_intent_of_the_entries(&version.logged_operations).map(|(_, intent)| intent))
        .ok_or_else(|| format!("no_intent_issued outcome={outcome}"))?;
    Ok((issued, outcome))
}

/// R 那块盘上起点在 R 里、未释放的记录。
fn unreleased_records_in(version: &TransactionOutput, region: CompactionRegion) -> Vec<&AllocationRecord> {
    version
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && record.slot.0 >= region.start_slot().0
                && record.slot.0 < region.end_slot().0
        })
        .collect()
}

/// R 罩住的槽里落着几个豁免映射的单元（映射树各节点、树表、实例表链：区域扫描判钉住的那三类）。
fn exempt_units_in(version: &TransactionOutput, region: CompactionRegion) -> usize {
    version
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::MappingTreeNodeBelowTheRoot(_)
                    | TransactionUnit::MappingTree
                    | TransactionUnit::TreeTable
                    | TransactionUnit::InstanceTable
                    | TransactionUnit::InstanceTablePageAfterTheFirst(_)
            )
        })
        .filter(|unit| {
            let start = unit.slot.0;
            let end = start + unit.identity.span_slots();
            start < region.end_slot().0 && end > region.start_slot().0
        })
        .count()
}

struct Judged {
    i56: &'static str,
    i56_detail: String,
    violated: Vec<&'static str>,
}

fn judged(devices: &Devices) -> Judged {
    let verdicts = check_pool_image(&memory_pool_of(devices));
    let (i56, i56_detail) = verdicts
        .iter()
        .find(|(name, _)| *name == COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED)
        .map_or(("absent", String::new()), |(_, verdict)| match verdict {
            InvariantVerdict::Holds => ("holds", String::new()),
            InvariantVerdict::Violated(detail) => ("violated", short(detail, 200)),
            InvariantVerdict::NotApplicable(reason) => ("not_applicable", short(reason, 80)),
            InvariantVerdict::PartiallyJudged(detail) => ("partially_judged", short(detail, 120)),
        });
    let violated = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    Judged { i56, i56_detail, violated }
}

struct Outcome {
    line: String,
    landed: bool,
    issue_published: bool,
    i56_red_after_issue: bool,
    other_red_after_issue: usize,
}

fn one_history(start: &Devices, ticket_intent: CompactionIntent, kind: Kind, count: usize, arm: Arm) -> Result<Outcome, String> {
    let parameters = parameters();
    let mut devices = copied(start);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| short(&format!("mount:{error:?}"), 120))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let v0_txg = session.current.root().checkpoint_txg;
    if v0_txg != ticket_intent.allocation_generation_at_issue {
        return Err(format!("ticket_txg_mismatch v0={} c0={}", v0_txg.0, ticket_intent.allocation_generation_at_issue.0));
    }
    let region = ticket_intent.region;
    let records_at_v0 = unreleased_records_in(session.current.version_of_trees().ok_or("v0_without_trees")?, region).len();
    foreground(kind, count, &mut session, &mut devices).map_err(|refused| format!("foreground_refused:{refused}"))?;
    let previous: TransactionOutput = session.current.version_of_trees().ok_or("current_without_trees")?.clone();
    let current_txg = previous.root.checkpoint_txg;
    let records_now = unreleased_records_in(&previous, region);
    let new_records = records_now.iter().filter(|record| record.generation.0 > v0_txg.0).count();
    let newest_generation = records_now.iter().map(|record| record.generation.0).max().unwrap_or(0);
    let in_cluster_segments = session.allocator.cluster_segments().contains(&region.start_slot());
    let exempt_units = exempt_units_in(&previous, region);
    let unit_area = {
        let device_map = session.allocator.devices.first().ok_or("no_device")?;
        UnitAreaOfOneDevice { device: device_map.device, start: device_map.unit_area_start(), slots: device_map.unit_area_slots() }
    };
    let intent = match arm {
        Arm::Ticket => ticket_intent,
        Arm::Current => CompactionIntent {
            region,
            allocation_generation_at_issue: current_txg,
            empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(&previous.allocation_records, unit_area),
        },
    };
    session.allocator.compaction_mut().issue(intent);
    let issued = {
        let switches = session.allocator.deletion_switches();
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        writer.set_container_tree_capacities(switches.container_tree_capacities());
        publish_compaction_step(&mut writer, &mut session.allocator, &previous, &[], CompactionIntentStep::Issue(intent), session.instance)
    };
    let (issue_published, issue_text) = match issued {
        Ok(published) => {
            session.current = PoolVersion::of_a_version_of_trees(published);
            (true, "published".to_string())
        }
        Err(refused) => {
            session.allocator.compaction_mut().withdraw_the_live_intent();
            (false, format!("refused:{}", short(&format!("{refused:?}"), 160)))
        }
    };
    let after_issue = judged(&devices);
    let run_text = match session.run_compaction(&mut devices) {
        Ok(run) => format!(
            "published={} batches={} outcome={}",
            run.versions_published.len(),
            run.batches_published,
            short(&format!("{:?}", run.outcome), 200)
        ),
        Err(refused) => format!("refused:{}", short(&format!("{refused:?}"), 160)),
    };
    drop(session);
    let after_run = judged(&devices);
    let mut remount_devices = copied(&devices);
    let writable_remount = match mount_writable(&parameters, &mut remount_devices) {
        Ok(_) => "ok".to_string(),
        Err(error) => short(&format!("{error:?}"), 100),
    };
    let other_red_after_issue = after_issue
        .violated
        .iter()
        .filter(|name| **name != COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED)
        .count();
    let line = format!(
        "HISTORY name=issue_after_the_window kind={} k={} arm={} v0_txg={} current_txg={} region_device={} region_start={} records_in_region_at_v0={} records_in_region_now={} new_records_in_region={} newest_generation_in_region={} c0={} segs0={} in_cluster_segments={} exempt_units_in_region={} issue=[{}] i56_after_issue={} i56_detail_after_issue=[{}] violated_after_issue=[{}] run_after=[{}] i56_after_run={} violated_after_run=[{}] writable_remount={}",
        kind.name(), count, arm.name(), v0_txg.0, current_txg.0, region.device().0, region.start_slot().0,
        records_at_v0, records_now.len(), new_records, newest_generation,
        intent.allocation_generation_at_issue.0, intent.empty_cluster_segments_at_issue,
        in_cluster_segments, exempt_units, issue_text, after_issue.i56, after_issue.i56_detail,
        after_issue.violated.join(","), run_text, after_run.i56, after_run.violated.join(","), writable_remount,
    );
    Ok(Outcome {
        line,
        landed: new_records > 0,
        issue_published,
        i56_red_after_issue: after_issue.i56 == "violated",
        other_red_after_issue,
    })
}

#[derive(Default)]
struct Tally {
    histories: u64,
    landed: u64,
    issue_published: u64,
    i56_red_after_issue: u64,
    i56_red_where_landed: u64,
    i56_red_where_not_landed: u64,
    other_red_after_issue: u64,
    setup_failed: u64,
    panicked: u64,
}

pub fn run() {
    let start = start_pool();
    let baseline = judged(&start);
    println!(
        "BASELINE name=issue_after_the_window start_pool_i56={} start_pool_violated=[{}]",
        baseline.i56,
        baseline.violated.join(",")
    );
    let (ticket_intent, ticket_outcome) = match intent_chosen_on_the_ticket_version(&start) {
        Ok(chosen) => chosen,
        Err(failure) => {
            println!("TICKET name=issue_after_the_window intent=none reason=[{failure}]");
            println!("  ✗ 起点池上今天的显式整理入口没签出意图：这一组历史判不了签发那一支，探针的起点池要换");
            std::process::exit(1);
        }
    };
    println!(
        "TICKET name=issue_after_the_window region_device={} region_start={} c0={} segs0={} explicit_run_on_v0=[{}]",
        ticket_intent.region.device().0,
        ticket_intent.region.start_slot().0,
        ticket_intent.allocation_generation_at_issue.0,
        ticket_intent.empty_cluster_segments_at_issue,
        ticket_outcome
    );
    let histories_in_total: usize = Kind::ALL.iter().map(|kind| kind.counts().len()).sum::<usize>() * Arm::ALL.len();
    let mut tallies: Vec<(Arm, Tally)> = Arm::ALL.iter().map(|arm| (*arm, Tally::default())).collect();
    let mut self_check_failures: Vec<String> = Vec::new();
    let mut histories_run = 0_usize;
    let started = Instant::now();
    for kind in Kind::ALL {
        for count in kind.counts() {
            for arm in Arm::ALL {
                let result = catch_unwind(AssertUnwindSafe(|| one_history(&start, ticket_intent, kind, *count, arm)));
                histories_run += 1;
                let tally = &mut tallies.iter_mut().find(|(candidate, _)| *candidate == arm).expect("每臂一份").1;
                tally.histories += 1;
                match result {
                    Ok(Ok(outcome)) => {
                        println!("{}", outcome.line);
                        tally.landed += u64::from(outcome.landed);
                        tally.issue_published += u64::from(outcome.issue_published);
                        tally.i56_red_after_issue += u64::from(outcome.i56_red_after_issue);
                        tally.i56_red_where_landed += u64::from(outcome.i56_red_after_issue && outcome.landed);
                        tally.i56_red_where_not_landed += u64::from(outcome.i56_red_after_issue && !outcome.landed);
                        tally.other_red_after_issue += u64::try_from(outcome.other_red_after_issue).expect("条数");
                        if arm == Arm::Current && outcome.i56_red_after_issue {
                            self_check_failures.push(format!("current 臂 kind={} k={} 签发之后 I-5.6 红：c0 取现行那一版还红，探针造签发的写法有错", kind.name(), count));
                        }
                        if !outcome.landed && outcome.i56_red_after_issue {
                            self_check_failures.push(format!("{} 臂 kind={} k={} 没有记录落进 R 而 I-5.6 红：判法认错了别的东西", arm.name(), kind.name(), count));
                        }
                    }
                    Ok(Err(failure)) => {
                        tally.setup_failed += 1;
                        println!("HISTORY name=issue_after_the_window kind={} k={} arm={} setup_failed=[{}]", kind.name(), count, arm.name(), failure);
                    }
                    Err(_payload) => {
                        tally.panicked += 1;
                        println!("HISTORY name=issue_after_the_window kind={} k={} arm={} panicked=1", kind.name(), count, arm.name());
                    }
                }
                if histories_run == 1 {
                    let first_seconds = started.elapsed().as_secs_f64();
                    let estimated_total_seconds = first_seconds * f64::from(u32::try_from(histories_in_total).expect("段数"));
                    println!(
                        "ESTIMATE name=issue_after_the_window histories={histories_in_total} first_history_seconds={first_seconds:.3} estimated_total_seconds={estimated_total_seconds:.1} limit_seconds={ESTIMATE_LIMIT_SECONDS}"
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
        println!(
            "RESULT name=issue_after_the_window arm={} histories={} landed={} issue_published={} i56_red_after_issue={} i56_red_where_landed={} i56_red_where_not_landed={} other_red_after_issue={} setup_failed={} panicked={}",
            arm.name(), tally.histories, tally.landed, tally.issue_published, tally.i56_red_after_issue,
            tally.i56_red_where_landed, tally.i56_red_where_not_landed, tally.other_red_after_issue, tally.setup_failed, tally.panicked
        );
    }
    if !baseline.violated.is_empty() {
        self_check_failures.push(format!("起点池本身就有红项 [{}]：判法读到的红不全是签发造的", baseline.violated.join(",")));
    }
    println!("WALL_SECONDS name=issue_after_the_window seconds={:.1}", started.elapsed().as_secs_f64());
    println!("SELF name=self_checks checked=3 failed={}", self_check_failures.len());
    for failure in &self_check_failures {
        println!("  ✗ 自检：{failure}");
    }
    if !self_check_failures.is_empty() {
        std::process::exit(1);
    }
}
