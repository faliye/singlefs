//! m2-final-code-r4 云端攻方 Z21：回退见证的两处修补（被罩住的删、按回退行补写）。只在冻结副本的拷贝上跑。
//! 同一份文件也放进第三轮冻结树的拷贝跑（对照：没有补写、没有罩住删），只用两棵树都有的接口。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use common::{
    build_pool, crash_state_devices, file_content, parameters, publish_overwrite_in_process,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_rollback, mount_writable, MountError, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot_holds_a_root,
    instance_table_of_root, readable_roots, recover, rollback_witness_of_the_pool, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::fault_injection::{
    NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget,
};
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

/// A（txg 3）之后覆盖写到 txg `last_txg`，实例 1。
fn pool_through(tag: &str, last_txg: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let mut current = pool.output.clone();
    let mut seed = 1;
    while current.root.checkpoint_txg.0 < last_txg {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(seed),
            FIXED_WRITE_TIME_SECONDS + 60 * seed as u64,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        seed += 1;
    }
    pool.output = current;
    pool
}

/// 同一个 last_txg 的起点镜像每个进程只建一次（建池要在临时目录里写 4 GiB 稀疏文件、mkfs 清 journal 环，I/O 重），之后在内存里拷。
/// `pool_through` 的内容与写入时间都写死，标签只进临时文件名，同一个 last_txg 建出来的镜像相同。
fn image_through(last_txg: u64) -> MemoryPool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<u64, MemoryPool>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(image) = cache.lock().expect("锁").get(&last_txg) {
        return image.clone();
    }
    let pool = pool_through(&format!("s-opus-cache-{last_txg}"), last_txg);
    let image = pool.memory_pool();
    drop(pool);
    cache.lock().expect("锁").insert(last_txg, image.clone());
    image
}

fn first_root_write(ops: &[RetainedOperation]) -> usize {
    ops.iter()
        .position(|op| op.operation.kind == RecordedOperationKind::WriteForceUnitAccess)
        .expect("写过根")
}

fn roots_of(image: &MemoryPool) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<_> = readable_roots(
        image,
        &sc.immutable.region_devices,
        &sc.immutable.sizes,
        &sc.immutable.filesystem_identifier,
    )
    .iter()
    .map(|root| (root.instance, root.checkpoint_txg))
    .collect();
    roots.sort_by_key(|(i, t)| (*t, *i));
    roots
}

fn slot_target(txgs: &[u64]) -> RootRingSlotTarget {
    let slots: Vec<_> = txgs
        .iter()
        .map(|txg| target_for_publish(CheckpointTxg(*txg), parameters().geometry.root_ring_slots_per_region))
        .collect();
    RootRingSlotTarget {
        named_slots: NamedRootRingSlots::naming(&slots),
        region_devices: parameters().region_devices,
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
    }
}

fn witness_of(image: &MemoryPool) -> Vec<(u32, u32, u64)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    rollback_witness_of_the_pool(image, &sc)
        .entries()
        .iter()
        .map(|e| (e.new_instance.0, e.rollback_target_instance.0, e.rollback_target_txg.0))
        .collect()
}

fn rows_of_newest(image: &MemoryPool) -> (String, usize, usize) {
    let sc = choose_system_configuration(image).expect("系统配置");
    let newest = choose_root(image, &sc).expect("有根");
    let table = instance_table_of_root(image, &newest).expect("实例表");
    let rendered = table
        .rows
        .iter()
        .map(|row| {
            format!(
                "({},{},{})",
                row.instance.0,
                row.selected_root_txg.0,
                if row.is_rollback { "rb" } else { "-" }
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let rollback_rows = table.rows.iter().filter(|row| row.is_rollback).count();
    (
        format!("newest=({},{}) rows=[{rendered}]", newest.instance.0, newest.checkpoint_txg.0),
        table.rows.len(),
        rollback_rows,
    )
}

fn full_writable_mount(image: &mut MemoryPool) -> Result<(), MountError> {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let result = mount_writable(&parameters(), &mut devices).map(|_| ());
    image.apply(&stream.retained_operations());
    result
}

fn describe(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { root, content } => format!(
            "FileRead root=({},{}) content_is_A={} content_len={}",
            root.0 .0,
            root.1 .0,
            content == &file_content(),
            content.len()
        ),
        other => format!("{other:?}"),
    }
}

/// Z21-A：回退到 mkfs 的第 0 代根 (0, 0)（目标实例 0 不写行，实例表里没有回退行），回退挂载崩在写行的根 FUA 落了、轮换没落；
/// 之后照常可写挂载 m 次（m = 0..=3 放开扫）；最后让实例 ≥ 2 的根全读不出（比被抛弃的 C（1, 5）新的全部根），恢复。
/// 对照：同一段历史回退挂载整次写完。
#[test]
fn z21_a_rollback_to_the_generation_zero_root_crashing_in_the_post_window_is_never_restored() {
    for m in 0..=3usize {
        for crash in [true, false] {
            let mut image = image_through(5);
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            mount_rollback(
                &parameters(),
                &mut devices,
                RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) },
                ShadowLedger::On,
            )
            .expect("回退到 (0, 0)");
            let ops = stream.retained_operations();
            let persisted = if crash { first_root_write(&ops) + 1 } else { ops.len() };
            image.apply(&ops[..persisted]);
            for _ in 0..m {
                full_writable_mount(&mut image).expect("可写挂载");
            }
            let (rows, _, _) = rows_of_newest(&image);
            let newer_than_c: Vec<u64> = roots_of(&image)
                .into_iter()
                .filter(|(i, t)| (t.0, i.0) > (5, 1))
                .map(|(_, t)| t.0)
                .collect();
            let reader = PoolReaderWithUnreadableRootRingSlots::new(&image, slot_target(&newer_than_c));
            let report = recover(&reader, JournalPolicy::Consult);
            println!(
                "Z21-A m={m} crash_between_row_root_and_rotation={crash} witness={:?} {rows} unreadable_root_txgs={:?} faults={} -> {}",
                witness_of(&image),
                newer_than_c,
                newer_than_c.len(),
                describe(&report.outcome)
            );
        }
    }
}

/// 连着回退 `rollbacks` 次，每次整次写完（不崩），目标每次取最新的根（空回退，用户定的那一步之一）；
/// `writable_between` 是每两次回退之间用户照常可写挂载几次（用户定的那一步，放开扫）。
fn healthy_rollback_series(tag: &str, rollbacks: usize, writable_between: usize) -> MemoryPool {
    let _ = tag;
    let mut image = image_through(5);
    for _ in 0..rollbacks {
        let sc = choose_system_configuration(&image).expect("系统配置");
        let newest = choose_root(&image, &sc).expect("有根");
        let stream = SharedStream::retaining_contents();
        let mut devices = crash_state_devices(&image, &[], &[], &stream);
        mount_rollback(
            &parameters(),
            &mut devices,
            RollbackTarget { instance: newest.instance, checkpoint_txg: newest.checkpoint_txg },
            ShadowLedger::On,
        )
        .expect("回退到最新的根");
        image.apply(&stream.retained_operations());
        for _ in 0..writable_between {
            full_writable_mount(&mut image).expect("可写挂载");
        }
    }
    image
}

/// 把最旧那条可读根的根环槽翻坏一个字节（介质坏、自证不过：D23 已定项 14「有槽持续读不出」那一形）。交回翻的是哪条根。
fn corrupt_the_oldest_root_slot(image: &mut MemoryPool) -> (u32, u64) {
    let (instance, txg) = roots_of(image)[0];
    let target = target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    image.flip_byte(
        device,
        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing),
        100,
    );
    (instance.0, txg.0)
}

/// Z21-B：一段健康的历史——连着 K 次回退（每次整次写完、根环全读得出），见证条目按删除规则删得干干净净；
/// 之后根环里一个槽坏了（翻一个字节），下一次普通可写挂载。
#[test]
fn z21_b_rollback_rows_resurrect_deleted_entries_when_one_ring_slot_goes_bad() {
    for writable_between in [0usize, 1] {
        for rollbacks in [22usize, 23, 24, 25, 30] {
            let mut image = healthy_rollback_series(
                &format!("r4-z21b-{writable_between}-{rollbacks}"),
                rollbacks,
                writable_between,
            );
            let sc = choose_system_configuration(&image).expect("系统配置");
            let all_readable = every_root_ring_slot_holds_a_root(&image, &sc).is_some();
            let witness_before = witness_of(&image);
            let (_, rows, rollback_rows) = rows_of_newest(&image);
            // 对照：坏槽之前，同一份镜像再普通可写挂载一次。
            let mut healthy = image.clone();
            let healthy_mount = full_writable_mount(&mut healthy);
            let corrupted = corrupt_the_oldest_root_slot(&mut image);
            let sc = choose_system_configuration(&image).expect("系统配置");
            let all_readable_after = every_root_ring_slot_holds_a_root(&image, &sc).is_some();
            let before = image.clone();
            let mut after = image.clone();
            let result = full_writable_mount(&mut after);
            println!(
                "Z21-B writable_between={writable_between} rollbacks={rollbacks} ring_all_readable={all_readable} witness_entries_on_disk={} rollback_rows_in_newest_table={rollback_rows} rows={rows} healthy_next_mount={:?} | corrupted_root=({},{}) ring_all_readable_after={all_readable_after} -> writable_mount={:?} disk_unchanged={}",
                witness_before.len(),
                healthy_mount.as_ref().map(|_| "Ok"),
                corrupted.0,
                corrupted.1,
                result.as_ref().map(|_| "Ok"),
                after == before
            );
        }
    }
}

/// 简单的线性同余，种子写死、逐段复现。
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

type Entry = (u32, u32, u64);

fn covers(by: &Entry, entry: &Entry) -> bool {
    by.0 >= entry.0 && (by.1, by.2) <= (entry.1, entry.2)
}

/// Z21-C：连着回退、每次随机地崩在「写行的根落了、轮换没落」那一格或整次写完，回退目标在根环里随机挑（不挑 mkfs 的第 0 代根，
/// 那一格归 Z21-A），两次回退之间随机插 0..2 次普通可写挂载；最后整次写完一次普通可写挂载。
/// 判：盘上每一条见证条目都是某一次回退挂载本来要写的那一条（没有补出错的 N）；本来要写而盘上没有的，要么被盘上一条罩住，
/// 要么照第一条删除规则删得掉（根环全读得出、环里没有实例落在 [r_old, N) 的根）；池级 checker 一条不红。
#[test]
fn z21_c_random_rollback_series_with_crashes_in_the_post_window_restore_exactly_the_intended_entries() {
    let series: u64 = std::env::var("R4_OPUS_Z21C_SERIES").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
    let base: u64 = std::env::var("R4_OPUS_Z21C_BASE").ok().and_then(|v| v.parse().ok()).unwrap_or(91_000);
    let mut bad = 0u64;
    let mut restored_by_a_later_mount = 0u64;
    let mut crashes_total = 0u64;
    let mut rollbacks_total = 0u64;
    for index in 0..series {
        let seed = base + index;
        let mut rng = Lcg(seed);
        let last_txg = 5 + rng.below(6);
        let mut image = image_through(last_txg);
        let steps = 1 + rng.below(6) as usize;
        let mut intended: Vec<Entry> = Vec::new();
        let mut log = Vec::new();
        for _ in 0..steps {
            let mut roots = roots_of(&image);
            roots.retain(|(i, _)| i.0 != 0);
            // 随机次序逐个试，第一个是候选的就回退到它（不是候选的在任何写之前被拒，丢掉那次的流）。
            let mut order: Vec<usize> = (0..roots.len()).collect();
            for k in (1..order.len()).rev() {
                let j = rng.below(k as u64 + 1) as usize;
                order.swap(k, j);
            }
            let crash = rng.below(2) == 0;
            let mut done = false;
            for position in order {
                let (instance, txg) = roots[position];
                let stream = SharedStream::retaining_contents();
                let mut devices = crash_state_devices(&image, &[], &[], &stream);
                match mount_rollback(
                    &parameters(),
                    &mut devices,
                    RollbackTarget { instance, checkpoint_txg: txg },
                    ShadowLedger::On,
                ) {
                    Ok(mounted) => {
                        let ops = stream.retained_operations();
                        let persisted = if crash { first_root_write(&ops) + 1 } else { ops.len() };
                        image.apply(&ops[..persisted]);
                        intended.push((mounted.output.instance.0, instance.0, txg.0));
                        rollbacks_total += 1;
                        if crash {
                            crashes_total += 1;
                        }
                        log.push(format!(
                            "rb#{}->({},{}){}",
                            mounted.output.instance.0,
                            instance.0,
                            txg.0,
                            if crash { "!crash" } else { "" }
                        ));
                        done = true;
                        break;
                    }
                    Err(MountError::RollbackTargetNotACandidate { .. }) => continue,
                    Err(other) => {
                        log.push(format!("rb-refused({:?})", format!("{other:?}").chars().take(80).collect::<String>()));
                        done = true;
                        break;
                    }
                }
            }
            if !done {
                log.push("no-candidate".to_string());
            }
            for _ in 0..rng.below(3) {
                match full_writable_mount(&mut image) {
                    Ok(()) => log.push("mount".to_string()),
                    Err(e) => log.push(format!("mount-refused({})", format!("{e:?}").chars().take(80).collect::<String>())),
                }
            }
        }
        let final_mount = full_writable_mount(&mut image);
        let witness = witness_of(&image);
        let sc = choose_system_configuration(&image).expect("系统配置");
        let ring = every_root_ring_slot_holds_a_root(&image, &sc);
        let wrong: Vec<&Entry> = witness.iter().filter(|e| !intended.contains(e)).collect();
        let missing: Vec<&Entry> = intended
            .iter()
            .filter(|e| !witness.contains(e))
            .filter(|e| !witness.iter().any(|w| covers(w, e)))
            .filter(|e| match &ring {
                None => true,
                Some(roots) => roots.iter().any(|root| e.1 <= root.instance.0 && root.instance.0 < e.0),
            })
            .collect();
        let violated: Vec<&'static str> = singlefs_checker::walk::check_pool_image(&image)
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                singlefs_checker::image::InvariantVerdict::Violated(_) => Some(*invariant),
                _ => None,
            })
            .collect();
        let ok = wrong.is_empty() && missing.is_empty() && violated.is_empty() && final_mount.is_ok();
        if !ok {
            bad += 1;
        }
        restored_by_a_later_mount += intended.iter().filter(|e| witness.contains(e)).count() as u64;
        println!(
            "Z21-C seed={seed} last_txg={last_txg} steps=[{}] final_mount={:?} intended={intended:?} witness={witness:?} wrong={wrong:?} missing_and_not_deletable={missing:?} checker_red={violated:?} ok={ok}",
            log.join(" "),
            final_mount.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}").chars().take(80).collect::<String>()),
        );
    }
    println!(
        "Z21-C summary series={series} base={base} rollbacks={rollbacks_total} crashed_in_post_window={crashes_total} intended_entries_present_at_end={restored_by_a_later_mount} bad_series={bad}"
    );
}

/// Z21-B2：Z21-B 那一格（24 次健康回退、之后一个根槽坏）之后，管理员能做的每一样：再试一次普通可写挂载、回退到最新的根、
/// 回退到环里每一条别的可读根；只读恢复照常。坏槽复原（字节翻回去）之后同一次挂载做成——拒的是那一个坏槽。
#[test]
fn z21_b2_after_the_refusal_every_writable_path_is_refused_while_the_slot_stays_bad() {
    let mut image = healthy_rollback_series("r4-z21b2", 24, 0);
    let pristine = image.clone();
    let corrupted = corrupt_the_oldest_root_slot(&mut image);
    let mut again = image.clone();
    println!("Z21-B2 corrupted_root=({},{})", corrupted.0, corrupted.1);
    for attempt in 1..=2 {
        let result = full_writable_mount(&mut again);
        println!("Z21-B2 writable_attempt={attempt} -> {:?} disk_unchanged={}", result.as_ref().map(|_| "Ok"), again == image);
    }
    let mut refused = 0;
    let mut other = Vec::new();
    for (instance, txg) in roots_of(&image) {
        let stream = SharedStream::retaining_contents();
        let mut devices = crash_state_devices(&image, &[], &[], &stream);
        match mount_rollback(
            &parameters(),
            &mut devices,
            RollbackTarget { instance, checkpoint_txg: txg },
            ShadowLedger::On,
        ) {
            Err(MountError::Recovery(failure)) if format!("{failure:?}").starts_with("RollbackWitnessTableFull") => refused += 1,
            result => other.push(format!("({},{})->{}", instance.0, txg.0, match result {
                Ok(_) => "Ok".to_string(),
                Err(e) => format!("{e:?}").chars().take(60).collect(),
            })),
        }
    }
    println!("Z21-B2 rollback_targets_refused_with_witness_full={refused} others={other:?}");
    let report = recover(&image, JournalPolicy::Consult);
    println!("Z21-B2 read_only_recovery -> {}", describe(&report.outcome));
    let mut restored = pristine.clone();
    let result = full_writable_mount(&mut restored);
    println!("Z21-B2 slot_restored_same_pool -> writable_mount={:?} witness_after={}", result.as_ref().map(|_| "Ok"), witness_of(&restored).len());
}

// ===================== m2-safety-r1 云端攻方（Opus）加的：S2 / S3 候选的判别子 =====================
// 候选由副本补丁的环境变量选：S_OPUS_S2=narrow（只补 N = 所选根实例）| clearflag（删除规则 ① 判得出可删时清回退位）；
// S_OPUS_S3=infer（按行形推 (N, 0, 0)）| row-zero（回退到实例 0 的根也写回退行）| refuse（不许回退到实例 0 的根）| pre（r_old = 0 时取号就写见证）。

fn slot_generation_newest(image: &MemoryPool, device: singlefs_core::address::DeviceIdentity) -> Option<u64> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut best: Option<(u64, u64)> = None;
    let mut valid = 0;
    for offset in [0, spacing] {
        let Some(bytes) = singlefs_core::recovery::PoolReader::read(
            image,
            device,
            singlefs_core::address::DeviceOffsetInBytes(offset),
            slot_bytes,
        ) else {
            continue;
        };
        if let Ok(sc) = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
            valid += 1;
            let generation = sc.quantities.slot_generation;
            if best.map_or(true, |(g, _)| generation > g) {
                best = Some((generation, offset));
            }
        }
    }
    // 只在两槽都自证过时交出较新的那一槽：翻坏它之后旧槽还在（不把一块盘的系统配置整个弄没）。
    if valid == 2 { best.map(|(_, offset)| offset) } else { None }
}

/// 两块盘各自最新的那个系统配置槽各翻坏一个字节（持续坏）：见证读退回各盘旧槽（两个故障）。两块盘都各有两槽自证过才翻，否则不动。
fn degrade_the_newest_system_configuration_slots(image: &mut MemoryPool) -> bool {
    let devices = [singlefs_core::address::DeviceIdentity(0), singlefs_core::address::DeviceIdentity(1)];
    let offsets: Vec<Option<u64>> = devices.iter().map(|device| slot_generation_newest(image, *device)).collect();
    if offsets.iter().any(Option::is_none) {
        return false;
    }
    for (device, offset) in devices.iter().zip(offsets) {
        image.flip_byte(*device, singlefs_core::address::DeviceOffsetInBytes(offset.expect("判过")), 100);
    }
    true
}

fn writable_mount_choosing(image: &mut MemoryPool) -> Result<(u32, u64), String> {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(image, &[], &[], &stream);
    let result = mount_writable(&parameters(), &mut devices)
        .map(|m| (m.output.chosen_root.instance.0, m.output.chosen_root.checkpoint_txg.0))
        .map_err(|e| format!("{e:?}").chars().take(90).collect::<String>());
    image.apply(&stream.retained_operations());
    result
}

type Effective = (u32, u32, u64);

fn abandoned_by(effective: &[Effective], root: (u32, u64)) -> bool {
    effective
        .iter()
        .any(|(n, r, t)| (*r, *t) < (root.0, root.1) && root.0 < *n)
}

/// S2 / S3 的判别子：随机历史（回退、可写挂载、在录制流里任一步崩、最旧根槽坏、两盘最新系统配置槽坏），
/// 真值 = 生效了的回退（写行那次的根落了）按条款的抛弃判法算出来的被抛弃根集合。最后整次写完一次可写挂载，然后：
/// ① 回退撤销：把环里每一条没被抛弃的根都弄成读不出，恢复择到被抛弃的根就是中（C332 那一族），记要几个根故障；
/// ② 最后那次可写挂载择到的根是不是真值里最新的未被抛弃的可读根（见证多抛弃了会择到更旧的）；
/// ③ 池级 checker；④ 最后那次可写挂载做没做成（写满之类的拒）。
#[test]
fn s23_random_series_oracle() {
    let series: u64 = std::env::var("S_OPUS_SERIES").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
    let base: u64 = std::env::var("S_OPUS_BASE").ok().and_then(|v| v.parse().ok()).unwrap_or(510_000);
    let zero_share: u64 = std::env::var("S_OPUS_ZERO_SHARE").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let max_steps: u64 = std::env::var("S_OPUS_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let degrade_share: u64 = std::env::var("S_OPUS_DEGRADE_SHARE").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    let mut hits = 0u64;
    let mut wrong_choice = 0u64;
    let mut checker_red = 0u64;
    let mut final_refused = 0u64;
    let mut rollbacks = 0u64;
    let mut effective_rollbacks = 0u64;
    for index in 0..series {
        let seed = base + index;
        let mut rng = Lcg(seed);
        let last_txg = 5 + rng.below(6);
        let mut image = image_through(last_txg);
        let mut effective: Vec<Effective> = Vec::new();
        let mut log: Vec<String> = Vec::new();
        let mut faults_before_the_end = 0u64;
        let steps = 1 + rng.below(max_steps);
        for _ in 0..steps {
            let action = rng.below(100);
            if action < 55 {
                let mut roots = roots_of(&image);
                let allow_zero = rng.below(zero_share.max(1)) == 0;
                if !allow_zero {
                    roots.retain(|(i, _)| i.0 != 0);
                }
                if roots.is_empty() {
                    continue;
                }
                let (instance, txg) = roots[rng.below(roots.len() as u64) as usize];
                let crash = rng.below(2) == 0;
                let stream = SharedStream::retaining_contents();
                let mut devices = crash_state_devices(&image, &[], &[], &stream);
                match mount_rollback(
                    &parameters(),
                    &mut devices,
                    RollbackTarget { instance, checkpoint_txg: txg },
                    ShadowLedger::On,
                ) {
                    Ok(mounted) => {
                        rollbacks += 1;
                        let ops = stream.retained_operations();
                        let k = if crash { rng.below(ops.len() as u64 + 1) as usize } else { ops.len() };
                        let root_landed = ops[..k]
                            .iter()
                            .any(|op| op.operation.kind == RecordedOperationKind::WriteForceUnitAccess);
                        image.apply(&ops[..k]);
                        if root_landed {
                            effective.push((mounted.output.instance.0, instance.0, txg.0));
                            effective_rollbacks += 1;
                        }
                        log.push(format!(
                            "rb#{}->({},{}){}",
                            mounted.output.instance.0,
                            instance.0,
                            txg.0,
                            if crash { format!("!crash@{k}/{}{}", ops.len(), if root_landed { "+root" } else { "" }) } else { String::new() }
                        ));
                    }
                    Err(MountError::RollbackTargetNotACandidate { .. }) => log.push(format!("rb-notcand({},{})", instance.0, txg.0)),
                    Err(other) => log.push(format!("rb-refused({})", format!("{other:?}").chars().take(60).collect::<String>())),
                }
            } else if action < 85 {
                let crash = rng.below(3) == 0;
                let stream = SharedStream::retaining_contents();
                let mut devices = crash_state_devices(&image, &[], &[], &stream);
                match mount_writable(&parameters(), &mut devices) {
                    Ok(_) => {
                        let ops = stream.retained_operations();
                        let k = if crash { rng.below(ops.len() as u64 + 1) as usize } else { ops.len() };
                        image.apply(&ops[..k]);
                        log.push(if crash { format!("mount!crash@{k}/{}", ops.len()) } else { "mount".to_string() });
                    }
                    Err(e) => log.push(format!("mount-refused({})", format!("{e:?}").chars().take(60).collect::<String>())),
                }
            } else if action < 100 - degrade_share {
                let (i, t) = corrupt_the_oldest_root_slot(&mut image);
                faults_before_the_end += 1;
                log.push(format!("badroot({i},{t})"));
            } else {
                if degrade_the_newest_system_configuration_slots(&mut image) {
                    faults_before_the_end += 2;
                    log.push("badsc".to_string());
                }
            }
        }
        let expected_choice = roots_of(&image)
            .iter()
            .map(|(i, t)| (i.0, t.0))
            .filter(|root| !abandoned_by(&effective, *root))
            .max_by_key(|(i, t)| (*t, *i));
        let final_mount = writable_mount_choosing(&mut image);
        if final_mount.is_err() {
            final_refused += 1;
        }
        let readable = roots_of(&image);
        let live: Vec<(u32, u64)> = readable
            .iter()
            .map(|(i, t)| (i.0, t.0))
            .filter(|root| !abandoned_by(&effective, *root))
            .collect();
        let choice_ok = match (&final_mount, expected_choice) {
            (Ok(chosen), Some(expected)) => *chosen == expected,
            (Err(_), _) => true,
            (Ok(_), None) => false,
        };
        if !choice_ok {
            wrong_choice += 1;
        }
        let live_txgs: Vec<u64> = live.iter().map(|(_, t)| *t).collect();
        let reader = PoolReaderWithUnreadableRootRingSlots::new(&image, slot_target(&live_txgs));
        let report = recover(&reader, JournalPolicy::Consult);
        let landed = match &report.outcome {
            RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
            RecoveryOutcome::Failed { root, .. } => root.map(|(i, t)| (i.0, t.0)),
        };
        let hit = landed.is_some_and(|root| abandoned_by(&effective, root));
        if hit {
            hits += 1;
        }
        let violated: Vec<&'static str> = singlefs_checker::walk::check_pool_image(&image)
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                singlefs_checker::image::InvariantVerdict::Violated(_) => Some(*invariant),
                _ => None,
            })
            .collect();
        if !violated.is_empty() {
            checker_red += 1;
        }
        println!(
            "S23 seed={seed} steps=[{}] effective={effective:?} witness={:?} final_mount={final_mount:?} expected={expected_choice:?} choice_ok={choice_ok} C332_hit={hit} landed={landed:?} live_roots_made_unreadable={} faults_before={faults_before_the_end} checker_red={violated:?}",
            log.join(" "),
            witness_of(&image),
            live_txgs.len(),
        );
    }
    println!(
        "S23 summary series={series} base={base} rollbacks={rollbacks} effective={effective_rollbacks} C332_hits={hits} wrong_final_choice={wrong_choice} checker_red={checker_red} final_mount_refused={final_refused}"
    );
}

/// S2 收窄补法的定点反例候选：回退 N 崩在 post 窗口 → 可写挂载 M（择 N 的根、补上 N 那一条、取号只写每盘一槽）在录制流第 k 步崩
/// → 两盘最新系统配置槽都坏（M 取号写的那一槽，见证读退回旧槽）→ 最后一次可写挂载 → 撤销探针（把没被抛弃的根全弄成读不出）。
/// k 扫 M 的整条录制流；degrade 开 / 关各一遍。
#[test]
fn s2_narrow_hole_targeted() {
    for degrade in [false, true] {
        for k in 0..=200usize {
            let mut image = image_through(5);
            // 回退 N 到 (1, 3)，崩在写行根落了、轮换没落。
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            let n = mount_rollback(
                &parameters(),
                &mut devices,
                RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
                ShadowLedger::On,
            )
            .expect("回退");
            let ops = stream.retained_operations();
            image.apply(&ops[..first_root_write(&ops) + 1]);
            let effective: Vec<Effective> = vec![(n.output.instance.0, 1, 3)];
            // 可写挂载 M，录制流第 k 步崩。
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            let m = mount_writable(&parameters(), &mut devices).expect("可写挂载 M");
            let ops = stream.retained_operations();
            if k > ops.len() {
                break;
            }
            image.apply(&ops[..k]);
            let degraded = degrade && degrade_the_newest_system_configuration_slots(&mut image);
            let witness_before_final = witness_of(&image);
            let final_mount = writable_mount_choosing(&mut image);
            let live: Vec<u64> = roots_of(&image)
                .iter()
                .map(|(i, t)| (i.0, t.0))
                .filter(|root| !abandoned_by(&effective, *root))
                .map(|(_, t)| t)
                .collect();
            let reader = PoolReaderWithUnreadableRootRingSlots::new(&image, slot_target(&live));
            let report = recover(&reader, JournalPolicy::Consult);
            let landed = match &report.outcome {
                RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
                RecoveryOutcome::Failed { root, .. } => root.map(|(i, t)| (i.0, t.0)),
            };
            let hit = landed.is_some_and(|root| abandoned_by(&effective, root));
            println!(
                "S2HOLE degrade={degrade} degraded={degraded} N={} M={} k={k}/{} witness_before_final={witness_before_final:?} final_mount={final_mount:?} witness_after={:?} live_unreadable={} landed={landed:?} C332_hit={hit}",
                n.output.instance.0,
                m.output.instance.0,
                ops.len(),
                witness_of(&image),
                live.len()
            );
        }
    }
}

/// S3-乙 的新失败面：不是回退、而是恢复自己落到 mkfs 的第 0 代根（实例 1 的根暂时全读不出），之后那些根又读得出。
/// 行形与「回退到 (0, 0)」相同（(1, 0, 0) 打头），按行形推会给它补一条 (N, 0, 0)。之后再让 N 起的根全读不出，恢复落在哪。
#[test]
fn s3_infer_on_a_recovery_that_fell_to_generation_zero() {
    let mut image = image_through(5);
    let instance_one: Vec<(u32, u64)> = roots_of(&image).iter().filter(|(i, _)| i.0 == 1).map(|(i, t)| (i.0, t.0)).collect();
    let flip = |image: &mut MemoryPool, txg: u64| {
        let target = target_for_publish(CheckpointTxg(txg), parameters().geometry.root_ring_slots_per_region);
        let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
        image.flip_byte(device, slot_offset(target, parameters().geometry.fixed_structure_slot_spacing), 100);
    };
    for (_, txg) in &instance_one {
        flip(&mut image, *txg);
    }
    let first = writable_mount_choosing(&mut image);
    for (_, txg) in &instance_one {
        flip(&mut image, *txg); // 字节翻回去：那几条根又读得出（瞬时读不出那一形）。
    }
    let second = writable_mount_choosing(&mut image);
    let (rows, _, _) = rows_of_newest(&image);
    let later: Vec<u64> = roots_of(&image).iter().filter(|(i, _)| i.0 >= 2).map(|(_, t)| t.0).collect();
    let reader = PoolReaderWithUnreadableRootRingSlots::new(&image, slot_target(&later));
    let report = recover(&reader, JournalPolicy::Consult);
    println!(
        "S3INFER instance_one_roots={instance_one:?} first_mount_chose={first:?} second_mount_chose={second:?} {rows} witness={:?} roots_from_instance_2_unreadable={} -> {}",
        witness_of(&image),
        later.len(),
        describe(&report.outcome)
    );
}
