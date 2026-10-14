//! 攻方腿 m3-admission-r1-opus，A5（C573）：有普通分配的发布真写出的固定点槽数，与它接在后面那一版现算的 ckpt_cost（空发布上界，
//! 「落得下」那一判拿它顶固定点）比。超过就是那一判的上界不罩有普通分配的发布。只在冻结副本的拷贝上跑，不进仓库的 crates/。
mod common_admission;
#[path = "opus_probe/common.rs"]
mod probe_common;

use probe_common::start_pool;
use singlefs_core::admission::{
    checkpoint_cost_of_the_version_to_build_on, space_budget_of_role, SpaceBudgetOfARole,
};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::data_unit_payload_capacity;

fn split_slots(version: &TransactionOutput) -> (u64, u64, u64) {
    let mut data = 0;
    let mut ordinary_other = 0;
    let mut fixed = 0;
    for role in &version.rewritten {
        match (role, space_budget_of_role(*role)) {
            (TransactionUnit::Data(_), _) => data += role.span_slots(),
            (_, SpaceBudgetOfARole::OrdinaryAllocation) => ordinary_other += role.span_slots(),
            _ => fixed += role.span_slots(),
        }
    }
    (data, ordinary_other, fixed)
}

#[test]
fn opus_probe_c573_fixed_point_of_a_publish_with_ordinary_allocation_against_the_checkpoint_cost() {
    let slots: Vec<u64> = std::env::var("OPUS_FP_UNIT_AREA_SLOTS")
        .ok()
        .map(|t| t.split(',').map(|n| n.parse().expect("槽数")).collect())
        .unwrap_or_else(|| vec![1391, 1391]);
    let ring: u64 = std::env::var("OPUS_FP_JOURNAL_RING_BYTES").ok().map_or(6 << 20, |t| t.parse().expect("字节"));
    let sizes: Vec<usize> = std::env::var("OPUS_FP_SIZES")
        .ok()
        .map(|t| t.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![1, 8, 144, 145, 2, 300, 1, 289, 150, 3, 433, 1]);
    let rounds: usize = std::env::var("OPUS_FP_ROUNDS").ok().map_or(40, |t| t.parse().expect("轮"));
    let mut pool = start_pool([slots[0], slots[1]], ring);
    let (mut publishes, mut user_exceed, mut raise_exceed, mut max_user_excess, mut max_raise_excess) = (0, 0, 0, i64::MIN, i64::MIN);
    for round in 0..rounds {
        let size = sizes[round % sizes.len()];
        let before = pool.session().current.file_version().expect("带文件").clone();
        let outcome = pool.sequential_write(size * data_unit_payload_capacity());
        let Ok(published) = outcome else {
            println!("FP round={round} size={size} refused");
            continue;
        };
        let allocator = &pool.session().allocator;
        let mut previous = before;
        for raise in &published.floor_raises {
            for empty in &raise.publishes {
                let cost = checkpoint_cost_of_the_version_to_build_on(Some(&previous), allocator).0;
                let (_, _, fixed) = split_slots(empty);
                let excess = fixed as i64 - cost as i64;
                max_raise_excess = max_raise_excess.max(excess);
                if excess > 0 {
                    raise_exceed += 1;
                }
                println!("FP round={round} kind=raise txg={} built_on={} ckpt_cost={cost} fixed_point_slots={fixed} excess={excess}", empty.root.checkpoint_txg.0, previous.root.checkpoint_txg.0);
                previous = empty.clone();
            }
        }
        let user = published.versions_published.last().expect("这次改动那一版");
        let cost = checkpoint_cost_of_the_version_to_build_on(Some(&previous), allocator).0;
        let (data, ordinary_other, fixed) = split_slots(user);
        let excess = fixed as i64 - cost as i64;
        publishes += 1;
        max_user_excess = max_user_excess.max(excess);
        if excess > 0 {
            user_exceed += 1;
        }
        println!(
            "FP round={round} kind=user size={size} txg={} built_on={} ckpt_cost={cost} data_slots={data} ordinary_other_slots={ordinary_other} fixed_point_slots={fixed} excess={excess} mapping_tree_height={}",
            user.root.checkpoint_txg.0,
            previous.root.checkpoint_txg.0,
            user.central_mapping_tree.shape.height()
        );
    }
    println!(
        "FP summary geometry={slots:?} ring={ring} user_publishes={publishes} user_exceed={user_exceed} max_user_excess={max_user_excess} raise_exceed={raise_exceed} max_raise_excess={max_raise_excess}"
    );
}

fn role_kind(role: &TransactionUnit) -> &'static str {
    match role {
        TransactionUnit::AllocationTree | TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => "allocation_record_tree",
        TransactionUnit::MappingTree | TransactionUnit::MappingTreeNodeBelowTheRoot(_) => "central_mapping_tree",
        TransactionUnit::AccountingTree | TransactionUnit::AccountingTreeNodeBelowTheRoot(_) => "accounting_tree",
        TransactionUnit::TreeTable => "tree_table",
        _ => "other",
    }
}

/// 同一个起点（新池第一个文件之后），先写 k 单元占掉一截，再在「落得下那一判按 ckpt_cost 顶固定点正好够」附近扫一个写的单元数 n：
/// 看拒在哪一道（落得下那一判报 `SpaceAdmissionRefused`，走固定点取不到落点报 `PlacementRefused`）。
#[test]
fn opus_probe_c573_which_judgment_refuses_near_the_landing_bound() {
    use singlefs_core::mounted_session::UserChangeRefused;
    use singlefs_core::transaction::PublishError;
    let first: usize = std::env::var("OPUS_FP_FIRST").ok().map_or(2700, |t| t.parse().expect("单元数"));
    let mut pool = start_pool([16384, 16384], 64 << 20);
    let wrote = pool.sequential_write(first * data_unit_payload_capacity());
    assert!(wrote.is_ok());
    let smalls: usize = std::env::var("OPUS_FP_SMALLS").ok().map_or(0, |t| t.parse().expect("次数"));
    for _ in 0..smalls {
        assert!(pool.overwrite(2999).is_ok());
    }
    let unblocked = pool.session().allocator.devices[0].unblocked_slots();
    let cost = checkpoint_cost_of_the_version_to_build_on(pool.session().current.file_version(), &pool.session().allocator).0;
    println!("FPB first={first} smalls={smalls} unblocked={unblocked} ckpt_cost={cost}");
    let centre = (unblocked as usize - cost as usize - 43) / 2;
    for n in centre.saturating_sub(12)..=centre + 4 {
        let mut probe = probe_common::clone_pool(&pool);
        let outcome = probe.session.as_mut().map(|_| ()).and(Some(())).map(|_| {
            let mut session = probe.session.take().expect("会话");
            // 直接调会话一次（不经 PoolUnderTest::publish 的记账），只看第一次被拒是哪一种：推抬 F 之前的那一次记在 refusals 里。
            let result = session.publish_user_change(
                &mut probe.devices,
                singlefs_core::mounted_session::UserChange::SequentialWrite(singlefs_core::transaction::FirstFile {
                    content: &common_admission::content_of(n * data_unit_payload_capacity(), 7),
                    write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS + 5,
                }),
            );
            probe.session = Some(session);
            result
        });
        let text = match outcome.expect("跑了") {
            Ok(published) => format!(
                "Ok raises={} first_refusal={:?} fixed_point_by_tree={:?}",
                published.floor_raises.len(),
                published.refusals_that_pushed_the_floor_raises.first().map(short_refusal),
                {
                    let mut kinds = std::collections::BTreeMap::new();
                    for role in &published.versions_published.last().expect("那一版").rewritten {
                        if space_budget_of_role(*role) == SpaceBudgetOfARole::CheckpointReservePool {
                            *kinds.entry(role_kind(role)).or_insert(0u64) += role.span_slots();
                        }
                    }
                    kinds
                }
            ),
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => format!(
                "NoSpace first_refusal={:?} last_refusal={}",
                no_space.refusals_that_pushed_the_floor_raises.first().map(short_refusal),
                short_refusal(&no_space.last_refusal)
            ),
            Err(other) => format!("Other {}", format!("{other:?}").chars().take(120).collect::<String>()),
        };
        println!("FPB n={n} need_with_ckpt_cost={} {text}", 2 * n as u64 + 43 + cost);
    }

    fn short_refusal(refusal: &PublishError) -> String {
        match refusal {
            PublishError::SpaceAdmissionRefused(short) => format!(
                "SpaceAdmissionRefused(d0 avail_slots={} demand_slots={})",
                short.short_devices[0].available.0 / 16384,
                short.short_devices[0].demand.0 / 16384
            ),
            PublishError::PlacementRefused { unit, .. } => format!("PlacementRefused(unit={unit:?})"),
            other => format!("{}", format!("{other:?}").chars().take(80).collect::<String>()),
        }
    }
}
