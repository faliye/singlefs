//! 判决「三、改法」第 7 条（回锁那一次发布重判回退门）的 A/B。
//! 一段历史：起点池拷一份、可写挂载 → 签出活意图、取票（同 `replan_after_the_window`）→ 锁外那一段期间前台推抬 F 的空发布
//! （先照准入那一条抬到上限；F 已在上限就推不抬 F 的空发布，`core-patch` 里的 `push_one_floor_raise_or_an_empty_publish_keeping_the_floor`）
//! → 回锁，按臂发这一批（第 5 条的回锁重排）→ 读根环里还剩几个不同用户状态 → 会话丢掉 → 池级 checker。
//! 推到哪为止分两种（variant）：
//!   until_four_states：推到根环里不同用户状态不多于 4 个就停（派发提示的字面）。
//!   until_the_gate_closes：推到今天的回退门在现行那一版上判成关（门关的那一格才分辨两臂），至多 MAX_PUSHES 串。
//! 两臂：regate（回锁那一次发布之前照今天的 `stop_of_the_rollback_gate` 重判，门关就不发）、no_regate（不重判，照发）。
//! 门判与状态数都调核心层现成的（`mounted_session::stop_of_the_rollback_gate`、`mount::rollback_states_of_the_ring`）。
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::probe_replan_after_the_window::{
    issue_an_intent, publish_after_the_relock, push_one_floor_raise_or_an_empty_publish_keeping_the_floor,
    rollback_gate_stop, rollback_states, ticket_of_the_batch, ArmOfTheRelock, OutcomeOfTheRelock,
};
use singlefs_core::mounted_session::MountedSession;

use crate::judged::{short, violated_invariants};
use crate::pool::{copied, parameters, start_pool, Devices};

/// 推的串数上限：根环 3S 槽（S = 8）两圈。
const MAX_PUSHES: usize = 48;
/// 回退门保住的不同用户状态个数（D16（发布语义） 已定项 1）。
const ROLLBACK_STATES_KEPT: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    UntilFourStates,
    UntilTheGateCloses,
}

impl Variant {
    const ALL: [Variant; 2] = [Variant::UntilFourStates, Variant::UntilTheGateCloses];
    fn name(self) -> &'static str {
        match self {
            Variant::UntilFourStates => "until_four_states",
            Variant::UntilTheGateCloses => "until_the_gate_closes",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Regate,
    NoRegate,
}

impl Arm {
    const ALL: [Arm; 2] = [Arm::Regate, Arm::NoRegate];
    fn name(self) -> &'static str {
        match self {
            Arm::Regate => "regate",
            Arm::NoRegate => "no_regate",
        }
    }
}

struct Row {
    line: String,
    fingerprint: String,
    gate_closed_before_the_relock: bool,
    published: bool,
    states_after_the_publish: u64,
}

fn one_history(start: &Devices, variant: Variant, arm: Arm) -> Result<Row, String> {
    let parameters = parameters();
    let mut devices = copied(start);
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| short(&format!("mount:{error:?}"), 120))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    issue_an_intent(&mut session, devices.as_mut_slice()).map_err(|failure| format!("issue:{failure}"))?;
    let ticket = ticket_of_the_batch(&mut session, devices.as_slice()).map_err(|failure| format!("ticket:{failure}"))?;
    let states_at_the_ticket = rollback_states(&session, devices.as_slice())?;
    let mut pushes: Vec<String> = Vec::new();
    let (states_before, gate_before) = loop {
        let states = rollback_states(&session, devices.as_slice())?;
        let gate = rollback_gate_stop(&session, devices.as_slice());
        let reached = match variant {
            Variant::UntilFourStates => states.states_now <= ROLLBACK_STATES_KEPT,
            Variant::UntilTheGateCloses => gate.is_some(),
        };
        if reached || pushes.len() >= MAX_PUSHES {
            break (states, gate);
        }
        let pushed = push_one_floor_raise_or_an_empty_publish_keeping_the_floor(&mut session, &mut devices)
            .map_err(|failure| format!("push_{}:{failure}", pushes.len() + 1))?;
        pushes.push(pushed);
    };
    let outcome: OutcomeOfTheRelock = match (arm, &gate_before) {
        (Arm::Regate, Some(stop)) => OutcomeOfTheRelock {
            not_published_because: format!("rollback_gate_closed:{stop}"),
            ..OutcomeOfTheRelock::default()
        },
        (Arm::Regate, None) | (Arm::NoRegate, _) => {
            publish_after_the_relock(&mut session, devices.as_mut_slice(), &ticket, ArmOfTheRelock::ReplanPruning)
        }
    };
    let states_after = rollback_states(&session, devices.as_slice())?;
    let txg_after = session.current.root().checkpoint_txg.0;
    drop(session);
    let violated = violated_invariants(&devices);
    let pushes_text = pushes.iter().map(|pushed| short(pushed, 80)).collect::<Vec<_>>().join(",");
    let fingerprint = format!(
        "ticket_txg={} region_start={} ticket_planned_units={} states_at_ticket={} pushes={} states_now_before_relock={} states_after_next_publish_and_margin_before_relock={} gate_before_relock={} pushed=[{}]",
        ticket.version.root.checkpoint_txg.0,
        ticket.intent.region.start_slot().0,
        ticket.planned.len(),
        states_at_the_ticket.states_now,
        pushes.len(),
        states_before.states_now,
        states_before.states_after_the_next_publish_and_the_margin,
        gate_before.as_deref().unwrap_or("open"),
        pushes_text,
    );
    let line = format!(
        "HISTORY name=rollback_gate_after_the_window variant={} arm={} {} published={} publish_kind={} not_published_because=[{}] refused={} units_moved={} units_pruned={} txg_after={} states_now_after_publish={} states_after_next_publish_and_margin_after={} checker_red={} violated=[{}]",
        variant.name(),
        arm.name(),
        fingerprint,
        outcome.published,
        if outcome.kind_name.is_empty() { "none" } else { outcome.kind_name.as_str() },
        outcome.not_published_because,
        if outcome.refusal_member.is_empty() { "none" } else { outcome.refusal_member.as_str() },
        outcome.units_moved,
        outcome.units_pruned,
        txg_after,
        states_after.states_now,
        states_after.states_after_the_next_publish_and_the_margin,
        !violated.is_empty(),
        violated.join(","),
    );
    Ok(Row {
        line,
        fingerprint,
        gate_closed_before_the_relock: gate_before.is_some(),
        published: outcome.published,
        states_after_the_publish: states_after.states_now,
    })
}

pub fn run() {
    let start = start_pool();
    let mut checked = 0_u64;
    let mut failures: Vec<String> = Vec::new();
    let mut rows: Vec<(Variant, Arm, Row)> = Vec::new();
    for variant in Variant::ALL {
        for arm in Arm::ALL {
            match catch_unwind(AssertUnwindSafe(|| one_history(&start, variant, arm))) {
                Ok(Ok(row)) => {
                    println!("{}", row.line);
                    rows.push((variant, arm, row));
                }
                Ok(Err(failure)) => println!(
                    "HISTORY name=rollback_gate_after_the_window variant={} arm={} setup_failed=[{}]",
                    variant.name(),
                    arm.name(),
                    short(&failure, 300)
                ),
                Err(_payload) => println!(
                    "HISTORY name=rollback_gate_after_the_window variant={} arm={} panicked=1",
                    variant.name(),
                    arm.name()
                ),
            }
        }
    }
    for arm in Arm::ALL {
        let of_the_arm: Vec<&Row> = rows.iter().filter(|(_, candidate, _)| *candidate == arm).map(|(_, _, row)| row).collect();
        println!(
            "RESULT name=rollback_gate_after_the_window arm={} histories={} published={} gate_closed_before_relock={} states_after_publish=[{}] fewer_than_four_states_after_publish={}",
            arm.name(),
            of_the_arm.len(),
            of_the_arm.iter().filter(|row| row.published).count(),
            of_the_arm.iter().filter(|row| row.gate_closed_before_the_relock).count(),
            of_the_arm.iter().map(|row| row.states_after_the_publish.to_string()).collect::<Vec<_>>().join(","),
            of_the_arm.iter().filter(|row| row.states_after_the_publish < ROLLBACK_STATES_KEPT).count(),
        );
    }
    for variant in Variant::ALL {
        checked += 1;
        let of_the_variant: Vec<&Row> = rows.iter().filter(|(candidate, _, _)| *candidate == variant).map(|(_, _, row)| row).collect();
        if of_the_variant.len() != Arm::ALL.len() {
            failures.push(format!("variant={} 两臂只有 {} 行", variant.name(), of_the_variant.len()));
        } else if of_the_variant[0].fingerprint != of_the_variant[1].fingerprint {
            failures.push(format!("variant={} 两臂回锁之前的票、推的串与门判不同：重放不确定，两臂不可比", variant.name()));
        }
        checked += 1;
        if let Some((_, _, row)) = rows.iter().find(|(candidate, arm, _)| *candidate == variant && *arm == Arm::Regate) {
            if row.gate_closed_before_the_relock && row.published {
                failures.push(format!("variant={} 门关着而 regate 那一臂发了：探针的重判没接上", variant.name()));
            }
        }
    }
    println!("SELF name=rollback_gate_after_the_window checked={checked} failed={}", failures.len());
    for failure in &failures {
        println!("  ✗ 自检：{failure}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}
