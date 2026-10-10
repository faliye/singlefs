//! s2 攻方原型 K5-2 / K5-3：贴边建满之后 unlink，量删除类发布之后与稳态（抬 F 到准入上限、defer 放回）的可用，
//! 再看之后的释放（unlink 第一个文件）做不做得成、写入类发布被放过之后写不写得下。只在攻方副本上跑。
//! 输出一行一格：`name=edge ...`。参数（环境变量）：S2_NEW（建几个空 inode）、S2_PRE（先 unlink 哪个号，0 = 不）、
//! S2_VICTIM（贴边之后 unlink 哪个号）、S2_DU_FROM / S2_DU_TO（第一个文件顺序写的数据单元数扫描区间，步长 1）。
mod common_admission_pool;

use common_admission_pool::{
    checker_violations_on, device_bytes_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, PoolUnderTest,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::mount::{raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::PoolVersion;
use singlefs_core::write_request_split::data_unit_count_of_a_sequential_write;
use singlefs_format::{DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};

use singlefs_harness::memory_pool::SparseBlockDevice;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

type Pool = PoolUnderTest<SparseBlockDevice>;

/// 每盘（取第 0 块）：可用（槽，小数）、容量、已分配、隔离、挂载期、环窗口（槽）；全池摊到每盘：标签 6、ckpt 保留池（槽，小数）；
/// 物理空闲 = 容量 − 已分配 − 隔离 − 不可回收（槽）。
fn reading(pool: &Pool) -> String {
    let session = pool.session();
    let version = session.current.file_version();
    let reading = admission_reading_before_a_publish(&session.allocator, version);
    let available = reading.available_on_each_device()[0].1 .0;
    let terms = &reading.per_device()[0];
    let pw = reading.pool_wide();
    let replicas = 2.0_f64;
    let slot = SLOT_BYTES as f64;
    let free = (terms.capacity.0 - terms.allocated.0 - terms.isolated_for_abandoned_roots_and_unapplied_journal_records.0 - terms.unreclaimable.0) / SLOT_BYTES;
    format!(
        "avail={:.3} cap={} alloc={} iso={} mount={} window={} label6={:.3} ckpt={:.3} free={} txg={} F={}",
        (available as f64) / slot,
        terms.capacity.0 / SLOT_BYTES,
        terms.allocated.0 / SLOT_BYTES,
        terms.isolated_for_abandoned_roots_and_unapplied_journal_records.0 / SLOT_BYTES,
        terms.mount_time_commitment.0 / SLOT_BYTES,
        terms.ring_window.0 / SLOT_BYTES,
        (pw.committed_reservation.0 as f64) / replicas / slot,
        (pw.checkpoint_reserve_pool.0 as f64) / replicas / slot,
        free,
        session.current.root().checkpoint_txg.0,
        session.current.root().rollback_floor.0,
    )
}

fn steady(pool: &mut Pool) -> String {
    let mut session = pool.session.take().expect("会话");
    let mut rounds = 0;
    let mut reclaimed = 0;
    let mut outcome = String::new();
    // 迭代上界：至多 6 次抬 F（每次抬到准入上限）。
    for _ in 0..6 {
        let PoolVersion::WithFile(current) = &mut session.current else {
            outcome = "no-file-version".into();
            break;
        };
        match raise_rollback_floor_to_the_admission_ceiling(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            session.shadow_ledger,
        ) {
            Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
                rounds += 1;
                reclaimed += raised.reclaimed.len();
            }
            Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }) => break,
            Err(e) => {
                outcome = format!("raise-err:{e:?}").chars().take(160).collect();
                break;
            }
        }
    }
    pool.session = Some(session);
    format!("raises={rounds} reclaimed={reclaimed} {outcome}")
}

/// 静置时从哪个号起往下 unlink（S2_QUIESCE_FROM，默认 S2_NEW + 1，即最大的那个号）。
fn quiesce_from() -> u64 {
    env_u64("S2_QUIESCE_FROM", env_u64("S2_NEW", 233) + 1)
}

/// 静态稳态：正常卸载（F 抬到现行 txg）再可写挂载（准入判着）。交回挂载结局。
fn unmount_and_mount(pool: &mut Pool) -> String {
    let mut session = pool.session.take().expect("会话");
    let unmounted = singlefs_core::mount::unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    );
    if let Err(e) = unmounted {
        pool.session = Some(session);
        return format!("UNMOUNT-ERR({})", format!("{e:?}").chars().take(160).collect::<String>());
    }
    drop(session);
    match pool.crash_and_mount_writable() {
        Ok(_) => "mounted".to_string(),
        Err(e) => format!("MOUNT-REFUSED({})", format!("{e:?}").chars().take(200).collect::<String>()),
    }
}

fn payload() -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
}

fn unlink(pool: &mut Pool, inode: u64) -> String {
    pool.write_time_seconds += 1;
    let t = pool.write_time_seconds;
    match pool.publish(
        UserChange::Unlink {
            inode: InodeNumber(inode),
            write_time_seconds: t,
        },
        None,
    ) {
        Ok(p) => format!("ok(versions={},raises={})", p.versions_published.len(), p.floor_raises.len()),
        Err(e) => format!("ERR({})", format!("{e:?}").chars().take(200).collect::<String>()),
    }
}

#[test]
fn edge_then_unlink_sweep() {
    let new_inodes = env_u64("S2_NEW", 233);
    let pre = env_u64("S2_PRE", 0);
    let victim = env_u64("S2_VICTIM", 2);
    let du_from = env_u64("S2_DU_FROM", 1);
    let du_to = env_u64("S2_DU_TO", 140);
    // 迭代上界：du_to − du_from + 1 格。
    for du in du_from..=du_to {
        let mut pool: Pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
            parameters_of_the_384_slot_small_devices(),
            device_bytes_of_the_384_slot_small_devices(),
            plain_device,
        );
        pool.write_time_seconds += 1;
        let t = pool.write_time_seconds;
        pool.publish(UserChange::NewInodes { count: new_inodes, write_time_seconds: t }, None)
            .expect("建空 inode");
        let pre_outcome = if pre == 0 { "none".to_string() } else { unlink(&mut pool, pre) };
        let length = du * payload();
        assert_eq!(data_unit_count_of_a_sequential_write(length), du);
        let write = pool.sequential_write(usize::try_from(length).unwrap());
        if let Err(e) = write {
            println!(
                "name=edge new={new_inodes} pre={pre} victim={victim} du={du} write=refused({}) {}",
                format!("{e:?}").chars().take(60).collect::<String>(),
                reading(&pool)
            );
            break;
        }
        let after_write = reading(&pool);
        let leaves_before = pool.current_file_version().inode_leaf_containers.len();
        let unlink_outcome = unlink(&mut pool, victim);
        let after_unlink = reading(&pool);
        let retired = pool.current_file_version().retirement_containers.iter().map(|c| c.contents.records.len()).sum::<usize>();
        let leaves_after = pool.current_file_version().inode_leaf_containers.len();
        // 静置：再 unlink 几个不引起叶死亡的号（删除类发布、非空版本，推着抬 F 的上限往前走），之后抬 F 到准入上限。
        let mut quiesce = Vec::new();
        let mut steady_outcome = String::new();
        let rounds = env_u64("S2_ROUNDS", 8);
        let mut next_to_unlink = quiesce_from();
        // 迭代上界：S2_ROUNDS 轮，每轮 unlink 两个号（从 S2_QUIESCE_FROM 往下数）再抬 F 到准入上限。
        for _ in 0..rounds {
            for _ in 0..2 {
                quiesce.push(unlink(&mut pool, next_to_unlink));
                next_to_unlink -= 1;
            }
            steady_outcome = steady(&mut pool);
        }
        let quiesce: Vec<String> = quiesce.into_iter().filter(|o| !o.starts_with("ok(")).collect();
        let remount = unmount_and_mount(&mut pool);
        if pool.session.is_none() {
            println!("name=edge new={new_inodes} pre={pre} victim={victim} du={du} | write: {after_write} | unlink={unlink_outcome}: {after_unlink} | remount={remount}");
            continue;
        }
        let after_steady = reading(&pool);
        let retired_after_quiesce = pool.current_file_version().retirement_containers.iter().map(|c| c.contents.records.len()).sum::<usize>();
        // 之后的写入类发布：建一个 inode（准入判着）；放过就要写得下。
        let create = match pool.create_one_inode() {
            Ok(_) => "ok".to_string(),
            Err(e) => format!("refused({})", format!("{e:?}").chars().take(40).collect::<String>()),
        };
        // 之后的释放：unlink 第一个文件（删除类发布不判式子、只判落点）：做不成就是释放被拒。
        let release = unlink(&mut pool, 1);
        let after_release = reading(&pool);
        let violations = checker_violations_on(&pool.image());
        println!(
            "name=edge new={new_inodes} pre={pre}:{pre_outcome} victim={victim} du={du} | write: {after_write} | unlink={unlink_outcome} leaves {leaves_before}->{leaves_after} retired={retired}: {after_unlink} | quiesce={quiesce:?} {steady_outcome} remount={remount} retired={retired_after_quiesce}: {after_steady} | create={create} | release={release}: {after_release} | checker_red={}",
            violations.len()
        );
        for v in violations.iter().take(3) {
            println!("name=edge-red du={du} {v}");
        }
    }
}
