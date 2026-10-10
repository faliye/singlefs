//! s2 攻方原型（K2-2、K3-3、K1-2 的崩溃那一半）：自己造的几段小流，按层 0 的枚举域（`enumerate_layer0_versions`：每一段都展开，
//! 段内写的任意真子集逐个枚举）枚举每个崩溃状态，枚举器在每个状态上恢复、跑 oracle（读回的是哪一版的内容）与池级 checker。
//! 先用 `closed_form_state_count` 与层 0 口径（原地覆写三态）各算一遍状态数，超过 10⁶ 的那一段不跑、只报。
//! 用法：`s2_crash_states <场景>`，场景见 `main` 里的 match。一行一段：`name=crash scenario=… states=… violations=… checker_red=…`。
#[path = "REPO/crates/singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices, PoolUnderTest,
};
use singlefs_checker_tier::crash::{enumerate_layer0_versions, full_expansion, layer0_state_count_with_torn_in_place_overwrites};
use singlefs_core::address::InodeNumber;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::{PoolVersion, TransactionOutput};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Pool = PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>;

const STATES_AT_MOST: u64 = 1_000_000;

fn new_pool(stream: &SharedStream) -> Pool {
    let start = UnitAreaStart::following_the_journal_ring(6 << 20).expect("1408").slot().0;
    let for_devices = stream.clone();
    PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        (start + 1007 + 8_000) * SLOT_BYTES,
        move |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, for_devices.clone()),
    )
}

fn create(pool: &mut Pool, count: u64) {
    pool.write_time_seconds += 1;
    let t = pool.write_time_seconds;
    pool.publish(UserChange::NewInodes { count, write_time_seconds: t }, None).expect("建 inode");
}

fn unlink(pool: &mut Pool, inode: u64) {
    pool.write_time_seconds += 1;
    let t = pool.write_time_seconds;
    pool.publish(UserChange::Unlink { inode: InodeNumber(inode), write_time_seconds: t }, None).expect("unlink");
}

fn roll_back(pool: &mut Pool, target: &TransactionOutput) {
    let mut session = pool.session.take().expect("会话");
    let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
    roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        current,
        RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg },
    )
    .expect("回退做成");
    let root = current.root;
    pool.session = Some(session);
    pool.note_root_of_the_current_content(&root);
}

fn leaves(pool: &Pool) -> Vec<(u64, usize)> {
    pool.current_file_version().inode_leaf_containers.iter().map(|c| (c.contents.identity.container, c.contents.records.len())).collect()
}

fn retirement_records(pool: &Pool) -> usize {
    pool.current_file_version().retirement_containers.iter().map(|c| c.contents.records.len()).sum()
}

fn main() {
    let scenario = std::env::args().nth(1).expect("场景名");
    let stream = SharedStream::retaining_contents();
    let mut pool = new_pool(&stream);
    // 前缀（不录进被枚举的那一段）：各场景自己的
    let mut target_for_rollback: Option<TransactionOutput> = None;
    match scenario.as_str() {
        // 234 条记录 [1..=233]、[234]；被枚举的一段：unlink 2（意图 + 最后一批：左片吸收右片、开出退役树）。
        "absorb" => create(&mut pool, 233),
        // 四片 [233, 233, 233, 60]，第二片删到 117；被枚举的一段：unlink 350（第二片 116 条从右邻借 58 条，N1）。
        "borrow" => {
            create(&mut pool, 758);
            for inode in 234..=349 { unlink(&mut pool, inode); }
        }
        // 四片 [233, 233, 233, 1]，第二片删到 117；被枚举的一段：unlink 350（借完之后借出那片吸收第四片）。
        "lender-absorb" => {
            create(&mut pool, 699);
            for inode in 234..=349 { unlink(&mut pool, inode); }
        }
        // 建 233 个、unlink 2（吸收、一条退役记录）；被枚举的一段：回退到建完那一版（两片又都在，退役树随 cur）。
        "rollback-after-absorb" => {
            create(&mut pool, 233);
            target_for_rollback = Some(pool.current_file_version().clone());
            unlink(&mut pool, 2);
        }
        // 同上再做完回退；被枚举的一段：再 unlink 2（同一片叶的身份第二次退役，谱系上只有这一条）。
        "reunlink-after-rollback" => {
            create(&mut pool, 233);
            let target = pool.current_file_version().clone();
            unlink(&mut pool, 2);
            roll_back(&mut pool, &target);
        }
        other => panic!("不认的场景 {other}"),
    }
    let leaves_before = leaves(&pool);
    let base = pool.image();
    let operations_before = stream.operation_count();
    match scenario.as_str() {
        "absorb" | "reunlink-after-rollback" => unlink(&mut pool, 2),
        "borrow" | "lender-absorb" => unlink(&mut pool, 350),
        "rollback-after-absorb" => { let t = target_for_rollback.clone().expect("目标"); roll_back(&mut pool, &t) }
        _ => unreachable!(),
    }
    let leaves_after = leaves(&pool);
    let recorded = stream.retained_operations()[operations_before..].to_vec();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let (writes, segments) = writes_and_segments(&recorded, &geometry);
    let judged_root = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).last().expect("至少一次根槽写");
    let closed_form = closed_form_state_count(&segments);
    let torn = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    let segment_sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!(
        "name=plan scenario={scenario} leaves_before={leaves_before:?} leaves_after={leaves_after:?} retirement_records={} writes={} segments={} largest_segment={} closed_form_state_count={closed_form} torn_state_count={torn}",
        retirement_records(&pool), writes.len(), segments.len(), segment_sizes.iter().max().copied().unwrap_or(0)
    );
    if closed_form.max(torn) > STATES_AT_MOST || std::env::var("S2_PLAN_ONLY").is_ok() {
        println!("name=crash scenario={scenario} skipped=over-budget-or-plan-only");
        return;
    }
    let versions = pool.published_versions();
    let tally = enumerate_layer0_versions(&base, &writes, &segments, judged_root, &versions);
    let checker_red: u64 = tally.checker_violated_states.values().sum();
    let deletion_line: Vec<String> = tally
        .checker_evaluated_states
        .iter()
        .filter(|(name, _)| name.starts_with("I-9.3") || name.starts_with("I-3.13"))
        .map(|(name, n)| format!("{name}:{n}"))
        .collect();
    println!(
        "name=crash scenario={scenario} states={} violations={} first_violation={:?} checker_red={checker_red} checker_first={:?} failed_states={} no_file_states={} evaluated={deletion_line:?}",
        tally.states, tally.violations, tally.first_violation, tally.checker_first_violation, tally.failed_states, tally.no_file_states
    );
}
