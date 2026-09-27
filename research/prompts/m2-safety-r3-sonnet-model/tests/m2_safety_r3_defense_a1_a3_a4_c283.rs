//! `m2-safety-r3` 云端辩方（Sonnet）：替第二轮判决只被打中过一轮的 A1（Sonnet 读法：换下的块算进可用）、A3、A4 辩护——
//! 各自加上 C283（本文件自己实现，见 `r3_defense_common::DefensePool::overwrite_with_c283` /
//! `mount_writable_with_admission_candidate_and_c283`，不用攻方的）之后，在「正常卸载再挂」「崩了再挂」两支上
//! 量 S4、S4b、删了再写的步数、吸收态，还出不出局。
//!
//! 冻结副本：`/tmp/claude-1000/safety-r3-frozen/`。回退形态已经改成挂着时向前发布
//! （`mount::roll_back_by_a_forward_publish`）：第二轮的模型在改形态之前的冻结副本上写的、走的是旧的
//! `mount_rollback` 挂载入口——那个入口在这份代码上已经不存在，这份用例吸收态探针里的回退那一步改用
//! `roll_back_by_a_forward_publish`。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod r3_defense_common;

use r3_defense_common::{build_pool, Branch, DefensePool};
use singlefs_core::admission::S4Candidate;
use singlefs_core::mount::{MountError, RollbackTarget};
use singlefs_core::transaction::{PublishError, TransactionOutput};
use singlefs_harness::history::HistoryDeviceWidth;

const CANDIDATES: [S4Candidate; 4] = [S4Candidate::Baseline, S4Candidate::A1, S4Candidate::A3, S4Candidate::A4];
const BRANCHES: [Branch; 2] = [Branch::NormalUnmount, Branch::Crash];
const WIDTHS: [(&str, HistoryDeviceWidth); 3] = [
    ("240", HistoryDeviceWidth::UnitAreaOf240Slots),
    ("256", HistoryDeviceWidth::UnitAreaOf256Slots),
    ("384", HistoryDeviceWidth::UnitAreaOf384Slots),
];
/// 「删了再写的步数」与「吸收态」都在这个上界内数；超过就记「>STEP_CAP」（吸收态）。
const STEP_CAP: u64 = 12;

fn candidate_name(candidate: S4Candidate) -> &'static str {
    match candidate {
        S4Candidate::Baseline => "baseline",
        S4Candidate::A1 => "a1",
        S4Candidate::A3 => "a3",
        S4Candidate::A4 => "a4",
    }
}

fn branch_name(branch: Branch) -> &'static str {
    match branch {
        Branch::NormalUnmount => "normal_unmount",
        Branch::Crash => "crash",
    }
}

fn short_debug<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
        .split(['(', ' ', '{'])
        .next()
        .unwrap_or("?")
        .to_string()
}

fn publish_member(result: &Result<TransactionOutput, PublishError>) -> String {
    match result {
        Ok(_) => "Applied".to_string(),
        Err(error) => short_debug(error),
    }
}

fn mount_member(result: &Result<(), MountError>) -> String {
    match result {
        Ok(()) => "Applied".to_string(),
        Err(error) => short_debug(error),
    }
}

/// 一路覆盖写（候选自己判，不带 C283）直到第一次被拒，交回「放行了几次」与「第一次被拒的错误成员」。
fn fill_to_the_limit(pool: &mut DefensePool) -> (u64, String) {
    let mut admitted = 0u64;
    loop {
        match pool.overwrite(admitted + 1) {
            Ok(_) => admitted += 1,
            Err(error) => return (admitted, short_debug(&error)),
        }
        if admitted > 200 {
            return (admitted, "NeverRefusedWithin200".to_string());
        }
    }
}

/// 一条组合（宽度、候选、支、带不带 C283）跑完 S4 / S4b / 删了再写的步数 / 吸收态，逐行打印。
fn run_one_combination(width_name: &str, width: HistoryDeviceWidth, candidate: S4Candidate, branch: Branch, c283: bool) {
    let tag = format!(
        "{width_name}-{}-{}-c283{}",
        candidate_name(candidate),
        branch_name(branch),
        u8::from(c283)
    );
    let mut pool = build_pool(&tag, width, candidate);
    let (admitted, first_refusal) = fill_to_the_limit(&mut pool);
    let session_root_before_remount = pool.output.root;

    let remount = pool.remount(branch, candidate, c283);
    println!(
        "S4 width={width_name} candidate={} branch={} c283={c283} admitted_in_session={admitted} first_refusal_in_session={first_refusal} remount={}",
        candidate_name(candidate),
        branch_name(branch),
        mount_member(&remount)
    );
    if remount.is_err() {
        // S4 出局条件：会话里放行过至少一次写，紧接着的下一次可写挂载被拒。
        return;
    }

    // S4b + 删了再写的步数：挂载做成之后连续做同样大小的覆盖写，数第几次放行；每一步是否带 C283 跟 S4 那一列一致。
    let mut succeeded_at: Option<u64> = None;
    for step in 1..=STEP_CAP {
        let outcome = if c283 {
            let (result, raised) = pool.overwrite_with_c283(admitted + 1000 + step);
            if step == 1 {
                println!(
                    "S4b width={width_name} candidate={} branch={} c283={c283} step=1 result={} raised_floor_this_step={raised}",
                    candidate_name(candidate),
                    branch_name(branch),
                    publish_member(&result)
                );
            }
            result
        } else {
            let result = pool.overwrite(admitted + 1000 + step);
            if step == 1 {
                println!(
                    "S4b width={width_name} candidate={} branch={} c283={c283} step=1 result={}",
                    candidate_name(candidate),
                    branch_name(branch),
                    publish_member(&result)
                );
            }
            result
        };
        if outcome.is_ok() {
            succeeded_at = Some(step);
            break;
        }
    }
    match succeeded_at {
        Some(step) => println!(
            "D3I9 width={width_name} candidate={} branch={} c283={c283} steps_to_success={step}",
            candidate_name(candidate),
            branch_name(branch)
        ),
        None => println!(
            "D3I9 width={width_name} candidate={} branch={} c283={c283} steps_to_success=>{STEP_CAP}",
            candidate_name(candidate),
            branch_name(branch)
        ),
    }

    if succeeded_at.is_some() {
        return;
    }

    // 吸收态探针：STEP_CAP 步同样大小覆盖写全部被拒——再试一次同支重挂（应当幂等：F 已经在上一次抬到过上限），
    // 再试一次覆盖写；最后试一次回退（`roll_back_by_a_forward_publish`，不另包 C283，报告据实标）。
    let remount_again = pool.remount(branch, candidate, c283);
    let overwrite_after_remount_again = pool.overwrite(admitted + 2000);
    let rollback_target = RollbackTarget {
        instance: session_root_before_remount.instance,
        checkpoint_txg: session_root_before_remount.checkpoint_txg,
    };
    let rollback_result = pool.rollback_once(rollback_target);
    println!(
        "ABSORBING width={width_name} candidate={} branch={} c283={c283} remount_again={} overwrite_after_remount_again={} rollback_to_pre_remount_root={}",
        candidate_name(candidate),
        branch_name(branch),
        mount_member(&remount_again),
        publish_member(&overwrite_after_remount_again),
        rollback_result.as_ref().map(|()| "Applied".to_string()).unwrap_or_else(|error| short_debug(error))
    );
}

/// 这个测试只打印，不带断言——出不出局由报告里读这些行判（`.claude/singlefs-ai-sop/rules/show-me-test.md`
/// 「没实现的要明说」：候选与 C283 都是三方论证专用分支，产品路径不走它们，这里也不接产品级断言）。
#[test]
fn r3_defense_s4_s4b_delete_then_write_and_absorbing_across_candidates_branches_and_c283() {
    for (width_name, width) in WIDTHS {
        for candidate in CANDIDATES {
            for branch in BRANCHES {
                for c283 in [false, true] {
                    run_one_combination(width_name, width, candidate, branch, c283);
                }
            }
        }
    }
}
