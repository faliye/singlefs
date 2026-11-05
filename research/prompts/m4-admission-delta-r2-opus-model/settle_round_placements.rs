//! 设计轮 m4-admission-delta-r2 攻方腿的模型（攻方副本用，不进仓的 crates/）。`tools/rerun.sh` 先用 `tools/apply_settle_trace.py` 给副本的
//! `settle_the_allocation_record_tree` 套上逐轮记录（不改任何判定），再把这一份拷成
//! `crates/singlefs-harness/tests/m4_admission_delta_r2_attack_settle_rounds.rs` 跑。
//!
//! 量的是格一那一问：固定点「每一轮只处理这一轮新进出的分配记录」能不能与今天逐个相同。今天每一轮都从分配器的一份新拷贝起，
//! 把这次的释放与全部角色按 bump 次序重新取一遍落点。要是后一轮新进来的角色排在前一轮已有的角色前面（分配记录树的节点排在记账树、
//! 中央映射树、树表之前），或者同一个角色在两轮里取到的槽不同，只在前一轮的预演上把新进来的那几个补取一遍，就得不出今天那一份落点。
//! 这一份数每一次调用里相邻两个取落点的轮之间：同一个角色换了槽的有几个、新进来的角色排在前一轮已有角色前面的有几个。
//! 盘都是内存稀疏盘；只有一个用例（套上的记录是进程里一份全局表，同一个二进制里不并行跑别的用例）。没有崩溃点：只数预演里的落点。
#![allow(dead_code, reason = "搭池那一份只用到其中一部分")]

#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use std::collections::BTreeMap;

use common_admission_pool::{plain_device, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::InodeNumber;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::m4_admission_delta_r2_attack_trace::{
    take_the_settling_calls, RoundPlacements,
};
use singlefs_core::transaction::TransactionUnit;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::e142_parameters;

const JOURNAL_RING_OF_SIX_MEBIBYTES: u64 = 6 << 20;
const UNIT_AREA_SLOTS_OF_THE_SMALL_POOL: u64 = 16_384;

fn pool_on_the_six_mebibyte_ring() -> PoolUnderTest<SparseBlockDevice> {
    let mut parameters: MakeFilesystemParameters = e142_parameters(512, 512);
    parameters.geometry.journal_ring_bytes = JOURNAL_RING_OF_SIX_MEBIBYTES;
    let unit_area_start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_OF_SIX_MEBIBYTES)
        .expect("6 MiB 的环末尾的下一个槽 1408 = 22 × 64，落在聚簇段边界上")
        .slot()
        .0;
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters,
        (unit_area_start + UNIT_AREA_SLOTS_OF_THE_SMALL_POOL) * SLOT_BYTES,
        plain_device,
    )
}

/// 一次用户改动里全部固定点调用相邻取落点轮之间的比较，加起来。
#[derive(Default)]
struct RoundsCompared {
    settling_calls: usize,
    allocation_rounds_per_call: Vec<usize>,
    release_only_rounds: usize,
    roles_compared: usize,
    roles_shifted_between_rounds: usize,
    new_roles_placed_before_a_role_of_the_earlier_round: usize,
    first_shift: Option<String>,
}

impl RoundsCompared {
    fn of_the_calls(calls: &[Vec<RoundPlacements>]) -> Self {
        let mut compared = RoundsCompared {
            settling_calls: calls.len(),
            ..RoundsCompared::default()
        };
        for call in calls {
            let allocation_rounds: Vec<&RoundPlacements> =
                call.iter().filter(|round| !round.is_empty()).collect();
            compared.release_only_rounds += call.len() - allocation_rounds.len();
            compared.allocation_rounds_per_call.push(allocation_rounds.len());
            for pair in allocation_rounds.windows(2) {
                let earlier: BTreeMap<TransactionUnit, u64> = pair[0].iter().copied().collect();
                let later = pair[1];
                for (index, (role, slot)) in later.iter().enumerate() {
                    match earlier.get(role) {
                        Some(earlier_slot) => {
                            compared.roles_compared += 1;
                            if earlier_slot != slot {
                                compared.roles_shifted_between_rounds += 1;
                                compared.first_shift.get_or_insert_with(|| {
                                    format!("{role:?}:{earlier_slot}->{slot}")
                                });
                            }
                        }
                        None => {
                            let a_role_of_the_earlier_round_comes_after_it = later[index + 1..]
                                .iter()
                                .any(|(later_role, _)| earlier.contains_key(later_role));
                            if a_role_of_the_earlier_round_comes_after_it {
                                compared.new_roles_placed_before_a_role_of_the_earlier_round += 1;
                            }
                        }
                    }
                }
            }
        }
        compared
    }
}

fn content_bytes_of_a_sequential_write_into(data_units: u64) -> usize {
    let payload_bytes =
        u64::try_from(data_unit_payload_capacity()).expect("一个数据单元的净荷装得进 u64");
    usize::try_from((data_units - 1) * payload_bytes + 1).expect("几兆字节装得进 usize")
}

/// 一段历史：第一个文件之后六次覆盖写、三次建 inode、一次 64 个数据单元的顺序写、六次覆盖写、两次建了再删。每次用户改动之后取出
/// 这次改动里全部固定点调用的逐轮落点，打一行；末尾打合计。今天的代码判定照旧（套上的只是记录），这里不断言比较的结果，只断言记录真的接上了。
#[test]
fn placements_of_the_same_role_between_settling_rounds() {
    let mut pool = pool_on_the_six_mebibyte_ring();
    let calls_before_the_history = take_the_settling_calls();
    println!(
        "name=settle_rounds_before_the_history settling_calls={}",
        calls_before_the_history.len()
    );
    let mut steps: Vec<(String, RoundsCompared)> = Vec::new();
    for overwrite in 0..6 {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
        steps.push((format!("overwrite_{overwrite}"), RoundsCompared::of_the_calls(&take_the_settling_calls())));
    }
    let mut inodes: Vec<InodeNumber> = Vec::new();
    for creation in 0..3 {
        inodes.push(InodeNumber(pool.current_file_version().inode_number_watermark()));
        pool.create_one_inode().expect("建一个 inode 发成");
        steps.push((format!("create_inode_{creation}"), RoundsCompared::of_the_calls(&take_the_settling_calls())));
    }
    pool.sequential_write(content_bytes_of_a_sequential_write_into(64))
        .expect("顺序写 64 个数据单元发成");
    steps.push(("sequential_write_of_64_data_units".to_string(), RoundsCompared::of_the_calls(&take_the_settling_calls())));
    for overwrite in 0..6 {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写发成");
        steps.push((format!("overwrite_after_the_big_write_{overwrite}"), RoundsCompared::of_the_calls(&take_the_settling_calls())));
    }
    for (position, inode) in inodes.iter().take(2).enumerate() {
        pool.write_time_seconds += 1;
        let write_time_seconds = pool.write_time_seconds;
        pool.publish(
            UserChange::Unlink {
                inode: *inode,
                write_time_seconds,
            },
            None,
        )
        .expect("unlink 发成");
        steps.push((format!("unlink_{position}"), RoundsCompared::of_the_calls(&take_the_settling_calls())));
    }
    let mut settling_calls = 0;
    let mut calls_with_two_or_more_allocation_rounds = 0;
    let mut roles_compared = 0;
    let mut roles_shifted = 0;
    let mut new_roles_placed_before = 0;
    let mut most_allocation_rounds = 0;
    for (label, compared) in &steps {
        println!(
            "name=settle_rounds change={label} settling_calls={} allocation_rounds_per_call={:?} release_only_rounds={} \
             roles_compared={} roles_shifted_between_rounds={} new_roles_placed_before_a_role_of_the_earlier_round={} first_shift={}",
            compared.settling_calls,
            compared.allocation_rounds_per_call,
            compared.release_only_rounds,
            compared.roles_compared,
            compared.roles_shifted_between_rounds,
            compared.new_roles_placed_before_a_role_of_the_earlier_round,
            compared.first_shift.as_deref().unwrap_or("none")
        );
        settling_calls += compared.settling_calls;
        calls_with_two_or_more_allocation_rounds += compared
            .allocation_rounds_per_call
            .iter()
            .filter(|rounds| **rounds >= 2)
            .count();
        most_allocation_rounds = compared
            .allocation_rounds_per_call
            .iter()
            .copied()
            .fold(most_allocation_rounds, usize::max);
        roles_compared += compared.roles_compared;
        roles_shifted += compared.roles_shifted_between_rounds;
        new_roles_placed_before += compared.new_roles_placed_before_a_role_of_the_earlier_round;
    }
    println!(
        "name=settle_rounds_summary changes={} settling_calls={settling_calls} \
         calls_with_two_or_more_allocation_rounds={calls_with_two_or_more_allocation_rounds} \
         most_allocation_rounds_in_one_call={most_allocation_rounds} roles_compared={roles_compared} \
         roles_shifted_between_rounds={roles_shifted} new_roles_placed_before_a_role_of_the_earlier_round={new_roles_placed_before}",
        steps.len()
    );
    assert!(
        steps.iter().all(|(_, compared)| compared.settling_calls >= 1),
        "每次用户改动至少走一次固定点：记录没接上就是套记录那一步出了错"
    );
}
