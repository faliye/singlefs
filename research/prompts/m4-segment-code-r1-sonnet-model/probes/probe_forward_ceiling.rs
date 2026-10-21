//! 正推腿探针（T11 的实现一侧）：与 `append_to_model_tests.rs` 里的模型探针同一段历史，在真实现上走一遍，读真实现的抬 F 上限。
//! 四次覆盖写之后 建快照 S1 → 覆盖写 → 删 S1（意图加最后一批）→ 建快照 S2。S1 抄的是 S1 那一刻活头的根、S2 抄的是之后活头的根，
//! 两条根（覆盖写那一版与建 S2 那一版）活头相同、各带一个快照，只差快照条目抄的根指针。
//! 真实现按根指针字节去重（`mount::newest_first_distinct_states`），非空根从新到旧：建 S2、删 S1 最后一批、覆盖写、建 S1、……，第 4 新的是建 S1 那一版。
mod common;

use common::{build_pool, parameters};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::mount::{mount_writable, ring_slots_known_to_hold_a_root_by, rollback_floor_ceiling};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
use singlefs_core::transaction::FirstFile;

#[test]
fn probe_forward_ceiling_of_the_real_implementation_counts_roots_differing_only_in_the_roots_their_snapshots_copied_as_two_states() {
    let mut pool = build_pool("probe-forward-ceiling-snapshots");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for content_byte in 6_u8..=9 {
        session
            .publish_user_change(
                &mut devices,
                UserChange::Overwrite(FirstFile {
                    content: &[content_byte],
                    write_time_seconds: 1_788_000_000 + u64::from(content_byte),
                }),
            )
            .expect("覆盖写");
    }
    let txg_now = |session: &MountedSession| session.current.root().checkpoint_txg;
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建 S1");
    let txg_of_the_first_snapshot_version = txg_now(&session);
    let first_snapshot = session
        .current
        .version_of_trees()
        .expect("有树")
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("S1 的码 14")
        .tree;
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &[11],
                write_time_seconds: 1_788_000_100,
            }),
        )
        .expect("覆盖写");
    session
        .publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot: first_snapshot })
        .expect("删 S1");
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建 S2");
    let system_configuration = choose_system_configuration(&devices).expect("系统配置读得出");
    let table = instance_table_chain_of_root(&devices, session.current.root())
        .expect("实例表读得出")
        .records;
    let ceiling = rollback_floor_ceiling(
        &devices,
        &system_configuration,
        session.current.root().rollback_floor,
        &table,
        &ring_slots_known_to_hold_a_root_by(&session.allocator),
    )
    .expect("上限算得出");
    println!(
        "name=probe_forward_real_ceiling ceiling={:?} txg_of_the_version_that_created_the_first_snapshot={:?} txg_of_the_newest={:?}",
        ceiling,
        txg_of_the_first_snapshot_version,
        txg_now(&session)
    );
    assert_eq!(
        ceiling, txg_of_the_first_snapshot_version,
        "真实现：第 4 新的非空根是建 S1 那一版"
    );
    let _ = CheckpointTxg(0);
    pool.devices = Some(devices);
}
